// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! NAT traversal and STUN/TURN server configuration.

use serde::{Deserialize, Serialize};

/// STUN server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StunServerConfig {
    /// STUN server address
    pub address: String,
    /// Enable this server
    /// Enabled field
    pub enabled: bool,
}

/// TURN server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServerConfig {
    /// TURN server address
    pub address: String,
    /// Username for authentication
    /// Username field
    pub username: Option<String>,
    /// Password for authentication
    pub password: Option<String>,
    /// Enable this server
    /// Enabled field
    pub enabled: bool,
}

/// NAT traversal configuration - consolidates NAT configs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatTraversalConfig {
    /// Enable NAT traversal
    /// Enabled field
    pub enabled: bool,
    /// STUN server configuration
    /// Stun Servers field
    pub stun_servers: Vec<StunServerConfig>,
    /// TURN server configuration
    /// Turn Servers field
    pub turn_servers: Vec<TurnServerConfig>,
    /// `UPnP` settings
    /// Upnp Enabled field
    pub upnp_enabled: bool,
}

impl Default for NatTraversalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            stun_servers: vec![StunServerConfig {
                address: String::from("stun.nextcloud.com:3478"),
                enabled: true,
            }],
            turn_servers: Vec::new(),
            upnp_enabled: true,
        }
    }
}
