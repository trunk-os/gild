use crate::db::models::User;

use super::{axum_support::*, ServerState};
use anyhow::anyhow;
use axum::extract::{Path, State};
use axum_serde::Cbor;
use buckle::client::ZFSStat;
use std::sync::Arc;
use welds::{exts::VecStateExt, state::DbState};

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
//
pub(crate) async fn zfs_list(
    State(state): State<Arc<ServerState>>,
    Cbor(filter): Cbor<String>,
) -> Result<CborOut<Vec<ZFSStat>>> {
    let filter = if filter.is_empty() {
        None
    } else {
        Some(filter)
    };

    Ok(CborOut(state.client.zfs().await?.list(filter).await?))
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

pub(crate) async fn create_user(
    State(state): State<Arc<ServerState>>,
    Cbor(user): Cbor<User>,
) -> Result<CborOut<User>> {
    let mut user = DbState::new_uncreated(user);
    user.save(state.db.handle()).await?;
    Ok(CborOut(user.into_inner()))
}

pub(crate) async fn remove_user(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<u32>,
) -> Result<()> {
    if let Some(mut user) = User::find_by_id(state.db.handle(), id).await? {
        Ok(user.delete(state.db.handle()).await?)
    } else {
        Err(anyhow!("invalid user").into())
    }
}

pub(crate) async fn list_users(
    State(state): State<Arc<ServerState>>,
) -> Result<CborOut<Vec<User>>> {
    Ok(CborOut(
        User::all().run(state.db.handle()).await?.into_inners(),
    ))
}

pub(crate) async fn get_user(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<u32>,
) -> Result<CborOut<User>> {
    Ok(CborOut(
        User::find_by_id(state.db.handle(), id)
            .await?
            .ok_or(anyhow!("invalid user"))?
            .into_inner(),
    ))
}

pub(crate) async fn update_user(
    State(state): State<Arc<ServerState>>,
    Path(id): Path<u32>,
    Cbor(mut user): Cbor<User>,
) -> Result<()> {
    if let Some(fetched) = User::find_by_id(state.db.handle(), id).await? {
        // replace the id, but everything else is editable.
        let inner = fetched.into_inner();
        user.id = inner.id;
        let mut user: DbState<User> = DbState::db_loaded(user);
        Ok(user.save(state.db.handle()).await?)
    } else {
        Err(anyhow!("invalid user").into())
    }
}
