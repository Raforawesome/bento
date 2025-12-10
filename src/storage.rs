//! Storage traits and error types for the Bento application.
//!
//! This module defines the `AuthStore` and `ProjectStore` traits that abstract
//! over different storage backends (memory, redb, etc.).

pub mod error;
pub mod mem_authstore;
pub mod redb_authstore;
pub mod redb_projectstore;

pub use error::{AuthError, ProjectError};

use crate::types::{
    PasswordHash, Project, ProjectId, ProjectSummary, Role, Session, SessionId, SessionIp, User,
    UserId, Username,
};

/// Trait for authentication and user session storage.
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

/// Trait for project storage operations.
pub trait ProjectStore: Send + Sync {
    /// Create a new project for a user
    fn create_project(
        &self,
        owner_id: &UserId,
        name: String,
        description: Option<String>,
    ) -> impl Future<Output = Result<Project, ProjectError>> + Send;

    /// Get a project by ID
    fn get_project(
        &self,
        project_id: &ProjectId,
    ) -> impl Future<Output = Result<Project, ProjectError>> + Send;

    /// Get all projects owned by a user
    fn get_user_projects(
        &self,
        owner_id: &UserId,
    ) -> impl Future<Output = Result<Vec<ProjectSummary>, ProjectError>> + Send;

    /// Update a project's name and/or description
    fn update_project(
        &self,
        project_id: &ProjectId,
        name: Option<String>,
        description: Option<Option<String>>,
    ) -> impl Future<Output = Result<Project, ProjectError>> + Send;

    /// Delete a project
    fn delete_project(
        &self,
        project_id: &ProjectId,
    ) -> impl Future<Output = Result<(), ProjectError>> + Send;
}
