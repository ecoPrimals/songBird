// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![cfg_attr(
    test,
    expect(clippy::float_cmp, reason = "test: exact float comparison is intentional")
)]
//! Task lifecycle types
//!
//! Modern Rust types with:
//! - No unsafe code
//! - Zero-copy where possible (`Arc<str>`)
//! - Type safety over runtime checks
//! - Clear ownership semantics

use super::{TaskId, TowerId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Task status
///
/// NOTE: Using default (externally tagged) serde representation for `serde_json` compatibility.
/// Changed from bincode to `serde_json` (v3.21.0, Feb 5 2026) to support `serde_json::Value` in `TaskSpec`.
/// JSON format: `{"Queued": {}}` or `{"Running": {"started_at": ...}}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is queued, waiting for resources
    Queued,

    /// Task is currently running
    Running {
        started_at: DateTime<Utc>,
    },

    /// Task is paused (can be resumed)
    Paused {
        paused_at: DateTime<Utc>,
    },

    /// Task completed successfully
    Completed {
        completed_at: DateTime<Utc>,
    },

    /// Task failed
    Failed {
        failed_at: DateTime<Utc>,
        error: Arc<str>,
        retry_count: u32,
    },

    /// Task was cancelled by user
    Cancelled {
        cancelled_at: DateTime<Utc>,
        reason: Option<Arc<str>>,
    },
}

impl TaskStatus {
    /// Check if task is terminal (won't change)
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed { .. } | Self::Failed { .. } | Self::Cancelled { .. })
    }

    /// Check if task is active (running or queued)
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self, Self::Queued | Self::Running { .. })
    }

    /// Check if task can be paused
    #[must_use]
    pub const fn can_pause(&self) -> bool {
        matches!(self, Self::Running { .. })
    }

    /// Check if task can be resumed
    #[must_use]
    pub const fn can_resume(&self) -> bool {
        matches!(self, Self::Paused { .. })
    }
}

/// Task specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    /// Task type (capability-based, no hardcoding)
    pub task_type: Arc<str>,

    /// Task configuration (opaque JSON)
    pub config: serde_json::Value,

    /// Required capabilities
    pub required_capabilities: Vec<Arc<str>>,

    /// Estimated resource requirements
    pub resources: ResourceRequirements,

    /// Task priority
    pub priority: Priority,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: Option<u32>,
    pub memory_mb: Option<u64>,
    pub gpu_count: Option<u32>,
    pub network_mbps: Option<u32>,
    pub storage_gb: Option<u64>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_cores: Some(1),
            memory_mb: Some(1024),
            gpu_count: None,
            network_mbps: Some(10),
            storage_gb: Some(1),
        }
    }
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum Priority {
    Low = 0,
    #[default]
    Standard = 1,
    High = 2,
    Critical = 3,
}

/// Complete task lifecycle information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLifecycle {
    /// Unique task identifier
    pub id: TaskId,

    /// Current status
    pub status: TaskStatus,

    /// Progress (0.0 = not started, 1.0 = complete)
    pub progress: f32,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Estimated completion time (seconds)
    pub eta_seconds: Option<u64>,

    /// Tower currently executing the task
    pub current_tower: Option<TowerId>,

    /// User who owns this task
    pub owner: UserId,

    /// Task specification
    pub spec: TaskSpec,

    /// Checkpoint IDs (for resume)
    pub checkpoint_ids: Vec<Arc<str>>,

    /// Task can be paused
    pub pausable: bool,

    /// Task can be cancelled
    pub cancellable: bool,

    /// Task can be resumed from checkpoint
    pub resumable: bool,

    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

impl TaskLifecycle {
    /// Create a new task lifecycle
    #[must_use]
    pub fn new(owner: UserId, spec: TaskSpec) -> Self {
        let now = Utc::now();
        Self {
            id: TaskId::new(),
            status: TaskStatus::Queued,
            progress: 0.0,
            created_at: now,
            eta_seconds: None,
            current_tower: None,
            owner,
            spec,
            checkpoint_ids: Vec::new(),
            pausable: true,
            cancellable: true,
            resumable: true,
            last_updated: now,
        }
    }

    /// Update progress (0.0 - 1.0)
    pub fn update_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
        self.last_updated = Utc::now();
    }

    /// Transition to a new status
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn transition_to(&mut self, new_status: TaskStatus) -> Result<(), Arc<str>> {
        // Validate state transition
        if self.status.is_terminal() {
            return Err("Cannot transition from terminal state".into());
        }

        self.status = new_status;
        self.last_updated = Utc::now();
        Ok(())
    }

    /// Start the task
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn start(&mut self, tower: TowerId) -> Result<(), Arc<str>> {
        if !matches!(self.status, TaskStatus::Queued | TaskStatus::Paused { .. }) {
            return Err("Task can only start from Queued or Paused state".into());
        }

        self.status = TaskStatus::Running {
            started_at: Utc::now(),
        };
        self.current_tower = Some(tower);
        self.last_updated = Utc::now();
        Ok(())
    }

    /// Pause the task
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn pause(&mut self) -> Result<(), Arc<str>> {
        if !self.status.can_pause() {
            return Err("Task cannot be paused in current state".into());
        }

        self.status = TaskStatus::Paused {
            paused_at: Utc::now(),
        };
        self.last_updated = Utc::now();
        Ok(())
    }

    /// Resume the task
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn resume(&mut self, tower: TowerId) -> Result<(), Arc<str>> {
        if !self.status.can_resume() {
            return Err("Task cannot be resumed in current state".into());
        }

        self.status = TaskStatus::Running {
            started_at: Utc::now(),
        };
        self.current_tower = Some(tower);
        self.last_updated = Utc::now();
        Ok(())
    }

    /// Complete the task successfully
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn complete(&mut self) -> Result<(), Arc<str>> {
        if !self.status.is_active() {
            return Err("Task must be active to complete".into());
        }

        self.status = TaskStatus::Completed {
            completed_at: Utc::now(),
        };
        self.progress = 1.0;
        self.last_updated = Utc::now();
        Ok(())
    }

    /// Fail the task
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn fail(&mut self, error: impl Into<Arc<str>>, retry_count: u32) -> Result<(), Arc<str>> {
        self.status = TaskStatus::Failed {
            failed_at: Utc::now(),
            error: error.into(),
            retry_count,
        };
        self.last_updated = Utc::now();
        Ok(())
    }

    /// Cancel the task
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn cancel(&mut self, reason: Option<Arc<str>>) -> Result<(), Arc<str>> {
        if !self.cancellable {
            return Err("Task is not cancellable".into());
        }

        if self.status.is_terminal() {
            return Err("Cannot cancel terminal task".into());
        }

        self.status = TaskStatus::Cancelled {
            cancelled_at: Utc::now(),
            reason,
        };
        self.last_updated = Utc::now();
        Ok(())
    }
}

/// Task filter for queries
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TaskFilter {
    pub owner: Option<UserId>,
    pub status: Option<TaskStatus>,
    pub tower: Option<TowerId>,
    pub priority: Option<Priority>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: Option<usize>, // Maximum number of results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_lifecycle_creation() {
        let owner = UserId::from("alice");
        let spec = TaskSpec {
            task_type: "test-task".into(),
            config: serde_json::json!({}),
            required_capabilities: vec!["compute".into()],
            resources: ResourceRequirements::default(),
            priority: Priority::Standard,
        };

        let task = TaskLifecycle::new(owner, spec);
        assert_eq!(task.status, TaskStatus::Queued);
        assert_eq!(task.progress, 0.0);
        assert!(!task.status.is_terminal());
        assert!(task.status.is_active());
    }

    #[test]
    fn test_task_state_transitions() {
        let owner = UserId::from("alice");
        let spec = TaskSpec {
            task_type: "test-task".into(),
            config: serde_json::json!({}),
            required_capabilities: vec![],
            resources: ResourceRequirements::default(),
            priority: Priority::Standard,
        };

        let mut task = TaskLifecycle::new(owner, spec);

        // Queued -> Running
        assert!(task.start(TowerId::from("tower-1")).is_ok());
        assert!(matches!(task.status, TaskStatus::Running { .. }));

        // Running -> Paused
        assert!(task.pause().is_ok());
        assert!(matches!(task.status, TaskStatus::Paused { .. }));

        // Paused -> Running
        assert!(task.resume(TowerId::from("tower-1")).is_ok());
        assert!(matches!(task.status, TaskStatus::Running { .. }));

        // Running -> Completed
        assert!(task.complete().is_ok());
        assert!(matches!(task.status, TaskStatus::Completed { .. }));
        assert_eq!(task.progress, 1.0);

        // Cannot transition from terminal state
        assert!(task.start(TowerId::from("tower-1")).is_err());
    }

    #[test]
    fn test_progress_tracking() {
        let owner = UserId::from("alice");
        let spec = TaskSpec {
            task_type: "test-task".into(),
            config: serde_json::json!({}),
            required_capabilities: vec![],
            resources: ResourceRequirements::default(),
            priority: Priority::Standard,
        };

        let mut task = TaskLifecycle::new(owner, spec);

        // Progress starts at 0
        assert_eq!(task.progress, 0.0);

        // Update progress
        task.update_progress(0.5);
        assert_eq!(task.progress, 0.5);

        // Progress clamped to [0, 1]
        task.update_progress(1.5);
        assert_eq!(task.progress, 1.0);

        task.update_progress(-0.5);
        assert_eq!(task.progress, 0.0);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Low < Priority::Standard);
        assert!(Priority::Standard < Priority::High);
        assert!(Priority::High < Priority::Critical);
    }
}
