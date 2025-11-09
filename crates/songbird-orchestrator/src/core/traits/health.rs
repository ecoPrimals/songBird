//! Health Monitoring Trait
//!
//! Provides health checking capabilities for services
//!
//! **MIGRATION COMPLETE**: Now uses canonical HealthStatus from songbird-types

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_config::constants::health::{DEFAULT_CHECK_INTERVAL, DEFAULT_CHECK_TIMEOUT};
use std::collections::HashMap;
use std::time::Duration;

// **CANONICAL**: Use unified health status and result types from songbird-types
pub use songbird_types::health::CanonicalHealthStatus as HealthStatus;
pub use songbird_types::{SongbirdError, SongbirdResult};

// For internal module use, Result is SongbirdResult
type Result<T> = SongbirdResult<T>;

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Service identifier
    pub service_id: String,

    /// Current health status
    pub status: HealthStatus,

    /// Optional message providing details
    pub message: Option<String>,

    /// Timestamp when check was performed
    pub timestamp: DateTime<Utc>,

    /// Additional details about the health check
    #[serde(default)]
    pub details: HashMap<String, serde_json::Value>,
}

impl HealthCheckResult {
    /// Create a healthy result
    #[must_use]
    pub fn healthy(service_id: impl Into<String>) -> Self {
        Self {
            service_id: service_id.into(),
            status: HealthStatus::Healthy,
            message: None,
            timestamp: Utc::now(),
            details: HashMap::new(),
        }
    }

    /// Create an unhealthy result with message
    #[must_use]
    pub fn unhealthy(service_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            service_id: service_id.into(),
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            timestamp: Utc::now(),
            details: HashMap::new(),
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Interval between health checks
    #[serde(default = "default_interval")]
    pub interval: Duration,

    /// Timeout for each health check
    #[serde(default = "default_timeout")]
    pub timeout: Duration,

    /// Number of retries before marking unhealthy
    #[serde(default = "default_retries")]
    pub retries: u32,

    /// Optional custom endpoint for health checks
    pub endpoint: Option<String>,

    /// Whether health checks are enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_interval() -> Duration {
    DEFAULT_CHECK_INTERVAL
}

fn default_timeout() -> Duration {
    DEFAULT_CHECK_TIMEOUT
}

fn default_retries() -> u32 {
    3
}

fn default_enabled() -> bool {
    true
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: DEFAULT_CHECK_INTERVAL,
            timeout: DEFAULT_CHECK_TIMEOUT,
            retries: 3,
            endpoint: None,
            enabled: true,
        }
    }
}

/// Health monitoring trait
#[async_trait]
pub trait HealthMonitor: Send + Sync {
    /// Perform a health check on a service
    async fn check_health(&self, service_id: &str) -> Result<HealthCheckResult>;

    /// Get the current health status
    async fn get_status(&self, service_id: &str) -> Result<HealthStatus> {
        Ok(self.check_health()service_id).await?.status)
    }

    /// Check if service is operational (healthy or degraded)
    async fn is_operational(&self, service_id: &str) -> bool {
        self.get_status(service_id)
            .await
            .map(|s| s.is_operational())
            .unwrap_or(false)
    }
}

/// Detailed health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedHealthInfo {
    /// Overall health status
    pub overall_status: HealthStatus,

    /// Component-level health status
    pub components: HashMap<String, HealthStatus>,

    /// Health metrics
    pub metrics: HashMap<String, f64>,

    /// Last check timestamp
    pub last_check: DateTime<Utc>,

    /// Uptime in seconds
    pub uptime_seconds: u64,
}

impl DetailedHealthInfo {
    /// Create healthy detailed info
    #[must_use]
    pub fn healthy() -> Self {
        Self {
            overall_status: HealthStatus::Healthy,
            components: HashMap::new(),
            metrics: HashMap::new(),
            last_check: Utc::now(),
            uptime_seconds: 0,
        }
    }

    /// Add a component status
    pub fn with_component(mut self, name: impl Into<String>, status: HealthStatus) -> Self {
        self.components.insert(name.into(), status);
        // Update overall status based on worst component
        if status < self.overall_status {
            self.overall_status = status;
        }
        self
    }

    /// Add a metric
    pub fn with_metric(mut self, name: impl Into<String>, value: f64) -> Self {
        self.metrics.insert(name.into(), value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_result_creation() {
        let healthy = HealthCheckResult::healthy("test-service");
        assert_eq!(healthy.status, HealthStatus::Healthy)
        assert_eq!(healthy.service_id, "test-service");

        let unhealthy = HealthCheckResult::unhealthy("test-service", "Database connection failed");
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy)
        assert!(unhealthy.message.is_some());
    }

    #[test]
    fn test_default_config() {
        let config = HealthCheckConfig::default();
        assert!(config.enabled)
        assert_eq!(config.retries, 3)
    }

    #[test]
    fn test_detailed_health_info() {
        let info = DetailedHealthInfo::healthy()
            .with_component("database", HealthStatus::Healthy,
            .with_component("cache", HealthStatus::Degraded)
            .with_metric("cpu_usage", 45.2);

        // Overall status should be degraded (worst component)
        assert_eq!(info.overall_status, HealthStatus::Degraded)
        assert_eq!(info.components.len(), 2);
        assert_eq!(info.metrics.len(), 1);
    }
}
