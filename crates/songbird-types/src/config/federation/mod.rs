// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Canonical Federation configuration capability.
//!
//! This module provides the unified federation configuration structures,
//! migrated from `songbird-federation/src/types.rs` to the canonical location.
//!
//! Submodules are split by responsibility: `node`, `discovery`, `security_performance`,
//! `limits`, `peers`, `resources`.

#![allow(
    clippy::upper_case_acronyms,
    reason = "Federation API acronym casing preserved for wire compatibility"
)]

mod discovery;
mod limits;
mod node;
mod peers;
mod resources;
mod security_performance;

pub use discovery::*;
pub use limits::*;
pub use node::*;
pub use peers::*;
pub use resources::*;
pub use security_performance::*;

use serde::{Deserialize, Serialize};

/// **CANONICAL**: Federation Configuration - Single Source of Truth
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CanonicalFederationConfig {
    /// Local node configuration
    pub local_node: CanonicalLocalNodeConfig,
    /// Peer management configuration
    pub peers: PeerManagementConfig,
    /// Consensus configuration
    /// Consensus field
    pub consensus: ConsensusConfig,
    /// Resource management configuration
    pub resources: ResourceManagementConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::time::Duration;

    fn assert_json_roundtrip<T>(v: &T)
    where
        T: Serialize + DeserializeOwned + std::fmt::Debug,
    {
        let json = serde_json::to_value(v).unwrap();
        let back: T = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(serde_json::to_value(&back).unwrap(), json);
    }

    #[test]
    fn default_canonical_federation_config() {
        let c = CanonicalFederationConfig::default();
        assert_eq!(c.peers.max_peers, 100);
        assert_eq!(c.consensus.algorithm, ConsensusAlgorithm::Raft);
        assert_eq!(c.consensus.election_timeout, 1000);
        assert_eq!(c.resources.limits.memory_bytes, 1024 * 1024 * 1024);
    }

    #[test]
    fn default_canonical_local_node_config() {
        let c = CanonicalLocalNodeConfig::default();
        assert_eq!(c.node_type, CanonicalNodeType::default());
        assert_eq!(
            c.listen_addresses,
            vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080)]
        );
        assert!(!c.name.is_empty());
    }

    #[test]
    fn default_canonical_tower_capabilities() {
        let c = CanonicalTowerCapabilities::default();
        assert!(c.cpu_cores >= 1);
        assert_eq!(c.memory_gb, 8);
        assert_eq!(c.network_bandwidth_mbps, 1000);
        assert!(c.gpus.is_empty());
    }

    #[test]
    fn default_peer_and_consensus_and_limits() {
        assert_eq!(PeerManagementConfig::default().max_peers, 100);
        assert_eq!(PeerDiscoveryConfig::default().interval, 30);
        assert_eq!(PeerConnectionConfig::default().retry_attempts, 3);
        assert_eq!(ConsensusConfig::default().heartbeat_interval, 100);
        assert_eq!(CanonicalFederationLimits::default().max_message_size, 1024 * 1024);
        assert_eq!(CanonicalRateLimits::default().burst_size, 200);
        assert_eq!(CanonicalResourceLimits::default().cpu_percentage, 0.8);
        assert!(CanonicalFederationSecurityConfig::default().enable_tls);
        assert_eq!(CanonicalFederationPerformanceConfig::default().max_connections, 1000);
        assert_eq!(CanonicalBufferSizes::default().message_queue, 1000);
        assert_eq!(
            CanonicalFederationDiscoveryConfig::default().max_range,
            CanonicalNetworkProximity::Local
        );
        assert_eq!(CanonicalDiscoveryIntervals::default().heartbeat, Duration::from_secs(10));
        assert_eq!(ResourceMonitoringConfig::default().interval, 30);
        assert_eq!(ResourceThresholds::default().disk, 0.9);
        assert_eq!(ResourceAllocationConfig::default().strategy, AllocationStrategy::Balanced);
        assert_eq!(ReservedResources::default().cpu, 0.1);
        assert_eq!(ResourceManagementConfig::default().limits.disk_usage, 10 * 1024 * 1024 * 1024);
    }

    #[test]
    fn roundtrip_canonical_federation_config() {
        assert_json_roundtrip(&CanonicalFederationConfig::default());
    }

    #[test]
    fn roundtrip_canonical_local_node_config() {
        assert_json_roundtrip(&CanonicalLocalNodeConfig::default());
    }

    #[test]
    fn roundtrip_canonical_node_type_variants() {
        assert_json_roundtrip(&CanonicalNodeType::default());
        assert_json_roundtrip(&CanonicalNodeType::Tower {
            location: "dc1".into(),
            capabilities: CanonicalTowerCapabilities::default(),
        });
        assert_json_roundtrip(&CanonicalNodeType::Gateway {
            region: "us-west".into(),
            bandwidth_mbps: 0,
        });
        assert_json_roundtrip(&CanonicalNodeType::Relay {
            tier: CanonicalRelayTier::Global,
            global_endpoints: vec!["ep1".into()],
        });
    }

    #[test]
    fn roundtrip_canonical_tower_capabilities_edge_numeric() {
        let mut caps = CanonicalTowerCapabilities::default();
        caps.cpu_cores = 0;
        caps.memory_gb = u32::MAX;
        caps.storage_tb = 0;
        caps.network_bandwidth_mbps = u32::MAX;
        assert_json_roundtrip(&caps);
    }

    #[test]
    fn roundtrip_canonical_gpu_info() {
        assert_json_roundtrip(&CanonicalGpuInfo {
            model: "Test GPU".into(),
            memory_gb: 16,
            compute_capability: "8.0".into(),
        });
    }

    #[test]
    fn roundtrip_mobility_and_relay_tier() {
        assert_json_roundtrip(&CanonicalMobilityLevel::Mobile);
        assert_json_roundtrip(&CanonicalRelayTier::Continental);
    }

    #[test]
    fn roundtrip_federation_discovery_and_intervals() {
        assert_json_roundtrip(&CanonicalFederationDiscoveryConfig::default());
        assert_json_roundtrip(&CanonicalDiscoveryIntervals::default());
    }

    #[test]
    fn roundtrip_discovery_protocol_and_proximity() {
        assert_json_roundtrip(&CanonicalDiscoveryProtocol::Manual);
        assert_json_roundtrip(&CanonicalNetworkProximity::Global);
    }

    #[test]
    fn roundtrip_security_performance_buffers_limits() {
        assert_json_roundtrip(&CanonicalFederationSecurityConfig::default());
        assert_json_roundtrip(&CanonicalFederationPerformanceConfig::default());
        assert_json_roundtrip(&CanonicalBufferSizes::default());
        assert_json_roundtrip(&CanonicalFederationLimits::default());
        assert_json_roundtrip(&CanonicalRateLimits::default());
        assert_json_roundtrip(&CanonicalResourceLimits::default());
    }

    #[test]
    fn roundtrip_peer_consensus_enums() {
        assert_json_roundtrip(&PeerManagementConfig::default());
        assert_json_roundtrip(&PeerDiscoveryConfig::default());
        assert_json_roundtrip(&PeerConnectionConfig::default());
        assert_json_roundtrip(&ConsensusConfig::default());
        assert_json_roundtrip(&PeerDiscoveryMethod::Custom("x".into()));
        assert_json_roundtrip(&ConsensusAlgorithm::Custom("c".into()));
        assert_json_roundtrip(&ConsensusAlgorithm::Pbft);
    }

    #[test]
    fn roundtrip_resource_management_tree() {
        assert_json_roundtrip(&ResourceManagementConfig::default());
        assert_json_roundtrip(&ResourceMonitoringConfig::default());
        assert_json_roundtrip(&ResourceThresholds::default());
        assert_json_roundtrip(&ResourceAllocationConfig::default());
        assert_json_roundtrip(&AllocationStrategy::CpuOptimized);
        assert_json_roundtrip(&ReservedResources::default());
    }
}
