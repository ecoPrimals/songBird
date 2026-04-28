// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Bind address, dynamic port range, and common primal port discovery.

use songbird_types::error_helpers::SafeEnv;

/// Default configuration file path
pub const DEFAULT_CONFIG_PATH: &str = "songbird.toml";

/// IPv4 localhost address constant — re-export from canonical source.
pub const LOCALHOST_IPV4: &str = songbird_types::constants::LOCALHOST;

/// Default bind address for backwards compatibility with tests.
///
/// Production code should use [`get_bind_address()`] which respects environment configuration.
#[deprecated(since = "0.2.1", note = "Use get_bind_address() for runtime-aware binding")]
pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:8080";

/// Default localhost constant — re-export from canonical source.
pub const DEFAULT_LOCALHOST: &str = songbird_types::constants::LOCALHOST;

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
        songbird_types::constants::PRODUCTION_BIND_ADDRESS.to_string()
    } else {
        songbird_types::constants::DEVELOPMENT_BIND_ADDRESS.to_string()
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
        // Calculate based on enabled providers and features
        let mut count = 1; // Base Songbird service

        if SafeEnv::get_bool("SONGBIRD_ENABLE_SECURITY_PROVIDER", false) {
            count += 1;
        }
        if SafeEnv::get_bool("SONGBIRD_ENABLE_STORAGE_PROVIDER", false) {
            count += 1;
        }
        if SafeEnv::get_bool("SONGBIRD_ENABLE_COMPUTE_PROVIDER", false) {
            count += 1;
        }
        if SafeEnv::get_bool("SONGBIRD_ENABLE_AI_PROVIDER", false) {
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

        // Dynamically discover enabled primals from env vars (use overlay-aware iteration)
        for (key, value) in songbird_process_env::vars() {
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use songbird_test_utils::ScopedEnv;

    #[test]
    fn public_string_constants_match_expected() {
        assert_eq!(DEFAULT_CONFIG_PATH, "songbird.toml");
        assert_eq!(LOCALHOST_IPV4, "127.0.0.1");
        assert_eq!(DEFAULT_LOCALHOST, "127.0.0.1");
        assert!(DEFAULT_BIND_ADDRESS.contains(':'));
    }

    #[tokio::test]
    async fn get_bind_address_uses_valid_env_ip() {
        let _bind = ScopedEnv::set("SONGBIRD_BIND_ADDRESS", "10.11.12.13").await;
        assert_eq!(get_bind_address(), "10.11.12.13");
    }

    #[tokio::test]
    async fn get_bind_address_ignores_invalid_env_ip_and_uses_localhost_in_dev() {
        let _e = ScopedEnv::remove_and_set_many(
            ["KUBERNETES_SERVICE_HOST", "CONTAINER", "SONGBIRD_ENV"],
            [("SONGBIRD_BIND_ADDRESS", "not-an-ip-address")],
        )
        .await;
        assert_eq!(get_bind_address(), "127.0.0.1");
    }

    #[tokio::test]
    async fn get_bind_address_production_container_binds_all_interfaces() {
        let _e =
            ScopedEnv::remove_and_set_many(["SONGBIRD_BIND_ADDRESS"], [("CONTAINER", "1")]).await;
        assert_eq!(get_bind_address(), "0.0.0.0");
    }

    #[tokio::test]
    async fn get_port_range_start_end_respect_explicit_env() {
        let _e = ScopedEnv::set_multiple([
            ("SONGBIRD_PORT_START", "9100"),
            ("SONGBIRD_PORT_END", "9200"),
            ("SONGBIRD_PORT_RANGE_SIZE", "50"),
        ])
        .await;
        assert_eq!(get_port_range_start(), 9100);
        assert_eq!(get_port_range_end(), 9200);
    }

    #[tokio::test]
    async fn get_common_primal_ports_parses_override_list() {
        let _e = ScopedEnv::set("SONGBIRD_COMMON_PORTS", "7001, 7002 ,7003").await;
        let ports = get_common_primal_ports();
        assert_eq!(ports, vec![7001, 7002, 7003]);
    }

    #[tokio::test]
    async fn get_common_primal_ports_includes_base_and_enabled_primal_offsets() {
        let _e = ScopedEnv::remove_and_set_many(
            ["SONGBIRD_COMMON_PORTS"],
            [("SONGBIRD_PORT_START", "8000"), ("SONGBIRD_ENABLE_ALPHA", "true")],
        )
        .await;
        let ports = get_common_primal_ports();
        assert!(ports.contains(&8000));
        let name = "alpha";
        let hash =
            name.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(u32::from(b)));
        let offset = 10 + (hash % 990) as u16;
        assert!(ports.contains(&(8000 + offset)));
    }
}
