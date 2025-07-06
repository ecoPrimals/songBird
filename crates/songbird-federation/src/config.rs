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
    pub cluster_id: Option<String>,

    /// This node's ID in the federation
    pub node_id: Option<String>,

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
            cluster_id: None,
            node_id: None,
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
    pub heartbeat_interval: u64,

    /// Connection timeout in seconds
    pub connection_timeout: u64,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Auto-discovery enabled
    pub auto_discovery: bool,

    /// Node identifier
    pub node_id: Option<String>,

    /// Cluster identifier
    pub cluster_id: Option<String>,
}

impl Default for FederationConfig {
    fn default() -> Self {
        Self {
            cluster_endpoints: vec![],
            heartbeat_interval: 30,
            connection_timeout: 10,
            max_retries: 3,
            auto_discovery: true,
            node_id: None,
            cluster_id: None,
        }
    }
}
