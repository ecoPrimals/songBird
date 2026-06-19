// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Protocol port maps, external addressing, subnet defaults, and node identity.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use songbird_types::error_helpers::SafeEnv;

use super::{
    FALLBACK_PROTOCOL_SECURE_WEBSOCKET_PORT, FALLBACK_PROTOCOL_TCP_PORT,
    FALLBACK_PROTOCOL_UDP_PORT, FALLBACK_PROTOCOL_WEBSOCKET_PORT, get_bind_address,
};

/// Get protocol port mappings for gaming network
#[must_use]
pub fn protocol_port_mappings() -> HashMap<String, u16> {
    let mut mappings = HashMap::new();
    mappings.insert(
        String::from("udp"),
        SafeEnv::get_port("SONGBIRD_PROTOCOL_UDP_PORT", FALLBACK_PROTOCOL_UDP_PORT),
    );
    mappings.insert(
        String::from("tcp"),
        SafeEnv::get_port("SONGBIRD_PROTOCOL_TCP_PORT", FALLBACK_PROTOCOL_TCP_PORT),
    );
    mappings.insert(
        String::from("websocket"),
        SafeEnv::get_port("SONGBIRD_PROTOCOL_WEBSOCKET_PORT", FALLBACK_PROTOCOL_WEBSOCKET_PORT),
    );
    mappings.insert(
        String::from("secure_websocket"),
        SafeEnv::get_port(
            "SONGBIRD_PROTOCOL_SECURE_WEBSOCKET_PORT",
            FALLBACK_PROTOCOL_SECURE_WEBSOCKET_PORT,
        ),
    );
    mappings
}

/// Get external address for network configuration
#[must_use]
pub fn external_address() -> String {
    SafeEnv::get_or_default("SONGBIRD_EXTERNAL_ADDRESS", get_bind_address())
}

/// Get default subnet configuration
#[must_use]
pub fn default_subnet() -> String {
    SafeEnv::get_or_default("SONGBIRD_SUBNET", String::from("10.0.0.0/24"))
}

/// Generate a unique node ID for this instance
#[must_use]
pub fn node_id() -> String {
    // Create a unique node ID based on hostname and process ID
    let hostname = gethostname::gethostname();
    let pid = std::process::id();

    let mut hasher = DefaultHasher::new();
    hostname.hash(&mut hasher);
    pid.hash(&mut hasher);

    format!("songbird-{:x}", hasher.finish())
}
