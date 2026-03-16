use crate::entities::stream::Chunk;
use crate::entities::{AgentTask, ApiResponse};
use axum::http::Error as HttpError;
use axum::http::{HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::response::Json;
use axum::Error as AxumError;
use reqwest::header::InvalidHeaderValue;
use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeJsonError;
use std::env::VarError;
use std::io::Error as IoError;
use std::string::FromUtf8Error;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::error::SendError;
use tokio::sync::mpsc::error::SendError as MpscSendError;
use tokio::sync::oneshot::error::RecvError as OneshotRecvError;
use tokio::task::JoinError;
use url::ParseError;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    IOError(#[from] IoError),
    #[error("JSON error: {0}")]
    JSONError(#[from] SerdeJsonError),
    #[error("Network error: {0}")]
    NetError(#[from] ReqwestError),
    #[error("Header value error: {0}")]
    HeaderValueError(#[from] InvalidHeaderValue),
    #[error("Broadcast send error: {0}")]
    BroadcastSendError(#[from] SendError<String>),
    #[error("Broadcast receive error: {0}")]
    BroadcastRecvError(#[from] RecvError),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] ParseError),
    #[error("Task join error: {0}")]
    JoinError(#[from] JoinError),
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] FromUtf8Error),
    #[error("Axum error: {0}")]
    AxumError(#[from] AxumError),
    #[error("HTTP error: {0}")]
    HttpError(#[from] HttpError),
    #[error("Environment variable error: {0}")]
    EnvError(#[from] VarError),
    #[error("Tool not found: {0}")]
    NoSuchToolError(String),
    #[error("Action not approved: {0}")]
    NoApprovementActionError(String),
    #[error("Chunk send error: {0}")]
    ChunkSendError(#[from] SendError<Chunk>),
    #[error("Agent task send error: {0}")]
    AgentTaskSendError(#[from] MpscSendError<AgentTask>),
    #[error("Oneshot receive error: {0}")]
    OneshotRecvError(#[from] OneshotRecvError),
    #[error("Tool control send error: {0}")]
    ToolControlSendError(String),
}

impl From<SendError<bool>> for AppError {
    fn from(err: SendError<bool>) -> Self {
        AppError::ToolControlSendError(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status_code, error_message) = match &self {
            AppError::HeaderValueError(_)
            | AppError::UrlParseError(_)
            | AppError::JSONError(_)
            | AppError::Utf8Error(_) => (StatusCode::BAD_REQUEST, self.to_string()),

            AppError::NotFound(_) | AppError::NoSuchToolError(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }

            AppError::NoApprovementActionError(_) => (StatusCode::FORBIDDEN, self.to_string()),

            AppError::InternalError(_)
            | AppError::IOError(_)
            | AppError::NetError(_)
            | AppError::BroadcastSendError(_)
            | AppError::BroadcastRecvError(_)
            | AppError::JoinError(_)
            | AppError::AxumError(_)
            | AppError::HttpError(_)
            | AppError::ChunkSendError(_)
            | AppError::AgentTaskSendError(_)
            | AppError::OneshotRecvError(_)
            | AppError::EnvError(_)
            | AppError::ToolControlSendError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(ApiResponse::err(error_message));

        (
            status_code,
            [("Content-Type", HeaderValue::from_static("application/json"))],
            body,
        )
            .into_response()
    }
}
