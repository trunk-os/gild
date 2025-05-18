#![allow(dead_code)]
use crate::config::Config;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_serde::Cbor;
use buckle::client::Client;
use http::status::StatusCode;
use std::sync::Arc;

// axum requires a ton of boilerplate to do anything sane with a handler
// this is it. ah, rust. this literally all gets compiled out
type Result<T> = core::result::Result<T, AppError>;

struct AppError(anyhow::Error);

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

pub struct Server {
    config: Config,
    router: Router,
}

impl Server {
    pub fn new(config: Config) -> anyhow::Result<Self> {
        Ok(Self {
            router: Router::new()
                .route("/status/ping", get(ping))
                .route("/zfs/list", get(zfs_list))
                .with_state(Arc::new(Client::new(config.socket.clone().into())?)),
            config,
        })
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        // run our app with hyper, listening globally on port 3000
        let listener = tokio::net::TcpListener::bind(self.config.listen).await?;
        Ok(axum::serve(listener, self.router.clone()).await?)
    }
}

async fn ping(State(client): State<Arc<Client>>) -> Result<()> {
    client.status().await?.ping().await?;
    Ok(())
}

struct ZFSList(Vec<buckle::client::ZFSStat>);

impl IntoResponse for ZFSList {
    fn into_response(self) -> Response {
        Response::builder()
            .body(axum::body::Body::from(
                ciborium::cbor!(self.0).unwrap().into_bytes().unwrap(),
            ))
            .unwrap()
    }
}

async fn zfs_list(
    State(client): State<Arc<Client>>,
    Cbor(filter): Cbor<String>,
) -> Result<ZFSList> {
    let filter = if filter.len() > 0 { Some(filter) } else { None };
    Ok(ZFSList(client.zfs().await?.list(filter).await?))
}

#[cfg(test)]
mod tests {
    use crate::testutil::{start_server, TestClient};

    #[tokio::test]
    async fn test_ping() {
        let client = TestClient::new(start_server().await.unwrap());
        assert!(client.get::<()>("/status/ping").await.is_ok());
    }
}
