use anyhow::Result;
use serde::Deserialize;
use std::net::SocketAddr;

const DEFAULT_CONFIG_PATH: &str = "/trunk/gild.yaml";
const DEFAULT_BUCKLE_PATH: &str = "/tmp/buckled.sock";
const DEFAULT_DB: &str = "/gild.db";
const DEFAULT_LISTEN: &str = "0.0.0.0:3000";

fn default_db() -> std::path::PathBuf {
    DEFAULT_DB.into()
}

fn default_socket() -> std::path::PathBuf {
    DEFAULT_BUCKLE_PATH.into()
}

fn default_listen() -> SocketAddr {
    DEFAULT_LISTEN.parse().unwrap()
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_listen")]
    pub listen: SocketAddr,
    #[serde(default = "default_socket")]
    pub socket: std::path::PathBuf,
    #[serde(default = "default_db")]
    pub db: std::path::PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config::from_file(DEFAULT_CONFIG_PATH.into()).unwrap()
    }
}

impl Config {
    pub(crate) fn from_file(file: std::path::PathBuf) -> Result<Self> {
        let file = std::fs::OpenOptions::new().read(true).open(file)?;
        Ok(serde_yaml_ng::from_reader(file)?)
    }

    pub(crate) async fn get_db(&self) -> Result<crate::db::DB> {
        Ok(crate::db::DB::new(self.clone()).await?)
    }

    pub(crate) fn get_client(&self) -> Result<buckle::client::Client> {
        buckle::client::Client::new(self.socket.clone())
    }
}
