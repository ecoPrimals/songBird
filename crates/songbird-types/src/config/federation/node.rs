// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Federation node identity: local node settings, node kinds, tower hardware, mobility, and relay tiers.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

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
