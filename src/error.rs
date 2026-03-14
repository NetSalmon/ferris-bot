use std::env::VarError;
use std::io::Error as IoError;
use serde_json::Error as SerdeJsonError;
use reqwest::Error as ReqwestError;
use reqwest::header::InvalidHeaderValue;
use tokio::sync::broadcast::error::SendError;
use tokio::sync::broadcast::error::RecvError;
use url::ParseError;
use tokio::task::JoinError;
use std::string::FromUtf8Error;
use axum::Error as AxumError;
use axum::http::Error as HttpError;
use axum::response::IntoResponse;
use axum::http::{StatusCode, HeaderValue};
use axum::response::Json;
use serde::Serialize;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("internal error: {0}")]
    InternalError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    IOError(#[from] IoError),
    #[error("JSON error: {0}")]
    JSONError(#[from] SerdeJsonError),
    #[error("Net error: {0}")]
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
    #[error("No such environment variable {0}")]
    EnvError(#[from] VarError),
    #[error("No such tool: {0}")]
    NoSuchToolError(String),
    #[error("Use disallow this action: {0}")]
    NoApprovementActionError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status_code, error_message) = match &self {
            AppError::HeaderValueError(_) |
            AppError::UrlParseError(_) |
            AppError::JSONError(_) |
            AppError::Utf8Error(_) => (
                StatusCode::BAD_REQUEST,
                self.to_string()
            ),

            AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),

            AppError::NoSuchToolError(_) => (
                StatusCode::NOT_FOUND,
                self.to_string()
            ),

            AppError::NoApprovementActionError(_) => (
                StatusCode::FORBIDDEN,
                self.to_string()
            ),

            AppError::InternalError(_) |
            AppError::IOError(_) |
            AppError::NetError(_) |
            AppError::BroadcastSendError(_) |
            AppError::BroadcastRecvError(_) |
            AppError::JoinError(_) |
            AppError::AxumError(_) |
            AppError::HttpError(_) |
            AppError::EnvError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                self.to_string()
            ),
        };

        let body = Json(ErrorResponse {
            error: status_code.to_string(),
            message: error_message,
        });

        (
            status_code,
            [("Content-Type", HeaderValue::from_static("application/json"))],
            body,
        ).into_response()
    }
}