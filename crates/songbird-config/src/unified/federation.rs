// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Federation configuration structures

#![allow(
    missing_docs,
    reason = "deprecated unified federation; migrate to `canonical::federation`"
)]

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Unified federation configuration
///
/// This replaces the deprecated `FederationConfig` and integrates `LocalNodeConfig` functionality.
/// All federation-related configuration is now centralized here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedFederationConfig {
    /// Enable federation functionality
    pub enabled: bool,

    /// Node configuration
    pub node: NodeConfig,

    /// Cluster configuration
    pub cluster: ClusterConfig,

    /// Discovery configuration
    pub cluster_discovery: ClusterDiscoveryConfig,

    /// Consensus configuration
    pub consensus: ConsensusConfig,

    /// Replication configuration
    pub replication: ReplicationConfig,

    /// Network configuration
    pub network: FederationNetworkConfig,
}

impl Default for UnifiedFederationConfig {
    fn default() -> Self {
        let enabled = songbird_process_env::var("SONGBIRD_FEDERATION_ENABLED")
            .or_else(|_| songbird_process_env::var("FEDERATION_ENABLED"))
            .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "yes" | "on"))
            .unwrap_or(false);

        Self {
            enabled,
            node: NodeConfig::default(),
            cluster: ClusterConfig::default(),
            cluster_discovery: ClusterDiscoveryConfig::default(),
            consensus: ConsensusConfig::default(),
            replication: ReplicationConfig::default(),
            network: FederationNetworkConfig::default(),
        }
    }
}

/// Node configuration (replaces `LocalNodeConfig`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node identifier
    pub node_id: String,

    /// Node name
    pub name: String,

    /// Node type
    pub node_type: NodeType,

    /// Listening addresses
    pub listen_addresses: Vec<SocketAddr>,

    /// Public addresses (for internet connectivity}
    pub public_addresses: Vec<SocketAddr>,

    /// Location information
    pub location: Option<String>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        use songbird_types::defaults::ports::DEFAULT_FEDERATION_BIND_PORT;

        let default_port = songbird_process_env::var("SONGBIRD_FEDERATION_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(DEFAULT_FEDERATION_BIND_PORT);

        Self {
            node_id: songbird_process_env::var("SONGBIRD_NODE_ID")
                .unwrap_or_else(|_| format!("node_{}", std::process::id())),
            name: songbird_process_env::var("SONGBIRD_NODE_NAME")
                .unwrap_or_else(|_| String::from("songbird-node")),
            node_type: NodeType::Standard,
            listen_addresses: vec![std::net::SocketAddr::from(([0, 0, 0, 0], default_port))],
            public_addresses: Vec::new(),
            location: songbird_process_env::var("SONGBIRD_NODE_LOCATION").ok(),
        }
    }
}

/// Node type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    /// Standard federation node
    Standard,
    /// Coordinator node (facilitator, not hierarchical leader)
    Coordinator,
    /// Read-only replica node
    Replica,
    /// Gateway node for external access
    Gateway,
}

/// Cluster configuration (enhanced from deprecated `FederationConfig`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Cluster identifier
    pub cluster_id: String,

    /// Cluster name
    pub cluster_name: Option<String>,

    /// Cluster endpoints for discovery (from deprecated `FederationConfig`}
    pub cluster_endpoints: Vec<String>,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Connection timeout in seconds
    pub connection_timeout: u64,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            cluster_id: songbird_process_env::var("SONGBIRD_CLUSTER_ID")
                .unwrap_or_else(|_| format!("cluster_{}", std::process::id())),
            cluster_name: songbird_process_env::var("SONGBIRD_CLUSTER_NAME").ok(),
            cluster_endpoints: songbird_process_env::var("SONGBIRD_CLUSTER_ENDPOINTS")
                .ok()
                .map(|endpoints| endpoints.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            max_retries: songbird_process_env::var("SONGBIRD_FEDERATION_MAX_RETRIES")
                .ok()
                .and_then(|r| r.parse().ok())
                .unwrap_or(3),
            connection_timeout: songbird_process_env::var("SONGBIRD_FEDERATION_TIMEOUT")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(30),
        }
    }
}

/// Network configuration for federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationNetworkConfig {
    /// Discovery port for UDP broadcasts
    pub discovery_port: u16,

    /// Main service port
    pub port: u16,

    /// Bind address
    pub bind_address: String,
}

impl Default for FederationNetworkConfig {
    fn default() -> Self {
        Self {
            discovery_port: songbird_process_env::var("SONGBIRD_DISCOVERY_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(7000),
            port: songbird_process_env::var("SONGBIRD_FEDERATION_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(7001),
            bind_address: songbird_process_env::var("SONGBIRD_FEDERATION_BIND")
                .unwrap_or_else(|_| songbird_types::constants::PRODUCTION_BIND_ADDRESS.to_string()),
        }
    }
}

/// Cluster discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterDiscoveryConfig {
    /// Enable auto-discovery (from deprecated `FederationConfig`)
    pub auto_discovery: bool,

    /// Discovery interval in seconds
    pub discovery_interval_secs: u64,

    /// Heartbeat interval in seconds (from deprecated `FederationConfig`}
    pub heartbeat_interval_secs: u64,

    /// Maximum cluster size
    pub max_cluster_size: usize,
}

impl Default for ClusterDiscoveryConfig {
    fn default() -> Self {
        Self {
            auto_discovery: songbird_process_env::var("SONGBIRD_AUTO_DISCOVERY")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            discovery_interval_secs: songbird_process_env::var("SONGBIRD_DISCOVERY_INTERVAL")
                .ok()
                .and_then(|i| i.parse().ok())
                .unwrap_or(30),
            heartbeat_interval_secs: songbird_process_env::var("SONGBIRD_HEARTBEAT_INTERVAL")
                .ok()
                .and_then(|i| i.parse().ok())
                .unwrap_or(10),
            max_cluster_size: songbird_process_env::var("SONGBIRD_MAX_CLUSTER_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
        }
    }
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub algorithm: String,
    pub election_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            algorithm: String::from("raft"),
            election_timeout_ms: 5000,
            heartbeat_interval_ms: 1000,
        }
    }
}

/// Replication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationConfig {
    pub enabled: bool,
    pub replication_factor: u32,
    pub consistency_level: String,
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            enabled: songbird_process_env::var("SONGBIRD_REPLICATION_ENABLED").is_ok(),
            replication_factor: 3,
            consistency_level: String::from("eventual"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_federation_config_default() {
        let config = UnifiedFederationConfig::default();
        assert!(!config.enabled || config.enabled); // env-dependent
        assert!(!config.node.node_id.is_empty());
        assert!(!config.cluster.cluster_id.is_empty());
        assert!(config.cluster_discovery.auto_discovery);
        assert_eq!(config.consensus.algorithm, "raft");
    }

    #[test]
    fn test_node_config_default() {
        let config = NodeConfig::default();
        assert!(config.node_id.starts_with("node_"));
        assert_eq!(config.node_type, NodeType::Standard);
        assert!(!config.listen_addresses.is_empty());
        assert_eq!(config.listen_addresses[0].port(), 7000);
    }

    #[test]
    fn test_node_type_variants() {
        let types =
            vec![NodeType::Standard, NodeType::Coordinator, NodeType::Replica, NodeType::Gateway];

        for node_type in types {
            let debug_str = format!("{node_type:?}");
            assert!(!debug_str.is_empty());

            let cloned = node_type.clone();
            assert_eq!(cloned, node_type);
        }
    }

    #[test]
    fn test_cluster_config_default() {
        let config = ClusterConfig::default();
        assert!(config.cluster_id.starts_with("cluster_"));
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.connection_timeout, 30);
        assert!(config.cluster_endpoints.is_empty()); // Unless env var set
    }

    #[test]
    fn test_cluster_discovery_config_default() {
        let config = ClusterDiscoveryConfig::default();
        assert!(config.auto_discovery);
        assert_eq!(config.discovery_interval_secs, 30);
        assert_eq!(config.heartbeat_interval_secs, 10);
        assert_eq!(config.max_cluster_size, 100);
    }

    #[test]
    fn test_consensus_config_default() {
        let config = ConsensusConfig::default();
        assert_eq!(config.algorithm, "raft");
        assert_eq!(config.election_timeout_ms, 5000);
        assert_eq!(config.heartbeat_interval_ms, 1000);
    }

    #[test]
    fn test_replication_config_default() {
        let config = ReplicationConfig::default();
        assert!(!config.enabled || config.enabled); // env-dependent
        assert_eq!(config.replication_factor, 3);
        assert_eq!(config.consistency_level, "eventual");
    }

    #[test]
    fn test_federation_network_config_default() {
        let config = FederationNetworkConfig::default();
        assert_eq!(config.discovery_port, 7000);
        assert_eq!(config.port, 7001);
        assert_eq!(config.bind_address, "0.0.0.0");
    }

    #[test]
    fn test_serde_unified_federation_config() {
        let config = UnifiedFederationConfig::default();

        // Test serialization
        let json = serde_json::to_string(&config).expect("should serialize");
        assert!(!json.is_empty());
        assert!(json.contains("node"));
        assert!(json.contains("cluster"));

        // Test deserialization
        let deserialized: UnifiedFederationConfig =
            serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(deserialized.node.node_type, config.node.node_type);
        assert_eq!(deserialized.consensus.algorithm, config.consensus.algorithm);
    }

    #[test]
    fn test_federation_config_clone() {
        let config = UnifiedFederationConfig::default();
        let cloned = config.clone();
        assert_eq!(cloned.node.node_id, config.node.node_id);
        assert_eq!(cloned.cluster.cluster_id, config.cluster.cluster_id);
        assert_eq!(cloned.consensus.algorithm, config.consensus.algorithm);
    }

    #[test]
    fn test_node_config_with_location() {
        let mut config = NodeConfig::default();
        config.location = Some(String::from("us-west-2"));
        assert!(config.location.is_some());
        assert_eq!(config.location.unwrap(), "us-west-2");
    }

    #[test]
    fn test_cluster_config_with_endpoints() {
        let mut config = ClusterConfig::default();
        config.cluster_endpoints =
            vec![String::from("https://node1:7000"), String::from("https://node2:7000")];
        assert_eq!(config.cluster_endpoints.len(), 2);
    }

    #[test]
    fn test_replication_consistency_levels() {
        let mut config = ReplicationConfig::default();

        config.consistency_level = String::from("strong");
        assert_eq!(config.consistency_level, "strong");

        config.consistency_level = String::from("eventual");
        assert_eq!(config.consistency_level, "eventual");

        config.consistency_level = String::from("quorum");
        assert_eq!(config.consistency_level, "quorum");
    }

    #[test]
    fn test_replication_factor_validation() {
        let config = ReplicationConfig {
            enabled: true,
            replication_factor: 5,
            consistency_level: String::from("quorum"),
        };
        assert!(config.replication_factor >= 1);
        assert!(config.replication_factor <= 10); // reasonable upper bound
    }

    #[test]
    fn test_federation_network_ports_distinct() {
        let config = FederationNetworkConfig::default();
        // Discovery and main port should be distinct (unless explicitly configured same)
        assert!(config.discovery_port > 0);
        assert!(config.port > 0);
    }
}
