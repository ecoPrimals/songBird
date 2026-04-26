// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! In-memory consent and task persistence when no `storage.*` capability provider is available.

use crate::consent_management::{ConsentRecord, ConsentStatus};
use crate::task_lifecycle::{Checkpoint, TaskFilter, TaskId, TaskLifecycle};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

fn matches_task_filter(task: &TaskLifecycle, filter: &TaskFilter) -> bool {
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

/// Shared in-memory store for consent and task data (non-durable fallback).
#[derive(Debug)]
pub struct InMemoryStorage {
    consents: Arc<RwLock<HashMap<String, ConsentRecord>>>,
    tasks: Arc<RwLock<HashMap<TaskId, TaskLifecycle>>>,
    checkpoints: Arc<RwLock<HashMap<String, Checkpoint>>>,
}

impl InMemoryStorage {
    /// Create empty in-memory storage.
    #[must_use]
    pub fn new() -> Self {
        Self {
            consents: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryStorage {
    /// Persist a consent record.
    pub async fn consent_save(&self, record: &ConsentRecord) -> anyhow::Result<()> {
        let mut m = self.consents.write().await;
        m.insert(record.id.to_string(), record.clone());
        Ok(())
    }

    /// Retrieve a consent record by ID.
    pub async fn consent_get(&self, id: &str) -> anyhow::Result<Option<ConsentRecord>> {
        let m = self.consents.read().await;
        Ok(m.get(id).cloned())
    }

    /// List consent records for a user.
    pub async fn consent_list_by_user(
        &self,
        user_id: &crate::task_lifecycle::UserId,
    ) -> anyhow::Result<Vec<ConsentRecord>> {
        let m = self.consents.read().await;
        Ok(m.values().filter(|r| &r.user_id == user_id).cloned().collect())
    }

    /// List consent records for a task.
    pub async fn consent_list_by_task(
        &self,
        task_id: &TaskId,
    ) -> anyhow::Result<Vec<ConsentRecord>> {
        let m = self.consents.read().await;
        Ok(m.values().filter(|r| &r.task_id == task_id).cloned().collect())
    }

    /// List consent records with pending status.
    pub async fn consent_list_pending(&self) -> anyhow::Result<Vec<ConsentRecord>> {
        let m = self.consents.read().await;
        Ok(m.values().filter(|r| matches!(r.status, ConsentStatus::Pending)).cloned().collect())
    }

    /// Delete a consent record.
    pub async fn consent_delete(&self, id: &str) -> anyhow::Result<()> {
        let mut m = self.consents.write().await;
        m.remove(id);
        Ok(())
    }

    /// Best-effort flush for consent data (no-op in memory).
    pub async fn consent_flush(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Persist or update a task.
    pub async fn save_task(&self, task: &TaskLifecycle) -> anyhow::Result<()> {
        let mut m = self.tasks.write().await;
        m.insert(task.id, task.clone());
        Ok(())
    }

    /// Retrieve a task by ID.
    pub async fn get_task(&self, id: TaskId) -> anyhow::Result<Option<TaskLifecycle>> {
        let m = self.tasks.read().await;
        Ok(m.get(&id).cloned())
    }

    /// List tasks matching a filter.
    pub async fn list_tasks(&self, filter: &TaskFilter) -> anyhow::Result<Vec<TaskLifecycle>> {
        let m = self.tasks.read().await;
        let mut out: Vec<TaskLifecycle> =
            m.values().filter(|t| matches_task_filter(t, filter)).cloned().collect();
        if let Some(limit) = filter.limit {
            out.truncate(limit);
        }
        Ok(out)
    }

    /// Delete a task by ID.
    pub async fn delete_task(&self, id: TaskId) -> anyhow::Result<()> {
        let mut tasks = self.tasks.write().await;
        tasks.remove(&id);

        let mut cps = self.checkpoints.write().await;
        cps.retain(|_, cp| cp.task_id != id);
        Ok(())
    }

    /// Persist a checkpoint.
    pub async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> anyhow::Result<()> {
        let mut m = self.checkpoints.write().await;
        m.insert(checkpoint.id.to_string(), checkpoint.clone());
        Ok(())
    }

    /// Retrieve a checkpoint by ID.
    pub async fn get_checkpoint(&self, id: &str) -> anyhow::Result<Option<Checkpoint>> {
        let m = self.checkpoints.read().await;
        Ok(m.get(id).cloned())
    }

    /// List checkpoints for a task (most recent first).
    pub async fn list_checkpoints(&self, task_id: TaskId) -> anyhow::Result<Vec<Checkpoint>> {
        let m = self.checkpoints.read().await;
        let mut v: Vec<Checkpoint> = m.values().filter(|c| c.task_id == task_id).cloned().collect();
        v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(v)
    }

    /// Delete a checkpoint by ID.
    pub async fn delete_checkpoint(&self, id: &str) -> anyhow::Result<()> {
        let mut m = self.checkpoints.write().await;
        m.remove(id);
        Ok(())
    }

    /// Best-effort flush for task/checkpoint data (no-op in memory).
    pub async fn flush_tasks(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Remove checkpoints older than `max_age_seconds`.
    pub async fn cleanup_old_checkpoints(&self, max_age_seconds: u64) -> anyhow::Result<u64> {
        let cutoff = Utc::now().timestamp() - max_age_seconds as i64;
        let mut m = self.checkpoints.write().await;
        let before = m.len();
        m.retain(|_, cp| cp.created_at.timestamp() >= cutoff);
        let deleted = before.saturating_sub(m.len());
        Ok(deleted as u64)
    }

    /// Keep only the `keep_count` most recent checkpoints for a task.
    pub async fn delete_old_checkpoints(
        &self,
        task_id: TaskId,
        keep_count: usize,
    ) -> anyhow::Result<()> {
        let mut m = self.checkpoints.write().await;
        let mut for_task: Vec<Checkpoint> =
            m.values().filter(|c| c.task_id == task_id).cloned().collect();
        for_task.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        if for_task.len() <= keep_count {
            return Ok(());
        }
        for cp in for_task.iter().skip(keep_count) {
            m.remove(cp.id.as_ref());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use chrono::{Duration, Utc};
    use std::sync::Arc;

    use super::InMemoryStorage;
    use crate::consent_management::{ConsentRecord, ConsentStatus};
    use crate::task_lifecycle::{
        Checkpoint, CheckpointMetadata, CompressionAlgorithm, Priority, ResourceRequirements,
        TaskFilter, TaskId, TaskLifecycle, TaskSpec, TaskStatus, TowerId, UserId,
    };

    fn sample_task(owner: UserId) -> TaskLifecycle {
        TaskLifecycle::new(
            owner,
            TaskSpec {
                task_type: Arc::from("demo"),
                config: serde_json::json!({}),
                required_capabilities: vec![],
                resources: ResourceRequirements::default(),
                priority: Priority::Standard,
            },
        )
    }

    fn sample_consent(
        user_id: UserId,
        task_id: TaskId,
        id: &str,
        status: ConsentStatus,
    ) -> ConsentRecord {
        ConsentRecord {
            id: Arc::from(id),
            user_id,
            task_id,
            operation: Arc::from("read"),
            estimated_cost: None,
            status,
            requested_at: Utc::now(),
            responded_at: None,
            reason: None,
        }
    }

    #[tokio::test]
    async fn consent_crud_and_queries() {
        let store = InMemoryStorage::new();
        let user = UserId::from("u1");
        let task_id = TaskId::new();
        let pending = sample_consent(user.clone(), task_id, "c1", ConsentStatus::Pending);
        let approved = sample_consent(user.clone(), task_id, "c2", ConsentStatus::Approved);

        store.consent_save(&pending).await.unwrap();
        store.consent_save(&approved).await.unwrap();

        assert_eq!(store.consent_get("c1").await.unwrap().unwrap().status, ConsentStatus::Pending);

        let by_user = store.consent_list_by_user(&user).await.unwrap();
        assert_eq!(by_user.len(), 2);

        let by_task = store.consent_list_by_task(&task_id).await.unwrap();
        assert_eq!(by_task.len(), 2);

        let pending_only = store.consent_list_pending().await.unwrap();
        assert_eq!(pending_only.len(), 1);
        assert_eq!(pending_only[0].id.as_ref(), "c1");

        store.consent_delete("c1").await.unwrap();
        assert!(store.consent_get("c1").await.unwrap().is_none());

        store.consent_flush().await.unwrap();
    }

    #[tokio::test]
    async fn task_save_get_list_delete_and_checkpoint_cleanup() {
        let store = InMemoryStorage::new();
        let owner = UserId::from("alice");
        let mut task = sample_task(owner.clone());
        let tid = task.id;
        task.current_tower = Some(TowerId::new("tower-a"));
        let started_at = Utc::now();
        task.status = TaskStatus::Running {
            started_at,
        };

        store.save_task(&task).await.unwrap();
        assert_eq!(store.get_task(tid).await.unwrap().unwrap().owner, owner);

        let other = sample_task(UserId::from("bob"));
        store.save_task(&other).await.unwrap();

        let listed = store
            .list_tasks(&TaskFilter {
                owner: Some(owner.clone()),
                status: Some(TaskStatus::Running {
                    started_at,
                }),
                tower: Some(TowerId::new("tower-a")),
                limit: Some(10),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, tid);

        let limited = store
            .list_tasks(&TaskFilter {
                limit: Some(1),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(limited.len(), 1);

        let cp = Checkpoint {
            id: Arc::from("cp-1"),
            task_id: tid,
            created_at: Utc::now(),
            progress: 0.5,
            state: vec![1, 2, 3],
            metadata: CheckpointMetadata {
                size_bytes: 3,
                compression: Some(CompressionAlgorithm::None),
                checksum: Arc::from("chk"),
            },
        };
        store.save_checkpoint(&cp).await.unwrap();
        assert_eq!(store.get_checkpoint("cp-1").await.unwrap().unwrap().task_id, tid);

        let cps = store.list_checkpoints(tid).await.unwrap();
        assert_eq!(cps.len(), 1);

        store.delete_task(tid).await.unwrap();
        assert!(store.get_task(tid).await.unwrap().is_none());
        assert!(store.list_checkpoints(tid).await.unwrap().is_empty());

        store.flush_tasks().await.unwrap();
    }

    #[tokio::test]
    async fn cleanup_old_checkpoints_by_age() {
        let store = InMemoryStorage::new();
        let task_id = TaskId::new();
        let old = Checkpoint {
            id: Arc::from("old"),
            task_id,
            created_at: Utc::now() - Duration::days(400),
            progress: 0.1,
            state: vec![],
            metadata: CheckpointMetadata {
                size_bytes: 0,
                compression: None,
                checksum: Arc::from("x"),
            },
        };
        let recent = Checkpoint {
            id: Arc::from("recent"),
            task_id,
            created_at: Utc::now(),
            progress: 0.2,
            state: vec![],
            metadata: CheckpointMetadata {
                size_bytes: 0,
                compression: None,
                checksum: Arc::from("y"),
            },
        };
        store.save_checkpoint(&old).await.unwrap();
        store.save_checkpoint(&recent).await.unwrap();

        let deleted = store.cleanup_old_checkpoints(86_400).await.unwrap();
        assert_eq!(deleted, 1);
        assert!(store.get_checkpoint("old").await.unwrap().is_none());
        assert!(store.get_checkpoint("recent").await.unwrap().is_some());
    }

    #[tokio::test]
    async fn delete_old_checkpoints_keeps_most_recent() {
        let store = InMemoryStorage::new();
        let task_id = TaskId::new();
        let base = Utc::now() - Duration::seconds(120);
        for (i, secs) in [(1_i64, 10_i64), (2, 20), (3, 30)] {
            let cp = Checkpoint {
                id: Arc::from(format!("cp-{i}")),
                task_id,
                created_at: base + Duration::seconds(secs),
                progress: 0.1 * i as f32,
                state: vec![],
                metadata: CheckpointMetadata {
                    size_bytes: 0,
                    compression: None,
                    checksum: Arc::from(format!("c{i}")),
                },
            };
            store.save_checkpoint(&cp).await.unwrap();
        }

        store.delete_old_checkpoints(task_id, 2).await.unwrap();
        let remaining = store.list_checkpoints(task_id).await.unwrap();
        assert_eq!(remaining.len(), 2);
        assert!(remaining.iter().any(|c| c.id.as_ref() == "cp-3"));
        assert!(remaining.iter().any(|c| c.id.as_ref() == "cp-2"));
    }

    #[tokio::test]
    async fn delete_checkpoint_by_id() {
        let store = InMemoryStorage::new();
        let task_id = TaskId::new();
        let cp = Checkpoint {
            id: Arc::from("gone"),
            task_id,
            created_at: Utc::now(),
            progress: 0.0,
            state: vec![],
            metadata: CheckpointMetadata {
                size_bytes: 0,
                compression: None,
                checksum: Arc::from("z"),
            },
        };
        store.save_checkpoint(&cp).await.unwrap();
        store.delete_checkpoint("gone").await.unwrap();
        assert!(store.get_checkpoint("gone").await.unwrap().is_none());
    }
}
