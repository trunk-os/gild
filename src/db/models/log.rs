use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use validator::Validate;
use welds::WeldsModel;

#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    WeldsModel,
    Default,
    Serialize,
    Deserialize,
    Validate,
)]
#[welds(table = "audit_log")]
#[welds(BelongsTo(user, super::User, "user_id"))]
pub struct AuditLog {
    #[welds(primary_key)]
    pub id: u32,
    pub user_id: u32,
    pub time: chrono::DateTime<chrono::Local>,
    pub entry: String,
    pub endpoint: String,
    pub ip: String,
    pub data: String,
}
