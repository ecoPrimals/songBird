//! Comprehensive Federation Configuration Tests
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]

//!
//! Tests for `songbird_types::config::federation` module.

use songbird_types::config::federation::*;
use songbird_types::{SongbirdError, SongbirdResult};

// ============================================================================
// FEDERATION CONFIG TESTS
// ============================================================================

#[test]
fn test_federation_config_default() {
    let config = CanonicalFederationConfig::default();
    assert_eq!(config.local_node.node_type, CanonicalNodeType::default());
    assert!(config.peers.max_peers > 0);
    // Verify consensus config exists with default values
    assert!(matches!(config.consensus.algorithm, ConsensusAlgorithm::Raft));
}

#[test]
fn test_federation_config_clone() {
    let config = CanonicalFederationConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.peers.max_peers, config.peers.max_peers);
}

// ============================================================================
// LOCAL NODE CONFIG TESTS
// ============================================================================

#[test]
fn test_local_node_config_default() {
    let config = CanonicalLocalNodeConfig::default();
    assert!(!config.name.is_empty());
    assert!(!config.listen_addresses.is_empty());
    assert_eq!(config.public_addresses.len(), 0);
}

#[test]
fn test_local_node_config_custom() {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    let config = CanonicalLocalNodeConfig {
        name: "test-node".to_string(),
        node_type: CanonicalNodeType::default(),
        listen_addresses: vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9000)],
        public_addresses: vec![],
        location: Some("US-West".to_string()),
    };

    assert_eq!(config.name, "test-node");
    assert_eq!(config.location, Some("US-West".to_string()));
}

// ============================================================================
// NODE TYPE TESTS
// ============================================================================

#[test]
fn test_node_type_default() {
    let node_type = CanonicalNodeType::default();
    assert!(matches!(node_type, CanonicalNodeType::Edge { .. }));
}

#[test]
fn test_node_type_tower() {
    let tower = CanonicalNodeType::Tower {
        location: "Basement".to_string(),
        capabilities: CanonicalTowerCapabilities::default(),
    };

    assert!(matches!(tower, CanonicalNodeType::Tower { .. }));
}

#[test]
fn test_node_type_edge() {
    let edge = CanonicalNodeType::Edge {
        mobility: CanonicalMobilityLevel::Portable,
    };

    assert!(matches!(edge, CanonicalNodeType::Edge { .. }));
}

#[test]
fn test_node_type_gateway() {
    let gateway = CanonicalNodeType::Gateway {
        region: "US-West".to_string(),
        bandwidth_mbps: 1000,
    };

    assert!(matches!(gateway, CanonicalNodeType::Gateway { .. }));
}

#[test]
fn test_node_type_relay() {
    let relay = CanonicalNodeType::Relay {
        tier: CanonicalRelayTier::Regional,
        global_endpoints: vec!["relay.example.com".to_string()],
    };

    assert!(matches!(relay, CanonicalNodeType::Relay { .. }));
}

// ============================================================================
// TOWER CAPABILITIES TESTS
// ============================================================================

#[test]
fn test_tower_capabilities_default() {
    let caps = CanonicalTowerCapabilities::default();
    assert!(caps.cpu_cores > 0);
    assert!(caps.memory_gb > 0);
    assert!(caps.network_bandwidth_mbps > 0);
}

#[test]
fn test_tower_capabilities_custom() {
    let caps = CanonicalTowerCapabilities {
        cpu_cores: 16,
        memory_gb: 64,
        storage_tb: 10,
        gpus: vec![CanonicalGpuInfo {
            model: "NVIDIA RTX 4090".to_string(),
            memory_gb: 24,
            compute_capability: "8.9".to_string(),
        }],
        network_bandwidth_mbps: 10000,
        specializations: vec!["ML".to_string(), "Rendering".to_string()],
    };

    assert_eq!(caps.cpu_cores, 16);
    assert_eq!(caps.gpus.len(), 1);
    assert_eq!(caps.specializations.len(), 2);
}

// ============================================================================
// MOBILITY LEVEL TESTS
// ============================================================================

#[test]
fn test_mobility_levels() {
    let stationary = CanonicalMobilityLevel::Stationary;
    let portable = CanonicalMobilityLevel::Portable;
    let mobile = CanonicalMobilityLevel::Mobile;

    assert_eq!(stationary, CanonicalMobilityLevel::Stationary);
    assert_eq!(portable, CanonicalMobilityLevel::Portable);
    assert_eq!(mobile, CanonicalMobilityLevel::Mobile);
}

// ============================================================================
// RELAY TIER TESTS
// ============================================================================

#[test]
fn test_relay_tiers() {
    let regional = CanonicalRelayTier::Regional;
    let continental = CanonicalRelayTier::Continental;
    let global = CanonicalRelayTier::Global;

    assert_eq!(regional, CanonicalRelayTier::Regional);
    assert_eq!(continental, CanonicalRelayTier::Continental);
    assert_eq!(global, CanonicalRelayTier::Global);
}

// ============================================================================
// DISCOVERY CONFIG TESTS
// ============================================================================

#[test]
fn test_federation_discovery_config_default() {
    let config = CanonicalFederationDiscoveryConfig::default();
    assert!(!config.enabled_protocols.is_empty());
    assert_eq!(config.max_range, CanonicalNetworkProximity::Local);
}

#[test]
fn test_discovery_protocols() {
    let broadcast = CanonicalDiscoveryProtocol::Broadcast;
    let manual = CanonicalDiscoveryProtocol::Manual;

    assert_eq!(broadcast, CanonicalDiscoveryProtocol::Broadcast);
    assert_eq!(manual, CanonicalDiscoveryProtocol::Manual);
}

#[test]
fn test_discovery_intervals_default() {
    let intervals = CanonicalDiscoveryIntervals::default();
    assert!(intervals.fast_discovery.as_secs() > 0);
    assert!(intervals.slow_discovery.as_secs() > 0);
    assert!(intervals.heartbeat.as_secs() > 0);
}

// ============================================================================
// NETWORK PROXIMITY TESTS
// ============================================================================

#[test]
fn test_network_proximity_levels() {
    let local = CanonicalNetworkProximity::Local;
    let lan = CanonicalNetworkProximity::LAN;
    let regional = CanonicalNetworkProximity::Regional;
    let global = CanonicalNetworkProximity::Global;

    assert_eq!(local, CanonicalNetworkProximity::Local);
    assert_eq!(lan, CanonicalNetworkProximity::LAN);
    assert_eq!(regional, CanonicalNetworkProximity::Regional);
    assert_eq!(global, CanonicalNetworkProximity::Global);
}

// ============================================================================
// SECURITY CONFIG TESTS
// ============================================================================

#[test]
fn test_federation_security_config_default() {
    let config = CanonicalFederationSecurityConfig::default();
    assert!(config.enable_tls);
    assert!(!config.mutual_auth);
}

#[test]
fn test_federation_security_config_custom() {
    let config = CanonicalFederationSecurityConfig {
        enable_tls: true,
        cert_path: Some("/path/to/cert.pem".to_string()),
        key_path: Some("/path/to/key.pem".to_string()),
        trusted_cas: vec!["ca1.pem".to_string()],
        mutual_auth: true,
    };

    assert!(config.enable_tls);
    assert!(config.mutual_auth);
    assert_eq!(config.trusted_cas.len(), 1);
}

// ============================================================================
// PERFORMANCE CONFIG TESTS
// ============================================================================

#[test]
fn test_federation_performance_config_default() {
    let config = CanonicalFederationPerformanceConfig::default();
    assert!(config.max_connections > 0);
    assert!(config.connection_timeout.as_secs() > 0);
}

#[test]
fn test_buffer_sizes_default() {
    let buffers = CanonicalBufferSizes::default();
    assert_eq!(buffers.send_buffer, 64 * 1024);
    assert_eq!(buffers.recv_buffer, 64 * 1024);
    assert_eq!(buffers.message_queue, 1000);
}

// ============================================================================
// LIMITS TESTS
// ============================================================================

#[test]
fn test_federation_limits_default() {
    let limits = CanonicalFederationLimits::default();
    assert_eq!(limits.max_nodes, 10000);
    assert_eq!(limits.max_message_size, 1024 * 1024);
}

#[test]
fn test_rate_limits_default() {
    let limits = CanonicalRateLimits::default();
    assert_eq!(limits.requests_per_second, 100);
    assert_eq!(limits.burst_size, 200);
    assert_eq!(limits.bandwidth_limit, 10 * 1024 * 1024);
}

#[test]
fn test_resource_limits_default() {
    let limits = CanonicalResourceLimits::default();
    assert_eq!(limits.memory_bytes, 1024 * 1024 * 1024);
    assert!((limits.cpu_percentage - 0.8).abs() < f64::EPSILON);
    assert_eq!(limits.disk_usage, 10 * 1024 * 1024 * 1024);
}

// ============================================================================
// PEER MANAGEMENT TESTS
// ============================================================================

#[test]
fn test_peer_management_config_default() {
    let config = PeerManagementConfig::default();
    assert_eq!(config.max_peers, 100);
}

#[test]
fn test_peer_discovery_config_default() {
    let config = PeerDiscoveryConfig::default();
    assert!(!config.methods.is_empty());
    assert!(config.interval > 0);
}

#[test]
fn test_peer_discovery_methods() {
    let static_method = PeerDiscoveryMethod::Static;
    let mdns = PeerDiscoveryMethod::Mdns;
    let dht = PeerDiscoveryMethod::Dht;
    let custom = PeerDiscoveryMethod::Custom("custom".to_string());

    assert_eq!(static_method, PeerDiscoveryMethod::Static);
    assert_eq!(mdns, PeerDiscoveryMethod::Mdns);
    assert_eq!(dht, PeerDiscoveryMethod::Dht);
    assert!(matches!(custom, PeerDiscoveryMethod::Custom(_)));
}

#[test]
fn test_peer_connection_config_default() {
    let config = PeerConnectionConfig::default();
    assert_eq!(config.timeout, 30);
    assert_eq!(config.keep_alive, 60);
    assert_eq!(config.retry_attempts, 3);
}

// ============================================================================
// CONSENSUS TESTS
// ============================================================================

#[test]
fn test_consensus_config_default() {
    let config = ConsensusConfig::default();
    assert!(matches!(config.algorithm, ConsensusAlgorithm::Raft));
    assert_eq!(config.election_timeout, 1000);
    assert_eq!(config.heartbeat_interval, 100);
}

#[test]
fn test_consensus_algorithms() {
    let raft = ConsensusAlgorithm::Raft;
    let pbft = ConsensusAlgorithm::Pbft;
    let custom = ConsensusAlgorithm::Custom("custom".to_string());

    assert!(matches!(raft, ConsensusAlgorithm::Raft));
    assert!(matches!(pbft, ConsensusAlgorithm::Pbft));
    assert!(matches!(custom, ConsensusAlgorithm::Custom(_)));
}

// ============================================================================
// RESOURCE MANAGEMENT TESTS
// ============================================================================

#[test]
fn test_resource_management_config_default() {
    let config = ResourceManagementConfig::default();
    assert!(config.monitoring.enabled);
    assert!(matches!(config.allocation.strategy, AllocationStrategy::Balanced));
}

#[test]
fn test_resource_monitoring_config_default() {
    let config = ResourceMonitoringConfig::default();
    assert!(config.enabled);
    assert_eq!(config.interval, 30);
}

#[test]
fn test_resource_thresholds_default() {
    let thresholds = ResourceThresholds::default();
    assert!((thresholds.memory - 0.8).abs() < f64::EPSILON);
    assert!((thresholds.cpu - 0.8).abs() < f64::EPSILON);
    assert!((thresholds.disk - 0.9).abs() < f64::EPSILON);
}

#[test]
fn test_resource_allocation_config_default() {
    let config = ResourceAllocationConfig::default();
    assert!(matches!(config.strategy, AllocationStrategy::Balanced));
}

#[test]
fn test_allocation_strategies() -> SongbirdResult<()> {
    let balanced = AllocationStrategy::Balanced;
    let cpu_opt = AllocationStrategy::CpuOptimized;
    let mem_opt = AllocationStrategy::MemoryOptimized;
    let custom = AllocationStrategy::Custom("custom".to_string());

    assert!(matches!(balanced, AllocationStrategy::Balanced));
    assert!(matches!(cpu_opt, AllocationStrategy::CpuOptimized));
    assert!(matches!(mem_opt, AllocationStrategy::MemoryOptimized));
    assert!(matches!(custom, AllocationStrategy::Custom(_)));
    Ok(())
}

#[test]
fn test_reserved_resources_default() -> SongbirdResult<()> {
    let reserved = ReservedResources::default();
    assert_eq!(reserved.memory, 512 * 1024 * 1024);
    assert!((reserved.cpu - 0.1).abs() < f64::EPSILON);
    assert_eq!(reserved.disk, 1024 * 1024 * 1024);
    Ok(())
}

// ============================================================================
// SERIALIZATION TESTS
// ============================================================================

#[test]
fn test_federation_config_serialization() -> SongbirdResult<()> {
    let config = CanonicalFederationConfig::default();
    let json = serde_json::to_string(&config)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: CanonicalFederationConfig = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Failed to deserialize: {}", e)))?;

    assert_eq!(deserialized.peers.max_peers, config.peers.max_peers);
    Ok(())
}

#[test]
fn test_node_type_serialization() -> SongbirdResult<()> {
    let node_type = CanonicalNodeType::default();
    let json = serde_json::to_string(&node_type)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: CanonicalNodeType = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Failed to deserialize: {}", e)))?;

    assert_eq!(deserialized, node_type);
    Ok(())
}
