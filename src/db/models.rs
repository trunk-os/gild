use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use welds::WeldsModel;

#[derive(Debug, WeldsModel)]
#[welds(table = "users")]
#[welds(HasMany(sessions, Session, "user_id"))]
pub(crate) struct User {
    #[welds(rename = "user_id")]
    #[welds(primary_key)]
    id: uuid::Uuid,
    username: String,
    realname: Option<String>,
    email: Option<String>,
    phone: Option<String>,
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

#[derive(Debug, WeldsModel)]
#[welds(table = "sessions")]
#[welds(BelongsTo(user, User, "user_id"))]
pub(crate) struct Session {
    #[welds(rename = "session_id")]
    #[welds(primary_key)]
    id: uuid::Uuid,
    secret: Vec<u8>,
    expires: chrono::DateTime<chrono::Local>,
    user_id: uuid::Uuid,
}

#[cfg(test)]
mod tests {}
