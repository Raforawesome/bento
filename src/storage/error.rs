//! Storage error types for auth and project stores.
//!
//! This module consolidates error handling for storage backends,
//! using a macro to reduce boilerplate for common error conversions.

use thiserror::Error;

/// Macro to implement From traits for common storage backend errors.
///
/// This reduces duplication when multiple error types need the same
/// conversions from redb, bincode, and tokio errors.
macro_rules! impl_storage_error_conversions {
    ($error_type:ty) => {
        #[cfg(feature = "ssr")]
        impl From<redb::TransactionError> for $error_type {
            fn from(err: redb::TransactionError) -> Self {
                Self::Internal(err.to_string())
            }
        }

        #[cfg(feature = "ssr")]
        impl From<redb::TableError> for $error_type {
            fn from(err: redb::TableError) -> Self {
                Self::Internal(err.to_string())
            }
        }

        #[cfg(feature = "ssr")]
        impl From<redb::CommitError> for $error_type {
            fn from(err: redb::CommitError) -> Self {
                Self::Internal(err.to_string())
            }
        }

        #[cfg(feature = "ssr")]
        impl From<redb::StorageError> for $error_type {
            fn from(err: redb::StorageError) -> Self {
                Self::Internal(err.to_string())
            }
        }

        #[cfg(feature = "ssr")]
        impl From<redb::DatabaseError> for $error_type {
            fn from(err: redb::DatabaseError) -> Self {
                Self::Internal(err.to_string())
            }
        }

        #[cfg(feature = "ssr")]
        impl From<bincode::error::EncodeError> for $error_type {
            fn from(err: bincode::error::EncodeError) -> Self {
                Self::Internal(format!("Serialization error: {}", err))
            }
        }

        #[cfg(feature = "ssr")]
        impl From<bincode::error::DecodeError> for $error_type {
            fn from(err: bincode::error::DecodeError) -> Self {
                Self::Internal(format!("Deserialization error: {}", err))
            }
        }

        #[cfg(feature = "ssr")]
        impl From<tokio::task::JoinError> for $error_type {
            fn from(err: tokio::task::JoinError) -> Self {
                Self::Internal(format!("Task join error: {}", err))
            }
        }
    };
}

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

impl_storage_error_conversions!(AuthError);

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("Project not found")]
    NotFound,
    #[error("Project with this name already exists")]
    AlreadyExists,
    #[error("Unauthorized access to project")]
    Unauthorized,
    #[error("Internal error: {0}")]
    Internal(String),
}

impl_storage_error_conversions!(ProjectError);
