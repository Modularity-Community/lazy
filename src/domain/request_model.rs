use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthConfig {
    None,
    Bearer { token: String },
    Basic { username: String, password: String },
    ApiKey { key: String, value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSavedModel {
    pub id: Option<i64>,
    pub folder_id: Option<i64>,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<KeyValuePair>,
    pub params: Vec<KeyValuePair>,
    pub auth: AuthConfig,
    pub body: String,
}
