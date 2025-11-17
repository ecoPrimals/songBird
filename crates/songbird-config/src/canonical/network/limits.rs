//! Connection Limits Configuration
//!
//! Configuration for connection limits, pooling, rate limiting,
//! and load balancing.

use serde::{Deserialize, Serialize};

/// Connection limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionLimits {
    pub max_connections_per_host: usize,
    pub max_total_connections: usize,
    pub max_retries: u32,
    pub pool_idle_timeout_secs: u64,
}

impl Default for ConnectionLimits {
    fn default() -> Self {
        Self {
            max_connections_per_host: 10,
            max_total_connections: 100,
            max_retries: 3,
            pool_idle_timeout_secs: 300,
        }
    }
}

/// Load balancing configuration for distributing traffic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Enable load balancing
    pub enabled: bool,
    /// Load balancing strategy (`round_robin`, `least_connections`, random)
    pub strategy: String,
    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
    /// Backend servers
    pub backends: Vec<String>,
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            strategy: "round_robin".to_string(),
            health_check_interval_secs: 30,
            backends: Vec::new(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Enable rate limiting
    pub enabled: bool,
    /// Requests per second limit
    pub requests_per_second: u32,
    /// Burst size (max requests in short burst)
    pub burst_size: u32,
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            requests_per_second: 100,
            burst_size: 200,
        }
    }
}

/// Connection pool configuration for efficient connection reuse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolConfig {
    /// Maximum number of connections in the pool
    pub max_size: usize,
    /// Minimum number of idle connections to maintain
    pub min_idle: usize,
    /// Maximum lifetime of a connection (in seconds)
    pub max_lifetime_secs: u64,
    /// Idle timeout (in seconds)
    pub idle_timeout_secs: u64,
    /// Connection timeout (in seconds)
    pub connection_timeout_secs: u64,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_size: 100,
            min_idle: 10,
            max_lifetime_secs: 1800, // 30 minutes
            idle_timeout_secs: 600,  // 10 minutes
            connection_timeout_secs: 30,
        }
    }
}
