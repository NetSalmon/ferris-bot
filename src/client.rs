pub mod batch;
pub mod stream;

use crate::error::AppError;
use reqwest::header::AUTHORIZATION;
use reqwest::header::{CONTENT_TYPE, HeaderValue};

pub struct Client {
    client: reqwest::Client,
    base_url: reqwest::Url,
}

pub struct ClientBuilder {
    headers: reqwest::header::HeaderMap,
    base_url: Option<reqwest::Url>,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        ClientBuilder {
            headers,
            base_url: None,
        }
    }
}

impl ClientBuilder {
    pub fn set_base_url(&mut self, base_url: &reqwest::Url) -> &mut Self {
        self.base_url = Some(base_url.clone());
        self
    }

    pub fn set_api_key(&mut self, api_key: &str) -> Result<&mut Self, AppError> {
        let key = format!("Bearer {}", api_key);
        self.headers
            .insert(AUTHORIZATION, HeaderValue::from_str(&key)?);
        Ok(self)
    }

    pub fn build(&mut self) -> Result<Client, AppError> {
        let client = reqwest::Client::builder()
            .default_headers(self.headers.clone())
            .build()?;

        if let Some(base_url) = self.base_url.clone() {
            Ok(Client { client, base_url })
        } else {
            Err(AppError::InternalError("No base URL provided".to_string()))
        }
    }
}
