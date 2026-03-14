use crate::error::AppError;
use crate::tools::{Args, Output, Tool};
use serde_json::{json, Value};
use std::process::Command;

#[derive(Debug)]
pub struct Tail {}

impl Tool for Tail {
    fn new() -> Self {
        Tail {}
    }

    fn call(&self, arguments: &str) -> Result<String, AppError> {
        // 解析传入的参数数组
        let arguments: Vec<String> = serde_json::from_str::<Args>(arguments)?.args;
        let out = Command::new("tail")
            .args(arguments)
            .output()?;

        // 封装输出结果
        let output = Output {
            stdout: String::from_utf8(out.stdout)?,
            stderr: String::from_utf8(out.stderr)?,
            status: out.status.code(),
        };

        Ok(serde_json::to_string(&output)?)
    }

    fn name(&self) -> String {
        "tail".to_string()
    }

    fn description(&self) -> String {
        "linux tail - output the last part of files".to_string()
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
                    "description": "tail command arguments array, e.g., ['-n', '10', 'file.txt'] to output the last 10 lines of file, or ['-f', 'log.txt'] to follow file updates"
                }
            },
            "required": ["args"]
        })
    }
}
