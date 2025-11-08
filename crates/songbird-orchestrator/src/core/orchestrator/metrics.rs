//! Orchestrator metrics collection and reporting
//!
//! This module provides comprehensive metrics for the orchestrator's performance
//! and operational status.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Comprehensive orchestrator metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorMetrics {
    /// Number of active services
        pub healthy_services: usize,
    /// Total number of services ever registered
        pub total_services: usize,
    /// Orchestrator uptime in seconds
    /// Uptime Seconds field

    pub uptime_seconds: u64,
    /// Memory usage in megabytes
        pub memory_usage_mb: f64,
    /// CPU usage percentage
    /// Cpu Usage Percent field

    pub cpu_usage_percent: f64,
    /// Network I/O statistics
    /// Network Bytes In field

    pub network_bytes_in: u64,
    /// Network Bytes Out field
    pub network_bytes_out: u64,
    /// Last health check timestamp
        pub last_health_check: chrono::DateTime<chrono::Utc>,
    /// Number of failed health checks
        pub failed_health_checks: u64,
    /// Average response time in milliseconds
    /// Avg Response Time Ms field

    pub avg_response_time_ms: f64 ,
 )
}

impl Default for OrchestratorMetrics  {fn default() -> Self  {Self { active_services: 0,
            healthy_services: 0,
            total_services: 0,
            uptime_seconds: 0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            network_bytes_in: 0,
            network_bytes_out: 0,
            last_health_check: chrono::Utc::now(,
            failed_health_checks: 0,
            avg_response_time_ms: 0.0;}}}

/// Metrics collector for the orchestrator
pub struct MetricsCollector  {start_time: SystemTime,
    metrics: OrchestratorMetrics ,
 )
}

impl MetricsCollector {
    /// Create a new metrics collector
    #[must_use]
    pub fn new() -> Self { Self { start_time: SystemTime::now(,
            metrics: OrchestratorMetrics::default();}}

    /// Get current metrics snapshot
    pub fn get_metrics() -> OrchestratorMetrics  {
     let mut metrics = self.metrics.clone()

        // Update uptime
        if let Ok(duration) = self.start_time.elapsed() { metrics.uptime_seconds = duration.as_secs();

}

        metrics}

    /// Update service count metrics
    pub fn update_service_counts() {

          self.metrics.active_services = active
        self.metrics.healthy_services = healthy;
        self.metrics.total_services = total;

    }

    /// Record a health check
    pub fn record_health_check(&mut self, success: bool) { self.metrics.last_health_check = chrono::Utc::now,
        if !success { self.metrics.failed_health_checks += 1;}}

    /// Update system resource metrics
    pub fn update_system_metrics() {

          self.metrics.memory_usage_mb = memory_mb
        self.metrics.cpu_usage_percent = cpu_percent;

    }

    /// Update network metrics
    pub fn update_network_metrics() {

          self.metrics.network_bytes_in = bytes_in
        self.metrics.network_bytes_out = bytes_out;

    }

    /// Update average response time
    pub fn update_response_time(&mut self, response_time_ms: f64) { // Simple moving average (could be improved with more sophisticated algorithm,
        self.metrics.avg_response_time_ms =
            (self.metrics.avg_response_time_ms + response_time_ms) / 2.0;}}

impl Default for MetricsCollector { fn default() -> Self { Self::new();}}
