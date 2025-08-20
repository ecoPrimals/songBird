//! Constants module for Songbird configuration
//!
//! This module provides centralized constants used throughout the Songbird system.

pub mod gaming;
pub mod network;

/// Default network port for Songbird services
pub const DEFAULT_PORT: u16 = 8080;

/// Default discovery port
pub const DEFAULT_DISCOVERY_PORT: u16 = 8081;

/// Default federation port
pub const DEFAULT_FEDERATION_PORT: u16 = 8082;

/// Default configuration file path
pub const DEFAULT_CONFIG_PATH: &str = "songbird.toml";

/// Default log level
pub const DEFAULT_LOG_LEVEL: &str = "info";

use std::time::Duration;

/// Default cache TTL
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300);

/// Default evaluation timeout
pub const DEFAULT_EVALUATION_TIMEOUT: Duration = Duration::from_secs(30);

/// Default metrics collection interval
pub const DEFAULT_METRICS_INTERVAL: Duration = Duration::from_secs(60);

/// Health check constants
pub mod health {
    use std::time::Duration;

    /// Default health check interval
    pub const DEFAULT_HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);

    /// Default health check timeout
    pub const DEFAULT_HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

    /// Default check interval (alias for compatibility)
    pub const DEFAULT_CHECK_INTERVAL: Duration = Duration::from_secs(30);

    /// Default check timeout (alias for compatibility)
    pub const DEFAULT_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

    /// Maximum consecutive health check failures before marking unhealthy
    pub const MAX_HEALTH_CHECK_FAILURES: u32 = 3;
}

/// Resource management constants
pub mod resources {
    use std::time::Duration;

    /// Default memory limit in bytes
    pub const DEFAULT_MEMORY_LIMIT: u64 = 1_073_741_824; // 1GB

    /// Default CPU limit as percentage
    pub const DEFAULT_CPU_LIMIT: f64 = 80.0;

    /// Default disk space threshold in bytes
    pub const DEFAULT_DISK_THRESHOLD: u64 = 10_737_418_240; // 10GB

    /// Default resource cleanup interval
    pub const DEFAULT_CLEANUP_INTERVAL: Duration = Duration::from_secs(300); // 5 minutes

    /// Default leak detection interval
    pub const DEFAULT_LEAK_DETECTION_INTERVAL: Duration = Duration::from_secs(600); // 10 minutes

    /// Default maximum resource age
    pub const DEFAULT_MAX_RESOURCE_AGE: Duration = Duration::from_secs(3600); // 1 hour

    /// Default monitoring interval
    pub const DEFAULT_MONITORING_INTERVAL: Duration = Duration::from_secs(60); // 1 minute

    /// Default tracking interval
    pub const DEFAULT_TRACKING_INTERVAL: Duration = Duration::from_secs(30); // 30 seconds
}

/// Service management constants
pub mod services {
    use std::time::Duration;

    /// Default service startup timeout
    pub const DEFAULT_STARTUP_TIMEOUT: Duration = Duration::from_secs(60);

    /// Default service shutdown timeout
    pub const DEFAULT_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default service restart timeout
    pub const DEFAULT_RESTART_TIMEOUT: Duration = Duration::from_secs(45);

    /// Maximum number of service restarts
    pub const MAX_SERVICE_RESTARTS: u32 = 5;
}

// Add CLI constants module
pub mod cli {
    use std::time::Duration;

    /// Default network timeout for CLI operations
    pub const DEFAULT_NETWORK_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default discovery timeout for CLI commands
    pub const DEFAULT_DISCOVERY_TIMEOUT: Duration = Duration::from_secs(10);

    /// Default CLI output format
    pub const DEFAULT_OUTPUT_FORMAT: &str = "json";

    /// Default CLI log level
    pub const DEFAULT_LOG_LEVEL: &str = "info";

    /// Default CLI config file name
    pub const DEFAULT_CONFIG_FILE: &str = "songbird.toml";
}

// Re-export main constants for backward compatibility
pub use gaming::*;
pub use network::*;
