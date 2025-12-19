//! SQLite storage for task lifecycle
//!
//! Modern async storage with:
//! - No unsafe code
//! - Prepared statements (SQL injection safe)
//! - Connection pooling
//! - Atomic operations

use super::{Checkpoint, TaskFilter, TaskId, TaskLifecycle, TowerId, UserId};
use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use tracing::{debug, info};

/// SQLite-backed task storage
pub struct TaskStorage {
    pool: SqlitePool,
}

impl TaskStorage {
    /// Create a new task storage
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool =
            SqlitePool::connect(database_url).await.context("Failed to connect to database")?;

        info!("Connected to task storage database");

        let storage = Self {
            pool,
        };
        storage.migrate().await?;

        Ok(storage)
    }

    /// Run migrations
    async fn migrate(&self) -> Result<()> {
        debug!("Running database migrations");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                owner TEXT NOT NULL,
                status TEXT NOT NULL,
                progress REAL NOT NULL DEFAULT 0.0,
                created_at INTEGER NOT NULL,
                started_at INTEGER,
                completed_at INTEGER,
                current_tower TEXT,
                pausable INTEGER NOT NULL DEFAULT 1,
                cancellable INTEGER NOT NULL DEFAULT 1,
                resumable INTEGER NOT NULL DEFAULT 1,
                spec_json TEXT NOT NULL,
                last_updated INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create tasks table")?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                progress REAL NOT NULL,
                state_blob BLOB NOT NULL,
                size_bytes INTEGER NOT NULL,
                compression TEXT,
                checksum TEXT NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create checkpoints table")?;

        // Create indices
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_owner ON tasks(owner)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_checkpoints_task ON checkpoints(task_id)")
            .execute(&self.pool)
            .await?;

        info!("Database migrations complete");
        Ok(())
    }

    /// Save or update a task
    pub async fn save_task(&self, task: &TaskLifecycle) -> Result<()> {
        let status_json = serde_json::to_string(&task.status)?;
        let spec_json = serde_json::to_string(&task.spec)?;

        sqlx::query(
            r#"
            INSERT INTO tasks (
                id, owner, status, progress, created_at, current_tower,
                pausable, cancellable, resumable, spec_json, last_updated
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                progress = excluded.progress,
                current_tower = excluded.current_tower,
                last_updated = excluded.last_updated
            "#,
        )
        .bind(task.id.to_string())
        .bind(task.owner.as_str())
        .bind(status_json)
        .bind(task.progress)
        .bind(task.created_at.timestamp())
        .bind(task.current_tower.as_ref().map(|t| t.as_str()))
        .bind(task.pausable as i32)
        .bind(task.cancellable as i32)
        .bind(task.resumable as i32)
        .bind(spec_json)
        .bind(task.last_updated.timestamp())
        .execute(&self.pool)
        .await
        .context("Failed to save task")?;

        Ok(())
    }

    /// Get a task by ID
    pub async fn get_task(&self, id: TaskId) -> Result<Option<TaskLifecycle>> {
        let row = sqlx::query(
            r#"
            SELECT id, owner, status, progress, created_at, current_tower,
                   pausable, cancellable, resumable, spec_json, last_updated
            FROM tasks
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch task")?;

        let Some(row) = row else {
            return Ok(None);
        };

        let task = self.row_to_task(row)?;
        Ok(Some(task))
    }

    /// List tasks with filter
    pub async fn list_tasks(&self, filter: &TaskFilter) -> Result<Vec<TaskLifecycle>> {
        let mut query = String::from(
            "SELECT id, owner, status, progress, created_at, current_tower, \
             pausable, cancellable, resumable, spec_json, last_updated \
             FROM tasks WHERE 1=1",
        );

        let mut bindings = Vec::new();

        if let Some(ref owner) = filter.owner {
            query.push_str(" AND owner = ?");
            bindings.push(owner.as_str());
        }

        if let Some(ref tower) = filter.tower {
            query.push_str(" AND current_tower = ?");
            bindings.push(tower.as_str());
        }

        query.push_str(" ORDER BY created_at DESC LIMIT 1000");

        let mut sql_query = sqlx::query(&query);
        for binding in bindings {
            sql_query = sql_query.bind(binding);
        }

        let rows = sql_query.fetch_all(&self.pool).await?;

        let tasks: Result<Vec<_>> = rows.into_iter().map(|row| self.row_to_task(row)).collect();
        tasks
    }

    /// Delete a task
    pub async fn delete_task(&self, id: TaskId) -> Result<()> {
        sqlx::query("DELETE FROM tasks WHERE id = ?1")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .context("Failed to delete task")?;

        Ok(())
    }

    /// Save a checkpoint
    pub async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO checkpoints (
                id, task_id, created_at, progress, state_blob,
                size_bytes, compression, checksum
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(checkpoint.id.as_ref())
        .bind(checkpoint.task_id.to_string())
        .bind(checkpoint.created_at.timestamp())
        .bind(checkpoint.progress)
        .bind(&checkpoint.state)
        .bind(checkpoint.metadata.size_bytes as i64)
        .bind(checkpoint.metadata.compression.map(|c| format!("{:?}", c)))
        .bind(checkpoint.metadata.checksum.as_ref())
        .execute(&self.pool)
        .await
        .context("Failed to save checkpoint")?;

        Ok(())
    }

    /// Get a checkpoint
    pub async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>> {
        let row = sqlx::query(
            r#"
            SELECT id, task_id, created_at, progress, state_blob,
                   size_bytes, compression, checksum
            FROM checkpoints
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch checkpoint")?;

        let Some(row) = row else {
            return Ok(None);
        };

        let checkpoint = self.row_to_checkpoint(row)?;
        Ok(Some(checkpoint))
    }

    /// List checkpoints for a task
    pub async fn list_checkpoints(&self, task_id: TaskId) -> Result<Vec<Checkpoint>> {
        let rows = sqlx::query(
            r#"
            SELECT id, task_id, created_at, progress, state_blob,
                   size_bytes, compression, checksum
            FROM checkpoints
            WHERE task_id = ?1
            ORDER BY created_at DESC
            "#,
        )
        .bind(task_id.to_string())
        .fetch_all(&self.pool)
        .await
        .context("Failed to list checkpoints")?;

        let checkpoints: Result<Vec<_>> =
            rows.into_iter().map(|row| self.row_to_checkpoint(row)).collect();
        checkpoints
    }

    /// Delete old checkpoints for a specific task, keeping only the most recent ones
    pub async fn delete_old_checkpoints(&self, task_id: TaskId, keep_count: usize) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM checkpoints
            WHERE task_id = ?1
            AND id NOT IN (
                SELECT id FROM checkpoints
                WHERE task_id = ?1
                ORDER BY created_at DESC
                LIMIT ?2
            )
            "#,
        )
        .bind(task_id.to_string())
        .bind(keep_count as i64)
        .execute(&self.pool)
        .await
        .context("Failed to delete old checkpoints")?;

        Ok(())
    }

    /// Clean up checkpoints older than max_age seconds across all tasks
    pub async fn cleanup_old_checkpoints(&self, max_age_seconds: u64) -> Result<u64> {
        use chrono::Utc;

        let cutoff_time = Utc::now().timestamp() - max_age_seconds as i64;

        let result = sqlx::query(
            r#"
            DELETE FROM checkpoints
            WHERE created_at < ?1
            "#,
        )
        .bind(cutoff_time)
        .execute(&self.pool)
        .await
        .context("Failed to cleanup old checkpoints")?;

        Ok(result.rows_affected())
    }

    /// Convert database row to TaskLifecycle
    fn row_to_task(&self, row: sqlx::sqlite::SqliteRow) -> Result<TaskLifecycle> {
        use chrono::TimeZone;

        let id_str: String = row.try_get("id")?;
        let id: TaskId = id_str.parse()?;

        let owner: String = row.try_get("owner")?;
        let status_json: String = row.try_get("status")?;
        let progress: f64 = row.try_get("progress")?;
        let created_at_ts: i64 = row.try_get("created_at")?;
        let current_tower: Option<String> = row.try_get("current_tower")?;
        let pausable: i32 = row.try_get("pausable")?;
        let cancellable: i32 = row.try_get("cancellable")?;
        let resumable: i32 = row.try_get("resumable")?;
        let spec_json: String = row.try_get("spec_json")?;
        let last_updated_ts: i64 = row.try_get("last_updated")?;

        Ok(TaskLifecycle {
            id,
            status: serde_json::from_str(&status_json)?,
            progress: progress as f32,
            created_at: Utc.timestamp_opt(created_at_ts, 0).single().ok_or_else(|| {
                anyhow::anyhow!("Invalid created_at timestamp: {}", created_at_ts)
            })?,
            eta_seconds: None,
            current_tower: current_tower.map(TowerId::from),
            owner: UserId::from(owner),
            spec: serde_json::from_str(&spec_json)?,
            checkpoint_ids: Vec::new(),
            pausable: pausable != 0,
            cancellable: cancellable != 0,
            resumable: resumable != 0,
            last_updated: Utc.timestamp_opt(last_updated_ts, 0).single().ok_or_else(|| {
                anyhow::anyhow!("Invalid last_updated timestamp: {}", last_updated_ts)
            })?,
        })
    }

    /// Convert database row to Checkpoint
    fn row_to_checkpoint(&self, row: sqlx::sqlite::SqliteRow) -> Result<Checkpoint> {
        use super::CheckpointMetadata;
        use chrono::TimeZone;

        let id: String = row.try_get("id")?;
        let task_id_str: String = row.try_get("task_id")?;
        let task_id: TaskId = task_id_str.parse()?;
        let created_at_ts: i64 = row.try_get("created_at")?;
        let progress: f64 = row.try_get("progress")?;
        let state: Vec<u8> = row.try_get("state_blob")?;
        let size_bytes: i64 = row.try_get("size_bytes")?;
        let compression: Option<String> = row.try_get("compression")?;
        let checksum: String = row.try_get("checksum")?;

        Ok(Checkpoint {
            id: Arc::from(id),
            task_id,
            created_at: Utc.timestamp_opt(created_at_ts, 0).single().ok_or_else(|| {
                anyhow::anyhow!("Invalid created_at timestamp: {}", created_at_ts)
            })?,
            progress: progress as f32,
            state,
            metadata: CheckpointMetadata {
                size_bytes: size_bytes as u64,
                compression: compression.and_then(|s| {
                    if s == "Zstd" {
                        Some(super::CompressionAlgorithm::Zstd)
                    } else {
                        None
                    }
                }),
                checksum: Arc::from(checksum),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_lifecycle::types::{Priority, ResourceRequirements, TaskSpec};

    async fn create_test_storage() -> Result<TaskStorage> {
        TaskStorage::new("sqlite::memory:").await
    }

    #[tokio::test]
    async fn test_save_and_get_task() -> Result<()> {
        let storage = create_test_storage().await?;

        let owner = UserId::from("alice");
        let spec = TaskSpec {
            task_type: "test".into(),
            config: serde_json::json!({}),
            required_capabilities: vec![],
            resources: ResourceRequirements::default(),
            priority: Priority::Standard,
        };

        let task = TaskLifecycle::new(owner, spec);
        let task_id = task.id;

        // Save task
        storage.save_task(&task).await?;

        // Retrieve task
        let retrieved = storage.get_task(task_id).await?;
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, task_id);
        assert_eq!(retrieved.owner.as_str(), "alice");

        Ok(())
    }

    #[tokio::test]
    async fn test_list_tasks() -> Result<()> {
        let storage = create_test_storage().await?;

        let owner = UserId::from("alice");

        // Create multiple tasks
        for i in 0..5 {
            let spec = TaskSpec {
                task_type: format!("test-{}", i).into(),
                config: serde_json::json!({}),
                required_capabilities: vec![],
                resources: ResourceRequirements::default(),
                priority: Priority::Standard,
            };

            let task = TaskLifecycle::new(owner.clone(), spec);
            storage.save_task(&task).await?;
        }

        // List tasks
        let filter = TaskFilter {
            owner: Some(owner),
            ..Default::default()
        };

        let tasks = storage.list_tasks(&filter).await?;
        assert_eq!(tasks.len(), 5);

        Ok(())
    }

    #[tokio::test]
    async fn test_checkpoint_storage() -> Result<()> {
        let storage = create_test_storage().await?;

        // Create and save task first
        let owner = UserId::from("alice");
        let spec = TaskSpec {
            task_type: "test".into(),
            config: serde_json::json!({}),
            required_capabilities: vec![],
            resources: ResourceRequirements::default(),
            priority: Priority::Standard,
        };

        let task = TaskLifecycle::new(owner, spec);
        storage.save_task(&task).await?;

        // Create checkpoint
        let state = vec![1, 2, 3, 4, 5];
        let checkpoint = Checkpoint::new(task.id, 0.5, state);
        let checkpoint_id = checkpoint.id.clone();

        // Save checkpoint
        storage.save_checkpoint(&checkpoint).await?;

        // Retrieve checkpoint
        let retrieved = storage.get_checkpoint(&checkpoint_id).await?;
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, checkpoint_id);
        assert_eq!(retrieved.task_id, task.id);
        assert_eq!(retrieved.progress, 0.5);

        Ok(())
    }
}
