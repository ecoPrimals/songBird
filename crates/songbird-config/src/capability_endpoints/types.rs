// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability enum, endpoint record, and discovery-method classification.

use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::str::FromStr;

/// Capability type for service discovery
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CapabilityType {
    /// Security capabilities (authentication, encryption, key management)
    Security,
    /// Storage capabilities (data persistence, caching, backup)
    Storage,
    /// Compute capabilities (workload execution, container orchestration)
    Compute,
    /// AI/ML capabilities (inference, training, analysis)
    Ai,
    /// Orchestration capabilities (service coordination, workflow management)
    Orchestration,
    /// Observability capabilities (logging, metrics, tracing)
    Observability,
    /// Networking capabilities (service mesh, load balancing)
    Networking,
    /// Custom capability
    Custom(String),
}

impl FromStr for CapabilityType {
    type Err = Infallible;

    /// Parse capability type from string (always succeeds, falls back to Custom)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "security" | "auth" | "authentication" | "encryption" => Self::Security,
            "storage" | "database" | "persistence" | "cache" => Self::Storage,
            "compute" | "execution" | "runtime" | "container" => Self::Compute,
            "ai" | "ml" | "inference" | "intelligence" => Self::Ai,
            "orchestration" | "coordination" | "workflow" => Self::Orchestration,
            "observability" | "logging" | "metrics" | "tracing" => Self::Observability,
            "networking" | "mesh" | "loadbalancing" => Self::Networking,
            custom => Self::Custom(custom.to_string()),
        })
    }
}

impl CapabilityType {
    /// Get environment variable name for this capability
    #[must_use]
    pub fn env_var_name(&self) -> String {
        match self {
            Self::Security => "CAPABILITY_SECURITY_ENDPOINT".to_string(),
            Self::Storage => "CAPABILITY_STORAGE_ENDPOINT".to_string(),
            Self::Compute => "CAPABILITY_COMPUTE_ENDPOINT".to_string(),
            Self::Ai => "CAPABILITY_AI_ENDPOINT".to_string(),
            Self::Orchestration => "CAPABILITY_ORCHESTRATION_ENDPOINT".to_string(),
            Self::Observability => "CAPABILITY_OBSERVABILITY_ENDPOINT".to_string(),
            Self::Networking => "CAPABILITY_NETWORKING_ENDPOINT".to_string(),
            Self::Custom(name) => format!("CAPABILITY_{}_ENDPOINT", name.to_uppercase()),
        }
    }

    /// Get capability name as string
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Security => "security",
            Self::Storage => "storage",
            Self::Compute => "compute",
            Self::Ai => "ai",
            Self::Orchestration => "orchestration",
            Self::Observability => "observability",
            Self::Networking => "networking",
            Self::Custom(name) => name,
        }
    }
}

/// Discovered capability endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityEndpoint {
    /// Capability type
    pub capability: CapabilityType,
    /// Endpoint URL
    pub endpoint: String,
    /// Provider ID (if known)
    pub provider_id: Option<String>,
    /// Discovery method used
    pub discovery_method: DiscoveryMethod,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// When discovered
    pub discovered_at: std::time::SystemTime,
}

/// How the endpoint was discovered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// From environment variable
    Environment,
    /// From service registry
    ServiceRegistry,
    /// From container metadata
    ContainerMetadata,
    /// From DNS discovery
    Dns,
    /// From network scan
    NetworkScan,
    /// From configuration file
    ConfigFile,
}
