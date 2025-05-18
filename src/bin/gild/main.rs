use anyhow::Result;
use gild::config::Config;
use gild::server::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config {
        listen: "127.0.0.1:3000".parse()?,
        socket: buckle::testutil::make_server(None).await.unwrap(),
    };
    Server::new(config)?.start().await?;
    Ok(())
}
