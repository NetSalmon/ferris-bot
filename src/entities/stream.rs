use crate::entities::{CallFunction, Choice, FinishReason, Message, Response, ToolCall, Usage};
use crate::error::AppError;
use crate::error::AppError::InternalError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct ChoicesBuffer {
    #[serde(rename = "choices")]
    pub choices: Vec<DeltaChoice>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeltaChoice {
    index: u32,
    pub delta: DeltaMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<FinishReason>,
}

impl DeltaChoice {
    pub fn export(self) -> Result<Choice, AppError> {
        let result = Choice {
            index: self.index,
            message: self.delta.export()?,
            finish_reason: self
                .finish_reason
                .ok_or(InternalError("Missing finish_reason".into()))?,
        };

        Ok(result)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeltaMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<DeltaToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
}

impl DeltaMessage {
    pub fn export(self) -> Result<Message, AppError> {
        let tool_calls = if let Some(tool_calls) = self.tool_calls {
            let mut results = Vec::new();
            for tool_call in tool_calls {
                results.push(tool_call.export()?);
            }
            Some(results)
        } else {
            None
        };

        let result = Message::Assistant {
            content: self.content,
            reasoning_content: self.reasoning_content,
            tool_calls,
        };

        Ok(result)
    }

    pub fn merge(&mut self, delta: &DeltaMessage) {
        if let Some(content) = &delta.content {
            self.content
                .get_or_insert(String::from(""))
                .push_str(content);
        }

        if let Some(reasoning_content) = &delta.reasoning_content {
            self.reasoning_content
                .get_or_insert(String::from(""))
                .push_str(reasoning_content);
        }

        if let Some(tool_calls) = &delta.tool_calls {
            for item in tool_calls {
                let found = self
                    .tool_calls
                    .get_or_insert(Vec::new())
                    .iter_mut()
                    .find(|e| e.index == item.index);

                if let Some(found) = found {
                    found.function.arguments.push_str(&item.function.arguments);
                } else {
                    self.tool_calls.get_or_insert(Vec::new()).push(item.clone());
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeltaToolCall {
    index: u32,
    id: Option<String>,
    r#type: Option<String>,
    function: DeltaCallFunction,
}

impl DeltaToolCall {
    pub fn export(self) -> Result<ToolCall, AppError> {
        let result = ToolCall {
            id: self.id.ok_or(InternalError("No id provided".into()))?,
            r#type: self
                .r#type
                .ok_or(InternalError("No type provided".into()))?,
            function: self.function.export()?,
        };

        Ok(result)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeltaCallFunction {
    pub name: Option<String>,
    pub arguments: String,
}

impl DeltaCallFunction {
    pub fn export(self) -> Result<CallFunction, AppError> {
        let result = CallFunction {
            name: self.name.ok_or(InternalError("No name provided".into()))?,
            arguments: self.arguments,
        };

        Ok(result)
    }
}

impl ChoicesBuffer {
    fn merge(&mut self, delta_choice: &DeltaChoice) {
        let founded = self
            .choices
            .iter_mut()
            .find(|c| c.index == delta_choice.index);

        if let Some(founded) = founded {
            founded.finish_reason = delta_choice.finish_reason.clone();
            founded.delta.merge(&delta_choice.delta);
        } else {
            self.choices.push(delta_choice.clone());
        }
    }

    pub fn export(self) -> Result<Vec<Choice>, AppError> {
        let mut choices = Vec::new();
        for choice in self.choices {
            choices.push(choice.export()?);
        }
        Ok(choices)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseBuffer {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: ChoicesBuffer,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

impl ResponseBuffer {
    pub fn merge(&mut self, response: &ResponseBuffer) {
        let choices = &response.choices;
        let deltas: Vec<DeltaChoice> = choices.choices.clone();
        for delta in deltas {
            self.choices.merge(&delta);
        }
    }

    pub fn export(self) -> Result<Response, AppError> {
        let result = Response {
            id: self.id,
            object: self.object,
            created: self.created,
            model: self.model,
            choices: self.choices.export()?,
            usage: self.usage,
        };

        Ok(result)
    }
}
