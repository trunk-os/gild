#[cfg(test)]
mod tests;
use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use welds::WeldsModel;

#[derive(Debug, WeldsModel, Serialize, Deserialize)]
#[welds(table = "users")]
#[welds(HasMany(sessions, Session, "user_id"))]
pub(crate) struct User {
    #[welds(rename = "user_id")]
    #[welds(primary_key)]
    pub id: u32,
    pub username: String,
    pub realname: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    #[serde(skip)]
    password: Vec<u8>,
}

impl User {
    pub(crate) fn login(&mut self, password: String) -> Result<()> {
        let crypt = Argon2::default();
        let s = String::from_utf8(self.password.clone())?;
        let parsed = PasswordHash::new(s.as_str()).map_err(|e| anyhow!(e.to_string()))?;
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
            .to_string()
            .as_bytes()
            .to_vec();
        Ok(())
    }
}

#[derive(Debug, WeldsModel, Serialize, Deserialize)]
#[welds(table = "sessions")]
#[welds(BelongsTo(user, User, "user_id"))]
pub(crate) struct Session {
    #[welds(rename = "session_id")]
    #[welds(primary_key)]
    pub id: u32,
    pub expires: chrono::DateTime<chrono::Local>,
    pub user_id: u32,
    #[serde(skip)]
    secret: Vec<u8>,
}
