// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Consent Management
//!
//! Implements:
//! - Consent requests
//! - User preferences
//! - Auto-approval rules
//! - Enforcement
//!
//! Human-in-the-loop for expensive operations.

use crate::task_lifecycle::{TaskId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

mod enforcement;
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod enforcement_tests;
mod preferences;
mod request;
mod rules;

/// Async consent persistence backend.
///
/// Production path: [`NestGateStorage`](crate::storage_nestgate::NestGateStorage) delegates
/// to the `storage.*` capability provider (NestGate) via JSON-RPC at runtime.
/// Fallback: [`InMemoryStorage`](crate::storage_memory::InMemoryStorage) when no provider is available.
#[async_trait::async_trait]
pub trait ConsentStorageBackend: Send + Sync {
    /// Persist a consent record.
    async fn save(&self, record: &ConsentRecord) -> anyhow::Result<()>;

    /// Retrieve a consent record by ID.
    async fn get(&self, id: &str) -> anyhow::Result<Option<ConsentRecord>>;

    /// List records for a specific user.
    async fn list_by_user(&self, user_id: &UserId) -> anyhow::Result<Vec<ConsentRecord>>;

    /// List records for a specific task.
    async fn list_by_task(&self, task_id: &TaskId) -> anyhow::Result<Vec<ConsentRecord>>;

    /// List all records with pending status.
    async fn list_pending(&self) -> anyhow::Result<Vec<ConsentRecord>>;

    /// Delete a consent record.
    async fn delete(&self, id: &str) -> anyhow::Result<()>;

    /// Flush pending writes to durable storage.
    async fn flush(&self) -> anyhow::Result<()>;
}

pub use enforcement::*;
pub use preferences::*;
pub use request::*;
pub use rules::*;

/// Consent status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsentStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

/// Consent record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub id: Arc<str>,
    pub user_id: UserId,
    pub task_id: TaskId,
    pub operation: Arc<str>,
    pub estimated_cost: Option<f64>,
    pub status: ConsentStatus,
    pub requested_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
    pub reason: Option<Arc<str>>,
}

/// Consent manager
pub struct ConsentManager {
    records: Arc<RwLock<HashMap<Arc<str>, ConsentRecord>>>,
    preferences: Arc<RwLock<HashMap<UserId, UserPreferences>>>,

    /// Optional persistent storage backend (SB-03: trait-abstracted for storage provider migration)
    storage: Option<Arc<dyn ConsentStorageBackend>>,

    /// Notify waiters when a consent decision is made (event-driven)
    decision_notify: Arc<tokio::sync::Notify>,
}

impl ConsentManager {
    /// Create a new consent manager without persistent storage
    #[must_use]
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            preferences: Arc::new(RwLock::new(HashMap::new())),
            storage: None,
            decision_notify: Arc::new(tokio::sync::Notify::new()),
        }
    }

    /// Create a consent manager with resolved storage: NestGate (`storage.*` JSON-RPC on a
    /// capability-discovered Unix socket) when reachable, otherwise
    /// [`InMemoryStorage`](crate::storage_memory::InMemoryStorage).
    pub async fn with_storage(database_url: &str) -> anyhow::Result<Self> {
        #[cfg(unix)]
        {
            if let Ok(ep) = songbird_config::primal_discovery::get_storage_endpoint().await
                && let Some(path) = crate::storage_nestgate::storage_socket_path_from_endpoint(&ep)
            {
                match songbird_universal_ipc::tower_atomic::TowerAtomicClient::connect_unix_path(
                    &path,
                )
                .await
                {
                    Ok(_) => {
                        info!(
                            path = %path.display(),
                            "Consent storage: NestGate JSON-RPC (storage.* capability)"
                        );
                        return Ok(Self::with_backend(Arc::new(
                            crate::storage_nestgate::NestGateStorage::new(path),
                        )));
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            path = %path.display(),
                            "NestGate storage unreachable; using in-memory consent storage"
                        );
                    }
                }
            }
        }

        let _ = database_url;
        Ok(Self::with_backend(Arc::new(crate::storage_memory::InMemoryStorage::new())))
    }

    /// Create a consent manager using an explicit NestGate Unix socket (JSON-RPC `storage.*`).
    #[must_use]
    pub fn with_nestgate(socket_path: PathBuf) -> Self {
        info!(
            path = %socket_path.display(),
            "Consent storage: explicit NestGate socket path"
        );
        Self::with_backend(Arc::new(crate::storage_nestgate::NestGateStorage::new(socket_path)))
    }

    /// Create a consent manager with an arbitrary [`ConsentStorageBackend`].
    ///
    /// SB-03: allows injecting any [`ConsentStorageBackend`] implementation.
    #[must_use]
    pub fn with_backend(backend: Arc<dyn ConsentStorageBackend>) -> Self {
        Self {
            records: Arc::new(RwLock::new(HashMap::new())),
            preferences: Arc::new(RwLock::new(HashMap::new())),
            decision_notify: Arc::new(tokio::sync::Notify::new()),
            storage: Some(backend),
        }
    }

    /// Request consent for an operation
    pub async fn request_consent(
        &self,
        user_id: UserId,
        task_id: TaskId,
        operation: impl Into<Arc<str>>,
        estimated_cost: Option<f64>,
    ) -> Arc<str> {
        let id: Arc<str> = uuid::Uuid::new_v4().to_string().into();
        let operation = operation.into();

        // Check auto-approval rules
        let prefs = self.preferences.read().await;
        if let Some(user_prefs) = prefs.get(&user_id)
            && let Some(auto_approve_threshold) = user_prefs.auto_approve_under_cost
            && let Some(cost) = estimated_cost
            && cost <= auto_approve_threshold
        {
            // Auto-approve
            let record = ConsentRecord {
                id: id.clone(),
                user_id,
                task_id,
                operation,
                estimated_cost,
                status: ConsentStatus::Approved,
                requested_at: Utc::now(),
                responded_at: Some(Utc::now()),
                reason: Some("Auto-approved based on user preferences".into()),
            };
            drop(prefs);

            // Persist to storage if available
            if let Some(ref storage) = self.storage {
                let _ = storage.save(&record).await; // Best effort
            }

            let mut records = self.records.write().await;
            records.insert(id.clone(), record);
            return id;
        }
        drop(prefs);

        // Create pending request
        let record = ConsentRecord {
            id: id.clone(),
            user_id,
            task_id,
            operation,
            estimated_cost,
            status: ConsentStatus::Pending,
            requested_at: Utc::now(),
            responded_at: None,
            reason: None,
        };

        // Persist to storage if available
        if let Some(ref storage) = self.storage {
            let _ = storage.save(&record).await; // Best effort
        }

        let mut records = self.records.write().await;
        records.insert(id.clone(), record);

        id
    }

    /// Approve a consent request
    pub async fn approve(&self, consent_id: &str, reason: Option<Arc<str>>) -> bool {
        let mut records = self.records.write().await;

        if let Some(record) = records.get_mut(consent_id) {
            record.status = ConsentStatus::Approved;
            record.responded_at = Some(Utc::now());
            record.reason = reason.clone();

            // Persist to storage if available
            if let Some(ref storage) = self.storage {
                let _ = storage.save(record).await; // Best effort
            }

            // Wake any waiters (event-driven consent decision)
            self.decision_notify.notify_waiters();

            true
        } else {
            false
        }
    }

    /// Deny a consent request
    pub async fn deny(&self, consent_id: &str, reason: Option<Arc<str>>) -> bool {
        let mut records = self.records.write().await;

        if let Some(record) = records.get_mut(consent_id) {
            record.status = ConsentStatus::Denied;
            record.responded_at = Some(Utc::now());
            record.reason = reason.clone();

            // Persist to storage if available
            if let Some(ref storage) = self.storage {
                let _ = storage.save(record).await; // Best effort
            }

            // Wake any waiters (event-driven consent decision)
            self.decision_notify.notify_waiters();

            true
        } else {
            false
        }
    }

    /// Get consent status
    pub async fn get_status(&self, consent_id: &str) -> Option<ConsentStatus> {
        let records = self.records.read().await;
        records.get(consent_id).map(|r| r.status)
    }

    /// Get consent record by ID
    pub async fn get_consent(&self, consent_id: &str) -> Option<ConsentRecord> {
        let records = self.records.read().await;
        records.get(consent_id).cloned()
    }

    /// List all consent records for a user
    pub async fn list_by_user(&self, user_id: &UserId) -> Vec<ConsentRecord> {
        let records = self.records.read().await;
        records.values().filter(|r| &r.user_id == user_id).cloned().collect()
    }

    /// Set user-specific consent preferences (auto-approval thresholds, blocked operations, etc.)
    pub async fn set_user_preferences(&self, user_id: UserId, preferences: UserPreferences) {
        let mut prefs = self.preferences.write().await;
        prefs.insert(user_id, preferences);
    }

    /// Get user-specific consent preferences
    pub async fn get_user_preferences(&self, user_id: &UserId) -> Option<UserPreferences> {
        let prefs = self.preferences.read().await;
        prefs.get(user_id).cloned()
    }

    /// Wait for consent decision (with timeout)
    ///
    /// Event-driven: uses `tokio::sync::Notify` to wake instantly when
    /// `approve()` or `deny()` is called. Zero polling, zero CPU waste.
    pub async fn wait_for_decision(
        &self,
        consent_id: &str,
        timeout: std::time::Duration,
    ) -> Option<ConsentStatus> {
        // Check immediately
        if let Some(status) = self.get_status(consent_id).await
            && status != ConsentStatus::Pending
        {
            return Some(status);
        }

        // Event-driven wait with timeout
        tokio::time::timeout(timeout, async {
            loop {
                self.decision_notify.notified().await;
                if let Some(status) = self.get_status(consent_id).await
                    && status != ConsentStatus::Pending
                {
                    return status;
                }
            }
        })
        .await
        .ok()
    }
}

impl Default for ConsentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl ConsentManager {
    /// Test-only: set a consent record status directly (e.g. `Expired`) for enforcement unit tests.
    pub(crate) async fn test_set_consent_status(
        &self,
        consent_id: &str,
        status: ConsentStatus,
    ) -> bool {
        let mut records = self.records.write().await;
        if let Some(record) = records.get_mut(consent_id) {
            record.status = status;
            self.decision_notify.notify_waiters();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consent_request() {
        let manager = ConsentManager::new();

        let consent_id = manager
            .request_consent(
                UserId::from("alice"),
                TaskId::new(),
                "expensive_operation",
                Some(100.0),
            )
            .await;

        let status = manager.get_status(&consent_id).await.unwrap();
        assert_eq!(status, ConsentStatus::Pending);
    }

    #[tokio::test]
    async fn test_consent_approval() {
        let manager = ConsentManager::new();

        let consent_id = manager
            .request_consent(
                UserId::from("alice"),
                TaskId::new(),
                "expensive_operation",
                Some(100.0),
            )
            .await;

        manager.approve(&consent_id, Some("User approved".into())).await;

        let status = manager.get_status(&consent_id).await.unwrap();
        assert_eq!(status, ConsentStatus::Approved);
    }

    #[tokio::test]
    async fn test_consent_denial() {
        let manager = ConsentManager::new();

        let consent_id = manager
            .request_consent(
                UserId::from("alice"),
                TaskId::new(),
                "expensive_operation",
                Some(100.0),
            )
            .await;

        manager.deny(&consent_id, Some("Too expensive".into())).await;

        let status = manager.get_status(&consent_id).await.unwrap();
        assert_eq!(status, ConsentStatus::Denied);
    }

    #[tokio::test]
    async fn test_consent_workflow_with_storage() {
        use std::sync::atomic::{AtomicU32, Ordering};
        static CTR: AtomicU32 = AtomicU32::new(0);
        let id = CTR.fetch_add(1, Ordering::SeqCst);
        let db_path = format!("/tmp/songbird-test-consent-{}-{}", std::process::id(), id);
        let manager = ConsentManager::with_storage(&db_path)
            .await
            .expect("Failed to create manager with storage");

        let user_id = UserId::new("alice");
        let task_id = TaskId::new();

        // Request consent
        let consent_id =
            manager.request_consent(user_id.clone(), task_id, "train-model", Some(100.0)).await;

        // Verify it's pending
        let status = manager.get_status(&consent_id).await.unwrap();
        assert_eq!(status, ConsentStatus::Pending);

        // Approve it
        let approved = manager.approve(&consent_id, Some("Approved by user".into())).await;
        assert!(approved);

        // Verify approval
        let status = manager.get_status(&consent_id).await.unwrap();
        assert_eq!(status, ConsentStatus::Approved);

        // Verify we can retrieve the full record
        let record = manager.get_consent(&consent_id).await.unwrap();
        assert_eq!(record.status, ConsentStatus::Approved);
        assert_eq!(record.user_id, user_id);
        assert!(record.reason.is_some());
    }

    #[tokio::test]
    async fn test_list_by_user_with_storage() {
        use std::sync::atomic::{AtomicU32, Ordering};
        static CTR: AtomicU32 = AtomicU32::new(0);
        let id = CTR.fetch_add(1, Ordering::SeqCst);
        let db_path = format!("/tmp/songbird-test-consent-list-{}-{}", std::process::id(), id);
        let manager = ConsentManager::with_storage(&db_path)
            .await
            .expect("Failed to create manager with storage");

        let user_id = UserId::new("bob");

        // Create multiple consent requests for same user
        for i in 0..3 {
            manager
                .request_consent(
                    user_id.clone(),
                    TaskId::new(),
                    format!("operation-{i}"),
                    Some(50.0 * f64::from(i + 1)),
                )
                .await;
        }

        // List all consents for user
        let records = manager.list_by_user(&user_id).await;
        assert_eq!(records.len(), 3);

        // Verify all belong to the user
        for record in &records {
            assert_eq!(record.user_id, user_id);
            assert_eq!(record.status, ConsentStatus::Pending);
        }
    }

    #[tokio::test]
    async fn test_get_consent_method() {
        let manager = ConsentManager::new();

        let consent_id = manager
            .request_consent(UserId::new("charlie"), TaskId::new(), "analyze-data", Some(75.0))
            .await;

        // Get full consent record
        let record = manager.get_consent(&consent_id).await.unwrap();
        assert_eq!(record.id.as_ref(), consent_id.as_ref());
        assert_eq!(record.user_id.as_str(), "charlie");
        assert_eq!(record.operation.as_ref(), "analyze-data");
        assert_eq!(record.estimated_cost, Some(75.0));
        assert_eq!(record.status, ConsentStatus::Pending);

        // Non-existent consent should return None
        let missing = manager.get_consent("non-existent-id").await;
        assert!(missing.is_none());
    }
}
