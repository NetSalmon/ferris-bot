use crate::error::AppError;
use crate::error::AppError::{InternalError, NotFound};
use crate::service::AgentState;
use crate::tools::control::ToolContent;
use axum::extract::{Path, State};
use axum::response::Sse;
use axum::response::sse::{Event, KeepAlive};
use futures_util::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

pub async fn list_agent(State(state): State<Arc<AgentState>>) -> Result<String, AppError> {
    let result = serde_json::to_string(
        &state
            .manager
            .handles
            .iter()
            .map(|entry| *entry.key())
            .collect::<Vec<_>>(),
    )?;
    Ok(result)
}

pub async fn create_agent(State(state): State<Arc<AgentState>>) -> Result<String, AppError> {
    let uuid = state.manager.create().await?;
    Ok(uuid.to_string())
}

pub async fn input(
    State(state): State<Arc<AgentState>>,
    Path(uuid): Path<Uuid>,
    body: String,
) -> String {
    match state.manager.input(&uuid, &body) {
        Ok(_) => "Message sent to agent".to_string(),
        Err(_) => "Agent is not listening".to_string(),
    }
}

pub async fn content(
    State(state): State<Arc<AgentState>>,
    Path(uuid): Path<Uuid>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let Some(handle) = state.manager.handles.get(&uuid) else {
        return Err(NotFound(uuid.to_string()));
    };
    let mut c_rx = handle.agent_handle.content_tx.subscribe();
    let mut r_rx = handle.agent_handle.reason_tx.subscribe();
    // Release the DashMap lock before entering the stream

    let stream = async_stream::stream! {
        loop {
            let res = tokio::select! {
                res = r_rx.recv() => Some(("reason", res)),
                res = c_rx.recv() => Some(("content", res)),
                _ = tokio::time::sleep(Duration::from_secs(30)) => None,
            };

            match res {
                Some((event_type, Ok(data))) => {
                    yield Ok(Event::default().event(event_type).data(data));
                }
                Some((_, Err(RecvError::Lagged(_)))) => continue,
                _ => break,
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

pub async fn tool_output(
    State(state): State<Arc<AgentState>>,
    Path(uuid): Path<Uuid>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let Some(handle) = state.manager.handles.get(&uuid) else {
        return Err(NotFound(uuid.to_string()));
    };
    let mut output_rx = handle.tool_handle.output_tx.subscribe();
    // Release the DashMap lock before entering the stream

    let stream = async_stream::stream! {
        loop {
            match output_rx.recv().await {
                Ok(result) => {
                    let event_type = match result {
                        ToolContent::Output(_) => "output",
                        ToolContent::Calling{..} => "calling"
                    };

                    let Ok(result) = serde_json::to_string(&result) else {
                        continue;
                    };

                    yield Ok(Event::default().event(event_type).data(result))
                }
                Err(RecvError::Lagged(_)) => continue,
                Err(_) => break,
            }
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
    body: axum::Json<bool>,
) -> Result<(), AppError> {
    let state = Arc::clone(&state);
    let Some(handle) = state.manager.handles.get(&uuid) else {
        return Err(NotFound(uuid.to_string()));
    };
    let approve = *body;
    handle
        .tool_handle
        .control_tx
        .send(approve)
        .map_err(|e| InternalError(format!("Failed to send tool control: {}", e)))?;
    Ok(())
}

pub async fn remove_agent(
    State(state): State<Arc<AgentState>>,
    Path(uuid): Path<Uuid>,
) -> Result<(), AppError> {
    println!("Receive remove request {}", uuid.to_string());
    let state = Arc::clone(&state);
    state.manager.remove(uuid).await?;

    Ok(())
}
