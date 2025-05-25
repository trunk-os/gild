use std::sync::Arc;

use anyhow::anyhow;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use hmac::{Hmac, Mac};
use jwt::{Header, Token, Verified, VerifyWithKey};
use problem_details::ProblemDetails;

use crate::db::models::{AuditLog, JWTClaims, Session, User};

use super::ServerState;

pub(crate) type Result<T> = core::result::Result<T, AppError>;

#[derive(Debug, Clone, Default)]
pub(crate) struct AppError(ProblemDetails);

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        let e: anyhow::Error = err.into();
        Self(
            ProblemDetails::new()
                .with_detail(e.to_string())
                .with_title("Uncategorized Error"),
        )
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.0.into_response()
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct CborOut<T>(pub T);

impl<T> IntoResponse for CborOut<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> Response {
        let mut inner = Vec::with_capacity(65535);
        let mut buf = std::io::Cursor::new(&mut inner);

        if let Err(e) = ciborium::into_writer(&self.0, &mut buf) {
            return Into::<AppError>::into(anyhow!(e)).into_response();
        }

        Response::builder()
            .header("Content-Type", "application/cbor")
            .body(axum::body::Body::from(buf.into_inner().to_vec()))
            .unwrap()
    }
}

pub(crate) struct Account<T>(pub T);

async fn read_jwt(parts: &mut Parts, state: &Arc<ServerState>) -> Result<Option<User>> {
    // FIXME: we want to hide the error from the end user to avoid giving them information about this
    // process. We should, however, log the errors for debugging purposes, which isn't done yet.
    let err = AppError(
        ProblemDetails::new()
            .with_detail("Please enter correct credentials")
            .with_status(http::StatusCode::UNAUTHORIZED)
            .with_title("Invalid Login"),
    );

    let token = parts
        .headers
        .get(http::header::AUTHORIZATION)
        .ok_or(err.clone())?;

    let token = token.to_str()?.strip_prefix("Bearer ").unwrap();
    let signing_key: Hmac<sha2::Sha384> =
        Hmac::new_from_slice(&state.config.signing_key).map_err(|_| err.clone())?;
    let token: Token<Header, JWTClaims, Verified> = token
        .verify_with_key(&signing_key)
        .map_err(|_| err.clone())?;

    let session = Session::from_jwt(&state.db, token.claims().clone())
        .await
        .map_err(|_| err.clone())?;
    // FIXME not sure why relationships are useless here
    if let Some(user) = User::find_by_id(state.db.handle(), session.user_id)
        .await
        .map_err(|_| err.clone())?
    {
        Ok(Some(user.into_inner()))
    } else {
        Ok(None)
    }
}

impl FromRequestParts<Arc<ServerState>> for Account<User> {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<ServerState>,
    ) -> core::result::Result<Self, Self::Rejection> {
        if let Some(user) = read_jwt(parts, state).await? {
            Ok(Account(user))
        } else {
            Err(anyhow!("user is not logged in").into())
        }
    }
}

impl FromRequestParts<Arc<ServerState>> for Account<Option<User>> {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<ServerState>,
    ) -> core::result::Result<Self, Self::Rejection> {
        Ok(Account(read_jwt(parts, state).await.unwrap_or_default()))
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Log(pub(crate) AuditLog);

impl FromRequestParts<Arc<ServerState>> for Log {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<ServerState>,
    ) -> core::result::Result<Self, Self::Rejection> {
        let mut this = Self(
            AuditLog::builder()
                .from_uri(parts.uri.clone())
                .from_headers(parts.headers.clone())
                .clone(),
        );

        if let Some(user) = read_jwt(parts, state).await.unwrap_or_default() {
            this.0 = this.0.from_user(&user).clone();
        }

        Ok(this)
    }
}
