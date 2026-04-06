// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![cfg_attr(
    test,
    expect(clippy::float_cmp, reason = "test: exact float comparison is intentional")
)]
//! Task Lifecycle Manager
//!
//! Coordinates all task lifecycle operations with:
//! - No unsafe code
//! - Async operations throughout
//! - Event emission for observability
//! - Background cleanup tasks

use super::{
    Checkpoint, CheckpointConfig, TaskFilter, TaskId, TaskLifecycle, TaskSpec, TaskStorageBackend,
    TowerId, UserId,
};
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

/// Task event for streaming
#[derive(Debug, Clone)]
pub enum TaskEvent {
    Created {
        task_id: TaskId,
        owner: UserId,
    },
    Started {
        task_id: TaskId,
        tower: TowerId,
    },
    ProgressUpdated {
        task_id: TaskId,
        progress: f32,
    },
    Paused {
        task_id: TaskId,
    },
    Resumed {
        task_id: TaskId,
        tower: TowerId,
    },
    CheckpointCreated {
        task_id: TaskId,
        checkpoint_id: Arc<str>,
    },
    Completed {
        task_id: TaskId,
    },
    Failed {
        task_id: TaskId,
        error: Arc<str>,
    },
    Cancelled {
        task_id: TaskId,
        reason: Option<Arc<str>>,
    },
}

/// Manages the complete lifecycle of tasks in Songbird.
///
/// Provides comprehensive task management including:
/// - Task creation and tracking
/// - Progress monitoring with real-time updates
/// - Checkpointing for long-running tasks with compression
/// - Event streaming for observability
/// - Pause/resume capability
/// - Graceful cancellation
///
/// # Architecture
///
/// The manager coordinates between:
/// - **Storage layer**: `SQLite` for persistent task state
/// - **Checkpoint system**: gzip compression (flate2) with SHA-256 verification
/// - **Event system**: Broadcast channels for real-time updates
/// - **Background tasks**: Automatic cleanup of old tasks
///
/// # Example
/// ```rust,ignore
/// # use songbird_orchestrator::task_lifecycle::*;
/// # async fn example() -> anyhow::Result<()> {
/// // Initialize manager with database
/// let manager = TaskLifecycleManager::new("/tmp/songbird-data/tasks.db").await?;
///
/// // Create a task
/// let user_id = UserId::from("alice");
/// let spec = TaskSpec {
///     task_type: "ml_training".into(),
///     parameters: serde_json::json!({"model": "gpt", "epochs": 100}),
/// };
/// let task_id = manager.create_task(user_id, spec).await?;
///
/// // Start the task on a specific tower
/// let tower_id = TowerId::from("tower-001");
/// manager.start_task(task_id, tower_id).await?;
///
/// // Update progress periodically
/// manager.update_progress(task_id, 0.5).await?;
///
/// // Create checkpoints for recovery
/// manager.create_checkpoint(task_id, vec![1, 2, 3]).await?;
///
/// // Complete the task
/// manager.complete_task(task_id).await?;
/// # Ok(())
/// # }
/// ```
pub struct TaskLifecycleManager {
    storage: Arc<dyn TaskStorageBackend>,
    checkpoint_config: CheckpointConfig,
    event_tx: broadcast::Sender<TaskEvent>,
    cleanup_interval: std::time::Duration,
}

impl TaskLifecycleManager {
    /// Create a new task lifecycle manager
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn new(database_url: &str) -> Result<Self> {
        let storage: Arc<dyn TaskStorageBackend> = {
            #[cfg(feature = "sled-storage")]
            {
                Arc::new(
                    super::TaskStorage::new(database_url)
                        .await
                        .context("Failed to create task storage")?,
                )
            }
            #[cfg(not(feature = "sled-storage"))]
            {
                let _ = database_url;
                Arc::new(crate::storage_memory::InMemoryStorage::new())
            }
        };

        let (event_tx, _) = broadcast::channel(1000);

        let manager = Self {
            storage,
            checkpoint_config: CheckpointConfig::default(),
            event_tx,
            cleanup_interval: std::time::Duration::from_secs(3600), // 1 hour
        };

        // Start background cleanup task
        manager.start_cleanup_task();

        info!("TaskLifecycleManager initialized");
        Ok(manager)
    }

    /// Create a new task
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn create_task(&self, owner: UserId, spec: TaskSpec) -> Result<TaskId> {
        let task = TaskLifecycle::new(owner.clone(), spec);
        let task_id = task.id;

        self.storage.save_task(&task).await.context("Failed to save new task")?;

        // Emit event
        let _ = self.event_tx.send(TaskEvent::Created {
            task_id,
            owner: owner.clone(),
        });

        info!("Task created: {} by {}", task_id, owner);
        Ok(task_id)
    }

    /// Retrieve a task by its ID.
    ///
    /// # Arguments
    /// * `task_id` - The unique task identifier
    ///
    /// # Returns
    /// * `Some(TaskLifecycle)` if the task exists
    /// * `None` if the task was not found
    ///
    /// # Errors
    /// Returns error if database query fails
    pub async fn get_task(&self, task_id: TaskId) -> Result<Option<TaskLifecycle>> {
        self.storage.get_task(task_id).await.context("Failed to get task")
    }

    /// List all tasks matching the specified filter.
    ///
    /// # Arguments
    /// * `filter` - Filter criteria (owner, status, tower, time range)
    ///
    /// # Returns
    /// Vector of tasks matching the filter, ordered by creation time
    ///
    /// # Errors
    /// Returns error if database query fails
    pub async fn list_tasks(&self, filter: &TaskFilter) -> Result<Vec<TaskLifecycle>> {
        self.storage.list_tasks(filter).await.context("Failed to list tasks")
    }

    /// Start a pending task on the specified tower.
    ///
    /// Transitions the task from `Pending` to `Running` state and assigns it to a tower.
    ///
    /// # Arguments
    /// * `task_id` - The task to start
    /// * `tower` - The tower that will execute this task
    ///
    /// # Errors
    /// Returns error if:
    /// - Task not found
    /// - Task is not in `Pending` state
    /// - Database update fails
    pub async fn start_task(&self, task_id: TaskId, tower: TowerId) -> Result<()> {
        let mut task = self
            .storage
            .get_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        task.start(tower.clone()).map_err(|e| anyhow::anyhow!("{e}"))?;

        self.storage.save_task(&task).await?;

        // Emit event
        let _ = self.event_tx.send(TaskEvent::Started {
            task_id,
            tower: tower.clone(),
        });

        info!("Task started: {} on {}", task_id, tower);
        Ok(())
    }

    /// Update task progress
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn update_progress(&self, task_id: TaskId, progress: f32) -> Result<()> {
        let mut task = self
            .storage
            .get_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        task.update_progress(progress);
        self.storage.save_task(&task).await?;

        // Emit event
        let _ = self.event_tx.send(TaskEvent::ProgressUpdated {
            task_id,
            progress,
        });

        debug!("Task progress updated: {} -> {:.1}%", task_id, progress * 100.0);
        Ok(())
    }

    /// Pause a task
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn pause_task(&self, task_id: TaskId) -> Result<()> {
        let mut task = self
            .storage
            .get_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        task.pause().map_err(|e| anyhow::anyhow!("{e}"))?;
        self.storage.save_task(&task).await?;

        // Emit event
        let _ = self.event_tx.send(TaskEvent::Paused {
            task_id,
        });

        info!("Task paused: {}", task_id);
        Ok(())
    }

    /// Resume a task
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn resume_task(&self, task_id: TaskId, tower: TowerId) -> Result<()> {
        let mut task = self
            .storage
            .get_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        task.resume(tower.clone()).map_err(|e| anyhow::anyhow!("{e}"))?;

        self.storage.save_task(&task).await?;

        // Emit event
        let _ = self.event_tx.send(TaskEvent::Resumed {
            task_id,
            tower: tower.clone(),
        });

        info!("Task resumed: {} on {}", task_id, tower);
        Ok(())
    }

    /// Complete a task
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn complete_task(&self, task_id: TaskId) -> Result<()> {
        let mut task = self
            .storage
            .get_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        task.complete().map_err(|e| anyhow::anyhow!("{e}"))?;
        self.storage.save_task(&task).await?;

        // Emit event
        let _ = self.event_tx.send(TaskEvent::Completed {
            task_id,
        });

        info!("Task completed: {}", task_id);
        Ok(())
    }

    /// Fail a task
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn fail_task(&self, task_id: TaskId, error: Arc<str>) -> Result<()> {
        let mut task = self
            .storage
            .get_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        task.fail(error.clone(), 0).map_err(|e| anyhow::anyhow!("{e}"))?;

        self.storage.save_task(&task).await?;

        // Emit event
        let _ = self.event_tx.send(TaskEvent::Failed {
            task_id,
            error: error.clone(),
        });

        warn!("Task failed: {} - {}", task_id, error);
        Ok(())
    }

    /// Cancel a task
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn cancel_task(&self, task_id: TaskId, reason: Option<Arc<str>>) -> Result<()> {
        let mut task = self
            .storage
            .get_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        task.cancel(reason.clone()).map_err(|e| anyhow::anyhow!("{e}"))?;

        self.storage.save_task(&task).await?;

        // Emit event
        let _ = self.event_tx.send(TaskEvent::Cancelled {
            task_id,
            reason: reason.clone(),
        });

        info!("Task cancelled: {} - {:?}", task_id, reason);
        Ok(())
    }

    /// Create a checkpoint for a task
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn create_checkpoint(&self, task_id: TaskId, state: Vec<u8>) -> Result<Arc<str>> {
        let task = self
            .storage
            .get_task(task_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Task not found"))?;

        // Decide whether to compress based on size (gzip requires `task-checkpoint-gzip` feature)
        let checkpoint = if cfg!(feature = "task-checkpoint-gzip")
            && state.len() > self.checkpoint_config.compression_threshold as usize
        {
            Checkpoint::new_compressed(task_id, task.progress, state)?
        } else {
            Checkpoint::new(task_id, task.progress, state)
        };

        let checkpoint_id = checkpoint.id.clone();

        self.storage.save_checkpoint(&checkpoint).await?;

        // Clean up old checkpoints
        self.storage
            .delete_old_checkpoints(task_id, self.checkpoint_config.max_checkpoints_per_task)
            .await?;

        // Emit event
        let _ = self.event_tx.send(TaskEvent::CheckpointCreated {
            task_id,
            checkpoint_id: checkpoint_id.clone(),
        });

        debug!("Checkpoint created: {} for task {}", checkpoint_id, task_id);
        Ok(checkpoint_id)
    }

    /// Resume from a checkpoint
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn resume_from_checkpoint(&self, checkpoint_id: &str) -> Result<(TaskId, Vec<u8>)> {
        let checkpoint = self
            .storage
            .get_checkpoint(checkpoint_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Checkpoint not found"))?;

        // Verify integrity
        checkpoint.verify().context("Checkpoint integrity check failed")?;

        // Get decompressed state
        let state = checkpoint.get_state()?;

        info!("Resuming from checkpoint: {} for task {}", checkpoint_id, checkpoint.task_id);

        Ok((checkpoint.task_id, state))
    }

    /// Subscribe to task events
    #[must_use]
    pub fn subscribe_events(&self) -> broadcast::Receiver<TaskEvent> {
        self.event_tx.subscribe()
    }

    /// Start background cleanup task
    fn start_cleanup_task(&self) {
        let storage = Arc::clone(&self.storage);
        let interval = self.cleanup_interval;
        let max_age = self.checkpoint_config.max_age_seconds;

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;

                debug!("Running checkpoint cleanup");

                // Delete checkpoints older than max_age
                match storage.cleanup_old_checkpoints(max_age).await {
                    Ok(count) => {
                        if count > 0 {
                            info!("Cleaned up {} old checkpoints", count);
                        }
                    }
                    Err(e) => {
                        warn!("Checkpoint cleanup failed: {}", e);
                    }
                }
            }
        });

        debug!("Background cleanup task started");
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::task_lifecycle::types::{Priority, ResourceRequirements};

    async fn create_test_manager() -> Result<TaskLifecycleManager> {
        // Use a unique temp directory for each test to avoid data persistence
        let temp_dir = std::env::temp_dir().join(format!("songbird-test-{}", uuid::Uuid::new_v4()));
        TaskLifecycleManager::new(temp_dir.to_str().unwrap()).await
    }

    #[tokio::test]
    async fn test_task_creation() -> Result<()> {
        let manager = create_test_manager().await?;

        let owner = UserId::from("alice");
        let spec = TaskSpec {
            task_type: "test-task".into(),
            config: serde_json::json!({}),
            required_capabilities: vec!["compute".into()],
            resources: ResourceRequirements::default(),
            priority: Priority::Standard,
        };

        let task_id = manager.create_task(owner.clone(), spec).await?;

        let task = manager.get_task(task_id).await?;
        assert!(task.is_some());

        let task = task.unwrap();
        assert_eq!(task.owner, owner);

        Ok(())
    }

    #[tokio::test]
    async fn test_task_lifecycle() -> Result<()> {
        let manager = create_test_manager().await?;

        let owner = UserId::from("alice");
        let spec = TaskSpec {
            task_type: "test-task".into(),
            config: serde_json::json!({}),
            required_capabilities: vec![],
            resources: ResourceRequirements::default(),
            priority: Priority::Standard,
        };

        // Create task
        let task_id = manager.create_task(owner, spec).await?;

        // Start task
        let tower = TowerId::from("tower-1");
        manager.start_task(task_id, tower.clone()).await?;

        // Update progress
        manager.update_progress(task_id, 0.5).await?;

        // Pause task
        manager.pause_task(task_id).await?;

        // Resume task
        manager.resume_task(task_id, tower).await?;

        // Complete task
        manager.complete_task(task_id).await?;

        // Verify final state
        let task = manager.get_task(task_id).await?.unwrap();
        assert!(matches!(task.status, super::super::types::TaskStatus::Completed { .. }));
        assert_eq!(task.progress, 1.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_checkpoint_and_resume() -> Result<()> {
        let manager = create_test_manager().await?;

        let owner = UserId::from("alice");
        let spec = TaskSpec {
            task_type: "test-task".into(),
            config: serde_json::json!({}),
            required_capabilities: vec![],
            resources: ResourceRequirements::default(),
            priority: Priority::Standard,
        };

        // Create and start task
        let task_id = manager.create_task(owner, spec).await?;
        let tower = TowerId::from("tower-1");
        manager.start_task(task_id, tower).await?;

        // Create checkpoint
        let state = vec![1, 2, 3, 4, 5];
        let checkpoint_id = manager.create_checkpoint(task_id, state.clone()).await?;

        // Resume from checkpoint
        let (resumed_task_id, resumed_state) =
            manager.resume_from_checkpoint(&checkpoint_id).await?;

        assert_eq!(resumed_task_id, task_id);
        assert_eq!(resumed_state, state);

        Ok(())
    }

    #[tokio::test]
    async fn test_event_emission() -> Result<()> {
        let manager = create_test_manager().await?;
        let mut rx = manager.subscribe_events();

        let owner = UserId::from("alice");
        let spec = TaskSpec {
            task_type: "test-task".into(),
            config: serde_json::json!({}),
            required_capabilities: vec![],
            resources: ResourceRequirements::default(),
            priority: Priority::Standard,
        };

        // Create task (should emit Created event)
        let task_id = manager.create_task(owner.clone(), spec).await?;

        // Receive event
        let event = rx.recv().await?;
        assert!(matches!(event, TaskEvent::Created { .. }));

        Ok(())
    }

    #[tokio::test]
    async fn test_list_tasks() -> Result<()> {
        let manager = create_test_manager().await?;

        let owner = UserId::from("alice");

        // Create multiple tasks
        for i in 0..5 {
            let spec = TaskSpec {
                task_type: format!("test-{i}").into(),
                config: serde_json::json!({}),
                required_capabilities: vec![],
                resources: ResourceRequirements::default(),
                priority: Priority::Standard,
            };

            manager.create_task(owner.clone(), spec).await?;
        }

        // List all tasks for user
        let filter = TaskFilter {
            owner: Some(owner),
            ..Default::default()
        };

        let tasks = manager.list_tasks(&filter).await?;
        assert_eq!(tasks.len(), 5);

        Ok(())
    }
}
