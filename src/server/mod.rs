mod axum_support;
mod handlers;
#[cfg(test)]
mod tests;
use self::handlers::*;

use crate::db::DB;
use crate::{config::Config, db::models::AuditLog};
use anyhow::Result;
use axum::body::Body;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use axum_support::WithLog;
use buckle::client::{Client, Info};
use http::{header::*, Method, Request};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::cors::CorsLayer;
use tracing::{error, info, Span};
use validator::Validate;

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

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub(crate) struct Pagination {
    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<chrono::DateTime<chrono::Local>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    per_page: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    page: Option<u8>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub(crate) struct PingResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    info: Option<Info>,
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
                .route("/status/log", post(log))
                .route("/zfs/list", post(zfs_list))
                .route("/zfs/create_volume", post(zfs_create_volume))
                .route("/zfs/create_dataset", post(zfs_create_dataset))
                .route("/zfs/modify_dataset", post(zfs_modify_dataset))
                .route("/zfs/modify_volume", post(zfs_modify_volume))
                .route("/zfs/destroy", post(zfs_destroy))
                .route("/users", put(create_user).get(list_users))
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
                            // FIXME: definitely doing this wrong
                            tower_http::trace::TraceLayer::new_for_http()
                                .make_span_with(|_: &Request<Body>| {
                                    tracing::info_span!("http-request")
                                })
                                .on_request(|req: &Request<Body>, span: &Span| {
                                    info!(
                                        "[{}] {} {}",
                                        span.id()
                                            .unwrap_or_else(|| tracing::Id::from_u64(1))
                                            .into_u64(),
                                        req.method(),
                                        req.uri().path()
                                    );
                                })
                                .on_failure(|error: ServerErrorsFailureClass, _, span: &Span| {
                                    error!(
                                        "[{}] Error on request: {}",
                                        span.id()
                                            .unwrap_or_else(|| tracing::Id::from_u64(1))
                                            .into_u64(),
                                        error
                                    )
                                }),
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
                                .allow_credentials(true)
                                .allow_origin(tower_http::cors::AllowOrigin::exact(
                                    HeaderValue::from_str(&config.origin)?,
                                ))
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
