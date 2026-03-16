use crate::client::Client;
use crate::entities::stream::Chunk;
use crate::entities::stream::Chunk::Messages;
use crate::entities::{AgentTask, Message, Request};
use crate::error::AppError;
use crate::tools::control::ToolControl;
use crate::EXIT;
use tokio::sync::broadcast::Sender;

pub struct Agent {
    client: Client,
    request: Request,
    input_rx: tokio::sync::mpsc::Receiver<AgentTask>,
    output_tx: Sender<Chunk>,
    tool_control: ToolControl,
}

impl Agent {
    pub fn new(
        client: Client,
        request: Request,
        output_tx: Sender<Chunk>,
        input_rx: tokio::sync::mpsc::Receiver<AgentTask>,
        tool_control: ToolControl,
    ) -> Agent {
        let tools = tool_control.tool_entities.clone();

        let mut agent = Agent {
            client,
            request,
            output_tx: output_tx.clone(),
            input_rx,
            tool_control,
        };

        match &mut agent.request.tools {
            Some(inside_tools) => {
                for tool in tools {
                    inside_tools.push(tool);
                }
            }
            None => {
                agent.request.tools = Some(tools);
            }
        }

        agent
    }

    pub async fn run(&mut self) -> Result<(), AppError> {
        let mut in_feedback = false;

        loop {
            if !in_feedback {
                let input = loop {
                    let Some(input) = self.input_rx.recv().await else {
                        continue;
                    };
                    match input {
                        AgentTask::Input {content} => {
                            break content;
                        }
                        AgentTask::MessageRequest { channel } => {
                            let chunk = Messages {
                                messages: self.request.messages.clone(),
                            };
                            // 如果发送失败，说明接收端已关闭，忽略即可
                            let _ = channel.send(chunk);
                        }
                    }
                };
                if input.to_lowercase().trim() == EXIT {
                    break;
                }
                let new_message = Message::User {
                    content: input.to_string(),
                    name: None,
                };

                self.request.push_message(new_message);
            }

            let response = if self.request.stream == Some(true) {
                use crate::client::stream::StreamAPI;
                StreamAPI::send(&mut self.client, &self.output_tx, &self.request).await?
            } else {
                use crate::client::batch::BatchAPI;
                BatchAPI::send(&mut self.client, &self.output_tx, &self.request).await?
            };

            for choice in &response.choices {
                self.request.push_message(choice.message.clone());
            }

            for choice in &response.choices {
                let Message::Assistant { tool_calls, .. } = choice.message.clone() else {
                    in_feedback = false;
                    continue;
                };

                let Some(tool_calls) = tool_calls.filter(|c| !c.is_empty()) else {
                    in_feedback = false;
                    continue;
                };

                if tool_calls.len() == 0 {
                    in_feedback = false;
                    continue;
                }

                in_feedback = true;
                for tool_call in tool_calls {
                    let result = self
                        .tool_control
                        .call(
                            &tool_call.function.name,
                            &tool_call.function.arguments,
                            &tool_call.id,
                        )
                        .await;

                    let message = match result {
                        Ok(message) => message,
                        Err(e) => Message::Tool {
                            content: format!("Tool Call Error: {}", e.to_string()),
                            tool_call_id: tool_call.id,
                        },
                    };

                    self.request.push_message(message);
                }
            }
        }

        Ok(())
    }
}
