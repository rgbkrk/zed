use chrono::{DateTime, Utc};
use serde::{
    de::{self, DeserializeOwned},
    Deserialize, Serialize,
};

use crate::iopub_content_messages::{DisplayData, ExecuteInput, ExecuteResult, Status, Stream};

pub enum Message {
    DisplayData(IoPubEnvelope<DisplayData>),
    ExecuteResult(IoPubEnvelope<ExecuteResult>),
    Status(IoPubEnvelope<Status>),
    Stream(IoPubEnvelope<Stream>),
    ExecuteInput(IoPubEnvelope<ExecuteInput>),
    UnknownType(DynamicEnvelope),
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let dynamic_envelope = DynamicEnvelope::deserialize(deserializer)?;
        match dynamic_envelope.header.msg_type.as_str() {
            // OUTPUT
            "display_data" => Ok(Message::DisplayData(
                unwrap_dynamic_envelope(dynamic_envelope).map_err(|e| de::Error::custom(e))?,
            )),
            "execute_result" => Ok(Message::ExecuteResult(
                unwrap_dynamic_envelope(dynamic_envelope).map_err(|e| de::Error::custom(e))?,
            )),
            "stream" => Ok(Message::Stream(
                unwrap_dynamic_envelope(dynamic_envelope).map_err(|e| de::Error::custom(e))?,
            )),
            // ECHOing of the original execution
            "execute_input" => Ok(Message::ExecuteInput(
                unwrap_dynamic_envelope(dynamic_envelope).map_err(|e| de::Error::custom(e))?,
            )),
            // Is the runtime/kernel busy or idle?
            "status" => Ok(Message::Status(
                unwrap_dynamic_envelope(dynamic_envelope).map_err(|e| de::Error::custom(e))?,
            )),
            _ => Ok(Message::UnknownType(dynamic_envelope)),
        }
    }
}

fn unwrap_dynamic_envelope<T: DeserializeOwned>(
    dynamic_envelope: DynamicEnvelope,
) -> Result<IoPubEnvelope<T>, serde_json::Error> {
    Ok(IoPubEnvelope {
        header: dynamic_envelope.header,
        parent_header: dynamic_envelope.parent_header.expect("header is required"),
        metadata: dynamic_envelope.metadata,
        content: serde_json::from_value(dynamic_envelope.content)?,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IoPubEnvelope<T> {
    pub header: Header,
    pub parent_header: Header,
    pub metadata: Option<Metadata>,
    pub content: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DynamicEnvelope {
    pub header: Header,
    pub parent_header: Option<Header>,
    pub metadata: Option<Metadata>,
    pub content: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub header: Header,
    pub parent_header: Option<Header>,
    pub metadata: Option<Metadata>,
    pub content: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub msg_id: String,
    session: String,
    username: String,
    date: DateTime<Utc>,
    pub msg_type: String,
    version: String,
}

impl Header {
    pub fn new(msg_type: String) -> Self {
        Header {
            msg_id: uuid::Uuid::new_v4().to_string(),
            session: uuid::Uuid::new_v4().to_string(),
            username: "kernel_sidecar".to_string(),
            date: Utc::now(),
            msg_type,
            version: "5.3".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata(serde_json::Value);
