// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! In-memory [`crate::consent_management::ConsentStorageBackend`] and [`crate::task_lifecycle::TaskStorageBackend`]
//! when NestGate is not used.

use crate::consent_management::{ConsentRecord, ConsentStatus, ConsentStorageBackend};
use crate::task_lifecycle::{Checkpoint, TaskFilter, TaskId, TaskLifecycle, TaskStorageBackend};
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

#[async_trait::async_trait]
impl ConsentStorageBackend for InMemoryStorage {
    async fn save(&self, record: &ConsentRecord) -> anyhow::Result<()> {
        let mut m = self.consents.write().await;
        m.insert(record.id.to_string(), record.clone());
        Ok(())
    }

    async fn get(&self, id: &str) -> anyhow::Result<Option<ConsentRecord>> {
        let m = self.consents.read().await;
        Ok(m.get(id).cloned())
    }

    async fn list_by_user(
        &self,
        user_id: &crate::task_lifecycle::UserId,
    ) -> anyhow::Result<Vec<ConsentRecord>> {
        let m = self.consents.read().await;
        Ok(m.values().filter(|r| &r.user_id == user_id).cloned().collect())
    }

    async fn list_by_task(&self, task_id: &TaskId) -> anyhow::Result<Vec<ConsentRecord>> {
        let m = self.consents.read().await;
        Ok(m.values().filter(|r| &r.task_id == task_id).cloned().collect())
    }

    async fn list_pending(&self) -> anyhow::Result<Vec<ConsentRecord>> {
        let m = self.consents.read().await;
        Ok(m.values().filter(|r| matches!(r.status, ConsentStatus::Pending)).cloned().collect())
    }

    async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let mut m = self.consents.write().await;
        m.remove(id);
        Ok(())
    }

    async fn flush(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl TaskStorageBackend for InMemoryStorage {
    async fn save_task(&self, task: &TaskLifecycle) -> anyhow::Result<()> {
        let mut m = self.tasks.write().await;
        m.insert(task.id, task.clone());
        Ok(())
    }

    async fn get_task(&self, id: TaskId) -> anyhow::Result<Option<TaskLifecycle>> {
        let m = self.tasks.read().await;
        Ok(m.get(&id).cloned())
    }

    async fn list_tasks(&self, filter: &TaskFilter) -> anyhow::Result<Vec<TaskLifecycle>> {
        let m = self.tasks.read().await;
        let mut out: Vec<TaskLifecycle> =
            m.values().filter(|t| matches_task_filter(t, filter)).cloned().collect();
        if let Some(limit) = filter.limit {
            out.truncate(limit);
        }
        Ok(out)
    }

    async fn delete_task(&self, id: TaskId) -> anyhow::Result<()> {
        let mut tasks = self.tasks.write().await;
        tasks.remove(&id);

        let mut cps = self.checkpoints.write().await;
        cps.retain(|_, cp| cp.task_id != id);
        Ok(())
    }

    async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> anyhow::Result<()> {
        let mut m = self.checkpoints.write().await;
        m.insert(checkpoint.id.to_string(), checkpoint.clone());
        Ok(())
    }

    async fn get_checkpoint(&self, id: &str) -> anyhow::Result<Option<Checkpoint>> {
        let m = self.checkpoints.read().await;
        Ok(m.get(id).cloned())
    }

    async fn list_checkpoints(&self, task_id: TaskId) -> anyhow::Result<Vec<Checkpoint>> {
        let m = self.checkpoints.read().await;
        let mut v: Vec<Checkpoint> = m.values().filter(|c| c.task_id == task_id).cloned().collect();
        v.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(v)
    }

    async fn delete_checkpoint(&self, id: &str) -> anyhow::Result<()> {
        let mut m = self.checkpoints.write().await;
        m.remove(id);
        Ok(())
    }

    async fn flush(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn cleanup_old_checkpoints(&self, max_age_seconds: u64) -> anyhow::Result<u64> {
        let cutoff = Utc::now().timestamp() - max_age_seconds as i64;
        let mut m = self.checkpoints.write().await;
        let before = m.len();
        m.retain(|_, cp| cp.created_at.timestamp() >= cutoff);
        let deleted = before.saturating_sub(m.len());
        Ok(deleted as u64)
    }

    async fn delete_old_checkpoints(
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
