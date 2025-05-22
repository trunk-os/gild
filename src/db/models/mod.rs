#[cfg(test)]
mod tests;
use std::{collections::BTreeMap, ops::Deref};

use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use validator::Validate;
use welds::{state::DbState, WeldsModel};

use super::DB;

#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    WeldsModel,
    Default,
    Serialize,
    Deserialize,
    Validate,
)]
#[welds(table = "users")]
#[welds(HasMany(sessions, Session, "user_id"))]
pub(crate) struct User {
    #[welds(rename = "user_id")]
    #[welds(primary_key)]
    pub id: u32,
    #[validate(length(min = 3, max = 30))]
    pub username: String,
    #[validate(length(min = 3, max = 50))]
    pub realname: Option<String>,
    #[validate(length(min = 6, max = 100), email)] // a@b.cd
    pub email: Option<String>,
    #[validate(length(min = 10, max = 20))]
    pub phone: Option<String>,
    #[welds(ignore)]
    #[serde(rename = "password")]
    #[validate(length(min = 8, max = 100))]
    pub plaintext_password: Option<String>,
    #[serde(skip)]
    pub(crate) password: String,
}

impl User {
    pub(crate) fn login(&mut self, password: String) -> Result<()> {
        let crypt = Argon2::default();
        let parsed = PasswordHash::new(&self.password).map_err(|e| anyhow!(e.to_string()))?;
        Ok(crypt
            .verify_password(password.as_bytes(), &parsed)
            .map_err(|e| anyhow!(e.to_string()))?)
    }

    pub(crate) fn set_password(&mut self, password: String) -> Result<()> {
        let crypt = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        self.password = crypt
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!(e.to_string()))?
            .to_string();
        Ok(())
    }
}

#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    WeldsModel,
    Default,
    Serialize,
    Deserialize,
    Validate,
)]
#[welds(table = "sessions")]
#[welds(BelongsTo(user, User, "user_id"))]
pub(crate) struct Session {
    #[welds(rename = "session_id")]
    #[welds(primary_key)]
    pub id: u32,
    pub expires: chrono::DateTime<chrono::Local>,
    pub user_id: u32,
}

type JWTClaims<'a> = BTreeMap<&'a str, String>;

const JWT_SESSION_ID_KEY: &str = "kid";
const JWT_EXPIRATION_TIME: &str = "exp";

impl Session {
    pub fn new_assigned(user: User) -> DbState<Self> {
        DbState::new_uncreated(Self {
            user_id: user.id,
            expires: chrono::Local::now()
                .checked_add_signed(chrono::TimeDelta::days(7))
                .unwrap()
                .into(),
            ..Default::default()
        })
    }

    pub(crate) async fn from_jwt<'a>(db: &'a DB, claims: JWTClaims<'a>) -> Result<DbState<Self>> {
        let session_id: u32 = claims[JWT_SESSION_ID_KEY].parse()?;
        let list = Self::all()
            .where_col(|c| c.id.equal(session_id))
            .run(db.handle())
            .await?;
        let session = match list.first() {
            Some(inner) => inner.deref(),
            None => return Err(anyhow!("invalid session")),
        };

        let expires: chrono::DateTime<chrono::Local> = claims[JWT_EXPIRATION_TIME].parse()?;
        if session.expires.signed_duration_since(expires).num_seconds() < 0 {
            return Err(anyhow!("session is expired"));
        }
        Ok(DbState::db_loaded(session.clone()))
    }

    pub(crate) fn to_jwt(&self) -> JWTClaims {
        let mut claims = JWTClaims::default();
        claims.insert(JWT_SESSION_ID_KEY, self.id.to_string());
        claims.insert(JWT_EXPIRATION_TIME, self.expires.to_rfc3339());
        claims
    }
}
