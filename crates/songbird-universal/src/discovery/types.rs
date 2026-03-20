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
