use crate::error::AppError;
use crate::error::AppError::InternalError;
use crate::service::AgentState;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive};
use axum::response::Sse;
use futures_util::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use crate::tools::control::ToolContent;

pub async fn input(
    State(state): State<Arc<AgentState>>,
    body: String,
) -> String {
    match state.input_tx.send(body) {
        Ok(_) => "Message sent to agent".to_string(),
        Err(_) => "Agent is not listening".to_string(),
    }
}
pub async fn content(
    State(state): State<Arc<AgentState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let state = Arc::clone(&state);

    let stream = async_stream::stream! {
        loop {
            let mut c_rx = state.content_rx.lock().await;
            let mut r_rx = state.reason_rx.lock().await;

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
                Some((_, Err(_))) | None => break,
            }
        }
    };

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

pub async fn tool_output(
    State(state): State<Arc<AgentState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        loop {
            let mut lock = state.tool_output_rx.lock().await;

            match lock.recv().await {
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

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

pub async fn tool_control(
    State(state): State<Arc<AgentState>>,
    body: axum::Json<bool>,
) -> Result<(), AppError> {
    let approve = *body;
    state.tool_control_tx.send(approve)
        .map_err(|e| InternalError(format!("Failed to send tool control: {}", e)))?;
    Ok(())
}