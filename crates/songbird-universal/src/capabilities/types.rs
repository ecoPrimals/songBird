//! Capability type definitions
//!
//! Core types for representing capabilities, `QoS` metrics, and resource usage.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal primal capability definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(clippy::struct_field_names)]
pub struct Capability {
    /// Capability type (e.g., "compute", "storage", "security", "ai")
    pub capability_type: String,
    /// Capability name (e.g., "encryption", "`container_runtime`", "`model_inference`")
    pub name: String,
    /// Version of the capability
    pub version: String,
    /// Parameters supported by this capability
    pub parameters: HashMap<String, serde_json::Value>,
    /// Quality of service metrics
    pub qos_metrics: QoSMetrics,
    /// Whether this capability is currently available
    pub available: bool,
}

/// Quality of Service metrics for capabilities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QoSMetrics {
    /// Average latency in milliseconds
    pub latency_ms: f64,
    /// Throughput in operations per second
    pub throughput_ops_sec: f64,
    /// Availability percentage (0.0 to 1.0)
    pub availability: f64,
    /// Reliability percentage (0.0 to 1.0)
    pub reliability: f64,
    /// Resource usage metrics
    pub resource_usage: ResourceMetrics,
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResourceMetrics {
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage in MB
    pub memory_mb: u64,
    /// Network bandwidth usage in Mbps
    pub network_mbps: f64,
    /// Storage usage in MB
    pub storage_mb: u64,
}

/// Discovery configuration
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// How often to refresh capabilities
    pub refresh_interval: std::time::Duration,
    /// Timeout for capability discovery requests
    pub discovery_timeout: std::time::Duration,
    /// Maximum number of concurrent discovery requests
    pub max_concurrent_discoveries: usize,
    /// Whether to enable automatic discovery
    pub auto_discovery: bool,
    /// Whether to enable network-based discovery
    pub enable_network_discovery: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            refresh_interval: std::time::Duration::from_secs(300), // 5 minutes
            discovery_timeout: std::time::Duration::from_secs(10),
            max_concurrent_discoveries: 10,
            auto_discovery: true,
            enable_network_discovery: false,
        }
    }
}

impl Default for QoSMetrics {
    fn default() -> Self {
        Self {
            latency_ms: 100.0,
            throughput_ops_sec: 1000.0,
            availability: 0.99,
            reliability: 0.99,
            resource_usage: ResourceMetrics::default(),
        }
    }
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            cpu_percent: 10.0,
            memory_mb: 512,
            network_mbps: 10.0,
            storage_mb: 1024,
        }
    }
}

/// Response format for capability queries
#[derive(Debug, Deserialize)]
pub(super) struct CapabilityResponse {
    pub capabilities: Vec<Capability>,
}

/// Primal type classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimalType {
    /// Security services (beardog, auth)
    Security,
    /// Compute services (toadstool, containers)
    Compute,
    /// Storage services (nestgate, databases)
    Storage,
    /// AI/ML services (squirrel, models)
    AI,
    /// Generic/unknown services
    Generic,
}
