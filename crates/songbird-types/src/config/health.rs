//! Health /// Configuration capability // Configuration

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// **CANONICAL**: Health configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalHealthConfig {
    /// Enable health checks
    /// Enabled field
    pub enabled: bool,
    /// Health check interval
    /// Check Interval field
    pub check_interval: Duration,
    /// Health check timeout
    pub check_timeout: Duration,
}

impl Default for CanonicalHealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(5),
        }
    }
}

/// Health check configuration - alias for canonical type
pub type HealthCheckConfig = CanonicalHealthConfig;
