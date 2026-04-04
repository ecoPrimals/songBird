// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Federation discovery: protocols, timing, proximity bounds, and bootstrap peers.

use serde::{Deserialize, Serialize};
use std::time::Duration;

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
