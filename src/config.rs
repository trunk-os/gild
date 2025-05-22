use anyhow::{anyhow, Result};
use rand::Fill;
use rand_core::OsRng;
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

fn default_random() -> Vec<u8> {
    let mut v: [u8; 64] = [0u8; 64];
    v.try_fill(&mut OsRng).unwrap();
    v.to_vec()
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_listen")]
    pub listen: SocketAddr,
    #[serde(default = "default_socket")]
    pub socket: std::path::PathBuf,
    #[serde(default = "default_db")]
    pub db: std::path::PathBuf,
    #[serde(default = "default_random")]
    pub signing_key: Vec<u8>,
    #[serde(default = "default_random")]
    pub signing_key_salt: Vec<u8>,
}

impl Default for Config {
    fn default() -> Self {
        Config::from_file(DEFAULT_CONFIG_PATH.into()).unwrap()
    }
}

impl Config {
    pub(crate) fn from_file(file: std::path::PathBuf) -> Result<Self> {
        let file = std::fs::OpenOptions::new().read(true).open(file)?;
        let mut this: Self = serde_yaml_ng::from_reader(file)?;
        this.convert_signing_key()?;
        Ok(this)
    }

    fn convert_signing_key(&mut self) -> Result<()> {
        let mut buf: [u8; 64] = [0u8; 64];
        let kdf = argon2::Argon2::default();
        kdf.hash_password_into(
            self.signing_key.as_slice(),
            self.signing_key_salt.as_slice(),
            &mut buf,
        )
        .map_err(|e| anyhow!(e.to_string()))?;

        // overwrite the keys in the config
        // we never generate (just parse) this format so this is a safe conversion.
        self.signing_key = buf.to_vec();
        self.signing_key_salt = Vec::default();

        Ok(())
    }

    pub(crate) async fn get_db(&self) -> Result<crate::db::DB> {
        Ok(crate::db::DB::new(self.clone()).await?)
    }

    pub(crate) fn get_client(&self) -> Result<buckle::client::Client> {
        buckle::client::Client::new(self.socket.clone())
    }
}
