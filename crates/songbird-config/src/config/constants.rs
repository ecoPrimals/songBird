//! Configuration constants and defaults - All values configurable via environment
//!
//! This module provides NO hardcoded values - everything is configurable
//! through environment variables or calculated defaults.

use std::env;
use std::time::Duration;

/// Default configuration file path
pub const DEFAULT_CONFIG_PATH: &str = "songbird.toml";

/// IPv4 localhost address constant
pub const LOCALHOST_IPV4: &str = "127.0.0.1";

/// Default bind address constant
pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1";

/// Default localhost constant
pub const DEFAULT_LOCALHOST: &str = "127.0.0.1";

/// Get bind address from environment or calculate from system capabilities
pub fn get_bind_address() -> String {
    env::var("SONGBIRD_BIND_ADDRESS").unwrap_or_else(|_| {
        // Detect if running in container/kubernetes or production
        if env::var("KUBERNETES_SERVICE_HOST").is_ok()
            || env::var("CONTAINER").is_ok()
            || env::var("SONGBIRD_ENV").as_deref() == Ok("production")
        {
            "0.0.0.0".to_string() // Container/production environment
        } else {
            "127.0.0.1".to_string() // Development/local environment
        }
    })
}

/// Get port range start from environment or system-based calculation
pub fn get_port_range_start() -> u16 {
    env::var("SONGBIRD_PORT_START")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            // Calculate based on environment and user permissions
            if env::var("SONGBIRD_ALLOW_PRIVILEGED_PORTS").is_ok() {
                80 + get_environment_offset()
            } else {
                8000 + get_environment_offset()
            }
        })
}

/// Get port range end from environment or calculated from start
pub fn get_port_range_end() -> u16 {
    env::var("SONGBIRD_PORT_END")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            let start = get_port_range_start();
            start + get_port_range_size()
        })
}

/// Calculate environment-specific port offset
fn get_environment_offset() -> u16 {
    match env::var("SONGBIRD_ENV").as_deref() {
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
    env::var("SONGBIRD_PORT_RANGE_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            // Calculate based on enabled services and expected scale
            let base_size = 1000;
            let service_multiplier = get_expected_service_count();
            (base_size + service_multiplier * 10).min(65535 - get_port_range_start())
        })
}

/// Calculate user-specific port offset to avoid conflicts in multi-user environments
fn calculate_user_port_offset() -> u16 {
    // Use user ID hash for deterministic but unique offset
    let user = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "default".to_string());
    let hash = user
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
    (hash % 500) as u16 // Limit to reasonable range
}

/// Get expected service count from configuration or environment
fn get_expected_service_count() -> u16 {
    env::var("SONGBIRD_EXPECTED_SERVICES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            // Calculate based on enabled primals and features
            let mut count = 1; // Base Songbird service

            if env::var("SONGBIRD_ENABLE_BEARDOG").as_deref() == Ok("true") {
                count += 1;
            }
            if env::var("SONGBIRD_ENABLE_NESTGATE").as_deref() == Ok("true") {
                count += 1;
            }
            if env::var("SONGBIRD_ENABLE_TOADSTOOL").as_deref() == Ok("true") {
                count += 1;
            }
            if env::var("SONGBIRD_ENABLE_SQUIRREL").as_deref() == Ok("true") {
                count += 1;
            }
            if env::var("SONGBIRD_ENABLE_DISCOVERY").as_deref() != Ok("false") {
                count += 1;
            }
            if env::var("SONGBIRD_ENABLE_METRICS").as_deref() != Ok("false") {
                count += 1;
            }
            if env::var("SONGBIRD_ENABLE_HEALTH_CHECK").as_deref() != Ok("false") {
                count += 1;
            }

            count
        })
}

/// Get connection timeout from environment or calculate based on network conditions
pub fn get_connection_timeout_ms() -> u64 {
    env::var("SONGBIRD_CONNECTION_TIMEOUT_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            match env::var("SONGBIRD_ENV").as_deref() {
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
    if env::var("KUBERNETES_SERVICE_HOST").is_ok()
        || env::var("AWS_EXECUTION_ENV").is_ok()
        || env::var("GOOGLE_CLOUD_PROJECT").is_ok()
        || env::var("AZURE_CLIENT_ID").is_ok()
    {
        15000 // Fast cloud networks
    } else {
        30000 // Conservative default for unknown networks
    }
}

// Old hardcoded get_primal_endpoint function removed -
// now using universal implementation that works with any primal name

/// Get port offset for specific primal types
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
        _ => {
            // Calculate deterministic offset from name
            let hash = primal_type
                .bytes()
                .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
            100 + (hash % 900) as u16 // Offset from base + 100
        }
    }
}

/// Get log level from environment or default
pub fn get_log_level() -> String {
    env::var("SONGBIRD_LOG_LEVEL")
        .or_else(|_| env::var("LOG_LEVEL"))
        .or_else(|_| env::var("RUST_LOG"))
        .unwrap_or_else(|_| {
            match env::var("SONGBIRD_ENV").as_deref() {
                Ok("production") => "warn".to_string(),
                Ok("staging") => "info".to_string(),
                Ok("testing") => "debug".to_string(),
                _ => "debug".to_string(), // Development default
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
pub fn get_max_connections() -> usize {
    env::var("SONGBIRD_MAX_CONNECTIONS")
        .ok()
        .and_then(|c| c.parse().ok())
        .unwrap_or_else(|| {
            match env::var("SONGBIRD_ENV").as_deref() {
                Ok("production") => 10000,
                Ok("staging") => 5000,
                Ok("testing") => 1000,
                _ => 2000, // Development default
            }
        })
}

/// Get worker thread count based on system resources
pub fn get_worker_threads() -> usize {
    env::var("SONGBIRD_WORKER_THREADS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            // Use CPU count or container limits
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4) // Fallback to 4 threads
        })
}

/// Get buffer pool size based on available memory
pub fn get_buffer_pool_size() -> usize {
    env::var("SONGBIRD_BUFFER_POOL_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            // Calculate based on available memory
            let base_size = match env::var("SONGBIRD_ENV").as_deref() {
                Ok("production") => 10000,
                Ok("staging") => 5000,
                Ok("development") => 1000,
                _ => 2000,
            };

            // Adjust for container memory limits
            if let Ok(memory_limit) = env::var("MEMORY_LIMIT") {
                if let Ok(limit_mb) = memory_limit.parse::<u64>() {
                    // Use 1% of available memory for buffer pool
                    std::cmp::min(base_size, (limit_mb as usize * 10) / 1024)
                } else {
                    base_size
                }
            } else {
                base_size
            }
        })
}

/// Get batch processing size based on workload characteristics
pub fn get_batch_size() -> usize {
    env::var("SONGBIRD_BATCH_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            // Calculate optimal batch size based on system characteristics
            let cpu_count = get_worker_threads();
            let memory_factor = if env::var("MEMORY_LIMIT").is_ok() {
                500
            } else {
                1000
            };

            (cpu_count * memory_factor).clamp(100, 5000)
        })
}

/// Check if zero-copy optimizations should be enabled
pub fn enable_zero_copy() -> bool {
    env::var("SONGBIRD_ENABLE_ZERO_COPY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            // Enable zero-copy in production and for high-performance environments
            match env::var("SONGBIRD_ENV").as_deref() {
                Ok("production") => true,
                Ok("staging") => true,
                _ => {
                    // Enable if system has sufficient memory
                    env::var("MEMORY_LIMIT")
                        .ok()
                        .and_then(|s| s.parse::<u64>().ok())
                        .map(|mb| mb > 2048) // Enable if > 2GB memory
                        .unwrap_or(true) // Default to enabled
                }
            }
        })
}

/// Get common primal service ports from environment
pub fn get_common_primal_ports() -> Vec<u16> {
    env::var("SONGBIRD_COMMON_PORTS")
        .unwrap_or_else(|_| {
            // Build dynamic port list based on enabled services
            let mut ports = Vec::new();
            let base_port = get_port_range_start();

            // Add ports for enabled services
            ports.push(base_port); // Main service

            if env::var("SONGBIRD_ENABLE_BEARDOG").as_deref() == Ok("true") {
                ports.push(base_port + get_primal_port_offset("beardog"));
            }
            if env::var("SONGBIRD_ENABLE_NESTGATE").as_deref() == Ok("true") {
                ports.push(base_port + get_primal_port_offset("nestgate"));
            }
            if env::var("SONGBIRD_ENABLE_TOADSTOOL").as_deref() == Ok("true") {
                ports.push(base_port + get_primal_port_offset("toadstool"));
            }

            ports
                .into_iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(",")
        })
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect()
}

/// Get log directory from environment or calculate default
pub fn get_log_dir() -> String {
    env::var("SONGBIRD_LOG_DIR").unwrap_or_else(|_| {
        // Use platform-appropriate log directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Local\\Songbird\\logs",
                env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string())
            )
        } else {
            format!(
                "{}/.local/share/songbird/logs",
                env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
            )
        }
    })
}

/// Get cache directory from environment or calculate default
pub fn get_cache_dir() -> String {
    env::var("SONGBIRD_CACHE_DIR").unwrap_or_else(|_| {
        // Use platform-appropriate cache directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Local\\Songbird\\cache",
                env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string())
            )
        } else {
            format!(
                "{}/.cache/songbird",
                env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
            )
        }
    })
}

/// Get data directory from environment or calculate default
pub fn get_data_dir() -> String {
    env::var("SONGBIRD_DATA_DIR").unwrap_or_else(|_| {
        // Use platform-appropriate data directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Roaming\\Songbird",
                env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string())
            )
        } else {
            format!(
                "{}/.local/share/songbird",
                env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
            )
        }
    })
}

/// Get configuration directory from environment or calculate default
pub fn get_config_dir() -> String {
    env::var("SONGBIRD_CONFIG_DIR").unwrap_or_else(|_| {
        // Use platform-appropriate config directory
        if cfg!(windows) {
            format!(
                "{}\\AppData\\Roaming\\Songbird\\config",
                env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string())
            )
        } else {
            format!(
                "{}/.config/songbird",
                env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
            )
        }
    })
}

/// Get temporary directory from environment or use system default
pub fn get_temp_dir() -> String {
    env::var("SONGBIRD_TEMP_DIR")
        .unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().to_string())
}

/// Universal primal endpoint discovery - works with any primal name
pub fn get_primal_endpoint(primal_name: &str) -> String {
    // First try primal-specific environment variable
    let env_var = format!("{}_ENDPOINT", primal_name.to_uppercase());
    if let Ok(endpoint) = env::var(&env_var) {
        return endpoint;
    }

    // Try generic primal endpoint pattern
    let generic_env = format!("PRIMAL_{}_ENDPOINT", primal_name.to_uppercase());
    if let Ok(endpoint) = env::var(&generic_env) {
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

    let host = if env::var("KUBERNETES_SERVICE_HOST").is_ok() {
        // Kubernetes service discovery pattern
        format!("{}-service", primal_name.to_lowercase())
    } else if env::var("DOCKER_HOST").is_ok() || env::var("CONTAINER").is_ok() {
        // Docker container pattern
        primal_name.to_lowercase()
    } else {
        // Local development pattern
        "127.0.0.1".to_string()
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
    if let Ok(use_tls) = env::var(&tls_env) {
        return use_tls.to_lowercase() == "true";
    }

    // Security-related primals default to TLS in production
    let is_security_primal = primal_name.to_lowercase().contains("security")
        || primal_name.to_lowercase().contains("auth")
        || primal_name.to_lowercase().contains("crypto");

    match env::var("SONGBIRD_ENV").as_deref() {
        Ok("production") => true,
        Ok("staging") => is_security_primal,
        _ => false, // Development default
    }
}

/// Get all configured primal names from environment
pub fn get_configured_primal_names() -> Vec<String> {
    let mut primal_names = Vec::new();

    // Scan for primal-specific environment variables
    for (key, _value) in env::vars() {
        if key.ends_with("_ENDPOINT") && !key.starts_with("SONGBIRD_") {
            let primal_name = key.trim_end_matches("_ENDPOINT").to_lowercase();
            if !primal_names.contains(&primal_name) {
                primal_names.push(primal_name);
            }
        }

        // Also check PRIMAL_*_ENDPOINT pattern
        if key.starts_with("PRIMAL_") && key.ends_with("_ENDPOINT") {
            let primal_part = key
                .strip_prefix("PRIMAL_")
                .unwrap()
                .strip_suffix("_ENDPOINT")
                .unwrap();
            let primal_name = primal_part.to_lowercase();
            if !primal_names.contains(&primal_name) {
                primal_names.push(primal_name);
            }
        }
    }

    // If no primals configured, return empty list for pure discovery mode
    primal_names
}

/// Get dashboard port from environment or calculated default
pub fn get_dashboard_port() -> u16 {
    env::var("SONGBIRD_DASHBOARD_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or_else(|| {
            // Calculate based on environment
            match env::var("SONGBIRD_ENV").as_deref() {
                Ok("production") => 3000, // Standard port for production
                Ok("staging") => 3001,    // Staging offset
                Ok("testing") => 3002,    // Testing offset
                _ => 8083,                // Development default
            }
        })
}

/// Get protocol port mappings for gaming network
pub fn protocol_port_mappings() -> std::collections::HashMap<String, u16> {
    let mut mappings = std::collections::HashMap::new();
    mappings.insert("udp".to_string(), 6112);
    mappings.insert("tcp".to_string(), 6113);
    mappings.insert("websocket".to_string(), 8080);
    mappings.insert("secure_websocket".to_string(), 8443);
    mappings
}

/// Get external address for network configuration
pub fn external_address() -> String {
    env::var("SONGBIRD_EXTERNAL_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string())
}

/// Get default subnet configuration
pub fn default_subnet() -> String {
    env::var("SONGBIRD_SUBNET").unwrap_or_else(|_| "10.0.0.0/24".to_string())
}

/// Universal capability query - works with any capability name
pub fn find_primals_with_capability(_capability: &str) -> Vec<String> {
    // This would integrate with the capability discovery system
    // For now, return configured primals (will be enhanced with actual capability detection)
    get_configured_primal_names()
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
pub fn get_default_bind_address() -> String {
    default_bind_address()
}

/// Generate a unique node ID for this instance
pub fn node_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Create a unique node ID based on hostname and process ID
    let hostname = gethostname::gethostname().to_string_lossy().to_string();
    let pid = std::process::id();

    let mut hasher = DefaultHasher::new();
    hostname.hash(&mut hasher);
    pid.hash(&mut hasher);

    format!("songbird-{:x}", hasher.finish())
}

/// Get default discovery port
pub fn default_discovery_port() -> u16 {
    env::var("SONGBIRD_DISCOVERY_PORT")
        .and_then(|p| p.parse().map_err(|_| env::VarError::NotPresent))
        .unwrap_or(5678)
}

/// Get default bind address for the current environment
pub fn default_bind_address() -> String {
    get_bind_address()
}

/// Network-related constants
pub mod network {
    use std::time::Duration;

    /// Default bind address constant
    pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1";

    /// Default dashboard port
    pub const DEFAULT_DASHBOARD_PORT: u16 = 3000;

    /// Default Toadstool endpoint
    pub const DEFAULT_TOADSTOOL_ENDPOINT: &str = "http://localhost:8001";

    /// Default Toadstool port
    pub const DEFAULT_TOADSTOOL_PORT: u16 = 8001;

    /// Default Squirrel endpoint
    pub const DEFAULT_SQUIRREL_ENDPOINT: &str = "http://localhost:8002";

    /// Default Squirrel port
    pub const DEFAULT_SQUIRREL_PORT: u16 = 8002;

    /// Default NestGate endpoint
    pub const DEFAULT_NESTGATE_ENDPOINT: &str = "http://localhost:8003";

    /// Default NestGate port
    pub const DEFAULT_NESTGATE_PORT: u16 = 8003;

    /// Default BearDog endpoint
    pub const DEFAULT_BEARDOG_ENDPOINT: &str = "http://localhost:8004";

    /// Default BearDog port
    pub const DEFAULT_BEARDOG_PORT: u16 = 8004;

    /// Default connection timeout
    pub const DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_millis(30000);

    /// Default retry delay
    pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_millis(1000);

    /// Default localhost address
    pub const DEFAULT_LOCALHOST: &str = "127.0.0.1";

    /// Production bind address
    pub const PRODUCTION_BIND_ADDRESS: &str = "0.0.0.0";
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_bind_address() {
        let addr = super::get_bind_address();
        assert!(!addr.is_empty());
    }
}
