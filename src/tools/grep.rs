use crate::error::AppError;
use crate::tools::{Args, Output, Tool};
use serde_json::{Value, json};
use std::process::Command;

#[derive(Debug)]
pub struct Grep {}

impl Tool for Grep {
    fn new() -> Self {
        Grep {}
    }

    fn call(&self, arguments: &str) -> Result<Output, AppError> {
        let arguments: Vec<String> = serde_json::from_str::<Args>(arguments)?.args;

        let out = Command::new("grep").args(arguments).output()?;

        let output = Output {
            stdout: String::from_utf8(out.stdout)?,
            stderr: String::from_utf8(out.stderr)?,
            status: out.status.code(),
        };

        Ok(output)
    }

    fn name(&self) -> String {
        "grep".to_string()
    }

    fn description(&self) -> String {
        "linux grep - print lines that match patterns".to_string()
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "args": {
                    "type": "array",
                    "items": {
                        "type": "string"
                    },
                    "description": "grep command arguments array, e.g., ['-i', 'pattern', 'file.txt'] to search for pattern case-insensitively in file, or ['-r', 'error', '.'] to recursively search in current directory"
                }
            },
            "required": ["args"]
        })
    }
}
