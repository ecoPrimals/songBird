// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Universal Primal Discovery Types
//!
//! Core types for capability-based primal discovery.
//! This module is part of the smart refactoring of discovery.rs

use crate::capabilities::Capability;
use crate::types::PrimalType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::Duration;

/// Discovery configuration for universal adapters
///
/// Aligns with canonical discovery pattern with nested mechanisms.
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Discovery mechanisms to enable
    pub mechanisms: DiscoveryMechanisms,
    /// Timeout for discovery operations
    pub timeout: Duration,
}

/// Discovery mechanisms configuration
#[derive(Debug, Clone)]
pub struct DiscoveryMechanisms {
    /// Enable environment variable scanning
    pub enable_environment_scan: bool,
    /// Enable network scanning for services
    pub enable_network_scanning: bool,
    /// Enable container/orchestration discovery
    pub enable_container_discovery: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            mechanisms: DiscoveryMechanisms {
                enable_environment_scan: true,
                enable_network_scanning: true,
                enable_container_discovery: true,
            },
            timeout: Duration::from_secs(30),
        }
    }
}

/// A discovered primal with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Name of the primal (capability-based, not hardcoded)
    pub name: String,
    /// Primal type (Security, Storage, AI, Compute, etc.)
    pub primal_type: PrimalType,
    /// Endpoint URL for connecting to this primal
    pub endpoint: String,
    /// Discovered capabilities this primal offers
    pub capabilities: Vec<Capability>,
    /// Current health status
    pub health: PrimalHealth,
    /// Method used to discover this primal
    pub discovery_method: DiscoveryMethod,
    /// Additional metadata key-value pairs
    pub metadata: HashMap<String, String>,
}

/// Method used to discover a primal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// Discovered via environment variables
    Environment,
    /// Discovered via network scanning
    NetworkScan,
    /// Discovered via container orchestration (Docker, K8s, etc.)
    ContainerOrchestration,
    /// Discovered via service registry (Consul, etcd, etc.)
    ServiceRegistry,
    /// Discovered via mDNS/Bonjour
    MDNS,
    /// Manually configured
    Manual,
}

/// Health status of a discovered primal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrimalHealth {
    /// Primal is healthy and operational
    Healthy,
    /// Primal is degraded but functional
    Degraded,
    /// Primal is unhealthy
    Unhealthy,
    /// Health status unknown
    Unknown,
}

/// Discovery error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum DiscoveryError {
    /// No primals discovered
    #[error("No primals discovered")]
    NoPrimalsFound,

    /// Discovery timeout
    #[error("Discovery timeout after {0:?}")]
    Timeout(Duration),

    /// Network error during discovery
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Health check failed
    #[error("Health check failed for {primal}: {reason}")]
    HealthCheckFailed {
        /// Primal name
        primal: String,
        /// Failure reason
        reason: String,
    },
}

impl DiscoveredPrimal {
    /// Create a new discovered primal
    #[must_use]
    pub fn new(
        name: String,
        primal_type: PrimalType,
        endpoint: String,
        capabilities: Vec<Capability>,
        discovery_method: DiscoveryMethod,
    ) -> Self {
        Self {
            name,
            primal_type,
            endpoint,
            capabilities,
            health: PrimalHealth::Unknown,
            discovery_method,
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the discovered primal
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if primal is healthy
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self.health, PrimalHealth::Healthy)
    }

    /// Check if primal has a specific capability
    #[must_use]
    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::{DiscoveredPrimal, DiscoveryConfig, DiscoveryError, DiscoveryMethod, PrimalHealth};
    use crate::capabilities::Capability;
    use crate::types::PrimalType;
    use songbird_test_utils::canonical_test_framework::TestContext;
    use tokio::time::Duration;

    fn sample_capability() -> Capability {
        Capability::from_string("encryption").expect("encryption maps")
    }

    #[test]
    fn discovery_config_default() {
        let c = DiscoveryConfig::default();
        assert_eq!(c.timeout, Duration::from_secs(30));
        assert!(c.mechanisms.enable_environment_scan);
    }

    #[test]
    fn discovered_primal_new_and_metadata() {
        let ctx = TestContext::new("discovered_primal");
        let cap = sample_capability();
        let p = DiscoveredPrimal::new(
            "n".to_string(),
            PrimalType::new("security"),
            "https://x".to_string(),
            vec![cap.clone()],
            DiscoveryMethod::Manual,
        )
        .with_metadata("k".to_string(), "v".to_string());
        assert!(!p.is_healthy());
        assert!(p.has_capability(&cap));
        assert_eq!(p.metadata.get("k"), Some(&"v".to_string()));
        assert!(!ctx.is_timeout());
    }

    #[test]
    fn discovery_method_serde_roundtrip() {
        for m in [
            DiscoveryMethod::Environment,
            DiscoveryMethod::NetworkScan,
            DiscoveryMethod::ContainerOrchestration,
            DiscoveryMethod::ServiceRegistry,
            DiscoveryMethod::MDNS,
            DiscoveryMethod::Manual,
        ] {
            let j = serde_json::to_string(&m).unwrap();
            let back: DiscoveryMethod = serde_json::from_str(&j).unwrap();
            assert_eq!(m, back);
        }
    }

    #[test]
    fn primal_health_serde_roundtrip() {
        for h in [
            PrimalHealth::Healthy,
            PrimalHealth::Degraded,
            PrimalHealth::Unhealthy,
            PrimalHealth::Unknown,
        ] {
            let j = serde_json::to_string(&h).unwrap();
            let back: PrimalHealth = serde_json::from_str(&j).unwrap();
            assert_eq!(h, back);
        }
    }

    #[test]
    fn discovery_error_display() {
        let e = DiscoveryError::ConfigError("bad".to_string());
        let s = e.to_string();
        assert!(s.contains("bad"));
        let t = DiscoveryError::Timeout(Duration::from_secs(1));
        assert!(t.to_string().contains("timeout") || t.to_string().contains('1'));
    }

    #[test]
    fn discovered_primal_serde_roundtrip() {
        let cap = sample_capability();
        let p = DiscoveredPrimal::new(
            "p1".to_string(),
            PrimalType::new("compute"),
            "http://localhost:1".to_string(),
            vec![cap],
            DiscoveryMethod::Environment,
        );
        let j = serde_json::to_string(&p).unwrap();
        let back: DiscoveredPrimal = serde_json::from_str(&j).unwrap();
        assert_eq!(p.name, back.name);
        assert_eq!(p.endpoint, back.endpoint);
    }
}
