//! Health Monitoring Module
//!
//! Comprehensive health monitoring system

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_util::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::errors::Result;

/// Health monitor trait
#[async_trait]
pub trait HealthMonitor: Send + Sync {
    /// Register a service for health monitoring
    async fn register_service(
        &self,
        service_id: &str,
        checks: Vec<Box<dyn HealthCheck>>,
    ) -> Result<()>;

    /// Check health of a specific service
    async fn check_health(&self, service_id: &str) -> Result<HealthStatus>;

    /// Get health history for a service
    async fn get_health_history(
        &self,
        service_id: &str,
        duration: Duration,
    ) -> Result<Vec<HealthRecord>>;

    /// Watch health status changes
    async fn watch_health(&self, service_id: &str) -> impl Stream<Item = HealthStatus>;

    /// Set health thresholds
    async fn set_health_thresholds(
        &self,
        service_id: &str,
        thresholds: HealthThresholds,
    ) -> Result<()>;
}

/// Health check trait
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform the health check
    async fn check(&self) -> Result<HealthCheckResult>;

    /// Get the name of this health check
    fn name(&self) -> &str;

    /// Get the description
    fn description(&self) -> &str;

    /// Get the timeout for this check
    fn timeout(&self) -> Duration;

    /// Get the interval for this check
    fn interval(&self) -> Duration;
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub status: HealthState,
    pub message: String,
    pub metrics: HashMap<String, f64>,
    pub timestamp: DateTime<Utc>,
    pub duration: Duration,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub service_id: String,
    pub overall_status: HealthState,
    pub checks: Vec<HealthCheckResult>,
    pub last_updated: DateTime<Utc>,
    pub uptime: Duration,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Health state enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthState {
    Healthy,
    Degraded { reason: String, severity: u8 },
    Unhealthy { reason: String },
    Unknown,
    Maintenance,
}

/// Health record for history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRecord {
    pub timestamp: DateTime<Utc>,
    pub status: HealthStatus,
}

/// Health thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthThresholds {
    pub degraded_threshold: u8,  // 0-100
    pub unhealthy_threshold: u8, // 0-100
    pub recovery_threshold: u8,  // 0-100
    pub check_interval: Duration,
    pub failure_count_threshold: u32,
}
