// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Core capability and primal abstractions
//!
//! This module defines the fundamental types for representing primals,
//! their capabilities, quality metrics, and discovery mechanisms.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal primal type classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PrimalType {
    /// Primary category of the primal (e.g., "ai", "storage", "compute")
    pub category: String,
    /// Optional subcategory for finer classification
    pub subcategory: Option<String>,
    /// Version of the primal type specification
    pub version: String,
}

impl PrimalType {
    /// Creates a new `PrimalType` with the given category
    #[must_use]
    pub fn new(category: &str) -> Self {
        Self {
            category: category.to_string(),
            subcategory: None,
            version: "1.0".to_string(),
        }
    }

    /// Create from string (for backward compatibility)
    #[must_use]
    pub fn from_string(category: &str) -> Self {
        Self::new(category)
    }

    /// Returns the category as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.category
    }
}

impl std::fmt::Display for PrimalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.category)
    }
}

impl Default for PrimalType {
    fn default() -> Self {
        Self {
            category: "unknown".to_string(),
            subcategory: None,
            version: "1.0".to_string(),
        }
    }
}

/// Security level classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SecurityLevel {
    /// No security requirements
    None,
    /// Basic security (authentication only)
    Basic,
    /// Standard security (authentication + encryption)
    #[default]
    Standard,
    /// High security (standard + authorization)
    High,
    /// Maximum security (all features + audit logging)
    Maximum,
}

/// Quality of Service metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct QosMetrics {
    /// Average latency in milliseconds
    pub latency_ms: Option<f64>,
    /// Throughput in operations per second
    pub throughput_ops_sec: Option<f64>,
    /// Availability percentage (0.0 to 1.0)
    pub availability: Option<f64>,
    /// Reliability score (0.0 to 1.0)
    pub reliability: Option<f64>,
}

/// Primal capability definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrimalCapability {
    /// Type of capability (e.g., "inference", "storage", "compute")
    pub capability_type: String,
    /// Version of the capability specification
    pub version: String,
    /// Capability-specific configuration parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Quality of service metrics for this capability
    pub qos_metrics: QosMetrics,
}

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum HealthStatus {
    /// Service is fully operational
    Healthy,
    /// Service is operational but with reduced performance
    Degraded,
    /// Service is not operational
    Unhealthy,
    /// Health status is unknown or not yet determined
    #[default]
    Unknown,
}

/// Discovered capability with deployment information
///
/// This type represents a capability that has been discovered from a primal service,
/// including its deployment details (endpoint, provider, health status).
///
/// **Note**: For capability definitions and specifications, use
/// `crate::capabilities::Capability` instead. This type is specifically for
/// representing capabilities that have been discovered and are ready for use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredCapability {
    /// Name of the capability
    pub name: String,
    /// Version of the capability
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Provider identifier
    pub provider: String,
    /// Network endpoint for accessing the capability
    pub endpoint: String,
    /// Quality of service metrics
    pub qos_metrics: QosMetrics,
    /// Current health status
    pub health_status: HealthStatus,
}

// ✅ REMOVED: Deprecated type alias (Nov 9, 2025)
// Use DiscoveredCapability directly for discovered capabilities with deployment info,
// or capabilities::Capability for capability definitions
// Historical note: Previously aliased Capability = DiscoveredCapability (removed Nov 2025)

/// Discovery filters for primal search
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryFilters {
    /// Filter by specific capability types
    pub capability_types: Vec<String>,
    /// Filter by security level requirements
    pub security_levels: Vec<SecurityLevel>,
    /// Filter by geographic regions
    pub geographic_regions: Vec<String>,
    /// Filter by performance requirements
    pub performance_requirements: Option<QosMetrics>,
}

/// Capability requirement specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequirement {
    /// Type of capability required
    pub capability_type: String,
    /// Minimum version required
    pub minimum_version: String,
    /// Required `QoS` guarantees
    pub required_qos: Option<QosMetrics>,
    /// Whether this capability is optional
    pub optional: bool,
}

/// Service capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCapability {
    /// Unique identifier for this capability
    pub id: String,
    /// Type of the capability
    pub capability_type: String,
    /// Version of this capability implementation
    pub version: String,
    /// Endpoints that provide this capability
    pub endpoints: Vec<String>,
    /// Observed `QoS` metrics for this capability
    pub qos_metrics: QosMetrics,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn primal_type_new_from_string_as_str_display_and_default() {
        let p = PrimalType::new("ai");
        assert_eq!(p.category, "ai");
        assert!(p.subcategory.is_none());
        assert_eq!(p.version, "1.0");
        assert_eq!(PrimalType::from_string("storage").category, "storage");
        assert_eq!(p.as_str(), "ai");
        assert_eq!(format!("{p}"), "ai");
        let d = PrimalType::default();
        assert_eq!(d.category, "unknown");
        assert_eq!(d.version, "1.0");
    }

    #[test]
    fn security_level_default_and_serde_roundtrip() {
        assert_eq!(SecurityLevel::default(), SecurityLevel::Standard);
        for level in [
            SecurityLevel::None,
            SecurityLevel::Basic,
            SecurityLevel::Standard,
            SecurityLevel::High,
            SecurityLevel::Maximum,
        ] {
            let json = serde_json::to_string(&level).unwrap();
            let back: SecurityLevel = serde_json::from_str(&json).unwrap();
            assert_eq!(back, level);
        }
    }

    #[test]
    fn qos_metrics_default_serde_roundtrip() {
        let q = QosMetrics {
            latency_ms: Some(1.5),
            throughput_ops_sec: None,
            availability: Some(0.5),
            reliability: None,
        };
        let json = serde_json::to_string(&q).unwrap();
        let back: QosMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(back, q);
        let empty = QosMetrics::default();
        assert!(empty.latency_ms.is_none());
    }

    #[test]
    fn primal_capability_and_requirement_serde_roundtrip() {
        let mut params = HashMap::new();
        params.insert("k".to_string(), serde_json::json!(42));
        let cap = PrimalCapability {
            capability_type: "inference".to_string(),
            version: "2".to_string(),
            parameters: params,
            qos_metrics: QosMetrics::default(),
        };
        let json = serde_json::to_string(&cap).unwrap();
        let back: PrimalCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cap);

        let req = CapabilityRequirement {
            capability_type: "compute".to_string(),
            minimum_version: "1.0.0".to_string(),
            required_qos: Some(QosMetrics {
                latency_ms: Some(10.0),
                throughput_ops_sec: None,
                availability: None,
                reliability: None,
            }),
            optional: true,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: CapabilityRequirement = serde_json::from_str(&json).unwrap();
        assert_eq!(back.capability_type, req.capability_type);
        assert_eq!(back.minimum_version, req.minimum_version);
        assert_eq!(back.optional, req.optional);
        assert_eq!(back.required_qos, req.required_qos);
    }

    #[test]
    fn health_status_default_and_serde() {
        assert_eq!(HealthStatus::default(), HealthStatus::Unknown);
        let h = HealthStatus::Degraded;
        let json = serde_json::to_string(&h).unwrap();
        let back: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, h);
    }

    #[test]
    fn discovered_capability_serde_roundtrip() {
        let d = DiscoveredCapability {
            name: "n".to_string(),
            version: "v".to_string(),
            description: "d".to_string(),
            provider: "p".to_string(),
            endpoint: "e".to_string(),
            qos_metrics: QosMetrics::default(),
            health_status: HealthStatus::Healthy,
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: DiscoveredCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, d.name);
        assert_eq!(back.health_status, d.health_status);
    }

    #[test]
    fn discovery_filters_default_serde() {
        let f = DiscoveryFilters {
            capability_types: vec!["a".into()],
            security_levels: vec![SecurityLevel::High],
            geographic_regions: vec!["us".into()],
            performance_requirements: None,
        };
        let json = serde_json::to_string(&f).unwrap();
        let back: DiscoveryFilters = serde_json::from_str(&json).unwrap();
        assert_eq!(back.capability_types, f.capability_types);
        assert_eq!(back.security_levels, f.security_levels);
        let def = DiscoveryFilters::default();
        assert!(def.capability_types.is_empty());
    }

    #[test]
    fn service_capability_serde_roundtrip() {
        let s = ServiceCapability {
            id: "id1".into(),
            capability_type: "t".into(),
            version: "1".into(),
            endpoints: vec!["http://x".into()],
            qos_metrics: QosMetrics::default(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: ServiceCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, s.id);
        assert_eq!(back.capability_type, s.capability_type);
        assert_eq!(back.version, s.version);
        assert_eq!(back.endpoints, s.endpoints);
        assert_eq!(back.qos_metrics, s.qos_metrics);
    }
}
