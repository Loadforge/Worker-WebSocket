use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct DslConfig {
    pub name: String,
    pub target: String,
    pub method: HttpMethod,
    pub concurrency: u64,
    pub duration: u64,

    #[serde(default)]
    pub body: Option<Body>,

    #[serde(default)]
    pub auth: Option<Auth>,

    #[serde(default)]
    pub query_params: Option<HashMap<String, String>>,

    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "content")]
pub enum Body {
    Json(serde_json::Value),
    Xml(String),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "credentials")]
pub enum Auth {
    None,
    Basic { username: String, password: String },
    Bearer { token: String },
    ApiKey { key_name: String, key_value: String, add_to: String },
}
