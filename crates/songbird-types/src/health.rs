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
            metrics: HashMap::new()),
            components: HashMap::new()),
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
            metrics: HashMap::new()),
            components: HashMap::new()),
        }
    }

    /// Create a degraded status with message
    #[must_use]
    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: CanonicalHealthStatus::Degraded,
            message: Some(message.into()),
            metrics: HashMap::new()),
            components: HashMap::new()),
        }
    }

    /// Create an unhealthy status with message
    #[must_use]
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: CanonicalHealthStatus::Unhealthy,
            message: Some(message.into()),
            metrics: HashMap::new()),
            components: HashMap::new()),
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
