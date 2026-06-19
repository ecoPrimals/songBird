// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
            suggestion: Some(String::from("Check configuration file and environment variables")),
        })
    }

    fn or_network_error(self, context: &str) -> SongbirdResult<T>
    where
        E: std::fmt::Display,
    {
        self.map_err(|e| SongbirdError::Network {
            message: format!("{context}: {e}"),
            interface: None,
            suggestion: Some(String::from("Check network connectivity and firewall settings")),
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
            recovery_actions: vec![String::from("retry"), String::from("check service health")],
        })
    }

    fn or_discovery_error(self, backend: &str) -> SongbirdResult<T>
    where
        E: std::fmt::Display,
    {
        self.map_err(|e| SongbirdError::Discovery {
            message: e.to_string(),
            backend: Some(backend.to_string()),
            retry_strategy: Some(String::from("exponential_backoff")),
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
            message: String::from("Service not found"),
            suggested_alternatives: vec![],
            recovery_actions: vec![
                String::from("Check service name"),
                String::from("Verify service is registered"),
            ],
        })
    }

    fn or_resource_unavailable(self, resource: &str) -> SongbirdResult<T> {
        self.ok_or_else(|| SongbirdError::Network {
            message: format!("Resource '{resource}' is unavailable"),
            interface: None,
            suggestion: Some(String::from("Check resource availability and permissions")),
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
                suggestion: Some(String::from(
                    "Use a reasonable timeout value (e.g., 30000 for 30 seconds)",
                )),
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
                suggestion: Some(String::from("Use a positive duration value")),
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

/// Safe environment variable access via the process-env overlay.
///
/// All reads go through [`songbird_process_env::var`], which consults the in-memory overlay
/// first and falls back to the OS environment. This means values set via
/// [`songbird_process_env::set_var`] are visible here without touching the real process
/// environment (zero `unsafe`).
pub struct SafeEnv;

impl SafeEnv {
    /// Get environment variable (returns Result for explicit handling)
    ///
    /// # Errors
    ///
    /// Returns `env::VarError` when the variable is not set or invalid.
    pub fn get(key: &str) -> Result<String, env::VarError> {
        songbird_process_env::var(key)
    }

    /// Get environment variable with default value (safe - never panics)
    pub fn get_or_default(key: &str, default: impl Into<String>) -> String {
        songbird_process_env::var(key).unwrap_or_else(|_| default.into())
    }

    /// Get required environment variable
    ///
    /// # Errors
    ///
    /// Returns `SongbirdError::Configuration` when the variable is not set.
    pub fn get_required(key: &str) -> SongbirdResult<String> {
        songbird_process_env::var(key)
            .or_config_error(&format!("Missing required environment variable: {key}"))
    }

    /// Get port from environment with default (safe - falls back to default on parse failure)
    #[must_use]
    pub fn get_port(key: &str, default: u16) -> u16 {
        songbird_process_env::var(key)
            .map_or(default, |value| value.parse::<u16>().unwrap_or(default))
    }

    /// Get boolean from environment with default
    #[must_use]
    pub fn get_bool(key: &str, default: bool) -> bool {
        songbird_process_env::var(key)
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
        songbird_process_env::var(key)
            .map_or(default, |value| value.parse::<usize>().unwrap_or(default))
    }

    /// Generic parse with default value (safe - falls back to default on parse failure)
    pub fn parse<T>(key: &str, default: T) -> T
    where
        T: FromStr,
    {
        songbird_process_env::var(key).ok().and_then(|v| v.parse::<T>().ok()).unwrap_or(default)
    }
}

#[allow(
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
    fn unwrap_elimination_config_error() {
        let result: Result<i32, &str> = Err("test error");
        let err = result.or_config_error("test_field").unwrap_err();
        match err {
            SongbirdError::Configuration {
                field,
                ..
            } => {
                assert_eq!(field, Some(String::from("test_field")));
            }
            _ => panic!("Expected Configuration error"),
        }
    }

    #[test]
    fn unwrap_elimination_network_error() {
        let result: Result<i32, &str> = Err("connection refused");
        let err = result.or_network_error("eth0 test").unwrap_err();
        match err {
            SongbirdError::Network {
                message,
                suggestion,
                ..
            } => {
                assert!(message.contains("eth0 test"));
                assert!(message.contains("connection refused"));
                assert!(suggestion.is_some());
            }
            _ => panic!("Expected Network error"),
        }
    }

    #[test]
    fn unwrap_elimination_service_error() {
        let result: Result<i32, &str> = Err("timeout");
        let err = result.or_service_error("my-svc").unwrap_err();
        match err {
            SongbirdError::Service {
                service,
                recovery_actions,
                ..
            } => {
                assert_eq!(service, "my-svc");
                assert!(!recovery_actions.is_empty());
            }
            _ => panic!("Expected Service error"),
        }
    }

    #[test]
    fn unwrap_elimination_discovery_error() {
        let result: Result<i32, &str> = Err("no peers");
        let err = result.or_discovery_error("mdns").unwrap_err();
        match err {
            SongbirdError::Discovery {
                backend,
                retry_strategy,
                ..
            } => {
                assert_eq!(backend, Some(String::from("mdns")));
                assert!(retry_strategy.is_some());
            }
            _ => panic!("Expected Discovery error"),
        }
    }

    #[test]
    fn unwrap_elimination_registry_error() {
        let result: Result<i32, &str> = Err("duplicate");
        let err = result.or_registry_error("register").unwrap_err();
        match err {
            SongbirdError::Registry {
                operation,
                ..
            } => {
                assert_eq!(operation, "register");
            }
            _ => panic!("Expected Registry error"),
        }
    }

    #[test]
    fn unwrap_elimination_ok_passes_through() {
        let result: Result<i32, &str> = Ok(42);
        assert_eq!(result.or_config_error("field").unwrap(), 42);

        let result2: Result<&str, &str> = Ok("hello");
        assert_eq!(result2.or_network_error("ctx").unwrap(), "hello");
    }

    #[test]
    fn option_config_missing() {
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
    fn option_service_not_found() {
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

    #[test]
    fn option_resource_unavailable() {
        let opt: Option<u32> = None;
        let err = opt.or_resource_unavailable("gpu-0").unwrap_err();
        match err {
            SongbirdError::Network {
                message,
                ..
            } => {
                assert!(message.contains("gpu-0"));
            }
            _ => panic!("Expected Network error"),
        }
    }

    #[test]
    fn option_some_passes_through() {
        let opt: Option<i32> = Some(99);
        assert_eq!(opt.or_config_missing("field").unwrap(), 99);
        assert_eq!(Some("val").or_service_not_found("svc").unwrap(), "val");
        assert_eq!(Some(5u32).or_resource_unavailable("res").unwrap(), 5);
    }

    #[test]
    fn safe_parse_port_valid() {
        assert_eq!(SafeParse::port("8080", "test").unwrap(), 8080);
        assert_eq!(SafeParse::port("0", "test").unwrap(), 0);
        assert_eq!(SafeParse::port("65535", "test").unwrap(), 65535);
    }

    #[test]
    fn safe_parse_port_invalid() {
        assert!(SafeParse::port("invalid", "test").is_err());
        assert!(SafeParse::port("99999", "test").is_err());
        assert!(SafeParse::port("-1", "test").is_err());
        assert!(SafeParse::port("", "test").is_err());
    }

    #[test]
    fn safe_parse_socket_addr_valid() {
        let addr = SafeParse::socket_addr("127.0.0.1:8080", "bind").unwrap();
        assert_eq!(addr.port(), 8080);
    }

    #[test]
    fn safe_parse_socket_addr_invalid() {
        assert!(SafeParse::socket_addr("not-an-addr", "bind").is_err());
        assert!(SafeParse::socket_addr("127.0.0.1", "bind").is_err());
    }

    #[test]
    fn safe_parse_duration_millis() {
        let d = SafeParse::duration_from_millis(5000, "timeout").unwrap();
        assert_eq!(d, Duration::from_millis(5000));
    }

    #[test]
    fn safe_parse_duration_millis_zero_fails() {
        assert!(SafeParse::duration_from_millis(0, "timeout").is_err());
    }

    #[test]
    fn safe_parse_duration_secs() {
        let d = SafeParse::duration_from_secs(30).unwrap();
        assert_eq!(d, Duration::from_secs(30));
    }

    #[test]
    fn safe_parse_duration_secs_zero_fails() {
        assert!(SafeParse::duration_from_secs(0).is_err());
    }

    #[test]
    fn safe_parse_generic() {
        let val: i64 = SafeParse::parse("42", "count").unwrap();
        assert_eq!(val, 42);

        let val: f64 = SafeParse::parse("7.25", "ratio").unwrap();
        assert!((val - 7.25).abs() < f64::EPSILON);
    }

    #[test]
    fn safe_parse_generic_invalid() {
        assert!(SafeParse::parse::<i64>("abc", "count").is_err());
    }

    #[test]
    fn safe_env_defaults() {
        let value = SafeEnv::get_or_default("NONEXISTENT_VAR_TEST_99", "default");
        assert_eq!(value, "default");
    }

    #[test]
    fn safe_env_get_missing_returns_err() {
        assert!(SafeEnv::get("NONEXISTENT_VAR_TEST_99").is_err());
    }

    #[test]
    fn safe_env_get_required_missing_returns_config_error() {
        let err = SafeEnv::get_required("NONEXISTENT_VAR_TEST_99").unwrap_err();
        assert!(matches!(err, SongbirdError::Configuration { .. }));
    }

    #[test]
    fn safe_env_get_port_default() {
        let port = SafeEnv::get_port("NONEXISTENT_PORT_VAR_99", 3000);
        assert_eq!(port, 3000);
    }

    #[test]
    fn safe_env_get_port_with_overlay() {
        songbird_process_env::set_var("__TEST_HELPERS_PORT__", "9999");
        let port = SafeEnv::get_port("__TEST_HELPERS_PORT__", 3000);
        songbird_process_env::remove_var("__TEST_HELPERS_PORT__");
        assert_eq!(port, 9999);
    }

    #[test]
    fn safe_env_get_port_invalid_falls_back() {
        songbird_process_env::set_var("__TEST_HELPERS_PORT_BAD__", "not-a-number");
        let port = SafeEnv::get_port("__TEST_HELPERS_PORT_BAD__", 3000);
        songbird_process_env::remove_var("__TEST_HELPERS_PORT_BAD__");
        assert_eq!(port, 3000);
    }

    #[test]
    fn safe_env_get_bool_variants() {
        for (val, expected) in [
            ("true", true),
            ("1", true),
            ("yes", true),
            ("on", true),
            ("false", false),
            ("0", false),
            ("no", false),
            ("off", false),
        ] {
            songbird_process_env::set_var("__TEST_HELPERS_BOOL__", val);
            assert_eq!(
                SafeEnv::get_bool("__TEST_HELPERS_BOOL__", !expected),
                expected,
                "get_bool({val}) should be {expected}"
            );
        }
        songbird_process_env::remove_var("__TEST_HELPERS_BOOL__");
    }

    #[test]
    fn safe_env_get_bool_garbage_uses_default() {
        songbird_process_env::set_var("__TEST_HELPERS_BOOL_BAD__", "maybe");
        assert!(SafeEnv::get_bool("__TEST_HELPERS_BOOL_BAD__", true));
        assert!(!SafeEnv::get_bool("__TEST_HELPERS_BOOL_BAD__", false));
        songbird_process_env::remove_var("__TEST_HELPERS_BOOL_BAD__");
    }

    #[test]
    fn safe_env_get_usize_default() {
        assert_eq!(SafeEnv::get_usize("NONEXISTENT_USIZE_99", 100), 100);
    }

    #[test]
    fn safe_env_get_usize_with_overlay() {
        songbird_process_env::set_var("__TEST_HELPERS_USIZE__", "256");
        assert_eq!(SafeEnv::get_usize("__TEST_HELPERS_USIZE__", 100), 256);
        songbird_process_env::remove_var("__TEST_HELPERS_USIZE__");
    }

    #[test]
    fn safe_env_parse_generic() {
        songbird_process_env::set_var("__TEST_HELPERS_PARSE__", "42");
        let val: i32 = SafeEnv::parse("__TEST_HELPERS_PARSE__", 0);
        songbird_process_env::remove_var("__TEST_HELPERS_PARSE__");
        assert_eq!(val, 42);
    }

    #[test]
    fn safe_env_parse_missing_uses_default() {
        let val: i32 = SafeEnv::parse("__TEST_HELPERS_PARSE_MISSING__", 99);
        assert_eq!(val, 99);
    }
}
