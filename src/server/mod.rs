mod axum_support;
mod handlers;
#[cfg(test)]
mod tests;
use self::handlers::*;

use crate::config::Config;
use crate::db::DB;
use anyhow::Result;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use buckle::client::Client;
use http::{header::*, Method};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use validator::Validate;

#[derive(Debug, Clone)]
pub struct ServerState {
    client: Client,
    db: DB,
    config: Config,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub(crate) struct Token {
    pub(crate) token: String,
}

#[derive(Debug, Clone, Default, Validate, Serialize, Deserialize)]
pub struct Authentication {
    #[validate(length(min = 3, max = 30))]
    pub username: String,
    #[validate(length(min = 8, max = 100))]
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct Server {
    config: Config,
    router: Router,
}

impl Server {
    pub async fn new(config: Config) -> Result<Self> {
        Ok(Self {
            router: Router::new()
                .route("/status/ping", get(ping))
                .route("/zfs/list", post(zfs_list))
                .route("/zfs/create_volume", post(zfs_create_volume))
                .route("/zfs/create_dataset", post(zfs_create_dataset))
                .route("/zfs/destroy", post(zfs_destroy))
                .route("/users", put(create_user).get(list_users))
                .route(
                    "/user/{id}",
                    delete(remove_user).get(get_user).post(update_user),
                )
                .route("/session/login", post(login))
                .with_state(Arc::new(ServerState {
                    client: config.get_client()?,
                    db: config.get_db().await?,
                    config: config.clone(),
                }))
                .layer(
                    ServiceBuilder::new().layer(
                        CorsLayer::new()
                            .allow_methods([
                                Method::GET,
                                Method::POST,
                                Method::DELETE,
                                Method::PUT,
                                Method::PATCH,
                                Method::HEAD,
                                Method::TRACE,
                                Method::OPTIONS,
                            ])
                            .allow_credentials(true)
                            .allow_origin(tower_http::cors::AllowOrigin::exact(
                                HeaderValue::from_str("http://christopher-office:3000")?,
                            ))
                            .allow_headers([CONTENT_TYPE, ACCEPT])
                            .allow_private_network(true),
                    ),
                ),
            config: config.clone(),
        })
    }

    pub async fn start(&self) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(self.config.listen).await?;
        Ok(axum::serve(listener, self.router.clone()).await?)
    }
}
