// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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

#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default,
    reason = "intentional pattern; clippy false positive for this API"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_health_config() {
        let config = CanonicalHealthConfig::default();
        assert!(config.enabled);
        assert_eq!(config.check_interval, Duration::from_secs(30));
        assert_eq!(config.check_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_custom_health_config() {
        let config = CanonicalHealthConfig {
            enabled: false,
            check_interval: Duration::from_secs(60),
            check_timeout: Duration::from_secs(10),
        };
        assert!(!config.enabled);
        assert_eq!(config.check_interval, Duration::from_secs(60));
        assert_eq!(config.check_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_health_config_intervals_valid() {
        let config = CanonicalHealthConfig::default();
        // Check interval should be longer than timeout
        assert!(config.check_interval > config.check_timeout);
    }

    #[test]
    fn test_health_config_clone() {
        let config1 = CanonicalHealthConfig::default();
        let config2 = config1.clone();
        assert_eq!(config1.enabled, config2.enabled);
        assert_eq!(config1.check_interval, config2.check_interval);
        assert_eq!(config1.check_timeout, config2.check_timeout);
    }

    #[test]
    fn test_health_config_debug() {
        let config = CanonicalHealthConfig::default();
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("CanonicalHealthConfig"));
        assert!(debug_str.contains("enabled"));
    }

    #[test]
    fn test_health_check_config_alias() {
        let config: HealthCheckConfig = HealthCheckConfig::default();
        assert!(config.enabled);
        assert_eq!(config.check_interval, Duration::from_secs(30));
    }

    #[test]
    fn test_health_config_various_intervals() {
        let fast = CanonicalHealthConfig {
            enabled: true,
            check_interval: Duration::from_secs(5),
            check_timeout: Duration::from_secs(1),
        };
        assert_eq!(fast.check_interval.as_secs(), 5);

        let slow = CanonicalHealthConfig {
            enabled: true,
            check_interval: Duration::from_secs(300),
            check_timeout: Duration::from_secs(30),
        };
        assert_eq!(slow.check_interval.as_secs(), 300);
    }
}
