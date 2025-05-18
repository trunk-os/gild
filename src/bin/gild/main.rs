use anyhow::Result;
use gild::config::Config;
use gild::server::Server;

#[tokio::main]
async fn main() -> Result<()> {
    Server::new(Config::default())?.start().await?;
    Ok(())
}
