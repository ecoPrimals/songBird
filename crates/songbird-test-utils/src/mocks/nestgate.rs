//! Mock `NestGate` Storage Primal
//!
//! Provides HTTP endpoints that simulate `NestGate`'s storage and data management capabilities.

#![allow(clippy::unused_async)]

use super::common::{HealthStatus, MockPrimalServer, MockServerState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Storage metrics from `NestGate`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetrics {
    /// Total storage capacity in bytes
    pub total_capacity_bytes: u64,
    /// Used storage in bytes
    pub used_bytes: u64,
    /// Available storage in bytes
    pub available_bytes: u64,
    /// Number of stored objects
    pub object_count: u64,
    /// Average read latency in milliseconds
    pub avg_read_latency_ms: f64,
    /// Average write latency in milliseconds
    pub avg_write_latency_ms: f64,
}

impl Default for StorageMetrics {
    fn default() -> Self {
        Self {
            total_capacity_bytes: 1_000_000_000_000, // 1TB
            used_bytes: 250_000_000_000,             // 250GB
            available_bytes: 750_000_000_000,        // 750GB
            object_count: 1_500,
            avg_read_latency_ms: 15.0,
            avg_write_latency_ms: 25.0,
        }
    }
}

/// Mock `NestGate` storage server
#[derive(Debug, Clone)]
pub struct MockNestGate {
    state: Arc<MockServerState>,
    storage_metrics: Arc<RwLock<StorageMetrics>>,
    stored_objects: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MockNestGate {
    /// Create a new mock `NestGate` server
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(MockServerState::new(0)),
            storage_metrics: Arc::new(RwLock::new(StorageMetrics::default())),
            stored_objects: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the mock server
    ///
    /// # Errors
    ///
    /// Currently never returns an error, but signature allows for future error cases.
    pub async fn start(&mut self) -> Result<u16, Box<dyn std::error::Error>> {
        let port = fastrand::u16(10000..60000);
        self.state = Arc::new(MockServerState::new(port));
        Ok(port)
    }

    /// Stop the mock server
    pub async fn stop(&self) {
        // Server cleanup
    }

    /// Store an object (simulated)
    ///
    /// # Panics
    ///
    /// Panics if the internal locks are poisoned.
    pub fn store_object(&self, key: impl Into<String>, data: Vec<u8>) {
        let key = key.into();
        let size = data.len() as u64;

        let mut objects = self.stored_objects.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        objects.insert(key, data);

        let mut metrics = self.storage_metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.used_bytes += size;
        metrics.available_bytes = metrics.available_bytes.saturating_sub(size);
        metrics.object_count += 1;

        self.state.increment_requests();
    }

    /// Retrieve an object (simulated)
    ///
    /// # Panics
    ///
    /// Panics if the internal stored objects lock is poisoned.
    #[must_use]
    pub fn retrieve_object(&self, key: &str) -> Option<Vec<u8>> {
        self.state.increment_requests();
        let objects = self.stored_objects.read().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        objects.get(key).cloned()
    }

    /// Set storage usage percentage
    ///
    /// # Panics
    ///
    /// Panics if the internal storage metrics lock is poisoned.
    pub fn set_storage_usage(&self, percent: f64) {
        let mut metrics = self.storage_metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let used = (metrics.total_capacity_bytes as f64 * (percent / 100.0)) as u64;
        metrics.used_bytes = used;
        metrics.available_bytes = metrics.total_capacity_bytes - used;
    }

    /// Get storage metrics
    ///
    /// # Panics
    ///
    /// Panics if the internal storage metrics lock is poisoned.
    #[must_use]
    pub fn get_metrics(&self) -> StorageMetrics {
        self.storage_metrics
            .read()
            .unwrap_or_else(|poisoned| {
                tracing::warn!("RwLock poisoned in test mock, recovering");
                poisoned.into_inner()
            })
            .clone()
    }

    /// Simulate near-capacity scenario
    ///
    /// # Panics
    ///
    /// Panics if the internal storage metrics lock is poisoned.
    pub fn simulate_near_capacity(&self) {
        let mut metrics = self.storage_metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.used_bytes = 950_000_000_000; // 950GB
        metrics.available_bytes = 50_000_000_000; // 50GB
        metrics.avg_write_latency_ms = 150.0;
        self.state.set_health(HealthStatus::Degraded);
    }

    /// Simulate healthy storage
    ///
    /// # Panics
    ///
    /// Panics if the internal storage metrics lock is poisoned.
    pub fn simulate_healthy_storage(&self) {
        let mut metrics = self.storage_metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.used_bytes = 250_000_000_000; // 250GB
        metrics.available_bytes = 750_000_000_000; // 750GB
        metrics.avg_read_latency_ms = 15.0;
        metrics.avg_write_latency_ms = 25.0;
        self.state.set_health(HealthStatus::Healthy);
    }
}

impl Default for MockNestGate {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPrimalServer for MockNestGate {
    fn port(&self) -> u16 {
        self.state.port
    }

    fn set_health(&self, status: HealthStatus) {
        self.state.set_health(status);
    }

    fn get_health(&self) -> HealthStatus {
        self.state.get_health()
    }
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default
)]
#[cfg(test)]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::float_cmp)]
#[allow(clippy::useless_vec)]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::items_after_statements)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
mod tests {
    #![allow(clippy::all)]
    #![allow(unused)]

    use super::*;
    use songbird_types::SongbirdError;

    #[tokio::test]
    async fn test_mock_nestgate_storage() {
        let mock = MockNestGate::new();

        // Store and retrieve object
        let data = vec![1, 2, 3, 4, 5];
        mock.store_object("test_key", data.clone());

        let retrieved = mock.retrieve_object("test_key");
        assert_eq!(retrieved, Some(data));

        // Non-existent key
        let missing = mock.retrieve_object("nonexistent");
        assert_eq!(missing, None);
    }

    #[tokio::test]
    async fn test_mock_nestgate_metrics() {
        let mock = MockNestGate::new();

        let initial_metrics = mock.get_metrics();
        let initial_count = initial_metrics.object_count;

        mock.store_object("key1", vec![0; 1000]);
        mock.store_object("key2", vec![0; 2000]);

        let metrics = mock.get_metrics();
        assert_eq!(metrics.object_count, initial_count + 2);
        assert!(metrics.used_bytes >= initial_metrics.used_bytes + 3000);
    }

    #[tokio::test]
    async fn test_mock_nestgate_scenarios() {
        let mock = MockNestGate::new();

        // Test near capacity
        mock.simulate_near_capacity();
        let metrics = mock.get_metrics();
        assert!(metrics.available_bytes < 100_000_000_000); // Less than 100GB
        assert_eq!(mock.get_health(), HealthStatus::Degraded);

        // Test healthy
        mock.simulate_healthy_storage();
        let metrics = mock.get_metrics();
        assert!(metrics.available_bytes > 500_000_000_000); // More than 500GB
        assert_eq!(mock.get_health(), HealthStatus::Healthy);
    }

    // ========== NEW TESTS (5 tests to improve coverage) ==========

    #[tokio::test]
    async fn test_nestgate_server_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock = MockNestGate::new();
        let port = mock
            .start()
            .await
            .map_err(|e| SongbirdError::configuration(format!("Server should start: {}", e)))?;
        assert!(port > 0);
        assert_eq!(mock.port(), port);
        mock.stop().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_storage_metrics_default() {
        let mock = MockNestGate::new();
        let metrics = mock.get_metrics();
        assert_eq!(metrics.total_capacity_bytes, 1_000_000_000_000);
        assert_eq!(metrics.used_bytes, 250_000_000_000);
        assert_eq!(metrics.available_bytes, 750_000_000_000);
        assert_eq!(metrics.object_count, 1_500);
    }

    #[tokio::test]
    async fn test_object_retrieval() -> Result<(), Box<dyn std::error::Error>> {
        let mock = MockNestGate::new();
        let data = vec![1, 2, 3, 4, 5];
        mock.store_object("test_key", data.clone());

        let retrieved = mock.retrieve_object("test_key");
        assert!(retrieved.is_some());
        assert_eq!(
            retrieved.ok_or_else(|| SongbirdError::configuration(
                "Missing performance configuration".to_string()
            ))?,
            data
        );

        let missing = mock.retrieve_object("nonexistent");
        assert!(missing.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_storage_usage_percentage() {
        let mock = MockNestGate::new();
        mock.set_storage_usage(75.0);

        let metrics = mock.get_metrics();
        let usage_percent =
            (metrics.used_bytes as f64 / metrics.total_capacity_bytes as f64) * 100.0;
        assert!((usage_percent - 75.0).abs() < 1.0);
    }

    #[test]
    fn test_nestgate_default_trait() {
        let mock = MockNestGate::default();
        assert_eq!(mock.port(), 0);
        assert_eq!(mock.get_health(), HealthStatus::Healthy);
    }
}
