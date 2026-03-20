// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

// Module imports
//! Health Monitoring Module
//!
//! Comprehensive health monitoring system

use songbird_types::SongbirdResult;
type Result<T> = SongbirdResult<T>;
use std::sync::Arc;

/// Health monitor trait for implementing custom health monitoring
#[async_trait::async_trait]
pub trait HealthMonitor: Send + Sync {
    /// Get overall health status
    async fn get_health_status(&self) -> Result<HealthStatusDetails>;

    /// Get detailed health information
    async fn get_detailed_health(&self) -> Result<Vec<HealthCheckResult>>;

    /// Set health check thresholds
    async fn set_health_thresholds(&self, thresholds: HealthThresholds) -> Result<()>;
}

/// Coarse tri-state outcome for a single probe or rollup (maps to HTTP-style readiness).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// All checks within configured limits.
    Healthy,
    /// Some checks are slow or soft-failing; service may still accept traffic.
    Degraded,
    /// Hard failure or breach of thresholds; callers should treat as down.
    Unhealthy,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;

/// Single named probe outcome, suitable for aggregation into [`HealthStatusDetails`].
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Logical name of the check (e.g. `"database"` or `"disk"`).
    pub name: String,
    /// Outcome for this probe.
    pub status: HealthStatus,
    /// Human-readable detail or error text for operators.
    pub message: String,
    /// Round-trip latency of the probe in milliseconds.
    pub response_time_ms: u64,
}

/// Rollup view combining [`HealthState`], score, and optional JSON metadata for dashboards.
#[derive(Debug, Clone)]
pub struct HealthStatusDetails {
    /// Discrete lifecycle state (includes maintenance vs. unknown).
    pub state: HealthState,
    /// Normalized score in `0.0..=1.0` for sorting or UI gauges.
    pub score: f64,
    /// Count of checks that passed in the last evaluation.
    pub checks_passed: u32,
    /// Count of checks that failed in the last evaluation.
    pub checks_failed: u32,
    /// Wall-clock time of the last successful rollup.
    pub last_updated: std::time::SystemTime,
    /// Arbitrary structured fields (versions, build info, dependency snippets).
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Richer lifecycle state than [`HealthStatus`], including maintenance and unknown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthState {
    /// Fully within SLO; no action required.
    Healthy,
    /// Elevated risk or partial outage; may still serve degraded traffic.
    Degraded,
    /// Failing checks; do not route new work here without mitigation.
    Unhealthy,
    /// Catastrophic failure or data-loss risk; escalate immediately.
    Critical,
    /// Insufficient data to classify (startup, probes not yet run).
    Unknown,
    /// Intentionally taken out of rotation (drain, upgrade window).
    Maintenance,
}

/// Immutable snapshot of a past evaluation for auditing or trend charts.
pub struct HealthRecord {
    /// When this rollup was recorded.
    pub timestamp: std::time::SystemTime,
    /// Aggregate [`HealthState`] at that time.
    pub status: HealthState,
    /// Individual probe results included in the rollup.
    pub checks: Vec<HealthCheckResult>,
    /// Optional end-to-end latency of the full evaluation pass.
    pub response_time: Option<std::time::Duration>,
}

/// Tunable limits passed to [`HealthMonitor::set_health_thresholds`] for SLO-driven alerts.
#[allow(
    clippy::struct_field_names,
    reason = "threshold field names match external health check schema"
)]
pub struct HealthThresholds {
    /// Maximum acceptable probe latency before marking degraded.
    pub response_time_threshold: std::time::Duration,
    /// Fraction of failed requests (0.0–1.0) that flips status to unhealthy.
    pub error_rate_threshold: f64,
    /// CPU utilization ratio (0.0–1.0) that triggers alerts.
    pub cpu_threshold: f64,
    /// Memory utilization ratio (0.0–1.0) that triggers alerts.
    pub memory_threshold: f64,
    /// Disk utilization ratio (0.0–1.0) that triggers alerts.
    pub disk_threshold: f64,
    /// Consecutive probe failures before escalating to unhealthy.
    pub failure_count_threshold: u32,
}

/// Owns a list of [`HealthCheckAsync`] probes and runs them via [`check_all`](Self::check_all).
pub struct HealthChecker {
    checks: Vec<Arc<dyn HealthCheckAsync + Send + Sync>>,
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthChecker {
    /// Builds an empty runner; register probes with [`add_check`](Self::add_check).
    #[must_use]
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }

    /// Registers an async probe; order is preserved when running [`check_all`](Self::check_all).
    pub fn add_check(&mut self, check: Arc<dyn HealthCheckAsync + Send + Sync>) {
        self.checks.push(check);
    }

    /// Runs every registered probe sequentially and collects [`HealthCheckResult`] rows.
    pub async fn check_all(&self) -> Vec<HealthCheckResult> {
        let mut results = Vec::new();

        for check in &self.checks {
            match check.check().await {
                Ok(result) => results.push(result),
                Err(err) => results.push(HealthCheckResult {
                    name: "Unknown".to_string(),
                    status: HealthStatus::Unhealthy,
                    message: format!("Check failed: {err:?}"),
                    response_time_ms: 0,
                }),
            }
        }

        results
    }
}

/// Health check trait for async checks
#[async_trait::async_trait]
pub trait HealthCheckAsync: Send + Sync {
    /// Perform the health check
    async fn check(&self) -> Result<HealthCheckResult>;
}
