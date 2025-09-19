/*!
 * Federation Configuration Types
 *
 * This module contains all configuration structures and enums for the federation system:
 * - Federation operating modes
 * - Configuration structures
 * - Status tracking
 */

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Federation operating modes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FederationMode {
    /// Standalone mode - no federation
    Standalone,

    /// Client mode - connect to existing federation
    Client,

    /// Server mode - act as federation coordinator
    Server,

    /// Hybrid mode - can act as both client and server
    Hybrid,

    /// Clustered mode - participate in federation cluster
    Clustered,
}

impl Default for FederationMode {
    fn default() -> Self {
        Self::Standalone
    }
}

/// Federation connection and cluster status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationStatus {
    /// Whether federation is enabled
    pub enabled: bool,

    /// Whether connected to federation cluster
    pub connected: bool,

    /// Number of nodes in the federation
    pub node_count: u32,

    /// Last successful heartbeat timestamp
    pub last_heartbeat: Option<DateTime<Utc>>,

    /// Federation cluster ID
    pub cluster_id: String,

    /// This node's ID in the federation
    pub node_id: String,

    /// Federation protocol version
    pub protocol_version: String,
}

impl Default for FederationStatus {
    fn default() -> Self {
        Self {
            enabled: false,
            connected: false,
            node_count: 0,
            last_heartbeat: None,
            cluster_id: "default-cluster".to_string(),
            node_id: Uuid::new_v4().to_string(),
            protocol_version: "1.0".to_string(),
        }
    }
}

/// Federation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationConfig {
    /// Federation cluster endpoints
    pub cluster_endpoints: Vec<String>,

    /// Heartbeat interval in seconds
    pub heartbeat_interval: Option<u32>,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Auto-discovery enabled
    pub auto_discovery: bool,

    /// Node identifier
    pub node_id: String,

    /// Cluster identifier
    pub cluster_id: String,

    /// Discovery port for UDP broadcasts
    pub discovery_port: Option<u16>,

    /// Main service port
    pub port: Option<u16>,
}

impl Default for FederationConfig {
    fn default() -> Self {
        Self {
            cluster_endpoints: vec![],
            heartbeat_interval: Some(30),
            connection_timeout: 10,
            max_retries: 3,
            auto_discovery: true,
            node_id: Uuid::new_v4().to_string(),
            cluster_id: "default-cluster".to_string(),
            discovery_port: Some(8765),
            port: Some(8080),
        }
    }
}

impl FederationConfig {
    /// Create new federation config with cluster and node IDs
    pub fn new(cluster_id: String, node_id: String) -> Self {
        Self {
            cluster_id,
            node_id,
            ..Default::default()
        }
    }

    /// Create new federation config with auto-generated IDs
    pub fn new_with_auto_ids() -> Self {
        Self {
            cluster_id: format!("cluster-{}", Uuid::new_v4()),
            node_id: Uuid::new_v4().to_string(),
            ..Default::default()
        }
    }

    /// Add cluster endpoint
    pub fn add_endpoint(mut self, endpoint: String) -> Self {
        self.cluster_endpoints.push(endpoint);
        self
    }

    /// Set heartbeat interval
    pub fn with_heartbeat_interval(mut self, interval: u32) -> Self {
        self.heartbeat_interval = Some(interval);
        self
    }

    /// Enable/disable auto-discovery
    pub fn with_auto_discovery(mut self, enabled: bool) -> Self {
        self.auto_discovery = enabled;
        self
    }

    /// Set discovery port
    pub fn with_discovery_port(mut self, port: u16) -> Self {
        self.discovery_port = Some(port);
        self
    }

    /// Set main service port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }
}
