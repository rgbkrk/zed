// use collections::HashMap;
// use serde::{Deserialize, Serialize};

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct ExecuteRequest {
//     code: String
//     silent: bool,
//     store_history: bool,
//     user_expressions: HashMap<String, String>,
//     allow_stdin: bool,
//     stop_on_error: bool,
// }

// #[allow(dead_code)]
// #[derive(Deserialize, Debug)]
// pub struct ExecuteReply {
//     status: String,
//     execution_count: u32,
// }

// // KernelInfoRequest, related sub-structs, and impls
// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct KernelInfoRequest {}

// impl KernelInfoRequest {
//     pub fn new() -> Self {
//         KernelInfoRequest {}
//     }
// }

// // KernelInfoReply, related sub-structs, and impls
// #[derive(Serialize, Deserialize, Debug)]
// struct HelpLink {
//     text: String,
//     url: String,
// }

// #[derive(Serialize, Deserialize, Debug)]
// struct LanguageInfo {
//     name: String,
//     version: String,
//     mimetype: String,
//     file_extension: String,
//     pygments_lexer: Option<String>,
//     codemirror_mode: Option<serde_json::Value>,
//     nbconvert_exporter: Option<String>,
// }

// #[derive(Serialize, Deserialize, Debug)]
// pub struct KernelInfoReply {
//     banner: String,
//     help_links: Option<Vec<HelpLink>>,
//     implementation: String,
//     implementation_version: String,
//     language_info: LanguageInfo,
//     protocol_version: String,
//     status: String,
// }
