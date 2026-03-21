pub mod bash;
pub mod cat;
pub mod control;
pub mod grep;
pub mod ls;
pub mod sed;
pub mod tail;

use crate::error::AppError;
use crate::tools::bash::Bash;
use crate::tools::cat::Cat;
use crate::tools::grep::Grep;
use crate::tools::ls::Ls;
use crate::tools::sed::Sed;
use crate::tools::tail::Tail;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, OnceLock};

#[derive(Serialize)]
pub struct Output {
    stdout: String,
    stderr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<i32>,
}

#[derive(Deserialize)]
struct Args {
    args: Vec<String>,
}

pub trait Tool: Sync + Send + Debug {
    fn new() -> Self
    where
        Self: Sized;
    fn call(&self, arguments: &str) -> Result<Output, AppError>;
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn parameters(&self) -> Value;
}

pub static TOOL_ROUTERS: OnceLock<HashMap<String, Arc<dyn Tool>>> = OnceLock::new();

#[ctor::ctor]
fn init() {
    TOOL_ROUTERS.get_or_init(|| {
        let bash = Bash::new();
        let cat = Cat::new();
        let grep = Grep::new();
        let tail = Tail::new();
        let ls = Ls::new();
        let sed = Sed::new();

        let mut map: HashMap<String, Arc<dyn Tool>> = HashMap::new();

        map.insert(bash.name(), Arc::new(bash));
        map.insert(cat.name(), Arc::new(cat));
        map.insert(tail.name(), Arc::new(tail));
        map.insert(grep.name(), Arc::new(grep));
        map.insert(ls.name(), Arc::new(ls));
        map.insert(sed.name(), Arc::new(sed));

        map
    });
}
