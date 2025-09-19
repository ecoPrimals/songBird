//! # Songbird Errors
//!
//! Comprehensive error handling system providing structured error types, detailed
//! error information, and recovery guidance for the Songbird ecosystem.
//!
//! ## Features
//!
//! - **Structured Error Types**: Hierarchical error classification system
//! - **Detailed Error Information**: Context-rich error messages with suggestions
//! - **Error Recovery**: Built-in recovery strategies and guidance
//! - **Validation Errors**: Specialized validation error handling
//! - **Network Error Handling**: Network-specific error types and recovery
//! - **Security Error Handling**: Security-specific error classification
//! - **Tracing Integration**: Seamless integration with tracing and logging
//! - **Serialization Support**: Error serialization for API responses
//!
//! ## Architecture
//!
//! The errors crate is organized into focused modules:
//!
//! - `songbird_errors`: Core error types and classifications
//! - `validation`: Validation-specific error handling
//!
//! ## Usage
//!
//! ```rust,no_run
//! use songbird_errors::{SongbirdError, Result, ValidationError, DiscoveryError};
//!
//! fn example_function() -> Result<String> {
//!     // Function that might fail
//!     validate_input("example")?;
//!     Ok("Success".to_string())
//! }
//!
//! fn validate_input(input: &str) -> Result<()> {
//!     if input.is_empty() {
//!         return Err(SongbirdError::ValidationError(
//!             ValidationError::EmptyInput {
//!                 field: "input".to_string(),
//!                 suggestion: "Provide a non-empty input value".to_string(),
//!             }
//!         ));
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Error Categories
//!
//! Errors are organized into logical categories:
//!
//! ### Core Errors
//! - **Configuration**: Configuration loading and validation errors
//! - **Network**: Network connectivity and protocol errors
//! - **Security**: Authentication, authorization, and security errors
//! - **IO**: File system and input/output errors
//! - **Serialization**: Data serialization and deserialization errors
//!
//! ### Domain-Specific Errors
//! - **Discovery**: Service discovery and registration errors
//! - **Orchestration**: Service orchestration and management errors
//! - **Gaming**: Gaming-specific network and protocol errors
//! - **Federation**: Multi-region federation and coordination errors
//! - **Observability**: Monitoring and metrics collection errors
//!
//! ### Validation Errors
//! - **Input Validation**: User input validation errors
//! - **Schema Validation**: Data structure validation errors
//! - **Constraint Validation**: Business rule and constraint violations
//! - **Type Validation**: Type conversion and format validation errors
//!
//! ## Error Recovery
//!
//! Each error type includes recovery guidance:
//!
//! - **Retry Strategies**: Automatic retry with exponential backoff
//! - **Fallback Options**: Alternative approaches when primary fails
//! - **User Guidance**: Clear instructions for manual intervention
//! - **System Recommendations**: Suggested system-level fixes
//!
//! ## Error Reporting
//!
//! Comprehensive error reporting features:
//!
//! - **Structured Logging**: Structured error logging with context
//! - **Error Metrics**: Error rate and pattern metrics
//! - **Alert Integration**: Integration with alerting systems
//! - **Debug Information**: Detailed debug information for development
//!
//! ## Error Handling Best Practices
//!
//! - Use specific error types rather than generic errors
//! - Provide context and suggestions for recovery
//! - Log errors appropriately based on severity
//! - Handle errors gracefully with user-friendly messages
//! - Include debug information for troubleshooting
//!
//! ## Serialization
//!
//! Errors can be serialized for API responses:
//!
//! ```rust,no_run
//! use songbird_errors::SongbirdError;
//! use serde_json;
//!
//! fn serialize_error(error: &SongbirdError) -> String {
//!     serde_json::to_string(error).unwrap_or_else(|_| "Unknown error".to_string())
//! }
//! ```

pub mod validation;

// Re-export canonical error types from songbird-types
pub use songbird_types::errors::*;
pub use validation::*;

/// Helper function to create config errors with proper context
pub fn config_error(
    message: &str,
    field: Option<&str>,
    _context: Option<&str>,
    suggestion: Option<&str>,
) -> SongbirdError {
    let mut error = SongbirdError::configuration(message);
    if let SongbirdError::Configuration {
        field: f,
        suggestion: s,
        ..
    } = &mut error
    {
        *f = field.map(|field_str| field_str.to_string());
        *s = suggestion.map(|sug_str| sug_str.to_string());
    }
    error
}

/// Helper function to create config errors with just message and field
pub fn simple_config_error(message: &str, field: Option<&str>) -> SongbirdError {
    config_error(
        message,
        field,
        Some("Configuration validation"),
        Some("Check configuration settings and values"),
    )
}

/// Helper function to create discovery errors (using service error as closest match)
pub fn discovery_error(
    message: &str,
    service: Option<&str>,
    _timeout: Option<u64>,
    _suggestion: Option<&str>,
) -> SongbirdError {
    SongbirdError::service(
        service.unwrap_or("discovery"),
        format!("Discovery error: {}", message),
    )
}

/// Helper function to create simple discovery errors
pub fn simple_discovery_error(message: &str, service: Option<&str>) -> SongbirdError {
    discovery_error(
        message,
        service,
        None,
        Some("Check service configuration and network connectivity"),
    )
}

/// Helper function to create service errors
pub fn service_error(
    service: &str,
    message: &str,
    _status: Option<&str>,
    _suggestion: Option<&str>,
) -> SongbirdError {
    SongbirdError::service(service, message)
}

/// Helper function to create load balancer errors (using service error as closest match)
pub fn load_balancer_error(
    message: &str,
    backend: Option<&str>,
    _suggestion: Option<&str>,
) -> SongbirdError {
    SongbirdError::service(
        backend.unwrap_or("load_balancer"),
        format!("Load balancer error: {}", message),
    )
}
