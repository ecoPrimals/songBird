// Configuration constants
//! Configuration Constants
//!
//! Centralized default values and constants for the Songbird Orchestrator

use std::env;
use std::time::Duration;

/// Network configuration constants
pub mod network {
    use std::time::Duration;

    pub const DEFAULT_ORCHESTRATOR_PORT: u16 = 8080;
    pub const DEFAULT_GAMING_PORT: u16 = 6112;
    pub const DEFAULT_FEDERATION_PORT: u16 = 9090;
    pub const DEFAULT_API_PORT: u16 = 3000;
    pub const DEFAULT_METRICS_PORT: u16 = 5000;
    pub const DEFAULT_HEALTH_PORT: u16 = 9000;
    pub const DEFAULT_DASHBOARD_PORT: u16 = 3000;
    pub const DEFAULT_TOADSTOOL_PORT: u16 = 8082;
    pub const DEFAULT_SQUIRREL_PORT: u16 = 8083;
    pub const DEFAULT_NESTGATE_PORT: u16 = 8080;
    pub const DEFAULT_BEARDOG_PORT: u16 = 8443;

    pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1";
    pub const DEFAULT_LOCALHOST: &str = "localhost";
    pub const DEFAULT_PRODUCTION_BIND_ADDRESS: &str = "0.0.0.0";
    pub const PRODUCTION_BIND_ADDRESS: &str = DEFAULT_PRODUCTION_BIND_ADDRESS;
    pub const DEFAULT_PORT: u16 = DEFAULT_ORCHESTRATOR_PORT;

    // Service endpoint defaults
    pub const DEFAULT_BEARDOG_ENDPOINT: &str = "https://localhost:8443";
    pub const DEFAULT_NESTGATE_ENDPOINT: &str = "http://localhost:8080";
    pub const DEFAULT_TOADSTOOL_ENDPOINT: &str = "http://localhost:8082";
    pub const DEFAULT_SQUIRREL_ENDPOINT: &str = "http://localhost:8084";
    pub const DEFAULT_BIOMEOS_ENDPOINT: &str = "http://localhost:4000";
    pub const DEFAULT_CONSUL_ENDPOINT: &str = "http://localhost:8500";
    pub const DEFAULT_ETCD_ENDPOINT: &str = "http://localhost:2379";

    pub const DEFAULT_GAMING_PORT_RANGE_START: u16 = 8000;
    pub const DEFAULT_GAMING_PORT_RANGE_END: u16 = 8100;

    pub const DEFAULT_DISCOVERY_PORTS: &[u16] =
        &[8080, 8081, 8082, 8083, 8084, 8085, 3000, 5000, 9000];

    pub const DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);
    pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
    pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_millis(1000);
    pub const DEFAULT_HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(5);
    pub const DEFAULT_DISCOVERY_INTERVAL: Duration = Duration::from_secs(30);

    pub const MAX_CONNECTIONS: usize = 10000;
    pub const MAX_CONCURRENT_STREAMS: usize = 1000;
    pub const DEFAULT_BUFFER_SIZE: usize = 8192;
    pub const MAX_BANDWIDTH_MBPS: u64 = 1000;

    // Performance and benchmarking constants
    pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300);
    pub const DEFAULT_EVALUATION_TIMEOUT: Duration = Duration::from_secs(60);
    pub const DEFAULT_METRICS_INTERVAL: Duration = Duration::from_secs(30);

    // Benchmarking module functions
    pub fn toadstool_endpoint() -> String {
        DEFAULT_TOADSTOOL_ENDPOINT.to_string()
    }

    pub fn biomeos_endpoint() -> String {
        DEFAULT_BIOMEOS_ENDPOINT.to_string()
    }

    pub fn squirrel_endpoint() -> String {
        DEFAULT_SQUIRREL_ENDPOINT.to_string()
    }
}

/// Performance optimization constants
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300);
pub const DEFAULT_EVALUATION_TIMEOUT: Duration = Duration::from_secs(60);
pub const DEFAULT_METRICS_INTERVAL: Duration = Duration::from_secs(30);

/// Benchmarking module
pub mod benchmarks {
    use std::time::Duration;

    pub const DEFAULT_BENCHMARK_DURATION: Duration = Duration::from_secs(60);
    pub const DEFAULT_BENCHMARK_ITERATIONS: usize = 1000;
    pub const DEFAULT_BENCHMARK_WARMUP_TIME: Duration = Duration::from_secs(10);
    pub const DEFAULT_BENCHMARK_MICRO_INTERVAL: Duration = Duration::from_millis(100);
    pub const DEFAULT_BENCHMARK_MONITORING_INTERVAL: Duration = Duration::from_secs(5);
    pub const DEFAULT_SHORT_TEST_DURATION: Duration = Duration::from_secs(10);
    pub const DEFAULT_SHORT_WARMUP_DURATION: Duration = Duration::from_secs(2);
    pub const DEFAULT_TEST_DURATION: Duration = Duration::from_secs(30);
    pub const DEFAULT_WARMUP_DURATION: Duration = Duration::from_secs(5);
}

/// Performance and scaling constants
pub mod performance {
    use std::time::Duration;

    pub const DEFAULT_POOL_SIZE: usize = 1000;
    pub const DEFAULT_BUFFER_POOL_SIZE: usize = 2000;
    pub const DEFAULT_MESSAGE_POOL_SIZE: usize = 5000;
    pub const DEFAULT_REQUEST_POOL_SIZE: usize = 10000;

    pub const DEFAULT_CACHE_SIZE: usize = 10000;
    pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300);

    pub const DEFAULT_BATCH_SIZE: usize = 100;
    pub const DEFAULT_BATCH_TIMEOUT: Duration = Duration::from_millis(100);

    pub const DEFAULT_WARMUP_DURATION: Duration = Duration::from_secs(10);
    pub const DEFAULT_STEP_DURATION: Duration = Duration::from_millis(500);

    pub const CPU_THRESHOLD_HIGH: f64 = 80.0;
    pub const CPU_THRESHOLD_LOW: f64 = 30.0;
    pub const MEMORY_THRESHOLD_HIGH: f64 = 85.0;
    pub const MEMORY_THRESHOLD_LOW: f64 = 40.0;

    pub const DEFAULT_SCALE_FACTOR: f64 = 1.5;
    pub const MIN_SCALE_FACTOR: f64 = 1.0;
    pub const MAX_SCALE_FACTOR: f64 = 3.0;

    pub const DEFAULT_REQUESTS_PER_TEST: usize = 100000;
    pub const DEFAULT_SERVICE_INSTANCES: usize = 1000;
}

/// Security and access constants
pub mod security {
    use std::time::Duration;

    pub const DEFAULT_AUTH_TIMEOUT: Duration = Duration::from_secs(30);
    pub const DEFAULT_SESSION_TIMEOUT: Duration = Duration::from_secs(3600);
    pub const DEFAULT_TOKEN_REFRESH_INTERVAL: Duration = Duration::from_secs(300);

    pub const MIN_PASSWORD_LENGTH: usize = 8;
    pub const MAX_LOGIN_ATTEMPTS: usize = 5;
    pub const LOCKOUT_DURATION: Duration = Duration::from_secs(900);

    pub const DEFAULT_ENCRYPTION_KEY_SIZE: usize = 32;
    pub const DEFAULT_SIGNATURE_SIZE: usize = 64;
    pub const DEFAULT_NONCE_SIZE: usize = 16;

    pub const PRIVILEGED_PORTS_START: u16 = 1;
    pub const PRIVILEGED_PORTS_END: u16 = 1024;
    pub const EPHEMERAL_PORTS_START: u16 = 32768;
    pub const EPHEMERAL_PORTS_END: u16 = 65535;
}

/// File system and storage constants
pub mod storage {
    pub const DEFAULT_STORAGE_CAPACITY_GB: u64 = 1000;
    pub const DEFAULT_STORAGE_QUOTA_GB: u64 = 100;

    pub const MIN_FREE_SPACE_GB: u64 = 10;
    pub const STORAGE_CHECK_INTERVAL_SECS: u64 = 300;

    pub const DEFAULT_LOG_ROTATION_SIZE_MB: u64 = 100;
    pub const DEFAULT_LOG_RETENTION_DAYS: u64 = 30;

    pub const BYTES_PER_KB: u64 = 1024;
    pub const BYTES_PER_MB: u64 = 1024 * 1024;
    pub const BYTES_PER_GB: u64 = 1024 * 1024 * 1024;

    pub const DEFAULT_BACKUP_RETENTION_DAYS: u64 = 7;
    pub const DEFAULT_SNAPSHOT_INTERVAL_HOURS: u64 = 24;
}

/// Gaming-specific constants
pub mod gaming {
    use std::time::Duration;

    pub const STARCRAFT_DEFAULT_PORT: u16 = 6112;
    pub const STARCRAFT_BROADCAST_PORT: u16 = 6113;
    pub const AGE_OF_EMPIRES_PORT: u16 = 2300;
    pub const DIABLO_PORT: u16 = 6112;
    pub const WARCRAFT_PORT: u16 = 6112;

    pub const DEFAULT_GAME_TIMEOUT: Duration = Duration::from_secs(300);
    pub const DEFAULT_LOBBY_TIMEOUT: Duration = Duration::from_secs(120);
    pub const DEFAULT_MATCH_TIMEOUT: Duration = Duration::from_secs(3600);

    pub const MAX_PLAYERS_PER_GAME: usize = 8;
    pub const MAX_SPECTATORS_PER_GAME: usize = 16;
    pub const MAX_CONCURRENT_GAMES: usize = 100;

    pub const DEFAULT_PING_INTERVAL: Duration = Duration::from_secs(30);
    pub const MAX_PING_TIMEOUT: Duration = Duration::from_secs(5);
    pub const HIGH_LATENCY_THRESHOLD_MS: u64 = 150;
}

/// AI and ML constants
pub mod ai {
    use std::time::Duration;

    pub const DEFAULT_INFERENCE_TIMEOUT: Duration = Duration::from_secs(30);
    pub const DEFAULT_BATCH_SIZE: usize = 32;
    pub const DEFAULT_STREAM_BUFFER_SIZE: usize = 1024;

    pub const MAX_MODEL_SIZE_MB: u64 = 10000;
    pub const DEFAULT_CACHE_SIZE_MB: u64 = 1000;
    pub const DEFAULT_GPU_MEMORY_MB: u64 = 8000;

    pub const PRIORITY_CRITICAL: u8 = 0;
    pub const PRIORITY_HIGH: u8 = 1;
    pub const PRIORITY_MEDIUM: u8 = 2;
    pub const PRIORITY_LOW: u8 = 3;
    pub const PRIORITY_BULK: u8 = 4;

    pub const DEFAULT_WARMUP_ITERATIONS: usize = 10;
    pub const DEFAULT_BENCHMARK_ITERATIONS: usize = 1000;

    pub const CACHE_HIT_RATE_THRESHOLD: f64 = 0.85;
    pub const THROUGHPUT_THRESHOLD_OPS_PER_SEC: f64 = 1000.0;
}

/// Validation constants
pub mod validation {
    pub const MIN_PORT: u16 = 1024;
    pub const MAX_PORT: u16 = 65535;

    pub const MIN_TIMEOUT_MS: u64 = 100;
    pub const MAX_TIMEOUT_MS: u64 = 300000;

    pub const MIN_RETRY_COUNT: usize = 1;
    pub const MAX_RETRY_COUNT: usize = 10;

    pub const MIN_THREAD_POOL_SIZE: usize = 1;
    pub const MAX_THREAD_POOL_SIZE: usize = 1000;

    pub const MIN_MEMORY_LIMIT_MB: u64 = 128;
    pub const MAX_MEMORY_LIMIT_MB: u64 = 1024 * 1024; // 1TB

    pub const MIN_BUFFER_SIZE: usize = 64;
    pub const MAX_BUFFER_SIZE: usize = 64 * 1024 * 1024; // 64MB

    pub const MIN_PERCENTAGE: f64 = 0.0;
    pub const MAX_PERCENTAGE: f64 = 100.0;

    pub const MIN_RATE_LIMIT: f64 = 0.1;
    pub const MAX_RATE_LIMIT: f64 = 100_000.0;
}

/// Default paths and directories
pub mod paths {
    pub const DEFAULT_CONFIG_DIR: &str = "config";
    pub const DEFAULT_DATA_DIR: &str = "data";
    pub const DEFAULT_LOG_DIR: &str = "logs";
    pub const DEFAULT_CACHE_DIR: &str = "cache";
    pub const DEFAULT_TEMP_DIR: &str = "tmp";

    pub const DEFAULT_CONFIG_FILE: &str = "songbird.toml";
    pub const DEFAULT_LOG_FILE: &str = "songbird.log";
    pub const DEFAULT_PID_FILE: &str = "songbird.pid";

    pub const UNIX_VAR_LIB: &str = "/var/lib/songbird";
    pub const UNIX_VAR_LOG: &str = "/var/log/songbird";
    pub const UNIX_VAR_CACHE: &str = "/var/cache/songbird";
    pub const UNIX_VAR_RUN: &str = "/var/run/songbird";
    pub const UNIX_ETC: &str = "/etc/songbird";

    pub const WINDOWS_PROGRAM_DATA: &str = r"C:\ProgramData\Songbird";
    pub const WINDOWS_PROGRAM_FILES: &str = r"C:\Program Files\Songbird";
    pub const WINDOWS_USER_DATA: &str = r"AppData\Local\Songbird";
}

/// Service management constants
pub mod services {
    use std::time::Duration;

    /// Default maximum number of services
    pub const DEFAULT_MAX_SERVICES: usize = 100;
    /// Default service startup timeout
    pub const DEFAULT_STARTUP_TIMEOUT: Duration = Duration::from_secs(60);
    /// Default service shutdown timeout
    pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);
    /// Default service restart backoff
    pub const DEFAULT_RESTART_BACKOFF: Duration = Duration::from_secs(10);
    /// Default maximum restart attempts
    pub const DEFAULT_MAX_RESTART_ATTEMPTS: u32 = 3;
}

/// Health checking constants
pub mod health {
    use std::time::Duration;

    /// Default health check interval
    pub const DEFAULT_CHECK_INTERVAL: Duration = Duration::from_secs(30);
    /// Default health check timeout
    pub const DEFAULT_CHECK_TIMEOUT: Duration = Duration::from_secs(10);
    /// Default failure threshold
    pub const DEFAULT_FAILURE_THRESHOLD: u32 = 3;
    /// Default recovery threshold
    pub const DEFAULT_RECOVERY_THRESHOLD: u32 = 2;
    /// Default retry delay
    pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_secs(1);
}

/// Monitoring and metrics constants
pub mod monitoring {
    use std::time::Duration;

    /// Default metrics collection interval
    pub const DEFAULT_METRICS_INTERVAL: Duration = Duration::from_secs(60);
    /// Default Prometheus endpoint
    pub const DEFAULT_PROMETHEUS_ENDPOINT: &str = "/metrics";
    /// Default tracing sample rate
    pub const DEFAULT_SAMPLE_RATE: f64 = 0.1;
    /// Default export timeout
    pub const DEFAULT_EXPORT_TIMEOUT: Duration = Duration::from_secs(10);
}

/// Discovery constants
pub mod discovery {
    use std::time::Duration;

    /// Default registration TTL
    pub const DEFAULT_REGISTRATION_TTL: Duration = Duration::from_secs(30);
    /// Default announcement interval
    pub const DEFAULT_ANNOUNCEMENT_INTERVAL: Duration = Duration::from_secs(15);
    /// Default discovery interval
    pub const DEFAULT_DISCOVERY_INTERVAL: Duration = Duration::from_secs(10);
    /// Default multicast address for IPv4
    pub const DEFAULT_MULTICAST_IPV4: &str = "224.0.0.251";
    /// Default multicast address for discovery
    pub const DEFAULT_DISCOVERY_MULTICAST: &str = "239.1.1.1";
}

/// Resource management constants
pub mod resources {
    use std::time::Duration;

    /// Default resource tracking interval
    pub const DEFAULT_TRACKING_INTERVAL: Duration = Duration::from_secs(10);
    /// Default cleanup interval
    pub const DEFAULT_CLEANUP_INTERVAL: Duration = Duration::from_secs(60);
    /// Default maximum resource age
    pub const DEFAULT_MAX_RESOURCE_AGE: Duration = Duration::from_secs(3600);
    /// Default monitoring interval
    pub const DEFAULT_MONITORING_INTERVAL: Duration = Duration::from_secs(30);
    /// Default leak detection interval
    pub const DEFAULT_LEAK_DETECTION_INTERVAL: Duration = Duration::from_secs(300);
}

/// Feature flags constants
pub mod feature_flags {
    use std::time::Duration;

    /// Default flag refresh interval
    pub const DEFAULT_REFRESH_INTERVAL: Duration = Duration::from_secs(300);
    /// Default evaluation timeout
    pub const DEFAULT_EVALUATION_TIMEOUT: Duration = Duration::from_secs(1);
    /// Default cache TTL
    pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300);
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
    pub fn get_log_level_for_environment() -> &'static str {
        match std::env::var("SONGBIRD_ENVIRONMENT").as_deref() {
            Ok("production") | Ok("prod") => "warn",
            Ok("staging") => "info",
            Ok("test") | Ok("testing") => "error",
            Ok("development") | Ok("dev") => "debug",
            _ => DEFAULT_LOG_LEVEL,
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
        use std::time::Duration;

        pub const BIND_ADDRESS: &str = network::DEFAULT_BIND_ADDRESS;
        pub const LOG_LEVEL: &str = "debug";
        pub const METRICS_INTERVAL: Duration = Duration::from_secs(30);
        pub const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(10);
    }

    /// Production environment constants
    pub mod production {
        use super::super::network;
        use std::time::Duration;

        pub const BIND_ADDRESS: &str = network::PRODUCTION_BIND_ADDRESS;
        pub const LOG_LEVEL: &str = "warn";
        pub const METRICS_INTERVAL: Duration = Duration::from_secs(60);
        pub const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);
    }

    /// Testing environment constants
    pub mod testing {
        use super::super::network;
        use std::time::Duration;

        pub const BIND_ADDRESS: &str = network::DEFAULT_BIND_ADDRESS;
        pub const LOG_LEVEL: &str = "error";
        pub const METRICS_INTERVAL: Duration = Duration::from_secs(5);
        pub const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(1);
    }
}
/// Utility functions for working with constants
pub mod utils {
    use super::{environments, health, logging, monitoring, network, EnvironmentDefaults};
    use std::time::Duration;

    /// Get environment-specific defaults
    pub fn get_environment_defaults(env: &str) -> EnvironmentDefaults {
        match env.to_lowercase().as_str() {
            "development" | "dev" => EnvironmentDefaults {
                bind_address: environments::development::BIND_ADDRESS,
                log_level: environments::development::LOG_LEVEL,
                metrics_interval: environments::development::METRICS_INTERVAL,
                health_check_interval: environments::development::HEALTH_CHECK_INTERVAL,
            },
            "production" | "prod" => EnvironmentDefaults {
                bind_address: environments::production::BIND_ADDRESS,
                log_level: environments::production::LOG_LEVEL,
                metrics_interval: environments::production::METRICS_INTERVAL,
                health_check_interval: environments::production::HEALTH_CHECK_INTERVAL,
            },
            "test" | "testing" => EnvironmentDefaults {
                bind_address: environments::testing::BIND_ADDRESS,
                log_level: environments::testing::LOG_LEVEL,
                metrics_interval: environments::testing::METRICS_INTERVAL,
                health_check_interval: environments::testing::HEALTH_CHECK_INTERVAL,
            },
            _ => EnvironmentDefaults {
                bind_address: network::DEFAULT_BIND_ADDRESS,
                log_level: logging::DEFAULT_LOG_LEVEL,
                metrics_interval: monitoring::DEFAULT_METRICS_INTERVAL,
                health_check_interval: health::DEFAULT_CHECK_INTERVAL,
            },
        }
    }
    /// Validate that a value is within acceptable ranges
    pub fn validate_timeout(timeout: Duration) -> bool {
        timeout >= Duration::from_millis(100) && timeout <= Duration::from_secs(3600)
    }

    /// Validate port number
    pub fn validate_port(port: u16) -> bool {
        port > 1024 && port < 65535
    }

    /// Validate log level
    pub fn validate_log_level(level: &str) -> bool {
        logging::LOG_LEVELS.contains(&level.to_lowercase().as_str())
    }
}

// Re-export commonly used constants for backward compatibility
pub use network::{DEFAULT_LOCALHOST, DEFAULT_PORT, PRODUCTION_BIND_ADDRESS};

// Re-export EnvironmentDefaults at the top level
pub use utils::*;

/// Environment-specific default values
#[derive(Debug, Clone)]
pub struct EnvironmentDefaults {
    pub bind_address: &'static str,
    pub log_level: &'static str,
    pub metrics_interval: Duration,
    pub health_check_interval: Duration,
}

/// Default bind address (configurable via SONGBIRD_BIND_ADDRESS)
pub fn default_bind_address() -> String {
    if is_production() {
        network::PRODUCTION_BIND_ADDRESS.to_string()
    } else {
        network::DEFAULT_BIND_ADDRESS.to_string()
    }
}

/// Default orchestrator port (configurable via SONGBIRD_ORCHESTRATOR_PORT)
pub fn default_orchestrator_port() -> u16 {
    network::DEFAULT_PORT
}

/// Default discovery port (configurable via SONGBIRD_DISCOVERY_PORT)
pub fn default_discovery_port() -> u16 {
    env::var("SONGBIRD_DISCOVERY_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(5000)
}

/// Default health port (configurable via SONGBIRD_HEALTH_PORT)
pub fn default_health_port() -> u16 {
    env::var("SONGBIRD_HEALTH_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8082)
}

/// Default dashboard port (configurable via SONGBIRD_DASHBOARD_PORT)
pub fn default_dashboard_port() -> u16 {
    env::var("SONGBIRD_DASHBOARD_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8083)
}

/// Default gaming ports (configurable via SONGBIRD_GAMING_PORTS)
pub fn default_gaming_ports() -> Vec<u16> {
    env::var("SONGBIRD_GAMING_PORTS")
        .ok()
        .and_then(|ports| {
            ports
                .split(',')
                .map(|p| p.trim().parse().ok())
                .collect::<Option<Vec<u16>>>()
        })
        .unwrap_or_else(|| vec![6112, 6113, 6114, 6115, 2300])
}

/// Get subnet configuration (configurable via SONGBIRD_SUBNET)
pub fn default_subnet() -> String {
    env::var("SONGBIRD_SUBNET").unwrap_or_else(|_| network::DEFAULT_BIND_ADDRESS.to_string())
}

/// Get gateway configuration (configurable via SONGBIRD_GATEWAY)
pub fn default_gateway() -> String {
    env::var("SONGBIRD_GATEWAY").unwrap_or_else(|_| network::DEFAULT_BIND_ADDRESS.to_string())
}

/// Get STUN servers (configurable via SONGBIRD_STUN_SERVERS)
pub fn default_stun_servers() -> Vec<String> {
    env::var("SONGBIRD_STUN_SERVERS")
        .ok()
        .map(|servers| servers.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_else(|| {
            vec![
                "stun.l.google.com:19302".to_string(),
                "stun1.l.google.com:19302".to_string(),
            ]
        })
}

/// Get base URL for services (configurable via SONGBIRD_BASE_URL)
pub fn default_base_url() -> String {
    format!(
        "http://{}:{}",
        default_bind_address(),
        default_orchestrator_port()
    )
}

/// Get environment mode (configurable via SONGBIRD_ENVIRONMENT)
pub fn environment_mode() -> String {
    env::var("SONGBIRD_ENVIRONMENT").unwrap_or_else(|_| "production".to_string())
}

/// Check if we're in development mode
pub fn is_development() -> bool {
    environment_mode().to_lowercase() == "development"
}

/// Get development-safe bind address
pub fn development_bind_address() -> String {
    if is_development() {
        env::var("SONGBIRD_DEV_BIND_ADDRESS")
            .unwrap_or_else(|_| network::DEFAULT_BIND_ADDRESS.to_string())
    } else {
        default_bind_address()
    }
}

/// Get external address for node discovery (configurable via SONGBIRD_EXTERNAL_ADDRESS)
pub fn external_address() -> Option<String> {
    env::var("SONGBIRD_EXTERNAL_ADDRESS").ok()
}

/// Get cluster name (configurable via SONGBIRD_CLUSTER_NAME)
pub fn cluster_name() -> String {
    env::var("SONGBIRD_CLUSTER_NAME").unwrap_or_else(|_| "default".to_string())
}

/// Get node ID (configurable via SONGBIRD_NODE_ID)
pub fn node_id() -> String {
    env::var("SONGBIRD_NODE_ID").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string())
}

/// Protocol-specific port mappings (configurable via environment)
pub fn protocol_port_mappings() -> std::collections::HashMap<String, u16> {
    let mut mappings = std::collections::HashMap::new();

    // StarCraft
    mappings.insert(
        "starcraft".to_string(),
        env::var("SONGBIRD_STARCRAFT_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(6112),
    );

    // Warcraft
    mappings.insert(
        "warcraft".to_string(),
        env::var("SONGBIRD_WARCRAFT_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(6113),
    );

    // Command & Conquer
    mappings.insert(
        "cnc".to_string(),
        env::var("SONGBIRD_CNC_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(6114),
    );

    // Age of Empires
    mappings.insert(
        "aoe".to_string(),
        env::var("SONGBIRD_AOE_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(6115),
    );

    // DirectPlay
    mappings.insert(
        "directplay".to_string(),
        env::var("SONGBIRD_DIRECTPLAY_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(2300),
    );

    // TCP generic
    mappings.insert(
        "tcp".to_string(),
        env::var("SONGBIRD_TCP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(80),
    );

    mappings
}

/// Get timeout configurations
pub fn default_timeout_ms() -> u64 {
    env::var("SONGBIRD_TIMEOUT_MS")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(30000) // 30 seconds
}

/// Get connection retry count
pub fn default_retry_count() -> u32 {
    env::var("SONGBIRD_RETRY_COUNT")
        .ok()
        .and_then(|r| r.parse().ok())
        .unwrap_or(3)
}

/// Check if running in production
pub fn is_production() -> bool {
    env::var("SONGBIRD_ENV").unwrap_or_default() == "production"
}

// DEPRECATED CONSTANTS - SHOULD NOT BE USED
// These exist only for migration purposes and will be removed

#[deprecated(note = "Use default_bind_address() instead")]
pub const DEFAULT_BIND_ADDRESS: &str = "DEPRECATED";

#[deprecated(note = "Use development_bind_address() instead")]
pub const DEVELOPMENT_BIND_ADDRESS: &str = "DEPRECATED";
