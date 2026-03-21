use crate::entities::MarkedMessage;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AgentSettings {
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub tools: Option<Vec<String>>,
    pub tool_choice: Option<String>,
    pub stream: Option<bool>,
    pub thinking: Option<bool>,
    pub model: String,
    pub max_tokens: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Chunk {
    Block {
        reason: String,
        content: String,
    },
    ToolCall {
        name: String,
        arguments: String,
    },
    Messages {
        messages: Vec<MarkedMessage>,
    },
    ToolOutput {
        stdout: String,
        stderr: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<i32>,
    },
    ReasonChunk {
        content: String,
    },
    ContentChunk {
        content: String,
    },
    EventEnd,
}

#[derive(Serialize, Debug)]
pub struct ApiResponse<T> {
    pub ok: bool,
    pub result: T,
}

impl<T> ApiResponse<T> {
    pub fn ok(result: T) -> Self {
        Self { ok: true, result }
    }

    pub fn err(result: T) -> Self {
        Self { ok: false, result }
    }
}
