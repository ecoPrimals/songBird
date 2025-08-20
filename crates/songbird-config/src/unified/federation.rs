//! Federation configuration structures

use serde::{Deserialize, Serialize};
use std::env;
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
        Self {
            enabled: env::var("SONGBIRD_FEDERATION_ENABLED").is_ok(),
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

    /// Public addresses (for internet connectivity)
    pub public_addresses: Vec<SocketAddr>,

    /// Location information
    pub location: Option<String>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        let default_port = env::var("SONGBIRD_FEDERATION_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(7000);

        Self {
            node_id: env::var("SONGBIRD_NODE_ID")
                .unwrap_or_else(|_| format!("node_{}", std::process::id())),
            name: env::var("SONGBIRD_NODE_NAME").unwrap_or_else(|_| "songbird-node".to_string()),
            node_type: NodeType::Standard,
            listen_addresses: vec![format!("0.0.0.0:{default_port}")
                .parse()
                .or_else(|_| "0.0.0.0:7000".parse())
                .unwrap_or_else(|_| std::net::SocketAddr::from(([0, 0, 0, 0], 7000)))],
            public_addresses: Vec::new(),
            location: env::var("SONGBIRD_NODE_LOCATION").ok(),
        }
    }
}

/// Node type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    /// Standard federation node
    Standard,
    /// Leader/coordinator node
    Leader,
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

    /// Cluster endpoints for discovery (from deprecated `FederationConfig`)
    pub cluster_endpoints: Vec<String>,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Connection timeout in seconds
    pub connection_timeout: u64,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            cluster_id: env::var("SONGBIRD_CLUSTER_ID")
                .unwrap_or_else(|_| format!("cluster_{}", std::process::id())),
            cluster_name: env::var("SONGBIRD_CLUSTER_NAME").ok(),
            cluster_endpoints: env::var("SONGBIRD_CLUSTER_ENDPOINTS")
                .ok()
                .map(|endpoints| endpoints.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default(),
            max_retries: env::var("SONGBIRD_FEDERATION_MAX_RETRIES")
                .ok()
                .and_then(|r| r.parse().ok())
                .unwrap_or(3),
            connection_timeout: env::var("SONGBIRD_FEDERATION_TIMEOUT")
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
            discovery_port: env::var("SONGBIRD_DISCOVERY_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(7000),
            port: env::var("SONGBIRD_FEDERATION_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(7001),
            bind_address: env::var("SONGBIRD_FEDERATION_BIND")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
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

    /// Heartbeat interval in seconds (from deprecated `FederationConfig`)
    pub heartbeat_interval_secs: u64,

    /// Maximum cluster size
    pub max_cluster_size: usize,
}

impl Default for ClusterDiscoveryConfig {
    fn default() -> Self {
        Self {
            auto_discovery: env::var("SONGBIRD_AUTO_DISCOVERY")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
            discovery_interval_secs: env::var("SONGBIRD_DISCOVERY_INTERVAL")
                .ok()
                .and_then(|i| i.parse().ok())
                .unwrap_or(30),
            heartbeat_interval_secs: env::var("SONGBIRD_HEARTBEAT_INTERVAL")
                .ok()
                .and_then(|i| i.parse().ok())
                .unwrap_or(10),
            max_cluster_size: env::var("SONGBIRD_MAX_CLUSTER_SIZE")
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
            algorithm: "raft".to_string(),
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
            enabled: env::var("SONGBIRD_REPLICATION_ENABLED").is_ok(),
            replication_factor: 3,
            consistency_level: "eventual".to_string(),
        }
    }
}
