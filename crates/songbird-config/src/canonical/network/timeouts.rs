//! Network Timeout Configuration
//!
//! Timeout configurations for network operations including connections,
//! requests, health checks, and service operations.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Network timeout configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTimeouts {
    pub connection: Duration,
    pub request: Duration,
    pub health_check: Duration,
    pub default: Duration,
}

impl Default for NetworkTimeouts {
    fn default() -> Self {
        Self {
            connection: Duration::from_secs(10),
            request: Duration::from_secs(60),
            health_check: Duration::from_secs(5),
            default: Duration::from_secs(30),
        }
    }
}

/// Network timeout configuration (alternative structure)
///
/// **Merged from**: `config/network/mod.rs`\
/// **Purpose**: Centralized timeout configuration for all network operations
///
/// # Examples
///
/// ```rust
/// use songbird_config::canonical::network::TimeoutConfig;
///
/// let timeouts = TimeoutConfig {
///     default_timeout_secs: 30,
///     connection_timeout_secs: 10,
///     health_check_timeout_secs: 5,
///     registration_timeout_secs: 15,
///     discovery_timeout_secs: 30,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TimeoutConfig {
    /// Default operation timeout in seconds
    pub default_timeout_secs: u64,
    
    /// Connection establishment timeout in seconds
    pub connection_timeout_secs: u64,
    
    /// Health check timeout in seconds
    pub health_check_timeout_secs: u64,
    
    /// Service registration timeout in seconds
    pub registration_timeout_secs: u64,
    
    /// Service discovery timeout in seconds
    pub discovery_timeout_secs: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default_timeout_secs: 30,
            connection_timeout_secs: 10,
            health_check_timeout_secs: 5,
            registration_timeout_secs: 15,
            discovery_timeout_secs: 30,
        }
    }
}

