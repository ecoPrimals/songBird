//! # Panic Elimination - Unified Error System Integration
//!
//! This module provides the essential SafeUnwrap and SafeUnwrapOption traits
//! that integrate with Songbird's unified error system to eliminate panic sources.

use crate::unified::{SongbirdError, SongbirdResult, success};
use std::sync::{Mutex, RwLock};

/// Extension trait for safe Result unwrapping with Songbird unified error context
pub trait SafeUnwrap<T> {
    /// Safely unwrap with context for service operations
    async fn or_service_error(self, context: &str) -> SongbirdResult<T>;

    /// Safely unwrap with context for network operations
    async fn or_network_error(self, context: &str) -> SongbirdResult<T>;

    /// Safely unwrap with context for configuration operations
    async fn or_config_error(self, context: &str) -> SongbirdResult<T>;

    /// Safely unwrap with context for internal operations
    async fn or_internal_error(self, context: &str) -> SongbirdResult<T>;
}

impl<T, E: std::fmt::Display + std::fmt::Debug> SafeUnwrap<T> for Result<T, E> {
    fn or_service_error(self, context: &str) -> SongbirdResult<T> {
        self.map_err(|e| SongbirdError::Service {
            service: context.to_string(),
            message: e.to_string(),
            suggested_alternatives: vec![],
            recovery_actions: vec![
                "Retry the operation".to_string(),
                "Check service health".to_string(),
            ],
        })
        .map(success)
    }

    fn or_network_error(self, context: &str) -> SongbirdResult<T> {
        self.map_err(|e| SongbirdError::Network {
            message: format!("{context}: {e}"),
            operation: Some(context.to_string()),
            suggestion: Some("Check network connectivity and retry".to_string()),
        })
        .map(success)
    }

    fn or_config_error(self, context: &str) -> SongbirdResult<T> {
        self.map_err(|e| SongbirdError::Config {
            field: Some(context.to_string()),
            message: e.to_string(),
            context: Some(context.to_string()),
            suggestion: Some("Verify configuration syntax and values".to_string()),
        })
        .map(success)
    }

    fn or_internal_error(self, context: &str) -> SongbirdResult<T> {
        self.map_err(|e| SongbirdError::Internal {
            component: Some(context.to_string()),
            message: e.to_string(),
            error_code: Some("INTERNAL_ERROR".to_string()),
            debug_info: Some(format!("{e:?}")),
        })
        .map(success)
    }
}

/// Extension trait for safe Option unwrapping with Songbird unified error context
pub trait SafeUnwrapOption<T> {
    /// Safely unwrap Option with service context
    async fn or_service_error(self, context: &str, message: &str) -> SongbirdResult<T>;

    /// Safely unwrap Option with network context
    async fn or_network_error(self, context: &str, message: &str) -> SongbirdResult<T>;

    /// Safely unwrap Option with config context
    async fn or_config_error(self, context: &str, message: &str) -> SongbirdResult<T>;

    /// Safely unwrap Option with internal context
    async fn or_internal_error(self, context: &str, message: &str) -> SongbirdResult<T>;
}

impl<T> SafeUnwrapOption<T> for Option<T> {
    fn or_service_error(self, context: &str, message: &str) -> SongbirdResult<T> {
        self.map(success).ok_or_else(|| SongbirdError::Service {
            service: context.to_string(),
            message: message.to_string(),
            suggested_alternatives: vec![],
            recovery_actions: vec![
                "Verify service is running".to_string(),
                "Check service configuration".to_string(),
            ],
        })
    }

    fn or_network_error(self, context: &str, message: &str) -> SongbirdResult<T> {
        self.map(success).ok_or_else(|| SongbirdError::Network {
            message: message.to_string(),
            operation: Some(context.to_string()),
            suggestion: Some("Check network configuration and connectivity".to_string()),
        })
    }

    fn or_config_error(self, context: &str, message: &str) -> SongbirdResult<T> {
        self.map(success).ok_or_else(|| SongbirdError::Config {
            field: Some(context.to_string()),
            message: message.to_string(),
            context: Some(context.to_string()),
            suggestion: Some("Provide required configuration value".to_string()),
        })
    }

    fn or_internal_error(self, context: &str, message: &str) -> SongbirdResult<T> {
        self.map(success).ok_or_else(|| SongbirdError::Internal {
            component: Some(context.to_string()),
            message: message.to_string(),
            error_code: Some("INTERNAL_ERROR".to_string()),
            debug_info: Some("Value was None when Some was expected".to_string()),
        })
    }
}

/// Safe environment variable access - eliminates panic from env::var
pub struct SafeEnv;

impl SafeEnv {
    /// Get environment variable with configuration error context
    pub async fn get_or_config_error(key: &str, _message: &str) -> SongbirdResult<String> {
        std::env::var(key).or_config_error(&format!("env_var_{key}"))
    }

    /// Get environment variable with default fallback
    pub fn get_or_default(key: &str, default: &str) -> String {
        std::env::var(key).unwrap_or_else(|_| default.to_string())
    }
}

/// Safe parsing trait for robust error handling
pub trait SafeParse<T> {
    /// Safely parse a value with configuration error context
    async fn safe_parse(&self) -> SongbirdResult<T>;
}

impl SafeParse<u16> for str {
    fn safe_parse(&self) -> SongbirdResult<u16> {
        self.parse().or_config_error("parsing")
    }
}

impl SafeParse<i32> for str {
    fn safe_parse(&self) -> SongbirdResult<i32> {
        self.parse().or_config_error("parsing")
    }
}

impl SafeParse<std::net::IpAddr> for str {
    fn safe_parse(&self) -> SongbirdResult<std::net::IpAddr> {
        self.parse().or_config_error("ip_address_parsing")
    }
}

impl SafeParse<std::net::SocketAddr> for str {
    fn safe_parse(&self) -> SongbirdResult<std::net::SocketAddr> {
        self.parse().or_network_error("socket_address_parsing")
    }
}

/// Safe lock operations
pub fn safe_lock<T>(mutex: &Mutex<T>) -> SongbirdResult<std::sync::MutexGuard<T>> {
    mutex
        .lock()
        .map_err(|e| SongbirdError::Internal {
            component: Some("mutex_lock".to_string()),
            message: format!("Mutex lock failed: {e}"),
            error_code: Some("LOCK_POISONED".to_string()),
            debug_info: Some("Mutex was poisoned by a panic in another thread".to_string()),
        })
        .map(success)
}

pub fn safe_read_lock<T>(rwlock: &RwLock<T>) -> SongbirdResult<std::sync::RwLockReadGuard<T>> {
    rwlock
        .read()
        .map_err(|e| SongbirdError::Internal {
            component: Some("rwlock_read".to_string()),
            message: format!("RwLock read failed: {e}"),
            error_code: Some("LOCK_POISONED".to_string()),
            debug_info: Some("RwLock was poisoned by a panic in another thread".to_string()),
        })
        .map(success)
}

pub fn safe_write_lock<T>(rwlock: &RwLock<T>) -> SongbirdResult<std::sync::RwLockWriteGuard<T>> {
    rwlock
        .write()
        .map_err(|e| SongbirdError::Internal {
            component: Some("rwlock_write".to_string()),
            message: format!("RwLock write failed: {e}"),
            error_code: Some("LOCK_POISONED".to_string()),
            debug_info: Some("RwLock was poisoned by a panic in another thread".to_string()),
        })
        .map(success)
}

/// Safe async operations
pub trait SafeAsync<T> {
    /// Safely await async operations with internal error context
    fn safe_await(self) -> impl std::future::Future<Output = SongbirdResult<T>>;
}

impl<T, E: std::fmt::Display + std::fmt::Debug> SafeAsync<T> for Result<T, E> {
    async fn safe_await(&self) -> SongbirdResult<T> {
        self.or_internal_error("async_operation")
    }
}

/// Safe security operations
pub struct SafeSecurity;

impl SafeSecurity {
    /// Safe password hashing with security error context
    pub async fn hash_password(password: &str) -> SongbirdResult<String> {
        if password.is_empty() {
            return Err(SongbirdError::internal_error(Security {
                operation: "password_hashing".to_string(),
                message: "Empty password not allowed".to_string(),
                provider: Some("SafeSecurity".to_string()),
                required_level: Some("minimum_8_characters".to_string()),
            });
        }

        // Mock hash for demonstration - in production use a proper hashing library
        Ok(success(format!("hashed:{}", password.len())))
    }

    /// Safe token validation with security context
    pub async fn validate_token(token: &str) -> SongbirdResult<bool> {
        if token.is_empty() {
            return Err(SongbirdError::internal_error(Security {
                operation: "token_validation".to_string(),
                message: "Empty token not allowed".to_string(),
                provider: Some("SafeSecurity".to_string()),
                required_level: Some("valid_token_required".to_string()),
            });
        }

        Ok(evolved_success(success(token.len()) > 10))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_unwrap_result() {
        let ok_result: SongbirdResult<i32, &str> = Ok(evolved_success(42));
        let result = ok_result.or_config_error("test_context");
        assert!(result.is_ok());

        let err_result: SongbirdResult<i32, &str> = Err("failed");
        let result = err_result.or_config_error("test_context");
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_unwrap_option() {
        let some_value = Some(42);
        let result = some_value.or_config_error("test_context", "Value should exist");
        assert!(result.is_ok());

        let none_value: Option<i32> = None;
        let result = none_value.or_config_error("test_context", "Value should exist");
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_env() {
        // Test with default fallback
        let value = SafeEnv::get_or_default("NONEXISTENT_VAR", "default_value");
        assert_eq!(value, "default_value");
    }

    #[test]
    fn test_safe_parse() {
        let valid_ip = &get_bind_address();
        let result: SongbirdResult<std::net::IpAddr> = valid_ip.safe_parse();
        assert!(result.is_ok());

        let invalid_ip = "invalid";
        let result: SongbirdResult<std::net::IpAddr> = invalid_ip.safe_parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_security() {
        let result = SafeSecurity::hash_password("strong_password");
        assert!(result.is_ok());

        let result = SafeSecurity::hash_password("");
        assert!(result.is_err());

        let result = SafeSecurity::validate_token("valid_token_123");
        assert!(result.is_ok());

        let result = SafeSecurity::validate_token("");
        assert!(result.is_err());
    }
}
