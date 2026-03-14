use crate::agent::{Agent, AgentHandle};
use crate::client::Client;
use crate::entities::runtime::Env;
use crate::entities::{Message, Request};
use crate::error::AppError;
use crate::tools::control::{ToolControl, ToolHandle};
use crate::tools::{bash, cat, get_weather, grep, ls, sed, tail, Tool};
use std::sync::Arc;
use dashmap::DashMap;
use uuid::Uuid;
use crate::EXIT;

pub struct AgentManager {
    pub handles: DashMap<Uuid, Arc<ServerHandle>>,
    pub env: Option<Env>,
}

pub struct ServerHandle {
    pub agent_handle: AgentHandle,
    pub tool_handle: ToolHandle,
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            handles: DashMap::new(),
            env: None,
        }
    }

    pub fn from(env: Env) -> Self {
        Self {
            handles: DashMap::new(),
            env: Some(env),
        }
    }

    pub fn input(&self, uuid: &Uuid, body: &str) -> Result<(), AppError> {
        let handle = self.handles.get(uuid)
            .ok_or(AppError::NotFound(uuid.to_string()))?;

        handle.agent_handle.input_tx.send(body.to_string())?;

        Ok(())
    }

    pub async fn create(&self) -> Result<Uuid, AppError> {
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
            if let Err(e) = agent.run().await {
                println!("Agent error {}", e);
            } else {
                println!("Agent exit");
            };
        });

        self.handles.insert(uuid.clone(), Arc::new(handle));

        Ok(uuid)
    }

    pub async fn remove(&self, uuid: Uuid) -> Result<(), AppError> {
        let handle = {
            match self.handles.remove(&uuid) {
                Some((_, h)) => h,
                None => return Err(AppError::NotFound(format!("Agent {}", uuid))),
            }
        };

        println!("Agent removing {}", uuid);
        let _ = handle.agent_handle.input_tx.send(EXIT.to_string());
        println!("Agent removed");
        Ok(())
    }
}