use editor::Editor;
use gpui::{actions, red, AppContext, EventEmitter, FocusHandle, FocusableView, View};
use ui::{IntoElement, ParentElement as _, Styled as _, ViewContext, VisualContext};
use workspace::{item::Item, Workspace};

actions!(notebook, [Deploy]);

pub fn init(cx: &mut AppContext) {
    cx.observe_new_views(
        |workspace: &mut Workspace, _cx: &mut ViewContext<Workspace>| {
            workspace.register_action(|workspace, _: &Deploy, cx| {
                let item = cx.new_view(|cx| Notebook::new(cx));
                workspace.add_item_to_active_pane(Box::new(item), cx);
            });
        },
    )
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

pub struct Notebook {
    focus_handle: FocusHandle,
    editor: View<Editor>,
}

impl Notebook {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            editor: cx.new_view(|cx| Editor::auto_height(4, cx)),
        }
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
        gpui::div().size_full().bg(red()).child(self.editor.clone())
    }
}

impl FocusableView for Notebook {
    fn focus_handle(&self, _cx: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// languages: Arc<LanguageRegistry>,
// _subscriptions: Vec<Subscription>,
// notebook: NotebookConnection,
// struct NotebookConnection {}

// struct ActiveConversationEditor {
//     editor: View<ConversationEditor>,
//     _subscriptions: Vec<Subscription>,
// }

// type CellId = String;
// type OutputId = String;

// struct NotebookCell {
//     source: Model<Buffer>,
//     outputs: Vec<OutputId>,
// }

// struct Notebook {
//     cells_by_id: HashMap<CellId, NotebookCell>,
//     cell_order: Vec<CellId>,

//     outputs_by_id: HashMap<OutputId, NotebookOutput>,

//     source: Model<Buffer>,
// }

// impl Notebook {}

// impl Render for Notebook {
//     fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
//         let editor = self.active_conversation_editor().unwrap();
//         let conversation = editor.read(cx).conversation.clone();
//         div()
//             .size_full()
//             .child(editor.clone())
//             .child(
//                 h_flex()
//                     .absolute()
//                     .gap_1()
//                     .top_3()
//                     .right_5()
//                     .child(self.render_model(&conversation, cx))
//                     .children(self.render_remaining_tokens(&conversation, cx)),
//             )
//             .into_any_element()
//     }
// }

// enum ConversationEvent {
//     MessagesEdited,
//     SummaryChanged,
//     StreamedCompletion,
// }

// #[derive(Default)]
// struct Summary {
//     text: String,
//     done: bool,
// }

// struct PendingCompletion {
//     id: usize,
//     _task: Task<()>,
// }

// enum ConversationEditorEvent {
//     TabContentChanged,
// }

// #[derive(Copy, Clone, Debug, PartialEq)]
// struct ScrollPosition {
//     offset_before_cursor: gpui::Point<f32>,
//     cursor: Anchor,
// }

// struct ConversationEditor {
//     conversation: Model<Conversation>,
//     fs: Arc<dyn Fs>,
//     workspace: WeakView<Workspace>,
//     editor: View<Editor>,
//     blocks: HashSet<BlockId>,
//     scroll_position: Option<ScrollPosition>,
//     _subscriptions: Vec<Subscription>,
// }

// impl ConversationEditor {
//     fn new(
//         model: LanguageModel,
//         language_registry: Arc<LanguageRegistry>,
//         fs: Arc<dyn Fs>,
//         workspace: WeakView<Workspace>,
//         cx: &mut ViewContext<Self>,
//     ) -> Self {
//         let conversation = cx.new_model(|cx| Conversation::new(model, language_registry, cx));
//         Self::for_conversation(conversation, fs, workspace, cx)
//     }

//     fn for_conversation(
//         conversation: Model<Conversation>,
//         fs: Arc<dyn Fs>,
//         workspace: WeakView<Workspace>,
//         cx: &mut ViewContext<Self>,
//     ) -> Self {
//         let editor = cx.new_view(|cx| {
//             let mut editor = Editor::for_buffer(conversation.read(cx).buffer.clone(), None, cx);
//             editor.set_soft_wrap_mode(SoftWrap::EditorWidth, cx);
//             editor.set_show_gutter(false, cx);
//             editor.set_show_wrap_guides(false, cx);
//             editor
//         });

//         let _subscriptions = vec![
//             cx.observe(&conversation, |_, _, cx| cx.notify()),
//             cx.subscribe(&conversation, Self::handle_conversation_event),
//             cx.subscribe(&editor, Self::handle_editor_event),
//         ];

//         let mut this = Self {
//             conversation,
//             editor,
//             blocks: Default::default(),
//             scroll_position: None,
//             fs,
//             workspace,
//             _subscriptions,
//         };
//         this.update_message_headers(cx);
//         this
//     }

//     fn assist(&mut self, _: &Assist, cx: &mut ViewContext<Self>) {
//         self.conversation.update(cx, |conversation, cx| {
//             report_assistant_event(
//                 self.workspace.clone(),
//                 Some(conversation),
//                 AssistantKind::Panel,
//                 cx,
//             )
//         });

//         let cursors = self.cursors(cx);

//         let user_messages = self.conversation.update(cx, |conversation, cx| {
//             let selected_messages = conversation
//                 .messages_for_offsets(cursors, cx)
//                 .into_iter()
//                 .map(|message| message.id)
//                 .collect();
//             conversation.assist(selected_messages, cx)
//         });
//         let new_selections = user_messages
//             .iter()
//             .map(|message| {
//                 let cursor = message
//                     .start
//                     .to_offset(self.conversation.read(cx).buffer.read(cx));
//                 cursor..cursor
//             })
//             .collect::<Vec<_>>();
//         if !new_selections.is_empty() {
//             self.editor.update(cx, |editor, cx| {
//                 editor.change_selections(
//                     Some(Autoscroll::Strategy(AutoscrollStrategy::Fit)),
//                     cx,
//                     |selections| selections.select_ranges(new_selections),
//                 );
//             });
//             // Avoid scrolling to the new cursor position so the assistant's output is stable.
//             cx.defer(|this, _| this.scroll_position = None);
//         }
//     }

//     fn cancel_last_assist(&mut self, _: &editor::actions::Cancel, cx: &mut ViewContext<Self>) {
//         if !self
//             .conversation
//             .update(cx, |conversation, _| conversation.cancel_last_assist())
//         {
//             cx.propagate();
//         }
//     }

//     fn cycle_message_role(&mut self, _: &CycleMessageRole, cx: &mut ViewContext<Self>) {
//         let cursors = self.cursors(cx);
//         self.conversation.update(cx, |conversation, cx| {
//             let messages = conversation
//                 .messages_for_offsets(cursors, cx)
//                 .into_iter()
//                 .map(|message| message.id)
//                 .collect();
//             conversation.cycle_message_roles(messages, cx)
//         });
//     }

//     fn cursors(&self, cx: &AppContext) -> Vec<usize> {
//         let selections = self.editor.read(cx).selections.all::<usize>(cx);
//         selections
//             .into_iter()
//             .map(|selection| selection.head())
//             .collect()
//     }

//     fn handle_conversation_event(
//         &mut self,
//         _: Model<Conversation>,
//         event: &ConversationEvent,
//         cx: &mut ViewContext<Self>,
//     ) {
//         match event {
//             ConversationEvent::MessagesEdited => {
//                 self.update_message_headers(cx);
//                 self.conversation.update(cx, |conversation, cx| {
//                     conversation.save(Some(Duration::from_millis(500)), self.fs.clone(), cx);
//                 });
//             }
//             ConversationEvent::SummaryChanged => {
//                 cx.emit(ConversationEditorEvent::TabContentChanged);
//                 self.conversation.update(cx, |conversation, cx| {
//                     conversation.save(None, self.fs.clone(), cx);
//                 });
//             }
//             ConversationEvent::StreamedCompletion => {
//                 self.editor.update(cx, |editor, cx| {
//                     if let Some(scroll_position) = self.scroll_position {
//                         let snapshot = editor.snapshot(cx);
//                         let cursor_point = scroll_position.cursor.to_display_point(&snapshot);
//                         let scroll_top =
//                             cursor_point.row() as f32 - scroll_position.offset_before_cursor.y;
//                         editor.set_scroll_position(
//                             point(scroll_position.offset_before_cursor.x, scroll_top),
//                             cx,
//                         );
//                     }
//                 });
//             }
//         }
//     }

//     fn handle_editor_event(
//         &mut self,
//         _: View<Editor>,
//         event: &EditorEvent,
//         cx: &mut ViewContext<Self>,
//     ) {
//         match event {
//             EditorEvent::ScrollPositionChanged { autoscroll, .. } => {
//                 let cursor_scroll_position = self.cursor_scroll_position(cx);
//                 if *autoscroll {
//                     self.scroll_position = cursor_scroll_position;
//                 } else if self.scroll_position != cursor_scroll_position {
//                     self.scroll_position = None;
//                 }
//             }
//             EditorEvent::SelectionsChanged { .. } => {
//                 self.scroll_position = self.cursor_scroll_position(cx);
//             }
//             _ => {}
//         }
//     }

//     fn cursor_scroll_position(&self, cx: &mut ViewContext<Self>) -> Option<ScrollPosition> {
//         self.editor.update(cx, |editor, cx| {
//             let snapshot = editor.snapshot(cx);
//             let cursor = editor.selections.newest_anchor().head();
//             let cursor_row = cursor.to_display_point(&snapshot.display_snapshot).row() as f32;
//             let scroll_position = editor
//                 .scroll_manager
//                 .anchor()
//                 .scroll_position(&snapshot.display_snapshot);

//             let scroll_bottom = scroll_position.y + editor.visible_line_count().unwrap_or(0.);
//             if (scroll_position.y..scroll_bottom).contains(&cursor_row) {
//                 Some(ScrollPosition {
//                     cursor,
//                     offset_before_cursor: point(scroll_position.x, cursor_row - scroll_position.y),
//                 })
//             } else {
//                 None
//             }
//         })
//     }

//     fn update_message_headers(&mut self, cx: &mut ViewContext<Self>) {
//         self.editor.update(cx, |editor, cx| {
//             let buffer = editor.buffer().read(cx).snapshot(cx);
//             let excerpt_id = *buffer.as_singleton().unwrap().0;
//             let old_blocks = std::mem::take(&mut self.blocks);
//             let new_blocks = self
//                 .conversation
//                 .read(cx)
//                 .messages(cx)
//                 .map(|message| BlockProperties {
//                     position: buffer.anchor_in_excerpt(excerpt_id, message.anchor),
//                     height: 2,
//                     style: BlockStyle::Sticky,
//                     render: Arc::new({
//                         let conversation = self.conversation.clone();
//                         move |_cx| {
//                             let message_id = message.id;
//                             let sender = ButtonLike::new("role")
//                                 .style(ButtonStyle::Filled)
//                                 .child(match message.role {
//                                     Role::User => Label::new("You").color(Color::Default),
//                                     Role::Assistant => Label::new("Assistant").color(Color::Info),
//                                     Role::System => Label::new("System").color(Color::Warning),
//                                 })
//                                 .tooltip(|cx| {
//                                     Tooltip::with_meta(
//                                         "Toggle message role",
//                                         None,
//                                         "Available roles: You (User), Assistant, System",
//                                         cx,
//                                     )
//                                 })
//                                 .on_click({
//                                     let conversation = conversation.clone();
//                                     move |_, cx| {
//                                         conversation.update(cx, |conversation, cx| {
//                                             conversation.cycle_message_roles(
//                                                 HashSet::from_iter(Some(message_id)),
//                                                 cx,
//                                             )
//                                         })
//                                     }
//                                 });

//                             h_flex()
//                                 .id(("message_header", message_id.0))
//                                 .h_11()
//                                 .relative()
//                                 .gap_1()
//                                 .child(sender)
//                                 // TODO: Only show this if the message if the message has been sent
//                                 .child(
//                                     Label::new(
//                                         FormatDistance::from_now(DateTimeType::Local(
//                                             message.sent_at,
//                                         ))
//                                         .hide_prefix(true)
//                                         .add_suffix(true)
//                                         .to_string(),
//                                     )
//                                     .size(LabelSize::XSmall)
//                                     .color(Color::Muted),
//                                 )
//                                 .children(
//                                     if let MessageStatus::Error(error) = message.status.clone() {
//                                         Some(
//                                             div()
//                                                 .id("error")
//                                                 .tooltip(move |cx| Tooltip::text(error.clone(), cx))
//                                                 .child(Icon::new(IconName::XCircle)),
//                                         )
//                                     } else {
//                                         None
//                                     },
//                                 )
//                                 .into_any_element()
//                         }
//                     }),
//                     disposition: BlockDisposition::Above,
//                 })
//                 .collect::<Vec<_>>();

//             editor.remove_blocks(old_blocks, None, cx);
//             let ids = editor.insert_blocks(new_blocks, None, cx);
//             self.blocks = HashSet::from_iter(ids);
//         });
//     }

//     fn quote_selection(
//         workspace: &mut Workspace,
//         _: &QuoteSelection,
//         cx: &mut ViewContext<Workspace>,
//     ) {
//         let Some(panel) = workspace.panel::<Notebook>(cx) else {
//             return;
//         };
//         let Some(editor) = workspace
//             .active_item(cx)
//             .and_then(|item| item.act_as::<Editor>(cx))
//         else {
//             return;
//         };

//         let editor = editor.read(cx);
//         let range = editor.selections.newest::<usize>(cx).range();
//         let buffer = editor.buffer().read(cx).snapshot(cx);
//         let start_language = buffer.language_at(range.start);
//         let end_language = buffer.language_at(range.end);
//         let language_name = if start_language == end_language {
//             start_language.map(|language| language.name())
//         } else {
//             None
//         };
//         let language_name = language_name.as_deref().unwrap_or("").to_lowercase();

//         let selected_text = buffer.text_for_range(range).collect::<String>();
//         let text = if selected_text.is_empty() {
//             None
//         } else {
//             Some(if language_name == "markdown" {
//                 selected_text
//                     .lines()
//                     .map(|line| format!("> {}", line))
//                     .collect::<Vec<_>>()
//                     .join("\n")
//             } else {
//                 format!("```{language_name}\n{selected_text}\n```")
//             })
//         };

//         // Activate the panel
//         if !panel.focus_handle(cx).contains_focused(cx) {
//             workspace.toggle_panel_focus::<Notebook>(cx);
//         }

//         if let Some(text) = text {
//             panel.update(cx, |panel, cx| {
//                 let conversation = panel
//                     .active_conversation_editor()
//                     .cloned()
//                     .unwrap_or_else(|| panel.new_conversation(cx));
//                 conversation.update(cx, |conversation, cx| {
//                     conversation
//                         .editor
//                         .update(cx, |editor, cx| editor.insert(&text, cx))
//                 });
//             });
//         }
//     }

//     fn copy(&mut self, _: &editor::actions::Copy, cx: &mut ViewContext<Self>) {
//         let editor = self.editor.read(cx);
//         let conversation = self.conversation.read(cx);
//         if editor.selections.count() == 1 {
//             let selection = editor.selections.newest::<usize>(cx);
//             let mut copied_text = String::new();
//             let mut spanned_messages = 0;
//             for message in conversation.messages(cx) {
//                 if message.offset_range.start >= selection.range().end {
//                     break;
//                 } else if message.offset_range.end >= selection.range().start {
//                     let range = cmp::max(message.offset_range.start, selection.range().start)
//                         ..cmp::min(message.offset_range.end, selection.range().end);
//                     if !range.is_empty() {
//                         spanned_messages += 1;
//                         write!(&mut copied_text, "## {}\n\n", message.role).unwrap();
//                         for chunk in conversation.buffer.read(cx).text_for_range(range) {
//                             copied_text.push_str(&chunk);
//                         }
//                         copied_text.push('\n');
//                     }
//                 }
//             }

//             if spanned_messages > 1 {
//                 cx.write_to_clipboard(ClipboardItem::new(copied_text));
//                 return;
//             }
//         }

//         cx.propagate();
//     }

//     fn split(&mut self, _: &Split, cx: &mut ViewContext<Self>) {
//         self.conversation.update(cx, |conversation, cx| {
//             let selections = self.editor.read(cx).selections.disjoint_anchors();
//             for selection in selections.into_iter() {
//                 let buffer = self.editor.read(cx).buffer().read(cx).snapshot(cx);
//                 let range = selection
//                     .map(|endpoint| endpoint.to_offset(&buffer))
//                     .range();
//                 conversation.split_message(range, cx);
//             }
//         });
//     }

//     fn eval(&mut self, _: &Eval, cx: &mut ViewContext<Self>) {
//         self.conversation.update(cx, |conversation, cx| {
//             let selections = self.editor.read(cx).selections.disjoint_anchors();
//             let buffer = self.editor.read(cx).buffer().read(cx).snapshot(cx);
//             for selection in selections.into_iter() {
//                 let range = selection
//                     .map(|endpoint| endpoint.to_offset(&buffer))
//                     .range();
//                 conversation.split_message(range, cx);
//             }
//         });
//     }

//     fn save(&mut self, _: &Save, cx: &mut ViewContext<Self>) {
//         self.conversation.update(cx, |conversation, cx| {
//             conversation.save(None, self.fs.clone(), cx)
//         });
//     }

//     fn title(&self, cx: &AppContext) -> String {
//         self.conversation
//             .read(cx)
//             .summary
//             .as_ref()
//             .map(|summary| summary.text.clone())
//             .unwrap_or_else(|| "New Conversation".into())
//     }
// }

// impl EventEmitter<ConversationEditorEvent> for ConversationEditor {}

// impl Render for ConversationEditor {
//     fn render(&mut self, cx: &mut ViewContext<Self>) -> impl Element {
//         div()
//             .key_context("ConversationEditor")
//             .capture_action(cx.listener(ConversationEditor::cancel_last_assist))
//             .capture_action(cx.listener(ConversationEditor::save))
//             .capture_action(cx.listener(ConversationEditor::copy))
//             .capture_action(cx.listener(ConversationEditor::cycle_message_role))
//             .on_action(cx.listener(ConversationEditor::assist))
//             .on_action(cx.listener(ConversationEditor::split))
//             .on_action(cx.listener(ConversationEditor::eval))
//             .size_full()
//             .relative()
//             .child(
//                 div()
//                     .size_full()
//                     .pl_4()
//                     .bg(cx.theme().colors().editor_background)
//                     .child(self.editor.clone()),
//             )
//     }
// }

// impl FocusableView for ConversationEditor {
//     fn focus_handle(&self, cx: &AppContext) -> FocusHandle {
//         self.editor.focus_handle(cx)
//     }
// }

// #[derive(Clone, Debug)]
// struct MessageAnchor {
//     id: MessageId,
//     start: language::Anchor,
// }

// struct Conversation {
//     id: Option<String>,
//     buffer: Model<Buffer>,
//     message_anchors: Vec<MessageAnchor>,
//     messages_metadata: HashMap<MessageId, MessageMetadata>,
//     next_message_id: MessageId,
//     summary: Option<Summary>,
//     pending_summary: Task<Option<()>>,
//     completion_count: usize,
//     pending_completions: Vec<PendingCompletion>,
//     model: LanguageModel,
//     token_count: Option<usize>,
//     pending_token_count: Task<Option<()>>,
//     pending_save: Task<Result<()>>,
//     path: Option<PathBuf>,
//     _subscriptions: Vec<Subscription>,
// }

// impl EventEmitter<ConversationEvent> for Conversation {}

// impl Conversation {
//     fn new(
//         model: LanguageModel,
//         language_registry: Arc<LanguageRegistry>,
//         cx: &mut ModelContext<Self>,
//     ) -> Self {
//         let markdown = language_registry.language_for_name("Markdown");
//         let buffer = cx.new_model(|cx| {
//             let mut buffer = Buffer::new(0, BufferId::new(cx.entity_id().as_u64()).unwrap(), "");
//             buffer.set_language_registry(language_registry);
//             cx.spawn(|buffer, mut cx| async move {
//                 let markdown = markdown.await?;
//                 buffer.update(&mut cx, |buffer: &mut Buffer, cx| {
//                     buffer.set_language(Some(markdown), cx)
//                 })?;
//                 anyhow::Ok(())
//             })
//             .detach_and_log_err(cx);
//             buffer
//         });

//         let mut this = Self {
//             id: Some(Uuid::new_v4().to_string()),
//             message_anchors: Default::default(),
//             messages_metadata: Default::default(),
//             next_message_id: Default::default(),
//             summary: None,
//             pending_summary: Task::ready(None),
//             completion_count: Default::default(),
//             pending_completions: Default::default(),
//             token_count: None,
//             pending_token_count: Task::ready(None),
//             model,
//             _subscriptions: vec![cx.subscribe(&buffer, Self::handle_buffer_event)],
//             pending_save: Task::ready(Ok(())),
//             path: None,
//             buffer,
//         };
//         let message = MessageAnchor {
//             id: MessageId(post_inc(&mut this.next_message_id.0)),
//             start: language::Anchor::MIN,
//         };
//         this.message_anchors.push(message.clone());
//         this.messages_metadata.insert(
//             message.id,
//             MessageMetadata {
//                 role: Role::User,
//                 sent_at: Local::now(),
//                 status: MessageStatus::Done,
//             },
//         );

//         this.count_remaining_tokens(cx);
//         this
//     }

//     fn serialize(&self, cx: &AppContext) -> SavedConversation {
//         SavedConversation {
//             id: self.id.clone(),
//             zed: "conversation".into(),
//             version: SavedConversation::VERSION.into(),
//             text: self.buffer.read(cx).text(),
//             message_metadata: self.messages_metadata.clone(),
//             messages: self
//                 .messages(cx)
//                 .map(|message| SavedMessage {
//                     id: message.id,
//                     start: message.offset_range.start,
//                 })
//                 .collect(),
//             summary: self
//                 .summary
//                 .as_ref()
//                 .map(|summary| summary.text.clone())
//                 .unwrap_or_default(),
//         }
//     }

//     async fn deserialize(
//         saved_conversation: SavedConversation,
//         model: LanguageModel,
//         path: PathBuf,
//         language_registry: Arc<LanguageRegistry>,
//         cx: &mut AsyncAppContext,
//     ) -> Result<Model<Self>> {
//         let id = match saved_conversation.id {
//             Some(id) => Some(id),
//             None => Some(Uuid::new_v4().to_string()),
//         };

//         let markdown = language_registry.language_for_name("Markdown");
//         let mut message_anchors = Vec::new();
//         let mut next_message_id = MessageId(0);
//         let buffer = cx.new_model(|cx| {
//             let mut buffer = Buffer::new(
//                 0,
//                 BufferId::new(cx.entity_id().as_u64()).unwrap(),
//                 saved_conversation.text,
//             );
//             for message in saved_conversation.messages {
//                 message_anchors.push(MessageAnchor {
//                     id: message.id,
//                     start: buffer.anchor_before(message.start),
//                 });
//                 next_message_id = cmp::max(next_message_id, MessageId(message.id.0 + 1));
//             }
//             buffer.set_language_registry(language_registry);
//             cx.spawn(|buffer, mut cx| async move {
//                 let markdown = markdown.await?;
//                 buffer.update(&mut cx, |buffer: &mut Buffer, cx| {
//                     buffer.set_language(Some(markdown), cx)
//                 })?;
//                 anyhow::Ok(())
//             })
//             .detach_and_log_err(cx);
//             buffer
//         })?;

//         cx.new_model(|cx| {
//             let mut this = Self {
//                 id,
//                 message_anchors,
//                 messages_metadata: saved_conversation.message_metadata,
//                 next_message_id,
//                 summary: Some(Summary {
//                     text: saved_conversation.summary,
//                     done: true,
//                 }),
//                 pending_summary: Task::ready(None),
//                 completion_count: Default::default(),
//                 pending_completions: Default::default(),
//                 token_count: None,
//                 pending_token_count: Task::ready(None),
//                 model,
//                 _subscriptions: vec![cx.subscribe(&buffer, Self::handle_buffer_event)],
//                 pending_save: Task::ready(Ok(())),
//                 path: Some(path),
//                 buffer,
//             };
//             this.count_remaining_tokens(cx);
//             this
//         })
//     }

//     fn handle_buffer_event(
//         &mut self,
//         _: Model<Buffer>,
//         event: &language::Event,
//         cx: &mut ModelContext<Self>,
//     ) {
//         match event {
//             language::Event::Edited => {
//                 self.count_remaining_tokens(cx);
//                 cx.emit(ConversationEvent::MessagesEdited);
//             }
//             _ => {}
//         }
//     }

//     fn count_remaining_tokens(&mut self, cx: &mut ModelContext<Self>) {
//         let request = self.to_completion_request(cx);
//         self.pending_token_count = cx.spawn(|this, mut cx| {
//             async move {
//                 cx.background_executor()
//                     .timer(Duration::from_millis(200))
//                     .await;

//                 let token_count = cx
//                     .update(|cx| CompletionProvider::global(cx).count_tokens(request, cx))?
//                     .await?;

//                 this.update(&mut cx, |this, cx| {
//                     this.token_count = Some(token_count);
//                     cx.notify()
//                 })?;
//                 anyhow::Ok(())
//             }
//             .log_err()
//         });
//     }

//     fn remaining_tokens(&self) -> Option<isize> {
//         Some(self.model.max_token_count() as isize - self.token_count? as isize)
//     }

//     fn set_model(&mut self, model: LanguageModel, cx: &mut ModelContext<Self>) {
//         self.model = model;
//         self.count_remaining_tokens(cx);
//     }

//     fn assist(
//         &mut self,
//         selected_messages: HashSet<MessageId>,
//         cx: &mut ModelContext<Self>,
//     ) -> Vec<MessageAnchor> {
//         let mut user_messages = Vec::new();

//         let last_message_id = if let Some(last_message_id) =
//             self.message_anchors.iter().rev().find_map(|message| {
//                 message
//                     .start
//                     .is_valid(self.buffer.read(cx))
//                     .then_some(message.id)
//             }) {
//             last_message_id
//         } else {
//             return Default::default();
//         };

//         let mut should_assist = false;
//         for selected_message_id in selected_messages {
//             let selected_message_role =
//                 if let Some(metadata) = self.messages_metadata.get(&selected_message_id) {
//                     metadata.role
//                 } else {
//                     continue;
//                 };

//             if selected_message_role == Role::Assistant {
//                 if let Some(user_message) = self.insert_message_after(
//                     selected_message_id,
//                     Role::User,
//                     MessageStatus::Done,
//                     cx,
//                 ) {
//                     user_messages.push(user_message);
//                 }
//             } else {
//                 should_assist = true;
//             }
//         }

//         if should_assist {
//             if !CompletionProvider::global(cx).is_authenticated() {
//                 log::info!("completion provider has no credentials");
//                 return Default::default();
//             }

//             let request = self.to_completion_request(cx);
//             let stream = CompletionProvider::global(cx).complete(request);
//             let assistant_message = self
//                 .insert_message_after(last_message_id, Role::Assistant, MessageStatus::Pending, cx)
//                 .unwrap();

//             // Queue up the user's next reply.
//             let user_message = self
//                 .insert_message_after(assistant_message.id, Role::User, MessageStatus::Done, cx)
//                 .unwrap();
//             user_messages.push(user_message);

//             let task = cx.spawn({
//                 |this, mut cx| async move {
//                     let assistant_message_id = assistant_message.id;
//                     let stream_completion = async {
//                         let mut messages = stream.await?;

//                         while let Some(message) = messages.next().await {
//                             let text = message?;

//                             this.update(&mut cx, |this, cx| {
//                                 let message_ix = this
//                                     .message_anchors
//                                     .iter()
//                                     .position(|message| message.id == assistant_message_id)?;
//                                 this.buffer.update(cx, |buffer, cx| {
//                                     let offset = this.message_anchors[message_ix + 1..]
//                                         .iter()
//                                         .find(|message| message.start.is_valid(buffer))
//                                         .map_or(buffer.len(), |message| {
//                                             message.start.to_offset(buffer).saturating_sub(1)
//                                         });
//                                     buffer.edit([(offset..offset, text)], None, cx);
//                                 });
//                                 cx.emit(ConversationEvent::StreamedCompletion);

//                                 Some(())
//                             })?;
//                             smol::future::yield_now().await;
//                         }

//                         this.update(&mut cx, |this, cx| {
//                             this.pending_completions
//                                 .retain(|completion| completion.id != this.completion_count);
//                             this.summarize(cx);
//                         })?;

//                         anyhow::Ok(())
//                     };

//                     let result = stream_completion.await;

//                     this.update(&mut cx, |this, cx| {
//                         if let Some(metadata) =
//                             this.messages_metadata.get_mut(&assistant_message.id)
//                         {
//                             match result {
//                                 Ok(_) => {
//                                     metadata.status = MessageStatus::Done;
//                                 }
//                                 Err(error) => {
//                                     metadata.status = MessageStatus::Error(SharedString::from(
//                                         error.to_string().trim().to_string(),
//                                     ));
//                                 }
//                             }
//                             cx.emit(ConversationEvent::MessagesEdited);
//                         }
//                     })
//                     .ok();
//                 }
//             });

//             self.pending_completions.push(PendingCompletion {
//                 id: post_inc(&mut self.completion_count),
//                 _task: task,
//             });
//         }

//         user_messages
//     }

//     fn to_completion_request(
//         &mut self,
//         cx: &mut ModelContext<Conversation>,
//     ) -> LanguageModelRequest {
//         let request = LanguageModelRequest {
//             model: self.model.clone(),
//             messages: self
//                 .messages(cx)
//                 .filter(|message| matches!(message.status, MessageStatus::Done))
//                 .map(|message| message.to_open_ai_message(self.buffer.read(cx)))
//                 .collect(),
//             stop: vec![],
//             temperature: 1.0,
//         };
//         request
//     }

//     fn cancel_last_assist(&mut self) -> bool {
//         self.pending_completions.pop().is_some()
//     }

//     fn cycle_message_roles(&mut self, ids: HashSet<MessageId>, cx: &mut ModelContext<Self>) {
//         for id in ids {
//             if let Some(metadata) = self.messages_metadata.get_mut(&id) {
//                 metadata.role.cycle();
//                 cx.emit(ConversationEvent::MessagesEdited);
//                 cx.notify();
//             }
//         }
//     }

//     fn insert_message_after(
//         &mut self,
//         message_id: MessageId,
//         role: Role,
//         status: MessageStatus,
//         cx: &mut ModelContext<Self>,
//     ) -> Option<MessageAnchor> {
//         if let Some(prev_message_ix) = self
//             .message_anchors
//             .iter()
//             .position(|message| message.id == message_id)
//         {
//             // Find the next valid message after the one we were given.
//             let mut next_message_ix = prev_message_ix + 1;
//             while let Some(next_message) = self.message_anchors.get(next_message_ix) {
//                 if next_message.start.is_valid(self.buffer.read(cx)) {
//                     break;
//                 }
//                 next_message_ix += 1;
//             }

//             let start = self.buffer.update(cx, |buffer, cx| {
//                 let offset = self
//                     .message_anchors
//                     .get(next_message_ix)
//                     .map_or(buffer.len(), |message| message.start.to_offset(buffer) - 1);
//                 buffer.edit([(offset..offset, "\n")], None, cx);
//                 buffer.anchor_before(offset + 1)
//             });
//             let message = MessageAnchor {
//                 id: MessageId(post_inc(&mut self.next_message_id.0)),
//                 start,
//             };
//             self.message_anchors
//                 .insert(next_message_ix, message.clone());
//             self.messages_metadata.insert(
//                 message.id,
//                 MessageMetadata {
//                     role,
//                     sent_at: Local::now(),
//                     status,
//                 },
//             );
//             cx.emit(ConversationEvent::MessagesEdited);
//             Some(message)
//         } else {
//             None
//         }
//     }

//     fn split_message(
//         &mut self,
//         range: Range<usize>,
//         cx: &mut ModelContext<Self>,
//     ) -> (Option<MessageAnchor>, Option<MessageAnchor>) {
//         let start_message = self.message_for_offset(range.start, cx);
//         let end_message = self.message_for_offset(range.end, cx);
//         if let Some((start_message, end_message)) = start_message.zip(end_message) {
//             // Prevent splitting when range spans multiple messages.
//             if start_message.id != end_message.id {
//                 return (None, None);
//             }

//             let message = start_message;
//             let role = message.role;
//             let mut edited_buffer = false;

//             let mut suffix_start = None;
//             if range.start > message.offset_range.start && range.end < message.offset_range.end - 1
//             {
//                 if self.buffer.read(cx).chars_at(range.end).next() == Some('\n') {
//                     suffix_start = Some(range.end + 1);
//                 } else if self.buffer.read(cx).reversed_chars_at(range.end).next() == Some('\n') {
//                     suffix_start = Some(range.end);
//                 }
//             }

//             let suffix = if let Some(suffix_start) = suffix_start {
//                 MessageAnchor {
//                     id: MessageId(post_inc(&mut self.next_message_id.0)),
//                     start: self.buffer.read(cx).anchor_before(suffix_start),
//                 }
//             } else {
//                 self.buffer.update(cx, |buffer, cx| {
//                     buffer.edit([(range.end..range.end, "\n")], None, cx);
//                 });
//                 edited_buffer = true;
//                 MessageAnchor {
//                     id: MessageId(post_inc(&mut self.next_message_id.0)),
//                     start: self.buffer.read(cx).anchor_before(range.end + 1),
//                 }
//             };

//             self.message_anchors
//                 .insert(message.index_range.end + 1, suffix.clone());
//             self.messages_metadata.insert(
//                 suffix.id,
//                 MessageMetadata {
//                     role,
//                     sent_at: Local::now(),
//                     status: MessageStatus::Done,
//                 },
//             );

//             let new_messages =
//                 if range.start == range.end || range.start == message.offset_range.start {
//                     (None, Some(suffix))
//                 } else {
//                     let mut prefix_end = None;
//                     if range.start > message.offset_range.start
//                         && range.end < message.offset_range.end - 1
//                     {
//                         if self.buffer.read(cx).chars_at(range.start).next() == Some('\n') {
//                             prefix_end = Some(range.start + 1);
//                         } else if self.buffer.read(cx).reversed_chars_at(range.start).next()
//                             == Some('\n')
//                         {
//                             prefix_end = Some(range.start);
//                         }
//                     }

//                     let selection = if let Some(prefix_end) = prefix_end {
//                         cx.emit(ConversationEvent::MessagesEdited);
//                         MessageAnchor {
//                             id: MessageId(post_inc(&mut self.next_message_id.0)),
//                             start: self.buffer.read(cx).anchor_before(prefix_end),
//                         }
//                     } else {
//                         self.buffer.update(cx, |buffer, cx| {
//                             buffer.edit([(range.start..range.start, "\n")], None, cx)
//                         });
//                         edited_buffer = true;
//                         MessageAnchor {
//                             id: MessageId(post_inc(&mut self.next_message_id.0)),
//                             start: self.buffer.read(cx).anchor_before(range.end + 1),
//                         }
//                     };

//                     self.message_anchors
//                         .insert(message.index_range.end + 1, selection.clone());
//                     self.messages_metadata.insert(
//                         selection.id,
//                         MessageMetadata {
//                             role,
//                             sent_at: Local::now(),
//                             status: MessageStatus::Done,
//                         },
//                     );
//                     (Some(selection), Some(suffix))
//                 };

//             if !edited_buffer {
//                 cx.emit(ConversationEvent::MessagesEdited);
//             }
//             new_messages
//         } else {
//             (None, None)
//         }
//     }

//     fn eval_message(&mut self, range: Range<usize>, cx: &mut ModelContext<Self>) {
//         let start_message = self.message_for_offset(range.start, cx);
//         let end_message = self.message_for_offset(range.end, cx);
//         if let Some((start_message, end_message)) = start_message.zip(end_message) {
//             // Prevent splitting when range spans multiple messages.
//             if start_message.id != end_message.id {
//                 return;
//             }

//             let message = start_message;

//             let new_messages =
//                 if range.start == range.end || range.start == message.offset_range.start {
//                     (None, Some(suffix))
//                 } else {
//                     let mut prefix_end = None;
//                     if range.start > message.offset_range.start
//                         && range.end < message.offset_range.end - 1
//                     {
//                         if self.buffer.read(cx).chars_at(range.start).next() == Some('\n') {
//                             prefix_end = Some(range.start + 1);
//                         } else if self.buffer.read(cx).reversed_chars_at(range.start).next()
//                             == Some('\n')
//                         {
//                             prefix_end = Some(range.start);
//                         }
//                     }

//                     let selection = if let Some(prefix_end) = prefix_end {
//                         cx.emit(ConversationEvent::MessagesEdited);
//                         MessageAnchor {
//                             id: MessageId(post_inc(&mut self.next_message_id.0)),
//                             start: self.buffer.read(cx).anchor_before(prefix_end),
//                         }
//                     } else {
//                         self.buffer.update(cx, |buffer, cx| {
//                             buffer.edit([(range.start..range.start, "\n")], None, cx)
//                         });
//                         edited_buffer = true;
//                         MessageAnchor {
//                             id: MessageId(post_inc(&mut self.next_message_id.0)),
//                             start: self.buffer.read(cx).anchor_before(range.end + 1),
//                         }
//                     };

//                     self.message_anchors
//                         .insert(message.index_range.end + 1, selection.clone());
//                     self.messages_metadata.insert(
//                         selection.id,
//                         MessageMetadata {
//                             role,
//                             sent_at: Local::now(),
//                             status: MessageStatus::Done,
//                         },
//                     );
//                     (Some(selection), Some(suffix))
//                 };

//             if !edited_buffer {
//                 cx.emit(ConversationEvent::MessagesEdited);
//             }
//             new_messages
//         } else {
//             (None, None)
//         }
//     }

//     fn summarize(&mut self, cx: &mut ModelContext<Self>) {
//         if self.message_anchors.len() >= 2 && self.summary.is_none() {
//             if !CompletionProvider::global(cx).is_authenticated() {
//                 return;
//             }

//             let messages = self
//                 .messages(cx)
//                 .take(2)
//                 .map(|message| message.to_open_ai_message(self.buffer.read(cx)))
//                 .chain(Some(LanguageModelRequestMessage {
//                     role: Role::User,
//                     content: "Summarize the conversation into a short title without punctuation"
//                         .into(),
//                 }));
//             let request = LanguageModelRequest {
//                 model: self.model.clone(),
//                 messages: messages.collect(),
//                 stop: vec![],
//                 temperature: 1.0,
//             };

//             let stream = CompletionProvider::global(cx).complete(request);
//             self.pending_summary = cx.spawn(|this, mut cx| {
//                 async move {
//                     let mut messages = stream.await?;

//                     while let Some(message) = messages.next().await {
//                         let text = message?;
//                         this.update(&mut cx, |this, cx| {
//                             this.summary
//                                 .get_or_insert(Default::default())
//                                 .text
//                                 .push_str(&text);
//                             cx.emit(ConversationEvent::SummaryChanged);
//                         })?;
//                     }

//                     this.update(&mut cx, |this, cx| {
//                         if let Some(summary) = this.summary.as_mut() {
//                             summary.done = true;
//                             cx.emit(ConversationEvent::SummaryChanged);
//                         }
//                     })?;

//                     anyhow::Ok(())
//                 }
//                 .log_err()
//             });
//         }
//     }

//     fn message_for_offset(&self, offset: usize, cx: &AppContext) -> Option<Message> {
//         self.messages_for_offsets([offset], cx).pop()
//     }

//     fn messages_for_offsets(
//         &self,
//         offsets: impl IntoIterator<Item = usize>,
//         cx: &AppContext,
//     ) -> Vec<Message> {
//         let mut result = Vec::new();

//         let mut messages = self.messages(cx).peekable();
//         let mut offsets = offsets.into_iter().peekable();
//         let mut current_message = messages.next();
//         while let Some(offset) = offsets.next() {
//             // Locate the message that contains the offset.
//             while current_message.as_ref().map_or(false, |message| {
//                 !message.offset_range.contains(&offset) && messages.peek().is_some()
//             }) {
//                 current_message = messages.next();
//             }
//             let Some(message) = current_message.as_ref() else {
//                 break;
//             };

//             // Skip offsets that are in the same message.
//             while offsets.peek().map_or(false, |offset| {
//                 message.offset_range.contains(offset) || messages.peek().is_none()
//             }) {
//                 offsets.next();
//             }

//             result.push(message.clone());
//         }
//         result
//     }

//     fn messages<'a>(&'a self, cx: &'a AppContext) -> impl 'a + Iterator<Item = Message> {
//         let buffer = self.buffer.read(cx);
//         let mut message_anchors = self.message_anchors.iter().enumerate().peekable();
//         iter::from_fn(move || {
//             while let Some((start_ix, message_anchor)) = message_anchors.next() {
//                 let metadata = self.messages_metadata.get(&message_anchor.id)?;
//                 let message_start = message_anchor.start.to_offset(buffer);
//                 let mut message_end = None;
//                 let mut end_ix = start_ix;
//                 while let Some((_, next_message)) = message_anchors.peek() {
//                     if next_message.start.is_valid(buffer) {
//                         message_end = Some(next_message.start);
//                         break;
//                     } else {
//                         end_ix += 1;
//                         message_anchors.next();
//                     }
//                 }
//                 let message_end = message_end
//                     .unwrap_or(language::Anchor::MAX)
//                     .to_offset(buffer);
//                 return Some(Message {
//                     index_range: start_ix..end_ix,
//                     offset_range: message_start..message_end,
//                     id: message_anchor.id,
//                     anchor: message_anchor.start,
//                     role: metadata.role,
//                     sent_at: metadata.sent_at,
//                     status: metadata.status.clone(),
//                 });
//             }
//             None
//         })
//     }

//     fn save(
//         &mut self,
//         debounce: Option<Duration>,
//         fs: Arc<dyn Fs>,
//         cx: &mut ModelContext<Conversation>,
//     ) {
//         self.pending_save = cx.spawn(|this, mut cx| async move {
//             if let Some(debounce) = debounce {
//                 cx.background_executor().timer(debounce).await;
//             }

//             let (old_path, summary) = this.read_with(&cx, |this, _| {
//                 let path = this.path.clone();
//                 let summary = if let Some(summary) = this.summary.as_ref() {
//                     if summary.done {
//                         Some(summary.text.clone())
//                     } else {
//                         None
//                     }
//                 } else {
//                     None
//                 };
//                 (path, summary)
//             })?;

//             if let Some(summary) = summary {
//                 let conversation = this.read_with(&cx, |this, cx| this.serialize(cx))?;
//                 let path = if let Some(old_path) = old_path {
//                     old_path
//                 } else {
//                     let mut discriminant = 1;
//                     let mut new_path;
//                     loop {
//                         new_path = CONVERSATIONS_DIR.join(&format!(
//                             "{} - {}.zed.json",
//                             summary.trim(),
//                             discriminant
//                         ));
//                         if fs.is_file(&new_path).await {
//                             discriminant += 1;
//                         } else {
//                             break;
//                         }
//                     }
//                     new_path
//                 };

//                 fs.create_dir(CONVERSATIONS_DIR.as_ref()).await?;
//                 fs.atomic_write(path.clone(), serde_json::to_string(&conversation).unwrap())
//                     .await?;
//                 this.update(&mut cx, |this, _| this.path = Some(path))?;
//             }

//             Ok(())
//         });
//     }
// }
