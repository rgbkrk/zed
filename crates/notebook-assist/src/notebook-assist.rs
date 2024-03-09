use std::{any::Any, path::Path, sync::Arc};

use collections::{HashMap, HashSet};
use editor::{
    display_map::{BlockDisposition, BlockId, BlockProperties, BlockStyle},
    Editor,
};
use fs::Fs;
use futures::SinkExt as _;
use gpui::{actions, white, AppContext, EventEmitter, FocusHandle, FocusableView, Model, View};
use language::{language_settings, Anchor, Buffer, BufferId, LanguageRegistry, ToOffset};
use message::Message;
use serde_json::Value;
use smol::stream::StreamExt;
use ui::{
    div, h_flex, v_flex, Context as _, InteractiveElement, IntoElement, ParentElement as _,
    Styled as _, ViewContext, VisualContext,
};
use util::{http::HttpClient, ResultExt};
use workspace::{item::Item, Workspace};

mod iopub_content_messages;
mod message;
mod shell_content_messages;

actions!(notebook, [Deploy]);

pub fn init(
    fs: Arc<dyn Fs>,
    http: Arc<dyn HttpClient>,
    language: Arc<LanguageRegistry>,
    cx: &mut AppContext,
) {
    cx.observe_new_views({
        move |workspace: &mut Workspace, _cx: &mut ViewContext<Workspace>| {
            workspace.register_action({
                let http = http.clone();
                let fs = fs.clone();
                let language = language.clone();
                move |workspace, _: &Deploy, cx| {
                    let item = cx.new_view({
                        let http = http.clone();
                        let fs = fs.clone();
                        let language = language.clone();
                        move |cx| Notebook::new(fs, http, language, cx)
                    });
                    workspace.add_item_to_active_pane(Box::new(item), cx);
                }
            });
        }
    })
    .detach();
}

// when opening an .ipynb file, create a notebook item instead of an editor
//
//
// 1. We have the relevant assistant code sorta cordoned off below here
// 2. We discovered that the API we need to change is `Project::open_path`
//  a. This needs to change by returning a new model type, that represents the notebook
//  b. Then this notebook needs to register itself as 'project item builder', for that type
//  c. Because of the dependency graph, we'll need two crates for this, one for the UI and one
//     for the model returned by the project.
//  d. This could be as simple as a newtype wrapper around a normal buffer, that is defined in the
//     project crate.

type CellId = String;

enum Output {
    DisplayData {
        data: HashMap<String, Value>,
    },
    Stream {
        name: String,
        text: String,
    },
    ExecuteResult {
        data: HashMap<String, Value>,
    },
    Error {
        ename: String,
        evalue: String,
        traceback: Vec<String>,
    },
}

struct CodeCell {
    id: CellId,
    code_anchor: language::Anchor,
    block_anchor: language::Anchor,
    output: Option<HashMap<String, Value>>,
    // execution_count: Option<u64>
}

pub struct Notebook {
    focus_handle: FocusHandle,
    buffer: Model<Buffer>,
    editor: View<Editor>,
    cells: Vec<CodeCell>,
    blocks: HashSet<BlockId>,
}

impl Notebook {
    pub fn new(
        fs: Arc<dyn Fs>,
        _http: Arc<dyn HttpClient>,
        language: Arc<LanguageRegistry>,
        cx: &mut ViewContext<Self>,
    ) -> Self {
        let (mut tx, mut rx) = futures::channel::mpsc::unbounded();

        cx.background_executor()
            .spawn(async move {
                // TODO: Get this from a socket or somethign
                let response = fs
                    .load(Path::new(
                        "crates/notebook-assist/src/jupyter_messages_fixture.json",
                    ))
                    .await
                    .log_err();

                if let Some(response) = response {
                    // message.header.msg_type // descriminator

                    let messages = serde_json::from_str::<Vec<Message>>(&response).unwrap();
                    for message in messages {
                        tx.send(message).await.ok();
                    }
                }
            })
            .detach();

        cx.spawn(|this, mut cx| async move {
            while let Some(msg) = rx.next().await {
                this.update(&mut cx, |this, cx| {
                    this.handle_message(msg, cx);
                })
                .ok();
            }
        })
        .detach();

        // We need a buffer for this somewhere

        let markdown = language.language_for_name("Python");
        let buffer = cx.new_model(|cx| {
            let mut buffer = Buffer::new(0, BufferId::new(cx.entity_id().as_u64()).unwrap(), "");
            buffer.set_language_registry(language);
            cx.spawn(|buffer, mut cx| async move {
                let markdown = markdown.await?;
                buffer.update(&mut cx, |buffer: &mut Buffer, cx| {
                    buffer.set_language(Some(markdown), cx)
                })?;
                anyhow::Ok(())
            })
            .detach_and_log_err(cx);
            buffer
        });

        let editor = cx.new_view(|cx| {
            let mut editor = Editor::for_buffer(buffer.clone(), None, cx);
            editor.set_soft_wrap_mode(language_settings::SoftWrap::EditorWidth, cx);
            editor.set_show_gutter(false, cx);
            editor.set_show_wrap_guides(false, cx);
            editor
        });

        Self {
            focus_handle: cx.focus_handle(),
            editor,
            buffer,
            cells: vec![],
            blocks: HashSet::default(),
        }
    }
}

impl Notebook {
    fn push_code_cell_to_end(
        &mut self,
        cell_code: String,
        cell_id: String,
        cx: &mut ViewContext<Self>,
    ) {
        // 1. We need to find the location of the last cell in our list
        let code_anchor = if let Some(last_cell) = self.cells.last() {
            // If we're not the first location, append a new line and grab the anchor after it:
            let last_anchor = self.buffer.read(cx).anchor_after(last_cell.block_anchor);
            let start_anchor = self.buffer.update(cx, |buffer, cx| {
                buffer.edit([(last_anchor..last_anchor, "\n")], None, cx);
                buffer.anchor_before(last_anchor.to_offset(&buffer) + 1)
            });

            start_anchor
        } else {
            // If we're the first cell, just use the start of the buffer
            Anchor::MIN
        };

        // Then we need to jump to a known location in the buffer, and start putting the blocks in there
        let block_anchor = self.buffer.update(cx, |buffer, cx| {
            buffer.edit([(code_anchor..code_anchor, "\n\n\n")], None, cx);
            buffer.anchor_before(code_anchor.to_offset(&buffer) + 2)
        });

        // The code in a cell is stored at:
        //  - buffer.bytes_in_range(code_anchor..block_anchor)
        self.cells.push(CodeCell {
            id: cell_id,
            code_anchor,
            block_anchor,
            output: None,
        });

        // TODO: Remove this when we're not working with dummy input
        self.buffer.update(cx, |buffer, cx| {
            buffer.edit([(code_anchor..code_anchor, cell_code)], None, cx);
        })

        // TODO: Build and send in the block
    }

    fn handle_message(&mut self, message: Message, cx: &mut ViewContext<Self>) {
        match message {
            // msg.parent_header.msg_id

            // User sent code to run. This message echoes back from the IOPUB channel.
            Message::ExecuteInput(execute_input) => self.push_code_cell_to_end(
                execute_input.content.code,
                // HACK: We're treating the parent header message ID as the cell ID
                execute_input.parent_header.msg_id,
                cx,
            ),
            // Outputs
            // Want to append these to an output area that is an anchor in the text editor
            Message::DisplayData(display_data) => {
                let Some(cell) = self
                    .cells
                    .iter_mut()
                    .find(|cell| cell.id == display_data.parent_header.msg_id)
                else {
                    return; // TODO
                };

                cell.output = Some(display_data.content.data.into_iter().collect());

                self.update_blocks(cx);
            }
            Message::ExecuteResult(execute_result) => {
                let Some(cell) = self
                    .cells
                    .iter_mut()
                    .find(|cell| cell.id == execute_result.parent_header.msg_id)
                else {
                    return; // TODO
                };

                cell.output = Some(execute_result.content.data.into_iter().collect());

                self.update_blocks(cx);
            }
            Message::Stream(stream) => {
                // editor.insert(&format!("STREAM!!! {:?}\n", stream), cx);
            }
            // Status
            Message::Status(status) => {
                // editor.insert(&format!("STATUS!!! {:?}\n", status), cx);
            }
            Message::UnknownType(dynamic_data) => {
                // editor.insert(&format!("DYNAMIC_DATA!!! {:?}\n", dynamic_data), cx);
            }
        };
        // msg_type is "execute_input", assume for this fixture demo that the user wrote that as code which was submitted
        // then all the messages after are in the "outputs" for the cell
        //
        // "display_data" -> "data" -> "text/plain" -> div(text)
        // "stream" -> "text" -> div(text)

        // update the notebook with the message
    }

    fn update_blocks(&mut self, cx: &mut ViewContext<Self>) {
        self.editor.update(cx, |editor, cx| {
            let buffer = editor.buffer().read(cx).snapshot(cx);
            let excerpt_id = *buffer.as_singleton().unwrap().0;
            let old_blocks = std::mem::take(&mut self.blocks);
            let new_blocks = self
                .cells
                .iter()
                .map(|cell| BlockProperties {
                    position: buffer
                        .anchor_in_excerpt(excerpt_id, cell.block_anchor)
                        .unwrap(),
                    height: 2,
                    style: BlockStyle::Sticky,
                    render: Arc::new({
                        let id = cell.id.clone();
                        let output = cell.output.clone();
                        move |_cx| {
                            v_flex()
                                .h_11()
                                .relative()
                                .gap_1()
                                .child(id.clone())
                                .children(output.as_ref().map(|output| {
                                    div().children(output.into_iter().map(|(k, v)| {
                                        div()
                                            .bg(white())
                                            .rounded_md()
                                            .shadow_md()
                                            .child(k.clone())
                                            .child(format!("{:?}", v.clone()))
                                    }))
                                }))
                                .into_any_element()
                        }
                    }),
                    disposition: BlockDisposition::Above,
                })
                .collect::<Vec<_>>();

            editor.remove_blocks(old_blocks, None, cx);
            let ids = editor.insert_blocks(new_blocks, None, cx);
            self.blocks = HashSet::from_iter(ids);
        });
    }
}

impl Item for Notebook {
    type Event = ();

    fn tab_content(
        &self,
        _detail: Option<usize>,
        _selected: bool,
        _cx: &ui::prelude::WindowContext,
    ) -> gpui::AnyElement {
        "Notebook".into_any_element()
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        None
    }

    fn to_item_events(_event: &Self::Event, _f: impl FnMut(workspace::item::ItemEvent)) {}
}

impl EventEmitter<()> for Notebook {}

impl gpui::Render for Notebook {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        self.editor.clone()
    }
}

impl FocusableView for Notebook {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}
