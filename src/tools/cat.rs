use crate::error::AppError;
use crate::tools::{Args, Output, Tool};
use serde_json::{Value, json};
use std::process::Command;

#[derive(Debug)]
pub struct Cat {}

impl Tool for Cat {
    fn new() -> Self {
        Cat {}
    }

    fn call(&self, arguments: &str) -> Result<Output, AppError> {
        let arguments: Vec<String> = serde_json::from_str::<Args>(arguments)?.args;
        let out = Command::new("cat").args(arguments).output()?;

        let output = Output {
            stdout: String::from_utf8(out.stdout)?,
            stderr: String::from_utf8(out.stderr)?,
            status: out.status.code(),
        };

        Ok(output)
    }

    fn name(&self) -> String {
        "cat".to_string()
    }

    fn description(&self) -> String {
        "linux cat - concatenate files and print on the standard output".to_string()
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
                    "description": "cat command arguments array, e.g., ['file.txt'] to display file contents, or ['-n', 'file.txt'] to number all output lines"
                }
            },
            "required": ["args"]
        })
    }
}
