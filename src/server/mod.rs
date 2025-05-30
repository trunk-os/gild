mod axum_support;
mod handlers;
pub mod messages;
#[cfg(test)]
mod tests;
use self::handlers::*;

use crate::db::DB;
use crate::{config::Config, db::models::AuditLog};
use anyhow::Result;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use axum_support::WithLog;
use buckle::client::Client;
use http::{header::*, Method};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest};
use tracing::Level;

#[derive(Debug, Clone)]
pub struct ServerState {
    client: Client,
    db: DB,
    config: Config,
}

impl ServerState {
    pub(crate) fn with_log<T>(&self, resp: axum_support::Result<T>, log: AuditLog) -> WithLog<T> {
        WithLog(resp, log, self.clone().into())
    }
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
                .route("/status/log", post(log))
                .route("/zfs/list", post(zfs_list))
                .route("/zfs/create_volume", post(zfs_create_volume))
                .route("/zfs/create_dataset", post(zfs_create_dataset))
                .route("/zfs/modify_dataset", post(zfs_modify_dataset))
                .route("/zfs/modify_volume", post(zfs_modify_volume))
                .route("/zfs/destroy", post(zfs_destroy))
                .route("/users", put(create_user).post(list_users))
                .route(
                    "/user/{id}",
                    delete(remove_user).get(get_user).post(update_user),
                )
                .route("/session/login", post(login))
                .route("/session/me", get(me))
                .with_state(Arc::new(ServerState {
                    client: config.get_client()?,
                    db: config.get_db().await?,
                    config: config.clone(),
                }))
                .layer(
                    ServiceBuilder::new()
                        .layer(
                            tower_http::trace::TraceLayer::new_for_http()
                                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                                .on_request(DefaultOnRequest::new().level(Level::INFO))
                                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
                        )
                        .layer(
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
                                .allow_origin(Any)
                                .allow_headers([CONTENT_TYPE, ACCEPT, AUTHORIZATION])
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
