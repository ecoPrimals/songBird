// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Type definitions for capability registry

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request to register a capability provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRegistrationRequest {
    /// Unique identifier for the provider
    pub provider_id: String,

    /// Human-readable provider name
    pub provider_name: String,

    /// Type of provider (compute, storage, security, etc.)
    pub provider_type: String,

    /// Provider version
    pub version: String,

    /// Base HTTP endpoint for the provider
    pub endpoint: String,

    /// List of capabilities this provider offers
    pub capabilities: Vec<CapabilityDescriptor>,

    /// Relative path for workload execution
    pub workload_endpoint: String,

    /// Relative path for health checks
    pub health_endpoint: String,

    /// Additional provider metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Description of a capability offered by a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    /// Capability name (e.g., "`compute_gpu`", "`ml_training`")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Capability-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Response to a registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRegistrationResponse {
    /// Whether registration was successful
    pub success: bool,

    /// Registered provider details
    pub data: Option<RegistrationData>,

    /// Error message if registration failed
    pub error: Option<String>,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

/// Registration confirmation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationData {
    /// Provider ID
    pub provider_id: String,

    /// Unique registration ID
    pub registration_id: String,

    /// Registration status
    pub status: String,

    /// Expected heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,

    /// Heartbeat endpoint path
    pub heartbeat_endpoint: String,
}

/// Request to send a heartbeat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    /// Provider ID
    pub provider_id: String,

    /// Registration ID for verification
    pub registration_id: String,

    /// Current health status
    pub health_status: Option<ProviderHealthStatus>,

    /// Request timestamp
    pub timestamp: DateTime<Utc>,
}

/// Health status in heartbeat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealthStatus {
    /// Overall status
    pub status: String,

    /// Number of active tasks
    pub active_tasks: usize,

    /// Available capacity for new tasks
    pub available_capacity: usize,

    /// Resource utilization metrics
    pub resource_usage: ResourceUsageMetrics,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageMetrics {
    /// CPU utilization percentage
    pub cpu_percent: f64,

    /// Memory utilization percentage
    pub memory_percent: f64,

    /// GPU utilization percentages (one per GPU)
    pub gpu_utilization: Vec<f64>,
}

/// Response to a heartbeat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    /// Whether heartbeat was acknowledged
    pub success: bool,

    /// Response data
    pub data: Option<HeartbeatData>,

    /// Error message if heartbeat failed
    pub error: Option<String>,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

/// Heartbeat acknowledgment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatData {
    /// Whether heartbeat was acknowledged
    pub acknowledged: bool,

    /// Time until next expected heartbeat (milliseconds)
    pub next_heartbeat_ms: u64,
}

/// Internal representation of a registered provider
///
/// **ZERO-COPY OPTIMIZATION** (Dec 8, 2025 Phase 2):
/// Registration ID uses `Arc<str>` to eliminate clones during verification checks.
///
/// **VIRTUAL-TIME** (Apr 2026): Health monitoring uses `tokio::time::Instant`
/// for elapsed-time checks, enabling deterministic testing with `start_paused = true`.
/// `last_heartbeat` (`DateTime<Utc>`) is retained for API display only.
#[derive(Debug, Clone)]
pub struct RegisteredProvider {
    /// Original registration request
    pub registration: CapabilityRegistrationRequest,

    /// Unique registration ID
    /// **ZERO-COPY**: `Arc<str>` for efficient verification without clones
    pub registration_id: std::sync::Arc<str>,

    /// Current health status
    pub health: ProviderHealth,

    /// When this provider was registered
    pub registered_at: DateTime<Utc>,

    /// Last successful heartbeat (wall-clock, for API display / serialization)
    pub last_heartbeat: DateTime<Utc>,

    /// Last successful heartbeat (monotonic, for health monitor elapsed checks).
    /// Advances with `tokio::time::advance()` under `start_paused = true`.
    pub last_heartbeat_instant: tokio::time::Instant,

    /// Number of tasks currently assigned to this provider
    pub active_tasks: usize,
}

/// Provider health information
#[derive(Debug, Clone)]
pub struct ProviderHealth {
    /// Overall health status
    pub status: HealthStatus,

    /// Available capacity for new tasks
    pub available_capacity: usize,

    /// Current resource utilization
    pub resource_usage: ResourceUsage,
}

/// Health status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Provider is healthy and accepting tasks
    Healthy,

    /// Provider is degraded but still functional
    Degraded,

    /// Provider is unhealthy and should not receive new tasks
    Unhealthy,

    /// Provider is offline and will be removed
    Offline,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unhealthy => write!(f, "unhealthy"),
            Self::Offline => write!(f, "offline"),
        }
    }
}

/// Resource utilization tracking
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// CPU utilization percentage (0-100)
    pub cpu_percent: f64,

    /// Memory utilization percentage (0-100)
    pub memory_percent: f64,

    /// GPU utilization percentages (one per GPU, 0-100)
    pub gpu_utilization: Vec<f64>,
}

/// Provider list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderListResponse {
    /// Whether request was successful
    pub success: bool,

    /// Provider list data
    pub data: Option<ProviderListData>,

    /// Error message if request failed
    pub error: Option<String>,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,
}

/// Provider list data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderListData {
    /// List of providers
    pub providers: Vec<ProviderSummary>,

    /// Total number of providers
    pub total_count: usize,
}

/// Summary information about a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSummary {
    /// Provider ID
    pub provider_id: String,

    /// Provider name
    pub provider_name: String,

    /// Current status
    pub status: String,

    /// List of capability names
    pub capabilities: Vec<String>,

    /// Last heartbeat timestamp
    pub last_heartbeat: DateTime<Utc>,

    /// Number of active tasks
    pub active_tasks: usize,
}

impl From<&RegisteredProvider> for ProviderSummary {
    fn from(provider: &RegisteredProvider) -> Self {
        Self {
            provider_id: provider.registration.provider_id.clone(),
            provider_name: provider.registration.provider_name.clone(),
            status: provider.health.status.to_string(),
            capabilities: provider
                .registration
                .capabilities
                .iter()
                .map(|c| c.name.clone())
                .collect(),
            last_heartbeat: provider.last_heartbeat,
            active_tasks: provider.active_tasks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
        assert_eq!(HealthStatus::Unhealthy.to_string(), "unhealthy");
        assert_eq!(HealthStatus::Offline.to_string(), "offline");
    }

    #[test]
    fn test_health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
    }
}
