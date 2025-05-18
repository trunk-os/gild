use crate::{config::Config, server::Server};
use anyhow::{anyhow, Result};
use buckle::testutil::make_server;
use reqwest::Client;
use serde::de::{Deserialize, DeserializeOwned};
use std::net::SocketAddr;

pub fn find_listener() -> Result<SocketAddr> {
    for port in 3000..32767 {
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
        match std::net::TcpListener::bind(addr) {
            Ok(_) => return Ok(addr),
            _ => {}
        }
    }

    Err(anyhow!("no open port found"))
}

pub async fn start_server() -> Result<SocketAddr> {
    let addr = find_listener()?;
    tokio::spawn(async move {
        Server::new(Config {
            listen: addr,
            socket: make_server(None).await.unwrap(),
        })
        .unwrap()
        .start()
        .await
        .unwrap()
    });
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    Ok(addr)
}

pub struct TestClient {
    client: Client,
    baseurl: String,
}

impl TestClient {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            client: Client::new(),
            baseurl: format!("http://{}", addr),
        }
    }

    pub async fn get<T>(&self, path: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + DeserializeOwned + Default,
    {
        let byt: Vec<u8> = self
            .client
            .get(&format!("{}{}", self.baseurl, path))
            .send()
            .await?
            .bytes()
            .await?
            .to_vec();
        if byt.len() > 0 {
            Ok(ciborium::from_reader(std::io::Cursor::new(byt))?)
        } else {
            Ok(T::default())
        }
    }
}
