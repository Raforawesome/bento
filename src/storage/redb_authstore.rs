use redb::{
    Database, MultimapTableDefinition, ReadTransaction, ReadableDatabase, ReadableMultimapTable,
    ReadableTable, TableDefinition, WriteTransaction,
};
use serde::{Serialize, de::DeserializeOwned};
use std::path::Path;
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::task::spawn_blocking;
use tracing::{debug, error, trace};

use super::{AuthError, AuthStore};
use crate::config::SESSION_DURATION;
use crate::types::{PasswordHash, Role, Session, SessionId, SessionIp, User, UserId, Username};

// Table definitions
const USERS_TABLE: TableDefinition<u128, Vec<u8>> = TableDefinition::new("users");
const USERNAMES_TABLE: TableDefinition<&str, u128> = TableDefinition::new("usernames");
const SESSIONS_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("sessions");

/// Multimap index: user_id -> session_id for O(1) add/remove operations
const USER_SESSIONS_INDEX: MultimapTableDefinition<u128, &str> =
    MultimapTableDefinition::new("user_sessions_v2");

/// Reverse index: session_id -> user_id for O(1) lookup without deserializing session
const SESSION_USER_INDEX: TableDefinition<&str, u128> = TableDefinition::new("session_user");

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
            let _ = write_txn.open_multimap_table(USER_SESSIONS_INDEX)?;
            let _ = write_txn.open_table(SESSION_USER_INDEX)?;
        }
        write_txn.commit()?;

        Ok(Self {
            db: Arc::new(db),
            max_sessions_per_user,
        })
    }

    // ==================== Serialization Helpers ====================

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

    // ==================== Transaction Helpers ====================

    /// Execute a read-only operation within a transaction
    async fn with_read_txn<T, F>(&self, f: F) -> Result<T, AuthError>
    where
        T: Send + 'static,
        F: FnOnce(&ReadTransaction) -> Result<T, AuthError> + Send + 'static,
    {
        let db = self.db.clone();
        spawn_blocking(move || {
            let txn = db.begin_read()?;
            f(&txn)
        })
        .await?
    }

    /// Execute a write operation within a transaction
    async fn with_write_txn<T, F>(&self, f: F) -> Result<T, AuthError>
    where
        T: Send + 'static,
        F: FnOnce(&WriteTransaction) -> Result<T, AuthError> + Send + 'static,
    {
        let db = self.db.clone();
        spawn_blocking(move || {
            let txn = db.begin_write()?;
            let result = f(&txn)?;
            txn.commit()?;
            Ok(result)
        })
        .await?
    }

    // ==================== Multimap Index Operations ====================

    /// Gets all session IDs for a user using multimap table - O(n) where n = user's session count
    fn get_user_session_ids(
        table: &redb::MultimapTable<u128, &'static str>,
        user_id: u128,
    ) -> Result<Vec<String>, AuthError> {
        let mut session_ids = Vec::new();
        let values = table.get(user_id)?;
        for value_result in values {
            let value = value_result?;
            session_ids.push(value.value().to_string());
        }
        Ok(session_ids)
    }

    /// Removes a session from all relevant tables and indexes - O(log N)
    fn remove_session(
        sessions_table: &mut redb::Table<&str, Vec<u8>>,
        user_sessions_table: &mut redb::MultimapTable<u128, &'static str>,
        session_user_table: &mut redb::Table<&str, u128>,
        user_id: u128,
        session_id: &str,
    ) -> Result<(), AuthError> {
        sessions_table.remove(session_id)?;
        user_sessions_table.remove(user_id, session_id)?;
        session_user_table.remove(session_id)?;
        trace!(session_id = %session_id, "Session removed from all tables");
        Ok(())
    }

    /// Batch removes multiple sessions - O(k log N) where k = number of sessions
    fn remove_sessions_batch(
        sessions_table: &mut redb::Table<&str, Vec<u8>>,
        user_sessions_table: &mut redb::MultimapTable<u128, &'static str>,
        session_user_table: &mut redb::Table<&str, u128>,
        user_id: u128,
        session_ids: &[String],
    ) -> Result<(), AuthError> {
        for session_id in session_ids {
            sessions_table.remove(session_id.as_str())?;
            user_sessions_table.remove(user_id, session_id.as_str())?;
            session_user_table.remove(session_id.as_str())?;
        }
        if !session_ids.is_empty() {
            trace!(count = session_ids.len(), "Batch removed expired sessions");
        }
        Ok(())
    }

    /// Removes all sessions for a user - used during user deletion
    fn remove_all_user_sessions(
        sessions_table: &mut redb::Table<&str, Vec<u8>>,
        user_sessions_table: &mut redb::MultimapTable<u128, &'static str>,
        session_user_table: &mut redb::Table<&str, u128>,
        user_id: u128,
    ) -> Result<(), AuthError> {
        let session_ids = Self::get_user_session_ids(user_sessions_table, user_id)?;

        for session_id in &session_ids {
            sessions_table.remove(session_id.as_str())?;
            session_user_table.remove(session_id.as_str())?;
        }
        // Remove all entries for this user from the multimap
        user_sessions_table.remove_all(user_id)?;

        trace!(user_id = %user_id, count = session_ids.len(), "Removed all user sessions");
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
        let username = username.clone();

        self.with_write_txn(move |txn| {
            let mut usernames_table = txn.open_table(USERNAMES_TABLE)?;
            let mut users_table = txn.open_table(USERS_TABLE)?;

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
        })
        .await
    }

    async fn get_user_by_id(&self, id: &UserId) -> Result<User, AuthError> {
        let id = *id;

        self.with_read_txn(move |txn| {
            let users_table = txn.open_table(USERS_TABLE)?;

            match users_table.get(id.0.as_u128())? {
                Some(user_bytes) => {
                    let user = Self::deserialize(&user_bytes.value())?;
                    debug!(user_id = %id.0, "User found");
                    Ok(user)
                }
                None => {
                    debug!(user_id = %id.0, "User not found");
                    Err(AuthError::NotFound)
                }
            }
        })
        .await
    }

    async fn get_user_by_username(&self, username: &Username) -> Result<User, AuthError> {
        let username = username.clone();

        self.with_read_txn(move |txn| {
            let usernames_table = txn.open_table(USERNAMES_TABLE)?;
            let users_table = txn.open_table(USERS_TABLE)?;

            let user_id = match usernames_table.get(username.as_ref())? {
                Some(id) => id.value(),
                None => {
                    debug!("User not found");
                    return Err(AuthError::NotFound);
                }
            };

            match users_table.get(user_id)? {
                Some(user_bytes) => {
                    let user: User = Self::deserialize(&user_bytes.value())?;
                    debug!(user_id = %user.id.0, "User found");
                    Ok(user)
                }
                None => {
                    error!("Inconsistency: Username found but User data missing");
                    Err(AuthError::NotFound)
                }
            }
        })
        .await
    }

    async fn set_password_hash(
        &self,
        id: &UserId,
        new_hash: PasswordHash,
    ) -> Result<PasswordHash, AuthError> {
        let id = *id;

        self.with_write_txn(move |txn| {
            let mut users_table = txn.open_table(USERS_TABLE)?;

            let user_bytes = users_table
                .get(id.0.as_u128())?
                .map(|bytes| bytes.value().to_vec())
                .ok_or(AuthError::NotFound)?;

            let mut user: User = Self::deserialize(&user_bytes)?;
            user.password_hash = new_hash.clone();

            let new_user_bytes = Self::serialize(&user)?;
            users_table.insert(id.0.as_u128(), new_user_bytes)?;

            Ok(new_hash)
        })
        .await
    }

    async fn delete_user(&self, id: &UserId) -> Result<(), AuthError> {
        let id = *id;

        self.with_write_txn(move |txn| {
            let mut users_table = txn.open_table(USERS_TABLE)?;
            let mut usernames_table = txn.open_table(USERNAMES_TABLE)?;
            let mut sessions_table = txn.open_table(SESSIONS_TABLE)?;
            let mut user_sessions_table = txn.open_multimap_table(USER_SESSIONS_INDEX)?;
            let mut session_user_table = txn.open_table(SESSION_USER_INDEX)?;

            let user_bytes = users_table
                .remove(id.0.as_u128())?
                .ok_or(AuthError::NotFound)?;

            let user: User = Self::deserialize(&user_bytes.value())?;
            usernames_table.remove(user.username.as_ref())?;

            // Clean up all sessions for this user
            Self::remove_all_user_sessions(
                &mut sessions_table,
                &mut user_sessions_table,
                &mut session_user_table,
                id.0.as_u128(),
            )?;

            trace!(user_id = %id.0, "User deleted successfully");
            Ok(())
        })
        .await
    }

    async fn issue_session(&self, id: &UserId, ip: SessionIp) -> Result<Session, AuthError> {
        let id = *id;
        let max_sessions = self.max_sessions_per_user;

        self.with_write_txn(move |txn| {
            let now = OffsetDateTime::now_utc();
            let expires = now + SESSION_DURATION;

            let mut sessions_table = txn.open_table(SESSIONS_TABLE)?;
            let users_table = txn.open_table(USERS_TABLE)?;
            let mut user_sessions_table = txn.open_multimap_table(USER_SESSIONS_INDEX)?;
            let mut session_user_table = txn.open_table(SESSION_USER_INDEX)?;

            // Verify user exists
            if users_table.get(id.0.as_u128())?.is_none() {
                debug!(user_id = %id.0, "User not found during session creation");
                return Err(AuthError::NotFound);
            }

            // Get session IDs and partition into active/expired
            let session_ids = Self::get_user_session_ids(&user_sessions_table, id.0.as_u128())?;
            let mut active_count = 0;
            let mut expired_session_ids = Vec::new();

            for session_id in &session_ids {
                match sessions_table.get(session_id.as_str())? {
                    Some(session_bytes) => {
                        let session: Session = Self::deserialize(&session_bytes.value())?;
                        if session.expires_at > now {
                            active_count += 1;
                        } else {
                            expired_session_ids.push(session_id.clone());
                        }
                    }
                    None => {
                        // Session in index but not in sessions table - orphaned entry
                        expired_session_ids.push(session_id.clone());
                    }
                }
            }

            // Batch clean up expired/orphaned sessions
            Self::remove_sessions_batch(
                &mut sessions_table,
                &mut user_sessions_table,
                &mut session_user_table,
                id.0.as_u128(),
                &expired_session_ids,
            )?;

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

            // Add to indexes
            user_sessions_table.insert(id.0.as_u128(), session.id.as_str())?;
            session_user_table.insert(session.id.as_str(), id.0.as_u128())?;

            trace!(
                user_id = %id.0,
                session_id = %session.id.0,
                "Session created successfully"
            );
            Ok(session)
        })
        .await
    }

    async fn fetch_session(&self, token: &SessionId) -> Result<Session, AuthError> {
        let db = self.db.clone();
        let token = token.clone();

        // Use read-first approach: only acquire write lock if cleanup is needed
        spawn_blocking(move || {
            let now = OffsetDateTime::now_utc();

            // First, try with a read transaction (common path)
            {
                let read_txn = db.begin_read()?;
                let sessions_table = read_txn.open_table(SESSIONS_TABLE)?;

                match sessions_table.get(token.as_str())? {
                    Some(session_bytes) => {
                        let session: Session = Self::deserialize(&session_bytes.value())?;
                        if session.expires_at > now {
                            debug!(session_id = %token.0, "Valid session found");
                            return Ok(session);
                        }
                        // Session expired - fall through to cleanup with write transaction
                        debug!(
                            session_id = %token.0,
                            expired_at = %session.expires_at,
                            "Session expired, will clean up"
                        );
                    }
                    None => {
                        debug!(session_id = %token.0, "Session not found");
                        return Err(AuthError::InvalidSession);
                    }
                }
            }

            // Session was expired - acquire write transaction to clean up
            let write_txn = db.begin_write()?;
            {
                let mut sessions_table = write_txn.open_table(SESSIONS_TABLE)?;
                let mut user_sessions_table = write_txn.open_multimap_table(USER_SESSIONS_INDEX)?;
                let mut session_user_table = write_txn.open_table(SESSION_USER_INDEX)?;

                // Get user_id from reverse index (no deserialization needed)
                let user_id = session_user_table.get(token.as_str())?.map(|v| v.value());

                if let Some(user_id) = user_id {
                    Self::remove_session(
                        &mut sessions_table,
                        &mut user_sessions_table,
                        &mut session_user_table,
                        user_id,
                        token.as_str(),
                    )?;
                }
            }
            write_txn.commit()?;

            Err(AuthError::InvalidSession)
        })
        .await?
    }

    async fn extend_session(&self, token: &SessionId) -> Result<Session, AuthError> {
        let db = self.db.clone();
        let token = token.clone();

        // Read-first: check if session is valid before acquiring write lock
        spawn_blocking(move || {
            let now = OffsetDateTime::now_utc();
            let new_expires = now + SESSION_DURATION;

            // First, verify session exists and is not expired with read transaction
            let session_valid = {
                let read_txn = db.begin_read()?;
                let sessions_table = read_txn.open_table(SESSIONS_TABLE)?;

                match sessions_table.get(token.as_str())? {
                    Some(session_bytes) => {
                        let session: Session = Self::deserialize(&session_bytes.value())?;
                        if session.expires_at <= now {
                            debug!(
                                session_id = %token.0,
                                expired_at = %session.expires_at,
                                "Session expired, cannot extend"
                            );
                            false
                        } else {
                            true
                        }
                    }
                    None => {
                        debug!(session_id = %token.0, "Session not found");
                        return Err(AuthError::InvalidSession);
                    }
                }
            };

            if !session_valid {
                // Clean up expired session
                let write_txn = db.begin_write()?;
                {
                    let mut sessions_table = write_txn.open_table(SESSIONS_TABLE)?;
                    let mut user_sessions_table =
                        write_txn.open_multimap_table(USER_SESSIONS_INDEX)?;
                    let mut session_user_table = write_txn.open_table(SESSION_USER_INDEX)?;

                    let user_id = session_user_table.get(token.as_str())?.map(|v| v.value());

                    if let Some(user_id) = user_id {
                        Self::remove_session(
                            &mut sessions_table,
                            &mut user_sessions_table,
                            &mut session_user_table,
                            user_id,
                            token.as_str(),
                        )?;
                    }
                }
                write_txn.commit()?;
                return Err(AuthError::InvalidSession);
            }

            // Session is valid - acquire write transaction to extend
            let write_txn = db.begin_write()?;
            let result = {
                let mut sessions_table = write_txn.open_table(SESSIONS_TABLE)?;

                // Re-fetch and update (session might have changed between transactions)
                let session_data = sessions_table
                    .get(token.as_str())?
                    .map(|b| b.value().to_vec());

                match session_data {
                    Some(session_bytes) => {
                        let mut session: Session = Self::deserialize(&session_bytes)?;

                        // Re-check expiry (could have expired between read and write)
                        if session.expires_at <= now {
                            return Err(AuthError::InvalidSession);
                        }

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
                    None => Err(AuthError::InvalidSession),
                }
            };

            if result.is_ok() {
                write_txn.commit()?;
            }
            result
        })
        .await?
    }

    async fn revoke_session(&self, token: &SessionId) -> Result<(), AuthError> {
        let token = token.clone();

        self.with_write_txn(move |txn| {
            let mut sessions_table = txn.open_table(SESSIONS_TABLE)?;
            let mut user_sessions_table = txn.open_multimap_table(USER_SESSIONS_INDEX)?;
            let mut session_user_table = txn.open_table(SESSION_USER_INDEX)?;

            // Use reverse index to get user_id directly - O(log N), no deserialization
            let user_id = session_user_table
                .get(token.as_str())?
                .map(|v| v.value())
                .ok_or_else(|| {
                    debug!(session_id = %token.0, "Session not found for revocation");
                    AuthError::InvalidSession
                })?;

            Self::remove_session(
                &mut sessions_table,
                &mut user_sessions_table,
                &mut session_user_table,
                user_id,
                token.as_str(),
            )?;

            debug!(session_id = %token.0, "Session revoked successfully");
            Ok(())
        })
        .await
    }
}
