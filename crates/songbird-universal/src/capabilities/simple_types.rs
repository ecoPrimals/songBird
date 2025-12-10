//! Simple types for E2E testing and integration

use std::collections::HashMap;

/// Simple service health information for E2E testing
#[derive(Debug, Clone)]
pub struct SimpleServiceHealth {
    /// Current health status
    pub status: crate::types::HealthStatus,
}

/// Simple capability request for E2E testing
#[derive(Debug, Clone)]
pub struct SimpleCapabilityRequest {
    /// Capability type
    pub capability: String,
    /// Operation to perform
    pub operation: String,
    /// Request parameters
    pub parameters: HashMap<String, String>,
    /// Request timeout
    pub timeout: std::time::Duration,
}

/// Simple capability response for E2E testing
#[derive(Debug, Clone)]
pub struct SimpleCapabilityResponse {
    /// Whether the request succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Simple service info for E2E testing
#[derive(Debug, Clone)]
pub struct SimpleServiceInfo {
    /// Service ID
    pub id: String,
    /// Service name
    pub name: String,
    /// Service capabilities
    pub capabilities: Vec<String>,
}
