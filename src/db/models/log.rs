use anyhow::{anyhow, Result};
use http::HeaderValue;
use serde::{Deserialize, Serialize};
use validator::Validate;
use welds::{state::DbState, WeldsModel};

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

impl AuditLog {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn from_request<T>(&mut self, request: axum::http::Request<T>) -> &mut Self {
        self.endpoint = request.uri().to_string();
        self.ip = request
            .headers()
            .get("X-Real-IP")
            .map(|e| e.to_str().unwrap())
            .unwrap_or_else(|| {
                request
                    .headers()
                    .get("X-Forwarded-For")
                    .map(|e| e.to_str().unwrap().split("; ").next().unwrap_or_default())
                    .unwrap_or_else(|| "")
            })
            .to_string();

        self
    }

    pub fn from_user(&mut self, user: super::User) -> &mut Self {
        self.user_id = user.id;
        self
    }

    pub fn with_entry(&mut self, entry: String) -> &mut Self {
        self.entry = entry;
        self
    }

    pub fn with_data<T>(&mut self, data: T) -> Result<&mut Self>
    where
        T: serde::Serialize,
    {
        self.data = serde_json::to_string(&data)?;
        Ok(self)
    }

    pub async fn complete(mut self, db: &super::super::DB) -> Result<()> {
        self.time = chrono::Local::now();
        let mut state = DbState::new_uncreated(self);
        Ok(state
            .save(db.handle())
            .await
            .map_err(|e| anyhow!(e.to_string()))?)
    }
}
