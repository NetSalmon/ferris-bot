use crate::error::AppError;
use crate::tools::{Args, Output, Tool};
use serde_json::{Value, json};
use std::process::Command;

#[derive(Debug)]
pub struct Sed {}

impl Tool for Sed {
    fn new() -> Self {
        Sed {}
    }

    fn call(&self, arguments: &str) -> Result<String, AppError> {
        let arguments: Vec<String> = serde_json::from_str::<Args>(arguments)?.args;
        let out = Command::new("sed").args(arguments).output()?;

        // 封装输出结果
        let output = Output {
            stdout: String::from_utf8(out.stdout)?,
            stderr: String::from_utf8(out.stderr)?,
            status: out.status.code(),
        };

        Ok(serde_json::to_string(&output)?)
    }

    fn name(&self) -> String {
        "sed".to_string()
    }

    fn description(&self) -> String {
        "linux sed - stream editor for filtering and transforming text".to_string()
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
                    "description": "sed command arguments array, e.g., ['s/old/new/g', 'file.txt'] to replace all occurrences of 'old' with 'new' in file, or ['-n', '5,10p', 'file.txt'] to print lines 5 to 10"
                }
            },
            "required": ["args"]
        })
    }
}
