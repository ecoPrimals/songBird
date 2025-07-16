// Configuration constants
//! Configuration Constants
//!
//! Centralized default values and constants for the Songbird Orchestrator

use std::env;

/// Network and communication constants
pub mod network {
    /// Default bind address for development
    pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1";
    /// Default bind address for production (all interfaces)
    pub const PRODUCTION_BIND_ADDRESS: &str = "0.0.0.0";
    /// Default localhost address
    pub const DEFAULT_LOCALHOST: &str = "127.0.0.1";
    /// Default API port
    pub const DEFAULT_PORT: u16 = 8080;
    /// Default port range for services
    pub const DEFAULT_PORT_RANGE: (u16, u16) = (8000, 9000);
    /// Default connection timeout
    pub const DEFAULT_CONNECTION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
    /// Default request timeout
    pub const DEFAULT_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
    /// Default read timeout
    pub const DEFAULT_READ_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
    /// Default write timeout
    pub const DEFAULT_WRITE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
    /// Default idle timeout
    pub const DEFAULT_IDLE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(300);
}
/// Service management constants
pub mod services {
    /// Default maximum number of services
    pub const DEFAULT_MAX_SERVICES: usize = 100;
    /// Default service startup timeout
    pub const DEFAULT_STARTUP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);
    /// Default service shutdown timeout
    pub const DEFAULT_SHUTDOWN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
    /// Default service restart backoff
    pub const DEFAULT_RESTART_BACKOFF: std::time::Duration = std::time::Duration::from_secs(10);
    /// Default maximum restart attempts
    pub const DEFAULT_MAX_RESTART_ATTEMPTS: u32 = 3;
}

/// Health checking constants
pub mod health {
    /// Default health check interval
    pub const DEFAULT_CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);
    /// Default health check timeout
    pub const DEFAULT_CHECK_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
    /// Default failure threshold
    pub const DEFAULT_FAILURE_THRESHOLD: u32 = 3;
    /// Default recovery threshold
    pub const DEFAULT_RECOVERY_THRESHOLD: u32 = 2;
    /// Default retry delay
    pub const DEFAULT_RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(1);
}

/// Monitoring and metrics constants
pub mod monitoring {
    /// Default metrics collection interval
    pub const DEFAULT_METRICS_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60);
    /// Default Prometheus endpoint
    pub const DEFAULT_PROMETHEUS_ENDPOINT: &str = "/metrics";
    /// Default tracing sample rate
    pub const DEFAULT_SAMPLE_RATE: f64 = 0.1;
    /// Default export timeout
    pub const DEFAULT_EXPORT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
}

/// Discovery constants
pub mod discovery {
    /// Default registration TTL
    pub const DEFAULT_REGISTRATION_TTL: std::time::Duration = std::time::Duration::from_secs(30);
    /// Default announcement interval
    pub const DEFAULT_ANNOUNCEMENT_INTERVAL: std::time::Duration =
        std::time::Duration::from_secs(15);
    /// Default discovery interval
    pub const DEFAULT_DISCOVERY_INTERVAL: std::time::Duration = std::time::Duration::from_secs(10);
    /// Default discovery timeout
    pub const DEFAULT_DISCOVERY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
    /// Default multicast address for IPv4
    pub const DEFAULT_MULTICAST_IPV4: &str = "224.0.0.251";
    /// Default multicast address for discovery
    pub const DEFAULT_DISCOVERY_MULTICAST: &str = "239.1.1.1";
}

/// Security constants
pub mod security {
    /// Default session timeout
    pub const DEFAULT_SESSION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3600);
    /// Default rate limit (requests per minute)
    pub const DEFAULT_RATE_LIMIT: u32 = 1000;
    /// Default burst size for rate limiting
    pub const DEFAULT_BURST_SIZE: u32 = 100;
    /// Default maximum connections
    pub const DEFAULT_MAX_CONNECTIONS: u32 = 1000;
}

/// Resource management constants
pub mod resources {
    /// Default resource tracking interval
    pub const DEFAULT_TRACKING_INTERVAL: std::time::Duration = std::time::Duration::from_secs(10);
    /// Default cleanup interval
    pub const DEFAULT_CLEANUP_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60);
    /// Default maximum resource age
    pub const DEFAULT_MAX_RESOURCE_AGE: std::time::Duration = std::time::Duration::from_secs(3600);
    /// Default monitoring interval
    pub const DEFAULT_MONITORING_INTERVAL: std::time::Duration = std::time::Duration::from_secs(30);
    /// Default leak detection interval
    pub const DEFAULT_LEAK_DETECTION_INTERVAL: std::time::Duration =
        std::time::Duration::from_secs(300);
}

/// Feature flags constants
pub mod feature_flags {
    /// Default flag refresh interval
    pub const DEFAULT_REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(300);
    /// Default evaluation timeout
    pub const DEFAULT_EVALUATION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);
    /// Default cache TTL
    pub const DEFAULT_CACHE_TTL: std::time::Duration = std::time::Duration::from_secs(300);
    /// Default cache size
    pub const DEFAULT_CACHE_SIZE: u32 = 1000;
}
/// Logging constants
pub mod logging {
    /// Default log level
    pub const DEFAULT_LOG_LEVEL: &str = "info";
    /// Available log levels
    pub const LOG_LEVELS: &[&str] = &["trace", "debug", "info", "warn", "error"];
    /// Get environment-appropriate log level
    #[must_use]
    pub fn get_log_level_for_environment() -> &'static str {
        match crate::environment_mode().as_str() {
            "development" => "debug",
            _ => "info", // Combined production and default case
        }
    }
}

/// Protocol constants
pub mod protocols {
    /// HTTP protocol identifier
    pub const HTTP: &str = "http";
    /// HTTPS protocol identifier
    pub const HTTPS: &str = "https";
    /// WebSocket protocol identifier
    pub const WEBSOCKET: &str = "websocket";
    /// WebSocket secure protocol identifier
    pub const WEBSOCKET_SECURE: &str = "wss";
    /// In-memory protocol identifier
    pub const IN_MEMORY: &str = "memory";
}
/// Environment-specific constant sets
pub mod environments {

    /// Development environment constants
    pub mod development {
        use super::super::network;
        /// Development environment binds to default local interface
        pub const BIND_ADDRESS: &str = network::DEFAULT_BIND_ADDRESS;
        pub const LOG_LEVEL: &str = "debug";
        pub const METRICS_INTERVAL: u64 = 60;
        pub const HEALTH_CHECK_INTERVAL: u64 = 30;
    }

    /// Production environment constants
    pub mod production {
        use super::super::network;
        /// Production environment may bind to all interfaces
        pub const BIND_ADDRESS: &str = network::PRODUCTION_BIND_ADDRESS;
        pub const LOG_LEVEL: &str = "info";
        pub const METRICS_INTERVAL: u64 = 300;
        pub const HEALTH_CHECK_INTERVAL: u64 = 60;
    }

    /// Testing environment constants
    pub mod testing {
        use super::super::network;
        /// Testing environment binds to default local interface
        pub const BIND_ADDRESS: &str = network::DEFAULT_BIND_ADDRESS;
        pub const LOG_LEVEL: &str = "error";
        pub const METRICS_INTERVAL: u64 = 5;
        pub const HEALTH_CHECK_INTERVAL: u64 = 1;
    }
}
/// Utility functions for working with constants
pub mod utils {
    use super::EnvironmentDefaults;

    /// Get environment-specific defaults
    #[must_use]
    pub fn get_environment_defaults(env: &str) -> EnvironmentDefaults {
        match env {
            "development" => super::development_defaults(),
            _ => super::production_defaults(), // Use production as default instead of development
        }
    }

    /// Validate that a value is within acceptable ranges
    #[must_use]
    pub const fn validate_timeout(timeout: std::time::Duration) -> bool {
        timeout.as_secs() > 0 && timeout.as_secs() < 3600
    }

    /// Validate port number
    #[must_use]
    pub const fn validate_port(port: u16) -> bool {
        port > 1024 && port < 65535
    }

    /// Validate log level string
    #[must_use]
    pub fn validate_log_level(level: &str) -> bool {
        matches!(level, "trace" | "debug" | "info" | "warn" | "error")
    }
}

// Re-export EnvironmentDefaults at the top level
pub use utils::*;
/// Environment-specific default values
#[derive(Debug, Clone)]
pub struct EnvironmentDefaults {
    pub bind_address: &'static str,
    pub log_level: &'static str,
    pub metrics_interval: u64,
    pub health_check_interval: u64,
}

#[must_use]
pub const fn development_defaults() -> EnvironmentDefaults {
    EnvironmentDefaults {
        bind_address: environments::development::BIND_ADDRESS,
        log_level: environments::development::LOG_LEVEL,
        metrics_interval: environments::development::METRICS_INTERVAL,
        health_check_interval: environments::development::HEALTH_CHECK_INTERVAL,
    }
}

#[must_use]
pub const fn production_defaults() -> EnvironmentDefaults {
    EnvironmentDefaults {
        bind_address: environments::production::BIND_ADDRESS,
        log_level: environments::production::LOG_LEVEL,
        metrics_interval: environments::production::METRICS_INTERVAL,
        health_check_interval: environments::production::HEALTH_CHECK_INTERVAL,
    }
}

/// Default bind address (configurable via `SONGBIRD_BIND_ADDRESS`)
#[must_use]
pub fn default_bind_address() -> String {
    std::env::var("SONGBIRD_BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string())
}

/// Default orchestrator port (configurable via `SONGBIRD_ORCHESTRATOR_PORT`)
#[must_use]
pub const fn default_orchestrator_port() -> u16 {
    network::DEFAULT_PORT
}

/// Default discovery port (configurable via `SONGBIRD_DISCOVERY_PORT`)
#[must_use]
pub fn default_discovery_port() -> u16 {
    std::env::var("SONGBIRD_DISCOVERY_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8081)
}

/// Default health port (configurable via `SONGBIRD_HEALTH_PORT`)
#[must_use]
pub fn default_health_port() -> u16 {
    std::env::var("SONGBIRD_HEALTH_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8082)
}

/// Default dashboard port (configurable via `SONGBIRD_DASHBOARD_PORT`)
#[must_use]
pub fn default_dashboard_port() -> u16 {
    std::env::var("SONGBIRD_DASHBOARD_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000)
}

/// Default gaming ports (configurable via `SONGBIRD_GAMING_PORTS`)
#[must_use]
pub fn default_gaming_ports() -> Vec<u16> {
    std::env::var("SONGBIRD_GAMING_PORTS").ok().map_or_else(
        || vec![7777, 7778, 7779, 7780],
        |ports| {
            ports
                .split(',')
                .filter_map(|p| p.trim().parse().ok())
                .collect()
        },
    )
}

/// Get subnet configuration (configurable via `SONGBIRD_SUBNET`)
#[must_use]
pub fn default_subnet() -> String {
    std::env::var("SONGBIRD_SUBNET").unwrap_or_else(|_| "10.0.0.0/24".to_string())
}

/// Get gateway configuration (configurable via `SONGBIRD_GATEWAY`)
#[must_use]
pub fn default_gateway() -> String {
    std::env::var("SONGBIRD_GATEWAY").unwrap_or_else(|_| "10.0.0.1".to_string())
}

/// Get STUN servers (configurable via `SONGBIRD_STUN_SERVERS`)
#[must_use]
pub fn default_stun_servers() -> Vec<String> {
    env::var("SONGBIRD_STUN_SERVERS").ok().map_or_else(
        || {
            vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
            ]
        },
        |servers| servers.split(',').map(|s| s.trim().to_string()).collect(),
    )
}

/// Get base URL for services (configurable via `SONGBIRD_BASE_URL`)
#[must_use]
pub fn default_base_url() -> String {
    std::env::var("SONGBIRD_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

/// Get environment mode (configurable via `SONGBIRD_ENVIRONMENT`)
#[must_use]
pub fn environment_mode() -> String {
    std::env::var("SONGBIRD_ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
}

/// Check if we're in development mode
#[must_use]
pub fn is_development() -> bool {
    environment_mode() == "development"
}

/// Get development-safe bind address
#[must_use]
pub fn development_bind_address() -> String {
    std::env::var("SONGBIRD_DEVELOPMENT_BIND").unwrap_or_else(|_| "127.0.0.1".to_string())
}

/// Get external address for node discovery (configurable via `SONGBIRD_EXTERNAL_ADDRESS`)
#[must_use]
pub fn external_address() -> Option<String> {
    std::env::var("SONGBIRD_EXTERNAL_ADDRESS").ok()
}

/// Get cluster name (configurable via `SONGBIRD_CLUSTER_NAME`)
#[must_use]
pub fn cluster_name() -> String {
    std::env::var("SONGBIRD_CLUSTER_NAME").unwrap_or_else(|_| "songbird-cluster".to_string())
}

/// Get node ID (configurable via `SONGBIRD_NODE_ID`)
#[must_use]
pub fn node_id() -> String {
    std::env::var("SONGBIRD_NODE_ID")
        .unwrap_or_else(|_| format!("songbird-node-{}", std::process::id()))
}

/// Protocol-specific port mappings (configurable via environment)
#[must_use]
pub fn protocol_port_mappings() -> std::collections::HashMap<String, u16> {
    let mut mappings = std::collections::HashMap::new();
    mappings.insert("http".to_string(), 8080);
    mappings.insert("https".to_string(), 8443);
    mappings.insert("ws".to_string(), 8081);
    mappings.insert("wss".to_string(), 8444);
    mappings.insert("tcp".to_string(), 7777);
    mappings.insert("udp".to_string(), 7778);
    mappings.insert("quic".to_string(), 7779);
    mappings.insert("custom".to_string(), 7780);
    mappings
}

/// Get timeout configurations
#[must_use]
pub fn default_timeout_ms() -> u64 {
    std::env::var("SONGBIRD_TIMEOUT_MS")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(30000)
}

/// Get connection retry count
#[must_use]
pub fn default_retry_count() -> u32 {
    std::env::var("SONGBIRD_RETRY_COUNT")
        .ok()
        .and_then(|c| c.parse().ok())
        .unwrap_or(3)
}

/// Check if running in production
#[must_use]
pub fn is_production() -> bool {
    environment_mode() == "production"
}

/// Benchmark constants
pub mod benchmarks {
    /// Default benchmark warmup duration
    pub const DEFAULT_WARMUP_DURATION: std::time::Duration = std::time::Duration::from_secs(5);
    /// Default benchmark test duration
    pub const DEFAULT_TEST_DURATION: std::time::Duration = std::time::Duration::from_secs(30);
    /// Default benchmark short warmup duration
    pub const DEFAULT_SHORT_WARMUP_DURATION: std::time::Duration =
        std::time::Duration::from_secs(1);
    /// Default benchmark short test duration
    pub const DEFAULT_SHORT_TEST_DURATION: std::time::Duration = std::time::Duration::from_secs(5);
    /// Default benchmark monitoring interval
    pub const DEFAULT_BENCHMARK_MONITORING_INTERVAL: std::time::Duration =
        std::time::Duration::from_millis(100);
    /// Default benchmark micro interval
    pub const DEFAULT_BENCHMARK_MICRO_INTERVAL: std::time::Duration =
        std::time::Duration::from_millis(10);
}

/// CLI constants
pub mod cli {
    /// Default CLI animation delay (1 second)
    pub const DEFAULT_CLI_ANIMATION_DELAY: std::time::Duration =
        std::time::Duration::from_millis(1000);
    /// Default CLI short animation delay (0.5 seconds)
    pub const DEFAULT_CLI_SHORT_ANIMATION_DELAY: std::time::Duration =
        std::time::Duration::from_millis(500);
}

// DEPRECATED CONSTANTS - SHOULD NOT BE USED
// These exist only for migration purposes and will be removed

#[deprecated(note = "Use default_bind_address() instead")]
pub const DEFAULT_BIND_ADDRESS: &str = "DEPRECATED";

#[deprecated(note = "Use development_bind_address() instead")]
pub const DEVELOPMENT_BIND_ADDRESS: &str = "DEPRECATED";

// Re-export commonly used constants for backward compatibility
pub use feature_flags::{DEFAULT_CACHE_TTL, DEFAULT_EVALUATION_TIMEOUT};
pub use monitoring::DEFAULT_METRICS_INTERVAL;
pub use health::{DEFAULT_CHECK_INTERVAL, DEFAULT_CHECK_TIMEOUT, DEFAULT_RETRY_DELAY};
pub use network::{DEFAULT_CONNECTION_TIMEOUT, DEFAULT_LOCALHOST, DEFAULT_PORT, PRODUCTION_BIND_ADDRESS};
pub use resources::{
    DEFAULT_CLEANUP_INTERVAL, DEFAULT_LEAK_DETECTION_INTERVAL, DEFAULT_MAX_RESOURCE_AGE,
    DEFAULT_MONITORING_INTERVAL, DEFAULT_TRACKING_INTERVAL,
};
pub use services::DEFAULT_SHUTDOWN_TIMEOUT;
