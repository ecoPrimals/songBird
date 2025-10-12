//! Error Types and Handling
//!
//! **CANONICAL**: Centralized error handling for the entire Songbird ecosystem

use serde::{Deserialize, Serialize};
use std::fmt;

/// **CANONICAL**: Result type for all Songbird operations
pub type SongbirdResult<T> = Result<T, SongbirdError>;

/// **CANONICAL**: Main error type for Songbird operations
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum SongbirdError {
    /// Configuration-related errors
    #[error("Configuration error: {message}")]
    Configuration {
        /// Error message
        message: String,
        /// Field that caused the error
        field: Option<String>,
        /// Suggested fix
        suggestion: Option<String>,
    },

    /// Network-related errors
    #[error("Network error: {message}")]
    Network {
        /// Error message
        message: String,
        /// Network interface involved
        interface: Option<String>,
        /// Suggested remediation
        suggestion: Option<String>,
    },

    /// Security-related errors
    #[error("Security error: {0}")]
    Security(SecurityError),

    /// Service-related errors
    #[error("Service error in {service}: {message}")]
    Service {
        /// Service name
        service: String,
        /// Error message
        message: String,
        /// Suggested alternatives
        suggested_alternatives: Vec<String>,
        /// Recovery actions
        recovery_actions: Vec<String>,
    },

    /// Serialization errors
    #[error("Serialization error: {message}")]
    Serialization {
        /// Format being serialized/deserialized
        format: Option<String>,
        /// Error message
        message: String,
        /// Debug information
        debug_info: Option<String>,
    },

    /// Async runtime errors
    #[error("Async runtime error: {message}")]
    Runtime {
        /// Error message
        message: String,
        /// Component that failed
        component: Option<String>,
        /// Debug information
        debug_info: Option<String>,
    },

    /// Validation errors
    #[error("Validation error: {message}")]
    Validation {
        /// Error message
        message: String,
        /// Field that failed validation
        field: Option<String>,
        /// Suggested fix
        suggestion: Option<String>,
    },
}

/// Security-specific error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityError {
    /// Error message
    pub message: String,
    /// Operation being performed
    pub operation: Option<String>,
    /// Required permission
    pub required_permission: Option<String>,
    /// Additional context
    pub context: Option<String>,
    /// Suggested remediation
    pub remediation: Option<String>,
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Security error: {}", self.message)
    }
}

impl std::error::Error for SecurityError {}

impl SongbirdError {
    /// Create a new configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
            field: None,
            suggestion: None,
        }
    }

    /// Create a new network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
            interface: None,
            suggestion: None,
        }
    }

    /// Create a new security error
    pub fn security(message: impl Into<String>) -> Self {
        Self::Security(SecurityError {
            message: message.into(),
            operation: None,
            required_permission: None,
            context: None,
            remediation: None,
        })
    }

    /// Create a new service error
    pub fn service(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Service {
            service: service.into(),
            message: message.into(),
            suggested_alternatives: Vec::new(),
            recovery_actions: Vec::new(),
        }
    }

    /// Add context to the error
    pub fn with_context(&mut self, context: impl Into<String>) -> &mut Self {
        match self {
            Self::Security(sec) => sec.context = Some(context.into()),
            Self::Configuration {
                suggestion,
                ..
            }
            | Self::Network {
                suggestion,
                ..
            } => {
                *suggestion = Some(context.into());
            }
            _ => {} // Other variants don't support context
        }
        self
    }

    /// Add suggestion to the error
    pub fn with_suggestion(&mut self, suggestion: impl Into<String>) -> &mut Self {
        match self {
            Self::Security(sec) => sec.remediation = Some(suggestion.into()),
            Self::Configuration {
                suggestion: s,
                ..
            }
            | Self::Network {
                suggestion: s,
                ..
            }
            | Self::Validation {
                suggestion: s,
                ..
            } => {
                *s = Some(suggestion.into());
            }
            _ => {} // Other variants don't have suggestion fields
        }
        self
    }
}

// Implement From traits for common error types
impl From<serde_json::Error> for SongbirdError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization {
            format: Some("JSON".to_string()),
            message: format!("JSON processing error: {error}"),
            debug_info: None,
        }
    }
}

// Note: tokio integration would be added when tokio dependency is available
// impl From<tokio::task::JoinError> for SongbirdError {
//     fn from(error: tokio::task::JoinError) -> Self {
//         Self::Runtime {
//             message: format!("Task join error: {}", error),
//             component: Some("tokio".to_string(),
//             debug_info: None,
//         }
//     }
// }
// }

impl From<std::net::AddrParseError> for SongbirdError {
    fn from(error: std::net::AddrParseError) -> Self {
        Self::Network {
            message: format!("Address parse error: {error}"),
            interface: None,
            suggestion: Some("Check the address format".to_string()),
        }
    }
}

impl From<&str> for SongbirdError {
    fn from(msg: &str) -> Self {
        Self::Configuration {
            message: msg.to_string(),
            field: None,
            suggestion: None,
        }
    }
}

impl From<String> for SongbirdError {
    fn from(msg: String) -> Self {
        Self::Configuration {
            message: msg,
            field: None,
            suggestion: None,
        }
    }
}

impl From<std::io::Error> for SongbirdError {
    fn from(error: std::io::Error) -> Self {
        Self::Network {
            message: format!("IO error: {error}"),
            interface: None,
            suggestion: Some("Check file permissions and network connectivity".to_string()),
        }
    }
}

// Note: From implementations for external crates (serde_yaml, reqwest, tokio)
// are implemented in their respective modules where those dependencies are available

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_error_creation() {
        let error = SecurityError {
            message: "Authentication failed".to_string(),
            operation: Some("login".to_string()),
            required_permission: Some("user".to_string()),
            context: Some("authentication".to_string()),
            remediation: None,
        };

        assert_eq!(error.operation, Some("login".to_string()));
        assert!(error.to_string().contains("Security error"));
    }

    #[test]
    fn test_songbird_error_variants() {
        // Test Network error
        let network_error = SongbirdError::Network {
            message: "Connection timeout".to_string(),
            interface: Some("eth0".to_string()),
            suggestion: None,
        };

        // Test Security error
        let security_error = SongbirdError::Security(SecurityError {
            message: "Unauthorized access".to_string(),
            operation: Some("read".to_string()),
            required_permission: Some("admin".to_string()),
            context: None,
            remediation: None,
        });

        assert!(network_error.to_string().contains("Network error"));
        assert!(security_error.to_string().contains("Unauthorized access"));
    }

    #[test]
    fn test_error_serialization() {
        let error = SongbirdError::Security(SecurityError {
            message: "Invalid token".to_string(),
            operation: Some("authenticate".to_string()),
            required_permission: Some("test_permission".to_string()),
            context: None,
            remediation: None,
        });

        // Test code: unwrap is acceptable for testing serialization
        let serialized = serde_json::to_string(&error).expect("test serialization should succeed");
        let deserialized: SongbirdError = serde_json::from_str(&serialized).expect("test deserialization should succeed");

        assert_eq!(deserialized.to_string(), error.to_string());
    }

    #[test]
    fn test_service_error_with_recovery() {
        let error = SongbirdError::Service {
            service: "database".to_string(),
            message: "Connection failed".to_string(),
            suggested_alternatives: vec!["backup-db".to_string(), "cache".to_string()],
            recovery_actions: vec!["retry".to_string(), "fallback".to_string()],
        };

        match error {
            SongbirdError::Service {
                suggested_alternatives,
                recovery_actions,
                ..
            } => {
                assert_eq!(suggested_alternatives.len(), 2);
                assert_eq!(recovery_actions.len(), 2);
                assert!(suggested_alternatives.contains(&"backup-db".to_string()));
            }
            _ => panic!("Expected Service error"),
        }
    }

    #[test]
    fn test_error_context_and_suggestions() {
        let mut security_error = SongbirdError::security("Invalid token");
        security_error.with_context("authentication");
        security_error.with_suggestion("Check network connectivity");

        let mut network_error = SongbirdError::network("Connection failed");
        network_error.with_suggestion("Check network connectivity");

        assert!(security_error.to_string().contains("Invalid token"));
        assert!(network_error.to_string().contains("Connection failed"));
    }
}
