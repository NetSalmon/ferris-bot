use crate::agent::{Agent, AgentHandle};
use crate::client::Client;
use crate::entities::runtime::Env;
use crate::entities::{Message, Request};
use crate::error::AppError;
use crate::tools::control::{ToolControl, ToolHandle};
use crate::tools::{bash, cat, get_weather, grep, ls, sed, tail, Tool};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use crate::EXIT;

pub struct AgentManager {
    pub handles: HashMap<Uuid, Arc<ServerHandle>>,
    pub env: Option<Env>,
}

pub struct ServerHandle {
    pub agent_handle: AgentHandle,
    pub tool_handle: ToolHandle,
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
            env: None,
        }
    }

    pub fn from(env: Env) -> Self {
        Self {
            handles: HashMap::new(),
            env: Some(env),
        }
    }

    pub async fn create(&mut self) -> Result<Uuid, AppError> {
        let (mut tool_control, tool_handle) = ToolControl::new();
        tool_control.add_tools(&[
            Arc::new(bash::Bash::new()),
            Arc::new(get_weather::GetWeather::new()),
            Arc::new(ls::Ls::new()),
            Arc::new(tail::Tail::new()),
            Arc::new(sed::Sed::new()),
            Arc::new(grep::Grep::new()),
            Arc::new(cat::Cat::new()),
        ]);

        let Some(env) = &self.env else {
            return Err(AppError::InternalError("No env provide".to_string()))
        };

        let client = Client::builder()
            .set_base_url(&env.base_url)
            .set_api_key(&env.api_key)?
            .build()?;

        let request = Request::builder()
            .set_model(&env.model)
            .push_message(Message::System {
                content: "You are an helpful assistant".to_string(),
                name: None,
            })
            .enable_stream()
            .enable_thinking()
            .build();

        let (mut agent, agent_handle) = Agent::new(client, request, tool_control);

        let handle = ServerHandle {
            agent_handle,
            tool_handle,
        };

        let uuid = Uuid::new_v4();

        tokio::spawn(async move {
            if let Ok(()) = agent.run().await {
                println!("Agent exit");
            } else {
                println!("Agent error");
            };
        });

        self.handles.insert(uuid.clone(), Arc::new(handle));

        Ok(uuid)
    }

    pub async fn remove(&mut self, uuid: Uuid) -> Result<(), AppError> {
        if let Some(handle) = self.handles.remove(&uuid) {
            let _ = handle.agent_handle.input_tx.send(EXIT.to_string());
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Agent {}", uuid)))
        }
    }
}