use redb::{
    Database, MultimapTableDefinition, ReadTransaction, ReadableDatabase, ReadableTable,
    TableDefinition, WriteTransaction,
};
use serde::{Serialize, de::DeserializeOwned};
use std::path::Path;
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::task::spawn_blocking;
use tracing::{debug, trace};

use super::{ProjectError, ProjectStore};
use crate::types::{Project, ProjectId, ProjectSummary, UserId};

// Table definitions
/// Primary table: project_id (u128) -> Project (serialized)
const PROJECTS_TABLE: TableDefinition<u128, Vec<u8>> = TableDefinition::new("projects");

/// Index: owner_id (u128) -> project_id (u128) for efficient user project lookups
const USER_PROJECTS_INDEX: MultimapTableDefinition<u128, u128> =
    MultimapTableDefinition::new("user_projects");

#[derive(Clone)]
pub struct RedbProjectStore {
    db: Arc<Database>,
}

impl RedbProjectStore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, ProjectError> {
        let db = Database::create(path)?;

        // Initialize tables
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(PROJECTS_TABLE)?;
            let _ = write_txn.open_multimap_table(USER_PROJECTS_INDEX)?;
        }
        write_txn.commit()?;

        Ok(Self { db: Arc::new(db) })
    }

    // ==================== Serialization Helpers ====================

    fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, ProjectError> {
        Ok(bincode::serde::encode_to_vec(
            value,
            bincode::config::standard(),
        )?)
    }

    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, ProjectError> {
        let (result, _) = bincode::serde::decode_from_slice(bytes, bincode::config::standard())?;
        Ok(result)
    }

    // ==================== Transaction Helpers ====================

    /// Execute a read-only operation within a transaction
    async fn with_read_txn<T, F>(&self, f: F) -> Result<T, ProjectError>
    where
        T: Send + 'static,
        F: FnOnce(&ReadTransaction) -> Result<T, ProjectError> + Send + 'static,
    {
        let db = self.db.clone();
        spawn_blocking(move || {
            let txn = db.begin_read()?;
            f(&txn)
        })
        .await?
    }

    /// Execute a write operation within a transaction
    async fn with_write_txn<T, F>(&self, f: F) -> Result<T, ProjectError>
    where
        T: Send + 'static,
        F: FnOnce(&WriteTransaction) -> Result<T, ProjectError> + Send + 'static,
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
}

impl ProjectStore for RedbProjectStore {
    async fn create_project(
        &self,
        owner_id: &UserId,
        name: String,
        description: Option<String>,
    ) -> Result<Project, ProjectError> {
        let owner_id = *owner_id;
        let now = OffsetDateTime::now_utc();

        self.with_write_txn(move |txn| {
            let mut projects_table = txn.open_table(PROJECTS_TABLE)?;
            let mut user_projects_table = txn.open_multimap_table(USER_PROJECTS_INDEX)?;

            let project = Project {
                id: ProjectId::new(),
                owner_id,
                name,
                description,
                created_at: now,
                updated_at: now,
            };

            let project_bytes = Self::serialize(&project)?;
            let project_id_u128 = project.id.0.as_u128();
            let owner_id_u128 = owner_id.0.as_u128();

            projects_table.insert(project_id_u128, project_bytes)?;
            user_projects_table.insert(owner_id_u128, project_id_u128)?;

            trace!(project_id = %project.id.0, owner_id = %owner_id.0, "Project created successfully");
            Ok(project)
        })
        .await
    }

    async fn get_project(&self, project_id: &ProjectId) -> Result<Project, ProjectError> {
        let project_id = *project_id;

        self.with_read_txn(move |txn| {
            let projects_table = txn.open_table(PROJECTS_TABLE)?;

            match projects_table.get(project_id.0.as_u128())? {
                Some(project_bytes) => {
                    let project: Project = Self::deserialize(&project_bytes.value())?;
                    debug!(project_id = %project_id.0, "Project found");
                    Ok(project)
                }
                None => {
                    debug!(project_id = %project_id.0, "Project not found");
                    Err(ProjectError::NotFound)
                }
            }
        })
        .await
    }

    async fn get_user_projects(
        &self,
        owner_id: &UserId,
    ) -> Result<Vec<ProjectSummary>, ProjectError> {
        let owner_id = *owner_id;

        self.with_read_txn(move |txn| {
            let projects_table = txn.open_table(PROJECTS_TABLE)?;
            let user_projects_table = txn.open_multimap_table(USER_PROJECTS_INDEX)?;

            let mut summaries = Vec::new();

            // Get all project IDs for this user from the index
            let project_ids = user_projects_table.get(owner_id.0.as_u128())?;

            for project_id_result in project_ids {
                let project_id = project_id_result?.value();

                if let Some(project_bytes) = projects_table.get(project_id)? {
                    let project: Project = Self::deserialize(&project_bytes.value())?;
                    summaries.push(ProjectSummary::from(&project));
                }
            }

            // Sort by created_at descending (newest first)
            summaries.sort_by(|a, b| b.created_at.cmp(&a.created_at));

            debug!(owner_id = %owner_id.0, count = summaries.len(), "Retrieved user projects");
            Ok(summaries)
        })
        .await
    }

    async fn update_project(
        &self,
        project_id: &ProjectId,
        name: Option<String>,
        description: Option<Option<String>>,
    ) -> Result<Project, ProjectError> {
        let project_id = *project_id;

        self.with_write_txn(move |txn| {
            let mut projects_table = txn.open_table(PROJECTS_TABLE)?;

            let project_bytes = projects_table
                .get(project_id.0.as_u128())?
                .map(|bytes| bytes.value().to_vec())
                .ok_or(ProjectError::NotFound)?;

            let mut project: Project = Self::deserialize(&project_bytes)?;

            // Update fields if provided
            if let Some(new_name) = name {
                project.name = new_name;
            }
            if let Some(new_description) = description {
                project.description = new_description;
            }
            project.updated_at = OffsetDateTime::now_utc();

            let new_project_bytes = Self::serialize(&project)?;
            projects_table.insert(project_id.0.as_u128(), new_project_bytes)?;

            trace!(project_id = %project_id.0, "Project updated successfully");
            Ok(project)
        })
        .await
    }

    async fn delete_project(&self, project_id: &ProjectId) -> Result<(), ProjectError> {
        let project_id = *project_id;

        self.with_write_txn(move |txn| {
            let mut projects_table = txn.open_table(PROJECTS_TABLE)?;
            let mut user_projects_table = txn.open_multimap_table(USER_PROJECTS_INDEX)?;

            // First get the project to find the owner_id for index cleanup
            let project_bytes = projects_table
                .remove(project_id.0.as_u128())?
                .ok_or(ProjectError::NotFound)?;

            let project: Project = Self::deserialize(&project_bytes.value())?;

            // Remove from the user_projects index
            user_projects_table.remove(project.owner_id.0.as_u128(), project_id.0.as_u128())?;

            trace!(project_id = %project_id.0, owner_id = %project.owner_id.0, "Project deleted successfully");
            Ok(())
        })
        .await
    }
}
