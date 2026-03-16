use crate::client::Client;
use crate::entities::stream::Chunk;
use crate::entities::{Message, Request, Response};
use crate::error::AppError;
use tokio::sync::broadcast::Sender;

pub trait BatchAPI {
    async fn send(
        &mut self,
        output: &Sender<Chunk>,
        request: &Request,
    ) -> Result<Response, AppError>;
}

impl BatchAPI for Client {
    async fn send(
        &mut self,
        output: &Sender<Chunk>,
        request: &Request,
    ) -> Result<Response, AppError> {
        if let Ok(json) = serde_json::to_string_pretty(&request) {
            println!("{}", json);
        }
        let text = self
            .client
            .post(self.base_url.clone())
            .body(serde_json::to_string(&request)?)
            .send()
            .await?
            .text()
            .await?;

        let result: Response = serde_json::from_str(&text)?;
        let (got_content, got_reason) = result.choices.iter().fold(
            (String::new(), String::new()),
            |(mut temp_content, mut temp_reason), choice| {
                match &choice.message {
                    Message::Assistant {
                        content,
                        reasoning_content,
                        ..
                    } => {
                        let content = content.clone().unwrap_or("".into());
                        let reasoning_content = reasoning_content.clone().unwrap_or("".into());
                        temp_content.push_str(&content);
                        temp_reason.push_str(&reasoning_content);
                    }
                    _ => (),
                }

                (temp_content, temp_reason)
            },
        );

        let chunk = Chunk::Block {
            reason: got_reason,
            content: got_content,
        };

        output.send(chunk)?;

        Ok(result)
    }
}
