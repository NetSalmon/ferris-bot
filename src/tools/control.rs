use crate::entities::stream::Chunk;
use crate::entities::{FunctionDetail, Message};
use crate::error::AppError;
use crate::tools::Tool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};

pub struct ToolControl {
    pub tools: Vec<Arc<dyn Tool>>,
    pub tool_entities: Vec<crate::entities::Tool>,
    pub tool_router: HashMap<String, Arc<dyn Tool>>,
    pub output_tx: Sender<Chunk>,
    pub control_rx: Receiver<bool>,
}

impl ToolControl {
    pub fn new(control_tx: Sender<bool>, output_tx: Sender<Chunk>) -> Self {
        let control = Self {
            tools: vec![],
            tool_entities: vec![],
            tool_router: HashMap::new(),
            output_tx,
            control_rx: control_tx.subscribe(),
        };

        control
    }

    pub fn add_tool(&mut self, tool: Arc<dyn Tool>) {
        self.tool_router.insert(tool.name(), tool.clone());
        self.tools.push(tool.clone());

        let entity = crate::entities::Tool {
            r#type: "function".to_string(),
            function: FunctionDetail {
                name: tool.name(),
                description: tool.description(),
                parameters: tool.parameters(),
            },
        };

        self.tool_entities.push(entity);
    }

    pub fn add_tools(&mut self, tools: &[Arc<dyn Tool>]) {
        for tool in tools {
            self.add_tool(Arc::clone(tool));
        }
    }

    pub async fn call(
        &mut self,
        name: &str,
        arguments: &str,
        id: &str,
    ) -> Result<Message, AppError> {
        let content = Chunk::ToolCall {
            name: name.to_string(),
            arguments: arguments.to_string(),
        };

        self.output_tx
            .send(content)
            .map_err(|err| AppError::InternalError(err.to_string()))?;

        if let Ok(approvement) = self.control_rx.recv().await
            && approvement
        {
            let Some(tool) = self.tool_router.get(name) else {
                return Err(AppError::NoSuchToolError(name.to_string()));
            };

            let result = tool.call(arguments)?;

            let string = serde_json::to_string(&result)?;

            let chunk = Chunk::ToolOutput {
                stdout: result.stdout,
                stderr: result.stderr,
                status: result.status,
            };

            self.output_tx
                .send(chunk)
                .map_err(|err| AppError::InternalError(err.to_string()))?;

            let msg = Message::Tool {
                tool_call_id: id.to_string(),
                content: string,
            };

            Ok(msg)
        } else {
            Err(AppError::NoApprovementActionError(name.to_string()))
        }
    }
}
