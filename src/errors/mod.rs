//! Error handling for Songbird Orchestrator

use serde::{Deserialize, Serialize};
use std::fmt;

/// Result type alias for Songbird operations
pub type Result<T> = std::result::Result<T, SongbirdError>;

/// Main error type for Songbird Orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SongbirdError {
    /// Service-related errors
    Service {
        message: String,
    },

    /// Network communication errors
    Network {
        message: String,
    },

    /// Discovery system errors
    Discovery {
        message: String,
    },

    /// Configuration errors
    Configuration {
        field: String,
        message: String,
    },

    /// Load balancer errors
    LoadBalancer {
        message: String,
    },

    /// Serialization/deserialization errors
    Serialization {
        message: String,
    },

    /// Internal system errors
    Internal {
        message: String,
    },

    /// Validation errors
    ValidationFailed {
        field: String,
        issue: String,
    },

    /// Rate limiting errors (for HPC robustness)
    RateLimit {
        message: String,
    },

    /// Circuit breaker errors (for unreliable consumer hardware)
    CircuitBreakerOpen {
        message: String,
        service_id: String,
    },

    /// Health check errors (for tower monitoring)
    HealthCheck {
        message: String,
    },

    /// Security-related errors
    SecurityError(String),
}

impl fmt::Display for SongbirdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SongbirdError::Service { message } => {
                write!(f, "Service error: {}", message)
            }
            SongbirdError::Network { message } => {
                write!(f, "Network error: {}", message)
            }
            SongbirdError::Discovery { message } => {
                write!(f, "Discovery error: {}", message)
            }
            SongbirdError::Configuration { field, message } => {
                write!(f, "Configuration error in '{}': {}", field, message)
            }
            SongbirdError::LoadBalancer { message } => {
                write!(f, "Load balancer error: {}", message)
            }
            SongbirdError::Serialization { message } => {
                write!(f, "Serialization error: {}", message)
            }
            SongbirdError::Internal { message } => {
                write!(f, "Internal error: {}", message)
            }
            SongbirdError::ValidationFailed { field, issue } => {
                write!(f, "Validation failed for '{}': {}", field, issue)
            }
            SongbirdError::RateLimit { message } => {
                write!(f, "Rate limit error: {}", message)
            }
            SongbirdError::CircuitBreakerOpen { message, service_id } => {
                write!(f, "Circuit breaker open for '{}': {}", service_id, message)
            }
            SongbirdError::HealthCheck { message } => {
                write!(f, "Health check error: {}", message)
            }
            SongbirdError::SecurityError(message) => {
                write!(f, "Security error: {}", message)
            }
        }
    }
}

impl std::error::Error for SongbirdError {}

// Helper methods for common error patterns in HPC system
impl SongbirdError {
    /// Create a service error with service ID context
    pub fn service_error(service_id: &str, message: String) -> Self {
        Self::Service {
            message: format!("[{}] {}", service_id, message),
        }
    }

    /// Create a health check failure error
    pub fn health_check_failed(service_id: &str, message: String) -> Self {
        Self::HealthCheck {
            message: format!("[{}] {}", service_id, message),
        }
    }

    /// Create a configuration error with simple message
    pub fn configuration_error(message: String) -> Self {
        Self::Configuration {
            field: "general".to_string(),
            message,
        }
    }
}

// Implement From traits for common error types
impl From<serde_json::Error> for SongbirdError {
    fn from(err: serde_json::Error) -> Self {
        SongbirdError::Serialization {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for SongbirdError {
    fn from(err: std::io::Error) -> Self {
        SongbirdError::Network {
            message: err.to_string(),
        }
    }
}

impl From<reqwest::Error> for SongbirdError {
    fn from(err: reqwest::Error) -> Self {
        SongbirdError::Network {
            message: err.to_string(),
        }
    }
}

impl From<std::net::AddrParseError> for SongbirdError {
    fn from(err: std::net::AddrParseError) -> Self {
        SongbirdError::Configuration {
            field: "address".to_string(),
            message: format!("Invalid address format: {}", err),
        }
    }
}

impl From<Box<dyn std::error::Error>> for SongbirdError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        SongbirdError::Internal {
            message: err.to_string(),
        }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for SongbirdError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        SongbirdError::Internal {
            message: err.to_string(),
        }
    }
} 