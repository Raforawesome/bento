use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Serialize, de::DeserializeOwned};
use std::path::Path;
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::task::spawn_blocking;
use tracing::{debug, error, trace};

use super::{AuthError, AuthStore};
use crate::config::SESSION_DURATION;
use crate::types::{PasswordHash, Role, Session, SessionId, SessionIp, User, UserId, Username};

const USERS_TABLE: TableDefinition<u128, Vec<u8>> = TableDefinition::new("users");
const USERNAMES_TABLE: TableDefinition<&str, u128> = TableDefinition::new("usernames");
const SESSIONS_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("sessions");
/// Maps user_id (u128) -> Vec<session_id strings> for O(1) session counting
const USER_SESSIONS_INDEX: TableDefinition<u128, Vec<u8>> = TableDefinition::new("user_sessions");

#[derive(Clone)]
pub struct RedbAuthStore {
    db: Arc<Database>,
    max_sessions_per_user: usize,
}

impl RedbAuthStore {
    pub fn new(path: impl AsRef<Path>, max_sessions_per_user: usize) -> Result<Self, AuthError> {
        let db = Database::create(path)?;

        // Initialize tables
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(USERS_TABLE)?;
            let _ = write_txn.open_table(USERNAMES_TABLE)?;
            let _ = write_txn.open_table(SESSIONS_TABLE)?;
            let _ = write_txn.open_table(USER_SESSIONS_INDEX)?;
        }
        write_txn.commit()?;

        Ok(Self {
            db: Arc::new(db),
            max_sessions_per_user,
        })
    }

    fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, AuthError> {
        Ok(bincode::serde::encode_to_vec(
            value,
            bincode::config::standard(),
        )?)
    }

    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, AuthError> {
        let (result, _) = bincode::serde::decode_from_slice(bytes, bincode::config::standard())?;
        Ok(result)
    }

    /// Gets the list of session IDs for a user from the index
    fn get_user_session_ids(
        table: &impl ReadableTable<u128, Vec<u8>>,
        user_id: u128,
    ) -> Result<Vec<String>, AuthError> {
        if let Some(bytes) = table.get(user_id)? {
            Self::deserialize(&bytes.value())
        } else {
            Ok(Vec::new())
        }
    }

    /// Adds a session ID to the user's session index
    fn add_session_to_index(
        table: &mut redb::Table<u128, Vec<u8>>,
        user_id: u128,
        session_id: &str,
    ) -> Result<(), AuthError> {
        let mut session_ids = Self::get_user_session_ids(table, user_id)?;
        session_ids.push(session_id.to_string());
        let bytes = Self::serialize(&session_ids)?;
        table.insert(user_id, bytes)?;
        Ok(())
    }

    /// Removes a session ID from the user's session index
    fn remove_session_from_index(
        table: &mut redb::Table<u128, Vec<u8>>,
        user_id: u128,
        session_id: &str,
    ) -> Result<(), AuthError> {
        let mut session_ids = Self::get_user_session_ids(table, user_id)?;
        session_ids.retain(|id| id != session_id);
        if session_ids.is_empty() {
            table.remove(user_id)?;
        } else {
            let bytes = Self::serialize(&session_ids)?;
            table.insert(user_id, bytes)?;
        }
        Ok(())
    }

    /// Removes a session from both the sessions table and user sessions index.
    ///
    /// This is the canonical way to clean up a session, ensuring both the session
    /// data and the index entry are removed atomically within a transaction.
    fn remove_session(
        sessions_table: &mut redb::Table<&str, Vec<u8>>,
        user_sessions_table: &mut redb::Table<u128, Vec<u8>>,
        user_id: u128,
        session_id: &str,
    ) -> Result<(), AuthError> {
        sessions_table.remove(session_id)?;
        Self::remove_session_from_index(user_sessions_table, user_id, session_id)?;
        trace!(session_id = %session_id, "Session removed");
        Ok(())
    }
}

impl AuthStore for RedbAuthStore {
    fn max_sessions_per_user(&self) -> usize {
        self.max_sessions_per_user
    }

    async fn create_user(
        &self,
        username: &Username,
        password_hash: PasswordHash,
        role: Role,
    ) -> Result<User, AuthError> {
        let db = self.db.clone();
        let username = username.clone();

        spawn_blocking(move || {
            let write_txn = db.begin_write()?;

            {
                let mut usernames_table = write_txn.open_table(USERNAMES_TABLE)?;
                let mut users_table = write_txn.open_table(USERS_TABLE)?;

                if usernames_table.get(username.as_ref())?.is_some() {
                    debug!("User creation failed: username already exists");
                    return Err(AuthError::UserExists);
                }

                let user = User {
                    id: UserId::new(),
                    role,
                    username: username.clone(),
                    password_hash,
                };

                let user_bytes = Self::serialize(&user)?;

                users_table.insert(user.id.0.as_u128(), user_bytes)?;
                usernames_table.insert(username.as_ref(), user.id.0.as_u128())?;

                trace!(user_id = %user.id.0, "User created successfully");
                Ok(user)
            }
            .and_then(|user| {
                write_txn.commit()?;
                Ok(user)
            })
        })
        .await?
    }

    async fn get_user_by_id(&self, id: &UserId) -> Result<User, AuthError> {
        let db = self.db.clone();
        let id = *id;

        spawn_blocking(move || {
            let read_txn = db.as_ref().begin_read()?;
            let users_table = read_txn.open_table(USERS_TABLE)?;

            if let Some(user_bytes) = users_table.get(id.0.as_u128())? {
                let user = Self::deserialize(&user_bytes.value())?;
                debug!(user_id = %id.0, "User found");
                Ok(user)
            } else {
                debug!(user_id = %id.0, "User not found");
                Err(AuthError::NotFound)
            }
        })
        .await?
    }

    async fn get_user_by_username(&self, username: &Username) -> Result<User, AuthError> {
        let db = self.db.clone();
        let username = username.clone();

        spawn_blocking(move || {
            let read_txn = db.as_ref().begin_read()?;
            let usernames_table = read_txn.open_table(USERNAMES_TABLE)?;
            let users_table = read_txn.open_table(USERS_TABLE)?;

            if let Some(user_id) = usernames_table.get(username.as_ref())? {
                if let Some(user_bytes) = users_table.get(user_id.value())? {
                    let user: User = Self::deserialize(&user_bytes.value())?;
                    debug!(user_id = %user.id.0, "User found");
                    Ok(user)
                } else {
                    // Inconsistency: Username exists but User data missing
                    error!("Inconsistency: Username found but User data missing");
                    Err(AuthError::NotFound)
                }
            } else {
                debug!("User not found");
                Err(AuthError::NotFound)
            }
        })
        .await?
    }

    async fn set_password_hash(
        &self,
        id: &UserId,
        new_hash: PasswordHash,
    ) -> Result<PasswordHash, AuthError> {
        let db = self.db.clone();
        let id = *id;

        spawn_blocking(move || {
            let write_txn = db.begin_write()?;
            {
                let mut users_table = write_txn.open_table(USERS_TABLE)?;

                let user_data = {
                    let user_bytes_opt = users_table.get(id.0.as_u128())?;

                    user_bytes_opt.map(|bytes| bytes.value().to_vec())
                };

                if let Some(user_bytes) = user_data {
                    let mut user: User = Self::deserialize(&user_bytes)?;
                    user.password_hash = new_hash.clone();
                    let new_user_bytes = Self::serialize(&user)?;
                    users_table.insert(id.0.as_u128(), new_user_bytes)?;
                    Ok(new_hash)
                } else {
                    Err(AuthError::NotFound)
                }
            }
            .and_then(|hash| {
                write_txn.commit()?;
                Ok(hash)
            })
        })
        .await?
    }

    async fn delete_user(&self, id: &UserId) -> Result<(), AuthError> {
        let db = self.db.clone();
        let id = *id;

        spawn_blocking(move || {
            let write_txn = db.begin_write()?;
            {
                let mut users_table = write_txn.open_table(USERS_TABLE)?;
                let mut usernames_table = write_txn.open_table(USERNAMES_TABLE)?;
                let mut sessions_table = write_txn.open_table(SESSIONS_TABLE)?;
                let mut user_sessions_table = write_txn.open_table(USER_SESSIONS_INDEX)?;

                if let Some(user_bytes) = users_table.remove(id.0.as_u128())? {
                    let user: User = Self::deserialize(&user_bytes.value())?;
                    usernames_table.remove(user.username.as_ref())?;

                    // Clean up all sessions for this user
                    let session_ids =
                        Self::get_user_session_ids(&user_sessions_table, id.0.as_u128())?;
                    for session_id in session_ids {
                        sessions_table.remove(session_id.as_str())?;
                    }
                    // Remove the entire index entry for this user
                    user_sessions_table.remove(id.0.as_u128())?;

                    Ok(())
                } else {
                    Err(AuthError::NotFound)
                }
            }
            .and_then(|_| {
                write_txn.commit()?;
                Ok(())
            })
        })
        .await?
    }

    async fn issue_session(&self, id: &UserId, ip: SessionIp) -> Result<Session, AuthError> {
        let db = self.db.clone();
        let id = *id;
        let max_sessions = self.max_sessions_per_user;

        spawn_blocking(move || {
            let write_txn = db.begin_write()?;
            let now = OffsetDateTime::now_utc();
            let expires = now + SESSION_DURATION;

            {
                let mut sessions_table = write_txn.open_table(SESSIONS_TABLE)?;
                let users_table = write_txn.open_table(USERS_TABLE)?;
                let mut user_sessions_table = write_txn.open_table(USER_SESSIONS_INDEX)?;

                // Verify user exists
                if users_table.get(id.0.as_u128())?.is_none() {
                    debug!(user_id = %id.0, "User not found during session creation");
                    return Err(AuthError::NotFound);
                }

                // Get session IDs from index and count active (non-expired) sessions
                let session_ids = Self::get_user_session_ids(&user_sessions_table, id.0.as_u128())?;
                let mut active_count = 0;
                let mut expired_session_ids = Vec::new();

                for session_id in &session_ids {
                    if let Some(session_bytes) = sessions_table.get(session_id.as_str())? {
                        let session: Session = Self::deserialize(&session_bytes.value())?;
                        if session.expires_at > now {
                            active_count += 1;
                        } else {
                            expired_session_ids.push(session_id.clone());
                        }
                    } else {
                        // Session in index but not in sessions table - orphaned entry
                        expired_session_ids.push(session_id.clone());
                    }
                }

                // Clean up expired/orphaned sessions
                for session_id in &expired_session_ids {
                    Self::remove_session(
                        &mut sessions_table,
                        &mut user_sessions_table,
                        id.0.as_u128(),
                        session_id,
                    )?;
                }

                if active_count >= max_sessions {
                    debug!(
                        user_id = %id.0,
                        active_count,
                        max_sessions,
                        "Maximum active sessions reached"
                    );
                    return Err(AuthError::SessionLimitReached);
                }

                // Create new session
                let session = Session {
                    id: SessionId::new(),
                    user_id: id,
                    ip,
                    created_at: now,
                    expires_at: expires,
                };

                let session_bytes = Self::serialize(&session)?;
                sessions_table.insert(session.id.as_str(), session_bytes)?;

                // Add to user sessions index
                Self::add_session_to_index(
                    &mut user_sessions_table,
                    id.0.as_u128(),
                    session.id.as_str(),
                )?;

                trace!(
                    user_id = %id.0,
                    session_id = %session.id.0,
                    "Session created successfully"
                );
                Ok(session)
            }
            .and_then(|session| {
                write_txn.commit()?;
                Ok(session)
            })
        })
        .await?
    }

    async fn fetch_session(&self, token: &SessionId) -> Result<Session, AuthError> {
        let db = self.db.clone();
        let token = token.clone();

        spawn_blocking(move || {
            let write_txn = db.begin_write()?;
            let now = OffsetDateTime::now_utc();

            {
                let mut sessions_table = write_txn.open_table(SESSIONS_TABLE)?;
                let mut user_sessions_table = write_txn.open_table(USER_SESSIONS_INDEX)?;

                let session_data = sessions_table
                    .get(token.as_str())?
                    .map(|bytes| bytes.value().to_vec());

                if let Some(session_bytes) = session_data {
                    let session: Session = Self::deserialize(&session_bytes)?;

                    if session.expires_at <= now {
                        debug!(
                            session_id = %token.0,
                            expired_at = %session.expires_at,
                            "Session expired, removing"
                        );
                        Self::remove_session(
                            &mut sessions_table,
                            &mut user_sessions_table,
                            session.user_id.0.as_u128(),
                            token.as_str(),
                        )?;
                        Err(AuthError::InvalidSession)
                    } else {
                        debug!(session_id = %token.0, "Valid session found");
                        Ok(session)
                    }
                } else {
                    debug!(session_id = %token.0, "Session not found");
                    Err(AuthError::InvalidSession)
                }
            }
            .and_then(|session| {
                write_txn.commit()?;
                Ok(session)
            })
        })
        .await?
    }

    async fn extend_session(&self, token: &SessionId) -> Result<Session, AuthError> {
        let db = self.db.clone();
        let token = token.clone();

        spawn_blocking(move || {
            let write_txn = db.begin_write()?;
            let now = OffsetDateTime::now_utc();
            let new_expires = now + SESSION_DURATION;

            {
                let mut sessions_table = write_txn.open_table(SESSIONS_TABLE)?;
                let mut user_sessions_table = write_txn.open_table(USER_SESSIONS_INDEX)?;

                let session_data = sessions_table
                    .get(token.as_str())?
                    .map(|bytes| bytes.value().to_vec());

                if let Some(session_bytes) = session_data {
                    let mut session: Session = Self::deserialize(&session_bytes)?;

                    if session.expires_at <= now {
                        debug!(
                            session_id = %token.0,
                            expired_at = %session.expires_at,
                            "Session expired, cannot extend"
                        );
                        Self::remove_session(
                            &mut sessions_table,
                            &mut user_sessions_table,
                            session.user_id.0.as_u128(),
                            token.as_str(),
                        )?;
                        Err(AuthError::InvalidSession)
                    } else {
                        session.expires_at = new_expires;
                        let new_session_bytes = Self::serialize(&session)?;
                        sessions_table.insert(token.as_str(), new_session_bytes)?;

                        trace!(
                            session_id = %token.0,
                            new_expires = %new_expires,
                            "Session extended successfully"
                        );
                        Ok(session)
                    }
                } else {
                    debug!(session_id = %token.0, "Session not found");
                    Err(AuthError::InvalidSession)
                }
            }
            .and_then(|session| {
                write_txn.commit()?;
                Ok(session)
            })
        })
        .await?
    }

    async fn revoke_session(&self, token: &SessionId) -> Result<(), AuthError> {
        let db = self.db.clone();
        let token = token.clone();

        spawn_blocking(move || {
            let write_txn = db.begin_write()?;

            {
                let mut sessions_table = write_txn.open_table(SESSIONS_TABLE)?;
                let mut user_sessions_table = write_txn.open_table(USER_SESSIONS_INDEX)?;

                // First get the session to find the user_id
                let session_data = sessions_table
                    .get(token.as_str())?
                    .map(|bytes| bytes.value().to_vec());

                if let Some(session_bytes) = session_data {
                    let session: Session = Self::deserialize(&session_bytes)?;
                    Self::remove_session(
                        &mut sessions_table,
                        &mut user_sessions_table,
                        session.user_id.0.as_u128(),
                        token.as_str(),
                    )?;
                    debug!(session_id = %token.0, "Session revoked successfully");
                    Ok(())
                } else {
                    debug!(session_id = %token.0, "Session not found for revocation");
                    Err(AuthError::InvalidSession)
                }
            }
            .and_then(|_| {
                write_txn.commit()?;
                Ok(())
            })
        })
        .await?
    }
}
