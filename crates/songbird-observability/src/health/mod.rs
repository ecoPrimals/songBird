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

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod types_tests;

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub response_time_ms: u64,
}

/// Health status details
#[derive(Debug, Clone)]
pub struct HealthStatusDetails {
    pub state: HealthState,
    pub score: f64,
    pub checks_passed: u32,
    pub checks_failed: u32,
    pub last_updated: std::time::SystemTime,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Health state enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
    Critical,
    Unknown,
    Maintenance,
}

/// Health record for history
pub struct HealthRecord {
    pub timestamp: std::time::SystemTime,
    pub status: HealthState,
    pub checks: Vec<HealthCheckResult>,
    pub response_time: Option<std::time::Duration>,
}

/// Health thresholds
#[allow(clippy::struct_field_names)]
pub struct HealthThresholds {
    pub response_time_threshold: std::time::Duration,
    pub error_rate_threshold: f64,
    pub cpu_threshold: f64,
    pub memory_threshold: f64,
    pub disk_threshold: f64,
    pub failure_count_threshold: u32,
}

/// Collection of health checks
pub struct HealthChecker {
    checks: Vec<Arc<dyn HealthCheckAsync + Send + Sync>>,
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthChecker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
        }
    }

    pub fn add_check(&mut self, check: Arc<dyn HealthCheckAsync + Send + Sync>) {
        self.checks.push(check);
    }

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
