use crate::{config::Config, server::Server};
use anyhow::Result;
use buckle::{config::ZFSConfig, testutil::make_server};
use reqwest::Client;
use serde::{
    de::{Deserialize, DeserializeOwned},
    Serialize,
};
use std::net::SocketAddr;
use tempfile::NamedTempFile;

pub async fn find_listener() -> Result<(tokio::net::TcpListener, SocketAddr)> {
    loop {
        let port: u16 = rand::random();
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
        match tokio::net::TcpListener::bind(addr).await {
            Ok(x) => return Ok((x, addr)),
            _ => {}
        }
    }
}

pub async fn start_server(poolname: Option<String>) -> Result<SocketAddr> {
    let (_, dbfile) = NamedTempFile::new_in("tmp")?.keep()?;
    let (socket, addr) = find_listener().await?;
    drop(socket);
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    tokio::spawn(async move {
        let config = Config {
            listen: addr,
            socket: make_server(if let Some(poolname) = poolname {
                Some(buckle::config::Config {
                    socket: buckle::testutil::find_listener().unwrap(),
                    zfs: ZFSConfig { pool: poolname },
                })
            } else {
                None
            })
            .await
            .unwrap(),
            db: dbfile,
        };

        Server::new(config).await.unwrap().start().await.unwrap()
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

    pub async fn post<I, O>(&self, path: &str, input: I) -> Result<O>
    where
        I: Serialize,
        O: for<'de> Deserialize<'de> + DeserializeOwned + Default,
    {
        let mut inner = Vec::with_capacity(65535);
        let mut body = std::io::Cursor::new(&mut inner);
        ciborium::into_writer(&input, &mut body)?;

        let byt: Vec<u8> = self
            .client
            .post(&format!("{}{}", self.baseurl, path))
            .header("Content-type", "application/cbor")
            .body(body.into_inner().to_vec())
            .send()
            .await?
            .bytes()
            .await?
            .to_vec();
        if byt.len() > 0 {
            Ok(ciborium::from_reader(std::io::Cursor::new(byt))?)
        } else {
            Ok(O::default())
        }
    }
}
