// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::wire::{parse_get_value_string, parse_list_keys};
use crate::consent_management::{ConsentRecord, ConsentStatus};
use crate::task_lifecycle::{Checkpoint, TaskFilter, TaskId, TaskLifecycle, UserId};
use anyhow::{Context, Result};
use serde_json::{Value, json};
use songbird_universal_ipc::tower_atomic::TowerAtomicClient;
use std::path::{Path, PathBuf};
use tracing::debug;

/// JSON-RPC storage backend: one new [`TowerAtomicClient`] per operation (SB-03).
///
/// Connects to whichever primal provides the `storage.*` capability domain
/// via a Unix socket discovered at runtime. No primal identity assumed.
#[derive(Debug)]
pub struct IpcStorageBackend {
    socket_path: PathBuf,
}

impl IpcStorageBackend {
    /// Create a backend that talks to a `storage.*` capability provider at the given socket path.
    #[must_use]
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
        }
    }

    /// Socket path used for JSON-RPC.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    async fn rpc(&self, method: &str, params: Value) -> Result<Value> {
        debug!(method, path = %self.socket_path.display(), "storage capability RPC");
        let client =
            TowerAtomicClient::connect_unix_path(&self.socket_path).await.with_context(|| {
                format!("storage provider connect failed ({})", self.socket_path.display())
            })?;
        client.call(method, params).await.with_context(|| format!("storage RPC {method} failed"))
    }

    async fn storage_put_str(&self, key: &str, value: &str) -> Result<()> {
        self.rpc("storage.put", json!({ "key": key, "value": value })).await?;
        Ok(())
    }

    async fn storage_get_str(&self, key: &str) -> Result<Option<String>> {
        let v = self.rpc("storage.get", json!({ "key": key })).await?;
        parse_get_value_string(&v)
    }

    async fn storage_delete(&self, key: &str) -> Result<()> {
        self.rpc("storage.delete", json!({ "key": key })).await?;
        Ok(())
    }

    async fn storage_list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let v = self.rpc("storage.list", json!({ "prefix": prefix })).await?;
        parse_list_keys(&v)
    }

    async fn storage_flush(&self) -> Result<()> {
        self.rpc("storage.flush", json!({})).await?;
        Ok(())
    }
}

fn consent_main_key(id: &str) -> String {
    format!("songbird/consent/main/{id}")
}

fn consent_user_idx(user: &UserId, id: &str) -> String {
    format!("songbird/consent/user/{}/{}", user.as_str(), id)
}

fn consent_task_idx(task_id: &TaskId, id: &str) -> String {
    format!("songbird/consent/task/{}/{}", task_id, id)
}

fn task_main_key(id: TaskId) -> String {
    format!("songbird/task/task/{id}")
}

fn task_owner_idx(owner: &UserId, id: TaskId) -> String {
    format!("songbird/task/owner_tasks/{}/{}", owner.as_str(), id)
}

fn task_tower_idx(tower: &crate::task_lifecycle::TowerId, id: TaskId) -> String {
    format!("songbird/task/tower_tasks/{}/{}", tower.as_str(), id)
}

fn checkpoint_main_key(id: &str) -> String {
    format!("songbird/checkpoint/{id}")
}

fn checkpoint_task_idx(task_id: TaskId, cp_id: &str) -> String {
    format!("songbird/checkpoint/task_checkpoints/{task_id}/{cp_id}")
}

impl IpcStorageBackend {
    /// Persist a consent record.
    pub async fn consent_save(&self, record: &ConsentRecord) -> Result<()> {
        let json = serde_json::to_string(record).context("serialize consent record")?;
        let id = record.id.as_ref();
        self.storage_put_str(&consent_main_key(id), &json).await?;
        self.storage_put_str(&consent_user_idx(&record.user_id, id), id)
            .await
            .context("consent user index")?;
        self.storage_put_str(&consent_task_idx(&record.task_id, id), id)
            .await
            .context("consent task index")?;
        Ok(())
    }

    /// Retrieve a consent record by ID.
    pub async fn consent_get(&self, id: &str) -> Result<Option<ConsentRecord>> {
        let Some(s) = self.storage_get_str(&consent_main_key(id)).await? else {
            return Ok(None);
        };
        let r: ConsentRecord = serde_json::from_str(&s).context("deserialize consent")?;
        Ok(Some(r))
    }

    /// List consent records for a user.
    pub async fn consent_list_by_user(&self, user_id: &UserId) -> Result<Vec<ConsentRecord>> {
        let prefix = format!("songbird/consent/user/{}/", user_id.as_str());
        let keys = self.storage_list_keys(&prefix).await?;
        let mut out = Vec::new();
        for k in keys {
            let Some(id) = k.strip_prefix(&prefix) else {
                continue;
            };
            if let Some(r) = self.consent_get(id).await? {
                out.push(r);
            }
        }
        Ok(out)
    }

    /// List consent records for a task.
    pub async fn consent_list_by_task(&self, task_id: &TaskId) -> Result<Vec<ConsentRecord>> {
        let prefix = format!("songbird/consent/task/{}/", task_id);
        let keys = self.storage_list_keys(&prefix).await?;
        let mut out = Vec::new();
        for k in keys {
            let Some(id) = k.strip_prefix(&prefix) else {
                continue;
            };
            if let Some(r) = self.consent_get(id).await? {
                out.push(r);
            }
        }
        Ok(out)
    }

    /// List consent records with pending status.
    pub async fn consent_list_pending(&self) -> Result<Vec<ConsentRecord>> {
        let keys = self.storage_list_keys("songbird/consent/main/").await?;
        let mut out = Vec::new();
        for k in keys {
            let Some(id) = k.strip_prefix("songbird/consent/main/") else {
                continue;
            };
            if let Some(r) = self.consent_get(id).await?
                && matches!(r.status, ConsentStatus::Pending)
            {
                out.push(r);
            }
        }
        Ok(out)
    }

    /// Delete a consent record.
    pub async fn consent_delete(&self, id: &str) -> Result<()> {
        let Some(s) = self.storage_get_str(&consent_main_key(id)).await? else {
            return Ok(());
        };
        let record: ConsentRecord = serde_json::from_str(&s).context("deserialize for delete")?;
        self.storage_delete(&consent_main_key(id)).await?;
        self.storage_delete(&consent_user_idx(&record.user_id, id)).await?;
        self.storage_delete(&consent_task_idx(&record.task_id, id)).await?;
        Ok(())
    }

    /// Flush consent-related writes to the storage provider.
    pub async fn consent_flush(&self) -> Result<()> {
        self.storage_flush().await
    }

    /// Persist or update a task.
    pub async fn save_task(&self, task: &TaskLifecycle) -> Result<()> {
        let json = serde_json::to_string(task).context("serialize task")?;
        let id = task.id;
        self.storage_put_str(&task_main_key(id), &json).await?;
        self.storage_put_str(&task_owner_idx(&task.owner, id), &id.to_string()).await?;
        let status_str = format!("{:?}", task.status);
        self.storage_put_str(
            &format!("songbird/task/status_tasks/{status_str}/{id}"),
            &id.to_string(),
        )
        .await?;
        if let Some(tower) = &task.current_tower {
            self.storage_put_str(&task_tower_idx(tower, id), &id.to_string()).await?;
        }
        Ok(())
    }

    /// Retrieve a task by ID.
    pub async fn get_task(&self, id: TaskId) -> Result<Option<TaskLifecycle>> {
        let Some(s) = self.storage_get_str(&task_main_key(id)).await? else {
            return Ok(None);
        };
        let t: TaskLifecycle = serde_json::from_str(&s).context("deserialize task")?;
        Ok(Some(t))
    }

    /// List tasks matching a filter.
    pub async fn list_tasks(&self, filter: &TaskFilter) -> Result<Vec<TaskLifecycle>> {
        let mut tasks = Vec::new();
        if let Some(owner) = &filter.owner {
            let prefix = format!("songbird/task/owner_tasks/{}/", owner.as_str());
            let keys = self.storage_list_keys(&prefix).await?;
            for k in keys {
                let Some(rest) = k.strip_prefix(&prefix) else {
                    continue;
                };
                if let Ok(tid) = TaskId::from_string(rest)
                    && let Some(t) = self.get_task(tid).await?
                    && matches_filter(&t, filter)
                {
                    tasks.push(t);
                }
            }
        } else if let Some(status) = &filter.status {
            let status_str = format!("{status:?}");
            let prefix = format!("songbird/task/status_tasks/{status_str}/");
            let keys = self.storage_list_keys(&prefix).await?;
            for k in keys {
                let Some(rest) = k.strip_prefix(&prefix) else {
                    continue;
                };
                if let Ok(tid) = TaskId::from_string(rest)
                    && let Some(t) = self.get_task(tid).await?
                    && matches_filter(&t, filter)
                {
                    tasks.push(t);
                }
            }
        } else {
            let keys = self.storage_list_keys("songbird/task/task/").await?;
            for k in keys {
                let Some(rest) = k.strip_prefix("songbird/task/task/") else {
                    continue;
                };
                if let Ok(tid) = TaskId::from_string(rest)
                    && let Some(t) = self.get_task(tid).await?
                    && matches_filter(&t, filter)
                {
                    tasks.push(t);
                }
            }
        }
        if let Some(limit) = filter.limit {
            tasks.truncate(limit);
        }
        Ok(tasks)
    }

    /// Delete a task by ID.
    pub async fn delete_task(&self, id: TaskId) -> Result<()> {
        let Some(s) = self.storage_get_str(&task_main_key(id)).await? else {
            return Ok(());
        };
        let task: TaskLifecycle = serde_json::from_str(&s).context("deserialize task")?;
        self.storage_delete(&task_main_key(id)).await?;
        self.storage_delete(&task_owner_idx(&task.owner, id)).await?;
        let status_str = format!("{:?}", task.status);
        self.storage_delete(&format!("songbird/task/status_tasks/{status_str}/{id}")).await?;
        if let Some(tower) = &task.current_tower {
            self.storage_delete(&task_tower_idx(tower, id)).await?;
        }
        let cps = self.list_checkpoints(id).await?;
        for cp in cps {
            self.delete_checkpoint(cp.id.as_ref()).await?;
        }
        Ok(())
    }

    /// Persist a checkpoint.
    pub async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<()> {
        let json = serde_json::to_string(checkpoint).context("serialize checkpoint")?;
        let id = checkpoint.id.as_ref();
        self.storage_put_str(&checkpoint_main_key(id), &json).await?;
        self.storage_put_str(&checkpoint_task_idx(checkpoint.task_id, id), checkpoint.id.as_ref())
            .await?;
        Ok(())
    }

    /// Retrieve a checkpoint by ID.
    pub async fn get_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>> {
        let Some(s) = self.storage_get_str(&checkpoint_main_key(id)).await? else {
            return Ok(None);
        };
        let c: Checkpoint = serde_json::from_str(&s).context("deserialize checkpoint")?;
        Ok(Some(c))
    }

    /// List checkpoints for a task (most recent first).
    pub async fn list_checkpoints(&self, task_id: TaskId) -> Result<Vec<Checkpoint>> {
        let prefix = format!("songbird/checkpoint/task_checkpoints/{task_id}/");
        let keys = self.storage_list_keys(&prefix).await?;
        let mut checkpoints = Vec::new();
        for k in keys {
            let Some(cp_id) = k.strip_prefix(&prefix) else {
                continue;
            };
            if let Some(c) = self.get_checkpoint(cp_id).await? {
                checkpoints.push(c);
            }
        }
        checkpoints.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(checkpoints)
    }

    /// Delete a checkpoint by ID.
    pub async fn delete_checkpoint(&self, id: &str) -> Result<()> {
        let Some(s) = self.storage_get_str(&checkpoint_main_key(id)).await? else {
            return Ok(());
        };
        let cp: Checkpoint = serde_json::from_str(&s).context("deserialize checkpoint")?;
        self.storage_delete(&checkpoint_main_key(id)).await?;
        self.storage_delete(&checkpoint_task_idx(cp.task_id, id)).await?;
        Ok(())
    }

    /// Flush task/checkpoint writes to the storage provider.
    pub async fn flush_tasks(&self) -> Result<()> {
        self.storage_flush().await
    }

    /// Remove checkpoints older than `max_age_seconds`.
    pub async fn cleanup_old_checkpoints(&self, max_age_seconds: u64) -> Result<u64> {
        use chrono::Utc;
        let keys = self.storage_list_keys("songbird/checkpoint/").await?;
        let cutoff = Utc::now().timestamp() - max_age_seconds as i64;
        let mut deleted = 0u64;
        for k in keys {
            if k.contains("task_checkpoints/") {
                continue;
            }
            let parts: Vec<&str> = k.split('/').collect();
            if parts.len() != 3 {
                continue;
            }
            if let Some(s) = self.storage_get_str(&k).await?
                && let Ok(cp) = serde_json::from_str::<Checkpoint>(&s)
                && cp.created_at.timestamp() < cutoff
            {
                self.delete_checkpoint(cp.id.as_ref()).await?;
                deleted += 1;
            }
        }
        Ok(deleted)
    }

    /// Keep only the `keep_count` most recent checkpoints for a task.
    pub async fn delete_old_checkpoints(&self, task_id: TaskId, keep_count: usize) -> Result<()> {
        let mut cps = self.list_checkpoints(task_id).await?;
        cps.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        if cps.len() <= keep_count {
            return Ok(());
        }
        for cp in cps.iter().skip(keep_count) {
            self.delete_checkpoint(cp.id.as_ref()).await?;
        }
        Ok(())
    }
}

fn matches_filter(task: &TaskLifecycle, filter: &TaskFilter) -> bool {
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use uuid::Uuid;

    #[allow(
        unused_imports,
        reason = "wildcard documents parent module; private helpers imported below"
    )]
    use super::*;
    use super::{
        checkpoint_main_key, checkpoint_task_idx, consent_main_key, consent_task_idx,
        consent_user_idx, matches_filter, task_main_key, task_owner_idx, task_tower_idx,
    };
    use crate::task_lifecycle::{
        Priority, TaskFilter, TaskId, TaskLifecycle, TaskSpec, TaskStatus, TowerId, UserId,
    };

    #[test]
    fn consent_main_key_formats_path() {
        assert_eq!(consent_main_key("abc"), "songbird/consent/main/abc");
    }

    #[test]
    fn consent_user_idx_formats_path() {
        let u = UserId::from("alice");
        assert_eq!(consent_user_idx(&u, "rec-1"), "songbird/consent/user/alice/rec-1");
    }

    #[test]
    fn consent_task_idx_formats_path() {
        let tid = TaskId::from_uuid(Uuid::nil());
        assert_eq!(
            consent_task_idx(&tid, "c-9"),
            "songbird/consent/task/00000000-0000-0000-0000-000000000000/c-9"
        );
    }

    #[test]
    fn task_main_key_formats_path() {
        let id = TaskId::from_uuid(Uuid::from_u128(u128::MAX));
        assert_eq!(task_main_key(id), "songbird/task/task/ffffffff-ffff-ffff-ffff-ffffffffffff");
    }

    #[test]
    fn task_owner_idx_formats_path() {
        let owner = UserId::from("bob");
        let id = TaskId::from_uuid(Uuid::nil());
        assert_eq!(
            task_owner_idx(&owner, id),
            "songbird/task/owner_tasks/bob/00000000-0000-0000-0000-000000000000"
        );
    }

    #[test]
    fn task_tower_idx_formats_path() {
        let tower = TowerId::from("east-1");
        let id = TaskId::from_string("550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(
            task_tower_idx(&tower, id),
            "songbird/task/tower_tasks/east-1/550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn checkpoint_main_key_formats_path() {
        assert_eq!(checkpoint_main_key("cp-7"), "songbird/checkpoint/cp-7");
    }

    #[test]
    fn checkpoint_task_idx_formats_path() {
        let task_id = TaskId::from_uuid(Uuid::nil());
        assert_eq!(
            checkpoint_task_idx(task_id, "snap-a"),
            "songbird/checkpoint/task_checkpoints/00000000-0000-0000-0000-000000000000/snap-a"
        );
    }

    fn minimal_spec() -> TaskSpec {
        TaskSpec {
            task_type: "unit-test".into(),
            config: serde_json::json!({}),
            required_capabilities: vec![],
            resources: crate::task_lifecycle::types::ResourceRequirements::default(),
            priority: Priority::Standard,
        }
    }

    fn sample_task(owner: &str, status: TaskStatus, tower: Option<TowerId>) -> TaskLifecycle {
        let mut t = TaskLifecycle::new(UserId::from(owner), minimal_spec());
        t.status = status;
        t.current_tower = tower;
        t
    }

    #[test]
    fn matches_filter_empty_filter_accepts_all() {
        let task = sample_task("alice", TaskStatus::Queued, None);
        let filter = TaskFilter::default();
        assert!(matches_filter(&task, &filter));
    }

    #[test]
    fn matches_filter_owner() {
        let task = sample_task("alice", TaskStatus::Queued, None);
        let mut f = TaskFilter::default();
        f.owner = Some(UserId::from("alice"));
        assert!(matches_filter(&task, &f));
        f.owner = Some(UserId::from("bob"));
        assert!(!matches_filter(&task, &f));
    }

    #[test]
    fn matches_filter_status() {
        let task = sample_task("alice", TaskStatus::Queued, None);
        let mut f = TaskFilter::default();
        f.status = Some(TaskStatus::Queued);
        assert!(matches_filter(&task, &f));
        f.status = Some(TaskStatus::Completed {
            completed_at: chrono::Utc::now(),
        });
        assert!(!matches_filter(&task, &f));
    }

    #[test]
    fn matches_filter_tower_some_matches_current() {
        let tw = TowerId::from("t1");
        let task = sample_task(
            "alice",
            TaskStatus::Running {
                started_at: chrono::Utc::now(),
            },
            Some(tw.clone()),
        );
        let mut f = TaskFilter::default();
        f.tower = Some(tw);
        assert!(matches_filter(&task, &f));
    }

    #[test]
    fn matches_filter_tower_rejects_wrong_or_missing() {
        let task_no_tower = sample_task("alice", TaskStatus::Queued, None);
        let mut f = TaskFilter::default();
        f.tower = Some(TowerId::from("t1"));
        assert!(!matches_filter(&task_no_tower, &f));

        let tw = TowerId::from("t-a");
        let task = sample_task(
            "alice",
            TaskStatus::Running {
                started_at: chrono::Utc::now(),
            },
            Some(tw),
        );
        f.tower = Some(TowerId::from("t-b"));
        assert!(!matches_filter(&task, &f));
    }

    #[test]
    fn matches_filter_combined_constraints() {
        let tw = TowerId::from("tower-x");
        let task = sample_task(
            "u1",
            TaskStatus::Running {
                started_at: chrono::Utc::now(),
            },
            Some(tw.clone()),
        );
        let mut f = TaskFilter::default();
        f.owner = Some(UserId::from("u1"));
        f.status = Some(task.status.clone());
        f.tower = Some(tw);
        assert!(matches_filter(&task, &f));

        f.owner = Some(UserId::from("other"));
        assert!(!matches_filter(&task, &f));
    }
}
