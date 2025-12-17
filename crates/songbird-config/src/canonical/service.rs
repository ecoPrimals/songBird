//! Service Configuration
//!
//! Canonical service configuration types and utilities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use songbird_config; // FIXED: Circular import removed

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service name
    pub name: String,
    /// Service address
    pub address: String,
    /// Service port
    pub port: u16,
    /// Service metadata
    pub metadata: HashMap<String, String>,
    /// Health check configuration
    pub health_check: Option<HealthCheckConfig>,
}

/// Canonical Service Information (backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceInfo {
    pub service_id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub metadata: HashMap<String, String>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check endpoint
    pub endpoint: String,
    /// Check interval in seconds
    pub interval: u64,
    /// Timeout in seconds
    pub timeout: u64,
}

// Type aliases for backward compatibility
pub type CanonicalServiceInfo = ServiceInfo;
pub type UniversalServiceInfo = ServiceInfo;

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            name: "default-service".to_string(),
            address: "localhost".to_string(),
            port: 8080,
            metadata: HashMap::new(),
            health_check: Some(HealthCheckConfig::default()),
        }
    }
}

impl Default for ServiceInfo {
    fn default() -> Self {
        Self {
            service_id: "default-service".to_string(),
            name: "default-service".to_string(),
            host: "localhost".to_string(),
            port: 8080,
            metadata: HashMap::new(),
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            endpoint: "/health".to_string(),
            interval: 30,
            timeout: 5,
        }
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
