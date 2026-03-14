use crate::entities::{FunctionDetail, Message};
use crate::error::AppError;
use crate::tools::Tool;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};

pub struct ToolControl {
    pub tools: Vec<Arc<dyn Tool>>,
    pub tool_entities: Vec<crate::entities::Tool>,
    pub tool_router: HashMap<String, Arc<dyn Tool>>,
    pub control_rx: Receiver<bool>,
    pub output_tx: Sender<ToolContent>,
}

pub struct ToolHandle {
    pub control_tx: Sender<bool>,
    pub output_rx: Receiver<ToolContent>,
}

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ToolContent {
    Calling{name: String, arguments: String },
    Output(String),
}

impl ToolControl {
    pub fn new() -> (Self, ToolHandle) {
        let (control_tx, control_rx) = tokio::sync::broadcast::channel::<bool>(1024);
        let (output_tx, output_rx) = tokio::sync::broadcast::channel::<ToolContent>(1024);
        
        let control = Self {
            tools: vec![],
            tool_entities: vec![],
            tool_router: HashMap::new(),
            control_rx,
            output_tx,
        };
        
        let handle = ToolHandle {
            control_tx,
            output_rx,
        };

        (control, handle)
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
            }
        };
        
        self.tool_entities.push(entity);
    }
    
    pub fn add_tools(&mut self, tools: &[Arc<dyn Tool>]) {
        for tool in tools {
            self.add_tool(Arc::clone(tool));
        }
    }
    
    pub async fn call(&mut self, name: &str, arguments: &str, id: &str) -> Result<Message, AppError> {
        let content = ToolContent::Calling {
            name: name.to_string(),
            arguments: arguments.to_string(),
        };
        
        self.output_tx.send(content).map_err(|err| AppError::InternalError(err.to_string()))?;
        
        if let Ok(approvement) = self.control_rx.recv().await && approvement{
            let Some(tool) = self.tool_router.get(name) else {
                return Err(AppError::NoSuchToolError(name.to_string()));
            };
            
            let result = tool.call(arguments)?;
            
            self.output_tx.send(ToolContent::Output(result.clone())).map_err(|err| AppError::InternalError(err.to_string()))?;
            
            let msg = Message::Tool {
                tool_call_id: id.to_string(),
                content: result,
            };
            
            Ok(msg)
        } else {
            Err(AppError::NoApprovementActionError(name.to_string()))
        }
    }
}