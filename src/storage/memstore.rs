use papaya::HashMap;
use time::{Duration, OffsetDateTime};
use tracing::{debug, trace};

use super::{AuthError, AuthStore};
use crate::types::{PasswordHash, Role, Session, SessionId, SessionIp, User, UserId, Username};

/// An in-memory auth store designed for non-persistent usage.
#[derive(Clone)]
pub struct MemoryAuthStore {
    pub(self) users: HashMap<UserId, User>,
    pub(self) sessions: HashMap<SessionId, Session>,
    pub(self) max_sessions_per_user: usize,
}

impl MemoryAuthStore {
    pub fn new(max_sessions_per_user: usize) -> Self {
        MemoryAuthStore {
            users: HashMap::new(),
            sessions: HashMap::new(),
            max_sessions_per_user,
        }
    }

    pub fn new_unbounded() -> Self {
        Self::new(usize::MAX)
    }
}

impl Default for MemoryAuthStore {
    fn default() -> Self {
        Self::new(usize::MAX)
    }
}

impl AuthStore for MemoryAuthStore {
    fn max_sessions_per_user(&self) -> usize {
        self.max_sessions_per_user
    }

    async fn create_user(
        &self,
        username: &Username,
        password_hash: PasswordHash,
        role: Role,
    ) -> Result<User, AuthError> {
        let user_map = self.users.pin();

        if user_map.values().any(|u| &u.username == username) {
            debug!("User creation failed: username already exists");
            Err(AuthError::UserExists)
        } else {
            let user = User {
                id: UserId::new(),
                role,
                username: username.clone(),
                password_hash,
            };
            trace!(user_id = %user.id.0, "Creating new user");
            user_map.insert(user.id, user.clone());
            let result = user_map.get(&user.id);
            trace!(?result, "from map after insert");
            trace!(user_id = %user.id.0, "User created successfully");
            Ok(user)
        }
    }

    async fn get_user_by_id(&self, id: &UserId) -> Result<User, AuthError> {
        debug!(user_id = %id.0, "Looking up user by ID");
        let user_map = self.users.pin();
        let result = user_map.get(id).cloned().ok_or(AuthError::NotFound);

        match &result {
            Ok(user) => debug!(user_id = %id.0, username = %user.username.0, "User found"),
            Err(_) => debug!(user_id = %id.0, "User not found"),
        }

        result
    }

    async fn get_user_by_username(&self, username: &Username) -> Result<User, AuthError> {
        let user_map = self.users.pin();
        let result = user_map
            .values()
            .find(|&u| &u.username == username)
            .cloned()
            .ok_or(AuthError::NotFound);

        match &result {
            Ok(user) => debug!(user_id = %user.id.0, "User found"),
            Err(_) => debug!("User not found"),
        }

        result
    }

    async fn set_password_hash(
        &self,
        id: &UserId,
        new_hash: PasswordHash,
    ) -> Result<PasswordHash, AuthError> {
        debug!(user_id = %id.0, "Updating user password");
        let user_map = self.users.pin();
        let result = user_map
            .update(*id, |u| User {
                id: *id,
                username: u.username.clone(),
                password_hash: new_hash.clone(),
                role: u.role,
            })
            .map(|_| new_hash)
            .ok_or(AuthError::NotFound);

        match &result {
            Ok(_) => debug!(user_id = %id.0, "Password updated successfully"),
            Err(_) => debug!(user_id = %id.0, "Password update failed: user not found"),
        }

        result
    }

    async fn delete_user(&self, id: &UserId) -> Result<(), AuthError> {
        debug!(user_id = %id.0, "Deleting user");
        let user_map = self.users.pin();
        if user_map.remove(id).is_some() {
            debug!(user_id = %id.0, "User deleted successfully");
            Ok(())
        } else {
            debug!(user_id = %id.0, "Delete failed: user not found");
            Err(AuthError::NotFound)
        }
    }

    async fn issue_session(&self, id: &UserId, ip: SessionIp) -> Result<Session, AuthError> {
        debug!(user_id = %id.0, ip = %ip.0, "Issuing new session");
        let now = OffsetDateTime::now_utc();
        let expires = now + Duration::hours(1);

        let session_map = self.sessions.pin();

        if self.max_sessions_per_user != usize::MAX {
            let active_sessions = session_map
                .values()
                .filter(|session| session.user_id == *id && session.expires_at > now)
                .count();

            if active_sessions >= self.max_sessions_per_user {
                debug!(
                    user_id = %id.0,
                    max = self.max_sessions_per_user,
                    "Session limit reached"
                );
                return Err(AuthError::SessionLimitReached);
            }
        }

        let session = Session {
            id: SessionId::new(),
            user_id: *id,
            ip,
            created_at: now,
            expires_at: expires,
        };

        session_map.insert(session.id.clone(), session.clone());
        debug!(
            user_id = %id.0,
            token_len = session.id.0.len(),
            expires_at = %expires,
            "Session created successfully"
        );
        Ok(session)
    }

    async fn fetch_session(&self, token: &SessionId) -> Result<Session, AuthError> {
        debug!(token_len = token.0.len(), "Fetching session");
        let session_map = self.sessions.pin();

        if let Some(session) = session_map.get(token) {
            let now = OffsetDateTime::now_utc();
            if session.expires_at > now {
                debug!(
                    user_id = %session.user_id.0,
                    expires_in_secs = (session.expires_at - now).whole_seconds(),
                    "Valid session found"
                );
                Ok(session.clone())
            } else {
                debug!(
                    user_id = %session.user_id.0,
                    "Session expired, removing"
                );
                session_map.remove(token);
                Err(AuthError::InvalidSession)
            }
        } else {
            debug!("Session not found");
            Err(AuthError::InvalidSession)
        }
    }

    async fn extend_session(&self, token: &SessionId) -> Result<Session, AuthError> {
        debug!(token_len = token.0.len(), "Extending session");
        let session_map = self.sessions.pin();

        if let Some(mut session) = session_map.get(token).cloned() {
            let now = OffsetDateTime::now_utc();
            if session.expires_at > now {
                let new_expires = now + Duration::hours(1);
                debug!(
                    user_id = %session.user_id.0,
                    old_expires = %session.expires_at,
                    new_expires = %new_expires,
                    "Extending session"
                );
                session.expires_at = new_expires;
                session_map.insert(token.clone(), session.clone());
                Ok(session)
            } else {
                debug!(user_id = %session.user_id.0, "Cannot extend expired session, removing");
                session_map.remove(token);
                Err(AuthError::InvalidSession)
            }
        } else {
            debug!("Session not found for extension");
            Err(AuthError::InvalidSession)
        }
    }

    async fn revoke_session(&self, token: &SessionId) -> Result<(), AuthError> {
        debug!(token_len = token.0.len(), "Revoking session");
        let session_map = self.sessions.pin();
        if let Some(session) = session_map.remove(token) {
            debug!(user_id = %session.user_id.0, "Session revoked successfully");
            Ok(())
        } else {
            debug!("Session not found for revocation");
            Err(AuthError::InvalidSession)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;

    #[tokio::test]
    async fn enforces_session_limit() {
        let store = MemoryAuthStore::new(1);
        let user_id = UserId::new();
        let ip = SessionIp(IpAddr::from([127, 0, 0, 1]));

        let first = store
            .issue_session(&user_id, ip.clone())
            .await
            .expect("first session should succeed");

        match store.issue_session(&user_id, ip.clone()).await {
            Err(AuthError::SessionLimitReached) => {}
            other => panic!("expected session limit error, got {other:?}"),
        }

        store
            .revoke_session(&first.id)
            .await
            .expect("revocation should succeed");

        store
            .issue_session(&user_id, ip)
            .await
            .expect("session after revocation should succeed");
    }
}
