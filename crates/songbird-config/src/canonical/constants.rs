// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! # 🎯 Canonical Constants - Environment-Aware Defaults
//!
//! **ZERO HARDCODING SYSTEM**
//!
//! This module provides environment-aware defaults that eliminate hardcoded values
//! while maintaining secure defaults for development and production.
//!
//! **Status**: Phase 4 consolidation complete - merged from config/constants.rs
//! **Philosophy**: All values configurable via environment, calculated defaults for production

#![allow(
    missing_docs,
    reason = "constants are self-describing; top-level module doc explains policy"
)]

use songbird_types::error_helpers::SafeEnv;
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;

/// Process environment lookup (function pointer satisfies HRTB for injectable env readers).
pub(crate) fn read_process_env(key: &str) -> Result<String, std::env::VarError> {
    std::env::var(key)
}

pub(crate) fn env_parse_with<T: std::str::FromStr>(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: T,
) -> T {
    env(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

pub(crate) fn env_get_bool_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: bool,
) -> bool {
    env(key)
        .ok()
        .and_then(|v| match v.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => v.parse().ok(),
        })
        .unwrap_or(default)
}

pub(crate) fn env_get_or_default_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: impl Into<String>,
) -> String {
    env(key).unwrap_or_else(|_| default.into())
}

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

// NOTE: Removed hardcoded network constants per sovereignty principles
// Tests should set their own environment variables instead of relying on these constants
// For migration: Use get_bind_address() which is environment-aware

// ==================== NETWORK CONFIGURATION ====================

/// Get bind address from environment or calculate from system capabilities
#[must_use]
pub fn get_bind_address() -> String {
    get_bind_address_with(&read_process_env)
}

/// Same as [`get_bind_address`](get_bind_address) with an injectable env reader.
#[must_use]
pub fn get_bind_address_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> String {
    if let Ok(addr) = env("SONGBIRD_BIND_ADDRESS") {
        if addr.parse::<IpAddr>().is_ok() {
            return addr;
        }
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

// ==================== PORT CONFIGURATION ====================

/// Get port range start from environment or system-based calculation
#[must_use]
pub fn get_port_range_start() -> u16 {
    get_port_range_start_with(&read_process_env)
}

/// Same as [`get_port_range_start`](get_port_range_start) with an injectable env reader.
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

/// Same as [`get_port_range_end`](get_port_range_end) with an injectable env reader.
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
        // Calculate based on enabled primals and features
        let mut count = 1; // Base Songbird service

        if env_get_bool_with(env, "SONGBIRD_ENABLE_BEARDOG", false) {
            count += 1;
        }
        if env_get_bool_with(env, "SONGBIRD_ENABLE_NESTGATE", false) {
            count += 1;
        }
        if env_get_bool_with(env, "SONGBIRD_ENABLE_TOADSTOOL", false) {
            count += 1;
        }
        if env_get_bool_with(env, "SONGBIRD_ENABLE_SQUIRREL", false) {
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

/// Same as [`get_dashboard_port`](get_dashboard_port) with an injectable env reader.
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

/// Same as [`default_discovery_port`](default_discovery_port) with an injectable env reader.
#[must_use]
pub fn default_discovery_port_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> u16 {
    env_port_with(env, "SONGBIRD_DISCOVERY_PORT", 5678)
}

// ==================== TIMEOUT CONFIGURATION ====================

/// Get connection timeout from environment or calculate based on network conditions
#[must_use]
pub fn get_connection_timeout_ms() -> u64 {
    get_connection_timeout_ms_with(&read_process_env)
}

/// Same as [`get_connection_timeout_ms`](get_connection_timeout_ms) with an injectable env reader.
#[must_use]
pub fn get_connection_timeout_ms_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> u64 {
    env_parse_with(env, "SONGBIRD_CONNECTION_TIMEOUT_MS", {
        match env("SONGBIRD_ENV").as_deref() {
            Ok("production") => 30000,  // 30 seconds for production
            Ok("staging") => 45000,     // 45 seconds for staging
            Ok("development") => 60000, // 60 seconds for development
            _ => calculate_network_based_timeout_with(env),
        }
    })
}

/// Calculate timeout based on detected network conditions
fn calculate_network_based_timeout_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> u64 {
    // Check if we're in a cloud/container environment
    if env("KUBERNETES_SERVICE_HOST").is_ok()
        || env("AWS_EXECUTION_ENV").is_ok()
        || env("GOOGLE_CLOUD_PROJECT").is_ok()
        || env("AZURE_CLIENT_ID").is_ok()
    {
        15000 // Fast cloud networks
    } else {
        30000 // Conservative default for unknown networks
    }
}

/// Default cache TTL
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

/// Default evaluation timeout
pub const DEFAULT_EVALUATION_TIMEOUT: Duration = Duration::from_secs(30);

/// Default metrics interval
pub const DEFAULT_METRICS_INTERVAL: Duration = Duration::from_secs(60);

// ==================== RESOURCE MANAGEMENT ====================

/// Get maximum connections allowed
#[must_use]
pub fn get_max_connections() -> usize {
    get_max_connections_with(&read_process_env)
}

/// Same as [`get_max_connections`](get_max_connections) with an injectable env reader.
#[must_use]
pub fn get_max_connections_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> usize {
    env_parse_with(env, "SONGBIRD_MAX_CONNECTIONS", {
        match env("SONGBIRD_ENV").as_deref() {
            Ok("production") => 10000,
            Ok("staging") => 5000,
            Ok("testing") => 1000,
            _ => 2000, // Development default
        }
    })
}

/// Get worker thread count based on system resources
#[must_use]
pub fn get_worker_threads() -> usize {
    get_worker_threads_with(&read_process_env)
}

/// Same as [`get_worker_threads`](get_worker_threads) with an injectable env reader.
#[must_use]
pub fn get_worker_threads_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> usize {
    env_parse_with(env, "SONGBIRD_WORKER_THREADS", {
        // Use CPU count or container limits
        // Fallback to 4 threads
        std::thread::available_parallelism().map(std::num::NonZero::get).unwrap_or(4)
    })
}

/// Get buffer pool size based on available memory
#[must_use]
pub fn get_buffer_pool_size() -> usize {
    get_buffer_pool_size_with(&read_process_env)
}

/// Same as [`get_buffer_pool_size`](get_buffer_pool_size) with an injectable env reader.
#[must_use]
pub fn get_buffer_pool_size_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> usize {
    env_parse_with(env, "SONGBIRD_BUFFER_POOL_SIZE", {
        // Calculate based on available memory
        let base_size = match env("SONGBIRD_ENV").as_deref() {
            Ok("production") => 10000,
            Ok("staging") => 5000,
            Ok("development") => 1000,
            _ => 2000,
        };

        // Adjust for container memory limits
        env("MEMORY_LIMIT").ok().and_then(|memory_limit| memory_limit.parse::<u64>().ok()).map_or(
            base_size,
            |limit_mb| {
                // Use 1% of available memory for buffer pool
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "MEMORY_LIMIT parsed as u64; product scaled down for pool sizing"
                )]
                let adjusted_size = (limit_mb as usize * 10) / 1024;
                std::cmp::min(base_size, adjusted_size)
            },
        )
    })
}

/// Get batch processing size based on workload characteristics
#[must_use]
pub fn get_batch_size() -> usize {
    get_batch_size_with(&read_process_env)
}

/// Same as [`get_batch_size`](get_batch_size) with an injectable env reader.
#[must_use]
pub fn get_batch_size_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> usize {
    env_parse_with(env, "SONGBIRD_BATCH_SIZE", {
        // Calculate optimal batch size based on system characteristics
        let cpu_count = get_worker_threads_with(env);
        let memory_factor = if env("MEMORY_LIMIT").is_ok() {
            500
        } else {
            1000
        };

        (cpu_count * memory_factor).clamp(100, 5000)
    })
}

/// Check if zero-copy optimizations should be enabled
#[must_use]
pub fn enable_zero_copy() -> bool {
    enable_zero_copy_with(&read_process_env)
}

/// Same as [`enable_zero_copy`](enable_zero_copy) with an injectable env reader.
#[must_use]
pub fn enable_zero_copy_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> bool {
    env_get_bool_with(env, "SONGBIRD_ENABLE_ZERO_COPY", {
        // Enable zero-copy in production and for high-performance environments
        match env("SONGBIRD_ENV").as_deref() {
            Ok("production" | "staging") => true,
            _ => {
                // Enable if system has sufficient memory
                env("MEMORY_LIMIT")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .is_none_or(|mb| mb > 2048) // Default to enabled
            }
        }
    })
}

// ==================== PRIMAL CONFIGURATION ====================

/// Universal primal endpoint discovery - works with any primal name
#[must_use]
pub fn get_primal_endpoint(primal_name: &str) -> String {
    get_primal_endpoint_with(primal_name, &read_process_env)
}

/// Same as [`get_primal_endpoint`](get_primal_endpoint) with an injectable env reader.
#[must_use]
pub fn get_primal_endpoint_with(
    primal_name: &str,
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> String {
    // First try primal-specific environment variable
    let env_var = format!("{}_ENDPOINT", primal_name.to_uppercase());
    if let Ok(endpoint) = env(&env_var) {
        return endpoint;
    }

    // Try generic primal endpoint pattern
    let generic_env = format!("PRIMAL_{}_ENDPOINT", primal_name.to_uppercase());
    if let Ok(endpoint) = env(&generic_env) {
        return endpoint;
    }

    // Calculate default endpoint based on environment and primal name
    calculate_default_primal_endpoint(primal_name)
}

/// Calculate default endpoint for any primal based on naming conventions
fn calculate_default_primal_endpoint(primal_name: &str) -> String {
    let base_port = get_port_range_start();
    let primal_offset = calculate_primal_port_offset(primal_name);
    let port = base_port + primal_offset;

    let host = if SafeEnv::get("KUBERNETES_SERVICE_HOST").is_ok() {
        // Kubernetes service discovery pattern
        format!("{}-service", primal_name.to_lowercase())
    } else if SafeEnv::get("DOCKER_HOST").is_ok() || SafeEnv::get("CONTAINER").is_ok() {
        // Docker container pattern
        primal_name.to_lowercase()
    } else {
        // Local development pattern
        get_bind_address()
    };

    let protocol = if should_use_tls_for_primal(primal_name) {
        "https"
    } else {
        "http"
    };

    format!("{protocol}://{host}:{port}")
}

/// Calculate port offset for any primal name using consistent hashing
fn calculate_primal_port_offset(primal_name: &str) -> u16 {
    // Use consistent hashing to assign port offsets
    // This ensures the same primal name always gets the same offset
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    primal_name.hash(&mut hasher);
    let hash = hasher.finish();

    // Map hash to reasonable port offset (0-999)
    (hash % 1000) as u16
}

/// Calculate deterministic port offset for any primal name.
///
/// Delegates to `calculate_primal_port_offset` — no hardcoded primal names.
fn get_primal_port_offset(primal_type: &str) -> u16 {
    calculate_primal_port_offset(primal_type)
}

/// Determine if primal should use TLS based on environment and naming
fn should_use_tls_for_primal(primal_name: &str) -> bool {
    should_use_tls_for_primal_with(primal_name, &read_process_env)
}

fn should_use_tls_for_primal_with(
    primal_name: &str,
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> bool {
    // Check primal-specific TLS setting
    let tls_env = format!("{}_USE_TLS", primal_name.to_uppercase());
    if env_get_bool_with(env, &tls_env, false) {
        return true;
    }

    // Security-related primals default to TLS in production
    let is_security_primal = primal_name.to_lowercase().contains("security")
        || primal_name.to_lowercase().contains("auth")
        || primal_name.to_lowercase().contains("crypto");

    match env("SONGBIRD_ENV").as_deref() {
        Ok("production") => true,
        Ok("staging") => is_security_primal,
        _ => false, // Development default
    }
}

/// Get all configured primal names from a snapshot of environment variables.
#[must_use]
pub fn get_configured_primal_names_in_env(env: &HashMap<String, String>) -> Vec<String> {
    let mut primal_names = Vec::new();

    for key in env.keys() {
        if key.ends_with("_ENDPOINT") && !key.starts_with("SONGBIRD_") {
            let primal_name = key.trim_end_matches("_ENDPOINT").to_lowercase();
            if !primal_names.contains(&primal_name) {
                primal_names.push(primal_name);
            }
        }

        if key.starts_with("PRIMAL_")
            && key.ends_with("_ENDPOINT")
            && let Some(primal_part) =
                key.strip_prefix("PRIMAL_").and_then(|s| s.strip_suffix("_ENDPOINT"))
        {
            let primal_name = primal_part.to_lowercase();
            if !primal_names.contains(&primal_name) {
                primal_names.push(primal_name);
            }
        }
    }

    primal_names
}

/// Get all configured primal names from environment
#[must_use]
pub fn get_configured_primal_names() -> Vec<String> {
    let map: HashMap<String, String> = std::env::vars().collect();
    get_configured_primal_names_in_env(&map)
}

/// Get common primal service ports from environment.
///
/// Dynamically discovers enabled primals via `SONGBIRD_ENABLE_*` env vars
/// rather than hardcoding specific primal names. Any primal can be enabled
/// by setting `SONGBIRD_ENABLE_{NAME}=true`.
#[must_use]
pub fn get_common_primal_ports() -> Vec<u16> {
    let map: HashMap<String, String> = std::env::vars().collect();
    get_common_primal_ports_from_env_map(&map)
}

/// Common primal ports from a snapshot of environment variables (concurrent-safe tests).
#[must_use]
pub fn get_common_primal_ports_from_env_map(env: &HashMap<String, String>) -> Vec<u16> {
    let default_computed = {
        let mut ports = Vec::new();
        let base_port = get_port_range_start();

        // Always include the main service port
        ports.push(base_port);

        // Dynamically discover enabled primals from env vars
        for (key, value) in env {
            if let Some(primal_name) = key.strip_prefix("SONGBIRD_ENABLE_")
                && (value.eq_ignore_ascii_case("true") || value == "1")
            {
                let name = primal_name.to_lowercase();
                ports.push(base_port + get_primal_port_offset(&name));
            }
        }

        ports.into_iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",")
    };

    env.get("SONGBIRD_COMMON_PORTS")
        .cloned()
        .unwrap_or(default_computed)
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect()
}

fn normalize_capability_env_key(capability: &str) -> String {
    capability.trim().to_lowercase().replace(['-', ' '], "_").to_uppercase()
}

fn normalize_capability_match_key(s: &str) -> String {
    s.trim().to_lowercase().replace(['-', ' '], "_")
}

fn trim_nonempty(s: &str) -> Option<&str> {
    let t = s.trim();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

fn capability_env_for_primal_in(
    primal_lower: &str,
    env: &HashMap<String, String>,
) -> Option<String> {
    let u = primal_lower.to_uppercase();
    env.get(&format!("{u}_CAPABILITIES"))
        .cloned()
        .or_else(|| env.get(&format!("PRIMAL_{u}_CAPABILITIES")).cloned())
        .or_else(|| env.get(&format!("SONGBIRD_PRIMAL_{u}_CAPABILITIES")).cloned())
}

fn primal_declares_capability_in(
    primal_lower: &str,
    capability_query: &str,
    env: &HashMap<String, String>,
) -> bool {
    let Some(list) = capability_env_for_primal_in(primal_lower, env) else {
        return false;
    };
    let want = normalize_capability_match_key(capability_query);
    list.split(',').filter_map(trim_nonempty).map(normalize_capability_match_key).any(|c| c == want)
}

/// Filters by capability using an env snapshot (tests avoid mutating process environment).
#[must_use]
pub fn find_primals_with_capability_in_env(
    capability: &str,
    env: &HashMap<String, String>,
) -> Vec<String> {
    let key = normalize_capability_env_key(capability);
    let providers_key = format!("SONGBIRD_CAPABILITY_{key}_PROVIDERS");
    if let Some(raw) = env.get(&providers_key) {
        let providers: Vec<String> =
            raw.split(',').filter_map(trim_nonempty).map(str::to_lowercase).collect();
        if !providers.is_empty() {
            return providers;
        }
    }

    let mut out = Vec::new();
    for primal in get_configured_primal_names_in_env(env) {
        if primal_declares_capability_in(&primal, capability, env) && !out.contains(&primal) {
            out.push(primal);
        }
    }
    out
}

/// Filters by capability: `SONGBIRD_CAPABILITY_<CAP>_PROVIDERS` first, else primals from `*_ENDPOINT`
/// with matching `{ID}_CAPABILITIES` / `PRIMAL_{ID}_CAPABILITIES` / `SONGBIRD_PRIMAL_{ID}_CAPABILITIES`.
#[must_use]
pub fn find_primals_with_capability(capability: &str) -> Vec<String> {
    let map: HashMap<String, String> = std::env::vars().collect();
    find_primals_with_capability_in_env(capability, &map)
}

fn env_port_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: u16,
) -> u16 {
    env(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

fn env_or_default_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
    key: &str,
    default: impl Into<String>,
) -> String {
    env(key).unwrap_or_else(|_| default.into())
}

fn default_production_base_url_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> String {
    let host = get_bind_address_with(env);
    let port = env_port_with(env, "SONGBIRD_PRODUCTION_HTTPS_PORT", FALLBACK_PRODUCTION_HTTPS_PORT);
    format!("https://{host}:{port}")
}

fn default_staging_base_url_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> String {
    let host = get_bind_address_with(env);
    let port = env_port_with(env, "SONGBIRD_STAGING_HTTP_PORT", FALLBACK_STAGING_HTTP_PORT);
    format!("http://{host}:{port}")
}

// ==================== ENDPOINT CONFIGURATION ====================

/// Get canonical endpoint URL based on environment and service (injectable env reader).
#[must_use]
pub fn get_canonical_endpoint_with(
    service_name: &str,
    default_port: u16,
    env: impl Fn(&str) -> Result<String, std::env::VarError>,
) -> String {
    let base_url = match env_or_default_with(&env, "SONGBIRD_ENVIRONMENT", "development").as_str() {
        "production" | "prod" => {
            env_or_default_with(&env, "SONGBIRD_BASE_URL", default_production_base_url_with(&env))
        }
        "staging" => env_or_default_with(
            &env,
            "SONGBIRD_BASE_URL",
            env_or_default_with(
                &env,
                "SONGBIRD_STAGING_BASE_URL",
                default_staging_base_url_with(&env),
            ),
        ),
        _ => {
            let host = get_bind_address_with(&env);
            env_or_default_with(&env, "SONGBIRD_BASE_URL", format!("http://{host}:{default_port}"))
        }
    };

    env_or_default_with(
        &env,
        &format!("SONGBIRD_{}_ENDPOINT", service_name.to_uppercase()),
        base_url,
    )
}

/// Get canonical endpoint URL based on environment and service
#[must_use]
pub fn get_canonical_endpoint(service_name: &str, default_port: u16) -> String {
    get_canonical_endpoint_with(service_name, default_port, read_process_env)
}

/// Get canonical discovery endpoint
#[must_use]
pub fn get_canonical_discovery_endpoint() -> String {
    let default_port =
        SafeEnv::get_port("SONGBIRD_CANONICAL_DISCOVERY_PORT", FALLBACK_CANONICAL_DISCOVERY_PORT);
    get_canonical_endpoint("discovery", default_port)
}

/// Get canonical security endpoint
#[must_use]
pub fn get_canonical_security_endpoint() -> String {
    let default_port =
        SafeEnv::get_port("SONGBIRD_CANONICAL_SECURITY_PORT", FALLBACK_CANONICAL_SECURITY_PORT);
    get_canonical_endpoint("security", default_port)
}

/// Get canonical orchestrator endpoint
#[must_use]
pub fn get_canonical_orchestrator_endpoint() -> String {
    let default_port = SafeEnv::get_port(
        "SONGBIRD_CANONICAL_ORCHESTRATOR_PORT",
        FALLBACK_CANONICAL_ORCHESTRATOR_PORT,
    );
    get_canonical_endpoint("orchestrator", default_port)
}

/// Get canonical gaming endpoint
#[must_use]
pub fn get_canonical_gaming_endpoint() -> String {
    let default_port =
        SafeEnv::get_port("SONGBIRD_CANONICAL_GAMING_PORT", FALLBACK_CANONICAL_GAMING_PORT);
    get_canonical_endpoint("gaming", default_port)
}

// ==================== DIRECTORY CONFIGURATION ====================

/// Get log directory from environment or calculate default
#[must_use]
pub fn get_log_dir() -> String {
    get_log_dir_with(&read_process_env)
}

/// Same as [`get_log_dir`](get_log_dir) with an injectable env reader.
#[must_use]
pub fn get_log_dir_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> String {
    env_get_or_default_with(env, "SONGBIRD_LOG_DIR", {
        // Use platform-appropriate log directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Local\\Songbird\\logs",
                env_get_or_default_with(env, "USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!(
                "{}/.local/share/songbird/logs",
                env_get_or_default_with(env, "HOME", "/tmp".to_string()),
            )
        }
    })
}

/// Get cache directory from environment or calculate default
#[must_use]
pub fn get_cache_dir() -> String {
    get_cache_dir_with(&read_process_env)
}

/// Same as [`get_cache_dir`](get_cache_dir) with an injectable env reader.
#[must_use]
pub fn get_cache_dir_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> String {
    env_get_or_default_with(env, "SONGBIRD_CACHE_DIR", {
        // Use platform-appropriate cache directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Local\\Songbird\\cache",
                env_get_or_default_with(env, "USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!("{}/.cache/songbird", env_get_or_default_with(env, "HOME", "/tmp".to_string()),)
        }
    })
}

/// Get data directory from environment or calculate default
#[must_use]
pub fn get_data_dir() -> String {
    get_data_dir_with(&read_process_env)
}

/// Same as [`get_data_dir`](get_data_dir) with an injectable env reader.
#[must_use]
pub fn get_data_dir_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> String {
    env_get_or_default_with(env, "SONGBIRD_DATA_DIR", {
        // Use platform-appropriate data directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Roaming\\Songbird",
                env_get_or_default_with(env, "USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!(
                "{}/.local/share/songbird",
                env_get_or_default_with(env, "HOME", "/tmp".to_string()),
            )
        }
    })
}

/// Get configuration directory from environment or calculate default
#[must_use]
pub fn get_config_dir() -> String {
    get_config_dir_with(&read_process_env)
}

/// Same as [`get_config_dir`](get_config_dir) with an injectable env reader.
#[must_use]
pub fn get_config_dir_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> String {
    env_get_or_default_with(env, "SONGBIRD_CONFIG_DIR", {
        // Use platform-appropriate config directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Roaming\\Songbird\\config",
                env_get_or_default_with(env, "USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!("{}/.config/songbird", env_get_or_default_with(env, "HOME", "/tmp".to_string()),)
        }
    })
}

/// Get temporary directory from environment or use system default
#[must_use]
pub fn get_temp_dir() -> String {
    get_temp_dir_with(&read_process_env)
}

/// Same as [`get_temp_dir`](get_temp_dir) with an injectable env reader.
#[must_use]
pub fn get_temp_dir_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> String {
    env_get_or_default_with(
        env,
        "SONGBIRD_TEMP_DIR",
        std::env::temp_dir().to_string_lossy().to_string(),
    )
}

// ==================== LOGGING CONFIGURATION ====================

/// Get log level from environment or default
#[must_use]
pub fn get_log_level() -> String {
    get_log_level_with(&read_process_env)
}

/// Same as [`get_log_level`](get_log_level) with an injectable env reader.
#[must_use]
pub fn get_log_level_with(env: &impl Fn(&str) -> Result<String, std::env::VarError>) -> String {
    env("SONGBIRD_LOG_LEVEL")
        .or_else(|_| env("LOG_LEVEL"))
        .or_else(|_| env("RUST_LOG"))
        .unwrap_or_else(|_| {
            match env("SONGBIRD_ENV").as_deref() {
                Ok("production") => "warn".to_string(),
                Ok("staging") => "info".to_string(),
                _ => "debug".to_string(), // Testing and development default
            }
        })
}

// ==================== ENVIRONMENT CHECKS ====================

/// Check if running in development environment
#[must_use]
pub fn is_development_environment() -> bool {
    let env = SafeEnv::get_or_default("SONGBIRD_ENVIRONMENT", "development");
    env == "development" || env == "dev"
}

/// Check if running in production environment
#[must_use]
pub fn is_production_environment() -> bool {
    let env = SafeEnv::get_or_default("SONGBIRD_ENVIRONMENT", "development");
    env == "production" || env == "prod"
}

// ==================== CORS CONFIGURATION ====================

fn is_production_environment_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> bool {
    let e = env_or_default_with(env, "SONGBIRD_ENVIRONMENT", "development");
    e == "production" || e == "prod"
}

/// Get canonical CORS origins based on environment
///
/// SOVEREIGNTY: No hardcoded origins. All must be explicitly configured.
#[must_use]
pub fn get_canonical_cors_origins() -> Vec<String> {
    get_canonical_cors_origins_with(&read_process_env)
}

/// Same as [`get_canonical_cors_origins`](get_canonical_cors_origins) with an injectable env reader.
#[must_use]
pub fn get_canonical_cors_origins_with(
    env: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> Vec<String> {
    match env("SONGBIRD_CORS_ORIGINS") {
        Ok(origins) => origins.split(',').map(|s| s.trim().to_string()).collect(),
        Err(_) => {
            if is_production_environment_with(env) {
                // Production: No defaults - fail secure
                tracing::warn!(
                    "SONGBIRD_CORS_ORIGINS not set in production. CORS will deny all origins. \
                     Set SONGBIRD_CORS_ORIGINS to comma-separated list of allowed origins."
                );
                Vec::new() // Empty = deny all (secure default)
            } else {
                // Development: Calculate from bind address
                let bind_addr = get_bind_address_with(env);
                let default_ports = [3000, 8080, 8081];

                default_ports
                    .iter()
                    .flat_map(|port| {
                        vec![
                            format!("http://{}:{}", bind_addr, port),
                            format!("http://localhost:{}", port),
                        ]
                    })
                    .collect()
            }
        }
    }
}

// ==================== PROTOCOL CONFIGURATION ====================

/// Get protocol port mappings for gaming network
#[must_use]
pub fn protocol_port_mappings() -> std::collections::HashMap<String, u16> {
    let mut mappings = std::collections::HashMap::new();
    mappings.insert(
        "udp".to_string(),
        SafeEnv::get_port("SONGBIRD_PROTOCOL_UDP_PORT", FALLBACK_PROTOCOL_UDP_PORT),
    );
    mappings.insert(
        "tcp".to_string(),
        SafeEnv::get_port("SONGBIRD_PROTOCOL_TCP_PORT", FALLBACK_PROTOCOL_TCP_PORT),
    );
    mappings.insert(
        "websocket".to_string(),
        SafeEnv::get_port("SONGBIRD_PROTOCOL_WEBSOCKET_PORT", FALLBACK_PROTOCOL_WEBSOCKET_PORT),
    );
    mappings.insert(
        "secure_websocket".to_string(),
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
    SafeEnv::get_or_default("SONGBIRD_SUBNET", "10.0.0.0/24".to_string())
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

// ==================== NETWORK CONSTANTS ====================

/// Network-related constants
///
/// SOVEREIGNTY EVOLUTION: Hardcoded values removed. Use functions instead.
pub mod network {
    use super::{SafeEnv, get_bind_address};
    use std::time::Duration;

    /// Get default host for current environment
    #[must_use]
    pub fn default_host() -> String {
        get_bind_address()
    }

    /// Get default orchestrator port
    #[must_use]
    pub fn default_orchestrator_port() -> u16 {
        SafeEnv::get_port("SONGBIRD_ORCHESTRATOR_PORT", super::get_port_range_start())
    }

    /// Get default dashboard port
    #[must_use]
    pub fn default_dashboard_port() -> u16 {
        super::get_dashboard_port()
    }

    /// Default retry delay (kept as const - no sovereignty issue)
    pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_millis(1000);

    // DEPRECATED MIGRATION HELPERS (will be removed in v0.3.0)
    // These return function calls for backwards compatibility

    /// DEPRECATED: Use `network::default_host()` instead
    #[deprecated(since = "0.2.0", note = "Use network::default_host() function instead")]
    pub const DEFAULT_HOST: &str = "localhost";

    /// DEPRECATED: Use `get_bind_address()` instead  
    #[deprecated(since = "0.2.0", note = "Use get_bind_address() function instead")]
    pub const DEFAULT_HOST_V4: &str = "127.0.0.1";
}

// ==================== HEALTH CHECK CONSTANTS ====================

/// Health check related constants
pub mod health {
    use std::time::Duration;

    /// Default health check interval
    pub const DEFAULT_CHECK_INTERVAL: Duration = Duration::from_secs(30);

    /// Default health check timeout
    pub const DEFAULT_CHECK_TIMEOUT: Duration = Duration::from_secs(5);
}

// ==================== RESOURCE MANAGEMENT CONSTANTS ====================

/// Resource management related constants
pub mod resources {
    use std::time::Duration;

    /// Default resource cleanup interval
    pub const DEFAULT_CLEANUP_INTERVAL: Duration = Duration::from_secs(300);

    /// Default resource timeout
    pub const DEFAULT_RESOURCE_TIMEOUT: Duration = Duration::from_secs(60);

    /// Default max memory usage percentage
    pub const DEFAULT_MAX_MEMORY_USAGE: f64 = 0.8;

    /// Default max CPU usage percentage
    pub const DEFAULT_MAX_CPU_USAGE: f64 = 0.7;

    /// Default leak detection interval
    pub const DEFAULT_LEAK_DETECTION_INTERVAL: Duration = Duration::from_secs(600);

    /// Default max resource age
    pub const DEFAULT_MAX_RESOURCE_AGE: Duration = Duration::from_secs(3600);

    /// Default monitoring interval
    pub const DEFAULT_MONITORING_INTERVAL: Duration = Duration::from_secs(60);

    /// Default tracking interval
    pub const DEFAULT_TRACKING_INTERVAL: Duration = Duration::from_secs(10);
}

// ==================== SERVICE CONSTANTS ====================

/// Service related constants
pub mod services {
    use std::time::Duration;

    /// Default shutdown timeout
    pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default startup timeout
    pub const DEFAULT_STARTUP_TIMEOUT: Duration = Duration::from_secs(60);

    /// Default service check interval
    pub const DEFAULT_SERVICE_CHECK_INTERVAL: Duration = Duration::from_secs(15);
}

// ==================== NETWORK DEFAULTS STRUCT ====================

/// Environment-aware network configuration
pub struct CanonicalNetworkDefaults;

impl CanonicalNetworkDefaults {
    /// Get bind address as `IpAddr`
    #[must_use]
    pub fn bind_address() -> IpAddr {
        get_canonical_bind_address().parse().unwrap_or_else(|_| {
            if is_production_environment() {
                IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
            } else {
                IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)
            }
        })
    }

    /// Get allowed networks for security
    #[must_use]
    pub fn allowed_networks() -> Vec<String> {
        if is_production_environment() {
            SafeEnv::get_required("SONGBIRD_ALLOWED_NETWORKS").map_or_else(
                |_| {
                    vec![
                        "10.0.0.0/8".to_string(),     // Private networks
                        "172.16.0.0/12".to_string(),  // Private networks
                        "192.168.0.0/16".to_string(), // Private networks
                    ]
                },
                |nets| nets.split(',').map(String::from).collect(),
            )
        } else {
            vec![
                "127.0.0.0/8".to_string(), // Localhost only for development
                "10.0.0.0/8".to_string(),  // Local development networks
            ]
        }
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests;
