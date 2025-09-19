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
