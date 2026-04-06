// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Default host configuration with environment variable support
//!
//! # Environment Variables
//!
//! - `SONGBIRD_HOST` - Default service host (default: "127.0.0.1")
//! - `SONGBIRD_BIND_ADDRESS` - Bind address for services (default: "0.0.0.0")
//! - `SONGBIRD_DISCOVERY_HOST` - Discovery service host (default: same as `SONGBIRD_HOST`)
//! - `SONGBIRD_ORCHESTRATOR_HOST` - Orchestrator service host (default: same as `SONGBIRD_HOST`)

/// Get default service host from environment or localhost
///
/// # Environment Variable
/// `SONGBIRD_HOST` (default: "127.0.0.1")
///
/// # Examples
/// ```no_run
/// use songbird_config::defaults::hosts::default_host;
///
/// let host = default_host();
/// assert_eq!(host, "127.0.0.1"); // Or value from SONGBIRD_HOST
/// ```
#[must_use]
pub fn default_host() -> String {
    songbird_process_env::var("SONGBIRD_HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
}

/// Get bind address for services from environment or default
///
/// # Environment Variable
/// `SONGBIRD_BIND_ADDRESS` (default: "0.0.0.0")
///
/// # Notes
/// - Use "0.0.0.0" to bind to all interfaces (production)
/// - Use "127.0.0.1" to bind to localhost only (development)
#[must_use]
pub fn bind_address() -> String {
    songbird_process_env::var("SONGBIRD_BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0".to_string())
}

/// Get discovery service host from environment or default
///
/// # Environment Variable
/// `SONGBIRD_DISCOVERY_HOST` (default: value of `SONGBIRD_HOST`)
#[must_use]
pub fn discovery_host() -> String {
    songbird_process_env::var("SONGBIRD_DISCOVERY_HOST").unwrap_or_else(|_| default_host())
}

/// Get orchestrator service host from environment or default
///
/// # Environment Variable
/// `SONGBIRD_ORCHESTRATOR_HOST` (default: value of `SONGBIRD_HOST`)
#[must_use]
pub fn orchestrator_host() -> String {
    songbird_process_env::var("SONGBIRD_ORCHESTRATOR_HOST").unwrap_or_else(|_| default_host())
}

/// Get service host by name from environment or default
///
/// # Environment Variable Pattern
/// `SONGBIRD_{SERVICE}_HOST` where SERVICE is uppercase service name
///
/// # Examples
/// ```no_run
/// use songbird_config::defaults::hosts::service_host;
///
/// let host = service_host("METRICS");
/// ```
#[must_use]
pub fn service_host(service_name: &str) -> String {
    let env_var = format!("SONGBIRD_{}_HOST", service_name.to_uppercase());
    songbird_process_env::var(env_var).unwrap_or_else(|_| default_host())
}

/// Check if running in production mode based on environment
///
/// # Environment Variable
/// `SONGBIRD_ENVIRONMENT` (values: "production", "staging", "development")
///
/// Returns true if `SONGBIRD_ENVIRONMENT` is "production" or "staging"
#[must_use]
pub fn is_production() -> bool {
    songbird_process_env::var("SONGBIRD_ENVIRONMENT")
        .map(|e| e == "production" || e == "staging")
        .unwrap_or(false)
}

/// Get environment name
///
/// # Environment Variable
/// `SONGBIRD_ENVIRONMENT` (default: "development")
#[must_use]
pub fn environment() -> String {
    songbird_process_env::var("SONGBIRD_ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_host() {
        let host = default_host();
        assert!(!host.is_empty());
        // Should be localhost or env var value
        assert!(host == "127.0.0.1" || !host.is_empty());
    }

    #[test]
    fn test_bind_address() {
        let addr = bind_address();
        assert!(!addr.is_empty());
    }

    #[test]
    fn test_discovery_host() {
        let host = discovery_host();
        assert!(!host.is_empty());
    }

    #[test]
    fn test_service_host() {
        let host = service_host("TEST");
        assert!(!host.is_empty());
    }

    #[test]
    fn test_environment() {
        let env = environment();
        assert!(!env.is_empty());
    }
}
