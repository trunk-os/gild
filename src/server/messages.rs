use buckle::client::Info;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Pagination {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<chrono::DateTime<chrono::Local>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u8>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PingResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<Info>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Token {
    pub(crate) token: String,
}

#[derive(Debug, Clone, Default, Validate, Serialize, Deserialize)]
pub struct Authentication {
    #[validate(length(min = 3, max = 30))]
    pub username: String,
    #[validate(length(min = 8, max = 100))]
    pub password: String,
}
