mod handlers;

use crate::agent::Agent;
use crate::client::Client;
use crate::entities::runtime::Env;
use crate::entities::{Message, Request};
use crate::error::AppError;
use crate::tools::control::{ToolContent, ToolControl};
use crate::tools::{bash, cat, get_weather, grep, ls, sed, tail, Tool};
use axum::routing::post;
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

pub struct AgentState {
    content_rx: Mutex<Receiver<String>>,
    reason_rx: Mutex<Receiver<String>>,
    input_tx: Sender<String>,
    tool_output_rx: Mutex<Receiver<ToolContent>>,
    tool_control_tx: Sender<bool>,
}

pub async fn run(env: Env) -> Result<(), AppError> {
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

    let (mut tool_control, tool_handle) = ToolControl::new();
    tool_control.add_tools(
        &[
            Arc::new(bash::Bash::new()),
            Arc::new(get_weather::GetWeather::new()),
            Arc::new(ls::Ls::new()),
            Arc::new(tail::Tail::new()),
            Arc::new(sed::Sed::new()),
            Arc::new(grep::Grep::new()),
            Arc::new(cat::Cat::new()),
        ]
    );

    let (mut agent, handle) = Agent::new(
        client,
        request,
        tool_control,
    );

    let state = AgentState {
        content_rx: Mutex::new(handle.content_rx),
        reason_rx: Mutex::new(handle.reason_rx),
        input_tx: handle.input_tx,
        tool_control_tx: tool_handle.control_tx,
        tool_output_rx: Mutex::new(tool_handle.output_rx),
    };

    let router = axum::Router::new()
        .route("/agent", post(handlers::input).get(handlers::content))
        .route("/agent/tool", post(handlers::tool_control).get(handlers::tool_output))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state));

    let addr = tokio::net::TcpListener::bind("0.0.0.0:11451").await?;
    tokio::spawn(async move {
        if let Err(e) = agent.run().await {
            eprintln!("Agent error: {}", e);
        }
    });
    
    axum::serve(addr, router).await?;
    
    Ok(())
}