//! Configuration Constants
//!
//! Centralized default values and constants for the Songbird Orchestrator

use std::time::Duration;

/// Network and communication constants
pub mod network {
    use super::*;
    
    /// Default bind address for development
    pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1";
    
    /// Default bind address for production (all interfaces)
    pub const PRODUCTION_BIND_ADDRESS: &str = "0.0.0.0";
    
    /// Default API port
    pub const DEFAULT_PORT: u16 = 8080;
    
    /// Default port range for services
    pub const DEFAULT_PORT_RANGE: (u16, u16) = (8000, 9000);
    
    /// Default connection timeout
    pub const DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);
    
    /// Default request timeout
    pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
    
    /// Default read timeout
    pub const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(30);
    
    /// Default write timeout
    pub const DEFAULT_WRITE_TIMEOUT: Duration = Duration::from_secs(30);
    
    /// Default idle timeout
    pub const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(300);
}

/// Service management constants
pub mod services {
    use super::*;
    
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
    use super::*;
    
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
    use super::*;
    
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
    use super::*;
    
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

/// Security constants
pub mod security {
    use super::*;
    
    /// Default session timeout
    pub const DEFAULT_SESSION_TIMEOUT: Duration = Duration::from_secs(3600);
    
    /// Default rate limit (requests per minute)
    pub const DEFAULT_RATE_LIMIT: u32 = 1000;
    
    /// Default burst size for rate limiting
    pub const DEFAULT_BURST_SIZE: u32 = 100;
    
    /// Default maximum connections
    pub const DEFAULT_MAX_CONNECTIONS: u32 = 1000;
}

/// Resource management constants
pub mod resources {
    use super::*;
    
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
    use super::*;
    
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
    use super::*;
    
    /// Development environment constants
    pub mod development {
        use super::*;
        
        pub const BIND_ADDRESS: &str = network::DEFAULT_BIND_ADDRESS;
        pub const LOG_LEVEL: &str = "debug";
        pub const METRICS_INTERVAL: Duration = Duration::from_secs(30);
        pub const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(10);
    }
    
    /// Production environment constants
    pub mod production {
        use super::*;
        
        pub const BIND_ADDRESS: &str = network::PRODUCTION_BIND_ADDRESS;
        pub const LOG_LEVEL: &str = "warn";
        pub const METRICS_INTERVAL: Duration = Duration::from_secs(60);
        pub const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);
    }
    
    /// Testing environment constants
    pub mod testing {
        use super::*;
        
        pub const BIND_ADDRESS: &str = network::DEFAULT_BIND_ADDRESS;
        pub const LOG_LEVEL: &str = "error";
        pub const METRICS_INTERVAL: Duration = Duration::from_secs(5);
        pub const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(1);
    }
}

/// Utility functions for working with constants
pub mod utils {
    use super::*;
    
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

/// Environment-specific default values
#[derive(Debug, Clone)]
pub struct EnvironmentDefaults {
    pub bind_address: &'static str,
    pub log_level: &'static str,
    pub metrics_interval: Duration,
    pub health_check_interval: Duration,
} 