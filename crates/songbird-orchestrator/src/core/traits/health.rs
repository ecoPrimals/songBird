//! Health Monitoring Trait
//!
//! Provides health checking capabilities for services
//!
//! **MIGRATION COMPLETE**: Now uses canonical HealthStatus from songbird-types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
/// 
/// **CONSOLIDATED**: Now uses canonical definition from songbird-discovery
/// (November 10, 2025 - Unification effort)
pub use songbird_discovery::traits::health::HealthCheckConfig;

/// **CONSOLIDATED**: Now uses canonical definition from songbird-discovery
/// 
/// The canonical HealthMonitor trait provides full lifecycle management:
/// - Health checking (check_health, get_health_status)
/// - Service registration (register, unregister)  
/// - Monitoring control (start_monitoring, stop_monitoring)
/// - Configuration updates (update_config)
///
/// (November 10, 2025 - Trait Unification)
pub use songbird_discovery::traits::health::HealthMonitor;

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
