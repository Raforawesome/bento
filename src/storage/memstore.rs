use papaya::HashMap;

use super::{
    AuthError, AuthStore, PasswordHash, Role, Session, SessionToken, User, UserId, Username,
};

#[derive(Clone)]
pub struct MemoryAuthStore {
    pub(self) users: HashMap<UserId, User>,
    pub(self) sessions: HashMap<SessionToken, Session>,
}

impl AuthStore for MemoryAuthStore {
    async fn create_user(
        &self,
        username: Username,
        password_hash: PasswordHash,
    ) -> Result<User, AuthError> {
        todo!()
    }

    async fn get_user_by_id(&self, id: &UserId) -> Result<User, AuthError> {
        todo!()
    }

    async fn set_password_hash(
        &self,
        id: &UserId,
        new_hash: super::PasswordHash,
    ) -> Result<super::PasswordHash, AuthError> {
        todo!()
    }

    async fn delete_user(&self, id: &UserId) -> Result<(), AuthError> {
        todo!()
    }

    async fn create_session(
        &self,
        id: &UserId,
        ip: super::SessionIp,
    ) -> Result<Session, AuthError> {
        todo!()
    }

    async fn fetch_session(&self, token: &SessionToken) -> Result<Session, AuthError> {
        todo!()
    }

    async fn extend_session(&self, token: &SessionToken) -> Result<Session, AuthError> {
        todo!()
    }

    async fn revoke_session(&self, token: &SessionToken) -> Result<(), AuthError> {
        todo!()
    }
}
