//! # 🎯 Canonical Constants - Environment-Aware Defaults
//!
//! **ZERO HARDCODING SYSTEM**
//!
//! This module provides environment-aware defaults that eliminate hardcoded values
//! while maintaining secure defaults for development and production.
//!
//! **Status**: Phase 4 consolidation complete - merged from config/constants.rs
//! **Philosophy**: All values configurable via environment, calculated defaults for production

use songbird_types::error_helpers::SafeEnv;
use std::net::IpAddr;
use std::time::Duration;

/// Default configuration file path
pub const DEFAULT_CONFIG_PATH: &str = "songbird.toml";

// NOTE: Removed hardcoded network constants per sovereignty principles
// Tests should set their own environment variables instead of relying on these constants
// For migration: Use get_bind_address() which is environment-aware

// ==================== NETWORK CONFIGURATION ====================

/// Get bind address from environment or calculate from system capabilities
#[must_use]
pub fn get_bind_address() -> String {
    // Try to get from environment, but validate it
    if let Ok(addr) = SafeEnv::get("SONGBIRD_BIND_ADDRESS") {
        // Validate that it's a valid IP address
        if addr.parse::<IpAddr>().is_ok() {
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

/// Get default discovery port
#[must_use]
pub fn default_discovery_port() -> u16 {
    SafeEnv::get_port("SONGBIRD_DISCOVERY_PORT", 5678)
}

// ==================== TIMEOUT CONFIGURATION ====================

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
                #[allow(clippy::cast_possible_truncation)]
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

// ==================== PRIMAL CONFIGURATION ====================

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

/// Get port offset for specific primal types (legacy, prefer `calculate_primal_port_offset`)
fn get_primal_port_offset(primal_type: &str) -> u16 {
    match primal_type.to_lowercase().as_str() {
        "beardog" => 10,
        "nestgate" => 20,
        "toadstool" => 30,
        "squirrel" => 40,
        "discovery" => 50,
        "health" => 60,
        "metrics" => 70,
        "dashboard" => 80,
        _ => calculate_primal_port_offset(primal_type), // Fallback to hash-based
    }
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

/// Get common primal service ports from environment
#[must_use]
pub fn get_common_primal_ports() -> Vec<u16> {
    SafeEnv::get_or_default("SONGBIRD_COMMON_PORTS", {
        // Build dynamic port list based on enabled services
        let mut ports = Vec::new();
        let base_port = get_port_range_start();

        // Add ports for enabled services
        ports.push(base_port); // Main service

        if SafeEnv::get_bool("SONGBIRD_ENABLE_BEARDOG", false) {
            ports.push(base_port + get_primal_port_offset("beardog"));
        }
        if SafeEnv::get_bool("SONGBIRD_ENABLE_NESTGATE", false) {
            ports.push(base_port + get_primal_port_offset("nestgate"));
        }
        if SafeEnv::get_bool("SONGBIRD_ENABLE_TOADSTOOL", false) {
            ports.push(base_port + get_primal_port_offset("toadstool"));
        }

        ports.into_iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",")
    })
    .split(',')
    .filter_map(|s| s.trim().parse().ok())
    .collect()
}

/// Universal capability query - works with any capability name
#[must_use]
pub fn find_primals_with_capability(_capability: &str) -> Vec<String> {
    // This would integrate with the capability discovery system
    // For now, return configured primals (will be enhanced with actual capability detection)
    get_configured_primal_names()
}

// ==================== ENDPOINT CONFIGURATION ====================

/// Get canonical endpoint URL based on environment and service
#[must_use]
pub fn get_canonical_endpoint(service_name: &str, default_port: u16) -> String {
    let base_url = match SafeEnv::get_or_default("SONGBIRD_ENVIRONMENT", "development").as_str() {
        "production" | "prod" => SafeEnv::get_or_default(
            "SONGBIRD_BASE_URL",
            format!("https://{}:8443", get_canonical_bind_address()),
        ),
        "staging" => SafeEnv::get_or_default("SONGBIRD_BASE_URL", "http://staging.internal:8080"),
        _ => {
            SafeEnv::get_or_default("SONGBIRD_BASE_URL", format!("http://127.0.0.1:{default_port}"))
        }
    };

    // Service-specific endpoint override
    SafeEnv::get_or_default(&format!("SONGBIRD_{}_ENDPOINT", service_name.to_uppercase()), base_url)
}

/// Get canonical discovery endpoint
#[must_use]
pub fn get_canonical_discovery_endpoint() -> String {
    get_canonical_endpoint("discovery", 8081)
}

/// Get canonical security endpoint
#[must_use]
pub fn get_canonical_security_endpoint() -> String {
    get_canonical_endpoint("security", 8443)
}

/// Get canonical orchestrator endpoint
#[must_use]
pub fn get_canonical_orchestrator_endpoint() -> String {
    get_canonical_endpoint("orchestrator", 8080)
}

/// Get canonical gaming endpoint
#[must_use]
pub fn get_canonical_gaming_endpoint() -> String {
    get_canonical_endpoint("gaming", 6112)
}

// ==================== DIRECTORY CONFIGURATION ====================

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

// ==================== LOGGING CONFIGURATION ====================

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

/// Get canonical CORS origins based on environment
///
/// SOVEREIGNTY: No hardcoded origins. All must be explicitly configured.
#[must_use]
pub fn get_canonical_cors_origins() -> Vec<String> {
    // Always require explicit CORS configuration for security
    SafeEnv::get("SONGBIRD_CORS_ORIGINS").map_or_else(
        |_| {
            if is_production_environment() {
                // Production: No defaults - fail secure
                tracing::warn!(
                    "SONGBIRD_CORS_ORIGINS not set in production. CORS will deny all origins. \
                     Set SONGBIRD_CORS_ORIGINS to comma-separated list of allowed origins."
                );
                Vec::new() // Empty = deny all (secure default)
            } else {
                // Development: Calculate from bind address
                let bind_addr = get_bind_address();
                let default_ports = vec![3000, 8080, 8081];

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
        },
        |origins| origins.split(',').map(|s| s.trim().to_string()).collect(),
    )
}

// ==================== PROTOCOL CONFIGURATION ====================

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
    use super::*;
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

    /// DEPRECATED: Use network::default_host() instead
    #[deprecated(since = "0.2.0", note = "Use network::default_host() function instead")]
    pub const DEFAULT_HOST: &str = "localhost";

    /// DEPRECATED: Use get_bind_address() instead  
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

// ==================== TESTS ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bind_address() {
        let addr = get_bind_address();
        assert!(!addr.is_empty());
        // Should parse as valid IP
        assert!(addr.parse::<IpAddr>().is_ok());
    }

    #[test]
    fn test_port_range() {
        let start = get_port_range_start();
        let end = get_port_range_end();
        assert!(start > 0);
        assert!(end > start);
        // Port end is u16, so it's always <= 65535 by type constraint
        // Verify it's a reasonable value
        assert!(end >= 1024, "Port range end should be >= 1024");
    }

    #[test]
    fn test_environment_detection() {
        // Test functions don't panic
        let _ = is_development_environment();
        let _ = is_production_environment();
    }

    #[test]
    fn test_primal_endpoint_generation() {
        let endpoint = get_primal_endpoint("test_primal");
        assert!(endpoint.starts_with("http://") || endpoint.starts_with("https://"));
    }

    #[test]
    fn test_directory_configuration() {
        let log_dir = get_log_dir();
        let cache_dir = get_cache_dir();
        let data_dir = get_data_dir();
        let config_dir = get_config_dir();

        assert!(!log_dir.is_empty());
        assert!(!cache_dir.is_empty());
        assert!(!data_dir.is_empty());
        assert!(!config_dir.is_empty());
    }
}
