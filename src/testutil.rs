use crate::{config::Config, server::Server};
use anyhow::Result;
use buckle::{config::ZFSConfig, testutil::make_server};
use reqwest::Client;
use serde::{
    de::{Deserialize, DeserializeOwned},
    Serialize,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tempfile::NamedTempFile;

pub async fn find_listener() -> Result<SocketAddr> {
    loop {
        let port: u16 = rand::random();
        let addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
        match tokio::net::TcpListener::bind(addr).await {
            Ok(_) => return Ok(addr),
            _ => {}
        }
    }
}

pub async fn make_config(addr: Option<SocketAddr>, poolname: Option<String>) -> Result<Config> {
    let (_, dbfile) = NamedTempFile::new_in("tmp")?.keep()?;
    Ok(Config {
        listen: if let Some(addr) = addr {
            addr
        } else {
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)
        },

        socket: make_server(if let Some(poolname) = poolname {
            Some(buckle::config::Config {
                socket: buckle::testutil::find_listener()?,
                zfs: ZFSConfig { pool: poolname },
            })
        } else {
            None
        })
        .await?,
        db: dbfile,
    })
}

pub async fn start_server(poolname: Option<String>) -> Result<SocketAddr> {
    let addr = find_listener().await?;
    let ret = addr.clone();
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let config = make_config(Some(addr), poolname).await.unwrap();
    let call = async move {
        Server::new(config).await.unwrap().start().await.unwrap();
    };
    tokio::spawn(call);
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    Ok(ret)
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

    pub async fn delete<T>(&self, path: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de> + DeserializeOwned + Default,
    {
        let byt: Vec<u8> = self
            .client
            .delete(&format!("{}{}", self.baseurl, path))
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

    pub async fn put<I, O>(&self, path: &str, input: I) -> Result<O>
    where
        I: Serialize,
        O: for<'de> Deserialize<'de> + DeserializeOwned + Default,
    {
        let mut inner = Vec::with_capacity(65535);
        let mut body = std::io::Cursor::new(&mut inner);
        ciborium::into_writer(&input, &mut body)?;

        let byt: Vec<u8> = self
            .client
            .put(&format!("{}{}", self.baseurl, path))
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
