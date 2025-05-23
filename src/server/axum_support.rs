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

use crate::db::models::{JWTClaims, Session, User};

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
        match ciborium::into_writer(&self.0, &mut buf) {
            Err(e) => return Into::<AppError>::into(anyhow!(e)).into_response(),
            _ => {}
        }

        Response::builder()
            .body(axum::body::Body::from(buf.into_inner().to_vec()))
            .unwrap()
    }
}

pub(crate) struct Account<T>(pub T);

async fn read_jwt(parts: &mut Parts, state: &Arc<ServerState>) -> Result<Option<User>> {
    let err: AppError = anyhow!("invalid cookie").into();
    let cookies = parts.headers.get(http::header::COOKIE).ok_or(err.clone())?;
    let cookies = cookies.to_str().map_err(|_| err.clone())?.split("; ");
    let mut jwt: Option<Token<Header, JWTClaims, Verified>> = None;
    for cookie in cookies {
        let parts = cookie.splitn(2, "=").collect::<Vec<&str>>();
        if parts.len() != 2 {
            return Err(err);
        }

        if parts[0] == "jwt" {
            let signing_key: Hmac<sha2::Sha384> =
                Hmac::new_from_slice(&state.config.signing_key).map_err(|_| err.clone())?;
            let token: Token<Header, JWTClaims, Verified> = parts[1]
                .verify_with_key(&signing_key)
                .map_err(|_| err.clone())?;
            jwt.replace(token);
            break;
        }
    }

    if let Some(jwt) = jwt {
        let session = Session::from_jwt(&state.db, jwt.claims().clone())
            .await
            .map_err(|_| err.clone())?;
        // FIXME not sure why relationships are useless here
        if let Some(user) = User::find_by_id(state.db.handle(), session.user_id)
            .await
            .map_err(|_| err.clone())?
        {
            return Ok(Some(user.into_inner()));
        } else {
            return Err(err.clone());
        }
    } else {
        return Ok(None);
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
        Ok(Account(match read_jwt(parts, state).await {
            Ok(x) => x,
            Err(_) => None,
        }))
    }
}
