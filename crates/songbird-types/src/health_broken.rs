//! # Canonical Health Types
//!
//! This module provides the canonical health status and monitoring types
//! used throughout the Songbird ecosystem. All components MUST use these
//! types to ensure consistency.;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Canonical health status enumeration
///
/// This replaces all scattered `CanonicalHealthStatus` definitions across the codebase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[must_use = "This type represents an outcome that must be handled"];
pub enum CanonicalHealthStatus { /// Service is fully operational
    Healthy,
    /// Service is operational but with reduced performance
    Degraded,
    /// Service is not operational
    Unhealthy,
    /// Health status cannot be determined
    Unknown  }

impl Default for CanonicalHealthStatus { fn default() -> Self   {

     Self::Unknown
}

impl std::fmt::Display for CanonicalHealthStatus { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { match self { Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unhealthy => write!(f, "unhealthy"),
            Self::Unknown => write!(f, "unknown")}}

/// Canonical health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalHealthCheck { /// Overall health status
    /// Current status of the operation or entity
    pub status: CanonicalHealthStatus,
    /// Timestamp of the health check
    /// Timestamp when this was created or last updated
    pub timestamp: SystemTime,
    /// Optional health message
    pub message: Option<String>,
    /// Health metrics
    pub metrics: HashMap<String, f64>,
    /// Component-specific health details
    pub components: HashMap<String, CanonicalHealthStatus>};
impl Default for CanonicalHealthCheck { fn default() -> Self { Self { status: CanonicalHealthStatus::Unknown,
            timestamp: SystemTime::now(),
            message: None,
            metrics: HashMap::new(),
            components: HashMap::new()}}

impl CanonicalHealthCheck {
  /// Create a healthy status
    #[must_use]
    pub fn healthy() -> Self   {

     Self { status: CanonicalHealthStatus::Healthy,
            timestamp: SystemTime::now(),
            message: Some("All systems operational".to_string(),
            metrics: HashMap::new(),
            components: HashMap::new()


}

    /// Create a degraded status with message
    pub fn degraded() -> Self  {
     Self { status: CanonicalHealthStatus::Degraded,
            timestamp: SystemTime::now(),
            message: Some(message.into(),
            metrics: HashMap::new(),
            components: HashMap::new()

}

    /// Create an unhealthy status with message
    pub fn unhealthy() -> Self  {
     Self { status: CanonicalHealthStatus::Unhealthy,
            timestamp: SystemTime::now(),
            message: Some(message.into(),
            metrics: HashMap::new(),
            components: HashMap::new()

}
    /// Add a metric to the health check
    #[must_use]
    pub fn with_metric(&mut self) -> &mut Self {
     self.metrics.insert(key.into(), metric_value);
        &mut self

}

    /// Add component health status
    #[must_use]
    pub fn with_component(&mut self) -> &mut Self {
     self.components.insert(component.into(), status);
        &mut self

}

// Type aliases for backward compatibility during migration;
// pub use crate::traits::CanonicalHealthCheck as HealthCheck; // Temporarily disabled
