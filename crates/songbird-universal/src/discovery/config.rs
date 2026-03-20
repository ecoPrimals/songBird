// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery Configuration Types
//!
//! EVOLVED: Modern capability-based discovery configuration
//! Aligns with canonical config patterns while maintaining flexibility

use tokio::time::Duration;

/// Discovery configuration for universal adapters
///
/// **CANONICAL ALIGNMENT**: Nested structure mirrors canonical pattern
/// Maps to canonical discovery configs:
/// - `enable_environment_scan` → `capability_discovery.enabled`
/// - `enable_network_scanning` → `network_discovery.scan_local_network`
/// - `enable_container_discovery` → `service_discovery.enabled`
/// - `timeout` → `scan_timeout_secs` (Duration vs u64)
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Discovery mechanisms to enable
    pub mechanisms: DiscoveryMechanisms,
    /// Timeout for discovery operations
    pub timeout: Duration,
}

/// Discovery mechanisms configuration
///
/// **ARCHITECTURAL ALIGNMENT**: Nested approach mirrors canonical design
/// Each boolean maps to a specific canonical config's `enabled` field
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

impl Default for DiscoveryMechanisms {
    fn default() -> Self {
        Self {
            enable_environment_scan: true,
            enable_network_scanning: true,
            enable_container_discovery: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert_eq!(config.timeout.as_secs(), 30);
        assert!(config.mechanisms.enable_environment_scan);
        assert!(config.mechanisms.enable_network_scanning);
        assert!(config.mechanisms.enable_container_discovery);
    }

    #[test]
    fn test_discovery_mechanisms_default() {
        let mechanisms = DiscoveryMechanisms::default();
        assert!(mechanisms.enable_environment_scan);
        assert!(mechanisms.enable_network_scanning);
        assert!(mechanisms.enable_container_discovery);
    }

    #[test]
    fn test_discovery_mechanisms_custom() {
        let mechanisms = DiscoveryMechanisms {
            enable_environment_scan: true,
            enable_network_scanning: false,
            enable_container_discovery: true,
        };

        assert!(mechanisms.enable_environment_scan);
        assert!(!mechanisms.enable_network_scanning);
        assert!(mechanisms.enable_container_discovery);
    }

    #[test]
    fn test_discovery_config_custom_timeout() {
        let config = DiscoveryConfig {
            mechanisms: DiscoveryMechanisms::default(),
            timeout: Duration::from_secs(60),
        };

        assert_eq!(config.timeout.as_secs(), 60);
    }

    #[test]
    fn test_discovery_config_all_disabled() {
        let mechanisms = DiscoveryMechanisms {
            enable_environment_scan: false,
            enable_network_scanning: false,
            enable_container_discovery: false,
        };

        let config = DiscoveryConfig {
            mechanisms,
            timeout: Duration::from_secs(10),
        };

        assert!(!config.mechanisms.enable_environment_scan);
        assert!(!config.mechanisms.enable_network_scanning);
        assert!(!config.mechanisms.enable_container_discovery);
    }

    #[test]
    fn test_discovery_config_clone() {
        let config1 = DiscoveryConfig::default();
        let config2 = config1.clone();

        assert_eq!(config1.timeout, config2.timeout);
        assert_eq!(
            config1.mechanisms.enable_environment_scan,
            config2.mechanisms.enable_environment_scan
        );
    }
}
