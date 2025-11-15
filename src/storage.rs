pub mod memstore;

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
