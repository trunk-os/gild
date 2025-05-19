use super::{axum_support::*, ServerState};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use axum_serde::Cbor;
use std::sync::Arc;

//
// status handlers
//

pub(crate) async fn ping(State(state): State<Arc<ServerState>>) -> Result<()> {
    state.client.status().await?.ping().await?;
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
    State(state): State<Arc<ServerState>>,
    Cbor(filter): Cbor<String>,
) -> Result<ZFSList> {
    let filter = if filter.is_empty() {
        None
    } else {
        Some(filter)
    };

    Ok(ZFSList(state.client.zfs().await?.list(filter).await?))
}

pub(crate) async fn zfs_create_dataset(
    State(state): State<Arc<ServerState>>,
    Cbor(dataset): Cbor<buckle::client::Dataset>,
) -> Result<()> {
    state.client.zfs().await?.create_dataset(dataset).await?;
    Ok(())
}

pub(crate) async fn zfs_create_volume(
    State(state): State<Arc<ServerState>>,
    Cbor(volume): Cbor<buckle::client::Volume>,
) -> Result<()> {
    state.client.zfs().await?.create_volume(volume).await?;
    Ok(())
}

pub(crate) async fn zfs_destroy(
    State(state): State<Arc<ServerState>>,
    Cbor(name): Cbor<String>,
) -> Result<()> {
    state.client.zfs().await?.destroy(name).await?;
    Ok(())
}

//
// Auth handlers
//

#[allow(unused)]
pub(crate) async fn create_user() -> Result<()> {
    Ok(())
}

#[allow(unused)]
pub(crate) async fn remove_user() -> Result<()> {
    Ok(())
}

#[allow(unused)]
pub(crate) async fn list_users() -> Result<()> {
    Ok(())
}

#[allow(unused)]
pub(crate) async fn get_user() -> Result<()> {
    Ok(())
}

#[allow(unused)]
pub(crate) async fn update_user() -> Result<()> {
    Ok(())
}

#[allow(unused)]
pub(crate) async fn authenticate_user() -> Result<()> {
    Ok(())
}
