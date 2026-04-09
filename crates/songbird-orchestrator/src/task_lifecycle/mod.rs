// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Task Lifecycle Management
//!
//! Modern, idiomatic implementation of task lifecycle with:
//! - Zero unsafe code
//! - Capability-based design
//! - Runtime discovery
//! - Complete implementation (no mocks)

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

mod checkpoint;
mod manager;
#[cfg(feature = "sled-storage")]
mod storage_sled;
pub mod types;

pub use checkpoint::*;
pub use manager::{TaskEvent, TaskLifecycleManager};
#[cfg(feature = "sled-storage")]
pub use storage_sled::*;
pub use types::*;

/// Async task persistence backend.
///
/// Production path: [`NestGateStorage`](crate::storage_nestgate::NestGateStorage) delegates
/// to the `storage.*` capability provider (NestGate) via JSON-RPC at runtime.
/// Fallback: [`InMemoryStorage`](crate::storage_memory::InMemoryStorage) when no provider is available.
/// Legacy: `TaskStorage` (sled, behind deprecated `sled-storage` feature).
#[async_trait::async_trait]
pub trait TaskStorageBackend: Send + Sync {
    /// Persist or update a task.
    async fn save_task(&self, task: &TaskLifecycle) -> anyhow::Result<()>;

    /// Retrieve a task by ID.
    async fn get_task(&self, id: TaskId) -> anyhow::Result<Option<TaskLifecycle>>;

    /// List tasks matching a filter.
    async fn list_tasks(&self, filter: &TaskFilter) -> anyhow::Result<Vec<TaskLifecycle>>;

    /// Delete a task by ID.
    async fn delete_task(&self, id: TaskId) -> anyhow::Result<()>;

    /// Persist a checkpoint.
    async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> anyhow::Result<()>;

    /// Retrieve a checkpoint by ID.
    async fn get_checkpoint(&self, id: &str) -> anyhow::Result<Option<Checkpoint>>;

    /// List checkpoints for a task (most recent first).
    async fn list_checkpoints(&self, task_id: TaskId) -> anyhow::Result<Vec<Checkpoint>>;

    /// Delete a checkpoint by ID.
    async fn delete_checkpoint(&self, id: &str) -> anyhow::Result<()>;

    /// Flush pending writes to durable storage.
    async fn flush(&self) -> anyhow::Result<()>;

    /// Remove checkpoints older than `max_age_seconds` (relative to `Utc::now()`).
    async fn cleanup_old_checkpoints(&self, max_age_seconds: u64) -> anyhow::Result<u64>;

    /// Keep only the `keep_count` most recent checkpoints for a task (by creation time).
    async fn delete_old_checkpoints(
        &self,
        task_id: TaskId,
        keep_count: usize,
    ) -> anyhow::Result<()>;
}

/// Task identifier (UUID v7 for time-ordered IDs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(Uuid);

impl TaskId {
    /// Create a new time-ordered task ID
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Create from existing UUID
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Create from string (convenience method)
    ///
    /// # Errors
    ///
    /// Returns error if string is not a valid UUID
    pub fn from_string(s: &str) -> anyhow::Result<Self> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Get UUID value
    #[must_use]
    pub const fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for TaskId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// User identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(Arc<str>);

impl UserId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Tower identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TowerId(Arc<str>);

impl TowerId {
    pub fn new(id: impl Into<Arc<str>>) -> Self {
        Self(id.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TowerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for TowerId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for TowerId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_task_id_creation() {
        let id1 = TaskId::new();
        let id2 = TaskId::new();
        assert_ne!(id1, id2, "Task IDs should be unique");
    }

    #[test]
    fn test_task_id_roundtrip() {
        let id = TaskId::new();
        let s = id.to_string();
        let parsed: TaskId = s.parse().unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_user_id_zero_copy() {
        let s = "alice".to_string();
        let id1 = UserId::new(s);
        let id2 = id1.clone();
        // Arc means clone is cheap (just increment ref count)
        assert_eq!(id1, id2);
        assert_eq!(id1.as_str(), "alice");
    }
}
