pub mod memstore;
pub mod redbstore;

use crate::types::{PasswordHash, Role, Session, SessionId, SessionIp, User, UserId, Username};
use thiserror::Error;

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
    #[error("Internal error: {0}")]
    Internal(String),
}

// Implement From traits for redb error types
#[cfg(feature = "ssr")]
impl From<redb::TransactionError> for AuthError {
    fn from(err: redb::TransactionError) -> Self {
        AuthError::Internal(err.to_string())
    }
}

#[cfg(feature = "ssr")]
impl From<redb::TableError> for AuthError {
    fn from(err: redb::TableError) -> Self {
        AuthError::Internal(err.to_string())
    }
}

#[cfg(feature = "ssr")]
impl From<redb::CommitError> for AuthError {
    fn from(err: redb::CommitError) -> Self {
        AuthError::Internal(err.to_string())
    }
}

#[cfg(feature = "ssr")]
impl From<redb::StorageError> for AuthError {
    fn from(err: redb::StorageError) -> Self {
        AuthError::Internal(err.to_string())
    }
}

#[cfg(feature = "ssr")]
impl From<redb::DatabaseError> for AuthError {
    fn from(err: redb::DatabaseError) -> Self {
        AuthError::Internal(err.to_string())
    }
}

// Implement From trait for bincode error types
#[cfg(feature = "ssr")]
impl From<bincode::error::EncodeError> for AuthError {
    fn from(err: bincode::error::EncodeError) -> Self {
        AuthError::Internal(format!("Serialization error: {}", err))
    }
}

#[cfg(feature = "ssr")]
impl From<bincode::error::DecodeError> for AuthError {
    fn from(err: bincode::error::DecodeError) -> Self {
        AuthError::Internal(format!("Deserialization error: {}", err))
    }
}

// Implement From trait for tokio JoinError
#[cfg(feature = "ssr")]
impl From<tokio::task::JoinError> for AuthError {
    fn from(err: tokio::task::JoinError) -> Self {
        AuthError::Internal(format!("Task join error: {}", err))
    }
}

pub trait AuthStore: Send + Sync {
    fn max_sessions_per_user(&self) -> usize;

    fn create_user(
        &self,
        username: &Username,
        pass_hash: PasswordHash,
        role: Role,
    ) -> impl Future<Output = Result<User, AuthError>> + Send;

    fn create_standard_user(
        &self,
        username: &Username,
        pass_hash: PasswordHash,
    ) -> impl Future<Output = Result<User, AuthError>> + Send {
        self.create_user(username, pass_hash, Role::User)
    }

    fn create_admin(
        &self,
        username: &Username,
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
        token: &SessionId,
    ) -> impl Future<Output = Result<Session, AuthError>> + Send;

    fn extend_session(
        &self,
        token: &SessionId,
    ) -> impl Future<Output = Result<Session, AuthError>> + Send;

    fn revoke_session(
        &self,
        token: &SessionId,
    ) -> impl Future<Output = Result<(), AuthError>> + Send;
}
