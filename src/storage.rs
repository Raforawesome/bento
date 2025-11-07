pub mod memstore;

use argon2::{
    Argon2,
    password_hash::{
        PasswordHash as ArgonHash, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::OsRng as ArgonRng,
    },
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE as Base64Url};
use rand::{TryRngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;
use tracing::debug;
use uuid::Uuid;

/*
 * Newtype wrappers for strong typing
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Username(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PasswordHash(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionToken(pub String); // bearer

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionIp(pub String);

/// An enum to represent a user's permission level.
/// - Admins:
///   Can create other users
///
///   TODO: Add the ability to manage other user's workspaces
///
/// - Users:
///   Can manage their own workspaces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Admin,
    User,
}

/// Main user abstraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: Username,
    pub password_hash: PasswordHash,
    pub role: Role,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("User already exists")]
    UserExists,
    #[error("User not found")]
    NotFound,
    #[error("Invalid session")]
    InvalidSession,
    #[error("Maximum active sessions reached")]
    SessionLimitReached,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: SessionToken,
    pub user_id: UserId,
    pub ip: SessionIp,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
}

pub trait AuthStore: Send + Sync {
    fn max_sessions_per_user(&self) -> usize;

    fn create_user(
        &self,
        username: Username,
        pass_hash: PasswordHash,
        role: Role,
    ) -> impl Future<Output = Result<User, AuthError>> + Send;

    fn create_standard_user(
        &self,
        username: Username,
        pass_hash: PasswordHash,
    ) -> impl Future<Output = Result<User, AuthError>> + Send {
        self.create_user(username, pass_hash, Role::User)
    }

    fn create_admin(
        &self,
        username: Username,
        pass_hash: PasswordHash,
    ) -> impl Future<Output = Result<User, AuthError>> + Send {
        self.create_user(username, pass_hash, Role::Admin)
    }

    fn get_user_by_id(&self, id: &UserId) -> impl Future<Output = Result<User, AuthError>> + Send;

    fn get_user_by_username(
        &self,
        username: &Username,
    ) -> impl Future<Output = Result<User, AuthError>> + Send;

    fn set_password_hash(
        &self,
        id: &UserId,
        new_hash: PasswordHash,
    ) -> impl Future<Output = Result<PasswordHash, AuthError>> + Send;

    fn delete_user(&self, id: &UserId) -> impl Future<Output = Result<(), AuthError>> + Send;

    fn issue_session(
        &self,
        id: &UserId,
        ip: SessionIp,
    ) -> impl Future<Output = Result<Session, AuthError>> + Send;

    fn fetch_session(
        &self,
        token: &SessionToken,
    ) -> impl Future<Output = Result<Session, AuthError>> + Send;

    fn extend_session(
        &self,
        token: &SessionToken,
    ) -> impl Future<Output = Result<Session, AuthError>> + Send;

    fn revoke_session(
        &self,
        token: &SessionToken,
    ) -> impl Future<Output = Result<(), AuthError>> + Send;
}

/*
 * Implementations on newtype wrappers
 */
impl UserId {
    pub fn new() -> Self {
        UserId(Uuid::now_v7())
    }
}

impl Default for UserId {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionToken {
    pub fn new() -> Self {
        let mut buf = [0_u8; 120];
        if OsRng.try_fill_bytes(&mut buf).is_ok() {
            SessionToken(Base64Url.encode(buf))
        } else {
            panic!("Failed to generate secure numbers from the operating system.");
        }
    }
}

impl Default for SessionToken {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PasswordHash {
    pub fn from<B: AsRef<[u8]>>(password: B) -> Self {
        let pass_bytes: &[u8] = password.as_ref();
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(pass_bytes, &salt).unwrap().to_string();
        Self(password_hash)
    }

    pub fn verify<B: AsRef<[u8]>>(&self, password: B) -> bool {
        let pass_bytes: &[u8] = password.as_ref();
        let parsed_hash = ArgonHash::new(&self.0);
        if parsed_hash.is_err() {
            debug!("Failed to parse stored password hash, possible corruption?");
            return false;
        }

        let parsed_hash = parsed_hash.unwrap();
        Argon2::default()
            .verify_password(pass_bytes, &parsed_hash)
            .is_ok()
    }
}

impl TryFrom<&[u8]> for PasswordHash {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let pass_bytes: &[u8] = value;
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(pass_bytes, &salt)?.to_string();
        Ok(Self(password_hash))
    }
}

impl TryFrom<&str> for PasswordHash {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let pass_bytes: &[u8] = value.as_bytes();
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(pass_bytes, &salt)?.to_string();
        Ok(Self(password_hash))
    }
}
