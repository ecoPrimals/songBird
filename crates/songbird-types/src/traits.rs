//! **CANONICAL**: Trait definitions for the Songbird ecosystem
//!
//! This module provides the core traits that define the interfaces for
//! all Songbird components. All implementations MUST use these traits.

pub mod canonical;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::errors::SongbirdResult;

/// **CANONICAL**: Health check trait
#[async_trait]
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
#[async_trait]
pub trait CanonicalConfigProvider: Send + Sync {
    /// Get configuration value
    async fn get_config(&self, key: &str) -> SongbirdResult<Option<String>>;

    /// Set configuration value
    async fn set_config(&self, key: &str, value: &str) -> SongbirdResult<()>;
}

/// **CANONICAL**: Service discovery trait
#[async_trait]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    Stopped,
}

/// **CANONICAL**: Load balancer trait
#[async_trait]
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
#[async_trait]
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
#[async_trait]
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

impl Default for ServiceInstanceStatus {
    fn default() -> Self {
        Self::Stopped
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
