use super::axum_support::*;
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use axum_serde::Cbor;
use buckle::client::Client;
use std::sync::Arc;

//
// status handlers
//

pub(crate) async fn ping(State(client): State<Arc<Client>>) -> Result<()> {
    client.status().await?.ping().await?;
    Ok(())
}

//
// zfs handlers
//

#[derive(Debug, Clone, Default)]
pub(crate) struct ZFSList(Vec<buckle::client::ZFSStat>);

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

pub(crate) async fn zfs_list(
    State(client): State<Arc<Client>>,
    Cbor(filter): Cbor<String>,
) -> Result<ZFSList> {
    let filter = if filter.len() > 0 { Some(filter) } else { None };
    Ok(ZFSList(client.zfs().await?.list(filter).await?))
}

pub(crate) async fn zfs_create_dataset(
    State(client): State<Arc<Client>>,
    Cbor(dataset): Cbor<buckle::client::Dataset>,
) -> Result<()> {
    client.zfs().await?.create_dataset(dataset).await?;
    Ok(())
}

pub(crate) async fn zfs_create_volume(
    State(client): State<Arc<Client>>,
    Cbor(volume): Cbor<buckle::client::Volume>,
) -> Result<()> {
    client.zfs().await?.create_volume(volume).await?;
    Ok(())
}

pub(crate) async fn zfs_destroy(
    State(client): State<Arc<Client>>,
    Cbor(name): Cbor<String>,
) -> Result<()> {
    client.zfs().await?.destroy(name).await?;
    Ok(())
}
