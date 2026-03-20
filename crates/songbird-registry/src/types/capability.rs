// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Plugin capability types
//!
//! Defines what a plugin can do and how capabilities are represented.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of capability a plugin can provide
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CapabilityType {
    /// Encryption capability
    Encryption {
        /// Supported algorithms
        algorithms: Vec<String>,
        /// Key sizes supported
        key_sizes: Vec<u32>,
    },

    /// Service discovery capability
    ServiceDiscovery {
        /// Supported protocols (mdns, dns-sd, etc.)
        protocols: Vec<String>,
    },

    /// Compute capability
    Compute {
        /// CPU cores available
        cpu_cores: u32,
        /// Memory in GB
        memory_gb: u32,
    },

    /// Network capability
    Network {
        /// Bandwidth in Mbps
        bandwidth_mbps: u64,
        /// Latency in ms
        latency_ms: u64,
    },

    /// Storage capability
    Storage {
        /// Storage size in GB
        size_gb: u64,
        /// Storage type (ssd, hdd, nvme)
        storage_type: String,
    },

    /// Custom capability
    Custom {
        /// Capability name
        name: String,
        /// Arbitrary attributes
        attributes: HashMap<String, String>,
    },
}

/// A capability that a plugin provides
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub struct Capability {
    /// The type and details of this capability
    #[serde(flatten)]
    pub capability_type: CapabilityType,

    /// Version of this capability (for compatibility)
    pub version: String,

    /// Whether this capability is currently active
    #[serde(default = "default_true")]
    pub active: bool,
}

const fn default_true() -> bool {
    true
}

impl Capability {
    /// Create a new capability
    #[must_use]
    pub fn new(capability_type: CapabilityType) -> Self {
        Self {
            capability_type,
            version: String::from("1.0.0"),
            active: true,
        }
    }

    /// Set the version of this capability
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set whether this capability is active
    #[must_use]
    pub const fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Check if this capability is compatible with another
    #[must_use]
    pub fn compatible_with(&self, other: &Self) -> bool {
        // For now, just check if types match
        // Could be extended with version compatibility checking
        std::mem::discriminant(&self.capability_type)
            == std::mem::discriminant(&other.capability_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_creation() {
        let cap = Capability::new(CapabilityType::Encryption {
            algorithms: vec!["aes256".to_string()],
            key_sizes: vec![256],
        });

        assert!(cap.active);
        assert_eq!(cap.version, "1.0.0");
    }

    #[test]
    fn test_capability_compatibility() {
        let cap1 = Capability::new(CapabilityType::Encryption {
            algorithms: vec!["aes256".to_string()],
            key_sizes: vec![256],
        });

        let cap2 = Capability::new(CapabilityType::Encryption {
            algorithms: vec!["aes128".to_string()],
            key_sizes: vec![128],
        });

        assert!(cap1.compatible_with(&cap2));
    }
}
