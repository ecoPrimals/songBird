// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Peer management and consensus settings for federated clusters.

use serde::{Deserialize, Serialize};

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
