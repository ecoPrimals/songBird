// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Health monitoring types
//!
//! Types for health checking and status tracking.

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// Health status of a plugin or system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Whether the entity is healthy
    pub healthy: bool,

    /// Health score (0.0 to 1.0, where 1.0 is perfect health)
    pub score: f64,

    /// Timestamp of this health check
    pub timestamp: SystemTime,

    /// Optional message providing details
    pub message: Option<String>,

    /// Response time for the health check
    pub response_time: Duration,
}

impl HealthStatus {
    /// Create a healthy status
    #[must_use]
    pub fn healthy() -> Self {
        Self {
            healthy: true,
            score: 1.0,
            timestamp: SystemTime::now(),
            message: None,
            response_time: Duration::from_millis(0),
        }
    }

    /// Create an unhealthy status
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            healthy: false,
            score: 0.0,
            timestamp: SystemTime::now(),
            message: Some(message.into()),
            response_time: Duration::from_millis(0),
        }
    }

    /// Create a degraded status
    pub fn degraded(score: f64, message: impl Into<String>) -> Self {
        Self {
            healthy: score > 0.5,
            score: score.clamp(0.0, 1.0),
            timestamp: SystemTime::now(),
            message: Some(message.into()),
            response_time: Duration::from_millis(0),
        }
    }

    /// Set the response time for this check
    #[must_use]
    pub const fn with_response_time(mut self, response_time: Duration) -> Self {
        self.response_time = response_time;
        self
    }

    /// Add metadata to the health status message
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let metadata = format!("{}={}", key.into(), value.into());
        self.message = Some(match self.message {
            Some(existing) => format!("{existing}, {metadata}"),
            None => metadata,
        });
        self
    }
}

/// Type of health check to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HealthCheckType {
    /// HTTP endpoint check
    HttpEndpoint {
        /// URL to check
        url: String,
        /// Expected status code
        expected_status: u16,
    },

    /// Process existence check
    ProcessCheck {
        /// Process name to look for
        process_name: String,
    },

    /// Memory usage check
    MemoryUsage {
        /// Maximum memory percentage allowed
        max_percentage: f64,
    },

    /// CPU usage check
    CpuUsage {
        /// Maximum CPU percentage allowed
        max_percentage: f64,
    },

    /// Custom script execution
    CustomScript {
        /// Path to script
        script_path: String,
    },
}

/// Configuration for a health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Type of check to perform
    pub check_type: HealthCheckType,

    /// How often to perform the check
    pub interval: Duration,

    /// Timeout for the check
    pub timeout: Duration,

    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,

    /// Number of consecutive successes before marking healthy again
    pub success_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        use songbird_config::canonical::constants;

        let health_host = songbird_process_env::var("HEALTH_CHECK_HOST")
            .unwrap_or_else(|_| constants::network::default_host());
        let health_port = songbird_process_env::var("HEALTH_CHECK_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        Self {
            check_type: HealthCheckType::HttpEndpoint {
                url: format!("http://{health_host}:{health_port}/health"),
                expected_status: 200,
            },
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
            success_threshold: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]

    use super::*;

    #[test]
    fn test_health_status_healthy() {
        let status = HealthStatus::healthy();
        assert!(status.healthy);
        assert_eq!(status.score, 1.0);
    }

    #[test]
    fn test_health_status_unhealthy() {
        let status = HealthStatus::unhealthy("Service down");
        assert!(!status.healthy);
        assert_eq!(status.score, 0.0);
        assert!(status.message.is_some());
    }

    #[test]
    fn test_health_status_degraded() {
        let status = HealthStatus::degraded(0.7, "High load");
        assert!(status.healthy); // 0.7 > 0.5
        assert_eq!(status.score, 0.7);
    }
}
