use crate::client::Client;
use crate::entities::stream::ResponseBuffer;
use crate::entities::{Request, Response};
use futures_util::StreamExt;
use tokio::sync::broadcast::Sender;
use crate::error::AppError;
use crate::error::AppError::InternalError;

pub trait StreamAPI {
    async fn send(&mut self, content: &Sender<String>, reason: &Sender<String>, request: &Request) -> Result<Response, AppError>;
}

impl StreamAPI for Client {
    async fn send(&mut self, content: &Sender<String>, reason: &Sender<String>, request: &Request) -> Result<Response, AppError> {
        let mut stream = self.client.post(self.base_url.clone())
            .body(serde_json::to_string(&request)?)
            .send()
            .await?
            .bytes_stream();

        let mut result: Option<ResponseBuffer> = None;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let chunk_str = String::from_utf8_lossy(&chunk);

            for line in chunk_str.lines() {
                let line = line.trim();

                if line.is_empty() { continue; }

                if let Some(value) = line.strip_prefix("data:") {
                    let value = value.trim();

                    if value == "[DONE]" {
                        let Some(result) = result else {
                            return Err(InternalError("No data".to_string()))
                        };

                        return result.export();
                    }

                    match serde_json::from_str::<ResponseBuffer>(value) {
                        Ok(data) => {
                            let (got_content, got_reason) = data.choices.choices
                                .iter()
                                .fold((String::new(), String::new()), |mut acc, item| {
                                    if let Some(s) = &item.delta.content {
                                        acc.0.push_str(s);
                                    }
                                    if let Some(s) = &item.delta.reasoning_content {
                                        acc.1.push_str(s);
                                    }
                                    acc
                                });

                            if !got_content.is_empty() {
                                content.send(got_content)?;
                            }
                            if !got_reason.is_empty() {
                                reason.send(got_reason)?;
                            }

                            if let Some(res) = &mut result {
                                res.merge(&data);
                            } else {
                                result = Some(data);
                            }
                        }
                        Err(e) => {
                            println!("JSON Parse Error: {}, Content: '{}'", e, value);
                            continue;
                        }
                    }
                }
            }
        }

        let Some(result) = result else {
            return Err(InternalError("No data".to_string()))
        };

        result.export()
    }
}