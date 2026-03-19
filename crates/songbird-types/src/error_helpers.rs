//! Error Helper Traits for Unwrap Elimination
//!
//! This module provides extension traits to eliminate `.unwrap()` and `.expect()` calls
//! by converting errors into `SongbirdError` with proper context.

use crate::{SongbirdError, SongbirdResult};
use std::env;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

/// Extension trait for Result types to eliminate unwrap patterns
pub trait UnwrapElimination<T, E> {
    /// Convert to `SongbirdError` with configuration context
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Configuration` when the inner `Result` is `Err`.
    fn or_config_error(self, field: &str) -> SongbirdResult<T>
    where
        E: std::fmt::Display;

    /// Convert to `SongbirdError` with network context
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Network` when the inner `Result` is `Err`.
    fn or_network_error(self, context: &str) -> SongbirdResult<T>
    where
        E: std::fmt::Display;

    /// Convert to `SongbirdError` with service context
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Service` when the inner `Result` is `Err`.
    fn or_service_error(self, service: &str) -> SongbirdResult<T>
    where
        E: std::fmt::Display;

    /// Convert to `SongbirdError` with discovery context
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Discovery` when the inner `Result` is `Err`.
    fn or_discovery_error(self, backend: &str) -> SongbirdResult<T>
    where
        E: std::fmt::Display;

    /// Convert to `SongbirdError` with registry context
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Registry` when the inner `Result` is `Err`.
    fn or_registry_error(self, operation: &str) -> SongbirdResult<T>
    where
        E: std::fmt::Display;
}

impl<T, E> UnwrapElimination<T, E> for Result<T, E> {
    fn or_config_error(self, field: &str) -> SongbirdResult<T>
    where
        E: std::fmt::Display,
    {
        self.map_err(|e| SongbirdError::Configuration {
            message: format!("{field}: {e}"),
            field: Some(field.to_string()),
            suggestion: Some("Check configuration file and environment variables".to_string()),
        })
    }

    fn or_network_error(self, context: &str) -> SongbirdResult<T>
    where
        E: std::fmt::Display,
    {
        self.map_err(|e| SongbirdError::Network {
            message: format!("{context}: {e}"),
            interface: None,
            suggestion: Some("Check network connectivity and firewall settings".to_string()),
        })
    }

    fn or_service_error(self, service: &str) -> SongbirdResult<T>
    where
        E: std::fmt::Display,
    {
        self.map_err(|e| SongbirdError::Service {
            service: service.to_string(),
            message: e.to_string(),
            suggested_alternatives: vec![],
            recovery_actions: vec!["retry".to_string(), "check service health".to_string()],
        })
    }

    fn or_discovery_error(self, backend: &str) -> SongbirdResult<T>
    where
        E: std::fmt::Display,
    {
        self.map_err(|e| SongbirdError::Discovery {
            message: e.to_string(),
            backend: Some(backend.to_string()),
            retry_strategy: Some("exponential_backoff".to_string()),
        })
    }

    fn or_registry_error(self, operation: &str) -> SongbirdResult<T>
    where
        E: std::fmt::Display,
    {
        self.map_err(|e| SongbirdError::Registry {
            message: e.to_string(),
            service_name: None,
            operation: operation.to_string(),
        })
    }
}

/// Extension trait for Option types to eliminate unwrap on None
pub trait OptionElimination<T> {
    /// Convert None to configuration error
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Configuration` when the `Option` is `None`.
    fn or_config_missing(self, field: &str) -> SongbirdResult<T>;

    /// Convert None to service not found error
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Service` when the `Option` is `None`.
    fn or_service_not_found(self, service: &str) -> SongbirdResult<T>;

    /// Convert None to resource unavailable error
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Network` when the `Option` is `None`.
    fn or_resource_unavailable(self, resource: &str) -> SongbirdResult<T>;
}

impl<T> OptionElimination<T> for Option<T> {
    fn or_config_missing(self, field: &str) -> SongbirdResult<T> {
        self.ok_or_else(|| SongbirdError::Configuration {
            message: format!("Required configuration field '{field}' is missing"),
            field: Some(field.to_string()),
            suggestion: Some(format!("Set {field} in configuration file or environment variable")),
        })
    }

    fn or_service_not_found(self, service: &str) -> SongbirdResult<T> {
        self.ok_or_else(|| SongbirdError::Service {
            service: service.to_string(),
            message: "Service not found".to_string(),
            suggested_alternatives: vec![],
            recovery_actions: vec![
                "Check service name".to_string(),
                "Verify service is registered".to_string(),
            ],
        })
    }

    fn or_resource_unavailable(self, resource: &str) -> SongbirdResult<T> {
        self.ok_or_else(|| SongbirdError::Network {
            message: format!("Resource '{resource}' is unavailable"),
            interface: None,
            suggestion: Some("Check resource availability and permissions".to_string()),
        })
    }
}

/// Safe parsing utilities to replace `parse().unwrap()`
pub struct SafeParse;

impl SafeParse {
    /// Parse a socket address safely
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Network` when the input cannot be parsed as a socket address.
    pub fn socket_addr(input: &str, context: &str) -> SongbirdResult<SocketAddr> {
        input
            .parse::<SocketAddr>()
            .or_network_error(&format!("Invalid socket address in {context}"))
    }

    /// Parse a port number safely
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Configuration` when the input cannot be parsed as a port.
    pub fn port(input: &str, context: &str) -> SongbirdResult<u16> {
        input.parse::<u16>().or_config_error(&format!("Invalid port number in {context}"))
    }

    /// Parse a duration from milliseconds safely
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Configuration` when the value is out of range.
    pub fn duration_from_millis(ms: u64, context: &str) -> SongbirdResult<Duration> {
        if ms > 0 && ms < u64::MAX / 1_000_000 {
            Ok(Duration::from_millis(ms))
        } else {
            Err(SongbirdError::Configuration {
                message: format!("Invalid duration: {ms} ms in {context}"),
                field: Some(context.to_string()),
                suggestion: Some(
                    "Use a reasonable timeout value (e.g., 30000 for 30 seconds)".to_string(),
                ),
            })
        }
    }

    /// Parse a duration from seconds safely
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Configuration` when the value is out of range.
    pub fn duration_from_secs(secs: u64) -> SongbirdResult<Duration> {
        if secs > 0 && secs < u64::MAX {
            Ok(Duration::from_secs(secs))
        } else {
            Err(SongbirdError::Configuration {
                message: format!("Invalid duration: {secs} seconds"),
                field: None,
                suggestion: Some("Use a positive duration value".to_string()),
            })
        }
    }

    /// Parse any `FromStr` type safely
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Configuration` when the input cannot be parsed.
    pub fn parse<T>(input: &str, context: &str) -> SongbirdResult<T>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        input.parse::<T>().or_config_error(&format!("Failed to parse {context} from '{input}'"))
    }
}

/// Safe environment variable access
pub struct SafeEnv;

impl SafeEnv {
    /// Get environment variable (returns Result for explicit handling)
    ///
    /// # Errors
    ///
    /// Returns `env::VarError` when the variable is not set or invalid.
    pub fn get(key: &str) -> Result<String, env::VarError> {
        env::var(key)
    }

    /// Get environment variable with default value (safe - never panics)
    pub fn get_or_default(key: &str, default: impl Into<String>) -> String {
        env::var(key).unwrap_or_else(|_| default.into())
    }

    /// Get required environment variable
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Configuration` when the variable is not set.
    pub fn get_required(key: &str) -> SongbirdResult<String> {
        env::var(key).or_config_error(&format!("Missing required environment variable: {key}"))
    }

    /// Get port from environment with default (safe - falls back to default on parse failure)
    #[must_use]
    pub fn get_port(key: &str, default: u16) -> u16 {
        env::var(key).map_or(default, |value| value.parse::<u16>().unwrap_or(default))
    }

    /// Get boolean from environment with default
    #[must_use]
    pub fn get_bool(key: &str, default: bool) -> bool {
        env::var(key)
            .ok()
            .and_then(|v| match v.to_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => Some(true),
                "false" | "0" | "no" | "off" => Some(false),
                _ => v.parse::<bool>().ok(),
            })
            .unwrap_or(default)
    }

    /// Get integer from environment with default (safe - falls back to default on parse failure)
    #[must_use]
    pub fn get_usize(key: &str, default: usize) -> usize {
        env::var(key).map_or(default, |value| value.parse::<usize>().unwrap_or(default))
    }

    /// Generic parse with default value (safe - falls back to default on parse failure)
    pub fn parse<T>(key: &str, default: T) -> T
    where
        T: FromStr,
    {
        env::var(key).ok().and_then(|v| v.parse::<T>().ok()).unwrap_or(default)
    }
}

#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::field_reassign_with_default
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unwrap_elimination_result() {
        let result: Result<i32, &str> = Err("test error");
        let err = result.or_config_error("test_field").unwrap_err();

        match err {
            SongbirdError::Configuration {
                field,
                ..
            } => {
                assert_eq!(field, Some("test_field".to_string()));
            }
            _ => panic!("Expected Configuration error"),
        }
    }

    #[test]
    fn test_option_elimination() {
        let opt: Option<String> = None;
        let err = opt.or_config_missing("test_field").unwrap_err();

        match err {
            SongbirdError::Configuration {
                message,
                ..
            } => {
                assert!(message.contains("test_field"));
                assert!(message.contains("missing"));
            }
            _ => panic!("Expected Configuration error"),
        }
    }

    #[test]
    fn test_safe_parse_port() {
        let port = SafeParse::port("8080", "test").unwrap();
        assert_eq!(port, 8080);

        let err = SafeParse::port("invalid", "test").unwrap_err();
        assert!(matches!(err, SongbirdError::Configuration { .. }));
    }

    #[test]
    fn test_safe_env_defaults() {
        let value = SafeEnv::get_or_default("NONEXISTENT_VAR_TEST", "default");
        assert_eq!(value, "default");

        let bool_val = SafeEnv::get_bool("NONEXISTENT_BOOL_TEST", true);
        assert!(bool_val);
    }

    #[test]
    fn test_safe_parse_duration() {
        let duration = SafeParse::duration_from_millis(5000, "timeout").unwrap();
        assert_eq!(duration, Duration::from_millis(5000));

        let duration_secs = SafeParse::duration_from_secs(30).unwrap();
        assert_eq!(duration_secs, Duration::from_secs(30));
    }

    #[test]
    fn test_service_not_found() {
        let opt: Option<String> = None;
        let err = opt.or_service_not_found("test-service").unwrap_err();

        match err {
            SongbirdError::Service {
                service,
                ..
            } => {
                assert_eq!(service, "test-service");
            }
            _ => panic!("Expected Service error"),
        }
    }
}
