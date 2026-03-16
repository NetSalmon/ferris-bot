use crate::entities::stream::Chunk;
use crate::entities::{AgentTask, ApiResponse};
use crate::error::AppError;
use crate::error::AppError::NotFound;
use crate::service::AgentState;
use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive};
use axum::response::{Json, Sse};
use futures_util::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub async fn list_agent(
    State(state): State<Arc<AgentState>>,
) -> Result<Json<ApiResponse<Vec<Uuid>>>, AppError> {
    let result = state
        .manager
        .handles
        .iter()
        .map(|entry| *entry.key())
        .collect::<Vec<_>>();
    Ok(Json(ApiResponse::ok(result)))
}

pub async fn create_agent(
    State(state): State<Arc<AgentState>>,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let uuid = state.manager.create().await?;
    Ok(Json(ApiResponse::ok(uuid.to_string())))
}

pub async fn input(
    State(state): State<Arc<AgentState>>,
    Path(uuid): Path<Uuid>,
    body: String,
) -> Result<Json<ApiResponse<String>>, AppError> {
    let message = match state.manager.input(&uuid, &body).await {
        Ok(_) => "Message sent to agent",
        Err(_) => "Agent is not listening",
    };

    Ok(Json(ApiResponse::ok(message.to_string())))
}

pub async fn content(
    State(state): State<Arc<AgentState>>,
    Path(uuid): Path<Uuid>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let Some(handle) = state.manager.handles.get(&uuid) else {
        return Err(NotFound(uuid.to_string()));
    };
    let mut o_rx = handle.output_tx.subscribe();

    let stream = async_stream::stream! {
        loop {
            let Ok(res) = o_rx.recv().await else {
                continue;
            };

            let body = match res {
                Chunk::EventEnd => "[DONE]".to_string(),
                _ => {
                    let Ok(ret) = serde_json::to_string(&res) else {
                        continue;
                    };
                    ret
                },
            };

            yield Ok(Event::default().event("data").data(body));
        }
    };

    let sse = Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    );

    Ok(sse)
}

pub async fn tool_control(
    State(state): State<Arc<AgentState>>,
    Path(uuid): Path<Uuid>,
    body: Json<bool>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let state = Arc::clone(&state);
    let Some(handle) = state.manager.handles.get(&uuid) else {
        return Err(NotFound(uuid.to_string()));
    };
    let approve = *body;
    handle.control_tx.send(approve)?;
    Ok(Json(ApiResponse::ok(())))
}

pub async fn remove_agent(
    State(state): State<Arc<AgentState>>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    println!("Receive remove request {}", uuid.to_string());
    let state = Arc::clone(&state);
    state.manager.remove(uuid).await?;

    Ok(Json(ApiResponse::ok(())))
}

pub async fn get_messages(
    State(state): State<Arc<AgentState>>,
    Path(uuid): Path<Uuid>,
) -> Result<Json<ApiResponse<Chunk>>, AppError> {
    let state = Arc::clone(&state);
    let Some(t) = state.manager.handles.get(&uuid) else {
        return Err(NotFound(uuid.to_string()));
    };

    let (tx, rx) = tokio::sync::oneshot::channel::<Chunk>();

    t.input_tx
        .send(AgentTask::MessageRequest { channel: tx })
        .await?;

    let result = rx.await?;

    Ok(Json(ApiResponse::ok(result)))
}