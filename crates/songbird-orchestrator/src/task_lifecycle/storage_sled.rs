// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Task Lifecycle Storage - Pure Rust sled implementation
//!
//! **Evolution**: Migrated from sqlx to sled (Jan 27, 2026)
//! - ✅ 100% Pure Rust (TRUE ecoBin!)
//! - ✅ Embedded key-value store
//! - ✅ ACID transactions
//! - ✅ Zero-copy reads
//! - ✅ Crash-safe
//! - ✅ Simpler than SQL
//!
//! **Evolution**: Switched `TaskLifecycle` storage from bincode to JSON (Feb 5, 2026)
//! - Bincode doesn't support `serde_json::Value` (requires `deserialize_any`)
//! - JSON serialization naturally handles dynamic Value types
//! - Checkpoints still use bincode (no Value fields)
//!
//! Modern async-compatible storage with:
//! - No unsafe code
//! - Type-safe serialization
//! - Atomic operations
//! - Efficient indexing

use super::{Checkpoint, TaskFilter, TaskId, TaskLifecycle};
use anyhow::{Context, Result};
use sled::Db;
use std::sync::Arc;
use tracing::{debug, info};

/// Pure Rust task storage backed by sled
///
/// Thread-safe and async-compatible via `Arc<Db>` and `spawn_blocking`
#[derive(Clone)]
pub struct TaskStorage {
    db: Arc<Db>,
}

impl TaskStorage {
    /// Create a new task storage
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be opened
    pub async fn new(database_path: &str) -> Result<Self> {
        info!("Initializing task storage (sled): {}", database_path);

        let path = database_path.to_string();
        let db = tokio::task::spawn_blocking(move || {
            sled::open(&path).context("Failed to open sled database")
        })
        .await
        .context("Task panicked while opening database")??;

        info!("✅ Task storage initialized (100% Pure Rust!)");
        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Save or update a task
    ///
    /// # Errors
    ///
    /// Returns error if serialization or write fails
    pub async fn save_task(&self, task: &TaskLifecycle) -> Result<()> {
        let task = task.clone();
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            // Serialize task as JSON (bincode doesn't support serde_json::Value)
            let value = serde_json::to_vec(&task).context("Failed to serialize task")?;

            // Store with key: task/{task_id}
            let key = format!("task/{}", task.id);
            db.insert(key.as_bytes(), value).context("Failed to save task")?;

            // Create indices for efficient queries

            // Index by owner: owner_tasks/{owner}/{task_id}
            let owner_key = format!("owner_tasks/{}/{}", task.owner.as_str(), task.id);
            db.insert(owner_key.as_bytes(), task.id.to_string().as_bytes())
                .context("Failed to create owner index")?;

            // Index by status: status_tasks/{status}/{task_id}
            let status_str = format!("{:?}", task.status);
            let status_key = format!("status_tasks/{}/{}", status_str, task.id);
            db.insert(status_key.as_bytes(), task.id.to_string().as_bytes())
                .context("Failed to create status index")?;

            // Index by tower if assigned: tower_tasks/{tower_id}/{task_id}
            if let Some(tower_id) = &task.current_tower {
                let tower_key = format!("tower_tasks/{}/{}", tower_id.as_str(), task.id);
                db.insert(tower_key.as_bytes(), task.id.to_string().as_bytes())
                    .context("Failed to create tower index")?;
            }

            debug!("✅ Task saved: {}", task.id.to_string());
            Ok(())
        })
        .await
        .context("Task panicked while saving")?
    }

    /// Get a task by ID
    ///
    /// # Errors
    ///
    /// Returns error if deserialization fails
    pub async fn get_task(&self, id: TaskId) -> Result<Option<TaskLifecycle>> {
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let key = format!("task/{id}");
            let value = db.get(key.as_bytes()).context("Failed to fetch task")?;

            if let Some(bytes) = value {
                let task: TaskLifecycle =
                    serde_json::from_slice(&bytes).context("Failed to deserialize task")?;
                Ok(Some(task))
            } else {
                Ok(None)
            }
        })
        .await
        .context("Task panicked while getting")?
    }

    /// List tasks with filter
    ///
    /// # Errors
    ///
    /// Returns error if query or deserialization fails
    pub async fn list_tasks(&self, filter: &TaskFilter) -> Result<Vec<TaskLifecycle>> {
        let filter = filter.clone();
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            debug!("Listing tasks with filter: {:?}", filter);

            let mut tasks = Vec::new();

            // Use appropriate index based on filter
            if let Some(owner) = &filter.owner {
                // Query by owner index
                let prefix = format!("owner_tasks/{}/", owner.as_str());
                for item in db.scan_prefix(prefix.as_bytes()) {
                    let (_key, task_id_bytes) = item.context("Failed to scan owner tasks")?;
                    let task_id_str = String::from_utf8(task_id_bytes.to_vec())
                        .context("Invalid task ID in index")?;
                    let task_id = TaskId::from_string(&task_id_str)?;

                    // Sync read within blocking context
                    let key = format!("task/{task_id}");
                    if let Some(bytes) = db.get(key.as_bytes())? {
                        let task: TaskLifecycle = serde_json::from_slice(&bytes)?;
                        if Self::matches_filter_static(&task, &filter) {
                            tasks.push(task);
                        }
                    }
                }
            } else if let Some(status) = &filter.status {
                // Query by status index
                let status_str = format!("{status:?}");
                let prefix = format!("status_tasks/{status_str}/");
                for item in db.scan_prefix(prefix.as_bytes()) {
                    let (_key, task_id_bytes) = item.context("Failed to scan status tasks")?;
                    let task_id_str = String::from_utf8(task_id_bytes.to_vec())
                        .context("Invalid task ID in index")?;
                    let task_id = TaskId::from_string(&task_id_str)?;

                    // Sync read within blocking context
                    let key = format!("task/{task_id}");
                    if let Some(bytes) = db.get(key.as_bytes())? {
                        let task: TaskLifecycle = serde_json::from_slice(&bytes)?;
                        if Self::matches_filter_static(&task, &filter) {
                            tasks.push(task);
                        }
                    }
                }
            } else {
                // Full scan (no specific index)
                for item in db.scan_prefix(b"task/") {
                    let (_key, value) = item.context("Failed to scan tasks")?;
                    let task: TaskLifecycle =
                        serde_json::from_slice(&value).context("Failed to deserialize task")?;

                    if Self::matches_filter_static(&task, &filter) {
                        tasks.push(task);
                    }
                }
            }

            // Apply limit if specified
            if let Some(limit) = filter.limit {
                tasks.truncate(limit);
            }

            debug!("✅ Found {} tasks", tasks.len());
            Ok(tasks)
        })
        .await
        .context("Task panicked while listing")?
    }

    /// Check if task matches filter (static version for `spawn_blocking`)
    fn matches_filter_static(task: &TaskLifecycle, filter: &TaskFilter) -> bool {
        if let Some(owner) = &filter.owner
            && task.owner != *owner
        {
            return false;
        }

        if let Some(status) = &filter.status
            && task.status != *status
        {
            return false;
        }

        if let Some(tower) = &filter.tower
            && task.current_tower.as_ref() != Some(tower)
        {
            return false;
        }

        true
    }

    /// Delete a task
    ///
    /// # Errors
    ///
    /// Returns error if deletion fails
    pub async fn delete_task(&self, id: TaskId) -> Result<()> {
        debug!("Deleting task: {}", id.to_string());

        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            // Get task first to clean up indices
            let key = format!("task/{id}");
            if let Some(bytes) = db.get(key.as_bytes())? {
                let task: TaskLifecycle = serde_json::from_slice(&bytes)?;

                // Delete main record
                db.remove(key.as_bytes()).context("Failed to delete task")?;

                // Delete owner index
                let owner_key = format!("owner_tasks/{}/{}", task.owner.as_str(), id);
                db.remove(owner_key.as_bytes())?;

                // Delete status index
                let status_str = format!("{:?}", task.status);
                let status_key = format!("status_tasks/{status_str}/{id}");
                db.remove(status_key.as_bytes())?;

                // Delete tower index if exists
                if let Some(tower_id) = &task.current_tower {
                    let tower_key = format!("tower_tasks/{}/{}", tower_id.as_str(), id);
                    db.remove(tower_key.as_bytes())?;
                }

                debug!("✅ Task deleted: {}", id.to_string());
            }

            Ok(())
        })
        .await
        .context("Task panicked while deleting")?
    }

    /// Save a checkpoint
    ///
    /// # Errors
    ///
    /// Returns error if serialization or write fails
    pub async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let checkpoint = checkpoint.clone();
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let value =
                bincode::serialize(&checkpoint).context("Failed to serialize checkpoint")?;

            // Store with key: checkpoint/{checkpoint_id}
            let key = format!("checkpoint/{}", checkpoint.id);
            db.insert(key.as_bytes(), value).context("Failed to save checkpoint")?;

            // Index by task: task_checkpoints/{task_id}/{checkpoint_id}
            let task_key = format!("task_checkpoints/{}/{}", checkpoint.task_id, checkpoint.id);
            db.insert(task_key.as_bytes(), checkpoint.id.to_string().as_bytes())
                .context("Failed to create task checkpoint index")?;

            debug!("✅ Checkpoint saved: {}", checkpoint.id.to_string());
            Ok(())
        })
        .await
        .context("Task panicked while saving checkpoint")?
    }

    /// Get a checkpoint by ID
    ///
    /// # Errors
    ///
    /// Returns error if deserialization fails
    pub async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>> {
        let id = id.to_string();
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let key = format!("checkpoint/{id}");
            let value = db.get(key.as_bytes()).context("Failed to fetch checkpoint")?;

            if let Some(bytes) = value {
                let checkpoint: Checkpoint =
                    bincode::deserialize(&bytes).context("Failed to deserialize checkpoint")?;
                Ok(Some(checkpoint))
            } else {
                Ok(None)
            }
        })
        .await
        .context("Task panicked while getting checkpoint")?
    }

    /// List checkpoints for a task
    ///
    /// # Errors
    ///
    /// Returns error if query or deserialization fails
    pub async fn list_checkpoints(&self, task_id: TaskId) -> Result<Vec<Checkpoint>> {
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            debug!("Listing checkpoints for task: {}", task_id.to_string());

            let prefix = format!("task_checkpoints/{task_id}/");
            let mut checkpoints = Vec::new();

            for item in db.scan_prefix(prefix.as_bytes()) {
                let (_key, checkpoint_id_bytes) = item.context("Failed to scan checkpoints")?;
                let checkpoint_id = String::from_utf8(checkpoint_id_bytes.to_vec())
                    .context("Invalid checkpoint ID in index")?;

                // Read checkpoint directly
                let key = format!("checkpoint/{checkpoint_id}");
                if let Some(bytes) = db.get(key.as_bytes())? {
                    let checkpoint: Checkpoint = bincode::deserialize(&bytes)?;
                    checkpoints.push(checkpoint);
                }
            }

            // Sort by creation time (most recent first)
            checkpoints.sort_by(|a, b| b.created_at.cmp(&a.created_at));

            debug!("✅ Found {} checkpoints", checkpoints.len());
            Ok(checkpoints)
        })
        .await
        .context("Task panicked while listing checkpoints")?
    }

    /// Delete a checkpoint
    ///
    /// # Errors
    ///
    /// Returns error if deletion fails
    pub async fn delete_checkpoint(&self, id: &str) -> Result<()> {
        let id = id.to_string();
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            debug!("Deleting checkpoint: {}", id);

            let key = format!("checkpoint/{id}");
            if let Some(bytes) = db.get(key.as_bytes())? {
                let checkpoint: Checkpoint = bincode::deserialize(&bytes)?;

                // Delete main record
                db.remove(key.as_bytes()).context("Failed to delete checkpoint")?;

                // Delete task index
                let task_key = format!("task_checkpoints/{}/{}", checkpoint.task_id, id);
                db.remove(task_key.as_bytes())?;

                debug!("✅ Checkpoint deleted: {}", id);
            }

            Ok(())
        })
        .await
        .context("Task panicked while deleting checkpoint")?
    }

    /// Flush all pending writes to disk
    ///
    /// # Errors
    ///
    /// Returns error if flush fails
    pub async fn flush(&self) -> Result<()> {
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            db.flush().context("Failed to flush database")?;
            Ok(())
        })
        .await
        .context("Task panicked while flushing")?
    }

    /// Delete old checkpoints for a specific task, keeping only the most recent ones
    ///
    /// # Errors
    ///
    /// Returns error if query or deletion fails
    pub async fn delete_old_checkpoints(&self, task_id: TaskId, keep_count: usize) -> Result<()> {
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            let prefix = format!("task_checkpoints/{task_id}/");
            let mut checkpoints = Vec::new();

            // Collect all checkpoints for this task
            for item in db.scan_prefix(prefix.as_bytes()) {
                let (_key, checkpoint_id_bytes) = item.context("Failed to scan checkpoints")?;
                let checkpoint_id = String::from_utf8(checkpoint_id_bytes.to_vec())
                    .context("Invalid checkpoint ID in index")?;

                // Read checkpoint
                let key = format!("checkpoint/{checkpoint_id}");
                if let Some(bytes) = db.get(key.as_bytes())? {
                    let checkpoint: Checkpoint = bincode::deserialize(&bytes)?;
                    checkpoints.push(checkpoint);
                }
            }

            // Sort by creation time (most recent first)
            checkpoints.sort_by(|a, b| b.created_at.cmp(&a.created_at));

            // Delete old checkpoints (keep only keep_count most recent)
            if checkpoints.len() > keep_count {
                for checkpoint in checkpoints.iter().skip(keep_count) {
                    // Delete checkpoint
                    let key = format!("checkpoint/{}", checkpoint.id);
                    db.remove(key.as_bytes())?;

                    // Delete task index
                    let task_key =
                        format!("task_checkpoints/{}/{}", checkpoint.task_id, checkpoint.id);
                    db.remove(task_key.as_bytes())?;

                    debug!("Deleted old checkpoint: {}", checkpoint.id.to_string());
                }

                let deleted_count = checkpoints.len() - keep_count;
                debug!(
                    "✅ Deleted {} old checkpoints for task {}",
                    deleted_count,
                    task_id.to_string()
                );
            }

            Ok(())
        })
        .await
        .context("Task panicked while deleting old checkpoints")?
    }

    /// Clean up checkpoints older than `max_age` seconds across all tasks
    ///
    /// # Errors
    ///
    /// Returns error if query or deletion fails
    pub async fn cleanup_old_checkpoints(&self, max_age_seconds: u64) -> Result<u64> {
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            use chrono::Utc;

            let cutoff_time = Utc::now().timestamp() - max_age_seconds as i64;
            let mut deleted_count = 0u64;

            // Scan all checkpoints
            for item in db.scan_prefix(b"checkpoint/") {
                let (key, value) = item.context("Failed to scan checkpoints")?;

                // Deserialize to check age
                if let Ok(checkpoint) = bincode::deserialize::<Checkpoint>(&value)
                    && checkpoint.created_at.timestamp() < cutoff_time
                {
                    // Delete checkpoint
                    db.remove(&key)?;

                    // Delete task index
                    let task_key =
                        format!("task_checkpoints/{}/{}", checkpoint.task_id, checkpoint.id);
                    db.remove(task_key.as_bytes())?;

                    deleted_count += 1;
                    debug!("Deleted old checkpoint: {}", checkpoint.id.to_string());
                }
            }

            if deleted_count > 0 {
                info!("✅ Cleaned up {} old checkpoints", deleted_count);
            }

            Ok(deleted_count)
        })
        .await
        .context("Task panicked while cleaning up checkpoints")?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_lifecycle::{Priority, ResourceRequirements, TaskSpec, UserId};
    use tempfile::TempDir;

    fn create_test_task() -> TaskLifecycle {
        TaskLifecycle::new(
            UserId::new("test-user"),
            TaskSpec {
                task_type: "test".to_string().into(),
                config: serde_json::json!({"type": "test"}),
                required_capabilities: vec![],
                resources: ResourceRequirements {
                    cpu_cores: None,
                    memory_mb: None,
                    gpu_count: None,
                    network_mbps: None,
                    storage_gb: None,
                },
                priority: Priority::Standard,
            },
        )
    }

    #[tokio::test]
    async fn test_task_storage_new() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("tasks.db");

        let storage = TaskStorage::new(db_path.to_str().unwrap()).await;
        assert!(storage.is_ok());
    }

    #[tokio::test]
    async fn test_save_and_get_task() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("tasks.db");
        let storage = TaskStorage::new(db_path.to_str().unwrap()).await.unwrap();

        let task = create_test_task();
        let task_id = task.id;

        storage.save_task(&task).await.unwrap();

        let retrieved = storage.get_task(task_id).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_task = retrieved.unwrap();
        assert_eq!(retrieved_task.id, task.id);
        assert_eq!(retrieved_task.owner, task.owner);
    }

    #[tokio::test]
    async fn test_list_tasks_by_owner() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("tasks.db");
        let storage = TaskStorage::new(db_path.to_str().unwrap()).await.unwrap();

        let owner = UserId::new("test-user");

        // Create multiple tasks for same owner
        for _ in 0..3 {
            let mut task = create_test_task();
            task.owner = owner.clone();
            storage.save_task(&task).await.unwrap();
        }

        let filter = TaskFilter {
            owner: Some(owner),
            status: None,
            tower: None,
            limit: None,
            priority: None,
            created_after: None,
            created_before: None,
        };

        let tasks = storage.list_tasks(&filter).await.unwrap();
        assert_eq!(tasks.len(), 3);
    }

    #[tokio::test]
    async fn test_delete_task() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("tasks.db");
        let storage = TaskStorage::new(db_path.to_str().unwrap()).await.unwrap();

        let task = create_test_task();
        let task_id = task.id;

        storage.save_task(&task).await.unwrap();
        assert!(storage.get_task(task_id).await.unwrap().is_some());

        storage.delete_task(task_id).await.unwrap();
        assert!(storage.get_task(task_id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_checkpoint_operations() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("tasks.db");
        let storage = TaskStorage::new(db_path.to_str().unwrap()).await.unwrap();

        let task = create_test_task();
        let task_id = task.id;
        storage.save_task(&task).await.unwrap();

        // Create checkpoint
        let checkpoint = Checkpoint::new(task_id, 0.5, vec![1, 2, 3]);
        let checkpoint_id = checkpoint.id.clone();

        storage.save_checkpoint(&checkpoint).await.unwrap();

        // Retrieve checkpoint
        let retrieved = storage.get_checkpoint(checkpoint_id.as_ref()).await.unwrap();
        assert!(retrieved.is_some());

        // List checkpoints for task
        let checkpoints = storage.list_checkpoints(task_id).await.unwrap();
        assert_eq!(checkpoints.len(), 1);

        // Delete checkpoint
        storage.delete_checkpoint(checkpoint_id.as_ref()).await.unwrap();
        assert!(storage.get_checkpoint(checkpoint_id.as_ref()).await.unwrap().is_none());
    }
}
