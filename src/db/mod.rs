#![allow(dead_code)]
pub(crate) mod migrations;
pub(crate) mod models;
use crate::config::Config;
use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, Sqlite};
use welds::connections::sqlite::{connect, SqliteClient};

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
        match connect(&format!("sqlite:{}", config.db.to_str().unwrap())).await {
            Ok(c) => Ok(Self(c)),
            Err(_) => {
                Self::create(config).await?;
                Ok(Self(
                    connect(&format!("sqlite:{}", config.db.to_str().unwrap())).await?,
                ))
            }
        }
    }

    pub(crate) async fn migrate(&self) -> Result<()> {
        Ok(migrations::migrate(self).await?)
    }

    pub async fn create(config: &Config) -> anyhow::Result<()> {
        sqlx::sqlite::CREATE_DB_WAL.store(true, std::sync::atomic::Ordering::Release);

        Sqlite::create_database(&format!("sqlite:{}", config.db.to_str().unwrap())).await?;
        Ok(())
    }
}
