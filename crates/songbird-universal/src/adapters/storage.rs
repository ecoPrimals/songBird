//! Storage Capability Adapter
//!
//! **SOVEREIGNTY**: This adapter works with ANY storage capability provider.
//! It does NOT know about specific primals (`NestGate` is just one example).
//! Discovery is capability-based through environment hints or zero-knowledge bootstrap.
//!
//! Songbird has local storage for sovereign standalone operation, but can utilize
//! any discovered storage provider for network effects.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::time::Duration;
use tracing::{debug, warn};

/// Storage metrics from any storage capability provider
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
    /// Timestamp of metrics collection
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl StorageMetrics {
    /// Calculate storage usage percentage
    #[must_use]
    pub fn usage_percent(&self) -> f64 {
        if self.total_capacity_bytes == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        {
            (self.used_bytes as f64 / self.total_capacity_bytes as f64) * 100.0
        }
    }

    /// Check if storage is nearly full
    #[must_use]
    pub fn is_nearly_full(&self) -> bool {
        self.usage_percent() > 90.0
    }

    /// Check if latency is high
    #[must_use]
    pub fn is_high_latency(&self) -> bool {
        self.avg_read_latency_ms > 100.0 || self.avg_write_latency_ms > 200.0
    }

    /// Get storage health status
    #[must_use]
    pub fn health_status(&self) -> StorageHealth {
        let usage = self.usage_percent();

        if usage > 95.0 || self.avg_write_latency_ms > 500.0 {
            StorageHealth::Critical
        } else if usage > 85.0 || self.is_high_latency() {
            StorageHealth::Warning
        } else {
            StorageHealth::Healthy
        }
    }
}

/// Storage health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageHealth {
    /// Storage is healthy
    Healthy,
    /// Storage warnings present
    Warning,
    /// Storage is critical
    Critical,
}

/// **CAPABILITY-BASED STORAGE ADAPTER**
///
/// Works with ANY storage provider discovered through:
/// - Environment variable: `SONGBIRD_STORAGE_ENDPOINT`
/// - Capability discovery: `capability:storage`
/// - Zero-knowledge bootstrap
/// - Local filesystem fallback for sovereign operation
pub struct StorageAdapter {
    /// Endpoint URL for the storage capability provider
    endpoint: String,
    /// HTTP client for requests
    client: reqwest::Client,
    /// Request timeout
    timeout: Duration,
}

impl StorageAdapter {
    /// Create adapter from discovered storage capability
    ///
    /// Uses capability-based discovery:
    /// 1. Check `SONGBIRD_STORAGE_ENDPOINT` environment variable
    /// 2. Fall back to capability discovery
    /// 3. No hardcoded primal names anywhere
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # tokio_test::block_on(async {
    /// use songbird_universal::adapters::StorageAdapter;
    ///
    /// // Discovers any storage provider (could be NestGate, or local filesystem, or anyone)
    /// let adapter = StorageAdapter::from_discovery().await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if no storage capability can be discovered.
    #[allow(clippy::unused_async)] // Will be async when ZeroKnowledgeBootstrap integration is complete
    pub async fn from_discovery() -> SongbirdResult<Self> {
        // Try environment variable first
        if let Ok(endpoint) = std::env::var("SONGBIRD_STORAGE_ENDPOINT") {
            debug!("🔍 Storage capability discovered via SONGBIRD_STORAGE_ENDPOINT");
            return Self::new(endpoint);
        }

        // Fall back to capability discovery
        // TODO: Integrate with ZeroKnowledgeBootstrap for true infant discovery
        let endpoint =
            std::env::var("SONGBIRD_HOST").unwrap_or_else(|_| "http://localhost".to_string());
        let port = std::env::var("SONGBIRD_STORAGE_PORT").unwrap_or_else(|_| "8082".to_string());
        let discovered_endpoint = format!("{endpoint}:{port}");

        debug!("🔍 Storage capability discovered at: {}", discovered_endpoint);
        Self::new(discovered_endpoint)
    }

    /// Create adapter with explicit endpoint (for testing or explicit configuration)
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Base URL of any storage capability provider
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(endpoint: String) -> SongbirdResult<Self> {
        Ok(Self {
            endpoint,
            client: reqwest::Client::builder().timeout(Duration::from_secs(10)).build().map_err(
                |e| SongbirdError::configuration(format!("Failed to create HTTP client: {e}")),
            )?,
            timeout: Duration::from_secs(5),
        })
    }

    /// Set custom request timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Collect storage metrics from the capability provider
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network request fails
    /// - Service returns non-success status
    /// - Response cannot be parsed
    pub async fn collect_metrics(&self) -> SongbirdResult<StorageMetrics> {
        let url = format!("{}/metrics/storage", self.endpoint);

        debug!("Collecting storage metrics from: {}", url);

        let response = self.client.get(&url).timeout(self.timeout).send().await.map_err(|e| {
            warn!("Failed to reach storage capability provider: {e}");
            songbird_types::SongbirdError::network(format!("Failed to reach storage provider: {e}"))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            warn!("Storage capability provider returned error status: {}", status);
            return Err(songbird_types::SongbirdError::service(
                "storage",
                format!("HTTP {status}: Storage metrics unavailable"),
            ));
        }

        let mut metrics: StorageMetrics = response.json().await.map_err(|e| {
            warn!("Failed to parse storage metrics: {e}");
            songbird_types::SongbirdError::service(
                "storage",
                format!("Failed to parse storage metrics: {e}"),
            )
        })?;

        // Set timestamp if not provided
        if metrics.timestamp.timestamp() == 0 {
            metrics.timestamp = chrono::Utc::now();
        }

        debug!(
            "Collected storage metrics: Usage={}%, Objects={}, ReadLatency={}ms, WriteLatency={}ms",
            metrics.usage_percent(),
            metrics.object_count,
            metrics.avg_read_latency_ms,
            metrics.avg_write_latency_ms
        );

        Ok(metrics)
    }

    /// Check storage health
    ///
    /// # Errors
    ///
    /// Returns an error if the health check fails
    pub async fn check_health(&self) -> SongbirdResult<StorageHealth> {
        let metrics = self.collect_metrics().await?;
        Ok(metrics.health_status())
    }

    /// Get the endpoint URL
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

/// Trait for storage capability providers
#[async_trait]
pub trait StorageProvider {
    /// Collect current storage metrics
    async fn collect_storage_metrics(&self) -> SongbirdResult<StorageMetrics>;

    /// Check storage health
    async fn check_storage_health(&self) -> SongbirdResult<StorageHealth> {
        let metrics = self.collect_storage_metrics().await?;
        Ok(metrics.health_status())
    }
}

#[async_trait]
impl StorageProvider for StorageAdapter {
    async fn collect_storage_metrics(&self) -> SongbirdResult<StorageMetrics> {
        self.collect_metrics().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_metrics_calculations() {
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000, // 1TB
            used_bytes: 250_000_000_000,             // 250GB
            available_bytes: 750_000_000_000,        // 750GB
            object_count: 1_500,
            avg_read_latency_ms: 15.0,
            avg_write_latency_ms: 25.0,
            timestamp: chrono::Utc::now(),
        };

        assert!((metrics.usage_percent() - 25.0).abs() < 0.1);
        assert!(!metrics.is_nearly_full());
        assert!(!metrics.is_high_latency());
        assert_eq!(metrics.health_status(), StorageHealth::Healthy);
    }

    #[test]
    fn test_storage_nearly_full() {
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 960_000_000_000, // 96%
            available_bytes: 40_000_000_000,
            object_count: 50_000,
            avg_read_latency_ms: 20.0,
            avg_write_latency_ms: 600.0, // High write latency
            timestamp: chrono::Utc::now(),
        };

        assert!(metrics.is_nearly_full());
        assert_eq!(metrics.health_status(), StorageHealth::Critical);
    }

    #[test]
    fn test_storage_warning() {
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 870_000_000_000, // 87%
            available_bytes: 130_000_000_000,
            object_count: 25_000,
            avg_read_latency_ms: 120.0, // High read latency
            avg_write_latency_ms: 180.0,
            timestamp: chrono::Utc::now(),
        };

        assert!(!metrics.is_nearly_full());
        assert!(metrics.is_high_latency());
        assert_eq!(metrics.health_status(), StorageHealth::Warning);
    }

    #[test]
    fn test_adapter_creation() {
        let adapter = StorageAdapter::new("http://storage-provider:8082".to_string())
            .expect("Adapter creation should succeed");
        assert_eq!(adapter.endpoint(), "http://storage-provider:8082");
    }

    #[test]
    fn test_adapter_with_timeout() {
        let adapter = StorageAdapter::new("http://storage-provider:8082".to_string())
            .expect("Adapter creation should succeed")
            .with_timeout(Duration::from_secs(10));
        assert_eq!(adapter.timeout, Duration::from_secs(10));
    }
}
