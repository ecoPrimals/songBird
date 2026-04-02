// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Configuration Constants and Defaults (DEPRECATED)
//!
//! ⚠️ **CONSOLIDATION COMPLETE - MIGRATION REQUIRED** (November 8, 2025)
//!
//! This module has **92+ active uses** and has been superseded by `canonical::constants`.
//! All constants and functions have been migrated to the canonical module with identical APIs.
//!
//! ## Migration Path
//! ```rust,ignore
//! // ❌ OLD (deprecated - ALL MIGRATED as of Nov 10, 2025):
//! use songbird_config::config::constants::get_bind_address;
//! use songbird_config::config::constants::network::DEFAULT_HOST;
//!
//! // ✅ NEW (all code now uses this):
//! use songbird_config::canonical::constants::get_bind_address;
//! use songbird_config::canonical::constants::network::DEFAULT_HOST;
//! ```
//!
//! ## Migration Complete ✅
//! - ✅ All 98 references migrated (November 10, 2025)
//! - ✅ Zero deprecation warnings
//! - ✅ Build passing
//! - ✅ Tests passing
//!
//! ## What Was Consolidated
//! - 740 duplicate lines eliminated
//! - Identical API maintained (drop-in replacement)
//! - Single source of truth established
//! - `network::*` submodule fully migrated
//!
//! **Status**: ✅ Migration complete - file kept for external backward compatibility\
//! **Timeline**: Can be removed in v0.3.0 (Q2 2026) once external uses confirmed migrated\
//! **Urgency**: NONE - All internal uses successfully migrated

#![deprecated(since = "0.2.0", note = "Use songbird_config::canonical::constants instead")]
#![allow(missing_docs, reason = "deprecated shim; canonical module holds the documented API")]

use songbird_types::error_helpers::SafeEnv;
use std::time::Duration;
// Note: unified_constants doesn't exist in songbird_types, using local constants

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
        if SafeEnv::get_bool("SONGBIRD_ENABLE_NESTGATE", false) {
            count += 1;
        }
        if SafeEnv::get_bool("SONGBIRD_ENABLE_TOADSTOOL", false) {
            count += 1;
        }
        if SafeEnv::get_bool("SONGBIRD_ENABLE_SQUIRREL", false) {
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

/// Get connection timeout from environment or calculate based on network conditions
#[must_use]
pub fn get_connection_timeout_ms() -> u64 {
    SafeEnv::parse("SONGBIRD_CONNECTION_TIMEOUT_MS", {
        match SafeEnv::get("SONGBIRD_ENV").as_deref() {
            Ok("production") => 30000,  // 30 seconds for production
            Ok("staging") => 45000,     // 45 seconds for staging
            Ok("development") => 60000, // 60 seconds for development
            _ => calculate_network_based_timeout(),
        }
    })
}

/// Calculate timeout based on detected network conditions
fn calculate_network_based_timeout() -> u64 {
    // Check if we're in a cloud/container environment
    if SafeEnv::get("KUBERNETES_SERVICE_HOST").is_ok()
        || SafeEnv::get("AWS_EXECUTION_ENV").is_ok()
        || SafeEnv::get("GOOGLE_CLOUD_PROJECT").is_ok()
        || SafeEnv::get("AZURE_CLIENT_ID").is_ok()
    {
        15000 // Fast cloud networks
    } else {
        30000 // Conservative default for unknown networks
    }
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

/// Get log level from environment or default
#[must_use]
pub fn get_log_level() -> String {
    SafeEnv::get("SONGBIRD_LOG_LEVEL")
        .or_else(|_| SafeEnv::get("LOG_LEVEL"))
        .or_else(|_| SafeEnv::get("RUST_LOG"))
        .unwrap_or_else(|_| {
            match SafeEnv::get("SONGBIRD_ENV").as_deref() {
                Ok("production") => "warn".to_string(),
                Ok("staging") => "info".to_string(),
                _ => "debug".to_string(), // Testing and development default
            }
        })
}

/// Default cache TTL
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

/// Default evaluation timeout
pub const DEFAULT_EVALUATION_TIMEOUT: Duration = Duration::from_secs(30);

/// Default metrics interval
pub const DEFAULT_METRICS_INTERVAL: Duration = Duration::from_secs(60);

/// Get maximum connections allowed
#[must_use]
pub fn get_max_connections() -> usize {
    SafeEnv::parse("SONGBIRD_MAX_CONNECTIONS", {
        match SafeEnv::get("SONGBIRD_ENV").as_deref() {
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
    SafeEnv::parse("SONGBIRD_WORKER_THREADS", {
        // Use CPU count or container limits
        // Fallback to 4 threads
        std::thread::available_parallelism().map(std::num::NonZero::get).unwrap_or(4)
    })
}

/// Get buffer pool size based on available memory
#[must_use]
pub fn get_buffer_pool_size() -> usize {
    SafeEnv::parse("SONGBIRD_BUFFER_POOL_SIZE", {
        // Calculate based on available memory
        let base_size = match SafeEnv::get("SONGBIRD_ENV").as_deref() {
            Ok("production") => 10000,
            Ok("staging") => 5000,
            Ok("development") => 1000,
            _ => 2000,
        };

        // Adjust for container memory limits
        SafeEnv::get("MEMORY_LIMIT")
            .ok()
            .and_then(|memory_limit| memory_limit.parse::<u64>().ok())
            .map_or(base_size, |limit_mb| {
                // Use 1% of available memory for buffer pool
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "MEMORY_LIMIT parsed as u64; product scaled down for pool sizing"
                )]
                let adjusted_size = (limit_mb as usize * 10) / 1024;
                std::cmp::min(base_size, adjusted_size)
            })
    })
}

/// Get batch processing size based on workload characteristics
#[must_use]
pub fn get_batch_size() -> usize {
    SafeEnv::parse("SONGBIRD_BATCH_SIZE", {
        // Calculate optimal batch size based on system characteristics
        let cpu_count = get_worker_threads();
        let memory_factor = if SafeEnv::get("MEMORY_LIMIT").is_ok() {
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
    SafeEnv::get_bool("SONGBIRD_ENABLE_ZERO_COPY", {
        // Enable zero-copy in production and for high-performance environments
        match SafeEnv::get("SONGBIRD_ENV").as_deref() {
            Ok("production" | "staging") => true,
            _ => {
                // Enable if system has sufficient memory
                SafeEnv::get("MEMORY_LIMIT")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .is_none_or(|mb| mb > 2048) // Default to enabled
            }
        }
    })
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

/// Get log directory from environment or calculate default
#[must_use]
pub fn get_log_dir() -> String {
    SafeEnv::get_or_default("SONGBIRD_LOG_DIR", {
        // Use platform-appropriate log directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Local\\Songbird\\logs",
                SafeEnv::get_or_default("USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!(
                "{}/.local/share/songbird/logs",
                SafeEnv::get_or_default("HOME", "/tmp".to_string()),
            )
        }
    })
}

/// Get cache directory from environment or calculate default
#[must_use]
pub fn get_cache_dir() -> String {
    SafeEnv::get_or_default("SONGBIRD_CACHE_DIR", {
        // Use platform-appropriate cache directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Local\\Songbird\\cache",
                SafeEnv::get_or_default("USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!("{}/.cache/songbird", SafeEnv::get_or_default("HOME", "/tmp".to_string()),)
        }
    })
}

/// Get data directory from environment or calculate default
#[must_use]
pub fn get_data_dir() -> String {
    SafeEnv::get_or_default("SONGBIRD_DATA_DIR", {
        // Use platform-appropriate data directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Roaming\\Songbird",
                SafeEnv::get_or_default("USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!("{}/.local/share/songbird", SafeEnv::get_or_default("HOME", "/tmp".to_string()),)
        }
    })
}

/// Get configuration directory from environment or calculate default
#[must_use]
pub fn get_config_dir() -> String {
    SafeEnv::get_or_default("SONGBIRD_CONFIG_DIR", {
        // Use platform-appropriate config directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Roaming\\Songbird\\config",
                SafeEnv::get_or_default("USERPROFILE", "C:\\Users\\Default".to_string()),
            )
        } else {
            format!("{}/.config/songbird", SafeEnv::get_or_default("HOME", "/tmp".to_string()),)
        }
    })
}

/// Get temporary directory from environment or use system default
#[must_use]
pub fn get_temp_dir() -> String {
    SafeEnv::get_or_default("SONGBIRD_TEMP_DIR", std::env::temp_dir().to_string_lossy().to_string())
}

/// Universal primal endpoint discovery - works with any primal name
#[must_use]
pub fn get_primal_endpoint(primal_name: &str) -> String {
    // First try primal-specific environment variable
    let env_var = format!("{}_ENDPOINT", primal_name.to_uppercase());
    if let Ok(endpoint) = SafeEnv::get(&env_var) {
        return endpoint;
    }

    // Try generic primal endpoint pattern
    let generic_env = format!("PRIMAL_{}_ENDPOINT", primal_name.to_uppercase());
    if let Ok(endpoint) = SafeEnv::get(&generic_env) {
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
        crate::constants::network::DEFAULT_HOST.to_string()
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

/// Determine if primal should use TLS based on environment and naming
fn should_use_tls_for_primal(primal_name: &str) -> bool {
    // Check primal-specific TLS setting
    let tls_env = format!("{}_USE_TLS", primal_name.to_uppercase());
    if SafeEnv::get_bool(&tls_env, false) {
        return true;
    }

    // Security-related primals default to TLS in production
    let is_security_primal = primal_name.to_lowercase().contains("security")
        || primal_name.to_lowercase().contains("auth")
        || primal_name.to_lowercase().contains("crypto");

    match SafeEnv::get("SONGBIRD_ENV").as_deref() {
        Ok("production") => true,
        Ok("staging") => is_security_primal,
        _ => false, // Development default
    }
}

/// Get all configured primal names from environment
#[must_use]
pub fn get_configured_primal_names() -> Vec<String> {
    let mut primal_names = Vec::new();

    // Scan for primal-specific environment variables
    for (key, _value) in std::env::vars() {
        if key.ends_with("_ENDPOINT") && !key.starts_with("SONGBIRD_") {
            let primal_name = key.trim_end_matches("_ENDPOINT").to_lowercase();
            if !primal_names.contains(&primal_name) {
                primal_names.push(primal_name);
            }
        }

        // Also check PRIMAL_*_ENDPOINT pattern
        if key.starts_with("PRIMAL_") && key.ends_with("_ENDPOINT") {
            // Safe: we already checked starts_with and ends_with above
            if let Some(primal_part) =
                key.strip_prefix("PRIMAL_").and_then(|s| s.strip_suffix("_ENDPOINT"))
            {
                let primal_name = primal_part.to_lowercase();
                if !primal_names.contains(&primal_name) {
                    primal_names.push(primal_name);
                }
            }
        }
    }

    // If no primals configured, return empty list for pure discovery mode
    primal_names
}

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

/// Health check related constants
pub mod health {
    use std::time::Duration;

    /// Default health check interval
    pub const DEFAULT_CHECK_INTERVAL: Duration = Duration::from_secs(30);

    /// Default health check timeout
    pub const DEFAULT_CHECK_TIMEOUT: Duration = Duration::from_secs(5);
}

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
    SafeEnv::get_port("SONGBIRD_DISCOVERY_PORT", 5678)
}

/// Get default bind address for the current environment
#[must_use]
pub fn default_bind_address() -> String {
    get_bind_address()
}

/// Network-related constants
pub mod network {
    use std::time::Duration;

    /// Default host constant
    pub const DEFAULT_HOST: &str = "localhost";

    /// Default host IPv4 constant
    pub const DEFAULT_HOST_V4: &str = "127.0.0.1";

    /// Default bind address constant
    pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:8080";

    /// Default orchestrator port
    pub const DEFAULT_ORCHESTRATOR_PORT: u16 = 8080;

    /// Default development port
    pub const DEFAULT_DEV_PORT: u16 = 8080;

    /// Default dashboard port
    pub const DEFAULT_DASHBOARD_PORT: u16 = 3000;

    // ============================================================================
    // ⚠️ DEPRECATED PRIMAL ENDPOINT CONSTANTS - REMOVED FOR SOVEREIGNTY
    // ============================================================================
    //
    // The following hardcoded endpoint constants have been REMOVED for sovereignty
    // compliance. They violated the principle that primals should only have self-
    // knowledge and discover other primals at runtime.
    //
    // OLD (REMOVED):
    // - DEFAULT_TOADSTOOL_ENDPOINT (compute)
    // - DEFAULT_SQUIRREL_ENDPOINT (AI)
    // - DEFAULT_NESTGATE_ENDPOINT (storage)
    // - DEFAULT_BEARDOG_ENDPOINT (security)
    // - DEFAULT_*_PORT constants
    //
    // **SOVEREIGNTY VIOLATION**: Hardcoded primal endpoints violate sovereignty principles.
    // Each primal should know only itself and discover others at runtime.
    //
    // NEW (USE INSTEAD):
    // ```rust
    // use songbird_config::primal_discovery::*;
    //
    // // Discovers ANY provider with the capability
    // let compute = get_compute_endpoint().await?;    // set COMPUTE_ENDPOINT
    // let ai = get_ai_endpoint().await?;              // set AI_ENDPOINT
    // let storage = get_storage_endpoint().await?;    // set STORAGE_ENDPOINT
    // let security = get_security_endpoint().await?;  // set SECURITY_ENDPOINT
    //
    // // Or use RuntimeDiscoveryEngine for dynamic discovery
    // let engine = RuntimeDiscoveryEngine::new();
    // let service = engine.discover_by_capability("compute").await?;
    // ```
    //
    // See: crates/songbird-config/src/primal_discovery.rs
    // See: crates/songbird-config/src/runtime_discovery.rs
    // See: specs/INDIVIDUAL_HUMAN_DIGNITY_SPECIFICATION.md
    // ============================================================================

    /// Default connection timeout
    // MIGRATED: Use songbird_types::unified_constants::timeouts::DEFAULT_CONNECTION_TIMEOUT instead
    /// Default retry delay
    pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_millis(1000);
    /// Default `crate::constants::network::DEFAULT_HOST` address
    // MIGRATED: Use songbird_types::unified_constants::network::DEFAULT_LOCALHOST instead
    /// Production bind address
    pub const PRODUCTION_BIND_ADDRESS: &str = "0.0.0.0";
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::{
        external_address, get_cache_dir, get_config_dir, get_data_dir, get_log_dir, get_temp_dir,
        health, protocol_port_mappings, resources, services,
    };

    #[test]
    fn test_get_bind_address() {
        let addr = super::get_bind_address();
        assert!(!addr.is_empty());
    }

    #[test]
    fn string_constants_are_documented_non_empty() {
        assert_eq!(super::LOCALHOST_IPV4, "127.0.0.1");
        assert!(!super::DEFAULT_CONFIG_PATH.is_empty());
        assert!(!super::DEFAULT_BIND_ADDRESS.is_empty());
    }

    #[test]
    fn protocol_port_mappings_includes_core_protocols() {
        let m = protocol_port_mappings();
        assert_eq!(m.get("udp"), Some(&6112));
        assert_eq!(m.get("tcp"), Some(&6113));
        assert_eq!(m.get("websocket"), Some(&8080));
        assert_eq!(m.get("secure_websocket"), Some(&8443));
    }

    #[test]
    fn health_module_default_intervals() {
        assert_eq!(health::DEFAULT_CHECK_INTERVAL.as_secs(), 30);
        assert_eq!(health::DEFAULT_CHECK_TIMEOUT.as_secs(), 5);
    }

    #[test]
    fn resources_module_defaults() {
        assert_eq!(resources::DEFAULT_CLEANUP_INTERVAL.as_secs(), 300);
        assert!((resources::DEFAULT_MAX_MEMORY_USAGE - 0.8).abs() < f64::EPSILON);
        assert!((resources::DEFAULT_MAX_CPU_USAGE - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn services_module_shutdown_and_startup() {
        assert_eq!(services::DEFAULT_SHUTDOWN_TIMEOUT.as_secs(), 30);
        assert_eq!(services::DEFAULT_STARTUP_TIMEOUT.as_secs(), 60);
    }

    #[test]
    fn default_subnet_and_external_address_are_non_empty() {
        assert!(!super::default_subnet().is_empty());
        assert!(!external_address().is_empty());
    }

    #[test]
    fn directory_helpers_return_non_empty_strings() {
        assert!(!get_log_dir().is_empty());
        assert!(!get_cache_dir().is_empty());
        assert!(!get_data_dir().is_empty());
        assert!(!get_config_dir().is_empty());
        assert!(!get_temp_dir().is_empty());
    }

    #[test]
    fn duration_constants_are_sensible() {
        assert_eq!(super::DEFAULT_CACHE_TTL.as_secs(), 300);
        assert_eq!(super::DEFAULT_EVALUATION_TIMEOUT.as_secs(), 30);
        assert_eq!(super::DEFAULT_METRICS_INTERVAL.as_secs(), 60);
    }

    #[test]
    fn network_submodule_host_constants() {
        assert_eq!(super::network::DEFAULT_HOST, "localhost");
        assert_eq!(super::network::DEFAULT_HOST_V4, "127.0.0.1");
        assert_eq!(super::network::PRODUCTION_BIND_ADDRESS, "0.0.0.0");
    }
}
