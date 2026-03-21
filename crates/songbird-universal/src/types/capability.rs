// SPDX-License-Identifier: AGPL-3.0-only
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
