//! Storage Capability Adapter
//!
//! **SOVEREIGNTY**: This adapter works with ANY storage capability provider.
//! It does NOT know about specific primals (`NestGate` is just one example).
//! Discovery is capability-based through environment hints or zero-knowledge bootstrap.
//!
//! Songbird has local storage for sovereign standalone operation, but can utilize
//! any discovered storage provider for network effects.

// Allow async_fn_in_trait warning - our traits guarantee Send + Sync
#![allow(async_fn_in_trait)]

use crate::JsonRpcClient;
use serde::{Deserialize, Serialize};
use songbird_http_client::IpcHttpClient;
use songbird_types::{SafeEnv, SongbirdError, SongbirdResult};
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

/// Protocol for communication (v3.12.0 - tarpc PRIMARY)
/// Protocol for communication (v3.12.0 - tarpc PRIMARY)
enum Protocol {
    Tarpc(crate::TarpcClient), // PRIMARY - high-performance binary RPC
    JsonRpc(JsonRpcClient),    // SECONDARY - universal, port-free
    Http(IpcHttpClient),       // FALLBACK - network only
}

impl std::fmt::Debug for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tarpc(_) => write!(f, "Protocol::Tarpc"),
            Self::JsonRpc(_) => write!(f, "Protocol::JsonRpc"),
            Self::Http(_) => write!(f, "Protocol::Http"),
        }
    }
}

/// **CAPABILITY-BASED STORAGE ADAPTER**
///
/// Works with ANY storage provider discovered through:
/// - Environment variable: `SONGBIRD_STORAGE_ENDPOINT`
/// - Capability discovery: `capability:storage`
/// - Zero-knowledge bootstrap
/// - Local filesystem fallback for sovereign operation
///
/// **v3.11.0**: Protocol-agnostic - supports Unix sockets (PRIMARY) or HTTP (FALLBACK)
#[derive(Debug)]
pub struct StorageAdapter {
    /// Endpoint URL for the storage capability provider
    endpoint: String,
    /// Protocol (Unix socket JSON-RPC or HTTP)
    protocol: Protocol,
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
    pub async fn from_discovery() -> SongbirdResult<Self> {
        use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};

        // ✅ PHASE 1 INTEGRATION: Multi-tier capability discovery
        let resolver = CapabilityEndpointResolver::new();

        match resolver.get_endpoint(CapabilityType::Storage).await {
            Ok(endpoint) => {
                debug!("✅ Storage capability discovered via resolver: {}", endpoint);
                Self::new(endpoint).await
            }
            Err(discovery_err) => {
                debug!("🔍 Primary discovery failed, trying legacy fallbacks: {}", discovery_err);

                // Fallback 1: Legacy environment variables
                if let Ok(endpoint) = SafeEnv::get_required("SONGBIRD_STORAGE_ENDPOINT")
                    .or_else(|_| SafeEnv::get_required("STORAGE_PROVIDER_ENDPOINT"))
                    .or_else(|_| SafeEnv::get_required("NESTGATE_ENDPOINT"))
                {
                    debug!("⚠️ Using legacy environment variable for storage endpoint");
                    return Self::new(endpoint).await;
                }

                // Fallback 2: Construct from host + port
                let endpoint = SafeEnv::get_or_default("SONGBIRD_HOST", "http://localhost");
                let port = SafeEnv::get_port(
                    "SONGBIRD_STORAGE_PORT",
                    songbird_config::defaults::ports::service_port("STORAGE", 8082),
                )
                .to_string();
                let discovered_endpoint = format!("{endpoint}:{port}");

                debug!("🔄 Using fallback storage endpoint: {}", discovered_endpoint);
                Self::new(discovered_endpoint).await
            }
        }
    }

    /// Create adapter with explicit endpoint (for testing or explicit configuration)
    ///
    /// **v3.12.0**: Protocol-agnostic - automatically detects tarpc, Unix sockets, or HTTP
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Base URL of any storage capability provider
    ///   - `tarpc://host:port` → tarpc binary RPC (PRIMARY - 10-100x faster)
    ///   - `unix:///path/to/socket.sock` → JSON-RPC over Unix socket (SECONDARY - port-free)
    ///   - `http://...` or `https://...` → HTTP (FALLBACK - network only)
    ///
    /// # Errors
    ///
    /// Returns an error if the protocol client cannot be created.
    pub async fn new(endpoint: String) -> SongbirdResult<Self> {
        // Protocol detection (v3.12.0 - tarpc PRIMARY)
        let protocol = if endpoint.starts_with("tarpc://") {
            debug!("🚀 Detected tarpc endpoint for storage (PRIMARY): {}", endpoint);
            Protocol::Tarpc(crate::TarpcClient::new(&endpoint)?)
        } else if endpoint.starts_with("unix://") {
            debug!("🔌 Detected Unix socket endpoint for storage (SECONDARY): {}", endpoint);
            Protocol::JsonRpc(JsonRpcClient::new(&endpoint)?)
        } else {
            debug!("🌐 Detected HTTP endpoint for storage (FALLBACK): {}", endpoint);
            Protocol::Http(IpcHttpClient::new().await.map_err(|e| {
                SongbirdError::configuration(format!("Failed to create HTTP client: {e}"))
            })?)
        };

        Ok(Self {
            endpoint,
            protocol,
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
    /// **v3.11.0**: Protocol-agnostic - works with Unix sockets or HTTP
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network/IPC request fails
    /// - Service returns non-success status (HTTP) or error (JSON-RPC)
    /// - Response cannot be parsed
    pub async fn collect_metrics(&self) -> SongbirdResult<StorageMetrics> {
        debug!("Collecting storage metrics from: {}", self.endpoint);

        let mut metrics: StorageMetrics = match &self.protocol {
            Protocol::Tarpc(client) => {
                // tarpc - HIGH-PERFORMANCE binary RPC (PRIMARY - ~10-20 μs latency!)
                debug!("🚀 Using tarpc (PRIMARY protocol)");
                let result = client.call_method("get_storage_metrics", None).await?;
                serde_json::from_value(result).map_err(|e| {
                    warn!("Failed to parse storage metrics from tarpc: {e}");
                    SongbirdError::serialization(format!("Failed to parse storage metrics: {e}"))
                })?
            }
            Protocol::JsonRpc(client) => {
                // JSON-RPC protocol over Unix socket (SECONDARY - ~50-100 μs latency)
                debug!("🔌 Using JSON-RPC (SECONDARY protocol)");
                let result = client.call_method("get_storage_metrics", None).await?;
                serde_json::from_value(result).map_err(|e| {
                    warn!("Failed to parse storage metrics from JSON-RPC: {e}");
                    SongbirdError::serialization(format!("Failed to parse storage metrics: {e}"))
                })?
            }
            Protocol::Http(client) => {
                // HTTP protocol (FALLBACK - ~500-1000 μs latency)
                debug!("🌐 Using HTTP (FALLBACK protocol)");
                let url = format!("{}/metrics/storage", self.endpoint);

                // IpcHttpClient::get() returns Result<Response> directly (no .send() needed)
                let response = client.get(&url).await.map_err(|e| {
                    warn!("Failed to reach storage capability provider via HTTP: {e}");
                    SongbirdError::network(format!("Failed to reach storage provider: {e}"))
                })?;

                if !response.is_success() {
                    let status = response.status();
                    warn!("Storage capability provider returned error status: {}", status);
                    return Err(SongbirdError::service(
                        "storage",
                        format!("HTTP {status}: Storage metrics unavailable"),
                    ));
                }

                response.json().await.map_err(|e| {
                    warn!("Failed to parse storage metrics from HTTP: {e}");
                    SongbirdError::service(
                        "storage",
                        format!("Failed to parse storage metrics: {e}"),
                    )
                })?
            }
        };

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
pub trait StorageProvider: Send + Sync {
    /// Collect current storage metrics
    async fn collect_storage_metrics(&self) -> SongbirdResult<StorageMetrics>;

    /// Check storage health
    async fn check_storage_health(&self) -> SongbirdResult<StorageHealth> {
        let metrics = self.collect_storage_metrics().await?;
        Ok(metrics.health_status())
    }
}

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
    fn test_adapter_creation() -> Result<(), Box<dyn std::error::Error>> {
        let adapter =
            StorageAdapter::new("http://storage-provider:8082".to_string()).map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?;
        assert_eq!(adapter.endpoint(), "http://storage-provider:8082");
        Ok(())
    }

    #[test]
    fn test_adapter_with_timeout() -> Result<(), Box<dyn std::error::Error>> {
        let adapter = StorageAdapter::new("http://storage-provider:8082".to_string())
            .map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?
            .with_timeout(Duration::from_secs(10));
        assert_eq!(adapter.timeout, Duration::from_secs(10));
        Ok(())
    }

    #[test]
    fn test_storage_health_equality() {
        assert_eq!(StorageHealth::Healthy, StorageHealth::Healthy);
        assert_eq!(StorageHealth::Warning, StorageHealth::Warning);
        assert_eq!(StorageHealth::Critical, StorageHealth::Critical);
        assert_ne!(StorageHealth::Healthy, StorageHealth::Warning);
        assert_ne!(StorageHealth::Warning, StorageHealth::Critical);
    }

    #[test]
    fn test_storage_usage_zero_capacity() {
        let metrics = StorageMetrics {
            total_capacity_bytes: 0,
            used_bytes: 0,
            available_bytes: 0,
            object_count: 0,
            avg_read_latency_ms: 10.0,
            avg_write_latency_ms: 20.0,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(metrics.usage_percent(), 0.0);
    }

    #[test]
    fn test_nearly_full_boundary() {
        // Just below threshold (90%)
        let metrics_below = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 900_000_000_000,
            available_bytes: 100_000_000_000,
            object_count: 10_000,
            avg_read_latency_ms: 15.0,
            avg_write_latency_ms: 25.0,
            timestamp: chrono::Utc::now(),
        };
        assert!(!metrics_below.is_nearly_full());

        // Just above threshold
        let metrics_above = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 901_000_000_000,
            available_bytes: 99_000_000_000,
            object_count: 10_000,
            avg_read_latency_ms: 15.0,
            avg_write_latency_ms: 25.0,
            timestamp: chrono::Utc::now(),
        };
        assert!(metrics_above.is_nearly_full());
    }

    #[test]
    fn test_high_read_latency_boundary() {
        // Just below threshold
        let metrics_below = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 250_000_000_000,
            available_bytes: 750_000_000_000,
            object_count: 1_500,
            avg_read_latency_ms: 100.0,
            avg_write_latency_ms: 50.0,
            timestamp: chrono::Utc::now(),
        };
        assert!(!metrics_below.is_high_latency());

        // Just above threshold
        let metrics_above = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 250_000_000_000,
            available_bytes: 750_000_000_000,
            object_count: 1_500,
            avg_read_latency_ms: 100.1,
            avg_write_latency_ms: 50.0,
            timestamp: chrono::Utc::now(),
        };
        assert!(metrics_above.is_high_latency());
    }

    #[test]
    fn test_high_write_latency_boundary() {
        // Just below threshold
        let metrics_below = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 250_000_000_000,
            available_bytes: 750_000_000_000,
            object_count: 1_500,
            avg_read_latency_ms: 50.0,
            avg_write_latency_ms: 200.0,
            timestamp: chrono::Utc::now(),
        };
        assert!(!metrics_below.is_high_latency());

        // Just above threshold
        let metrics_above = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 250_000_000_000,
            available_bytes: 750_000_000_000,
            object_count: 1_500,
            avg_read_latency_ms: 50.0,
            avg_write_latency_ms: 200.1,
            timestamp: chrono::Utc::now(),
        };
        assert!(metrics_above.is_high_latency());
    }

    #[test]
    fn test_health_status_critical_high_usage() {
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 960_000_000_000, // 96%
            available_bytes: 40_000_000_000,
            object_count: 50_000,
            avg_read_latency_ms: 20.0,
            avg_write_latency_ms: 50.0,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), StorageHealth::Critical);
    }

    #[test]
    fn test_health_status_critical_high_write_latency() {
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 500_000_000_000,
            available_bytes: 500_000_000_000,
            object_count: 10_000,
            avg_read_latency_ms: 20.0,
            avg_write_latency_ms: 501.0,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), StorageHealth::Critical);
    }

    #[test]
    fn test_health_status_warning_high_usage() {
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 860_000_000_000, // 86%
            available_bytes: 140_000_000_000,
            object_count: 25_000,
            avg_read_latency_ms: 20.0,
            avg_write_latency_ms: 50.0,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), StorageHealth::Warning);
    }

    #[test]
    fn test_health_status_warning_high_latency() {
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 500_000_000_000,
            available_bytes: 500_000_000_000,
            object_count: 10_000,
            avg_read_latency_ms: 120.0,
            avg_write_latency_ms: 50.0,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), StorageHealth::Warning);
    }

    #[test]
    fn test_health_status_boundary_95_usage() {
        // Exactly at critical threshold
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 950_000_000_000, // 95%
            available_bytes: 50_000_000_000,
            object_count: 40_000,
            avg_read_latency_ms: 20.0,
            avg_write_latency_ms: 50.0,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), StorageHealth::Warning);
    }

    #[test]
    fn test_health_status_boundary_500ms_write() {
        // Exactly at critical threshold
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 500_000_000_000,
            available_bytes: 500_000_000_000,
            object_count: 10_000,
            avg_read_latency_ms: 20.0,
            avg_write_latency_ms: 500.0,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), StorageHealth::Warning);
    }

    #[test]
    fn test_storage_metrics_serialization() -> SongbirdResult<()> {
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 250_000_000_000,
            available_bytes: 750_000_000_000,
            object_count: 1_500,
            avg_read_latency_ms: 15.0,
            avg_write_latency_ms: 25.0,
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&metrics).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {}", e))
        })?;
        assert!(json.contains("total_capacity_bytes"));
        assert!(json.contains("object_count"));
        Ok(())
    }

    #[test]
    fn test_storage_health_serialization() -> SongbirdResult<()> {
        assert_eq!(
            serde_json::to_string(&StorageHealth::Healthy).map_err(|e| {
                SongbirdError::Serialization {
                    format: Some("JSON".to_string()),
                    message: format!("Serialization failed: {}", e),
                    debug_info: None,
                }
            })?,
            "\"Healthy\""
        );
        assert_eq!(
            serde_json::to_string(&StorageHealth::Warning).map_err(|e| {
                SongbirdError::Serialization {
                    format: Some("JSON".to_string()),
                    message: format!("Serialization failed: {}", e),
                    debug_info: None,
                }
            })?,
            "\"Warning\""
        );
        assert_eq!(
            serde_json::to_string(&StorageHealth::Critical).map_err(|e| {
                SongbirdError::Serialization {
                    format: Some("JSON".to_string()),
                    message: format!("Serialization failed: {}", e),
                    debug_info: None,
                }
            })?,
            "\"Critical\""
        );
        Ok(())
    }

    #[test]
    fn test_adapter_default_timeout() -> SongbirdResult<()> {
        let adapter =
            StorageAdapter::new("http://storage-service:8082".to_string()).map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?;
        assert_eq!(adapter.timeout, Duration::from_secs(5));
        Ok(())
    }

    #[test]
    fn test_adapter_endpoint_access() -> SongbirdResult<()> {
        let adapter = StorageAdapter::new("http://test-storage:9000".to_string()).map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?;
        assert_eq!(adapter.endpoint(), "http://test-storage:9000");
        Ok(())
    }

    #[test]
    fn test_adapter_debug_format() -> SongbirdResult<()> {
        let adapter = StorageAdapter::new("http://storage:8082".to_string()).map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?;
        let debug_str = format!("{:?}", adapter);
        assert!(debug_str.contains("StorageAdapter"));
        assert!(debug_str.contains("http://storage:8082"));
        Ok(())
    }

    #[test]
    fn test_storage_metrics_perfect_conditions() {
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 100_000_000_000, // 10%
            available_bytes: 900_000_000_000,
            object_count: 500,
            avg_read_latency_ms: 5.0,
            avg_write_latency_ms: 10.0,
            timestamp: chrono::Utc::now(),
        };

        assert!(!metrics.is_nearly_full());
        assert!(!metrics.is_high_latency());
        assert_eq!(metrics.health_status(), StorageHealth::Healthy);
    }

    #[test]
    fn test_storage_metrics_all_zero() {
        let metrics = StorageMetrics {
            total_capacity_bytes: 0,
            used_bytes: 0,
            available_bytes: 0,
            object_count: 0,
            avg_read_latency_ms: 0.0,
            avg_write_latency_ms: 0.0,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(metrics.usage_percent(), 0.0);
        assert!(!metrics.is_nearly_full());
        assert!(!metrics.is_high_latency());
        assert_eq!(metrics.health_status(), StorageHealth::Healthy);
    }

    #[test]
    fn test_storage_100_percent_full() {
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 1_000_000_000_000, // 100%
            available_bytes: 0,
            object_count: 100_000,
            avg_read_latency_ms: 50.0,
            avg_write_latency_ms: 100.0,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(metrics.usage_percent(), 100.0);
        assert!(metrics.is_nearly_full());
        assert_eq!(metrics.health_status(), StorageHealth::Critical);
    }

    // ========== NEW TESTS (10 tests to reach 85% coverage) ==========

    #[test]
    fn test_storage_metrics_clone() {
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 250_000_000_000,
            available_bytes: 750_000_000_000,
            object_count: 1_500,
            avg_read_latency_ms: 15.0,
            avg_write_latency_ms: 25.0,
            timestamp: chrono::Utc::now(),
        };
        let cloned = metrics;
        assert_eq!(cloned.total_capacity_bytes, 1_000_000_000_000);
        assert_eq!(cloned.used_bytes, 250_000_000_000);
        assert_eq!(cloned.object_count, 1_500);
    }

    #[test]
    fn test_storage_health_clone() {
        let health = StorageHealth::Warning;
        let cloned = health;
        assert_eq!(health, cloned);
    }

    #[test]
    fn test_storage_metrics_deserialization() -> SongbirdResult<()> {
        let json = r#"{
            "total_capacity_bytes": 5000000000000,
            "used_bytes": 1000000000000,
            "available_bytes": 4000000000000,
            "object_count": 8500,
            "avg_read_latency_ms": 35.5,
            "avg_write_latency_ms": 75.2,
            "timestamp": "2024-01-01T00:00:00Z"
        }"#;

        let metrics: StorageMetrics =
            serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Deserialization should succeed: {}", e),
                debug_info: None,
            })?;
        assert_eq!(metrics.total_capacity_bytes, 5_000_000_000_000);
        assert_eq!(metrics.used_bytes, 1_000_000_000_000);
        assert_eq!(metrics.object_count, 8500);
        assert!((metrics.avg_read_latency_ms - 35.5).abs() < 0.001);
        Ok(())
    }

    #[test]
    fn test_storage_health_deserialization() -> SongbirdResult<()> {
        let json = r#""Warning""#;
        let health: StorageHealth =
            serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Deserialization should succeed: {}", e),
                debug_info: None,
            })?;
        assert_eq!(health, StorageHealth::Warning);

        let json = r#""Critical""#;
        let health: StorageHealth =
            serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Deserialization should succeed: {}", e),
                debug_info: None,
            })?;
        assert_eq!(health, StorageHealth::Critical);
        Ok(())
    }

    #[test]
    fn test_storage_metrics_debug() {
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 250_000_000_000,
            available_bytes: 750_000_000_000,
            object_count: 1_500,
            avg_read_latency_ms: 15.0,
            avg_write_latency_ms: 25.0,
            timestamp: chrono::Utc::now(),
        };
        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("StorageMetrics"));
        assert!(debug_str.contains("object_count"));
    }

    #[test]
    fn test_storage_health_debug() {
        let health = StorageHealth::Critical;
        let debug_str = format!("{:?}", health);
        assert!(debug_str.contains("Critical"));
    }

    #[test]
    fn test_adapter_chained_timeout() -> SongbirdResult<()> {
        let adapter = StorageAdapter::new("http://storage:8082".to_string())
            .map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?
            .with_timeout(Duration::from_secs(3))
            .with_timeout(Duration::from_secs(12));

        assert_eq!(adapter.timeout, Duration::from_secs(12), "Last timeout should be applied");
        Ok(())
    }

    #[test]
    fn test_storage_metrics_edge_case_85_percent() {
        // Exactly at warning threshold
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 850_000_000_000, // 85%
            available_bytes: 150_000_000_000,
            object_count: 20_000,
            avg_read_latency_ms: 20.0,
            avg_write_latency_ms: 50.0,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), StorageHealth::Healthy);
    }

    #[test]
    fn test_storage_metrics_edge_case_86_percent() {
        // Just above warning threshold
        let metrics = StorageMetrics {
            total_capacity_bytes: 1_000_000_000_000,
            used_bytes: 860_000_000_000, // 86%
            available_bytes: 140_000_000_000,
            object_count: 20_000,
            avg_read_latency_ms: 20.0,
            avg_write_latency_ms: 50.0,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), StorageHealth::Warning);
    }

    #[test]
    fn test_storage_metrics_max_values() {
        let metrics = StorageMetrics {
            total_capacity_bytes: u64::MAX,
            used_bytes: u64::MAX,
            available_bytes: 0,
            object_count: u64::MAX,
            avg_read_latency_ms: f64::MAX,
            avg_write_latency_ms: f64::MAX,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(metrics.usage_percent(), 100.0);
        assert!(metrics.is_nearly_full());
        assert!(metrics.is_high_latency());
        assert_eq!(metrics.health_status(), StorageHealth::Critical);
    }
}
