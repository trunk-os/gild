use crate::db::models::{AuditLog, Session, User};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;

use super::{axum_support::*, messages::*, ServerState};
use anyhow::anyhow;
use axum::extract::{Path, State};
use axum_serde::Cbor;
use buckle::client::ZFSStat;
use std::{collections::HashMap, ops::Deref, sync::Arc};
use validator::Validate;
use welds::{exts::VecStateExt, state::DbState};

//
// status handlers
//

pub(crate) async fn ping(
    State(state): State<Arc<ServerState>>,
    Account(user): Account<Option<User>>,
) -> Result<CborOut<PingResult>> {
    Ok(CborOut(if user.is_some() {
        let start = std::time::Instant::now();
        let result = state.client.status().await?.ping().await;

        let mut error = None;
        let mut info = None;

        match result {
            Ok(result) => info = Some(result.info.unwrap_or_default().into()),
            Err(e) => error = Some(e.to_string()),
        }

        PingResult {
            health: Some(HealthStatus {
                buckle: Health {
                    latency: Some((std::time::Instant::now() - start).as_millis() as u64),
                    error,
                },
            }),
            info,
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
    Log(mut log): Log,
    Cbor(dataset): Cbor<buckle::client::Dataset>,
) -> Result<WithLog<()>> {
    let log = log
        .with_entry("Creating dataset")
        .with_data(&dataset)?
        .clone();
    state.client.zfs().await?.create_dataset(dataset).await?;
    Ok(state.with_log(Ok(()), log))
}

pub(crate) async fn zfs_modify_dataset(
    State(state): State<Arc<ServerState>>,
    Account(_): Account<User>,
    Log(mut log): Log,
    Cbor(dataset): Cbor<buckle::client::ModifyDataset>,
) -> Result<WithLog<()>> {
    let log = log
        .with_entry("Modifying dataset")
        .with_data(&dataset)?
        .clone();
    state.client.zfs().await?.modify_dataset(dataset).await?;
    Ok(state.with_log(Ok(()), log))
}

pub(crate) async fn zfs_create_volume(
    State(state): State<Arc<ServerState>>,
    Account(_): Account<User>,
    Log(mut log): Log,
    Cbor(volume): Cbor<buckle::client::Volume>,
) -> Result<WithLog<()>> {
    let log = log
        .with_entry("Creating volume")
        .with_data(&volume)?
        .clone();
    state.client.zfs().await?.create_volume(volume).await?;
    Ok(state.with_log(Ok(()), log))
}

pub(crate) async fn zfs_modify_volume(
    State(state): State<Arc<ServerState>>,
    Account(_): Account<User>,
    Log(mut log): Log,
    Cbor(volume): Cbor<buckle::client::ModifyVolume>,
) -> Result<WithLog<()>> {
    let log = log
        .with_entry("Modifying volume")
        .with_data(&volume)?
        .clone();
    state.client.zfs().await?.modify_volume(volume).await?;
    Ok(state.with_log(Ok(()), log))
}

pub(crate) async fn zfs_destroy(
    State(state): State<Arc<ServerState>>,
    Account(_): Account<User>,
    Log(mut log): Log,
    Cbor(name): Cbor<String>,
) -> Result<WithLog<()>> {
    let mut map: HashMap<&str, &str> = HashMap::default();
    map.insert("name", &name);

    let log = log
        .with_entry("Destroy volume or dataset")
        .with_data(&map)?
        .clone();

    state.client.zfs().await?.destroy(name).await?;
    Ok(state.with_log(Ok(()), log))
}

//
// Auth handlers
//

pub(crate) async fn create_user(
    State(state): State<Arc<ServerState>>,
    Account(login): Account<Option<User>>,
    Log(mut log): Log,
    Cbor(user): Cbor<User>,
) -> Result<WithLog<CborOut<User>>> {
    if login.is_none() {
        if !User::first_time_setup(&state.db).await? {
            return Err(anyhow!("invalid login").into());
        }
    }

    let mut user = DbState::new_uncreated(user);

    user.validate()?;

    // crypt the plaintext password if it is set, otherwise return error (passwords are required at
    // this step)
    if let Some(password) = user.plaintext_password.clone() {
        user.set_password(password)?;
    } else {
        return Err(anyhow!("password is required").into());
    }

    user.plaintext_password = None;

    user.save(state.db.handle()).await?;

    let inner = user.into_inner();
    let log = log.with_entry("Creating user").with_data(&inner)?.clone();
    Ok(state.with_log(Ok(CborOut(inner)), log))
}

pub(crate) async fn remove_user(
    State(state): State<Arc<ServerState>>,
    Account(_): Account<User>,
    Log(mut log): Log,
    Path(id): Path<u32>,
) -> Result<WithLog<()>> {
    if let Some(user) = &mut User::find_by_id(state.db.handle(), id).await? {
        user.deleted_at = Some(chrono::Local::now());
        let log = log
            .with_entry("Removing user")
            .with_data(&user.clone())?
            .clone();
        user.save(state.db.handle()).await?;
        Ok(state.with_log(Ok(()), log))
    } else {
        Err(anyhow!("invalid user").into())
    }
}

pub(crate) async fn list_users(
    State(state): State<Arc<ServerState>>,
    Account(_): Account<User>,
    Cbor(pagination): Cbor<Option<Pagination>>,
) -> Result<CborOut<Vec<User>>> {
    if let Some(pagination) = pagination {
        let mut query = User::all();

        if let Some(per_page) = pagination.per_page {
            query = query.limit(per_page.into());
        }

        if let Some(page) = pagination.page {
            query = query.offset((page * pagination.per_page.unwrap_or(20)).into());
        }

        Ok(CborOut(query.run(state.db.handle()).await?.into_inners()))
    } else {
        Ok(CborOut(
            User::all().run(state.db.handle()).await?.into_inners(),
        ))
    }
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
    Log(mut log): Log,
    Cbor(mut user): Cbor<User>,
) -> Result<WithLog<()>> {
    if User::find_by_id(state.db.handle(), id).await?.is_some() {
        // if we got the record, the id is correct
        user.id = id;
        user.validate()?;

        // crypt the plaintext password if it is set
        if let Some(password) = &user.plaintext_password {
            user.set_password(password.clone())?;
        }

        user.plaintext_password = None; // NOTE: so it doesn't appear in the logging that follows

        let log = log.with_entry("Modifying user").with_data(&user)?.clone();

        // welds doesn't realize the fields have already changed, these two lines force it to see
        // it.
        let mut dbstate: DbState<User> = DbState::db_loaded(user.clone());
        dbstate.replace_inner(user);
        Ok(state.with_log(Ok(dbstate.save(state.db.handle()).await?), log))
    } else {
        Err(anyhow!("invalid user").into())
    }
}

//
// Authentication
//

pub(crate) async fn login(
    State(state): State<Arc<ServerState>>,
    Log(mut log): Log,
    Cbor(form): Cbor<Authentication>,
) -> Result<WithLog<CborOut<Token>>> {
    form.validate()?;

    let users = User::all()
        .where_col(|c| c.username.equal(&form.username))
        .run(state.db.handle())
        .await?;

    let mut map: HashMap<&str, &str> = HashMap::default();
    map.insert("username", &form.username);

    let user = match users.first() {
        Some(user) => user.deref(),
        None => {
            let log = log
                .with_entry("Unsuccessful login attempt")
                .with_data(&map)?
                .clone();
            return Ok(state.with_log(Err(anyhow!("invalid login").into()), log));
        }
    };

    let log = log.from_user(user);

    if user.login(form.password).is_err() {
        let log = log
            .with_entry("Unsuccessful login attempt")
            .with_data(&map)?
            .clone();

        return Ok(state.with_log(Err(anyhow!("invalid login").into()), log));
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

    let log = log.with_entry("Successfully logged in").clone();

    Ok(state.with_log(Ok(CborOut(Token { token: jwt.into() })), log))
}

pub(crate) async fn me(
    State(_): State<Arc<ServerState>>,
    Account(user): Account<User>,
) -> Result<CborOut<User>> {
    Ok(CborOut(user))
}
