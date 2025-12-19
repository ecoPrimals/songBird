//! Consent Storage - SQLite persistence for consent records
//!
//! This module provides durable storage for consent records using SQLite.
//! Part of MVP Week 5: Consent Management
//!
//! Features:
//! - SQLite-based persistence
//! - Automatic schema migrations
//! - Efficient queries by user and task
//! - Type-safe serialization

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use std::sync::Arc;
use tracing::{debug, info};

use crate::consent_management::{ConsentRecord, ConsentStatus};
use crate::task_lifecycle::{TaskId, UserId};

/// Consent storage backed by SQLite
pub struct ConsentStorage {
    pool: SqlitePool,
}

impl ConsentStorage {
    /// Create a new consent storage with the given database URL
    ///
    /// Automatically runs migrations to create the consent_records table
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Initializing consent storage: {}", database_url);

        let pool = SqlitePool::connect(database_url)
            .await
            .context("Failed to connect to consent database")?;

        let storage = Self {
            pool,
        };
        storage.run_migrations().await?;

        info!("✅ Consent storage initialized");
        Ok(storage)
    }

    /// Run database migrations
    async fn run_migrations(&self) -> Result<()> {
        debug!("Running consent storage migrations...");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS consent_records (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                task_id TEXT NOT NULL,
                operation TEXT NOT NULL,
                estimated_cost REAL,
                status TEXT NOT NULL,
                requested_at INTEGER NOT NULL,
                responded_at INTEGER,
                reason TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create consent_records table")?;

        // Create indices for efficient queries
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_consent_user_id 
            ON consent_records(user_id)
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create user_id index")?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_consent_task_id 
            ON consent_records(task_id)
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create task_id index")?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_consent_status 
            ON consent_records(status)
            "#,
        )
        .execute(&self.pool)
        .await
        .context("Failed to create status index")?;

        debug!("✅ Consent storage migrations complete");
        Ok(())
    }

    /// Save a consent record
    pub async fn save(&self, record: &ConsentRecord) -> Result<()> {
        debug!("Saving consent record: {}", record.id);

        let status_str = format!("{:?}", record.status);
        let requested_at = record.requested_at.timestamp();
        let responded_at = record.responded_at.as_ref().map(|dt| dt.timestamp());

        sqlx::query(
            r#"
            INSERT INTO consent_records 
            (id, user_id, task_id, operation, estimated_cost, status, requested_at, responded_at, reason)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                responded_at = excluded.responded_at,
                reason = excluded.reason
            "#,
        )
        .bind(record.id.as_ref())
        .bind(record.user_id.as_str())
        .bind(record.task_id.to_string())
        .bind(record.operation.as_ref())
        .bind(record.estimated_cost)
        .bind(&status_str)
        .bind(requested_at)
        .bind(responded_at)
        .bind(record.reason.as_ref().map(|r| r.as_ref()))
        .execute(&self.pool)
        .await
        .context("Failed to save consent record")?;

        debug!("✅ Consent record saved: {}", record.id);
        Ok(())
    }

    /// Get a consent record by ID
    pub async fn get(&self, id: &str) -> Result<Option<ConsentRecord>> {
        debug!("Getting consent record: {}", id);

        let row = sqlx::query("SELECT * FROM consent_records WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .context("Failed to query consent record")?;

        match row {
            Some(row) => {
                let record = self.row_to_record(row)?;
                debug!("✅ Found consent record: {}", id);
                Ok(Some(record))
            }
            None => {
                debug!("Consent record not found: {}", id);
                Ok(None)
            }
        }
    }

    /// List all consent records for a user
    pub async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<ConsentRecord>> {
        debug!("Listing consent records for user: {}", user_id);

        let rows = sqlx::query(
            "SELECT * FROM consent_records WHERE user_id = ? ORDER BY requested_at DESC",
        )
        .bind(user_id.as_str())
        .fetch_all(&self.pool)
        .await
        .context("Failed to query consent records by user")?;

        let records: Result<Vec<_>> = rows.into_iter().map(|row| self.row_to_record(row)).collect();
        let records = records?;

        debug!("✅ Found {} consent records for user: {}", records.len(), user_id);
        Ok(records)
    }

    /// List all consent records for a task
    pub async fn list_by_task(&self, task_id: &TaskId) -> Result<Vec<ConsentRecord>> {
        debug!("Listing consent records for task: {}", task_id);

        let rows = sqlx::query(
            "SELECT * FROM consent_records WHERE task_id = ? ORDER BY requested_at DESC",
        )
        .bind(task_id.to_string())
        .fetch_all(&self.pool)
        .await
        .context("Failed to query consent records by task")?;

        let records: Result<Vec<_>> = rows.into_iter().map(|row| self.row_to_record(row)).collect();
        let records = records?;

        debug!("✅ Found {} consent records for task: {}", records.len(), task_id);
        Ok(records)
    }

    /// List all consent records with a specific status
    pub async fn list_by_status(&self, status: ConsentStatus) -> Result<Vec<ConsentRecord>> {
        debug!("Listing consent records with status: {:?}", status);

        let status_str = format!("{:?}", status);
        let rows = sqlx::query(
            "SELECT * FROM consent_records WHERE status = ? ORDER BY requested_at DESC",
        )
        .bind(&status_str)
        .fetch_all(&self.pool)
        .await
        .context("Failed to query consent records by status")?;

        let records: Result<Vec<_>> = rows.into_iter().map(|row| self.row_to_record(row)).collect();
        let records = records?;

        debug!("✅ Found {} consent records with status {:?}", records.len(), status);
        Ok(records)
    }

    /// Delete a consent record
    pub async fn delete(&self, id: &str) -> Result<bool> {
        debug!("Deleting consent record: {}", id);

        let result = sqlx::query("DELETE FROM consent_records WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete consent record")?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            debug!("✅ Consent record deleted: {}", id);
        } else {
            debug!("Consent record not found for deletion: {}", id);
        }

        Ok(deleted)
    }

    /// Convert a SQLite row to a ConsentRecord
    fn row_to_record(&self, row: sqlx::sqlite::SqliteRow) -> Result<ConsentRecord> {
        let id: String = row.try_get("id")?;
        let user_id: String = row.try_get("user_id")?;
        let task_id: String = row.try_get("task_id")?;
        let operation: String = row.try_get("operation")?;
        let estimated_cost: Option<f64> = row.try_get("estimated_cost")?;
        let status_str: String = row.try_get("status")?;
        let requested_at_timestamp: i64 = row.try_get("requested_at")?;
        let responded_at_timestamp: Option<i64> = row.try_get("responded_at")?;
        let reason: Option<String> = row.try_get("reason")?;

        // Parse status from string
        let status = match status_str.as_str() {
            "Pending" => ConsentStatus::Pending,
            "Approved" => ConsentStatus::Approved,
            "Denied" => ConsentStatus::Denied,
            "Expired" => ConsentStatus::Expired,
            _ => ConsentStatus::Pending, // Default to pending for unknown statuses
        };

        // Convert timestamps to DateTime<Utc>
        let requested_at =
            DateTime::from_timestamp(requested_at_timestamp, 0).unwrap_or_else(Utc::now);
        let responded_at = responded_at_timestamp.and_then(|ts| DateTime::from_timestamp(ts, 0));

        // Parse TaskId from string
        let task_id = task_id.parse::<TaskId>().context("Failed to parse task_id as UUID")?;

        Ok(ConsentRecord {
            id: id.into(),
            user_id: UserId::new(user_id),
            task_id,
            operation: operation.into(),
            estimated_cost,
            status,
            requested_at,
            responded_at,
            reason: reason.map(Arc::from),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    async fn create_test_storage() -> Result<ConsentStorage> {
        ConsentStorage::new("sqlite::memory:").await
    }

    #[tokio::test]
    async fn test_save_and_get() -> Result<()> {
        let storage = create_test_storage().await?;

        let record = ConsentRecord {
            id: "test-consent-1".into(),
            user_id: UserId::new("user-1"),
            task_id: TaskId::new(),
            operation: "train-model".into(),
            estimated_cost: Some(100.0),
            status: ConsentStatus::Pending,
            requested_at: Utc::now(),
            responded_at: None,
            reason: None,
        };

        storage.save(&record).await?;

        let retrieved = storage.get("test-consent-1").await?;
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id.as_ref(), "test-consent-1");
        assert_eq!(retrieved.user_id.as_str(), "user-1");
        assert_eq!(retrieved.status, ConsentStatus::Pending);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_status() -> Result<()> {
        let storage = create_test_storage().await?;

        let mut record = ConsentRecord {
            id: "test-consent-2".into(),
            user_id: UserId::new("user-2"),
            task_id: TaskId::new(),
            operation: "train-model".into(),
            estimated_cost: Some(200.0),
            status: ConsentStatus::Pending,
            requested_at: Utc::now(),
            responded_at: None,
            reason: None,
        };

        storage.save(&record).await?;

        // Update status
        record.status = ConsentStatus::Approved;
        record.responded_at = Some(Utc::now());
        storage.save(&record).await?;

        let retrieved = storage.get("test-consent-2").await?;
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.status, ConsentStatus::Approved);
        assert!(retrieved.responded_at.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_list_by_user() -> Result<()> {
        let storage = create_test_storage().await?;

        // Create multiple records for same user
        for i in 1..=3 {
            let record = ConsentRecord {
                id: format!("test-consent-{}", i).into(),
                user_id: UserId::new("user-3"),
                task_id: TaskId::new(),
                operation: "train-model".into(),
                estimated_cost: Some(i as f64 * 100.0),
                status: ConsentStatus::Pending,
                requested_at: Utc::now(),
                responded_at: None,
                reason: None,
            };
            storage.save(&record).await?;
        }

        let records = storage.list_by_user(&UserId::new("user-3")).await?;
        assert_eq!(records.len(), 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_list_by_status() -> Result<()> {
        let storage = create_test_storage().await?;

        // Create records with different statuses
        let statuses = [ConsentStatus::Pending, ConsentStatus::Approved, ConsentStatus::Denied];
        for (i, status) in statuses.iter().enumerate() {
            let record = ConsentRecord {
                id: format!("test-consent-status-{}", i).into(),
                user_id: UserId::new(format!("user-{}", i)),
                task_id: TaskId::new(),
                operation: "train-model".into(),
                estimated_cost: Some(100.0),
                status: *status,
                requested_at: Utc::now(),
                responded_at: if *status != ConsentStatus::Pending {
                    Some(Utc::now())
                } else {
                    None
                },
                reason: None,
            };
            storage.save(&record).await?;
        }

        let pending = storage.list_by_status(ConsentStatus::Pending).await?;
        assert_eq!(pending.len(), 1);

        let approved = storage.list_by_status(ConsentStatus::Approved).await?;
        assert_eq!(approved.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete() -> Result<()> {
        let storage = create_test_storage().await?;

        let record = ConsentRecord {
            id: "test-consent-delete".into(),
            user_id: UserId::new("user-delete"),
            task_id: TaskId::new(),
            operation: "train-model".into(),
            estimated_cost: Some(100.0),
            status: ConsentStatus::Pending,
            requested_at: Utc::now(),
            responded_at: None,
            reason: None,
        };

        storage.save(&record).await?;

        let deleted = storage.delete("test-consent-delete").await?;
        assert!(deleted);

        let retrieved = storage.get("test-consent-delete").await?;
        assert!(retrieved.is_none());

        Ok(())
    }
}
