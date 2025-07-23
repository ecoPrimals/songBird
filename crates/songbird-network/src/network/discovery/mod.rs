//! Network Discovery Engine - FRAGO Implementation
//!
//! Implements the exact NetworkDiscoveryEngine interface specified in the BearDog FRAGO
//! for sub-10ms peer discovery in LAN environments
//!
//! ## Refactored Architecture
//!
//! The network discovery system is organized into focused modules:
//! - `types` - All data structures, enums, and configurations
//! - `engine` - Main NetworkDiscoveryEngine coordinator and core logic  
//! - `upnp` - UPnP client and device discovery
//! - `stun` - STUN client for NAT traversal
//! - `turn` - TURN client for relay connectivity
//! - `peer_registry` - Peer management and capabilities tracking
//! - `topology` - Network topology mapping and measurements

pub mod engine;
pub mod peer_registry;
pub mod stun;
pub mod topology;
pub mod turn;
pub mod types;
pub mod upnp;

// Re-export main types for backward compatibility
pub use types::{
    ConnectionQuality, DiscoveredPeer, DiscoveryConfig, DiscoveryMethod, NetworkConnection,
    NetworkMeasurement, NetworkNode, NetworkTopology, PeerType, TURNRelay, UPnPDevice,
};

pub use engine::{DiscoveryStatistics, DiscoveryTestResults, NetworkDiscoveryEngine};
pub use peer_registry::{PeerRegistry, PeerStatistics};
pub use stun::STUNClient;
pub use topology::{TopologyMapper, TopologyStatistics};
pub use turn::{TURNAllocation, TURNClient};
pub use upnp::UPnPClient;

// Legacy compatibility - Re-export the main engine as the original name
pub use engine::NetworkDiscoveryEngine as DiscoveryEngine;

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_universal_primals::PrimalCapability;

    #[tokio::test]
    async fn test_discovery_engine_creation() {
        let config = DiscoveryConfig::default();
        let engine = NetworkDiscoveryEngine::new(config);

        // Test that the engine was created successfully
        assert!(engine.get_peer_registry().peer_count().await == 0);
    }

    #[tokio::test]
    async fn test_peer_registry_basic_operations() {
        let registry = PeerRegistry::new();

        // Test initial state
        assert_eq!(registry.peer_count().await, 0);
        assert!(!registry.has_peer("test-peer").await);

        // Test adding a peer
        let peer = DiscoveredPeer::new(
            "test-peer".to_string(),
            format!(
                "{}:8080",
                songbird_config::config::hardcoded_elimination::replace::bind_address()
            )
            .parse()
            .expect("Valid bind address"),
            PeerType::Orchestrator,
            DiscoveryMethod::UPnP,
        );

        let capabilities = vec![
            PrimalCapability::NetworkRouting {
                protocols: vec!["HTTP".to_string()],
            },
            PrimalCapability::Custom {
                name: "Gaming".to_string(),
                properties: vec![("optimized".to_string(), "true".to_string())]
                    .iter()
                    .cloned()
                    .collect(),
            },
            PrimalCapability::Custom {
                name: "NetworkConnectivity".to_string(),
                properties: vec![
                    ("bandwidth_mbps".to_string(), "100.0".to_string()),
                    ("latency_ms".to_string(), "5.0".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        ];

        registry.register_peer(peer, capabilities).await.unwrap();

        // Test peer was added
        assert_eq!(registry.peer_count().await, 1);
        assert!(registry.has_peer("test-peer").await);
    }

    #[tokio::test]
    async fn test_topology_basic_operations() {
        let mapper = TopologyMapper::new(std::time::Duration::from_secs(10));

        // Test adding nodes
        let capabilities = vec![
            PrimalCapability::NetworkRouting {
                protocols: vec!["HTTP".to_string()],
            },
            PrimalCapability::Custom {
                name: "Gaming".to_string(),
                properties: vec![("optimized".to_string(), "true".to_string())]
                    .iter()
                    .cloned()
                    .collect(),
            },
            PrimalCapability::Custom {
                name: "NetworkConnectivity".to_string(),
                properties: vec![
                    ("bandwidth_mbps".to_string(), "100.0".to_string()),
                    ("latency_ms".to_string(), "5.0".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
            },
        ];

        mapper
            .add_node(
                "node1".to_string(),
                format!(
                    "{}:8080",
                    songbird_config::config::hardcoded_elimination::replace::bind_address()
                )
                .parse()
                .expect("Valid bind address"),
                PeerType::Orchestrator,
                capabilities.clone(),
            )
            .await
            .unwrap();

        mapper
            .add_node(
                "node2".to_string(),
                "127.0.0.1:8081".parse().unwrap(),
                PeerType::Service,
                capabilities,
            )
            .await
            .unwrap();

        // Test adding connection
        mapper
            .add_connection("node1".to_string(), "node2".to_string(), 5)
            .await
            .unwrap();

        // Test topology
        let topology = mapper.get_topology().await;
        assert_eq!(topology.nodes.len(), 2);
        assert_eq!(topology.connections.len(), 1);
    }
}
