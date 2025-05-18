use anyhow::Result;
use serde::Deserialize;
use std::net::SocketAddr;

const DEFAULT_CONFIG_PATH: &str = "/trunk/gild.yaml";

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    listen: SocketAddr,
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
}
