use leptos::prelude::FromServerFnError;
use leptos::server_fn::codec::JsonEncoding;
use leptos::server_fn::error::ServerFnErrorErr;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use time::OffsetDateTime;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use argon2::{
    Argon2,
    password_hash::{
        PasswordHashString, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::OsRng as ArgonRng,
    },
};
#[cfg(feature = "ssr")]
use base64::{Engine as _, engine::general_purpose::URL_SAFE as Base64Url};
#[cfg(feature = "ssr")]
use rand::rngs::OsRng;

/*
 * Newtype wrappers for strong typing
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Username(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordHash(
    #[cfg(feature = "ssr")] PasswordHashString,
    #[cfg(not(feature = "ssr"))] String,
);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionIp(pub IpAddr);

/// An enum to represent a user's permission level;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub user_id: UserId,
    pub ip: SessionIp,
    pub created_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
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

#[cfg(feature = "ssr")]
impl SessionId {
    pub fn new() -> Self {
        use rand::TryRngCore as _;

        let mut buf = [0_u8; 32];
        if OsRng.try_fill_bytes(&mut buf).is_ok() {
            SessionId(Base64Url.encode(buf))
        } else {
            panic!("Failed to generate secure numbers from the operating system.");
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "ssr")]
impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "ssr")]
impl PasswordHash {
    pub fn verify<B: AsRef<[u8]>>(&self, password: B) -> bool {
        let pass_bytes: &[u8] = password.as_ref();

        Argon2::default()
            .verify_password(pass_bytes, &self.0.password_hash())
            .is_ok()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(feature = "ssr")]
impl TryFrom<&[u8]> for PasswordHash {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let pass_bytes: &[u8] = value;
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(pass_bytes, &salt)?.to_string();
        Ok(Self(PasswordHashString::new(&password_hash)?))
    }
}

#[cfg(feature = "ssr")]
impl TryFrom<&str> for PasswordHash {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let pass_bytes: &[u8] = value.as_bytes();
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(pass_bytes, &salt)?.to_string();
        Ok(Self(PasswordHashString::new(&password_hash)?))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Invalid credentials provided")]
    InvalidCreds,
    #[error("Client request error")]
    RequestError,
    #[error("An unknown error occurred")]
    Unknown,
}

/// Universal error type that automatically converts from any error.
///
/// This type implements `FromServerFnError` and uses downcasting
/// to provide user-friendly error messages for known error types, while
/// gracefully handling unknown errors.
///
/// No manual `From` implementations are needed - any `std::error::Error` can be
/// automatically converted using the `?` operator.
///
/// ## Example
/// ```rust
/// #[server]
/// pub async fn my_function() -> Result<Data, AppError> {
///     let user = auth_store.get_user(&id).await?;  // AuthError → AppError
///     let data = fetch_data().await?;              // Any error → AppError
///     Ok(data)
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppError(String);

impl AppError {
    /// Create a new AppError with a custom message
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        &self.0
    }
}

impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        Self::new(format!("Server function error: {:?}", value))
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Universal error conversion using downcasting for user-friendly messages
#[cfg(feature = "ssr")]
impl<E> From<E> for AppError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        use crate::storage::{AuthError, ProjectError};
        use std::any::Any;

        // Try to downcast to known error types for better messages
        let err_any: &dyn Any = &err;

        // Check for AuthError
        if let Some(auth_err) = err_any.downcast_ref::<AuthError>() {
            return Self::new(match auth_err {
                AuthError::NotFound => "User not found",
                AuthError::InvalidSession => "Your session has expired. Please log in again.",
                AuthError::UserExists => "A user with this username already exists",
                AuthError::SessionLimitReached => {
                    "Maximum number of active sessions reached. Please log out of another device."
                }
                AuthError::Internal(_) => "An internal error occurred. Please try again later.",
            });
        }

        // Check for ProjectError
        if let Some(project_err) = err_any.downcast_ref::<ProjectError>() {
            return Self::new(match project_err {
                ProjectError::NotFound => "Project not found",
                ProjectError::AlreadyExists => "A project with this name already exists",
                ProjectError::Unauthorized => "You don't have permission to access this project",
                ProjectError::Internal(_) => "An internal error occurred. Please try again later.",
            });
        }

        // Check for ServerError
        if let Some(server_err) = err_any.downcast_ref::<ServerError>() {
            return Self::new(match server_err {
                ServerError::InvalidCreds => "Invalid username or password",
                ServerError::RequestError => "Request error occurred",
                ServerError::Unknown => "An unknown error occurred",
            });
        }

        // Default: convert to string
        Self::new(err.to_string())
    }
}

// Client-side: can't use the generic From impl, so handle ServerFnErrorErr specifically
#[cfg(not(feature = "ssr"))]
impl From<ServerFnErrorErr> for AppError {
    fn from(err: ServerFnErrorErr) -> Self {
        Self::new(format!("Server function error: {:?}", err))
    }
}

impl serde::Serialize for PasswordHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[cfg(feature = "ssr")]
        {
            serializer.serialize_str(self.0.as_str())
        }
        #[cfg(not(feature = "ssr"))]
        {
            serializer.serialize_str(&self.0)
        }
    }
}

impl<'de> serde::Deserialize<'de> for PasswordHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        #[cfg(feature = "ssr")]
        {
            use argon2::password_hash::PasswordHashString;
            PasswordHashString::new(&s)
                .map(PasswordHash)
                .map_err(serde::de::Error::custom)
        }
        #[cfg(not(feature = "ssr"))]
        {
            Ok(PasswordHash(s))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub Uuid);

impl ProjectId {
    pub fn new() -> Self {
        ProjectId(Uuid::now_v7())
    }
}

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a user's project stored in the database
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub owner_id: UserId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// Lightweight project summary for listing/display purposes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
}

impl From<Project> for ProjectSummary {
    fn from(project: Project) -> Self {
        Self {
            id: project.id,
            name: project.name,
            description: project.description,
            created_at: project.created_at,
        }
    }
}

impl From<&Project> for ProjectSummary {
    fn from(project: &Project) -> Self {
        Self {
            id: project.id,
            name: project.name.clone(),
            description: project.description.clone(),
            created_at: project.created_at,
        }
    }
}

/// Legacy struct for UI display with computed metrics
/// TODO: Remove once UI is updated to use ProjectSummary
#[derive(Clone, PartialEq)]
pub struct ProjectData {
    pub name: String,
    pub project_id: ProjectId,
    pub db_used: String,
    pub users_count: String,
    pub active_connections: String,
}
