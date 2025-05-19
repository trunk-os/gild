#![allow(dead_code)]
use crate::config::Config;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    routing::{get, post},
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
                .route("/zfs/list", post(zfs_list))
                .route("/zfs/create_volume", post(zfs_create_volume))
                .route("/zfs/create_dataset", post(zfs_create_dataset))
                .route("/zfs/destroy", post(zfs_destroy))
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

//
// status handlers
//

async fn ping(State(client): State<Arc<Client>>) -> Result<()> {
    client.status().await?.ping().await?;
    Ok(())
}

//
// zfs handlers
//

#[derive(Debug, Clone, Default)]
struct ZFSList(Vec<buckle::client::ZFSStat>);

impl IntoResponse for ZFSList {
    fn into_response(self) -> Response {
        let mut inner = Vec::with_capacity(65536);
        let mut buf = std::io::Cursor::new(&mut inner);
        ciborium::into_writer(&self.0, &mut buf).unwrap();

        Response::builder()
            .body(axum::body::Body::from(buf.into_inner().to_vec()))
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

async fn zfs_create_dataset(
    State(client): State<Arc<Client>>,
    Cbor(dataset): Cbor<buckle::client::Dataset>,
) -> Result<()> {
    client.zfs().await?.create_dataset(dataset).await?;
    Ok(())
}

async fn zfs_create_volume(
    State(client): State<Arc<Client>>,
    Cbor(volume): Cbor<buckle::client::Volume>,
) -> Result<()> {
    client.zfs().await?.create_volume(volume).await?;
    Ok(())
}

async fn zfs_destroy(State(client): State<Arc<Client>>, Cbor(name): Cbor<String>) -> Result<()> {
    client.zfs().await?.destroy(name).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::testutil::{start_server, TestClient};

    #[tokio::test]
    async fn test_ping() {
        let client = TestClient::new(start_server(None).await.unwrap());
        assert!(client.get::<()>("/status/ping").await.is_ok());
    }

    #[cfg(feature = "zfs")]
    mod zfs {
        use buckle::client::ZFSStat;

        use crate::testutil::{start_server, TestClient};

        #[tokio::test]
        async fn test_zfs() {
            let _ = buckle::testutil::destroy_zpool("gild", None);
            let zpool = buckle::testutil::create_zpool("gild").unwrap();
            let client =
                TestClient::new(start_server(Some("buckle-test-gild".into())).await.unwrap());
            let result: Vec<ZFSStat> = client.post("/zfs/list", "").await.unwrap();
            assert_eq!(result.len(), 0);
            client
                .post::<_, ()>(
                    "/zfs/create_dataset",
                    buckle::client::Dataset {
                        name: "dataset".into(),
                        quota: None,
                    },
                )
                .await
                .unwrap();
            let result: Vec<ZFSStat> = client.post("/zfs/list", "").await.unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].name, "dataset");
            assert_eq!(result[0].full_name, "buckle-test-gild/dataset");
            assert_ne!(result[0].size, 0);
            assert_ne!(result[0].avail, 0);
            assert_ne!(result[0].refer, 0);
            assert_ne!(result[0].used, 0);
            assert_eq!(
                result[0].mountpoint,
                Some("/buckle-test-gild/dataset".into())
            );
            client
                .post::<_, ()>(
                    "/zfs/create_volume",
                    buckle::client::Volume {
                        name: "volume".into(),
                        size: 100 * 1024 * 1024,
                    },
                )
                .await
                .unwrap();
            let result: Vec<ZFSStat> = client.post("/zfs/list", "").await.unwrap();
            assert_eq!(result.len(), 2);
            let result: Vec<ZFSStat> = client.post("/zfs/list", "volume").await.unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].name, "volume");
            assert_eq!(result[0].full_name, "buckle-test-gild/volume");
            assert_ne!(result[0].size, 0);
            assert_ne!(result[0].avail, 0);
            assert_ne!(result[0].refer, 0);
            assert_ne!(result[0].used, 0);
            assert_eq!(result[0].mountpoint, None);

            let result: Vec<ZFSStat> = client.post("/zfs/list", "dataset").await.unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0].name, "dataset");
            assert_eq!(result[0].full_name, "buckle-test-gild/dataset");
            assert_ne!(result[0].size, 0);
            assert_ne!(result[0].avail, 0);
            assert_ne!(result[0].refer, 0);
            assert_ne!(result[0].used, 0);
            assert_eq!(
                result[0].mountpoint,
                Some("/buckle-test-gild/dataset".into())
            );

            client
                .post::<_, ()>("/zfs/destroy", "dataset")
                .await
                .unwrap();
            let result: Vec<ZFSStat> = client.post("/zfs/list", "dataset").await.unwrap();
            assert_eq!(result.len(), 0);
            let result: Vec<ZFSStat> = client.post("/zfs/list", "").await.unwrap();
            assert_eq!(result.len(), 1);
            client
                .post::<_, ()>("/zfs/destroy", "volume")
                .await
                .unwrap();
            let result: Vec<ZFSStat> = client.post("/zfs/list", "volume").await.unwrap();
            assert_eq!(result.len(), 0);

            buckle::testutil::destroy_zpool("gild", Some(&zpool)).unwrap();
        }
    }
}
