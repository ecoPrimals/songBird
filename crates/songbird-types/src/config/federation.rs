// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Canonical Federation /// Configuration capability Configuration
//!
//! This module provides the unified federation configuration structures,
//! migrated from `songbird-federation/src/types.rs` to the canonical location.

#![allow(
    clippy::upper_case_acronyms,
    reason = "Federation API acronym casing preserved for wire compatibility"
)]

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;

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

/// **CANONICAL**: Local node configuration
///
/// Migrated from: `songbird-federation/src/types.rs::LocalNodeConfig`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalLocalNodeConfig {
    /// Node name
    /// Name identifier
    pub name: String,
    /// Node type
    pub node_type: CanonicalNodeType,
    /// Listening addresses
    /// Listen Addresses field
    pub listen_addresses: Vec<SocketAddr>,
    /// Public addresses (for internet connectivity)
    /// Public Addresses field
    pub public_addresses: Vec<SocketAddr>,
    /// Location information
    /// Location field
    pub location: Option<String>,
}

impl Default for CanonicalLocalNodeConfig {
    fn default() -> Self {
        Self {
            name: songbird_process_env::var("HOSTNAME")
                .or_else(|_| songbird_process_env::var("COMPUTERNAME"))
                .unwrap_or_else(|_| "songbird-node".to_string()),
            node_type: CanonicalNodeType::default(),
            listen_addresses: vec![std::net::SocketAddr::new(
                std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
                8080,
            )],
            public_addresses: vec![],
            location: None,
        }
    }
}

/// **CANONICAL**: Node type enumeration
///
/// Migrated from: `songbird-federation/src/types.rs::NodeType`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CanonicalNodeType {
    /// Tower node (basement server, compute node)
    Tower {
        /// Physical location of the tower
        location: String,
        /// Hardware capabilities of the tower
        capabilities: CanonicalTowerCapabilities,
    },
    /// Edge node (laptop, mobile device)
    Edge {
        /// Mobility level of the edge device
        mobility: CanonicalMobilityLevel,
    },
    /// Gateway node (internet bridge, regional hub)
    Gateway {
        /// Geographic region served
        region: String,
        /// Available bandwidth in
        bandwidth_mbps: u32,
    },
    /// Relay node (worldwide mesh connector)
    Relay {
        /// Tier level in the relay hierarchy
        tier: CanonicalRelayTier,
        /// Global endpoint addresses
        global_endpoints: Vec<String>,
    },
}

impl Default for CanonicalNodeType {
    fn default() -> Self {
        Self::Edge {
            mobility: CanonicalMobilityLevel::Stationary,
        }
    }
}

/// **CANONICAL**: Tower capabilities for HPC federation
///
/// Migrated from: `songbird-federation/src/types.rs::TowerCapabilities`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalTowerCapabilities {
    /// CPU cores available
    pub cpu_cores: u32,
    /// Memory in
    pub memory_gb: u32,
    /// Storage in
    pub storage_tb: u32,
    /// GPU count and types
    pub gpus: Vec<CanonicalGpuInfo>,
    /// Network bandwidth in /// Mbps
    /// Network Bandwidth Mbps field
    pub network_bandwidth_mbps: u32,
    /// Specialized capabilities
    /// Specializations field
    pub specializations: Vec<String>,
}

impl Default for CanonicalTowerCapabilities {
    fn default() -> Self {
        Self {
            cpu_cores: u32::try_from(
                std::thread::available_parallelism().map_or(1, std::num::NonZero::get),
            )
            .unwrap_or(4),
            memory_gb: 8, // Conservative default
            storage_tb: 1,
            gpus: vec![],
            network_bandwidth_mbps: 1000, // 1 Gbps default
            specializations: vec![],
        }
    }
}

/// **CANONICAL**: GPU information
///
/// Migrated from: `songbird-federation/src/types.rs::GpuInfo`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CanonicalGpuInfo {
    /// GPU model name
    pub model: String,
    /// GPU memory in
    pub memory_gb: u32,
    /// Compute capability version
    pub compute_capability: String,
}

/// **CANONICAL**: Node mobility level for routing optimization
///
/// Migrated from: `songbird-federation/src/types.rs::MobilityLevel`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CanonicalMobilityLevel {
    /// Stationary (desktop, server)
    Stationary,
    /// Portable (laptop with power)
    Portable,
    /// Mobile (battery powered, changing networks)
    Mobile,
}

/// **CANONICAL**: Relay tier for global mesh
///
/// Migrated from: `songbird-federation/src/types.rs::RelayTier`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CanonicalRelayTier {
    /// Regional relay (country/state level)
    Regional,
    /// Continental relay (continent level)
    Continental,
    /// Global relay (worldwide)
    Global,
}

/// **CANONICAL**: Federation discovery configuration
///
/// Migrated from: `songbird-federation/src/types.rs::DiscoveryConfig`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalFederationDiscoveryConfig {
    /// Enabled discovery protocols
    /// Enabled Protocols field
    pub enabled_protocols: Vec<CanonicalDiscoveryProtocol>,
    /// Discovery intervals
    /// Intervals field
    pub intervals: CanonicalDiscoveryIntervals,
    /// Maximum discovery range
    /// Max Range field
    pub max_range: CanonicalNetworkProximity,
    /// Bootstrap nodes for initial discovery
    pub bootstrap_nodes: Vec<String>,
}

impl Default for CanonicalFederationDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled_protocols: vec![
                CanonicalDiscoveryProtocol::Broadcast,
                CanonicalDiscoveryProtocol::Manual,
            ],
            intervals: CanonicalDiscoveryIntervals::default(),
            max_range: CanonicalNetworkProximity::Local,
            bootstrap_nodes: vec![],
        }
    }
}

/// **CANONICAL**: Discovery protocol enumeration
///
/// Migrated from: `songbird-federation/src/types.rs::DiscoveryProtocol`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CanonicalDiscoveryProtocol {
    /// Multicast
    Broadcast,
    /// Manual configuration
    Manual,
}

/// **CANONICAL**: Discovery timing intervals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalDiscoveryIntervals {
    /// Fast discovery interval (nearby nodes)
    /// Fast Discovery field
    pub fast_discovery: Duration,
    /// Slow discovery interval (distant nodes)
    /// Slow Discovery field
    pub slow_discovery: Duration,
    /// Heartbeat interval
    pub heartbeat: Duration,
}

impl Default for CanonicalDiscoveryIntervals {
    fn default() -> Self {
        Self {
            fast_discovery: Duration::from_secs(5),
            slow_discovery: Duration::from_secs(30),
            heartbeat: Duration::from_secs(10),
        }
    }
}

/// **CANONICAL**: Network proximity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CanonicalNetworkProximity {
    /// Same machine
    Local,
    /// Same
    LAN,
    /// Same region/datacenter
    Regional,
    /// Internet-wide
    Global,
}

/// **CANONICAL**: Federation security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalFederationSecurityConfig {
    /// Enable TLS encryption
    pub enable_tls: bool,
    /// Certificate path
    pub cert_path: Option<String>,
    /// Private key path
    pub key_path: Option<String>,
    /// Trusted certificate authorities
    pub trusted_cas: Vec<String>,
    /// Enable mutual authentication
    pub mutual_auth: bool,
}

impl Default for CanonicalFederationSecurityConfig {
    fn default() -> Self {
        Self {
            enable_tls: true,
            cert_path: None,
            key_path: None,
            trusted_cas: vec![],
            mutual_auth: false,
        }
    }
}

/// **CANONICAL**: Federation performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalFederationPerformanceConfig {
    /// Maximum concurrent connections
    /// Max Connections field
    pub max_connections: usize,
    /// Connection timeout
    /// Connection Timeout field
    pub connection_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
    /// Keep-alive interval
    /// Keep Alive Interval field
    pub keep_alive_interval: Duration,
    /// Buffer sizes
    pub buffer_sizes: CanonicalBufferSizes,
}

impl Default for CanonicalFederationPerformanceConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
            keep_alive_interval: Duration::from_secs(30),
            buffer_sizes: CanonicalBufferSizes::default(),
        }
    }
}

/// **CANONICAL**: Buffer size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalBufferSizes {
    /// Send buffer size in bytes
    pub send_buffer: usize,
    /// Receive buffer size in bytes
    pub recv_buffer: usize,
    /// Message queue size
    pub message_queue: usize,
}

impl Default for CanonicalBufferSizes {
    fn default() -> Self {
        Self {
            send_buffer: 64 * 1024, // 64KB
            recv_buffer: 64 * 1024, // 64KB
            message_queue: 1000,    // 1000 messages
        }
    }
}

/// **CANONICAL**: Federation limits and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalFederationLimits {
    /// Maximum nodes in federation
    pub max_nodes: usize,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Rate limiting configuration
    pub rate_limits: CanonicalRateLimits,
    /// Resource limits
    /// Resource limitation configurations
    pub resource_limits: CanonicalResourceLimits,
}

impl Default for CanonicalFederationLimits {
    fn default() -> Self {
        Self {
            max_nodes: 10000,
            max_message_size: 1024 * 1024, // 1MB
            rate_limits: CanonicalRateLimits::default(),
            resource_limits: CanonicalResourceLimits::default(),
        }
    }
}

/// **CANONICAL**: Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalRateLimits {
    /// Requests per second per node
    /// Requests Per Second field
    pub requests_per_second: u32,
    /// Burst allowance
    pub burst_size: u32,
    /// Bandwidth limit in bytes per second
    /// Bandwidth Limit field
    pub bandwidth_limit: u64,
}

impl Default for CanonicalRateLimits {
    fn default() -> Self {
        Self {
            requests_per_second: 100,
            burst_size: 200,
            bandwidth_limit: 10 * 1024 * 1024, // 10 MB/s
        }
    }
}

/// **CANONICAL**: Resource limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalResourceLimits {
    /// Maximum memory usage in bytes
    pub memory_bytes: u64,
    /// Maximum CPU usage percentage (0.0-1.0)
    pub cpu_percentage: f64,
    /// Maximum disk usage in bytes
    pub disk_usage: u64,
}

impl Default for CanonicalResourceLimits {
    fn default() -> Self {
        Self {
            memory_bytes: 1024 * 1024 * 1024,    // 1GB
            cpu_percentage: 0.8,                 // 80%
            disk_usage: 10 * 1024 * 1024 * 1024, // 10GB
        }
    }
}

/// Peer management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerManagementConfig {
    /// Maximum number of peers
    pub max_peers: usize,
    /// Peer discovery settings
    pub discovery: PeerDiscoveryConfig,
    /// Peer connection settings
    /// Connection field
    pub connection: PeerConnectionConfig,
}

impl Default for PeerManagementConfig {
    fn default() -> Self {
        Self {
            max_peers: 100,
            discovery: PeerDiscoveryConfig::default(),
            connection: PeerConnectionConfig::default(),
        }
    }
}

/// Peer discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerDiscoveryConfig {
    /// Discovery methods
    pub methods: Vec<PeerDiscoveryMethod>,
    /// Discovery interval in seconds
    pub interval: u64,
    /// Discovery timeout in seconds
    pub timeout: u64,
}

impl Default for PeerDiscoveryConfig {
    fn default() -> Self {
        Self {
            methods: vec![PeerDiscoveryMethod::Mdns, PeerDiscoveryMethod::Static],
            interval: 30,
            timeout: 10,
        }
    }
}

/// Peer connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnectionConfig {
    /// Connection timeout in seconds
    pub timeout: u64,
    /// Keep-alive interval in seconds
    pub keep_alive: u64,
    /// Maximum retry attempts
    pub retry_attempts: u32,
}

impl Default for PeerConnectionConfig {
    fn default() -> Self {
        Self {
            timeout: 30,
            keep_alive: 60,
            retry_attempts: 3,
        }
    }
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Consensus algorithm
    pub algorithm: ConsensusAlgorithm,
    /// Election timeout in milliseconds
    /// Election Timeout field
    pub election_timeout: u64,
    /// Heartbeat interval in milliseconds
    /// Heartbeat Interval field
    pub heartbeat_interval: u64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            algorithm: ConsensusAlgorithm::Raft,
            election_timeout: 1000,
            heartbeat_interval: 100,
        }
    }
}

/// Peer discovery method configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PeerDiscoveryMethod {
    /// Static peer list
    Static,
    /// Multicast DNS discovery
    Mdns,
    /// Distributed hash table
    Dht,
    /// Custom discovery method
    Custom(String),
}

/// Consensus algorithm enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConsensusAlgorithm {
    /// Raft consensus algorithm
    Raft,
    /// PBFT consensus algorithm
    Pbft,
    /// Custom consensus algorithm
    Custom(String),
}

/// Resource management configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceManagementConfig {
    /// Resource limits
    pub limits: CanonicalResourceLimits,
    /// Resource monitoring
    /// Monitoring field
    pub monitoring: ResourceMonitoringConfig,
    /// Resource allocation
    /// Allocation field
    pub allocation: ResourceAllocationConfig,
}

/// Resource monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMonitoringConfig {
    /// Enable monitoring
    pub enabled: bool,
    /// Monitoring interval in seconds
    pub interval: u64,
    /// Alert thresholds
    pub thresholds: ResourceThresholds,
}

impl Default for ResourceMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: 30,
            thresholds: ResourceThresholds::default(),
        }
    }
}

/// Resource thresholds configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceThresholds {
    /// Memory usage threshold (0.0-1.0)
    /// Memory field
    pub memory: f64,
    /// CPU usage threshold (0.0-1.0)
    /// Cpu field
    pub cpu: f64,
    /// Disk usage threshold (0.0-1.0)
    /// Disk field
    pub disk: f64,
}

impl Default for ResourceThresholds {
    fn default() -> Self {
        Self {
            memory: 0.8,
            cpu: 0.8,
            disk: 0.9,
        }
    }
}

/// Resource allocation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocationConfig {
    /// Allocation strategy
    pub strategy: AllocationStrategy,
    /// Reserved resources
    pub reserved: ReservedResources,
}

impl Default for ResourceAllocationConfig {
    fn default() -> Self {
        Self {
            strategy: AllocationStrategy::Balanced,
            reserved: ReservedResources::default(),
        }
    }
}

/// Allocation strategy enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AllocationStrategy {
    /// Balanced allocation
    Balanced,
    /// CPU-optimized allocation
    CpuOptimized,
    /// Memory-optimized allocation
    MemoryOptimized,
    /// Custom allocation strategy
    Custom(String),
}

/// Reserved resources configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservedResources {
    /// Reserved memory in bytes
    pub memory: u64,
    /// Reserved CPU percentage (0.0-1.0)
    pub cpu: f64,
    /// Reserved disk space in bytes
    pub disk: u64,
}

impl Default for ReservedResources {
    fn default() -> Self {
        Self {
            memory: 512 * 1024 * 1024, // 512MB
            cpu: 0.1,                  // 10%
            disk: 1024 * 1024 * 1024,  // 1GB
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

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
        assert_eq!(CanonicalFederationSecurityConfig::default().enable_tls, true);
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
