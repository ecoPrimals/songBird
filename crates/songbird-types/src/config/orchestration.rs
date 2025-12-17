//! Orchestration /// Configuration capability Configuration
//!
//! Configuration for service discovery, networking, and orchestration.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{
    health::CanonicalHealthConfig,
    network::{CanonicalNetworkConfig, NetworkDiscoveryConfig},
    security::CanonicalSecurityConfig,
};

/// **CANONICAL**: Orchestration configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CanonicalOrchestrationConfig {
    /// Network configuration
    pub network: CanonicalNetworkConfig,
    /// Security configuration
    pub security: CanonicalSecurityConfig,
    /// Service discovery configuration
    pub discovery: NetworkDiscoveryConfig,
    /// Health monitoring configuration
    pub health: CanonicalHealthConfig,
    /// Federation configuration
    /// Federation field
    pub federation: CanonicalOrchestrationFederationConfig,
}

/// **CANONICAL**: Basic Federation configuration for orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalOrchestrationFederationConfig {
    /// Enable federation
    /// Enabled field
    pub enabled: bool,
    /// Federation discovery interval
    /// Discovery Interval field
    pub discovery_interval: Duration,
    /// Federation heartbeat interval
    /// Heartbeat Interval field
    pub heartbeat_interval: Duration,
    /// Maximum federation nodes
    pub max_nodes: usize,
    /// Federation protocol (gossip, raft, custom)
    /// Protocol field
    pub protocol: String,
    /// Enable cross-federation communication;
    /// Cross Federation field
    pub cross_federation: bool,
}
impl Default for CanonicalOrchestrationFederationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            discovery_interval: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(10),
            max_nodes: 100,
            protocol: "gossip".to_string(),
            cross_federation: false,
        }
    }
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_orchestration_config() {
        let config = CanonicalOrchestrationConfig::default();
        // Verify all sub-configs are properly initialized
        assert!(config.health.enabled);
        assert!(config.federation.enabled);
    }

    #[test]
    fn test_default_federation_config() {
        let config = CanonicalOrchestrationFederationConfig::default();
        assert!(config.enabled);
        assert_eq!(config.discovery_interval, Duration::from_secs(30));
        assert_eq!(config.heartbeat_interval, Duration::from_secs(10));
        assert_eq!(config.max_nodes, 100);
        assert_eq!(config.protocol, "gossip");
        assert!(!config.cross_federation);
    }

    #[test]
    fn test_custom_federation_config() {
        let config = CanonicalOrchestrationFederationConfig {
            enabled: false,
            discovery_interval: Duration::from_secs(60),
            heartbeat_interval: Duration::from_secs(5),
            max_nodes: 50,
            protocol: "raft".to_string(),
            cross_federation: true,
        };
        assert!(!config.enabled);
        assert_eq!(config.discovery_interval, Duration::from_secs(60));
        assert_eq!(config.heartbeat_interval, Duration::from_secs(5));
        assert_eq!(config.max_nodes, 50);
        assert_eq!(config.protocol, "raft");
        assert!(config.cross_federation);
    }

    #[test]
    fn test_federation_protocols() {
        let gossip = CanonicalOrchestrationFederationConfig {
            protocol: "gossip".to_string(),
            ..Default::default()
        };
        assert_eq!(gossip.protocol, "gossip");

        let raft = CanonicalOrchestrationFederationConfig {
            protocol: "raft".to_string(),
            ..Default::default()
        };
        assert_eq!(raft.protocol, "raft");

        let custom = CanonicalOrchestrationFederationConfig {
            protocol: "custom".to_string(),
            ..Default::default()
        };
        assert_eq!(custom.protocol, "custom");
    }

    #[test]
    fn test_federation_intervals_valid() {
        let config = CanonicalOrchestrationFederationConfig::default();
        // Discovery should be longer than heartbeat
        assert!(config.discovery_interval > config.heartbeat_interval);
    }

    #[test]
    fn test_federation_config_clone() {
        let config1 = CanonicalOrchestrationFederationConfig::default();
        let config2 = config1.clone();
        assert_eq!(config1.enabled, config2.enabled);
        assert_eq!(config1.max_nodes, config2.max_nodes);
        assert_eq!(config1.protocol, config2.protocol);
    }

    #[test]
    fn test_orchestration_config_clone() {
        let config1 = CanonicalOrchestrationConfig::default();
        let config2 = config1.clone();
        assert_eq!(config1.health.enabled, config2.health.enabled);
        assert_eq!(config1.federation.enabled, config2.federation.enabled);
    }

    #[test]
    fn test_federation_config_debug() {
        let config = CanonicalOrchestrationFederationConfig::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("CanonicalOrchestrationFederationConfig"));
        assert!(debug_str.contains("enabled"));
        assert!(debug_str.contains("protocol"));
    }

    #[test]
    fn test_max_nodes_reasonable() {
        let config = CanonicalOrchestrationFederationConfig::default();
        assert!(config.max_nodes > 0);
        assert!(config.max_nodes <= 1000); // Reasonable upper limit
    }

    #[test]
    fn test_federation_disabled_scenario() {
        let config = CanonicalOrchestrationFederationConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(!config.enabled);
        // Other fields should still be valid even when disabled
        assert_eq!(config.max_nodes, 100);
        assert_eq!(config.protocol, "gossip");
    }
}
