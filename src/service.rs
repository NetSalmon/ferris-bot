mod handlers;

use crate::agent_manager::AgentManager;
use crate::entities::runtime::Env;
use crate::error::AppError;
use axum::routing::{get, post};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub struct AgentState {
    manager: Arc<AgentManager>,
}

pub async fn run(env: Env) -> Result<(), AppError> {
    let manager = AgentManager::from(env);

    let state = AgentState {
        manager: Arc::new(manager),
    };

    let router = axum::Router::new()
        .route(
            "/agent/{uuid}",
            post(handlers::input)
                .get(handlers::content)
                .delete(handlers::remove_agent),
        )
        .route("/agent/{uuid}/tool", post(handlers::tool_control))
        .route("/all", get(handlers::list_agent))
        .route("/create", get(handlers::create_agent))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(state));

    let addr = tokio::net::TcpListener::bind("0.0.0.0:11451").await?;

    axum::serve(addr, router).await?;

    Ok(())
}
