// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Dashboard/discovery ports, external addressing, and node identity helpers.

use songbird_types::error_helpers::SafeEnv;

/// Get dashboard port from environment or calculated default
#[must_use]
pub fn get_dashboard_port() -> u16 {
    SafeEnv::parse("SONGBIRD_DASHBOARD_PORT", {
        // Calculate based on environment
        match SafeEnv::get("SONGBIRD_ENV").as_deref() {
            Ok("production") => 3000, // Standard port for production
            Ok("staging") => 3001,    // Staging offset
            Ok("testing") => 3002,    // Testing offset
            _ => 8083,                // Development default
        }
    })
}

/// Get protocol port mappings for gaming network
#[must_use]
pub fn protocol_port_mappings() -> std::collections::HashMap<String, u16> {
    let mut mappings = std::collections::HashMap::new();
    mappings.insert("udp".to_string(), 6112);
    mappings.insert("tcp".to_string(), 6113);
    mappings.insert("websocket".to_string(), 8080);
    mappings.insert("secure_websocket".to_string(), 8443);
    mappings
}

/// Get external address for network configuration
#[must_use]
pub fn external_address() -> String {
    SafeEnv::get_or_default(
        "SONGBIRD_EXTERNAL_ADDRESS",
        crate::constants::network::DEFAULT_HOST.to_string(),
    )
}

/// Get default subnet configuration
#[must_use]
pub fn default_subnet() -> String {
    SafeEnv::get_or_default("SONGBIRD_SUBNET", "10.0.0.0/24".to_string())
}

/// Universal capability query - works with any capability name
#[must_use]
pub fn find_primals_with_capability(capability: &str) -> Vec<String> {
    crate::canonical::constants::find_primals_with_capability(capability)
}

/// Get default bind address for the current environment
#[must_use]
pub fn get_default_bind_address() -> String {
    default_bind_address()
}

/// Generate a unique node ID for this instance
#[must_use]
pub fn node_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Create a unique node ID based on hostname and process ID
    let hostname = gethostname::gethostname();
    let pid = std::process::id();

    let mut hasher = DefaultHasher::new();
    hostname.hash(&mut hasher);
    pid.hash(&mut hasher);

    format!("songbird-{:x}", hasher.finish())
}

/// Get default discovery port
#[must_use]
pub fn default_discovery_port() -> u16 {
    crate::defaults::ports::discovery_port()
}

/// Get default bind address for the current environment
#[must_use]
pub fn default_bind_address() -> String {
    super::bind_and_ports::get_bind_address()
}
