// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Generic Compute Capability Adapter
//!
//! **SOVEREIGNTY PRINCIPLE**: This adapter is capability-based and works with
//! ANY service providing compute capabilities. It does NOT know about specific
//! primals like `ToadStool` - it only knows about "compute capability providers".
//!
//! ## Ecological Model
//!
//! Like in ecology, each organism exists independently:
//! - Songbird doesn't "know" `ToadStool` exists
//! - Songbird only knows "something provides compute capability"
//! - `ToadStool` happens to implement this capability in the ecosystem
//! - But it could be ANY compute provider
//!
//! ## Example
//!
//! ```rust,ignore
//! # tokio_test::block_on(async {
//! use songbird_universal::adapters::ComputeAdapter;
//!
//! // Discovers whoever provides compute capability
//! let adapter = ComputeAdapter::new_from_discovery().await?;
//! let metrics = adapter.collect_metrics().await?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! # });
//! ```

// Allow async_fn_in_trait warning - our traits guarantee Send + Sync
#![allow(async_fn_in_trait)]

use crate::JsonRpcClient;
use serde::{Deserialize, Serialize};
use songbird_http_client::SongbirdHttpClient;
use songbird_types::{SafeEnv, SongbirdError, SongbirdResult};
use std::time::Duration;
use tracing::{debug, warn};

/// Compute metrics from any compute capability provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeMetrics {
    /// CPU usage percentage (0.0 - 100.0)
    pub cpu_usage_percent: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Available memory in bytes
    pub memory_available_bytes: u64,
    /// Number of active containers/workloads
    pub active_containers: u32,
    /// Number of queued jobs
    pub queued_jobs: u32,
    /// Overall performance score (0.0 - 1.0)
    pub performance_score: f64,
    /// Timestamp of metrics collection
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ComputeMetrics {
    /// Calculate total memory in bytes
    #[must_use]
    pub const fn total_memory_bytes(&self) -> u64 {
        self.memory_usage_bytes + self.memory_available_bytes
    }

    /// Calculate memory usage percentage
    #[must_use]
    pub fn memory_usage_percent(&self) -> f64 {
        let total = self.total_memory_bytes();
        if total == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        {
            (self.memory_usage_bytes as f64 / total as f64) * 100.0
        }
    }

    /// Check if system is under high load
    #[must_use]
    pub fn is_high_load(&self) -> bool {
        self.cpu_usage_percent > 80.0 || self.memory_usage_percent() > 85.0 || self.queued_jobs > 10
    }

    /// Get health status based on metrics
    #[must_use]
    pub fn health_status(&self) -> HealthStatus {
        if self.cpu_usage_percent > 95.0 || self.memory_usage_percent() > 95.0 {
            HealthStatus::Unhealthy
        } else if self.is_high_load() {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}

/// Health status derived from metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System is degraded but functional
    Degraded,
    /// System is unhealthy
    Unhealthy,
}

/// Protocol for communication (v3.12.0 - tarpc PRIMARY)
enum Protocol {
    Tarpc(crate::TarpcClient), // PRIMARY - high-performance binary RPC
    JsonRpc(JsonRpcClient),    // SECONDARY - universal, port-free
    Http(SongbirdHttpClient),  // FALLBACK - direct HTTP (no IPC delegation)
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

/// Generic adapter for compute capability providers
///
/// **SOVEREIGNTY**: This adapter discovers compute providers by capability,
/// not by hardcoded primal names. It works with ANY service that implements
/// the compute capability interface.
///
/// **v3.11.0**: Protocol-agnostic - supports Unix sockets (PRIMARY) or HTTP (FALLBACK)
#[derive(Debug)]
pub struct ComputeAdapter {
    /// Endpoint URL for the compute service (discovered dynamically)
    endpoint: String,
    /// Protocol (Unix socket JSON-RPC or HTTP)
    protocol: Protocol,
    /// Request timeout
    timeout: Duration,
}

impl ComputeAdapter {
    /// Create adapter by discovering compute capability provider
    ///
    /// **SOVEREIGNTY**: Discovers whoever provides "compute" capability,
    /// doesn't assume any specific primal exists.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No compute capability provider found
    /// - HTTP client creation fails
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # tokio_test::block_on(async {
    /// use songbird_universal::adapters::ComputeAdapter;
    ///
    /// // Discovers compute provider dynamically
    /// let adapter = ComputeAdapter::new_from_discovery().await?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// # });
    /// ```
    pub async fn new_from_discovery() -> SongbirdResult<Self> {
        use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};

        // ✅ PHASE 1 INTEGRATION: Multi-tier capability discovery
        // Priority: Environment → Service Registry → Container Metadata → DNS SRV
        let resolver = CapabilityEndpointResolver::new();

        match resolver.get_endpoint(CapabilityType::Compute).await {
            Ok(endpoint) => {
                debug!("✅ Compute capability discovered via resolver: {}", endpoint);
                Self::new(endpoint).await
            }
            Err(discovery_err) => {
                debug!("🔍 Primary discovery failed, trying legacy fallbacks: {}", discovery_err);

                // Fallback 1: Legacy environment variables
                if let Ok(endpoint) = SafeEnv::get_required("SONGBIRD_COMPUTE_ENDPOINT")
                    .or_else(|_| SafeEnv::get_required("COMPUTE_CAPABILITY_ENDPOINT"))
                    .or_else(|_| SafeEnv::get_required("TOADSTOOL_ENDPOINT"))
                {
                    debug!("⚠️ Using legacy environment variable for compute endpoint");
                    return Self::new(endpoint).await;
                }

                // Fallback 2: Construct from host + port
                let host = SafeEnv::get_or_default("SONGBIRD_HOST", "http://localhost");
                let port = SafeEnv::get_port(
                    "SONGBIRD_COMPUTE_PORT",
                    songbird_config::defaults::ports::service_port("COMPUTE", 8080),
                )
                .to_string();
                let endpoint = format!("{host}:{port}");

                debug!("🔄 Using fallback compute endpoint: {}", endpoint);
                Self::new(endpoint).await
            }
        }
    }

    /// Create adapter with explicit endpoint
    ///
    /// **v3.11.0**: Protocol-agnostic - automatically detects Unix sockets or HTTP
    ///
    /// Use this when you already know the compute provider's endpoint
    /// (e.g., from service discovery).
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Base URL of any compute capability provider
    ///   - `tarpc://host:port` → tarpc binary RPC (PRIMARY - 10-100x faster)
    ///   - `unix:///path/to/socket.sock` → JSON-RPC over Unix socket (SECONDARY - port-free)
    ///   - `http://...` or `https://...` → HTTP (FALLBACK - network only)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use songbird_universal::adapters::ComputeAdapter;
    ///
    /// // Works with ANY compute provider - protocol auto-detected
    /// let adapter = ComputeAdapter::new("tarpc://localhost:9001".to_string())?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the protocol client cannot be created.
    #[allow(clippy::unused_async)] // async retained for API stability
    pub async fn new(endpoint: String) -> SongbirdResult<Self> {
        // Protocol detection (v3.12.0 - tarpc PRIMARY)
        let protocol = if endpoint.starts_with("tarpc://") {
            debug!("🚀 Detected tarpc endpoint for compute (PRIMARY): {}", endpoint);
            Protocol::Tarpc(crate::TarpcClient::new(&endpoint)?)
        } else if endpoint.starts_with("unix://") {
            debug!("🔌 Detected Unix socket endpoint for compute (SECONDARY): {}", endpoint);
            Protocol::JsonRpc(JsonRpcClient::new(&endpoint)?)
        } else {
            debug!("🌐 Detected HTTP endpoint for compute (FALLBACK): {}", endpoint);
            Protocol::Http(SongbirdHttpClient::from_env())
        };

        Ok(Self {
            endpoint,
            protocol,
            timeout: Duration::from_secs(5),
        })
    }

    /// Set custom request timeout
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Collect compute metrics from the service
    ///
    /// **v3.11.0**: Protocol-agnostic - works with Unix sockets or HTTP
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network/IPC request fails
    /// - Service returns non-success status (HTTP) or error (JSON-RPC)
    /// - Response cannot be parsed
    pub async fn collect_metrics(&self) -> SongbirdResult<ComputeMetrics> {
        debug!("Collecting compute metrics from: {}", self.endpoint);

        let mut metrics: ComputeMetrics = match &self.protocol {
            Protocol::Tarpc(client) => {
                // tarpc - HIGH-PERFORMANCE binary RPC (PRIMARY - ~10-20 μs latency!)
                debug!("🚀 Using tarpc (PRIMARY protocol)");
                let result = client.call_method("get_compute_metrics", None).await?;
                serde_json::from_value(result).map_err(|e| {
                    warn!("Failed to parse compute metrics from tarpc: {e}");
                    SongbirdError::serialization(format!("Failed to parse compute metrics: {e}"))
                })?
            }
            Protocol::JsonRpc(client) => {
                // JSON-RPC protocol over Unix socket (SECONDARY - ~50-100 μs latency)
                debug!("🔌 Using JSON-RPC (SECONDARY protocol)");
                let result = client.call_method("get_compute_metrics", None).await?;
                serde_json::from_value(result).map_err(|e| {
                    warn!("Failed to parse compute metrics from JSON-RPC: {e}");
                    SongbirdError::serialization(format!("Failed to parse compute metrics: {e}"))
                })?
            }
            Protocol::Http(client) => {
                // HTTP protocol (FALLBACK - direct TCP connection)
                debug!("🌐 Using HTTP (FALLBACK protocol)");
                let url = format!("{}/metrics/compute", self.endpoint);

                let response = tokio::time::timeout(self.timeout, client.get(&url))
                    .await
                    .map_err(|_| {
                        SongbirdError::network(format!(
                            "Timeout after {:?} reaching compute service",
                            self.timeout
                        ))
                    })?
                    .map_err(|e| {
                        warn!("Failed to reach compute service via HTTP: {e}");
                        SongbirdError::network(format!("Failed to reach compute service: {e}"))
                    })?;

                if !(200..300).contains(&response.status) {
                    let status = response.status;
                    warn!("Compute service returned error status: {}", status);
                    return Err(SongbirdError::service(
                        "compute",
                        format!("HTTP {status}: Metrics unavailable"),
                    ));
                }

                serde_json::from_value(response.body).map_err(|e| {
                    warn!("Failed to parse compute metrics from HTTP: {e}");
                    SongbirdError::service(
                        "compute",
                        format!("Failed to parse compute metrics: {e}"),
                    )
                })?
            }
        };

        // Set timestamp if not provided
        if metrics.timestamp.timestamp() == 0 {
            metrics.timestamp = chrono::Utc::now();
        }

        debug!(
            "Collected compute metrics: CPU={}%, Memory={}%, Active={}, Queued={}",
            metrics.cpu_usage_percent,
            metrics.memory_usage_percent(),
            metrics.active_containers,
            metrics.queued_jobs
        );

        Ok(metrics)
    }

    /// Check health of the compute service
    ///
    /// # Errors
    ///
    /// Returns an error if the health check fails
    pub async fn check_health(&self) -> SongbirdResult<HealthStatus> {
        let metrics = self.collect_metrics().await?;
        Ok(metrics.health_status())
    }

    /// Get the endpoint URL
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

/// Trait for compute metrics collection (capability-based)
pub trait ComputeMetricsProvider: Send + Sync {
    /// Collect current compute metrics
    async fn collect_compute_metrics(&self) -> SongbirdResult<ComputeMetrics>;

    /// Check compute service health
    async fn check_compute_health(&self) -> SongbirdResult<HealthStatus> {
        let metrics = self.collect_compute_metrics().await?;
        Ok(metrics.health_status())
    }
}

impl ComputeMetricsProvider for ComputeAdapter {
    async fn collect_compute_metrics(&self) -> SongbirdResult<ComputeMetrics> {
        self.collect_metrics().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_metrics_calculations() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 45.0,
            memory_usage_bytes: 2_000_000_000,
            memory_available_bytes: 6_000_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(metrics.total_memory_bytes(), 8_000_000_000);
        assert!((metrics.memory_usage_percent() - 25.0).abs() < 0.1);
        assert!(!metrics.is_high_load());
        assert_eq!(metrics.health_status(), HealthStatus::Healthy);
    }

    #[test]
    fn test_high_load_detection() -> SongbirdResult<()> {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 96.0,
            memory_usage_bytes: 7_600_000_000,
            memory_available_bytes: 400_000_000,
            active_containers: 20,
            queued_jobs: 15,
            performance_score: 0.45,
            timestamp: chrono::Utc::now(),
        };

        assert!(metrics.is_high_load());
        assert_eq!(metrics.health_status(), HealthStatus::Unhealthy);
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_creation() -> Result<(), Box<dyn std::error::Error>> {
        let adapter =
            ComputeAdapter::new("http://localhost:8080".to_string()).await.map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?;
        assert_eq!(adapter.endpoint(), "http://localhost:8080");
        Ok(())
    }

    #[test]
    fn test_health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_eq!(HealthStatus::Degraded, HealthStatus::Degraded);
        assert_eq!(HealthStatus::Unhealthy, HealthStatus::Unhealthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
        assert_ne!(HealthStatus::Degraded, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_memory_usage_zero_total() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 0,
            memory_available_bytes: 0,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(metrics.total_memory_bytes(), 0);
        assert_eq!(metrics.memory_usage_percent(), 0.0);
    }

    #[test]
    fn test_high_load_cpu_boundary() {
        // Just below threshold
        let metrics_below = ComputeMetrics {
            cpu_usage_percent: 80.0,
            memory_usage_bytes: 1_000_000_000,
            memory_available_bytes: 9_000_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };
        assert!(!metrics_below.is_high_load());

        // Just above threshold
        let metrics_above = ComputeMetrics {
            cpu_usage_percent: 80.1,
            memory_usage_bytes: 1_000_000_000,
            memory_available_bytes: 9_000_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };
        assert!(metrics_above.is_high_load());
    }

    #[test]
    fn test_high_load_memory_boundary() {
        // Just below threshold (85%)
        let metrics_below = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 8_499_000_000,
            memory_available_bytes: 1_501_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };
        assert!(!metrics_below.is_high_load());

        // Just above threshold
        let metrics_above = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 8_501_000_000,
            memory_available_bytes: 1_499_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };
        assert!(metrics_above.is_high_load());
    }

    #[test]
    fn test_high_load_queued_jobs_boundary() {
        // At threshold
        let metrics_at = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 1_000_000_000,
            memory_available_bytes: 9_000_000_000,
            active_containers: 5,
            queued_jobs: 10,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };
        assert!(!metrics_at.is_high_load());

        // Above threshold
        let metrics_above = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 1_000_000_000,
            memory_available_bytes: 9_000_000_000,
            active_containers: 5,
            queued_jobs: 11,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };
        assert!(metrics_above.is_high_load());
    }

    #[test]
    fn test_health_status_degraded_cpu() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 85.0,
            memory_usage_bytes: 2_000_000_000,
            memory_available_bytes: 8_000_000_000,
            active_containers: 10,
            queued_jobs: 5,
            performance_score: 0.7,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), HealthStatus::Degraded);
    }

    #[test]
    fn test_health_status_degraded_memory() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 9_000_000_000,
            memory_available_bytes: 1_000_000_000,
            active_containers: 10,
            queued_jobs: 5,
            performance_score: 0.7,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), HealthStatus::Degraded);
    }

    #[test]
    fn test_health_status_unhealthy_cpu() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 96.0,
            memory_usage_bytes: 2_000_000_000,
            memory_available_bytes: 8_000_000_000,
            active_containers: 10,
            queued_jobs: 5,
            performance_score: 0.3,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_status_unhealthy_memory() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 9_600_000_000,
            memory_available_bytes: 400_000_000,
            active_containers: 10,
            queued_jobs: 5,
            performance_score: 0.3,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_status_boundary_95_cpu() {
        // Exactly at threshold
        let metrics = ComputeMetrics {
            cpu_usage_percent: 95.0,
            memory_usage_bytes: 2_000_000_000,
            memory_available_bytes: 8_000_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.7,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), HealthStatus::Degraded);
    }

    #[test]
    fn test_health_status_boundary_95_memory() {
        // Exactly at threshold
        let metrics = ComputeMetrics {
            cpu_usage_percent: 50.0,
            memory_usage_bytes: 9_500_000_000,
            memory_available_bytes: 500_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.7,
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(metrics.health_status(), HealthStatus::Degraded);
    }

    #[test]
    fn test_compute_metrics_serialization() -> SongbirdResult<()> {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 45.0,
            memory_usage_bytes: 2_000_000_000,
            memory_available_bytes: 6_000_000_000,
            active_containers: 5,
            queued_jobs: 2,
            performance_score: 0.85,
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&metrics).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {}", e))
        })?;
        assert!(json.contains("cpu_usage_percent"));
        assert!(json.contains("memory_usage_bytes"));
        Ok(())
    }

    #[test]
    fn test_health_status_serialization() -> SongbirdResult<()> {
        assert_eq!(
            serde_json::to_string(&HealthStatus::Healthy).map_err(|e| {
                SongbirdError::Serialization {
                    format: Some("JSON".to_string()),
                    message: format!("Serialization failed: {}", e),
                    debug_info: None,
                }
            })?,
            "\"Healthy\""
        );
        assert_eq!(
            serde_json::to_string(&HealthStatus::Degraded).map_err(|e| {
                SongbirdError::Serialization {
                    format: Some("JSON".to_string()),
                    message: format!("Serialization failed: {}", e),
                    debug_info: None,
                }
            })?,
            "\"Degraded\""
        );
        assert_eq!(
            serde_json::to_string(&HealthStatus::Unhealthy).map_err(|e| {
                SongbirdError::Serialization {
                    format: Some("JSON".to_string()),
                    message: format!("Serialization failed: {}", e),
                    debug_info: None,
                }
            })?,
            "\"Unhealthy\""
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_with_timeout() -> SongbirdResult<()> {
        let adapter = ComputeAdapter::new("http://compute-service:8080".to_string())
            .await
            .map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?
            .with_timeout(Duration::from_secs(25));
        assert_eq!(adapter.timeout, Duration::from_secs(25));
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_default_timeout() -> SongbirdResult<()> {
        let adapter =
            ComputeAdapter::new("http://compute-service:8080".to_string()).await.map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?;
        assert_eq!(adapter.timeout, Duration::from_secs(5));
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_endpoint_access() -> SongbirdResult<()> {
        let adapter =
            ComputeAdapter::new("http://test-compute:9000".to_string()).await.map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?;
        assert_eq!(adapter.endpoint(), "http://test-compute:9000");
        Ok(())
    }

    #[tokio::test]
    async fn test_adapter_debug_format() -> SongbirdResult<()> {
        let adapter =
            ComputeAdapter::new("http://compute:8080".to_string()).await.map_err(|e| {
                SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
            })?;
        let debug_str = format!("{:?}", adapter);
        assert!(debug_str.contains("ComputeAdapter"));
        assert!(debug_str.contains("http://compute:8080"));
        Ok(())
    }

    #[test]
    fn test_compute_metrics_perfect_conditions() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 10.0,
            memory_usage_bytes: 1_000_000_000,
            memory_available_bytes: 9_000_000_000,
            active_containers: 2,
            queued_jobs: 0,
            performance_score: 0.99,
            timestamp: chrono::Utc::now(),
        };

        assert!(!metrics.is_high_load());
        assert_eq!(metrics.health_status(), HealthStatus::Healthy);
    }

    #[test]
    fn test_compute_metrics_all_zero() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 0.0,
            memory_usage_bytes: 0,
            memory_available_bytes: 0,
            active_containers: 0,
            queued_jobs: 0,
            performance_score: 0.0,
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(metrics.total_memory_bytes(), 0);
        assert_eq!(metrics.memory_usage_percent(), 0.0);
        assert!(!metrics.is_high_load());
        assert_eq!(metrics.health_status(), HealthStatus::Healthy);
    }
}
