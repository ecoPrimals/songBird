//! Error handling for the universal primal system
//! 
//! This module provides comprehensive error types and utilities for handling
//! errors across all primal integrations.

use thiserror::Error;

/// Result type for primal operations
pub type PrimalResult<T> = Result<T, PrimalError>;

/// Universal error type for primal operations
#[derive(Error, Debug)]
pub enum PrimalError {
    /// Network or connection error
    #[error("Network error: {0}")]
    Network(String),
    
    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    /// Authorization error
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Request timeout error
    #[error("Request timeout: {0}")]
    Timeout(String),
    
    /// Primal service unavailable
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    /// Invalid request format
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    /// Internal server error
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),
    
    /// Resource already exists
    #[error("Resource already exists: {0}")]
    AlreadyExists(String),
    
    /// Insufficient permissions
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
    
    /// Data corruption or integrity error
    #[error("Data integrity error: {0}")]
    DataIntegrity(String),
    
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    /// Encryption/decryption error
    #[error("Encryption error: {0}")]
    Encryption(String),
    
    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    /// Discovery error
    #[error("Discovery error: {0}")]
    Discovery(String),
    
    /// Registry error
    #[error("Registry error: {0}")]
    Registry(String),
    
    /// Multi-instance error
    #[error("Multi-instance error: {0}")]
    MultiInstance(String),
    
    /// Port management error
    #[error("Port management error: {0}")]
    PortManagement(String),
    
    /// Context error
    #[error("Context error: {0}")]
    Context(String),
    
    /// Custom error for specific use cases
    #[error("Custom error: {0}")]
    Custom(String),
}

impl PrimalError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, PrimalError::Network(_) | PrimalError::Timeout(_) | PrimalError::ServiceUnavailable(_) | PrimalError::Internal(_) | PrimalError::RateLimit(_))
    }
    
    /// Get error category for logging and monitoring
    pub fn category(&self) -> &'static str {
        match self {
            PrimalError::Network(_) => "network",
            PrimalError::Authentication(_) => "authentication",
            PrimalError::Authorization(_) => "authorization",
            PrimalError::Configuration(_) => "configuration",
            PrimalError::Validation(_) => "validation",
            PrimalError::Timeout(_) => "timeout",
            PrimalError::ServiceUnavailable(_) => "service_unavailable",
            PrimalError::InvalidRequest(_) => "invalid_request",
            PrimalError::Internal(_) => "internal",
            PrimalError::RateLimit(_) => "rate_limit",
            PrimalError::NotFound(_) => "not_found",
            PrimalError::AlreadyExists(_) => "already_exists",
            PrimalError::InsufficientPermissions(_) => "insufficient_permissions",
            PrimalError::DataIntegrity(_) => "data_integrity",
            PrimalError::Serialization(_) => "serialization",
            PrimalError::Encryption(_) => "encryption",
            PrimalError::Protocol(_) => "protocol",
            PrimalError::Discovery(_) => "discovery",
            PrimalError::Registry(_) => "registry",
            PrimalError::MultiInstance(_) => "multi_instance",
            PrimalError::PortManagement(_) => "port_management",
            PrimalError::Context(_) => "context",
            PrimalError::Custom(_) => "custom",
        }
    }
    
    /// Get HTTP status code for this error
    pub fn http_status(&self) -> u16 {
        match self {
            PrimalError::Network(_) => 503,
            PrimalError::Authentication(_) => 401,
            PrimalError::Authorization(_) => 403,
            PrimalError::Configuration(_) => 500,
            PrimalError::Validation(_) => 400,
            PrimalError::Timeout(_) => 408,
            PrimalError::ServiceUnavailable(_) => 503,
            PrimalError::InvalidRequest(_) => 400,
            PrimalError::Internal(_) => 500,
            PrimalError::RateLimit(_) => 429,
            PrimalError::NotFound(_) => 404,
            PrimalError::AlreadyExists(_) => 409,
            PrimalError::InsufficientPermissions(_) => 403,
            PrimalError::DataIntegrity(_) => 500,
            PrimalError::Serialization(_) => 500,
            PrimalError::Encryption(_) => 500,
            PrimalError::Protocol(_) => 500,
            PrimalError::Discovery(_) => 500,
            PrimalError::Registry(_) => 500,
            PrimalError::MultiInstance(_) => 500,
            PrimalError::PortManagement(_) => 500,
            PrimalError::Context(_) => 400,
            PrimalError::Custom(_) => 500,
        }
    }
}

// Implement From trait for common error types
impl From<reqwest::Error> for PrimalError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            PrimalError::Timeout(error.to_string())
        } else if error.is_connect() {
            PrimalError::Network(error.to_string())
        } else {
            PrimalError::Internal(error.to_string())
        }
    }
}

impl From<serde_json::Error> for PrimalError {
    fn from(error: serde_json::Error) -> Self {
        PrimalError::Serialization(error.to_string())
    }
} 