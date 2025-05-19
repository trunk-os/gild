mod axum_support;
mod handlers;
#[cfg(test)]
mod tests;
use self::handlers::*;

use crate::config::Config;
use axum::{
    routing::{get, post},
    Router,
};
use buckle::client::Client;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

pub struct Server {
    config: Config,
    router: Router,
}

impl Server {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        Ok(Self {
            router: Router::new()
                .route("/status/ping", get(ping))
                .route("/zfs/list", post(zfs_list))
                .route("/zfs/create_volume", post(zfs_create_volume))
                .route("/zfs/create_dataset", post(zfs_create_dataset))
                .route("/zfs/destroy", post(zfs_destroy))
                .with_state(Arc::new(Client::new(config.socket.clone().into())?))
                .layer(
                    ServiceBuilder::new().layer(
                        CorsLayer::new()
                            .allow_methods([http::Method::GET, http::Method::POST])
                            .allow_origin(Any)
                            .allow_headers([http::header::CONTENT_TYPE, http::header::ACCEPT])
                            .allow_private_network(true),
                    ),
                ),
            config,
        })
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        // run our app with hyper, listening globally on port 3000
        let listener = tokio::net::TcpListener::bind(self.config.listen).await?;
        Ok(axum::serve(listener, self.router.clone()).await?)
    }
}
