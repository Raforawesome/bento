use papaya::HashMap;
use time::{Duration, OffsetDateTime};

use super::{
    AuthError, AuthStore, PasswordHash, Role, Session, SessionIp, SessionToken, User, UserId,
    Username,
};

#[derive(Clone)]
pub struct MemoryAuthStore {
    pub(self) users: HashMap<UserId, User>,
    pub(self) sessions: HashMap<SessionToken, Session>,
}

impl MemoryAuthStore {
    pub fn new() -> Self {
        MemoryAuthStore {
            users: HashMap::new(),
            sessions: HashMap::new(),
        }
    }
}

impl AuthStore for MemoryAuthStore {
    async fn create_user(
        &self,
        username: Username,
        password_hash: PasswordHash,
    ) -> Result<User, AuthError> {
        let user_map = self.users.pin();

        if user_map.values().any(|u| u.username == username) {
            Err(AuthError::UserExists)
        } else {
            let user = User {
                id: UserId::new(),
                role: Role::User,
                username,
                password_hash,
            };
            user_map.insert(user.id.clone(), user.clone());
            Ok(user)
        }
    }

    async fn get_user_by_id(&self, id: &UserId) -> Result<User, AuthError> {
        let user_map = self.users.pin();
        user_map.get(id).cloned().ok_or(AuthError::NotFound)
    }

    async fn get_user_by_username(&self, username: &Username) -> Result<User, AuthError> {
        let user_map = self.users.pin();
        user_map
            .values()
            .find(|&u| &u.username == username)
            .cloned()
            .ok_or(AuthError::NotFound)
    }

    async fn set_password_hash(
        &self,
        id: &UserId,
        new_hash: PasswordHash,
    ) -> Result<PasswordHash, AuthError> {
        let user_map = self.users.pin();
        user_map
            .update(*id, |u| User {
                id: *id,
                username: u.username.clone(),
                password_hash: new_hash.clone(),
                role: u.role,
            })
            .map(|_| new_hash)
            .ok_or(AuthError::NotFound)
    }

    async fn delete_user(&self, id: &UserId) -> Result<(), AuthError> {
        let user_map = self.users.pin();
        if user_map.remove(id).is_some() {
            Ok(())
        } else {
            Err(AuthError::NotFound)
        }
    }

    async fn issue_session(&self, id: &UserId, ip: SessionIp) -> Result<Session, AuthError> {
        let session = Session {
            token: SessionToken::new(),
            user_id: *id,
            ip,
            created_at: OffsetDateTime::now_utc(),
            expires_at: OffsetDateTime::now_utc() + Duration::hours(1),
        };
        let session_map = self.sessions.pin();
        session_map.insert(session.token.clone(), session.clone());
        Ok(session)
    }

    async fn fetch_session(&self, token: &SessionToken) -> Result<Session, AuthError> {
        let session_map = self.sessions.pin();

        if let Some(session) = session_map.get(token) {
            if session.expires_at > OffsetDateTime::now_utc() {
                return Ok(session.clone());
            } else {
                session_map.remove(token);
                return Err(AuthError::InvalidSession);
            }
        } else {
            return Err(AuthError::InvalidSession);
        }
    }

    async fn extend_session(&self, token: &SessionToken) -> Result<Session, AuthError> {
        let session_map = self.sessions.pin();
        if let Some(mut session) = session_map.get(token).cloned() {
            let now = OffsetDateTime::now_utc();
            if session.expires_at > now {
                session.expires_at = now + Duration::hours(1);
                session_map.insert(token.clone(), session.clone());
                Ok(session)
            } else {
                session_map.remove(token);
                Err(AuthError::InvalidSession)
            }
        } else {
            Err(AuthError::InvalidSession)
        }
    }

    async fn revoke_session(&self, token: &SessionToken) -> Result<(), AuthError> {
        let session_map = self.sessions.pin();
        if session_map.remove(token).is_some() {
            Ok(())
        } else {
            Err(AuthError::InvalidSession)
        }
    }
}
