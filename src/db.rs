#![allow(dead_code)]
use anyhow::Result;
use welds::connections::sqlite::{connect, SqliteClient};

use crate::config::Config;

#[derive(Clone)]
pub(crate) struct DB(SqliteClient);

impl std::fmt::Debug for DB {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let options = self.0.as_sqlx_pool().connect_options();
        let filename = options.get_filename();
        f.write_str(&format!(
            "db client, connected to {}",
            filename.to_str().unwrap()
        ))
    }
}

impl DB {
    pub(crate) async fn new(config: &Config) -> Result<Self> {
        Ok(Self(
            connect(&format!("sqlite://{}", config.db.to_str().unwrap())).await?,
        ))
    }

    pub(crate) async fn migrate(&self) -> Result<()> {
        Ok(())
    }
}

pub(crate) mod models {
    use welds::WeldsModel;

    #[derive(Debug, WeldsModel)]
    pub(crate) struct User {
        #[welds(rename = "user_id")]
        #[welds(primary_key)]
        id: uuid::Uuid,
        username: String,
        realname: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        password: Vec<u8>,
    }

    #[derive(Debug, WeldsModel)]
    pub(crate) struct Session {
        #[welds(rename = "session_id")]
        #[welds(primary_key)]
        id: uuid::Uuid,
        secret: Vec<u8>,
        expires: chrono::DateTime<chrono::Local>,
    }
}
