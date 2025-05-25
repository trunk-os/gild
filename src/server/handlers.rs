use crate::db::models::{AuditLog, Session, User};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;

use super::{axum_support::*, Authentication, Pagination, PingResult, ServerState, Token};
use anyhow::anyhow;
use axum::extract::{Path, State};
use axum_serde::Cbor;
use buckle::client::ZFSStat;
use std::{ops::Deref, sync::Arc};
use validator::Validate;
use welds::{exts::VecStateExt, state::DbState};

//
// status handlers
//

pub(crate) async fn ping(
    State(state): State<Arc<ServerState>>,
    Account(user): Account<Option<User>>,
) -> Result<CborOut<PingResult>> {
    let result = state.client.status().await?.ping().await?;
    Ok(CborOut(if user.is_some() {
        PingResult {
            info: Some(result.info.unwrap_or_default().into()),
        }
    } else {
        PingResult::default()
    }))
}

pub(crate) async fn log(
    State(state): State<Arc<ServerState>>,
    Account(_): Account<User>,
    Cbor(pagination): Cbor<Pagination>,
) -> Result<CborOut<Vec<AuditLog>>> {
    let mut selector = AuditLog::all();

    if let Some(since) = pagination.since {
        selector = selector.where_col(|c| c.time.gt(since));
    }

    if let Some(page) = pagination.page {
        selector = selector
            .offset(page.into())
            .limit(pagination.per_page.unwrap_or(20).into());
    } else if let Some(per_page) = pagination.per_page {
        selector = selector.limit(per_page.into())
    }

    Ok(CborOut(
        selector.run(state.db.handle()).await?.into_inners(),
    ))
}

//
// zfs handlers
//

pub(crate) async fn zfs_list(
    State(state): State<Arc<ServerState>>,
    Account(_): Account<User>,
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
    Account(_): Account<User>,
    Cbor(dataset): Cbor<buckle::client::Dataset>,
) -> Result<()> {
    state.client.zfs().await?.create_dataset(dataset).await?;
    Ok(())
}

pub(crate) async fn zfs_modify_dataset(
    State(state): State<Arc<ServerState>>,
    Account(_): Account<User>,
    Cbor(dataset): Cbor<buckle::client::ModifyDataset>,
) -> Result<()> {
    state.client.zfs().await?.modify_dataset(dataset).await?;
    Ok(())
}

pub(crate) async fn zfs_create_volume(
    State(state): State<Arc<ServerState>>,
    Account(_): Account<User>,
    Cbor(volume): Cbor<buckle::client::Volume>,
) -> Result<()> {
    state.client.zfs().await?.create_volume(volume).await?;
    Ok(())
}

pub(crate) async fn zfs_modify_volume(
    State(state): State<Arc<ServerState>>,
    Account(_): Account<User>,
    Cbor(dataset): Cbor<buckle::client::ModifyVolume>,
) -> Result<()> {
    state.client.zfs().await?.modify_volume(dataset).await?;
    Ok(())
}

pub(crate) async fn zfs_destroy(
    State(state): State<Arc<ServerState>>,
    Account(_): Account<User>,
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
    Account(login): Account<Option<User>>,
    Cbor(user): Cbor<User>,
) -> Result<CborOut<User>> {
    if login.is_none() {
        let count = User::all().count(state.db.handle()).await?;
        if count != 0 {
            return Err(anyhow!("invalid login").into());
        }
    }

    let mut user = DbState::new_uncreated(user);

    user.validate()?;

    // crypt the plaintext password if it is set, otherwise return error (passwords are required at
    // this step)
    if let Some(password) = user.plaintext_password.clone() {
        user.set_password(password)?;
        user.plaintext_password = None;
    } else {
        return Err(anyhow!("password is required").into());
    }

    user.save(state.db.handle()).await?;
    Ok(CborOut(user.into_inner()))
}

pub(crate) async fn remove_user(
    State(state): State<Arc<ServerState>>,
    Account(_): Account<User>,
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
    Account(_): Account<User>,
) -> Result<CborOut<Vec<User>>> {
    Ok(CborOut(
        User::all().run(state.db.handle()).await?.into_inners(),
    ))
}

pub(crate) async fn get_user(
    State(state): State<Arc<ServerState>>,
    Account(_): Account<User>,
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
    Account(_): Account<User>,
    Cbor(mut user): Cbor<User>,
) -> Result<()> {
    if User::find_by_id(state.db.handle(), id).await?.is_some() {
        // if we got the record, the id is correct
        user.id = id;
        user.validate()?;

        // crypt the plaintext password if it is set
        if let Some(password) = &user.plaintext_password {
            user.set_password(password.clone())?;
        }

        // welds doesn't realize the fields have already changed, these two lines force it to see
        // it.
        let mut dbstate: DbState<User> = DbState::db_loaded(user.clone());
        dbstate.replace_inner(user);
        Ok(dbstate.save(state.db.handle()).await?)
    } else {
        Err(anyhow!("invalid user").into())
    }
}

//
// Authentication
//

pub(crate) async fn login(
    State(state): State<Arc<ServerState>>,
    Cbor(form): Cbor<Authentication>,
) -> Result<CborOut<Token>> {
    form.validate()?;

    let users = User::all()
        .where_col(|c| c.username.equal(form.username.clone()))
        .run(state.db.handle())
        .await?;

    let user = match users.first() {
        Some(user) => user.deref(),
        None => return Err(anyhow!("invalid login").into()),
    };

    if user.login(form.password).is_err() {
        return Err(anyhow!("invalid login").into());
    }

    let mut session = Session::new_assigned(user);
    session.save(state.db.handle()).await?;

    let key: Hmac<sha2::Sha384> = Hmac::new_from_slice(&state.config.signing_key)?;
    let header = jwt::Header {
        algorithm: jwt::AlgorithmType::Hs384,
        ..Default::default()
    };
    let claims = session.to_jwt();
    let jwt = jwt::Token::new(header, claims).sign_with_key(&key)?;
    Ok(CborOut(Token { token: jwt.into() }))
}

pub(crate) async fn me(
    State(_): State<Arc<ServerState>>,
    Account(user): Account<User>,
) -> Result<CborOut<User>> {
    Ok(CborOut(user))
}
