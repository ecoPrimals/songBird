// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Capability type definitions
//!
//! Core types for representing capabilities, `QoS` metrics, and resource usage.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal primal capability definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(clippy::struct_field_names, reason = "capability_type/name mirror discovery wire format")]
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

impl Capability {
    /// Create a Capability from a string description
    ///
    /// **Evolution**: Capability-based discovery without hardcoding
    #[must_use]
    pub fn from_string(s: &str) -> Option<Self> {
        let capability_type = match s.to_lowercase().as_str() {
            "security" | "auth" | "authentication" | "encryption" => "security",
            "storage" | "persistence" | "cache" | "database" => "storage",
            "ai" | "ml" | "inference" | "training" => "ai",
            "compute" | "execution" | "processing" | "container" => "compute",
            "discovery" => "discovery",
            "orchestration" => "orchestration",
            _ => return None, // Return None for unknown capabilities
        };

        Some(Self {
            capability_type: capability_type.to_string(),
            name: s.to_string(),
            version: "1.0.0".to_string(),
            parameters: HashMap::new(),
            qos_metrics: QoSMetrics::default(),
            available: true,
        })
    }

    /// Get the capability category
    #[must_use]
    pub fn category(&self) -> &str {
        &self.capability_type
    }
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

/// Discovery configuration for capabilities
///
/// **LOCAL DEFINITION**: Kept local due to flat vs nested structure differences.
/// Canonical version uses nested configs (service/capability/network) which would require
/// extensive call-site updates. Fields aligned with canonical where possible.
///
/// **Future**: Consider migrating to canonical nested structure in major version bump.
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// How often to refresh capabilities (aligns with `service_discovery.discovery_interval_secs`)
    pub refresh_interval: std::time::Duration,
    /// Timeout for capability discovery requests (aligns with `service_discovery.discovery_timeout_secs`)
    pub discovery_timeout: std::time::Duration,
    /// Maximum number of concurrent discovery requests (aligns with `service_discovery.max_concurrent_discoveries`)
    pub max_concurrent_discoveries: usize,
    /// Whether to enable automatic discovery (aligns with canonical `auto_discovery`)
    pub auto_discovery: bool,
    /// Whether to enable network-based discovery (aligns with `network_discovery.enabled`)
    pub enable_network_discovery: bool,
    /// Explicit provider base URLs per canonical capability (`compute`, `storage`, `ai`, `security`).
    ///
    /// Same role as `*_PROVIDER_ENDPOINT` environment variables, but scoped to this adapter
    /// instance (safe for concurrent tests).
    pub provider_endpoints: HashMap<String, String>,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            refresh_interval: std::time::Duration::from_secs(300), // 5 minutes (aligns with canonical default: 30s)
            discovery_timeout: std::time::Duration::from_secs(10), // Reasonable timeout
            max_concurrent_discoveries: 10,                        // Aligns with canonical default
            auto_discovery: true,                                  // Aligns with canonical default
            enable_network_discovery: false, // Secure default (canonical: false)
            provider_endpoints: HashMap::new(),
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
///
/// Used internally for deserializing HTTP responses from primal services.
/// Public within module for testing, not exposed in public API.
#[derive(Debug, Deserialize)]
#[allow(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
pub(super) struct CapabilityResponse {
    pub capabilities: Vec<Capability>,
}

/// Primal type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalType {
    /// Security services (beardog, auth)
    Security,
    /// Compute services (containers, batch jobs, GPU workloads)
    Compute,
    /// Storage services (storage provider, databases)
    Storage,
    /// AI/ML services (`ai` capability domain; any provider)
    AI,
    /// Discovery services
    Discovery,
    /// Orchestration services (songbird)
    Orchestration,
    /// Generic/unknown services
    Generic,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_from_string_security() {
        let cap = Capability::from_string("security").expect("'security' is a valid capability");
        assert_eq!(cap.capability_type, "security");
        assert_eq!(cap.category(), "security");
        assert!(cap.available);
    }

    #[test]
    fn test_capability_from_string_aliases() {
        // Test security aliases
        assert!(Capability::from_string("auth").is_some());
        assert!(Capability::from_string("authentication").is_some());
        assert!(Capability::from_string("encryption").is_some());

        // Test storage aliases
        assert!(Capability::from_string("storage").is_some());
        assert!(Capability::from_string("persistence").is_some());
        assert!(Capability::from_string("cache").is_some());

        // Test AI aliases
        assert!(Capability::from_string("ai").is_some());
        assert!(Capability::from_string("ml").is_some());
        assert!(Capability::from_string("inference").is_some());

        // Test compute aliases
        assert!(Capability::from_string("compute").is_some());
        assert!(Capability::from_string("execution").is_some());
        assert!(Capability::from_string("container").is_some());
    }

    #[test]
    fn test_capability_from_string_unknown() {
        let cap = Capability::from_string("unknown_capability");
        assert!(cap.is_none());
    }

    #[test]
    fn test_capability_from_string_case_insensitive() {
        assert!(Capability::from_string("SECURITY").is_some());
        assert!(Capability::from_string("Security").is_some());
        assert!(Capability::from_string("SeCuRiTy").is_some());
    }

    #[test]
    fn test_capability_category() {
        let cap = Capability::from_string("storage").expect("'storage' is a valid capability");
        assert_eq!(cap.category(), "storage");
    }

    #[test]
    fn test_capability_equality() {
        let cap1 = Capability::from_string("security").expect("'security' is a valid capability");
        let cap2 = Capability::from_string("security").expect("'security' is a valid capability");
        assert_eq!(cap1, cap2);
    }

    #[test]
    fn test_qos_metrics_default() {
        let qos = QoSMetrics::default();
        assert_eq!(qos.latency_ms, 100.0);
        assert_eq!(qos.throughput_ops_sec, 1000.0);
        assert_eq!(qos.availability, 0.99);
        assert_eq!(qos.reliability, 0.99);
    }

    #[test]
    fn test_resource_metrics_default() {
        let resources = ResourceMetrics::default();
        assert_eq!(resources.cpu_percent, 10.0);
        assert_eq!(resources.memory_mb, 512);
        assert_eq!(resources.network_mbps, 10.0);
        assert_eq!(resources.storage_mb, 1024);
    }

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert_eq!(config.refresh_interval.as_secs(), 300);
        assert_eq!(config.discovery_timeout.as_secs(), 10);
        assert_eq!(config.max_concurrent_discoveries, 10);
        assert!(config.auto_discovery);
        assert!(!config.enable_network_discovery);
        assert!(config.provider_endpoints.is_empty());
    }

    #[test]
    fn test_capability_serialization() {
        let cap = Capability::from_string("ai").expect("'ai' is a valid capability");
        let json = serde_json::to_string(&cap).expect("Capability should serialize to JSON");
        let deserialized: Capability =
            serde_json::from_str(&json).expect("JSON should deserialize to Capability");
        assert_eq!(cap, deserialized);
    }

    #[test]
    fn test_qos_metrics_serialization() {
        let qos = QoSMetrics::default();
        let json = serde_json::to_string(&qos).expect("QoSMetrics should serialize to JSON");
        let deserialized: QoSMetrics =
            serde_json::from_str(&json).expect("JSON should deserialize to QoSMetrics");
        assert_eq!(qos, deserialized);
    }

    #[test]
    fn test_resource_metrics_serialization() {
        let resources = ResourceMetrics::default();
        let json =
            serde_json::to_string(&resources).expect("ResourceMetrics should serialize to JSON");
        let deserialized: ResourceMetrics =
            serde_json::from_str(&json).expect("JSON should deserialize to ResourceMetrics");
        assert_eq!(resources, deserialized);
    }

    #[test]
    fn test_primal_type_variants() {
        assert_eq!(PrimalType::Security, PrimalType::Security);
        assert_ne!(PrimalType::Security, PrimalType::Compute);
        assert!(matches!(PrimalType::AI, PrimalType::AI));
    }

    #[test]
    fn test_primal_type_serialization() {
        let primal = PrimalType::Orchestration;
        let json = serde_json::to_string(&primal).expect("PrimalType should serialize to JSON");
        let deserialized: PrimalType =
            serde_json::from_str(&json).expect("JSON should deserialize to PrimalType");
        assert_eq!(primal, deserialized);
    }

    #[test]
    fn test_capability_all_types() {
        let types = vec!["security", "storage", "ai", "compute", "discovery", "orchestration"];
        for t in types {
            let cap = Capability::from_string(t);
            assert!(cap.is_some(), "Failed to create capability for: {}", t);
        }
    }
}
