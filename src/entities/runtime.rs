use url::Url;

pub struct Env {
    pub base_url: Url,
    pub api_key: String,
    pub model: String,
}