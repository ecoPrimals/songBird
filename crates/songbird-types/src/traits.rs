// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! **CANONICAL**: Trait definitions for the Songbird ecosystem
//!
//! This module provides the core traits that define the interfaces for
//! all Songbird components. All implementations MUST use these traits.

// Allow async_fn_in_trait warning - our traits guarantee Send + Sync
#![allow(async_fn_in_trait, reason = "async fn in trait (edition / trait-object compatibility)")]

pub mod canonical;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::errors::SongbirdResult;

/// **CANONICAL**: Health check trait
pub trait CanonicalHealthCheck: Send + Sync {
    /// Perform a health check
    async fn health_check(&self) -> SongbirdResult<HealthStatus>;
}

/// Health status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health status
    pub healthy: bool,
    /// Status message
    pub message: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Detailed health information with component breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedHealthInfo {
    /// Overall status
    pub status: HealthStatus,
    /// Component-specific health information
    pub components: HashMap<String, HealthStatus>,
}

/// **CANONICAL**: Configuration provider trait
pub trait CanonicalConfigProvider: Send + Sync {
    /// Get configuration value
    async fn get_config(&self, key: &str) -> SongbirdResult<Option<String>>;

    /// Set configuration value
    async fn set_config(&self, key: &str, value: &str) -> SongbirdResult<()>;
}

/// **CANONICAL**: Service discovery trait
pub trait CanonicalServiceDiscovery: Send + Sync {
    /// Register a service
    async fn register_service(
        &self,
        service_info: &crate::service::CanonicalServiceInfo,
    ) -> SongbirdResult<()>;

    /// Discover services by capability
    async fn discover_services(
        &self,
        capability: &str,
    ) -> SongbirdResult<Vec<crate::service::CanonicalServiceInfo>>;

    /// Unregister a service
    async fn unregister_service(&self, service_id: &str) -> SongbirdResult<()>;
}

/// Service instance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ServiceInstanceStatus {
    /// Service is starting up
    Starting,
    /// Service is running and healthy
    Running,
    /// Service is degraded but functional
    Degraded,
    /// Service is unhealthy
    Unhealthy,
    /// Service is stopping
    Stopping,
    /// Service is stopped
    #[default]
    Stopped,
}

/// **CANONICAL**: Load balancer trait
pub trait CanonicalLoadBalancer: Send + Sync {
    /// Select a service instance
    async fn select_instance(&self, service_name: &str) -> SongbirdResult<Option<String>>;

    /// Report instance health
    async fn report_health(
        &self,
        instance_id: &str,
        status: ServiceInstanceStatus,
    ) -> SongbirdResult<()>;
}

/// **CANONICAL**: Observability provider trait
pub trait CanonicalObservabilityProvider: Send + Sync {
    /// Record a metric
    async fn record_metric(
        &self,
        name: &str,
        value: f64,
        tags: &HashMap<String, String>,
    ) -> SongbirdResult<()>;

    /// Record an event
    async fn record_event(
        &self,
        event: &str,
        details: &HashMap<String, String>,
    ) -> SongbirdResult<()>;

    /// Get metrics
    async fn get_metrics(&self, name_pattern: &str) -> SongbirdResult<Vec<MetricValue>>;
}

/// Metric value with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// Metric name
    pub name: String,
    /// Metric value
    pub value: f64,
    /// Metric tags
    pub tags: HashMap<String, String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// **CANONICAL**: Error handling trait
pub trait CanonicalErrorHandler: Send + Sync {
    /// Handle an error
    /// Handle error
    ///
    /// # Errors
    /// Returns error if error handling fails
    fn handle_error(&self, error: &crate::errors::SongbirdError) -> SongbirdResult<()>;

    /// Check if error is recoverable
    fn is_recoverable(&self, error: &crate::errors::SongbirdError) -> bool;
}

/// **CANONICAL**: Capability provider trait
pub trait CanonicalCapabilityProvider: Send + Sync {
    /// Get available capabilities
    async fn get_capabilities(&self) -> SongbirdResult<Vec<String>>;

    /// Check if capability is supported
    async fn supports_capability(&self, capability: &str) -> SongbirdResult<bool>;

    /// Execute capability
    async fn execute_capability(
        &self,
        capability: &str,
        params: &HashMap<String, String>,
    ) -> SongbirdResult<serde_json::Value>;
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            healthy: false,
            message: "Unknown".to_string(),
            timestamp: chrono::Utc::now(),
        }
    }
}

impl fmt::Display for ServiceInstanceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Starting => write!(f, "starting"),
            Self::Running => write!(f, "running"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unhealthy => write!(f, "unhealthy"),
            Self::Stopping => write!(f, "stopping"),
            Self::Stopped => write!(f, "stopped"),
        }
    }
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    clippy::float_cmp,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[cfg(test)]
mod tests {
    use super::super::{SongbirdError, SongbirdResult};
    use super::*;

    #[test]
    fn test_health_status_default() {
        let status = HealthStatus::default();
        assert!(!status.healthy, "Default status should be unhealthy");
        assert_eq!(status.message, "Unknown");
    }

    #[test]
    fn test_health_status_creation() {
        let now = chrono::Utc::now();
        let status = HealthStatus {
            healthy: true,
            message: "All systems operational".to_string(),
            timestamp: now,
        };

        assert!(status.healthy);
        assert_eq!(status.message, "All systems operational");
        assert_eq!(status.timestamp, now);
    }

    #[test]
    fn test_service_instance_status_default() {
        let status = ServiceInstanceStatus::default();
        assert_eq!(status, ServiceInstanceStatus::Stopped);
    }

    #[test]
    fn test_service_instance_status_display() {
        assert_eq!(ServiceInstanceStatus::Starting.to_string(), "starting");
        assert_eq!(ServiceInstanceStatus::Running.to_string(), "running");
        assert_eq!(ServiceInstanceStatus::Degraded.to_string(), "degraded");
        assert_eq!(ServiceInstanceStatus::Unhealthy.to_string(), "unhealthy");
        assert_eq!(ServiceInstanceStatus::Stopping.to_string(), "stopping");
        assert_eq!(ServiceInstanceStatus::Stopped.to_string(), "stopped");
    }

    #[test]
    fn test_service_instance_status_equality() {
        assert_eq!(ServiceInstanceStatus::Running, ServiceInstanceStatus::Running);
        assert_ne!(ServiceInstanceStatus::Running, ServiceInstanceStatus::Stopped);
    }

    #[test]
    fn test_service_instance_status_serialization() -> SongbirdResult<()> {
        let status = ServiceInstanceStatus::Running;
        let json = serde_json::to_string(&status).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {}", e))
        })?;
        assert!(json.contains("Running"));

        let deserialized: ServiceInstanceStatus =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Deserialization should succeed: {}", e),
                debug_info: None,
            })?;
        assert_eq!(deserialized, status);
        Ok(())
    }

    #[test]
    fn test_metric_value_creation() {
        let now = chrono::Utc::now();
        let mut tags = HashMap::new();
        tags.insert("service".to_string(), "test".to_string());

        let metric = MetricValue {
            name: "requests_total".to_string(),
            value: 42.0,
            tags: tags.clone(),
            timestamp: now,
        };

        assert_eq!(metric.name, "requests_total");
        assert_eq!(metric.value, 42.0);
        assert_eq!(metric.tags.len(), 1);
        assert_eq!(metric.tags.get("service"), Some(&"test".to_string()));
    }

    #[test]
    fn test_metric_value_serialization() -> SongbirdResult<()> {
        let now = chrono::Utc::now();
        let metric = MetricValue {
            name: "cpu_usage".to_string(),
            value: 75.5,
            tags: HashMap::new(),
            timestamp: now,
        };

        let json = serde_json::to_string(&metric).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {}", e))
        })?;
        assert!(json.contains("cpu_usage"));
        assert!(json.contains("75.5"));

        let deserialized: MetricValue =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Deserialization should succeed: {}", e),
                debug_info: None,
            })?;
        assert_eq!(deserialized.name, metric.name);
        assert_eq!(deserialized.value, metric.value);
        Ok(())
    }

    #[test]
    fn test_detailed_health_info_creation() {
        let overall = HealthStatus {
            healthy: true,
            message: "System healthy".to_string(),
            timestamp: chrono::Utc::now(),
        };

        let mut components = HashMap::new();
        components.insert(
            "database".to_string(),
            HealthStatus {
                healthy: true,
                message: "Connected".to_string(),
                timestamp: chrono::Utc::now(),
            },
        );

        let detailed = DetailedHealthInfo {
            status: overall,
            components: components.clone(),
        };

        assert!(detailed.status.healthy);
        assert_eq!(detailed.components.len(), 1);
        assert!(detailed.components.contains_key("database"));
    }

    #[test]
    fn test_detailed_health_info_serialization() -> SongbirdResult<()> {
        let detailed = DetailedHealthInfo {
            status: HealthStatus::default(),
            components: HashMap::new(),
        };

        let json = serde_json::to_string(&detailed).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {}", e))
        })?;
        let deserialized: DetailedHealthInfo =
            serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
                format: Some("JSON".to_string()),
                message: format!("Deserialization should succeed: {}", e),
                debug_info: None,
            })?;

        assert_eq!(deserialized.status.healthy, detailed.status.healthy);
        assert_eq!(deserialized.components.len(), 0);
        Ok(())
    }
}
