mod impls;
pub mod runtime;
pub mod stream;

use serde::{Deserialize, Serialize};
use crate::entities::stream::Chunk;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    pub model: String,
    pub messages: Vec<MarkedMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_thinking: Option<bool>,
}

#[derive(Debug)]
pub enum AgentTask {
    Input { content: String },
    MessageRequest { channel: tokio::sync::oneshot::Sender<Chunk> }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct MarkedMessage {
    #[serde(skip)]
    id: usize,
    message: Message,
}

impl MarkedMessage {
    pub fn to_message(&self) -> Message {
        self.message.clone()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "role")]
pub enum Message {
    System {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    User {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    Assistant {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        reasoning_content: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<ToolCall>>,
    },
    Tool {
        content: String,
        tool_call_id: String, // Tool 角色强制要求 ID
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tool {
    pub r#type: String,
    pub function: FunctionDetail,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionDetail {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: CallFunction,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Choice {
    pub index: u32,
    pub message: Message,
    pub finish_reason: FinishReason,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[derive(Clone)]
pub enum FinishReason {
    Stop,
    ToolCalls,
    Length,
    ContentFilter,
    Null,
}
