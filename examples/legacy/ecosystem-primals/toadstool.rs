// SPDX-License-Identifier: AGPL-3.0-only
//! `ToadStool` Compute Metrics Adapter
//!
//! **LEGACY EXAMPLE**: This example uses `reqwest` for demonstration purposes.
//! For TRUE Pure Rust production code, use `IpcHttpClient` instead (no C dependencies).
//!
//! See modern implementation:
//! - `crates/songbird-http-client/examples/ipc_http_client_demo.rs`
//! - Migration guide: `ecoPrimals/sessions/feb-2026/reqwest-removal/`
//!
//! ---
//!
//! Ingests compute metrics from `ToadStool` primal via HTTP endpoints.
//! This adapter is capability-based and works with any service providing
//! compute metrics in the expected format.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_types::{SongbirdError, SongbirdResult};
use std::time::Duration;
use tracing::{debug, warn};

/// Compute metrics from `ToadStool` or any compute primal
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
    pub fn total_memory_bytes(&self) -> u64 {
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

/// Adapter for ingesting compute metrics from any compute primal
pub struct ToadStoolMetricsAdapter {
    /// Endpoint URL for the compute service
    endpoint: String,
    /// HTTP client for requests
    client: reqwest::Client,
    /// Request timeout
    timeout: Duration,
}

impl ToadStoolMetricsAdapter {
    /// Create a new `ToadStool` metrics adapter with default endpoint from configuration
    ///
    /// Uses environment variables for endpoint configuration:
    /// - `TOADSTOOL_ENDPOINT` - Direct endpoint override
    /// - `PRIMAL_TOADSTOOL_ENDPOINT` - Alternative format
    /// - Falls back to `$SONGBIRD_HOST:$TOADSTOOL_PORT`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use songbird_universal::adapters::ToadStoolMetricsAdapter;
    ///
    /// // Uses environment-configured endpoint
    /// let adapter = ToadStoolMetricsAdapter::new_default().unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new_default() -> SongbirdResult<Self> {
        let endpoint = songbird_config::endpoints::get_primal_endpoint("toadstool");
        Self::new(endpoint)
    }

    /// Create a new `ToadStool` metrics adapter with custom endpoint
    ///
    /// # Arguments
    ///
    /// * `endpoint` - Base URL of the compute service
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use songbird_universal::adapters::ToadStoolMetricsAdapter;
    ///
    /// // Custom endpoint
    /// let adapter = ToadStoolMetricsAdapter::new(format!("http://compute-service:{}", 
    ///     songbird_config::defaults::ports::orchestrator_port())).unwrap();
    ///
    /// // Or use default from configuration
    /// let adapter = ToadStoolMetricsAdapter::new_default().unwrap();
    /// ```
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

    /// Collect compute metrics from the service
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Network request fails
    /// - Service returns non-success status
    /// - Response cannot be parsed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use songbird_universal::adapters::ToadStoolMetricsAdapter;
    /// # async fn example() -> songbird_types::SongbirdResult<()> {
    /// // Test uses localhost - acceptable for unit tests
    /// let adapter = ToadStoolMetricsAdapter::new(format!("http://localhost:{}", 
    ///     songbird_config::defaults::ports::orchestrator_port())).unwrap();
    /// let metrics = adapter.collect_metrics().await?;
    /// println!("CPU usage: {}%", metrics.cpu_usage_percent);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn collect_metrics(&self) -> SongbirdResult<ComputeMetrics> {
        let url = format!("{}/metrics/compute", self.endpoint);

        debug!("Collecting compute metrics from: {}", url);

        let response = self.client.get(&url).timeout(self.timeout).send().await.map_err(|e| {
            warn!("Failed to reach compute service: {e}");
            songbird_types::SongbirdError::network(format!("Failed to reach compute service: {e}"))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            warn!("Compute service returned error status: {}", status);
            return Err(songbird_types::SongbirdError::service(
                "compute",
                format!("HTTP {status}: Metrics unavailable"),
            ));
        }

        let mut metrics: ComputeMetrics = response.json().await.map_err(|e| {
            warn!("Failed to parse compute metrics: {e}");
            songbird_types::SongbirdError::service(
                "compute",
                format!("Failed to parse compute metrics: {e}"),
            )
        })?;

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

/// Trait for compute metrics collection
#[async_trait]
pub trait ComputeMetricsProvider {
    /// Collect current compute metrics
    async fn collect_compute_metrics(&self) -> SongbirdResult<ComputeMetrics>;

    /// Check compute service health
    async fn check_compute_health(&self) -> SongbirdResult<HealthStatus> {
        let metrics = self.collect_compute_metrics().await?;
        Ok(metrics.health_status())
    }
}

#[async_trait]
impl ComputeMetricsProvider for ToadStoolMetricsAdapter {
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
            memory_usage_bytes: 2_000_000_000,     // 2GB
            memory_available_bytes: 6_000_000_000, // 6GB
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
    fn test_high_load_detection() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 96.0, // Above 95% threshold
            memory_usage_bytes: 7_600_000_000,
            memory_available_bytes: 400_000_000,
            active_containers: 20,
            queued_jobs: 15,
            performance_score: 0.45,
            timestamp: chrono::Utc::now(),
        };

        assert!(metrics.is_high_load());
        assert_eq!(metrics.health_status(), HealthStatus::Unhealthy);
    }

    #[test]
    fn test_degraded_status() {
        let metrics = ComputeMetrics {
            cpu_usage_percent: 85.0,
            memory_usage_bytes: 6_000_000_000,
            memory_available_bytes: 2_000_000_000,
            active_containers: 15,
            queued_jobs: 8,
            performance_score: 0.60,
            timestamp: chrono::Utc::now(),
        };

        assert!(metrics.is_high_load());
        assert_eq!(metrics.health_status(), HealthStatus::Degraded);
    }

    #[test]
    fn test_adapter_creation() {
        // Test uses localhost - acceptable for unit tests
        let port = songbird_config::defaults::ports::orchestrator_port();
        let endpoint = format!("http://localhost:{}", port);
        let adapter = ToadStoolMetricsAdapter::new(endpoint.clone())
            .expect("Test: adapter creation should succeed");
        assert_eq!(
            adapter.endpoint(), // Test uses localhost - acceptable for unit tests
            &endpoint
        );
    }

    #[test]
    fn test_adapter_with_timeout() {
        let adapter = ToadStoolMetricsAdapter::new(
            // Test uses localhost - acceptable for unit tests
            format!("http://localhost:{}", songbird_config::defaults::ports::orchestrator_port()),
        )
        .expect("Test: adapter creation should succeed")
        .with_timeout(Duration::from_secs(10));
        assert_eq!(adapter.timeout, Duration::from_secs(10));
    }
}
