//! Domain-specific error integration for Songbird
//!
//! This module provides domain-specific error types and integration patterns
//! for different parts of the Songbird ecosystem.

use crate::SongbirdError;
use std::collections::HashMap;
use std::fmt;

/// Domain-specific error types
#[derive(Debug, Clone)]
pub enum DomainError {
    /// Configuration domain error
    Configuration(String),
    /// Network domain error
    Network(String),
    /// Service domain error
    Service(String),
    /// Security domain error
    Security(String),
}

/// Domain-specific result type
pub type DomainResult<T> = Result<T, SongbirdError>;

/// Error context for domain operations
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Domain name
    pub domain: String,
    /// Operation being performed
    pub operation: String,
    /// Additional context data
    pub context: HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(domain: &str, operation: &str) -> Self {
        Self {
            domain: domain.to_string(),
            operation: operation.to_string(),
            context: HashMap::new(),
        }
    }
    
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
            DomainError::Network(msg) => write!(f, "Network error: {}", msg),
            DomainError::Service(msg) => write!(f, "Service error: {}", msg),
            DomainError::Security(msg) => write!(f, "Security error: {}", msg),
        }
    }
}

impl std::error::Error for DomainError {}

/// Helper function to create configuration errors with full context
pub fn config_error_detailed(
    message: &str,
    _field: Option<&str>,
    _context: Option<&str>,
    _suggestion: Option<&str>,
) -> SongbirdError {
    let error = SongbirdError::configuration_error(message);
    // Note: SongbirdError doesn't have with_field method in current implementation
    // This would be added in a future enhancement
    error
}

/// Helper function to create network errors with context
pub fn network_error_with_context(
    message: &str,
    _operation: Option<&str>,
    context: &ErrorContext,
) -> SongbirdError {
    let full_message = format!(
        "{} (domain: {}, operation: {})",
        message,
        context.domain,
        context.operation
    );
    SongbirdError::network_error(&full_message)
}

/// Helper function to create service errors with context
pub fn service_error_with_context(
    service: &str,
    message: &str,
    context: &ErrorContext,
) -> SongbirdError {
    let full_message = format!(
        "{} (domain: {}, operation: {})",
        message,
        context.domain,
        context.operation
    );
    SongbirdError::service_error(service, &full_message)
}

/// Generic error wrapper with context
pub fn with_error_context<T, F>(context: ErrorContext, operation: F) -> DomainResult<T>
where
    F: FnOnce() -> DomainResult<T>,
{
    operation().map_err(|e| {
        // Enhance the error with context information
        match e {
            SongbirdError::Config { message, .. } => {
                SongbirdError::configuration_error(&format!(
                    "{} (context: {} -> {})",
                    message, context.domain, context.operation
                ))
            }
            SongbirdError::Network { message, .. } => {
                SongbirdError::network_error(&format!(
                    "{} (context: {} -> {})",
                    message, context.domain, context.operation
                ))
            }
            other => other,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_error_display() {
        let config_err = DomainError::Configuration("Invalid setting".to_string());
        assert_eq!(format!("{}", config_err), "Configuration error: Invalid setting");

        let network_err = DomainError::Network("Connection failed".to_string());
        assert_eq!(format!("{}", network_err), "Network error: Connection failed");
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("config", "load")
            .with_context("file", "config.toml")
            .with_context("section", "database");
        
        assert_eq!(context.domain, "config");
        assert_eq!(context.operation, "load");
        assert_eq!(context.context.get("file"), Some(&"config.toml".to_string()));
        assert_eq!(context.context.get("section"), Some(&"database".to_string()));
    }

    #[test]
    fn test_config_error_detailed() {
        let error = config_error_detailed(
            "Invalid port number",
            Some("database.port"),
            Some("database configuration"),
            Some("Use a port between 1024 and 65535"),
        );
        
        // Should create a configuration error
        assert!(matches!(error, SongbirdError::Config { .. }));
    }

    #[test]
    fn test_network_error_with_context() {
        let context = ErrorContext::new("network", "connect");
        let error = network_error_with_context(
            "Connection timeout",
            Some("tcp_connect"),
            &context,
        );
        
        assert!(matches!(error, SongbirdError::Network { .. }));
    }

    #[test]
    fn test_service_error_with_context() {
        let context = ErrorContext::new("discovery", "register");
        let error = service_error_with_context(
            "registry",
            "Service registration failed",
            &context,
        );
        
        assert!(matches!(error, SongbirdError::Service { .. }));
    }
}
