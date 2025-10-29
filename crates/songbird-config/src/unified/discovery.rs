//! Discovery configuration structures

use serde::{Deserialize, Serialize};
use std::env;

/// Discovery configuration (backward compatibility alias)
pub type DiscoveryConfig = UnifiedDiscoveryConfig;

/// Unified discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedDiscoveryConfig {
    pub service_discovery: ServiceDiscoveryConfig,
    pub capability_discovery: CapabilityDiscoveryConfig,
    pub network_discovery: NetworkDiscoveryConfig,
    // Add missing field for backward compatibility
    pub auto_discovery: bool,
    // Legacy fields for backward compatibility
    pub common_ports: Vec<u16>,
    pub scan_timeout: std::time::Duration,
}

impl Default for UnifiedDiscoveryConfig {
    fn default() -> Self {
        Self {
            service_discovery: ServiceDiscoveryConfig::default(),
            capability_discovery: CapabilityDiscoveryConfig::default(),
            network_discovery: NetworkDiscoveryConfig::default(),
            auto_discovery: env::var("SONGBIRD_AUTO_DISCOVERY")
                .unwrap_or_else(|_| "true".to_string()
                == "true",
            common_ports: vec![22, 80, 443, 8080, 8443, 3000, 5000, 9090],
            scan_timeout: std::time::Duration::from_secs(5),
        }
    }
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    pub enabled: bool,
    pub discovery_interval_secs: u64,
    pub max_concurrent_discoveries: usize,
    pub discovery_timeout_secs: u64,
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            discovery_interval_secs: 30,
            max_concurrent_discoveries:
                crate::config::constants::network::discovery::DEFAULT_MAX_CONCURRENT_DISCOVERIES,
            discovery_timeout_secs:
                crate::config::constants::network::CAPABILITY_DISCOVERY_TIMEOUT_SECS,
        }
    }
}

/// Capability discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDiscoveryConfig {
    pub enabled: bool,
    pub cache_ttl_secs: u64,
    pub discovery_batch_size: usize,
    pub max_retry_attempts: usize,
}

impl Default for CapabilityDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_ttl_secs: crate::config::constants::network::CAPABILITY_CACHE_TTL_SECS,
            discovery_batch_size:
                crate::config::constants::network::discovery::DEFAULT_DISCOVERY_BATCH_SIZE,
            max_retry_attempts:
                crate::config::constants::network::discovery::MAX_DISCOVERY_RETRY_ATTEMPTS,
        }
    }
}

/// Network discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDiscoveryConfig {
    pub enabled: bool,
    pub scan_local_network: bool,
    pub scan_ports: Vec<u16>,
    pub discovery_protocols: Vec<String>,
}

impl Default for NetworkDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: env::var("SONGBIRD_NETWORK_DISCOVERY_ENABLED").is_ok(),
            scan_local_network: env::var("SONGBIRD_SCAN_LOCAL_NETWORK").is_ok(),
            scan_ports: vec![8080, 8443, 9090, 3000], // Common service ports
            discovery_protocols: vec!["http".to_string(), "https".to_string()],
        }
    }
}
