use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ChatCompletion {
    pub model: String,
    pub messages: Vec<Message>,
    pub temperature: f32,
}

#[derive(Debug, Deserialize)]
pub struct APIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct APIChoice {
    pub message: APIMessage,
}

#[derive(Debug, Deserialize)]
pub struct APIResponse {
    pub choices: Vec<APIChoice>,
}
