use papaya::HashMap;
use time::{Duration, OffsetDateTime};
use tracing::{debug, instrument};

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
            debug!("User creation failed: username already exists");
            Err(AuthError::UserExists)
        } else {
            let user = User {
                id: UserId::new(),
                role: Role::User,
                username,
                password_hash,
            };
            debug!(user_id = %user.id.0, "Creating new user");
            user_map.insert(user.id.clone(), user.clone());
            let result = user_map.get(&user.id);
            debug!(?result, "from map after insert");
            debug!(user_id = %user.id.0, "User created successfully");
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
        debug!("Looking up user by username");
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

        let session = Session {
            token: SessionToken::new(),
            user_id: *id,
            ip,
            created_at: now,
            expires_at: expires,
        };

        let session_map = self.sessions.pin();
        session_map.insert(session.token.clone(), session.clone());
        debug!(
            user_id = %id.0,
            token_len = session.token.0.len(),
            expires_at = %expires,
            "Session created successfully"
        );
        Ok(session)
    }

    async fn fetch_session(&self, token: &SessionToken) -> Result<Session, AuthError> {
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
                return Ok(session.clone());
            } else {
                debug!(
                    user_id = %session.user_id.0,
                    "Session expired, removing"
                );
                session_map.remove(token);
                return Err(AuthError::InvalidSession);
            }
        } else {
            debug!("Session not found");
            return Err(AuthError::InvalidSession);
        }
    }

    async fn extend_session(&self, token: &SessionToken) -> Result<Session, AuthError> {
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

    async fn revoke_session(&self, token: &SessionToken) -> Result<(), AuthError> {
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
