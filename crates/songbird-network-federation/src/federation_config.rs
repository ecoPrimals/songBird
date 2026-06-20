// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Federation configuration types and node info structures.

use serde::{Deserialize, Serialize};

use crate::discovery_mode::DiscoveryMode;
use crate::state::NodeRegistration;

/// Federation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationConfig {
    /// Whether federation is enabled
    pub enabled: bool,

    /// Bootstrap node address (IP:PORT or hostname:PORT)
    pub bootstrap_address: Option<String>,

    /// Self registration info (for joining federation)
    pub self_registration: Option<NodeRegistration>,

    /// Heartbeat interval in seconds
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval_secs: u64,

    /// Node timeout in seconds (mark as inactive after this)
    #[serde(default = "default_node_timeout")]
    pub node_timeout_secs: i64,

    /// Rendezvous server URL for internet-wide discovery (optional)
    pub rendezvous_url: Option<String>,

    /// Force discovery mode (if None, auto-detect based on security-provider availability)
    #[serde(default)]
    pub discovery_mode: Option<DiscoveryMode>,
}

const fn default_heartbeat_interval() -> u64 {
    30
}

const fn default_node_timeout() -> i64 {
    60
}

impl Default for FederationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bootstrap_address: None,
            self_registration: None,
            heartbeat_interval_secs: 30,
            node_timeout_secs: 60,
            rendezvous_url: None,
            discovery_mode: None,
        }
    }
}

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeInfo {
    /// Unique node identifier.
    pub node_id: String,
    /// Network address.
    pub address: String,
    /// Current status.
    pub status: String,
}
