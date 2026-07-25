// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! CRUD, transitions, and checkpoint operations.

use super::TaskLifecycleManager;
use super::events::TaskEvent;
use crate::task_lifecycle::{
    Checkpoint, TaskFilter, TaskId, TaskLifecycle, TaskSpec, TowerId, UserId,
};
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{debug, info, warn};

impl TaskLifecycleManager {
    fn emit(&self, event: TaskEvent) {
        let _ = self.event_tx.send(event);
    }

    async fn require_task(&self, task_id: TaskId) -> Result<TaskLifecycle> {
        self.storage.get_task(task_id).await?.ok_or_else(|| anyhow::anyhow!("Task not found"))
    }

    /// Create a new task
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn create_task(&self, owner: UserId, spec: TaskSpec) -> Result<TaskId> {
        let task = TaskLifecycle::new(owner.clone(), spec);
        let task_id = task.id;

        self.storage.save_task(&task).await.context("Failed to save new task")?;

        self.emit(TaskEvent::Created {
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
        let mut task = self.require_task(task_id).await?;

        task.start(tower.clone()).map_err(|e| anyhow::anyhow!("{e}"))?;

        self.storage.save_task(&task).await?;

        self.emit(TaskEvent::Started {
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
        let mut task = self.require_task(task_id).await?;

        task.update_progress(progress);
        self.storage.save_task(&task).await?;

        self.emit(TaskEvent::ProgressUpdated {
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
        let mut task = self.require_task(task_id).await?;

        task.pause().map_err(|e| anyhow::anyhow!("{e}"))?;
        self.storage.save_task(&task).await?;

        self.emit(TaskEvent::Paused {
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
        let mut task = self.require_task(task_id).await?;

        task.resume(tower.clone()).map_err(|e| anyhow::anyhow!("{e}"))?;

        self.storage.save_task(&task).await?;

        self.emit(TaskEvent::Resumed {
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
        let mut task = self.require_task(task_id).await?;

        task.complete().map_err(|e| anyhow::anyhow!("{e}"))?;
        self.storage.save_task(&task).await?;

        self.emit(TaskEvent::Completed {
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
        let mut task = self.require_task(task_id).await?;

        task.fail(Arc::clone(&error), 0).map_err(|e| anyhow::anyhow!("{e}"))?;

        self.storage.save_task(&task).await?;

        self.emit(TaskEvent::Failed {
            task_id,
            error: Arc::clone(&error),
        });

        warn!("Task failed: {} - {}", task_id, error);
        Ok(())
    }

    /// Cancel a task
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn cancel_task(&self, task_id: TaskId, reason: Option<Arc<str>>) -> Result<()> {
        let mut task = self.require_task(task_id).await?;

        task.cancel(reason.as_ref().map(Arc::clone)).map_err(|e| anyhow::anyhow!("{e}"))?;

        self.storage.save_task(&task).await?;

        self.emit(TaskEvent::Cancelled {
            task_id,
            reason: reason.as_ref().map(Arc::clone),
        });

        info!("Task cancelled: {} - {:?}", task_id, reason);
        Ok(())
    }

    /// Create a checkpoint for a task
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn create_checkpoint(&self, task_id: TaskId, state: Vec<u8>) -> Result<Arc<str>> {
        let task = self.require_task(task_id).await?;

        let checkpoint = self.make_checkpoint_async(task_id, task.progress, state).await?;

        let checkpoint_id = Arc::clone(&checkpoint.id);

        self.storage.save_checkpoint(&checkpoint).await?;

        self.storage
            .delete_old_checkpoints(task_id, self.checkpoint_config.max_checkpoints_per_task)
            .await?;

        self.emit(TaskEvent::CheckpointCreated {
            task_id,
            checkpoint_id: Arc::clone(&checkpoint_id),
        });

        debug!("Checkpoint created: {} for task {}", checkpoint_id, task_id);
        Ok(checkpoint_id)
    }

    async fn make_checkpoint_async(
        &self,
        task_id: TaskId,
        progress: f32,
        state: Vec<u8>,
    ) -> Result<Checkpoint> {
        let threshold = self.checkpoint_config.compression_threshold as usize;
        if cfg!(feature = "task-checkpoint-gzip") && state.len() > threshold {
            Checkpoint::new_compressed_with_crypto(task_id, progress, state, self.crypto.as_ref())
                .await
        } else {
            Checkpoint::new_with_crypto(task_id, progress, state, self.crypto.as_ref()).await
        }
    }

    #[expect(dead_code, reason = "sync fallback retained for non-async contexts and testing")]
    fn make_checkpoint(
        &self,
        task_id: TaskId,
        progress: f32,
        state: Vec<u8>,
    ) -> Result<Checkpoint> {
        let threshold = self.checkpoint_config.compression_threshold as usize;
        if cfg!(feature = "task-checkpoint-gzip") && state.len() > threshold {
            Checkpoint::new_compressed(task_id, progress, state)
        } else {
            Ok(Checkpoint::new(task_id, progress, state))
        }
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

        checkpoint
            .verify_with_crypto(self.crypto.as_ref())
            .await
            .context("Checkpoint integrity check failed")?;

        let state = checkpoint.get_state()?;

        info!("Resuming from checkpoint: {} for task {}", checkpoint_id, checkpoint.task_id);

        Ok((checkpoint.task_id, state))
    }

    /// Subscribe to task events
    #[must_use]
    pub fn subscribe_events(&self) -> tokio::sync::broadcast::Receiver<TaskEvent> {
        self.event_tx.subscribe()
    }
}
