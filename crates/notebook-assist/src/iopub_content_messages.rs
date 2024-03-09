use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
pub struct Transient {
    pub display_id: String,
}

#[derive(Deserialize, Debug)]
pub struct UpdateDisplayData {
    pub data: HashMap<String, serde_json::Value>,
    pub metadata: serde_json::Value,
    // Dev note: serde(default) is important here, when using custom deserialize_with and Option
    // then it will throw errors when the field is missing unless default is included.
    #[serde(default, deserialize_with = "deserialize_transient")]
    pub transient: Option<Transient>,
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
pub struct DisplayData {
    pub data: HashMap<String, serde_json::Value>,
    pub metadata: serde_json::Value,
    // Dev note: serde(default) is important here, when using custom deserialize_with and Option
    // then it will throw errors when the field is missing unless default is included.
    #[serde(default, deserialize_with = "deserialize_transient")]
    pub transient: Option<Transient>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct ExecuteInput {
    pub code: String,
    execution_count: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum KernelStatus {
    Busy,
    Idle,
    Starting,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub execution_state: KernelStatus,
}

#[derive(Clone, Serialize, PartialEq, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum StreamName {
    Stdout,
    Stderr,
}

#[allow(dead_code)]
#[derive(Clone, Serialize, PartialEq, Deserialize, Debug)]
pub struct Stream {
    pub name: StreamName,
    #[serde(deserialize_with = "list_or_string_to_string")]
    pub text: String,
}

#[allow(dead_code)]
#[derive(Clone, Serialize, PartialEq, Deserialize, Debug)]
pub struct ExecuteResult {
    pub execution_count: u32,
    pub data: HashMap<String, serde_json::Value>,
    pub metadata: serde_json::Value,
}

#[derive(Clone, Serialize, PartialEq, Deserialize, Debug)]
pub struct Error {
    pub ename: String,
    pub evalue: String,
    pub traceback: Vec<String>,
}

pub fn list_or_string_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize the source field as a serde_json::Value
    let source_value: serde_json::Value = Deserialize::deserialize(deserializer)?;

    // Check if the source is an array of strings
    if let Some(source_array) = source_value.as_array() {
        // Join the array of strings into a single string
        let source_string = source_array
            .iter()
            .map(|s| s.as_str().unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(source_string)
    } else if let Some(source_str) = source_value.as_str() {
        // If source is already a string, return it
        Ok(source_str.to_string())
    } else {
        Err(serde::de::Error::custom("Invalid source format"))
    }
}

// If the transient field is an empty dict, deserialize it as None
// otherwise deserialize it as Some(Transient)
fn deserialize_transient<'de, D>(deserializer: D) -> Result<Option<Transient>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Option<serde_json::Value> = Option::deserialize(deserializer)?;
    match v {
        Some(serde_json::Value::Object(map)) if map.is_empty() => Ok(None),
        Some(value) => serde_json::from_value(value)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}
