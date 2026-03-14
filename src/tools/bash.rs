use crate::error::AppError;
use crate::tools::{Output, Tool};
use serde_json::{json, Value};
use std::process::Command;

#[derive(Debug)]
pub struct Bash {}

impl Tool for Bash {
    fn new() -> Self {
        Self {}
    }

    fn call(&self, arguments: &str) -> Result<String, AppError> {
        let args = serde_json::from_str::<Value>(arguments)?;
        let script = args["script"].as_str().unwrap_or("echo \"ERROR\"");

        let output = Command::new("bash")
            .args(&["-c", script])
            .output()?;

        let output = Output {
            stdout: String::from_utf8(output.stdout)?,
            stderr: String::from_utf8(output.stderr)?,
            status: output.status.code(),
        };

        let output = serde_json::to_string(&output)?;

        println!("{}", output);

        Ok(output)
    }

    fn name(&self) -> String {
        String::from("bash")
    }

    fn description(&self) -> String {
        String::from("linux bash - GNU Bourne-Again SHell, execute commands or scripts")
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "script": {
                    "type": "string",
                    "description": "Bash script or command to execute, e.g., \"ls -la\" or \"echo $HOME\""
                }
            },
            "required": ["script"]
        })
    }
}