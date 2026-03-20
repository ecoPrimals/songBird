// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Mock `ToadStool` Compute Primal
//!
//! Provides HTTP endpoints that simulate `ToadStool`'s compute metrics and workload management.

#![allow(clippy::unused_async)]

use super::common::{HealthStatus, MockPrimalServer, MockServerState};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// `ToadStool` compute metrics
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
}

impl Default for ComputeMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 15.0,
            memory_usage_bytes: 1_000_000_000,     // 1GB
            memory_available_bytes: 7_000_000_000, // 7GB
            active_containers: 3,
            queued_jobs: 0,
            performance_score: 0.95,
        }
    }
}

/// Mock `ToadStool` compute server
#[derive(Debug, Clone)]
pub struct MockToadStool {
    state: Arc<MockServerState>,
    metrics: Arc<RwLock<ComputeMetrics>>,
}

impl MockToadStool {
    /// Create a new mock `ToadStool` server
    ///
    /// The server is not started until `start()` is called.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(MockServerState::new(0)),
            metrics: Arc::new(RwLock::new(ComputeMetrics::default())),
        }
    }

    /// Start the mock server on a random available port
    ///
    /// Returns the port number the server is listening on.
    ///
    /// # Errors
    ///
    /// Currently never returns an error, but signature allows for future error cases.
    pub async fn start(&mut self) -> Result<u16, Box<dyn std::error::Error>> {
        // In a real implementation, this would start an actual HTTP server
        // For now, we'll assign a random port for testing purposes
        let port = fastrand::u16(10000..60000);
        self.state = Arc::new(MockServerState::new(port));
        Ok(port)
    }

    /// Stop the mock server
    pub async fn stop(&self) {
        // Server cleanup would happen here
    }

    /// Set CPU usage percentage
    ///
    /// # Panics
    ///
    /// Panics if the internal metrics lock is poisoned.
    pub fn set_cpu_usage(&self, percent: f64) {
        let mut metrics = self.metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.cpu_usage_percent = percent.clamp(0.0, 100.0);
    }

    /// Set memory usage in bytes
    ///
    /// # Panics
    ///
    /// Panics if the internal metrics lock is poisoned.
    pub fn set_memory_usage(&self, bytes: u64) {
        let mut metrics = self.metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.memory_usage_bytes = bytes;
    }

    /// Set memory available in bytes
    ///
    /// # Panics
    ///
    /// Panics if the internal metrics lock is poisoned.
    pub fn set_memory_available(&self, bytes: u64) {
        let mut metrics = self.metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.memory_available_bytes = bytes;
    }

    /// Set number of active containers
    ///
    /// # Panics
    ///
    /// Panics if the internal metrics lock is poisoned.
    pub fn set_active_containers(&self, count: u32) {
        let mut metrics = self.metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.active_containers = count;
    }

    /// Set number of queued jobs
    ///
    /// # Panics
    ///
    /// Panics if the internal metrics lock is poisoned.
    pub fn set_queued_jobs(&self, count: u32) {
        let mut metrics = self.metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.queued_jobs = count;
    }

    /// Set performance score
    ///
    /// # Panics
    ///
    /// Panics if the internal metrics lock is poisoned.
    pub fn set_performance_score(&self, score: f64) {
        let mut metrics = self.metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.performance_score = score.clamp(0.0, 1.0);
    }

    /// Get current metrics
    ///
    /// # Panics
    ///
    /// Panics if the internal metrics lock is poisoned.
    #[must_use]
    pub fn get_metrics(&self) -> ComputeMetrics {
        self.metrics
            .read()
            .unwrap_or_else(|poisoned| {
                tracing::warn!("RwLock poisoned in test mock, recovering");
                poisoned.into_inner()
            })
            .clone()
    }

    /// Simulate high load scenario
    ///
    /// # Panics
    ///
    /// Panics if the internal metrics lock is poisoned.
    pub fn simulate_high_load(&self) {
        let mut metrics = self.metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.cpu_usage_percent = 95.0;
        metrics.memory_usage_bytes = 7_500_000_000; // 7.5GB
        metrics.memory_available_bytes = 500_000_000; // 500MB
        metrics.active_containers = 20;
        metrics.queued_jobs = 15;
        metrics.performance_score = 0.45;
        drop(metrics);
        self.state.set_health(HealthStatus::Degraded);
    }

    /// Simulate healthy idle scenario
    ///
    /// # Panics
    ///
    /// Panics if the internal metrics lock is poisoned.
    pub fn simulate_idle(&self) {
        let mut metrics = self.metrics.write().unwrap_or_else(|poisoned| {
            tracing::warn!("RwLock poisoned in test mock, recovering");
            poisoned.into_inner()
        });
        metrics.cpu_usage_percent = 5.0;
        metrics.memory_usage_bytes = 500_000_000; // 500MB
        metrics.memory_available_bytes = 7_500_000_000; // 7.5GB
        metrics.active_containers = 1;
        metrics.queued_jobs = 0;
        metrics.performance_score = 0.98;
        drop(metrics);
        self.state.set_health(HealthStatus::Healthy);
    }
}

impl Default for MockToadStool {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPrimalServer for MockToadStool {
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
    async fn test_mock_toadstool_creation() {
        let mock = MockToadStool::new();
        assert_eq!(mock.get_health(), HealthStatus::Healthy);

        let metrics = mock.get_metrics();
        #[allow(clippy::float_cmp)]
        {
            assert_eq!(metrics.cpu_usage_percent, 15.0);
        }
    }

    #[tokio::test]
    async fn test_mock_toadstool_metrics_modification() {
        let mock = MockToadStool::new();

        mock.set_cpu_usage(75.0);
        mock.set_active_containers(10);

        let metrics = mock.get_metrics();
        #[allow(clippy::float_cmp)]
        {
            assert_eq!(metrics.cpu_usage_percent, 75.0);
        }
        assert_eq!(metrics.active_containers, 10);
    }

    #[tokio::test]
    async fn test_mock_toadstool_scenarios() {
        let mock = MockToadStool::new();

        // Test high load scenario
        mock.simulate_high_load();
        let metrics = mock.get_metrics();
        assert!(metrics.cpu_usage_percent > 90.0);
        assert_eq!(mock.get_health(), HealthStatus::Degraded);

        // Test idle scenario
        mock.simulate_idle();
        let metrics = mock.get_metrics();
        assert!(metrics.cpu_usage_percent < 10.0);
        assert_eq!(mock.get_health(), HealthStatus::Healthy);
    }

    // ========== NEW TESTS (5 tests to improve coverage) ==========

    #[tokio::test]
    async fn test_toadstool_server_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
        let mut mock = MockToadStool::new();
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
    async fn test_compute_metrics_default() {
        let mock = MockToadStool::new();
        let metrics = mock.get_metrics();
        assert!((metrics.cpu_usage_percent - 15.0).abs() < 0.001);
        assert_eq!(metrics.memory_usage_bytes, 1_000_000_000);
        assert_eq!(metrics.active_containers, 3);
        assert_eq!(metrics.queued_jobs, 0);
    }

    #[tokio::test]
    async fn test_container_management() {
        let mock = MockToadStool::new();
        let initial_count = mock.get_metrics().active_containers;

        mock.set_active_containers(10);
        assert_eq!(mock.get_metrics().active_containers, 10);

        mock.set_active_containers(0);
        assert_eq!(mock.get_metrics().active_containers, 0);
    }

    #[tokio::test]
    async fn test_health_status_transitions() {
        let mock = MockToadStool::new();
        assert_eq!(mock.get_health(), HealthStatus::Healthy);

        mock.set_health(HealthStatus::Degraded);
        assert_eq!(mock.get_health(), HealthStatus::Degraded);

        mock.set_health(HealthStatus::Unhealthy);
        assert_eq!(mock.get_health(), HealthStatus::Unhealthy);
    }

    #[test]
    fn test_toadstool_default_trait() {
        let mock = MockToadStool::default();
        assert_eq!(mock.port(), 0);
        assert_eq!(mock.get_health(), HealthStatus::Healthy);
    }
}
