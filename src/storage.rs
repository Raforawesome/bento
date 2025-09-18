pub mod memstore;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

/*
 * Newtype wrappers for strong typing
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Username(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PasswordHash(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionToken(pub String); // bearer

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionIp(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    fn create_user(
        &self,
        username: Username,
        pass_hash: PasswordHash,
    ) -> impl Future<Output = Result<User, AuthError>> + Send;

    fn get_user_by_id(&self, id: &UserId) -> impl Future<Output = Result<User, AuthError>> + Send;

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

impl SessionToken {
    pub fn new() -> Self {
        let mut buf = [0_u8; 256];
        if OsRng.try_fill_bytes(&mut buf).is_ok() {
            SessionToken(Base64Url.encode(&buf))
        } else {
            panic!("Failed to generate secure numbers from the operating system.");
        }
    }
}
