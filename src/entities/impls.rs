use crate::entities::service::AgentSettings;
use crate::entities::{FunctionDetail, MarkedMessage, Message, Request, Tool};
use crate::tools::TOOL_ROUTERS;

impl Request {
    pub fn from(setting: AgentSettings) -> Self {
        let system_prompt = setting
            .system_prompt
            .clone()
            .unwrap_or("You are a helpful assistant.".to_string());

        let system_message = Message::System {
            content: system_prompt,
            name: None,
        };

        let messages: Vec<MarkedMessage> = vec![MarkedMessage {
            id: 1,
            message: system_message,
        }];

        let tools = if let Some(tools) = setting.tools {
            if let Some(map) = TOOL_ROUTERS.get() {
                let mut results = vec![];
                for tool in tools {
                    let Some(got) = map.get(&tool) else {
                        continue;
                    };
                    let t = Tool {
                        r#type: "function".to_string(),
                        function: FunctionDetail {
                            name: got.name(),
                            description: got.description(),
                            parameters: got.parameters(),
                        },
                    };
                    results.push(t);
                }
                Some(results)
            } else {
                None
            }
        } else {
            None
        };

        Self {
            model: setting.model,
            messages,
            tools,
            tool_choice: setting.tool_choice,
            temperature: setting.temperature,
            stream: setting.stream,
            max_tokens: setting.max_tokens,
            enable_thinking: setting.thinking,
        }
    }

    pub fn push_message(&mut self, message: Message) {
        let marked = MarkedMessage {
            id: self.messages.len(),
            message,
        };
        self.messages.push(marked);
    }
}
