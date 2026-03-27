// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Default timeout configuration with environment variable support
//!
//! # Environment Variables
//!
//! - `SONGBIRD_TIMEOUT_MS` - Standard timeout (default: 5000ms)
//! - `SONGBIRD_LONG_TIMEOUT_MS` - Long operation timeout (default: 30000ms)
//! - `SONGBIRD_REQUEST_TIMEOUT_MS` - HTTP request timeout (default: 30000ms)
//! - `SONGBIRD_CACHE_EXPIRY_MS` - Cache expiration time (default: 300000ms)
//! - `SONGBIRD_HEARTBEAT_INTERVAL_MS` - Heartbeat interval (default: 60000ms)
//! - `SONGBIRD_DISCOVERY_TIMEOUT_MS` - Service discovery timeout (default: 5000ms)

use std::collections::HashMap;
use std::time::Duration;

/// Get standard timeout from environment or default
///
/// # Environment Variable
/// `SONGBIRD_TIMEOUT_MS` (default: 5000ms / 5 seconds)
///
/// # Examples
/// ```no_run
/// use songbird_config::defaults::timeouts::standard_timeout;
///
/// let timeout = standard_timeout();
/// assert_eq!(timeout.as_secs(), 5);
/// ```
#[must_use]
pub fn standard_timeout() -> Duration {
    let ms = songbird_process_env::var("SONGBIRD_TIMEOUT_MS")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(5000);
    Duration::from_millis(ms)
}

/// Get standard timeout from environment map (for testing - concurrent safe)
#[must_use]
pub fn standard_timeout_from_map(env: &HashMap<String, String>) -> Duration {
    let ms = env.get("SONGBIRD_TIMEOUT_MS").and_then(|t| t.parse().ok()).unwrap_or(5000);
    Duration::from_millis(ms)
}

/// Get long operation timeout from environment or default
///
/// # Environment Variable
/// `SONGBIRD_LONG_TIMEOUT_MS` (default: 30000ms / 30 seconds)
#[must_use]
pub fn long_timeout() -> Duration {
    let ms = songbird_process_env::var("SONGBIRD_LONG_TIMEOUT_MS")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(30000);
    Duration::from_millis(ms)
}

/// Get long operation timeout from environment map (for testing - concurrent safe)
#[must_use]
pub fn long_timeout_from_map(env: &HashMap<String, String>) -> Duration {
    let ms = env.get("SONGBIRD_LONG_TIMEOUT_MS").and_then(|t| t.parse().ok()).unwrap_or(30000);
    Duration::from_millis(ms)
}

/// Get HTTP request timeout from environment or default
///
/// # Environment Variable
/// `SONGBIRD_REQUEST_TIMEOUT_MS` (default: 30000ms / 30 seconds)
#[must_use]
pub fn request_timeout() -> Duration {
    let ms = songbird_process_env::var("SONGBIRD_REQUEST_TIMEOUT_MS")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(30000);
    Duration::from_millis(ms)
}

/// Get HTTP request timeout from environment map (for testing - concurrent safe)
#[must_use]
pub fn request_timeout_from_map(env: &HashMap<String, String>) -> Duration {
    let ms = env.get("SONGBIRD_REQUEST_TIMEOUT_MS").and_then(|t| t.parse().ok()).unwrap_or(30000);
    Duration::from_millis(ms)
}

/// Get cache expiration time from environment or default
///
/// # Environment Variable
/// `SONGBIRD_CACHE_EXPIRY_MS` (default: 300000ms / 5 minutes)
#[must_use]
pub fn cache_expiry() -> Duration {
    let ms = songbird_process_env::var("SONGBIRD_CACHE_EXPIRY_MS")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(300_000);
    Duration::from_millis(ms)
}

/// Get cache expiration time from environment map (for testing - concurrent safe)
#[must_use]
pub fn cache_expiry_from_map(env: &HashMap<String, String>) -> Duration {
    let ms = env.get("SONGBIRD_CACHE_EXPIRY_MS").and_then(|t| t.parse().ok()).unwrap_or(300_000);
    Duration::from_millis(ms)
}

/// Get heartbeat interval from environment or default
///
/// # Environment Variable
/// `SONGBIRD_HEARTBEAT_INTERVAL_MS` (default: 60000ms / 1 minute)
#[must_use]
pub fn heartbeat_interval() -> Duration {
    let ms = songbird_process_env::var("SONGBIRD_HEARTBEAT_INTERVAL_MS")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(60000);
    Duration::from_millis(ms)
}

/// Get heartbeat interval from environment map (for testing - concurrent safe)
#[must_use]
pub fn heartbeat_interval_from_map(env: &HashMap<String, String>) -> Duration {
    let ms =
        env.get("SONGBIRD_HEARTBEAT_INTERVAL_MS").and_then(|t| t.parse().ok()).unwrap_or(60000);
    Duration::from_millis(ms)
}

/// Get service discovery timeout from environment or default
///
/// # Environment Variable
/// `SONGBIRD_DISCOVERY_TIMEOUT_MS` (default: 5000ms / 5 seconds)
#[must_use]
pub fn discovery_timeout() -> Duration {
    let ms = songbird_process_env::var("SONGBIRD_DISCOVERY_TIMEOUT_MS")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(5000);
    Duration::from_millis(ms)
}

/// Get service discovery timeout from environment map (for testing - concurrent safe)
#[must_use]
pub fn discovery_timeout_from_map(env: &HashMap<String, String>) -> Duration {
    let ms = env.get("SONGBIRD_DISCOVERY_TIMEOUT_MS").and_then(|t| t.parse().ok()).unwrap_or(5000);
    Duration::from_millis(ms)
}

/// Get connection timeout from environment or default
///
/// # Environment Variable
/// `SONGBIRD_CONNECTION_TIMEOUT_MS` (default: 10000ms / 10 seconds)
#[must_use]
pub fn connection_timeout() -> Duration {
    let ms = songbird_process_env::var("SONGBIRD_CONNECTION_TIMEOUT_MS")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(10000);
    Duration::from_millis(ms)
}

/// Get connection timeout from environment map (for testing - concurrent safe)
#[must_use]
pub fn connection_timeout_from_map(env: &HashMap<String, String>) -> Duration {
    let ms =
        env.get("SONGBIRD_CONNECTION_TIMEOUT_MS").and_then(|t| t.parse().ok()).unwrap_or(10000);
    Duration::from_millis(ms)
}

/// Get retry backoff delay from environment or default
///
/// # Environment Variable
/// `SONGBIRD_RETRY_BACKOFF_MS` (default: 1000ms / 1 second)
#[must_use]
pub fn retry_backoff() -> Duration {
    let ms = songbird_process_env::var("SONGBIRD_RETRY_BACKOFF_MS")
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or(1000);
    Duration::from_millis(ms)
}

/// Get retry backoff delay from environment map (for testing - concurrent safe)
#[must_use]
pub fn retry_backoff_from_map(env: &HashMap<String, String>) -> Duration {
    let ms = env.get("SONGBIRD_RETRY_BACKOFF_MS").and_then(|t| t.parse().ok()).unwrap_or(1000);
    Duration::from_millis(ms)
}

/// Get timeout by operation name from environment or use default
///
/// # Environment Variable Pattern
/// `SONGBIRD_{OPERATION}_TIMEOUT_MS` where OPERATION is uppercase operation name
///
/// # Examples
/// ```no_run
/// use songbird_config::defaults::timeouts::operation_timeout;
/// use std::time::Duration;
///
/// let timeout = operation_timeout("CUSTOM", Duration::from_secs(10));
/// ```
#[must_use]
pub fn operation_timeout(operation_name: &str, default: Duration) -> Duration {
    let env_var = format!("SONGBIRD_{}_TIMEOUT_MS", operation_name.to_uppercase());
    let ms = songbird_process_env::var(env_var)
        .ok()
        .and_then(|t| t.parse().ok())
        .unwrap_or_else(|| u64::try_from(default.as_millis()).unwrap_or(u64::MAX));
    Duration::from_millis(ms)
}

/// Get timeout by operation name from environment map (for testing - concurrent safe)
#[must_use]
pub fn operation_timeout_from_map(
    env: &HashMap<String, String>,
    operation_name: &str,
    default: Duration,
) -> Duration {
    let env_var = format!("SONGBIRD_{}_TIMEOUT_MS", operation_name.to_uppercase());
    let ms = env
        .get(&env_var)
        .and_then(|t| t.parse().ok())
        .unwrap_or_else(|| u64::try_from(default.as_millis()).unwrap_or(u64::MAX));
    Duration::from_millis(ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_timeout() {
        let timeout = standard_timeout();
        assert!(timeout.as_millis() >= 1000); // At least 1 second
    }

    #[test]
    fn test_long_timeout() {
        let timeout = long_timeout();
        assert!(timeout >= standard_timeout()); // Should be longer than standard
    }

    #[test]
    fn test_request_timeout() {
        let timeout = request_timeout();
        assert!(timeout.as_millis() > 0);
    }

    #[test]
    fn test_cache_expiry() {
        let expiry = cache_expiry();
        assert!(expiry.as_millis() > 0);
    }

    #[test]
    fn test_heartbeat_interval() {
        let interval = heartbeat_interval();
        assert!(interval.as_millis() > 0);
    }

    #[test]
    fn test_discovery_timeout() {
        let timeout = discovery_timeout();
        assert!(timeout.as_millis() > 0);
    }

    #[test]
    fn test_operation_timeout() {
        let timeout = operation_timeout("CUSTOM", Duration::from_secs(5));
        assert!(timeout.as_secs() >= 5);
    }
}
