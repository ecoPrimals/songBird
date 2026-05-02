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

mod cleanup;
mod events;
mod ops;
mod storage;

#[cfg(test)]
mod tests;

pub use events::TaskEvent;

use crate::task_lifecycle::{CheckpointConfig, TaskStorage};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

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
    pub(crate) storage: Arc<TaskStorage>,
    pub(crate) checkpoint_config: CheckpointConfig,
    pub(crate) event_tx: broadcast::Sender<TaskEvent>,
    pub(crate) cleanup_interval: std::time::Duration,
}

impl TaskLifecycleManager {
    /// Create a new task lifecycle manager
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn new(database_url: &str) -> Result<Self> {
        let storage = storage::connect_task_storage(database_url).await?;

        let (event_tx, _) = broadcast::channel(1000);

        let manager = Self {
            storage,
            checkpoint_config: CheckpointConfig::default(),
            event_tx,
            cleanup_interval: songbird_types::defaults::timeouts::DEFAULT_CLEANUP_INTERVAL,
        };

        cleanup::spawn_checkpoint_cleanup_task(
            Arc::clone(&manager.storage),
            manager.cleanup_interval,
            manager.checkpoint_config.max_age_seconds,
        );

        info!("TaskLifecycleManager initialized");
        Ok(manager)
    }
}
