// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Service lifecycle and identification types
//!
//! This module provides types for service registration, discovery,
//! health tracking, and resource management.

use super::capability::{DiscoveredCapability, HealthStatus, PrimalType, QosMetrics};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;

/// Service information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Name of the service
    pub name: String,
    /// Type of primal this service represents
    pub primal_type: PrimalType,
    /// Network endpoint for accessing the service
    pub endpoint: String,
    /// List of capabilities provided by this service
    pub capabilities: Vec<DiscoveredCapability>,
    /// Current health status of the service
    pub health: HealthStatus,
    /// Additional metadata key-value pairs
    pub metadata: HashMap<String, String>,
}

/// Service event for monitoring and observability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEvent {
    /// Name of the service that generated the event
    pub service_name: String,
    /// Type of event (e.g., "started", "stopped", "error")
    pub event_type: String,
    /// Timestamp when the event occurred
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional event details
    pub details: HashMap<String, serde_json::Value>,
}

/// Registered service in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredService {
    /// Unique identifier for this service registration
    pub id: String,
    /// Service information and capabilities
    pub service_info: ServiceInfo,
    /// Timestamp when the service was registered
    pub registration_time: chrono::DateTime<chrono::Utc>,
    /// Timestamp of the last heartbeat received
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
}

/// Service identification information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceIdentification {
    /// Unique service identifier
    pub service_id: String,
    /// Service name (human-readable)
    pub service_name: String,
    /// Service version
    pub version: String,
    /// Instance identifier (for multi-instance services)
    pub instance_id: String,
}

/// Service endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceEndpoint {
    /// Full URL of the service endpoint
    pub url: String,
    /// Protocol used by the endpoint
    pub protocol: String,
    /// Whether TLS is enabled for this endpoint
    pub tls_enabled: bool,
    /// Port number for the service
    pub port: u16,
}

impl Default for ServiceEndpoint {
    fn default() -> Self {
        Self {
            url: String::new(),
            protocol: "http".to_string(),
            tls_enabled: false,
            port: songbird_config::defaults::ports::orchestrator_port(),
        }
    }
}

/// Service health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    /// Current health status
    pub status: HealthStatus,
    /// Last successful health check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
    /// Number of consecutive successful checks
    pub consecutive_successes: u32,
    /// Number of consecutive failed checks
    pub consecutive_failures: u32,
    /// Recent quality of service metrics
    pub recent_qos: Option<QosMetrics>,
}

/// Resource specification for service deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    /// CPU cores requested
    pub cpu_cores: Option<f32>,
    /// Memory in MB
    pub memory_mb: Option<u64>,
    /// Disk space in MB
    pub disk_mb: Option<u64>,
    /// Network bandwidth in Mbps
    pub network_mbps: Option<f32>,
}
