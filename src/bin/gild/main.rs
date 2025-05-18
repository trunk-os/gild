use anyhow::Result;
use gild::config::Config;
use gild::server::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config {
        listen: "0.0.0.0:3000".parse()?,
        socket: buckle::testutil::make_server(None).await.unwrap(),
    };
    Server::new(config)?.start().await?;
    Ok(())
}
