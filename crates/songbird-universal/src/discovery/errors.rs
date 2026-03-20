// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery Errors
//!
//! EVOLVED: Comprehensive error types for discovery operations

use std::fmt;

/// Errors that can occur during discovery
#[derive(Debug)]
pub enum DiscoveryError {
    /// Network error during discovery
    NetworkError(String),
    /// Configuration error
    ConfigurationError(String),
    /// Timeout during discovery
    Timeout,
    /// No primals discovered
    NoPrimalsFound,
    /// Health check failed
    HealthCheckFailed(String),
    /// Invalid endpoint format
    InvalidEndpoint(String),
    /// Environment variable error
    EnvironmentError(String),
    /// Container discovery error
    ContainerError(String),
    /// Backend unavailable
    BackendUnavailable(String),
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NetworkError(msg) => write!(f, "Network error: {msg}"),
            Self::ConfigurationError(msg) => write!(f, "Configuration error: {msg}"),
            Self::Timeout => write!(f, "Discovery timeout"),
            Self::NoPrimalsFound => write!(f, "No primals discovered"),
            Self::HealthCheckFailed(msg) => write!(f, "Health check failed: {msg}"),
            Self::InvalidEndpoint(endpoint) => write!(f, "Invalid endpoint: {endpoint}"),
            Self::EnvironmentError(msg) => write!(f, "Environment error: {msg}"),
            Self::ContainerError(msg) => write!(f, "Container discovery error: {msg}"),
            Self::BackendUnavailable(msg) => write!(f, "Backend unavailable: {msg}"),
        }
    }
}

impl std::error::Error for DiscoveryError {}

/// Result type for discovery operations
pub type DiscoveryResult<T> = Result<T, DiscoveryError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_error_display_network() {
        let err = DiscoveryError::NetworkError("Connection refused".to_string());
        assert_eq!(err.to_string(), "Network error: Connection refused");
    }

    #[test]
    fn test_discovery_error_display_configuration() {
        let err = DiscoveryError::ConfigurationError("Invalid port".to_string());
        assert_eq!(err.to_string(), "Configuration error: Invalid port");
    }

    #[test]
    fn test_discovery_error_display_timeout() {
        let err = DiscoveryError::Timeout;
        assert_eq!(err.to_string(), "Discovery timeout");
    }

    #[test]
    fn test_discovery_error_display_no_primals() {
        let err = DiscoveryError::NoPrimalsFound;
        assert_eq!(err.to_string(), "No primals discovered");
    }

    #[test]
    fn test_discovery_error_display_health_check() {
        let err = DiscoveryError::HealthCheckFailed("Service unresponsive".to_string());
        assert_eq!(err.to_string(), "Health check failed: Service unresponsive");
    }

    #[test]
    fn test_discovery_error_display_invalid_endpoint() {
        let err = DiscoveryError::InvalidEndpoint("http://[invalid".to_string());
        assert_eq!(err.to_string(), "Invalid endpoint: http://[invalid");
    }

    #[test]
    fn test_discovery_error_display_environment() {
        let err = DiscoveryError::EnvironmentError("DISCOVERY_HOST not set".to_string());
        assert_eq!(err.to_string(), "Environment error: DISCOVERY_HOST not set");
    }

    #[test]
    fn test_discovery_error_display_container() {
        let err = DiscoveryError::ContainerError("Docker not available".to_string());
        assert_eq!(err.to_string(), "Container discovery error: Docker not available");
    }

    #[test]
    fn test_discovery_error_display_backend_unavailable() {
        let err = DiscoveryError::BackendUnavailable("mDNS not enabled".to_string());
        assert_eq!(err.to_string(), "Backend unavailable: mDNS not enabled");
    }

    #[test]
    fn test_discovery_error_is_error() {
        // Test that DiscoveryError implements std::error::Error
        fn assert_error<T: std::error::Error>() {}
        assert_error::<DiscoveryError>();
    }

    #[test]
    fn test_discovery_result_ok() {
        let result: DiscoveryResult<String> = Ok("found".to_string());
        assert!(matches!(result, Ok(ref s) if s == "found"));
    }

    #[test]
    fn test_discovery_result_err() {
        let result: DiscoveryResult<String> = Err(DiscoveryError::Timeout);
        assert!(matches!(result, Err(DiscoveryError::Timeout)));
    }

    #[test]
    fn test_discovery_error_debug_format() {
        let err = DiscoveryError::NetworkError("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NetworkError"));
        assert!(debug_str.contains("test"));
    }
}
