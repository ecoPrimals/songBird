// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery Configuration Module
//!
//! Canonical discovery configuration for service discovery, capability discovery,
//! and network discovery across the Songbird ecosystem.

use serde::{Deserialize, Serialize};

// ============================================================================
// DISCOVERY CONFIGURATION
// ============================================================================

/// Unified discovery configuration
///
/// **Canonical Source**: This is the definitive discovery configuration\
/// **Migrated from**: `unified/discovery.rs`\
/// **Purpose**: Configure all discovery mechanisms (service, capability, network)
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::discovery::DiscoveryConfig;
///
/// let config = DiscoveryConfig::default();
/// assert!(config.service_discovery.enabled);
/// assert!(config.capability_discovery.enabled);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Service discovery configuration
    pub service_discovery: ServiceDiscoveryConfig,

    /// Capability discovery configuration
    pub capability_discovery: CapabilityDiscoveryConfig,

    /// Network discovery configuration
    pub network_discovery: NetworkDiscoveryConfig,

    /// Enable automatic discovery
    pub auto_discovery: bool,

    /// Common ports to scan
    pub common_ports: Vec<u16>,

    /// Scan timeout in seconds
    pub scan_timeout_secs: u64,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            service_discovery: ServiceDiscoveryConfig::default(),
            capability_discovery: CapabilityDiscoveryConfig::default(),
            network_discovery: NetworkDiscoveryConfig::default(),
            auto_discovery: songbird_process_env::var("SONGBIRD_AUTO_DISCOVERY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            common_ports: songbird_process_env::var("SONGBIRD_COMMON_PORTS")
                .ok()
                .and_then(|s| {
                    s.split(',').filter_map(|p| p.trim().parse().ok()).collect::<Vec<u16>>().into()
                })
                .unwrap_or_else(|| vec![22, 80, 443, 8080, 8443, 3000, 5000, 9090]),
            scan_timeout_secs: songbird_process_env::var("SONGBIRD_SCAN_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
        }
    }
}

// ============================================================================
// SERVICE DISCOVERY CONFIGURATION
// ============================================================================

/// Service discovery configuration
///
/// **Migrated from**: `unified/discovery.rs`\
/// **Purpose**: Configure automatic service discovery and registration
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::discovery::ServiceDiscoveryConfig;
///
/// let config = ServiceDiscoveryConfig {
///     enabled: true,
///     discovery_interval_secs: 60,
///     max_concurrent_discoveries: 20,
///     discovery_timeout_secs: 10,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceDiscoveryConfig {
    /// Enable service discovery
    pub enabled: bool,

    /// Discovery interval in seconds
    pub discovery_interval_secs: u64,

    /// Maximum concurrent discovery operations
    pub max_concurrent_discoveries: usize,

    /// Discovery operation timeout in seconds
    pub discovery_timeout_secs: u64,
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: songbird_process_env::var("SONGBIRD_SERVICE_DISCOVERY_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            discovery_interval_secs: songbird_process_env::var("SONGBIRD_DISCOVERY_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            max_concurrent_discoveries: songbird_process_env::var(
                "SONGBIRD_MAX_CONCURRENT_DISCOVERIES",
            )
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10),
            discovery_timeout_secs: songbird_process_env::var("SONGBIRD_DISCOVERY_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
        }
    }
}

// ============================================================================
// CAPABILITY DISCOVERY CONFIGURATION
// ============================================================================

/// Capability discovery configuration
///
/// **Migrated from**: `unified/discovery.rs`\
/// **Purpose**: Configure capability-based service discovery
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::discovery::CapabilityDiscoveryConfig;
///
/// let config = CapabilityDiscoveryConfig {
///     enabled: true,
///     cache_ttl_secs: 600,
///     discovery_batch_size: 20,
///     max_retry_attempts: 5,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityDiscoveryConfig {
    /// Enable capability discovery
    pub enabled: bool,

    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,

    /// Discovery batch size
    pub discovery_batch_size: usize,

    /// Maximum retry attempts
    pub max_retry_attempts: usize,
}

impl Default for CapabilityDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: songbird_process_env::var("SONGBIRD_CAPABILITY_DISCOVERY_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            cache_ttl_secs: songbird_process_env::var("SONGBIRD_CAPABILITY_CACHE_TTL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
            discovery_batch_size: songbird_process_env::var("SONGBIRD_DISCOVERY_BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            max_retry_attempts: songbird_process_env::var("SONGBIRD_MAX_RETRY_ATTEMPTS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3),
        }
    }
}

// ============================================================================
// NETWORK DISCOVERY CONFIGURATION
// ============================================================================

/// Network discovery configuration
///
/// **Migrated from**: `unified/discovery.rs`\
/// **Purpose**: Configure network scanning and protocol discovery
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::discovery::NetworkDiscoveryConfig;
///
/// let config = NetworkDiscoveryConfig {
///     enabled: true,
///     scan_local_network: true,
///     scan_ports: vec![8080, 8443, 9090],
///     discovery_protocols: vec!["http".to_string(), "https".to_string()],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkDiscoveryConfig {
    /// Enable network discovery
    pub enabled: bool,

    /// Scan local network for services
    pub scan_local_network: bool,

    /// Ports to scan
    pub scan_ports: Vec<u16>,

    /// Discovery protocols to use
    pub discovery_protocols: Vec<String>,
}

impl Default for NetworkDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: songbird_process_env::var("SONGBIRD_NETWORK_DISCOVERY_ENABLED")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
            scan_local_network: songbird_process_env::var("SONGBIRD_SCAN_LOCAL_NETWORK")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
            scan_ports: songbird_process_env::var("SONGBIRD_SCAN_PORTS")
                .ok()
                .and_then(|s| {
                    s.split(',').filter_map(|p| p.trim().parse().ok()).collect::<Vec<u16>>().into()
                })
                .unwrap_or_else(|| vec![8080, 8443, 9090, 3000]),
            discovery_protocols: songbird_process_env::var("SONGBIRD_DISCOVERY_PROTOCOLS")
                .ok()
                .map_or_else(
                    || vec!["http".to_string(), "https".to_string()],
                    |s| s.split(',').map(|p| p.trim().to_string()).collect(),
                ),
        }
    }
}

// ============================================================================
// BACKWARD COMPATIBILITY ALIASES
// ============================================================================

/// Backward compatibility alias for `UnifiedDiscoveryConfig`
pub type UnifiedDiscoveryConfig = DiscoveryConfig;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_discovery_config_default() {
        let config = DiscoveryConfig::default();
        assert!(config.service_discovery.enabled);
        assert!(config.capability_discovery.enabled);
        assert!(config.auto_discovery);
        assert!(!config.common_ports.is_empty());
        assert_eq!(config.scan_timeout_secs, 5);
    }

    #[test]
    fn test_service_discovery_config_default() {
        let config = ServiceDiscoveryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.discovery_interval_secs, 30);
        assert_eq!(config.max_concurrent_discoveries, 10);
        assert_eq!(config.discovery_timeout_secs, 30);
    }

    #[test]
    fn test_capability_discovery_config_default() {
        let config = CapabilityDiscoveryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.cache_ttl_secs, 300);
        assert_eq!(config.discovery_batch_size, 10);
        assert_eq!(config.max_retry_attempts, 3);
    }

    #[test]
    fn test_network_discovery_config_default() {
        let config = NetworkDiscoveryConfig::default();
        // Network discovery is disabled by default for security
        assert!(!config.enabled);
        assert!(!config.scan_local_network);
        assert!(!config.scan_ports.is_empty());
        assert_eq!(config.discovery_protocols.len(), 2);
    }

    #[test]
    fn test_discovery_config_serialization() {
        let config = DiscoveryConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: DiscoveryConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.service_discovery.enabled, deserialized.service_discovery.enabled);
        assert_eq!(config.auto_discovery, deserialized.auto_discovery);
    }

    #[test]
    fn test_service_discovery_config_custom() {
        let config = ServiceDiscoveryConfig {
            enabled: false,
            discovery_interval_secs: 60,
            max_concurrent_discoveries: 20,
            discovery_timeout_secs: 45,
        };

        assert!(!config.enabled);
        assert_eq!(config.discovery_interval_secs, 60);
    }

    #[test]
    fn test_capability_discovery_config_serialization() {
        let config = CapabilityDiscoveryConfig {
            enabled: true,
            cache_ttl_secs: 600,
            discovery_batch_size: 50,
            max_retry_attempts: 5,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: CapabilityDiscoveryConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_network_discovery_config_custom_ports() {
        let config = NetworkDiscoveryConfig {
            enabled: true,
            scan_local_network: true,
            scan_ports: vec![80, 443, 8080],
            discovery_protocols: vec!["http".to_string()],
        };

        assert_eq!(config.scan_ports.len(), 3);
        assert_eq!(config.discovery_protocols.len(), 1);
    }

    #[test]
    fn test_discovery_config_common_ports() {
        let config = DiscoveryConfig::default();
        // Should have common ports configured
        assert!(config.common_ports.contains(&80));
        assert!(config.common_ports.contains(&443));
        assert!(config.common_ports.contains(&8080));
    }

    #[test]
    fn test_unified_discovery_config_alias_equivalence() {
        let u: UnifiedDiscoveryConfig = DiscoveryConfig::default();
        let d: DiscoveryConfig = u;
        assert_eq!(d.scan_timeout_secs, DiscoveryConfig::default().scan_timeout_secs);
    }

    #[test]
    fn test_network_discovery_config_serde_roundtrip() {
        let c = NetworkDiscoveryConfig {
            enabled: true,
            scan_local_network: false,
            scan_ports: vec![1, 2],
            discovery_protocols: vec!["http".to_string()],
        };
        let json = serde_json::to_string(&c).unwrap();
        let back: NetworkDiscoveryConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(c, back);
    }
}
