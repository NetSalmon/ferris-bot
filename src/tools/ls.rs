use crate::error::AppError;
use crate::tools::{Args, Output, Tool};
use serde_json::{Value, json};
use std::process::Command;

#[derive(Debug)]
pub struct Ls {}

impl Tool for Ls {
    fn new() -> Self {
        Self {}
    }

    fn call(&self, arguments: &str) -> Result<String, AppError> {
        let args: Vec<String> = serde_json::from_str::<Args>(arguments)?.args;
        let output = Command::new("ls").args(args).output()?;

        let output = Output {
            stdout: String::from_utf8(output.stdout)?,
            stderr: String::from_utf8(output.stderr)?,
            status: output.status.code(),
        };

        Ok(serde_json::to_string(&output)?)
    }

    fn name(&self) -> String {
        "ls".to_string()
    }

    fn description(&self) -> String {
        "linux ls - list directory contents".to_string()
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
                    "description": "ls command arguments array, e.g., ['-l', '-a'] to list in long format including hidden files, or ['/path/to/dir'] to list specific directory"
                }
            },
            "required": ["args"]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ls() {
        let ls = Ls::new();
        println!("{}", ls.call(r#"["/tmp"]"#).unwrap());
    }
}
