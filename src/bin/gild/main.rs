use anyhow::Result;
use gild::config::Config;
use gild::server::Server;
use rand::Fill;
use rand_core::OsRng;

#[tokio::main]
async fn main() -> Result<()> {
    let mut key: [u8; 64] = [0u8; 64];
    let mut salt: [u8; 32] = [0u8; 32];
    key.try_fill(&mut OsRng)?;
    salt.try_fill(&mut OsRng)?;
    let config = Config {
        db: "./gild.db".into(),
        listen: "0.0.0.0:3000".parse()?,
        socket: buckle::testutil::make_server(None).await.unwrap(),
        signing_key: key.to_vec(),
        signing_key_salt: salt.to_vec(),
    };
    Server::new(config).await?.start().await?;
    Ok(())
}
