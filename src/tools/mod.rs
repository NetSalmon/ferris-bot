pub mod control;
pub mod bash;
pub mod get_weather;
pub mod ls;
pub mod cat;
pub mod tail;
pub mod sed;
pub mod grep;

use crate::error::AppError;
use serde_json::Value;
use std::fmt::Debug;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Output {
    stdout: String,
    stderr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<i32>,
}

#[derive(Deserialize)]
struct Args {
    args: Vec<String>,
}

pub trait Tool : Sync + Send + Debug {
    fn new() -> Self where Self: Sized;
    fn call(&self, arguments: &str) -> Result<String, AppError>;
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn parameters(&self) -> Value;
}