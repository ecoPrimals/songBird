//! Health Check Types and Status
//!
//! **CANONICAL**: Centralized health checking for the entire Songbird ecosystem

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// **CANONICAL**: Health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalHealthStatus {
    /// System is healthy and operating normally
    Healthy,
    /// System is degraded but still functional
    Degraded,
    /// System is unhealthy and may not be functional
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

impl Default for CanonicalHealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl fmt::Display for CanonicalHealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status_str = match self {
            Self::Healthy => "Healthy",
            Self::Degraded => "Degraded",
            Self::Unhealthy => "Unhealthy",
            Self::Unknown => "Unknown",
        };
        write!(f, "{status_str}")
    }
}

/// **CANONICAL**: Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalHealthCheck {
    /// Overall health status
    pub status: CanonicalHealthStatus,
    /// Optional descriptive message
    pub message: Option<String>,
    /// Health metrics
    pub metrics: HashMap<String, f64>,
    /// Component-specific health status
    pub components: HashMap<String, CanonicalHealthStatus>,
}

impl Default for CanonicalHealthCheck {
    fn default() -> Self {
        Self {
            status: CanonicalHealthStatus::Unknown,
            message: None,
            metrics: HashMap::new(),
            components: HashMap::new(),
        }
    }
}

impl CanonicalHealthCheck {
    /// Create a healthy status
    #[must_use]
    pub fn healthy() -> Self {
        Self {
            status: CanonicalHealthStatus::Healthy,
            message: Some("All systems operational".to_string()),
            metrics: HashMap::new(),
            components: HashMap::new(),
        }
    }

    /// Create a degraded status with message
    #[must_use]
    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: CanonicalHealthStatus::Degraded,
            message: Some(message.into()),
            metrics: HashMap::new(),
            components: HashMap::new(),
        }
    }

    /// Create an unhealthy status with message
    #[must_use]
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: CanonicalHealthStatus::Unhealthy,
            message: Some(message.into()),
            metrics: HashMap::new(),
            components: HashMap::new(),
        }
    }

    /// Add a metric to the health check
    pub fn with_metric(&mut self, key: impl Into<String>, metric_value: f64) -> &mut Self {
        self.metrics.insert(key.into(), metric_value);
        self
    }

    /// Add a component status
    pub fn with_component(
        &mut self,
        name: impl Into<String>,
        status: CanonicalHealthStatus,
    ) -> &mut Self {
        self.components.insert(name.into(), status);
        self
    }

    /// Check if the overall status is healthy
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.status == CanonicalHealthStatus::Healthy
    }
}

/// **CANONICAL**: Health configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalHealthConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check endpoint path
    pub endpoint: String,
    /// Health check interval in seconds
    pub check_interval_seconds: u64,
    /// Health check timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for CanonicalHealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "/health".to_string(),
            check_interval_seconds: 30,
            timeout_seconds: 5,
        }
    }
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_default() {
        let status = CanonicalHealthStatus::default();
        assert_eq!(status, CanonicalHealthStatus::Unknown);
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(CanonicalHealthStatus::Healthy.to_string(), "Healthy");
        assert_eq!(CanonicalHealthStatus::Degraded.to_string(), "Degraded");
        assert_eq!(CanonicalHealthStatus::Unhealthy.to_string(), "Unhealthy");
        assert_eq!(CanonicalHealthStatus::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_health_status_equality() {
        assert_eq!(CanonicalHealthStatus::Healthy, CanonicalHealthStatus::Healthy);
        assert_ne!(CanonicalHealthStatus::Healthy, CanonicalHealthStatus::Degraded);
    }

    #[test]
    fn test_health_check_default() {
        let check = CanonicalHealthCheck::default();
        assert_eq!(check.status, CanonicalHealthStatus::Unknown);
        assert!(check.message.is_none());
        assert!(check.metrics.is_empty());
        assert!(check.components.is_empty());
    }

    #[test]
    fn test_health_check_healthy() {
        let check = CanonicalHealthCheck::healthy();
        assert_eq!(check.status, CanonicalHealthStatus::Healthy);
        assert_eq!(check.message, Some("All systems operational".to_string()));
        assert!(check.is_healthy());
    }

    #[test]
    fn test_health_check_degraded() {
        let check = CanonicalHealthCheck::degraded("Some services slow");
        assert_eq!(check.status, CanonicalHealthStatus::Degraded);
        assert_eq!(check.message, Some("Some services slow".to_string()));
        assert!(!check.is_healthy());
    }

    #[test]
    fn test_health_check_unhealthy() {
        let check = CanonicalHealthCheck::unhealthy("Database down");
        assert_eq!(check.status, CanonicalHealthStatus::Unhealthy);
        assert_eq!(check.message, Some("Database down".to_string()));
        assert!(!check.is_healthy());
    }

    #[test]
    fn test_health_check_with_metrics() {
        let mut check = CanonicalHealthCheck::healthy();
        check.with_metric("cpu_usage", 45.5);
        check.with_metric("memory_usage", 72.3);

        assert_eq!(check.metrics.get("cpu_usage"), Some(&45.5));
        assert_eq!(check.metrics.get("memory_usage"), Some(&72.3));
        assert_eq!(check.metrics.len(), 2);
    }

    #[test]
    fn test_health_check_with_components() {
        let mut check = CanonicalHealthCheck::healthy();
        check.with_component("database", CanonicalHealthStatus::Healthy);
        check.with_component("cache", CanonicalHealthStatus::Degraded);

        assert_eq!(check.components.get("database"), Some(&CanonicalHealthStatus::Healthy));
        assert_eq!(check.components.get("cache"), Some(&CanonicalHealthStatus::Degraded));
        assert_eq!(check.components.len(), 2);
    }

    #[test]
    fn test_health_check_fluent_api() {
        let mut check = CanonicalHealthCheck::healthy();
        check
            .with_metric("latency_ms", 15.2)
            .with_metric("throughput", 1000.0)
            .with_component("api", CanonicalHealthStatus::Healthy)
            .with_component("worker", CanonicalHealthStatus::Healthy);

        assert_eq!(check.metrics.len(), 2);
        assert_eq!(check.components.len(), 2);
        assert!(check.is_healthy());
    }

    #[test]
    fn test_health_config_default() {
        let config = CanonicalHealthConfig::default();
        assert!(config.enabled);
        assert_eq!(config.endpoint, "/health");
        assert_eq!(config.check_interval_seconds, 30);
        assert_eq!(config.timeout_seconds, 5);
    }

    #[test]
    fn test_health_check_serialization() {
        let check = CanonicalHealthCheck::healthy();
        let json = serde_json::to_string(&check).unwrap();
        assert!(json.contains("Healthy"));
        assert!(json.contains("All systems operational"));
    }

    #[test]
    fn test_health_status_serialization() {
        let status = CanonicalHealthStatus::Healthy;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: CanonicalHealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_health_check_is_healthy() {
        assert!(CanonicalHealthCheck::healthy().is_healthy());
        assert!(!CanonicalHealthCheck::degraded("issue").is_healthy());
        assert!(!CanonicalHealthCheck::unhealthy("down").is_healthy());
        assert!(!CanonicalHealthCheck::default().is_healthy());
    }
}
