use crate::client::Client;
use crate::entities::{Message, Request};
use crate::error::AppError;
use crate::tools::control::ToolControl;
use tokio::sync::broadcast::{Receiver, Sender};
use crate::EXIT;

pub struct Agent {
    client: Client,
    request: Request,
    content_tx: Sender<String>,
    reason_tx: Sender<String>,
    input_rx: Receiver<String>,
    tool_control: ToolControl,
}

pub struct AgentHandle {
    pub input_tx: Sender<String>,
    pub content_rx: Receiver<String>,
    pub reason_rx: Receiver<String>,
}

impl Agent {
    pub fn new(client: Client, request: Request, tool_control: ToolControl) -> (Agent, AgentHandle) {
        let (content_tx, content_rx) = tokio::sync::broadcast::channel(1024);
        let (reason_tx, reason_rx) = tokio::sync::broadcast::channel(1024);
        let (input_tx, input_rx) = tokio::sync::broadcast::channel(1024);

        let tools = tool_control.tool_entities.clone();

        let mut agent = Agent {
            client,
            request,
            content_tx,
            reason_tx,
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

        let handle = AgentHandle {
            input_tx,
            content_rx,
            reason_rx,
        };

        (agent, handle)
    }

    pub async fn kill(&self, input_tx: Sender<String>) -> Result<(), AppError> {
        input_tx.send(EXIT.to_string())?;

        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), AppError> {
        let mut in_feedback = false;

        loop {
            if !in_feedback {
                let input = tokio::select! {
                    Ok(i) = self.input_rx.recv() => i,
                    else => break,
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
                StreamAPI::send(&mut self.client, &self.content_tx, &self.reason_tx, &self.request).await?
            } else {
                use crate::client::batch::BatchAPI;
                BatchAPI::send(&mut self.client, &self.content_tx, &self.reason_tx, &self.request).await?
            };

            for choice in &response.choices {
                self.request.push_message(choice.message.clone());
            }

            for choice in &response.choices {
                let Message::Assistant { tool_calls, .. } = choice.message.clone() else {
                    in_feedback = false;
                    continue
                };

                let Some(tool_calls) = tool_calls.filter(|c| !c.is_empty()) else {
                    in_feedback = false;
                    continue
                };

                if tool_calls.len() == 0 {
                    in_feedback = false;
                    continue;
                }

                in_feedback = true;
                for tool_call in tool_calls {
                    let result = self.tool_control.call(
                        &tool_call.function.name,
                        &tool_call.function.arguments,
                        &tool_call.id
                    ).await;

                    let message = match result {
                        Ok(message) => message,
                        Err(e) => {
                            Message::Tool {
                                content: format!("Tool Call Error: {}", e.to_string()),
                                tool_call_id: tool_call.id,
                            }
                        }
                    };

                    self.request.push_message(message);
                }
            }
        }

        Ok(())
    }
}