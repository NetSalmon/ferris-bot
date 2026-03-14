use crate::error::AppError;
use crate::tools::Tool;
use serde_json::{Value, json};

#[derive(Debug)]
pub struct GetWeather {}

impl Tool for GetWeather {
    fn new() -> Self {
        Self {}
    }

    fn call(&self, arguments: &str) -> Result<String, AppError> {
        let args = serde_json::from_str::<Value>(arguments)?;
        let city = args["city"].as_str().unwrap_or("Unknown");

        Ok(format!("City: {}, weather: Sunny", city))
    }

    fn name(&self) -> String {
        String::from("get_weather")
    }

    fn description(&self) -> String {
        String::from("Get weather information for a given city")
    }

    fn parameters(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "City name to get weather for, e.g., \"Beijing\" or \"New York\""
                }
            },
            "required": ["city"]
        })
    }
}
