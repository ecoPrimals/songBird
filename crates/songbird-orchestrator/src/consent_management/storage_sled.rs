// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Consent Storage - Pure Rust sled persistence
//!
//! **Evolution**: Migrated from sqlx to sled (Jan 27, 2026)
//! - ✅ 100% Pure Rust (TRUE ecoBin!)
//! - ✅ Embedded key-value store (no external daemon)
//! - ✅ ACID transactions
//! - ✅ Zero-copy reads
//! - ✅ Crash-safe
//! - ✅ Simpler API than SQL
//!
//! This module provides durable storage for consent records using sled.
//! Part of MVP Week 5: Consent Management
//!
//! Features:
//! - Pure Rust persistence (no C dependencies!)
//! - Automatic durability (crash-safe)
//! - Efficient queries by user and task
//! - Type-safe serialization with bincode

use anyhow::{Context, Result};
use sled::Db;
use std::sync::Arc;
use tracing::{debug, info};

use crate::consent_management::{ConsentRecord, ConsentStatus};
use crate::task_lifecycle::{TaskId, UserId};

/// Consent storage backed by sled (100% Pure Rust!)
///
/// Thread-safe and async-compatible via `Arc<Db>` and `spawn_blocking`
#[derive(Clone)]
pub struct ConsentStorage {
    db: Arc<Db>,
}

impl ConsentStorage {
    /// Create a new consent storage with the given database path
    ///
    /// Automatically opens/creates the sled database
    ///
    /// # Errors
    ///
    /// Returns error if database cannot be opened
    pub async fn new(database_path: &str) -> Result<Self> {
        info!("Initializing consent storage (sled): {}", database_path);

        let path = database_path.to_string();
        let db = tokio::task::spawn_blocking(move || {
            sled::open(&path).context("Failed to open sled database")
        })
        .await
        .context("Task panicked while opening database")??;

        info!("✅ Consent storage initialized (100% Pure Rust!)");
        Ok(Self {
            db: Arc::new(db),
        })
    }

    /// Save a consent record
    ///
    /// # Errors
    ///
    /// Returns error if serialization or database write fails
    pub async fn save(&self, record: &ConsentRecord) -> Result<()> {
        let record = record.clone();
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            debug!("Saving consent record: {}", record.id);

            // Serialize record
            let value =
                bincode::serialize(&record).context("Failed to serialize consent record")?;

            // Store with composite key: consent/{id}
            let key = format!("consent/{}", record.id);
            db.insert(key.as_bytes(), value).context("Failed to save consent record")?;

            // Create indices for efficient queries
            // Index by user: user_consents/{user_id}/{consent_id}
            let user_index_key = format!("user_consents/{}/{}", record.user_id.as_str(), record.id);
            db.insert(user_index_key.as_bytes(), record.id.as_bytes())
                .context("Failed to create user index")?;

            // Index by task: task_consents/{task_id}/{consent_id}
            let task_index_key = format!("task_consents/{}/{}", record.task_id, record.id);
            db.insert(task_index_key.as_bytes(), record.id.as_bytes())
                .context("Failed to create task index")?;

            debug!("✅ Consent record saved: {}", record.id);
            Ok(())
        })
        .await
        .context("Task panicked while saving")?
    }

    /// Get a consent record by ID
    ///
    /// # Errors
    ///
    /// Returns error if deserialization fails
    pub async fn get(&self, id: &str) -> Result<Option<ConsentRecord>> {
        let id = id.to_string();
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            debug!("Getting consent record: {}", id);

            let key = format!("consent/{id}");
            let value = db.get(key.as_bytes()).context("Failed to query consent record")?;

            if let Some(bytes) = value {
                let record: ConsentRecord =
                    bincode::deserialize(&bytes).context("Failed to deserialize consent record")?;
                debug!("✅ Found consent record: {}", id);
                Ok(Some(record))
            } else {
                debug!("Consent record not found: {}", id);
                Ok(None)
            }
        })
        .await
        .context("Task panicked while getting")?
    }

    /// List all consent records for a user
    ///
    /// # Errors
    ///
    /// Returns error if query or deserialization fails
    pub async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<ConsentRecord>> {
        let user_id = user_id.clone();
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            debug!("Listing consent records for user: {}", user_id.as_str());

            let prefix = format!("user_consents/{}/", user_id.as_str());
            let mut records = Vec::new();

            // Scan index
            for item in db.scan_prefix(prefix.as_bytes()) {
                let (_key, consent_id_bytes) = item.context("Failed to scan user consents")?;
                let consent_id = String::from_utf8(consent_id_bytes.to_vec())
                    .context("Invalid consent ID in index")?;

                // Fetch the actual record directly
                let key = format!("consent/{consent_id}");
                if let Some(bytes) = db.get(key.as_bytes())? {
                    let record: ConsentRecord = bincode::deserialize(&bytes)?;
                    records.push(record);
                }
            }

            debug!("✅ Found {} consent records for user {}", records.len(), user_id.as_str());
            Ok(records)
        })
        .await
        .context("Task panicked while listing by user")?
    }

    /// List all consent records for a task
    ///
    /// # Errors
    ///
    /// Returns error if query or deserialization fails
    pub async fn list_by_task(&self, task_id: &TaskId) -> Result<Vec<ConsentRecord>> {
        let task_id = *task_id;
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            debug!("Listing consent records for task: {}", task_id.to_string());

            let prefix = format!("task_consents/{task_id}/");
            let mut records = Vec::new();

            // Scan index
            for item in db.scan_prefix(prefix.as_bytes()) {
                let (_key, consent_id_bytes) = item.context("Failed to scan task consents")?;
                let consent_id = String::from_utf8(consent_id_bytes.to_vec())
                    .context("Invalid consent ID in index")?;

                // Fetch the actual record directly
                let key = format!("consent/{consent_id}");
                if let Some(bytes) = db.get(key.as_bytes())? {
                    let record: ConsentRecord = bincode::deserialize(&bytes)?;
                    records.push(record);
                }
            }

            debug!("✅ Found {} consent records for task {}", records.len(), task_id.to_string());
            Ok(records)
        })
        .await
        .context("Task panicked while listing by task")?
    }

    /// List all pending consent requests
    ///
    /// # Errors
    ///
    /// Returns error if scan or deserialization fails
    pub async fn list_pending(&self) -> Result<Vec<ConsentRecord>> {
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            debug!("Listing pending consent records");

            let mut records = Vec::new();

            // Scan all consent records
            for item in db.scan_prefix(b"consent/") {
                let (_key, value) = item.context("Failed to scan consents")?;
                let record: ConsentRecord =
                    bincode::deserialize(&value).context("Failed to deserialize consent record")?;

                if matches!(record.status, ConsentStatus::Pending) {
                    records.push(record);
                }
            }

            debug!("✅ Found {} pending consent records", records.len());
            Ok(records)
        })
        .await
        .context("Task panicked while listing pending")?
    }

    /// Delete a consent record
    ///
    /// # Errors
    ///
    /// Returns error if deletion fails
    pub async fn delete(&self, id: &str) -> Result<()> {
        let id = id.to_string();
        let db = Arc::clone(&self.db);

        tokio::task::spawn_blocking(move || {
            debug!("Deleting consent record: {}", id);

            // Get record first to clean up indices
            let key = format!("consent/{id}");
            if let Some(bytes) = db.get(key.as_bytes())? {
                let record: ConsentRecord = bincode::deserialize(&bytes)?;

                // Delete main record
                db.remove(key.as_bytes()).context("Failed to delete consent record")?;

                // Delete user index
                let user_index_key = format!("user_consents/{}/{}", record.user_id.as_str(), id);
                db.remove(user_index_key.as_bytes()).context("Failed to delete user index")?;

                // Delete task index
                let task_index_key = format!("task_consents/{}/{}", record.task_id, id);
                db.remove(task_index_key.as_bytes()).context("Failed to delete task index")?;

                debug!("✅ Consent record deleted: {}", id);
            } else {
                debug!("Consent record not found for deletion: {}", id);
            }

            Ok(())
        })
        .await
        .context("Task panicked while deleting")?
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
}

#[async_trait::async_trait]
impl super::ConsentStorageBackend for ConsentStorage {
    async fn save(&self, record: &ConsentRecord) -> Result<()> {
        Self::save(self, record).await
    }

    async fn get(&self, id: &str) -> Result<Option<ConsentRecord>> {
        Self::get(self, id).await
    }

    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<ConsentRecord>> {
        Self::list_by_user(self, user_id).await
    }

    async fn list_by_task(&self, task_id: &TaskId) -> Result<Vec<ConsentRecord>> {
        Self::list_by_task(self, task_id).await
    }

    async fn list_pending(&self) -> Result<Vec<ConsentRecord>> {
        Self::list_pending(self).await
    }

    async fn delete(&self, id: &str) -> Result<()> {
        Self::delete(self, id).await
    }

    async fn flush(&self) -> Result<()> {
        Self::flush(self).await
    }
}

#[cfg(all(test, feature = "sled-storage"))]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::TempDir;

    fn create_test_record() -> ConsentRecord {
        ConsentRecord {
            id: "test-consent-1".to_string().into(),
            user_id: UserId::new("test-user"),
            task_id: TaskId::new(),
            operation: "test-operation".to_string().into(),
            estimated_cost: Some(100.0),
            status: ConsentStatus::Pending,
            requested_at: Utc::now(),
            responded_at: None,
            reason: None,
        }
    }

    #[tokio::test]
    async fn test_consent_storage_new() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("consents.db");

        let storage = ConsentStorage::new(db_path.to_str().unwrap()).await;
        assert!(storage.is_ok());
    }

    #[tokio::test]
    async fn test_save_and_get() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("consents.db");
        let storage = ConsentStorage::new(db_path.to_str().unwrap()).await.unwrap();

        let record = create_test_record();
        storage.save(&record).await.unwrap();

        let retrieved = storage.get(record.id.as_ref()).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_record = retrieved.unwrap();
        assert_eq!(retrieved_record.id, record.id);
        assert_eq!(retrieved_record.user_id, record.user_id);
    }

    #[tokio::test]
    async fn test_list_by_user() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("consents.db");
        let storage = ConsentStorage::new(db_path.to_str().unwrap()).await.unwrap();

        let user_id = UserId::new("test-user");

        // Create multiple records for same user
        for i in 0..3 {
            let mut record = create_test_record();
            record.id = format!("consent-{i}").into();
            record.user_id = user_id.clone();
            storage.save(&record).await.unwrap();
        }

        let records = storage.list_by_user(&user_id).await.unwrap();
        assert_eq!(records.len(), 3);
    }

    #[tokio::test]
    async fn test_list_by_task() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("consents.db");
        let storage = ConsentStorage::new(db_path.to_str().unwrap()).await.unwrap();

        let task_id = TaskId::new();

        // Create multiple records for same task
        for i in 0..3 {
            let mut record = create_test_record();
            record.id = format!("consent-{i}").into();
            record.task_id = task_id;
            storage.save(&record).await.unwrap();
        }

        let records = storage.list_by_task(&task_id).await.unwrap();
        assert_eq!(records.len(), 3);
    }

    #[tokio::test]
    async fn test_list_pending() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("consents.db");
        let storage = ConsentStorage::new(db_path.to_str().unwrap()).await.unwrap();

        // Create pending records
        for i in 0..2 {
            let mut record = create_test_record();
            record.id = format!("pending-{i}").into();
            record.status = ConsentStatus::Pending;
            storage.save(&record).await.unwrap();
        }

        // Create approved record
        let mut approved = create_test_record();
        approved.id = "approved-1".to_string().into();
        approved.status = ConsentStatus::Approved;
        storage.save(&approved).await.unwrap();

        let pending = storage.list_pending().await.unwrap();
        assert_eq!(pending.len(), 2);
    }

    #[tokio::test]
    async fn test_delete() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("consents.db");
        let storage = ConsentStorage::new(db_path.to_str().unwrap()).await.unwrap();

        let record = create_test_record();
        storage.save(&record).await.unwrap();

        // Verify it exists
        assert!(storage.get(record.id.as_ref()).await.unwrap().is_some());

        // Delete it
        storage.delete(record.id.as_ref()).await.unwrap();

        // Verify it's gone
        assert!(storage.get(record.id.as_ref()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_flush() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("consents.db");
        let storage = ConsentStorage::new(db_path.to_str().unwrap()).await.unwrap();

        let record = create_test_record();
        storage.save(&record).await.unwrap();

        // Should not panic
        storage.flush().await.unwrap();
    }
}
