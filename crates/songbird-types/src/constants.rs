//! **CANONICAL**: Core constants for the Songbird ecosystem
//!
//! This module provides all the canonical constants used throughout Songbird.
//! All components MUST use these constants to ensure consistency.

use std::env;

/// **CANONICAL**: Network addresses and endpoints
pub struct CanonicalNetworkAddresses;

impl CanonicalNetworkAddresses {
    /// Production bind address
    pub const PRODUCTION_BIND_ADDRESS: &'static str = "0.0.0.0";

    /// Localhost bind address for development
    pub const LOCALHOST_NAME: &'static str = "127.0.0.1";
    /// Get bind address string based on environment
    #[must_use]
    pub fn get_bind_address_string(production: bool) -> &'static str {
        if production {
            Self::PRODUCTION_BIND_ADDRESS
        } else {
            Self::LOCALHOST_NAME
        }
    }
}

/// **CANONICAL**: Network limits and constraints
pub struct CanonicalNetworkLimits;

impl CanonicalNetworkLimits {
    /// Maximum concurrent connections
    pub const MAX_CONNECTIONS: u32 = 1000;

    /// Default connection timeout in seconds
    pub const CONNECTION_TIMEOUT_SECONDS: u64 = 30;

    /// Maximum request size in bytes
    pub const MAX_REQUEST_SIZE: usize = 1_048_576;
    // 1MB
}

/// **CANONICAL**: Resource defaults
pub struct CanonicalResourceDefaults;

impl CanonicalResourceDefaults {
    /// Default memory limit in bytes
    pub const DEFAULT_MEMORY_LIMIT: u64 = 1_073_741_824; // 1GB;

    /// Default CPU limit percentage
    pub const DEFAULT_CPU_LIMIT: f64 = 80.0;

    /// Default disk space threshold
    pub const DEFAULT_DISK_THRESHOLD: u64 = 10_737_418_240;
    // 10GB
}

/// **CANONICAL**: Performance defaults
pub struct CanonicalPerformanceDefaults;

impl CanonicalPerformanceDefaults {
    /// Default response timeout in milliseconds
    pub const DEFAULT_RESPONSE_TIMEOUT_MS: u64 = 5000;

    /// Default retry attempts
    pub const DEFAULT_RETRY_ATTEMPTS: u32 = 3;

    /// Default batch size for operations
    pub const DEFAULT_BATCH_SIZE: usize = 100;
}

/// **CANONICAL**: Discovery defaults
pub struct CanonicalDiscoveryDefaults;

impl CanonicalDiscoveryDefaults {
    /// Default discovery interval in seconds
    pub const DEFAULT_DISCOVERY_INTERVAL_SECONDS: u64 = 30;

    /// Default health check interval in seconds
    pub const DEFAULT_HEALTH_CHECK_INTERVAL_SECONDS: u64 = 60;

    /// Default service timeout in seconds
    pub const DEFAULT_SERVICE_TIMEOUT_SECONDS: u64 = 10;
}

/// **CANONICAL**: Environment constants
pub struct CanonicalEnvironmentConstants;

impl CanonicalEnvironmentConstants {
    /// Get environment variable or default
    #[must_use]
    pub fn get_env_or_default(env_var: &str, default: &str) -> String {
        env::var(env_var).unwrap_or_else(|_| default.to_string())
    }

    /// Get bind address from environment
    #[must_use]
    pub fn get_bind_address() -> String {
        let bind_address = CanonicalNetworkAddresses::get_bind_address_string(false);
        Self::get_env_or_default("SONGBIRD_BIND_ADDRESS", &bind_address,
    }

    /// Get port from environment
    #[must_use]
    pub fn get_port(default_port: u16) -> u16 {
        env::var("SONGBIRD_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(default_port,
    }
}
