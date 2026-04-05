// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Bind addresses, port ranges, and discovery-related port defaults.

use std::net::IpAddr;

use tracing::warn;

use super::{env_get_bool_with, env_parse_with, env_port_with, read_process_env};

/// Default configuration file path
pub const DEFAULT_CONFIG_PATH: &str = "songbird.toml";

/// Documented fallbacks for URL/protocol construction.
///
/// Override via `SONGBIRD_BASE_URL`, `SONGBIRD_PRODUCTION_HTTPS_PORT`, `SONGBIRD_STAGING_HTTP_PORT`,
/// `SONGBIRD_CANONICAL_*_PORT`, or `SONGBIRD_PROTOCOL_*_PORT`.
pub const FALLBACK_PRODUCTION_HTTPS_PORT: u16 = 8443;
pub const FALLBACK_STAGING_HTTP_PORT: u16 = 8080;
pub const FALLBACK_CANONICAL_DISCOVERY_PORT: u16 = 8081;
pub const FALLBACK_CANONICAL_SECURITY_PORT: u16 = 8443;
pub const FALLBACK_CANONICAL_ORCHESTRATOR_PORT: u16 = 8080;
pub const FALLBACK_CANONICAL_GAMING_PORT: u16 = 6112;
pub const FALLBACK_PROTOCOL_UDP_PORT: u16 = 6112;
pub const FALLBACK_PROTOCOL_TCP_PORT: u16 = 6113;
pub const FALLBACK_PROTOCOL_WEBSOCKET_PORT: u16 = 8080;
pub const FALLBACK_PROTOCOL_SECURE_WEBSOCKET_PORT: u16 = 8443;

/// Get bind address from environment or calculate from system capabilities
#[must_use]
pub fn get_bind_address() -> String {
    get_bind_address_with(&read_process_env)
}

/// Same as [`get_bind_address`] with an injectable env reader.
#[must_use]
pub fn get_bind_address_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> String {
    if let Ok(addr) = env("SONGBIRD_BIND_ADDRESS")
        && addr.parse::<IpAddr>().is_ok()
    {
        return addr;
    }

    if env("KUBERNETES_SERVICE_HOST").is_ok()
        || env("CONTAINER").is_ok()
        || env("SONGBIRD_ENV").ok().as_deref() == Some("production")
    {
        "0.0.0.0".to_string()
    } else {
        "127.0.0.1".to_string()
    }
}

/// Get canonical bind address based on environment (alias for compatibility)
#[must_use]
pub fn get_canonical_bind_address() -> String {
    get_bind_address()
}

/// Get default bind address for the current environment (alias for compatibility)
#[must_use]
pub fn get_default_bind_address() -> String {
    get_bind_address()
}

/// Get default bind address for the current environment (alias for compatibility)
#[must_use]
pub fn default_bind_address() -> String {
    get_bind_address()
}

/// Get port range start from environment or system-based calculation
#[must_use]
pub fn get_port_range_start() -> u16 {
    get_port_range_start_with(&read_process_env)
}

/// Same as [`get_port_range_start`] with an injectable env reader.
#[must_use]
pub fn get_port_range_start_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> u16 {
    env_parse_with(env, "SONGBIRD_PORT_START", {
        // Calculate based on environment and user permissions
        if env("SONGBIRD_ALLOW_PRIVILEGED_PORTS").is_ok() {
            80 + get_environment_offset_with(env)
        } else {
            8000 + get_environment_offset_with(env)
        }
    })
}

/// Get port range end from environment or calculated from start
#[must_use]
pub fn get_port_range_end() -> u16 {
    get_port_range_end_with(&read_process_env)
}

/// Same as [`get_port_range_end`] with an injectable env reader.
#[must_use]
pub fn get_port_range_end_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> u16 {
    env_parse_with(env, "SONGBIRD_PORT_END", {
        let start = get_port_range_start_with(env);
        start + get_port_range_size_with(env)
    })
}

fn get_environment_offset_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> u16 {
    match env("SONGBIRD_ENV").as_deref() {
        Ok("production") => 0,
        Ok("staging") => 100,
        Ok("testing") => 200,
        Ok("development") => 300,
        _ => {
            // Calculate based on user ID for multi-user systems
            calculate_user_port_offset_with(env)
        }
    }
}

/// Calculate port range size based on expected service count
fn get_port_range_size_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> u16 {
    env_parse_with(env, "SONGBIRD_PORT_RANGE_SIZE", {
        // Calculate based on enabled services and expected scale
        let base_size = 1000;
        let service_multiplier = get_expected_service_count_with(env);
        (base_size + service_multiplier * 10).min(65535 - get_port_range_start_with(env))
    })
}

/// Calculate user-specific port offset to avoid conflicts in multi-user environments
fn calculate_user_port_offset_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> u16 {
    // Use user ID hash for deterministic but unique offset
    let user = env("USER").or_else(|_| env("USERNAME")).unwrap_or_else(|_| "default".to_string());
    let hash = user.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(u32::from(b)));
    (hash % 500) as u16 // Limit to reasonable range
}

/// Get expected service count from configuration or environment
fn get_expected_service_count_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> u16 {
    env_parse_with(env, "SONGBIRD_EXPECTED_SERVICES", {
        // Calculate based on enabled providers and features
        let mut count = 1; // Base Songbird service

        let legacy_beardog_enabled = env_get_bool_with(env, "SONGBIRD_ENABLE_BEARDOG", false);
        if legacy_beardog_enabled {
            warn!(
                "SONGBIRD_ENABLE_BEARDOG is deprecated; use SONGBIRD_ENABLE_SECURITY_PROVIDER instead"
            );
        }
        if env_get_bool_with(env, "SONGBIRD_ENABLE_SECURITY_PROVIDER", false)
            || legacy_beardog_enabled
        {
            count += 1;
        }
        let legacy_nestgate_enabled = env_get_bool_with(env, "SONGBIRD_ENABLE_NESTGATE", false);
        if legacy_nestgate_enabled {
            warn!(
                "SONGBIRD_ENABLE_NESTGATE is deprecated; use SONGBIRD_ENABLE_STORAGE_PROVIDER instead"
            );
        }
        if env_get_bool_with(env, "SONGBIRD_ENABLE_STORAGE_PROVIDER", false)
            || legacy_nestgate_enabled
        {
            count += 1;
        }
        let legacy_toadstool_enabled = env_get_bool_with(env, "SONGBIRD_ENABLE_TOADSTOOL", false);
        if legacy_toadstool_enabled {
            warn!(
                "SONGBIRD_ENABLE_TOADSTOOL is deprecated; use SONGBIRD_ENABLE_COMPUTE_PROVIDER instead"
            );
        }
        if env_get_bool_with(env, "SONGBIRD_ENABLE_COMPUTE_PROVIDER", false)
            || legacy_toadstool_enabled
        {
            count += 1;
        }
        let legacy_squirrel_enabled = env_get_bool_with(env, "SONGBIRD_ENABLE_SQUIRREL", false);
        if legacy_squirrel_enabled {
            warn!(
                "SONGBIRD_ENABLE_SQUIRREL is deprecated; use SONGBIRD_ENABLE_AI_PROVIDER instead"
            );
        }
        if env_get_bool_with(env, "SONGBIRD_ENABLE_AI_PROVIDER", false) || legacy_squirrel_enabled {
            count += 1;
        }
        if env_get_bool_with(env, "SONGBIRD_ENABLE_DISCOVERY", true) {
            count += 1;
        }
        if env_get_bool_with(env, "SONGBIRD_ENABLE_METRICS", true) {
            count += 1;
        }
        if env_get_bool_with(env, "SONGBIRD_ENABLE_HEALTH_CHECK", true) {
            count += 1;
        }

        count
    })
}

/// Get dashboard port from environment or calculated default
#[must_use]
pub fn get_dashboard_port() -> u16 {
    get_dashboard_port_with(&read_process_env)
}

/// Same as [`get_dashboard_port`] with an injectable env reader.
#[must_use]
pub fn get_dashboard_port_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> u16 {
    env_parse_with(env, "SONGBIRD_DASHBOARD_PORT", {
        // Calculate based on environment
        match env("SONGBIRD_ENV").as_deref() {
            Ok("production") => 3000, // Standard port for production
            Ok("staging") => 3001,    // Staging offset
            Ok("testing") => 3002,    // Testing offset
            _ => 8083,                // Development default
        }
    })
}

/// Get default discovery port
#[must_use]
pub fn default_discovery_port() -> u16 {
    default_discovery_port_with(&read_process_env)
}

/// Same as [`default_discovery_port`] with an injectable env reader.
#[must_use]
pub fn default_discovery_port_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> u16 {
    env_port_with(env, "SONGBIRD_DISCOVERY_PORT", 5678)
}
