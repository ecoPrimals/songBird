// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Bind address, dynamic port range, and common primal port discovery.

use songbird_types::error_helpers::SafeEnv;

/// Default configuration file path
pub const DEFAULT_CONFIG_PATH: &str = "songbird.toml";

/// IPv4 localhost address constant
pub const LOCALHOST_IPV4: &str = "127.0.0.1";

/// Default bind address constant (for backwards compatibility with tests)
pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:8080";

/// Default localhost constant (for backwards compatibility with tests)
pub const DEFAULT_LOCALHOST: &str = "127.0.0.1";

/// Get bind address from environment or calculate from system capabilities
#[must_use]
pub fn get_bind_address() -> String {
    // Try to get from environment, but validate it
    if let Ok(addr) = SafeEnv::get("SONGBIRD_BIND_ADDRESS") {
        // Validate that it's a valid IP address
        if addr.parse::<std::net::IpAddr>().is_ok() {
            return addr;
        }
        // Invalid IP in env var, fall through to calculated default
    }

    // Detect if running in container/kubernetes or production
    if SafeEnv::get("KUBERNETES_SERVICE_HOST").is_ok()
        || SafeEnv::get("CONTAINER").is_ok()
        || SafeEnv::get("SONGBIRD_ENV").as_deref() == Ok("production")
    {
        "0.0.0.0".to_string() // Container/production environment
    } else {
        "127.0.0.1".to_string() // Development/local environment
    }
}

/// Get port range start from environment or system-based calculation
#[must_use]
pub fn get_port_range_start() -> u16 {
    SafeEnv::parse("SONGBIRD_PORT_START", {
        // Calculate based on environment and user permissions
        if SafeEnv::get("SONGBIRD_ALLOW_PRIVILEGED_PORTS").is_ok() {
            80 + get_environment_offset()
        } else {
            8000 + get_environment_offset()
        }
    })
}

/// Get port range end from environment or calculated from start
#[must_use]
pub fn get_port_range_end() -> u16 {
    SafeEnv::parse("SONGBIRD_PORT_END", {
        let start = get_port_range_start();
        start + get_port_range_size()
    })
}

/// Calculate environment-specific port offset
fn get_environment_offset() -> u16 {
    match SafeEnv::get("SONGBIRD_ENV").as_deref() {
        Ok("production") => 0,
        Ok("staging") => 100,
        Ok("testing") => 200,
        Ok("development") => 300,
        _ => {
            // Calculate based on user ID for multi-user systems
            calculate_user_port_offset()
        }
    }
}

/// Calculate port range size based on expected service count
fn get_port_range_size() -> u16 {
    SafeEnv::parse("SONGBIRD_PORT_RANGE_SIZE", {
        // Calculate based on enabled services and expected scale
        let base_size = 1000;
        let service_multiplier = get_expected_service_count();
        (base_size + service_multiplier * 10).min(65535 - get_port_range_start())
    })
}

/// Calculate user-specific port offset to avoid conflicts in multi-user environments
fn calculate_user_port_offset() -> u16 {
    // Use user ID hash for deterministic but unique offset
    let user = SafeEnv::get("USER")
        .or_else(|_| SafeEnv::get("USERNAME"))
        .unwrap_or_else(|_| "default".to_string());
    let hash = user.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(u32::from(b)));
    (hash % 500) as u16 // Limit to reasonable range
}

/// Get expected service count from configuration or environment
fn get_expected_service_count() -> u16 {
    SafeEnv::parse("SONGBIRD_EXPECTED_SERVICES", {
        // Calculate based on enabled primals and features
        let mut count = 1; // Base Songbird service

        if SafeEnv::get_bool("SONGBIRD_ENABLE_BEARDOG", false) {
            count += 1;
        }
        if SafeEnv::get_bool("SONGBIRD_ENABLE_STORAGE_PROVIDER", false)
            || SafeEnv::get_bool("SONGBIRD_ENABLE_NESTGATE", false)
        {
            count += 1;
        }
        if SafeEnv::get_bool("SONGBIRD_ENABLE_COMPUTE_PROVIDER", false)
            || SafeEnv::get_bool("SONGBIRD_ENABLE_TOADSTOOL", false)
        {
            count += 1;
        }
        if SafeEnv::get_bool("SONGBIRD_ENABLE_AI_PROVIDER", false)
            || SafeEnv::get_bool("SONGBIRD_ENABLE_SQUIRREL", false)
        {
            count += 1;
        }
        if SafeEnv::get_bool("SONGBIRD_ENABLE_DISCOVERY", true) {
            count += 1;
        }
        if SafeEnv::get_bool("SONGBIRD_ENABLE_METRICS", true) {
            count += 1;
        }
        if SafeEnv::get_bool("SONGBIRD_ENABLE_HEALTH_CHECK", true) {
            count += 1;
        }

        count
    })
}

/// Calculate deterministic port offset for any primal name.
///
/// Uses consistent hashing to assign port offsets — no hardcoded primal names.
/// The same name always produces the same offset, ensuring deterministic behavior
/// while being fully agnostic to which primals exist.
fn get_primal_port_offset(primal_type: &str) -> u16 {
    let hash = primal_type
        .to_lowercase()
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(u32::from(b)));
    // Map to range 10..999 to avoid port 0 offset
    10 + (hash % 990) as u16
}

/// Get common primal service ports from environment.
///
/// Dynamically discovers enabled primals via `SONGBIRD_ENABLE_*` env vars
/// rather than hardcoding specific primal names. Any primal can be enabled
/// by setting `SONGBIRD_ENABLE_{NAME}=true`.
#[must_use]
pub fn get_common_primal_ports() -> Vec<u16> {
    SafeEnv::get_or_default("SONGBIRD_COMMON_PORTS", {
        let mut ports = Vec::new();
        let base_port = get_port_range_start();

        // Always include the main service port
        ports.push(base_port);

        // Dynamically discover enabled primals from env vars
        for (key, value) in std::env::vars() {
            if let Some(primal_name) = key.strip_prefix("SONGBIRD_ENABLE_")
                && (value.eq_ignore_ascii_case("true") || value == "1")
            {
                let name = primal_name.to_lowercase();
                ports.push(base_port + get_primal_port_offset(&name));
            }
        }

        ports.into_iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",")
    })
    .split(',')
    .filter_map(|s| s.trim().parse().ok())
    .collect()
}
