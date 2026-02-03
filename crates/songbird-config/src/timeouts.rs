//! Timeout configuration for Songbird
//!
//! Replaces hardcoded `Duration::from_secs(X)` values throughout the codebase
//! with a centralized, configurable timeout system.
//!
//! ## Deep Debt Evolution Principle
//!
//! **Before (Hardcoded)**:
//! ```ignore
//! let timeout = Duration::from_secs(30); // Magic number scattered everywhere
//! ```
//!
//! **After (Configurable)**:
//! ```ignore
//! let config = TimeoutConfig::from_env();
//! let timeout = config.request; // Centralized, documented, testable
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use songbird_config::timeouts::TimeoutConfig;
//! use std::time::Duration;
//!
//! // Load from environment (production)
//! let config = TimeoutConfig::from_env();
//!
//! // Or use specific profile
//! let config = TimeoutConfig::fast();  // Low latency
//! let config = TimeoutConfig::balanced();  // Default
//! let config = TimeoutConfig::reliable();  // High reliability
//!
//! // Apply timeouts
//! tokio::time::timeout(config.connect, async_operation()).await?;
//! ```
//!
//! ## Environment Variables
//!
//! - `SONGBIRD_TIMEOUT_CONNECT` - Connection timeout (default: 5s)
//! - `SONGBIRD_TIMEOUT_REQUEST` - Request timeout (default: 30s)
//! - `SONGBIRD_TIMEOUT_IDLE` - Idle connection timeout (default: 60s)
//! - `SONGBIRD_TIMEOUT_KEEPALIVE` - Keepalive interval (default: 300s)
//! - `SONGBIRD_TIMEOUT_HANDSHAKE` - TLS handshake timeout (default: 10s)
//! - `SONGBIRD_TIMEOUT_DISCOVERY` - Service discovery timeout (default: 15s)
//! - `SONGBIRD_TIMEOUT_HEALTH_CHECK` - Health check timeout (default: 5s)
//! - `SONGBIRD_TIMEOUT_SHUTDOWN` - Graceful shutdown timeout (default: 30s)

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Timeout configuration for all Songbird operations
///
/// Provides centralized timeout configuration to replace hardcoded values
/// scattered throughout the codebase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Connection establishment timeout
    ///
    /// Applied when creating new connections to primals, services, or external endpoints.
    /// Prevents hanging on unreachable destinations.
    ///
    /// Default: 5s
    pub connect: Duration,

    /// Request/response timeout
    ///
    /// Maximum time to wait for a request to complete (including processing time).
    /// Applied to HTTP requests, RPC calls, and IPC operations.
    ///
    /// Default: 30s
    pub request: Duration,

    /// Idle connection timeout
    ///
    /// How long to keep an idle connection open before closing it.
    /// Balances resource usage with connection reuse.
    ///
    /// Default: 60s
    pub idle: Duration,

    /// Keepalive interval
    ///
    /// How often to send keepalive probes on active connections.
    /// Detects dead connections and prevents intermediary timeouts.
    ///
    /// Default: 300s (5 minutes)
    pub keepalive: Duration,

    /// TLS handshake timeout
    ///
    /// Maximum time for TLS handshake to complete (all 13 steps).
    /// Prevents hanging on TLS negotiation failures.
    ///
    /// Default: 10s
    pub handshake: Duration,

    /// Service discovery timeout
    ///
    /// How long to wait for service discovery (mDNS, capability registry, etc.).
    /// Balances discovery thoroughness with startup time.
    ///
    /// Default: 15s
    pub discovery: Duration,

    /// Health check timeout
    ///
    /// Maximum time for a health check probe to complete.
    /// Must be fast to enable rapid failure detection.
    ///
    /// Default: 5s
    pub health_check: Duration,

    /// Graceful shutdown timeout
    ///
    /// Maximum time to wait for graceful shutdown before forcing termination.
    /// Allows in-flight requests to complete.
    ///
    /// Default: 30s
    pub shutdown: Duration,
}

impl Default for TimeoutConfig {
    /// Default timeouts (balanced profile)
    ///
    /// Suitable for most deployments. Provides good balance between
    /// responsiveness and reliability.
    fn default() -> Self {
        Self::balanced()
    }
}

impl TimeoutConfig {
    /// Load timeout configuration from environment variables
    ///
    /// Falls back to default values if environment variables are not set.
    /// Invalid values (non-numeric, negative) are logged and replaced with defaults.
    ///
    /// ## Example
    ///
    /// ```bash
    /// export SONGBIRD_TIMEOUT_CONNECT=3
    /// export SONGBIRD_TIMEOUT_REQUEST=60
    /// ```
    ///
    /// ```rust
    /// use songbird_config::timeouts::TimeoutConfig;
    ///
    /// let config = TimeoutConfig::from_env();
    /// assert_eq!(config.connect.as_secs(), 3);  // From env
    /// assert_eq!(config.request.as_secs(), 60); // From env
    /// ```
    pub fn from_env() -> Self {
        Self {
            connect: env_duration("SONGBIRD_TIMEOUT_CONNECT", Duration::from_secs(5)),
            request: env_duration("SONGBIRD_TIMEOUT_REQUEST", Duration::from_secs(30)),
            idle: env_duration("SONGBIRD_TIMEOUT_IDLE", Duration::from_secs(60)),
            keepalive: env_duration("SONGBIRD_TIMEOUT_KEEPALIVE", Duration::from_secs(300)),
            handshake: env_duration("SONGBIRD_TIMEOUT_HANDSHAKE", Duration::from_secs(10)),
            discovery: env_duration("SONGBIRD_TIMEOUT_DISCOVERY", Duration::from_secs(15)),
            health_check: env_duration("SONGBIRD_TIMEOUT_HEALTH_CHECK", Duration::from_secs(5)),
            shutdown: env_duration("SONGBIRD_TIMEOUT_SHUTDOWN", Duration::from_secs(30)),
        }
    }

    /// Fast profile: Optimized for low latency, may sacrifice reliability
    ///
    /// Use when:
    /// - Network is reliable (local, low latency)
    /// - Failures are acceptable (can retry)
    /// - Speed is critical
    ///
    /// Example: Local development, LAN deployments
    pub fn fast() -> Self {
        Self {
            connect: Duration::from_secs(2),
            request: Duration::from_secs(10),
            idle: Duration::from_secs(30),
            keepalive: Duration::from_secs(60),
            handshake: Duration::from_secs(5),
            discovery: Duration::from_secs(5),
            health_check: Duration::from_secs(2),
            shutdown: Duration::from_secs(10),
        }
    }

    /// Balanced profile: Good default for most deployments
    ///
    /// Balances responsiveness with reliability. Works well for:
    /// - Mixed network conditions
    /// - General production use
    /// - Most deployments
    ///
    /// This is the default profile.
    pub fn balanced() -> Self {
        Self {
            connect: Duration::from_secs(5),
            request: Duration::from_secs(30),
            idle: Duration::from_secs(60),
            keepalive: Duration::from_secs(300),
            handshake: Duration::from_secs(10),
            discovery: Duration::from_secs(15),
            health_check: Duration::from_secs(5),
            shutdown: Duration::from_secs(30),
        }
    }

    /// Reliable profile: Maximizes reliability over speed
    ///
    /// Use when:
    /// - Network is unreliable (mobile, satellite, high latency)
    /// - Failures are expensive
    /// - Reliability is critical
    ///
    /// Example: Mobile deployments, satellite links, IoT devices
    pub fn reliable() -> Self {
        Self {
            connect: Duration::from_secs(15),
            request: Duration::from_secs(120),
            idle: Duration::from_secs(300),
            keepalive: Duration::from_secs(600),
            handshake: Duration::from_secs(30),
            discovery: Duration::from_secs(60),
            health_check: Duration::from_secs(15),
            shutdown: Duration::from_secs(90),
        }
    }

    /// Custom profile with specific timeouts
    ///
    /// For specialized use cases where pre-defined profiles don't fit.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use songbird_config::timeouts::TimeoutConfig;
    /// use std::time::Duration;
    ///
    /// let config = TimeoutConfig::custom(
    ///     Duration::from_secs(3),   // connect
    ///     Duration::from_secs(45),  // request
    ///     Duration::from_secs(90),  // idle
    /// );
    /// ```
    pub fn custom(
        connect: Duration,
        request: Duration,
        idle: Duration,
    ) -> Self {
        Self {
            connect,
            request,
            idle,
            keepalive: Duration::from_secs(300),
            handshake: Duration::from_secs(10),
            discovery: Duration::from_secs(15),
            health_check: Duration::from_secs(5),
            shutdown: Duration::from_secs(30),
        }
    }

    /// Validate timeout configuration
    ///
    /// Checks for:
    /// - Timeouts are non-zero
    /// - Timeouts are reasonable (< 1 hour)
    /// - Logical relationships (connect < request, etc.)
    ///
    /// Returns errors describing any validation failures.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check for zero timeouts
        if self.connect.is_zero() {
            errors.push("connect timeout cannot be zero".to_string());
        }
        if self.request.is_zero() {
            errors.push("request timeout cannot be zero".to_string());
        }

        // Check for unreasonably large timeouts (> 1 hour)
        let max_timeout = Duration::from_secs(3600);
        if self.connect > max_timeout {
            errors.push(format!("connect timeout too large: {:?}", self.connect));
        }
        if self.request > max_timeout {
            errors.push(format!("request timeout too large: {:?}", self.request));
        }

        // Check logical relationships
        if self.connect > self.request {
            errors.push("connect timeout should be <= request timeout".to_string());
        }
        if self.handshake > self.request {
            errors.push("handshake timeout should be <= request timeout".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Read duration from environment variable with fallback
///
/// Parses environment variable as seconds (integer). Invalid values
/// log a warning and return the default.
fn env_duration(var: &str, default: Duration) -> Duration {
    match std::env::var(var) {
        Ok(val) => match val.parse::<u64>() {
            Ok(secs) => {
                if secs == 0 {
                    tracing::warn!(
                        "Invalid timeout {}: value cannot be 0, using default {:?}",
                        var,
                        default
                    );
                    default
                } else if secs > 3600 {
                    tracing::warn!(
                        "Suspicious timeout {}: {} seconds > 1 hour, using default {:?}",
                        var,
                        secs,
                        default
                    );
                    default
                } else {
                    Duration::from_secs(secs)
                }
            }
            Err(_) => {
                tracing::warn!(
                    "Invalid timeout {}: '{}' is not a valid number, using default {:?}",
                    var,
                    val,
                    default
                );
                default
            }
        },
        Err(_) => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile() {
        let config = TimeoutConfig::default();
        assert_eq!(config.connect, Duration::from_secs(5));
        assert_eq!(config.request, Duration::from_secs(30));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_fast_profile() {
        let config = TimeoutConfig::fast();
        assert_eq!(config.connect, Duration::from_secs(2));
        assert_eq!(config.request, Duration::from_secs(10));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_balanced_profile() {
        let config = TimeoutConfig::balanced();
        assert_eq!(config.connect, Duration::from_secs(5));
        assert_eq!(config.request, Duration::from_secs(30));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_reliable_profile() {
        let config = TimeoutConfig::reliable();
        assert_eq!(config.connect, Duration::from_secs(15));
        assert_eq!(config.request, Duration::from_secs(120));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation_zero_timeout() {
        let config = TimeoutConfig {
            connect: Duration::ZERO,
            request: Duration::from_secs(30),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_timeout_order() {
        let config = TimeoutConfig {
            connect: Duration::from_secs(60),
            request: Duration::from_secs(30),
            ..Default::default()
        };
        let errors = config.validate().unwrap_err();
        assert!(errors.iter().any(|e| e.contains("connect timeout should be <= request timeout")));
    }

    #[test]
    fn test_custom_profile() {
        let config = TimeoutConfig::custom(
            Duration::from_secs(3),
            Duration::from_secs(45),
            Duration::from_secs(90),
        );
        assert_eq!(config.connect, Duration::from_secs(3));
        assert_eq!(config.request, Duration::from_secs(45));
        assert_eq!(config.idle, Duration::from_secs(90));
        assert!(config.validate().is_ok());
    }
}
