// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Zero Hardcoding: Environment-Driven Timeouts
//!
//! This module provides configurable timeout durations with intelligent defaults.
//! NO hardcoded timeout values - everything comes from environment.
//!
//! ## Philosophy
//!
//! - **Configurable Everything**: All timeouts from environment
//! - **Sensible Defaults**: Production-tested default values
//! - **Consistent Units**: All timeouts in seconds (env vars)
//! - **Easy Override**: Single env var per timeout
//!
//! ## Usage
//!
//! ```rust
//! use songbird_config::zero_hardcoding::TimeoutConfig;
//!
//! // Read from environment or use defaults
//! let config = TimeoutConfig::from_env();
//! println!("Connect timeout: {:?}", config.connect);
//!
//! // Use in HTTP client
//! let client = reqwest::Client::builder()
//!     .connect_timeout(config.connect)
//!     .timeout(config.request)
//!     .build()?;
//! ```
//!
//! ## Environment Variables
//!
//! - `TIMEOUT_CONNECT` - Connection timeout in seconds (default: 10)
//! - `TIMEOUT_REQUEST` - Request timeout in seconds (default: 30)
//! - `TIMEOUT_IDLE` - Idle/keep-alive timeout in seconds (default: 60)
//! - `TIMEOUT_DISCOVERY` - Service discovery timeout in seconds (default: 5)
//! - `TIMEOUT_SHUTDOWN` - Graceful shutdown timeout in seconds (default: 30)

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Environment-driven timeout configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeoutConfig {
    /// Connection establishment timeout
    pub connect: Duration,
    
    /// Request/response timeout
    pub request: Duration,
    
    /// Idle connection timeout
    pub idle: Duration,
    
    /// Service discovery timeout
    pub discovery: Duration,
    
    /// Graceful shutdown timeout
    pub shutdown: Duration,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl TimeoutConfig {
    /// Create configuration from environment variables
    ///
    /// Falls back to production-tested defaults if env vars not set.
    pub fn from_env() -> Self {
        Self {
            connect: Self::parse_duration_env("TIMEOUT_CONNECT", 10),
            request: Self::parse_duration_env("TIMEOUT_REQUEST", 30),
            idle: Self::parse_duration_env("TIMEOUT_IDLE", 60),
            discovery: Self::parse_duration_env("TIMEOUT_DISCOVERY", 5),
            shutdown: Self::parse_duration_env("TIMEOUT_SHUTDOWN", 30),
        }
    }
    
    /// Create configuration with explicit values (for testing)
    #[must_use]
    pub const fn with_seconds(connect: u64, request: u64, idle: u64, discovery: u64, shutdown: u64) -> Self {
        Self {
            connect: Duration::from_secs(connect),
            request: Duration::from_secs(request),
            idle: Duration::from_secs(idle),
            discovery: Duration::from_secs(discovery),
            shutdown: Duration::from_secs(shutdown),
        }
    }
    
    /// Aggressive timeouts (for testing or high-performance scenarios)
    #[must_use]
    pub const fn aggressive() -> Self {
        Self::with_seconds(2, 10, 30, 2, 10)
    }
    
    /// Relaxed timeouts (for slow networks or development)
    #[must_use]
    pub const fn relaxed() -> Self {
        Self::with_seconds(30, 120, 300, 15, 60)
    }
    
    /// Parse duration from environment variable (in seconds)
    fn parse_duration_env(key: &str, default_secs: u64) -> Duration {
        std::env::var(key)
            .ok()
            .and_then(|v| v.parse().ok())
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(default_secs))
    }
}

/// Retry configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    
    /// Initial backoff duration
    pub initial_backoff: Duration,
    
    /// Maximum backoff duration
    pub max_backoff: Duration,
    
    /// Backoff multiplier
    pub multiplier: u32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl RetryConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            max_attempts: std::env::var("RETRY_MAX_ATTEMPTS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            initial_backoff: Duration::from_millis(
                std::env::var("RETRY_INITIAL_BACKOFF_MS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(100)
            ),
            max_backoff: Duration::from_secs(
                std::env::var("RETRY_MAX_BACKOFF_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(30)
            ),
            multiplier: std::env::var("RETRY_MULTIPLIER")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2),
        }
    }
    
    /// Conservative retry policy (fewer retries, longer backoff)
    #[must_use]
    pub const fn conservative() -> Self {
        Self {
            max_attempts: 2,
            initial_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(60),
            multiplier: 3,
        }
    }
    
    /// Aggressive retry policy (more retries, shorter backoff)
    #[must_use]
    pub const fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            initial_backoff: Duration::from_millis(50),
            max_backoff: Duration::from_secs(10),
            multiplier: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_timeouts() {
        let config = TimeoutConfig::from_env();
        
        // Defaults should be reasonable for production
        assert!(config.connect.as_secs() > 0);
        assert!(config.request.as_secs() > 0);
        assert!(config.idle.as_secs() > 0);
    }
    
    #[test]
    fn test_explicit_timeouts() {
        let config = TimeoutConfig::with_seconds(5, 15, 45, 3, 20);
        
        assert_eq!(config.connect, Duration::from_secs(5));
        assert_eq!(config.request, Duration::from_secs(15));
        assert_eq!(config.idle, Duration::from_secs(45));
        assert_eq!(config.discovery, Duration::from_secs(3));
        assert_eq!(config.shutdown, Duration::from_secs(20));
    }
    
    #[test]
    fn test_aggressive_vs_relaxed() {
        let aggressive = TimeoutConfig::aggressive();
        let relaxed = TimeoutConfig::relaxed();
        
        // Aggressive should be faster than relaxed
        assert!(aggressive.connect < relaxed.connect);
        assert!(aggressive.request < relaxed.request);
        assert!(aggressive.idle < relaxed.idle);
    }
    
    #[test]
    fn test_retry_defaults() {
        let config = RetryConfig::from_env();
        
        assert!(config.max_attempts > 0);
        assert!(config.initial_backoff.as_millis() > 0);
        assert!(config.max_backoff > config.initial_backoff);
    }
    
    #[test]
    fn test_retry_policies() {
        let conservative = RetryConfig::conservative();
        let aggressive = RetryConfig::aggressive();
        
        // Conservative: fewer retries, longer backoff
        assert!(conservative.max_attempts < aggressive.max_attempts);
        assert!(conservative.initial_backoff > aggressive.initial_backoff);
    }
}

