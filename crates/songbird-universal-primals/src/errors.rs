//! Error types for universal primals

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Error type for universal primal operations
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum PrimalError {
    /// Network connection errors
    #[error("Network error: {message}")]
    Network { message: String },

    /// Authentication and security errors
    #[error("Security error: {message}")]
    Security { message: String },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Configuration { message: String },

    /// Service unavailable or timeout
    #[error("Service unavailable: {message}")]
    ServiceUnavailable { message: String },

    /// Service not found error
    #[error("Service not found: {service_id}")]
    ServiceNotFound { service_id: Uuid },

    /// Discovery-related errors
    #[error("Discovery error: {message}")]
    Discovery { message: String },

    /// Serialization/deserialization errors
    #[error("Serialization error: {message}")]
    Serialization { message: String },

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation { message: String },

    /// Integration errors
    #[error("Integration error: {message}")]
    Integration { message: String },

    /// Resource exhaustion errors
    #[error("Resource exhausted: {message}")]
    ResourceExhausted { message: String },

    /// Generic internal errors
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl PrimalError {
    /// Create a network error
    pub fn network_error(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
        }
    }

    /// Create a security error
    pub fn security_error(message: impl Into<String>) -> Self {
        Self::Security {
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create a discovery error
    pub fn discovery_error(message: impl Into<String>) -> Self {
        Self::Discovery {
            message: message.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::Serialization {
            message: message.into(),
        }
    }

    /// Create a service not found error
    pub fn service_not_found(service_id: Uuid) -> Self {
        Self::ServiceNotFound { service_id }
    }
}

// From implementations for common error types
impl From<serde_json::Error> for PrimalError {
    fn from(err: serde_json::Error) -> Self {
        Self::serialization_error(err.to_string())
    }
}

impl From<reqwest::Error> for PrimalError {
    fn from(err: reqwest::Error) -> Self {
        Self::network_error(err.to_string())
    }
}

impl From<std::io::Error> for PrimalError {
    fn from(err: std::io::Error) -> Self {
        Self::network_error(err.to_string())
    }
}

impl From<songbird_errors::SongbirdError> for PrimalError {
    fn from(err: songbird_errors::SongbirdError) -> Self {
        match err {
            songbird_errors::SongbirdError::Network { message, .. } => Self::network_error(message),
            songbird_errors::SongbirdError::Configuration { message, .. } => {
                Self::configuration_error(message)
            }
            songbird_errors::SongbirdError::Security(sec_err) => {
                Self::security_error(sec_err.message.clone())
            }
            songbird_errors::SongbirdError::Validation { message, .. } => {
                Self::validation_error(message)
            }
            _ => Self::Internal {
                message: err.to_string(),
            },
        }
    }
}

/// Result type for primal operations
pub type PrimalResult<T> = Result<T, PrimalError>;
