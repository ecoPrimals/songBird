/// # Unified Songbird Error System
///
/// This module provides the core unified error type for Songbird,
/// with rich context and AI automation hints.
use crate::ai_first::{AIErrorCategory, AIFirstError, BackoffType, ErrorSeverity, RetryStrategy};
use crate::SongbirdResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Songbird's unified error type - implements ecosystem standards
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum SongbirdError {
    /// Network-related errors
    #[error("Network Error: {message}")]
    Network {
        /// Error message
        message: String,
        /// Operation that failed (if known)
        operation: Option<String>,
        /// Suggestion for fixing the error
        suggestion: Option<String>,
    },

    /// Network detection errors (for compatibility)
    #[error("Network Detection Error: {message}")]
    NetworkDetection {
        /// Error message
        message: String,
        /// Network interface that failed (if known)
        interface: Option<String>,
        /// Suggestion for fixing the error
        suggestion: Option<String>,
    },

    /// Configuration and validation errors
    #[error("Configuration Error: {message}")]
    Config {
        /// Configuration field name (if known)
        field: Option<String>,
        /// Error message
        message: String,
        /// Context where the error occurred
        context: Option<String>,
        /// Suggestion for fixing the error
        suggestion: Option<String>,
    },

    /// Service discovery and routing errors
    #[error("Service Error: {service} - {message}")]
    Service {
        /// Service name
        service: String,
        /// Error message
        message: String,
        /// Alternative services that could be used
        suggested_alternatives: Vec<String>,
        /// Recovery actions that can be taken
        recovery_actions: Vec<String>,
    },

    /// Communication errors
    #[error("Communication Error: {0}")]
    Communication(String),

    /// IO-related errors
    #[error("IO Error: {0}")]
    Io(String),

    /// Protocol-related errors
    #[error("Protocol Error: {0}")]
    Protocol(String),

    /// Security and authentication errors
    #[error("Security Error: {operation} - {message}")]
    Security {
        /// Security operation that failed
        operation: String,
        /// Error message
        message: String,
        /// Authentication provider (if applicable)
        provider: Option<String>,
        /// Required security level
        required_level: Option<String>,
    },

    /// Discovery service errors
    #[error("Discovery Error: {0}")]
    Discovery(String),

    /// Authentication errors
    #[error("Authentication Error: {message}")]
    Authentication {
        /// Error message
        message: String,
        /// Authentication provider
        provider: Option<String>,
        /// Required permission or scope
        required_permission: Option<String>,
    },

    /// Gaming-related errors  
    #[error("Gaming Error: {message}")]
    Gaming {
        /// Error message
        message: String,
        /// Game or protocol that failed
        game: Option<String>,
        /// Player count when error occurred
        player_count: Option<u32>,
    },

    /// Validation errors
    #[error("Validation Error: {message}")]
    Validation {
        /// Error message
        message: String,
        /// Field that failed validation
        field: Option<String>,
        /// Expected value or format
        expected: Option<String>,
    },

    /// Not found errors
    #[error("Not Found Error: {0}")]
    NotFound(String),

    /// Load balancer errors
    #[error("Load Balancer Error: {message}")]
    LoadBalancer {
        /// Error message
        message: String,
        /// Strategy that failed
        strategy: Option<String>,
    },

    /// Rate limiting errors
    #[error("Rate Limit Exceeded: {0}")]
    RateLimitExceeded(String),

    /// Circuit breaker errors
    #[error("Circuit Breaker Open: {0}")]
    CircuitBreakerOpen(String),

    /// Resource exhaustion errors
    #[error("Resource Error: {message}")]
    Resource {
        /// Error message
        message: String,
        /// Resource type (memory, disk, network, etc.)
        resource_type: Option<String>,
        /// Current usage when error occurred
        usage: Option<String>,
    },

    /// Storage unavailable errors
    #[error("Storage Unavailable: {0}")]
    StorageUnavailable(String),

    /// Configuration not found errors
    #[error("Config Not Found: {0}")]
    ConfigNotFound(String),

    /// Deployment errors
    #[error("Deployment Error: {0}")]
    Deployment(String),

    /// Plugin errors
    #[error("Plugin Not Found: {0}")]
    PluginNotFound(String),

    /// Unknown/generic errors
    #[error("Unknown Error: {0}")]
    Unknown(String),

    /// Federation-related errors
    #[error("Federation Error: {service} - {message}")]
    Federation {
        /// Federation service name
        service: String,
        /// Error message
        message: String,
        /// Federation peer information
        peer: Option<String>,
        /// Suggested recovery actions
        recovery_actions: Vec<String>,
    },

    /// Internal system errors
    #[error("Internal Error: {message}")]
    Internal {
        /// Error message
        message: String,
        /// Component where the error occurred
        component: Option<String>,
        /// Error code for debugging
        error_code: Option<String>,
        /// Debug information
        debug_info: Option<String>,
    },

    /// AI processing and automation errors
    #[error("AI Processing Error: {message}")]
    AIProcessing {
        /// Error message
        message: String,
        /// AI model or service that failed
        model: Option<String>,
        /// Confidence in the error diagnosis
        confidence: Option<f64>,
    },

    /// Operation and business logic errors
    #[error("Operation Error: {message}")]
    Operation {
        /// Error message
        message: String,
        /// Operation that failed
        operation: Option<String>,
        /// Suggestion for fixing the error
        suggestion: Option<String>,
    },
}

impl SongbirdError {
    /// Create an operation error
    pub fn operation_error(message: impl Into<String>) -> Self {
        SongbirdError::Operation {
            message: message.into(),
            operation: None,
            suggestion: None,
        }
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        SongbirdError::Internal {
            message: message.into(),
            component: None,
            error_code: None,
            debug_info: None,
        }
    }

    /// Create a validation error
    pub fn validation_error(message: impl Into<String>) -> Self {
        SongbirdError::Validation {
            message: message.into(),
            field: None,
            expected: None,
        }
    }

    /// Create a resource error
    pub fn resource_error(message: impl Into<String>) -> Self {
        SongbirdError::Resource {
            message: message.into(),
            resource_type: None,
            usage: None,
        }
    }

    /// Create an AI processing error
    pub fn ai_processing_error(message: impl Into<String>) -> Self {
        SongbirdError::AIProcessing {
            message: message.into(),
            model: None,
            confidence: None,
        }
    }

    /// Create a service error
    pub fn service_error(service: impl Into<String>, message: impl Into<String>) -> Self {
        SongbirdError::Service {
            service: service.into(),
            message: message.into(),
            suggested_alternatives: Vec::new(),
            recovery_actions: Vec::new(),
        }
    }

    /// Create a configuration error (compatibility method)
    pub fn configuration_error(message: impl Into<String>) -> Self {
        SongbirdError::configuration(message.into())
    }

    /// Create a network error (compatibility method)
    pub fn network_error(message: impl Into<String>) -> Self {
        SongbirdError::Network {
            message: message.into(),
            operation: None,
            suggestion: None,
        }
    }

    /// Create a gaming error
    pub fn gaming_error(message: impl Into<String>) -> Self {
        SongbirdError::Gaming {
            message: message.into(),
            game: None,
            player_count: None,
        }
    }

    /// Create an execution error (backward compatibility)
    pub fn execution_error(message: impl Into<String>) -> Self {
        SongbirdError::Operation {
            operation: Some("execution".to_string()),
            message: message.into(),
            suggestion: Some("Check operation parameters and system state".to_string()),
        }
    }

    /// Create an IO error (backward compatibility)
    pub fn io_error(message: impl Into<String>) -> Self {
        SongbirdError::Network {
            message: message.into(),
            operation: Some("io_operation".to_string()),
            suggestion: Some("Check network connectivity and system resources".to_string()),
        }
    }

    /// Create a security error (backward compatibility)
    pub fn security_error(message: impl Into<String>) -> Self {
        SongbirdError::Security {
            operation: "security_check".to_string(),
            message: message.into(),
            provider: Some("default".to_string()),
            required_level: Some("medium".to_string()),
        }
    }

    /// Create a config field error (backward compatibility)
    pub fn config_field(message: impl Into<String>) -> Self {
        SongbirdError::configuration(message.into())
    }

    /// Create a rate limit error (backward compatibility)
    pub fn rate_limit_error(message: impl Into<String>) -> Self {
        SongbirdError::Operation {
            operation: Some("rate_limiting".to_string()),
            message: message.into(),
            suggestion: Some("Reduce request frequency and try again".to_string()),
        }
    }

    /// Create a resource exhausted error (backward compatibility)
    pub fn resource_exhausted_error(message: impl Into<String>) -> Self {
        SongbirdError::Operation {
            operation: Some("resource_allocation".to_string()),
            message: message.into(),
            suggestion: Some("Free up resources and try again".to_string()),
        }
    }

    /// Create a circuit breaker error (backward compatibility)
    pub fn circuit_breaker_error(message: impl Into<String>) -> Self {
        SongbirdError::Operation {
            operation: Some("circuit_breaker".to_string()),
            message: message.into(),
            suggestion: Some("Wait for circuit breaker to reset".to_string()),
        }
    }

    /// Create a serialization error (backward compatibility)
    pub fn serialization_error(message: impl Into<String>) -> Self {
        SongbirdError::Validation {
            message: message.into(),
            field: Some("serialization".to_string()),
            expected: Some("valid serializable data".to_string()),
        }
    }

    /// Create a runtime error (backward compatibility)
    pub fn runtime_error(message: impl Into<String>) -> Self {
        SongbirdError::Operation {
            operation: Some("runtime_operation".to_string()),
            message: message.into(),
            suggestion: Some("Check system state and retry".to_string()),
        }
    }

    /// Add context to any error
    #[must_use]
    pub fn with_context(mut self, context: &str) -> Self {
        match &mut self {
            SongbirdError::Config { context: ctx, .. } => {
                *ctx = Some(context.to_string());
            }
            SongbirdError::Operation { suggestion, .. } => {
                *suggestion = Some(format!("Context: {context}"));
            }
            SongbirdError::Internal { component, .. } => {
                *component = Some(context.to_string());
            }
            _ => {} // Other error types don't have context fields
        }
        self
    }

    /// Add suggestion to any error that supports it
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        match &mut self {
            SongbirdError::Network {
                suggestion: sug, ..
            }
            | SongbirdError::NetworkDetection {
                suggestion: sug, ..
            }
            | SongbirdError::Config {
                suggestion: sug, ..
            }
            | SongbirdError::Operation {
                suggestion: sug, ..
            } => {
                *sug = Some(suggestion.to_string());
            }
            _ => {} // Other error types don't have suggestion fields
        }
        self
    }

    /// Get error category for AI classification
    #[must_use]
    pub fn get_category(&self) -> AIErrorCategory {
        match self {
            SongbirdError::Network { .. } | SongbirdError::NetworkDetection { .. } => {
                AIErrorCategory::Network
            }
            SongbirdError::Config { .. } => AIErrorCategory::Configuration,
            SongbirdError::Validation { .. } => AIErrorCategory::Validation,
            SongbirdError::Authentication { .. } => AIErrorCategory::Authentication,
            SongbirdError::Resource { .. } => AIErrorCategory::Resource,
            SongbirdError::Operation { .. } => AIErrorCategory::Operation,
            SongbirdError::Internal { .. } => AIErrorCategory::Internal,
            _ => AIErrorCategory::Unknown,
        }
    }

    /// Get error category for AI classification
    #[must_use]
    pub fn category(&self) -> AIErrorCategory {
        match self {
            SongbirdError::Network { .. } | SongbirdError::NetworkDetection { .. } => {
                AIErrorCategory::NetworkFailure
            }
            SongbirdError::Config { .. } => AIErrorCategory::ConfigurationIssue,
            SongbirdError::Service { .. }
            | SongbirdError::Discovery(_)
            | SongbirdError::LoadBalancer { .. }
            | SongbirdError::CircuitBreakerOpen(_) => AIErrorCategory::DependencyFailure,
            SongbirdError::Communication(_) => AIErrorCategory::NetworkFailure,
            SongbirdError::Security { .. } | SongbirdError::Authentication { .. } => {
                AIErrorCategory::SecurityViolation
            }
            SongbirdError::RateLimitExceeded(_) => AIErrorCategory::RateLimiting,
            SongbirdError::Resource { .. } => AIErrorCategory::ResourceLimitation,
            _ => AIErrorCategory::Unknown,
        }
    }

    /// Get suggested retry strategy
    #[must_use]
    pub fn retry_strategy(&self) -> RetryStrategy {
        match self {
            SongbirdError::Network { .. } | SongbirdError::NetworkDetection { .. } => {
                RetryStrategy {
                    should_retry: true,
                    delay_ms: 1000,
                    max_attempts: 3,
                    backoff_strategy: BackoffType::Exponential { base: 2.0 },
                    retry_conditions: vec!["network_available".to_string()],
                    success_probability: 0.7,
                }
            }
            SongbirdError::Config { .. } => RetryStrategy {
                should_retry: false,
                delay_ms: 0,
                max_attempts: 0,
                backoff_strategy: BackoffType::Linear,
                retry_conditions: vec![],
                success_probability: 0.0,
            },
            SongbirdError::RateLimitExceeded(_) => RetryStrategy {
                should_retry: true,
                delay_ms: 5000,
                max_attempts: 5,
                backoff_strategy: BackoffType::Linear,
                retry_conditions: vec!["rate_limit_reset".to_string()],
                success_probability: 0.8,
            },
            SongbirdError::CircuitBreakerOpen(_) => RetryStrategy {
                should_retry: true,
                delay_ms: 10000,
                max_attempts: 2,
                backoff_strategy: BackoffType::Exponential { base: 2.0 },
                retry_conditions: vec!["circuit_breaker_closed".to_string()],
                success_probability: 0.5,
            },
            _ => RetryStrategy {
                should_retry: true,
                delay_ms: 500,
                max_attempts: 3,
                backoff_strategy: BackoffType::Exponential { base: 2.0 },
                retry_conditions: vec!["service_available".to_string()],
                success_probability: 0.6,
            },
        }
    }

    /// Convert to AI-first error format
    #[must_use]
    pub fn to_ai_first_error(&self) -> AIFirstError {
        AIFirstError {
            code: format!("{self:?}").to_uppercase().replace(' ', "_"),
            message: self.to_string(),
            category: self.category(),
            retry_strategy: self.retry_strategy(),
            automation_hints: self.get_automation_hints(),
            severity: self.get_severity(),
            requires_human_intervention: self.requires_human_intervention(),
            context: HashMap::new(),
        }
    }

    /// Get automation hints for this error
    #[must_use]
    pub fn get_automation_hints(&self) -> Vec<String> {
        match self {
            SongbirdError::Network { suggestion, .. } => {
                let mut hints = vec!["check_network_connectivity".to_string()];
                if let Some(s) = suggestion {
                    hints.push(s.clone());
                }
                hints
            }
            SongbirdError::Config { suggestion, .. } => {
                let mut hints = vec!["validate_configuration".to_string()];
                if let Some(s) = suggestion {
                    hints.push(s.clone());
                }
                hints
            }
            SongbirdError::Service {
                recovery_actions, ..
            } => recovery_actions.clone(),
            _ => vec!["retry_operation".to_string()],
        }
    }

    /// Get error severity
    #[must_use]
    pub fn get_severity(&self) -> ErrorSeverity {
        match self {
            SongbirdError::Config { .. } => ErrorSeverity::Critical,
            SongbirdError::Security { .. } | SongbirdError::Authentication { .. } => {
                ErrorSeverity::High
            }
            SongbirdError::Network { .. } | SongbirdError::NetworkDetection { .. } => {
                ErrorSeverity::Medium
            }
            _ => ErrorSeverity::Low,
        }
    }

    /// Check if this error requires human intervention
    #[must_use]
    pub fn requires_human_intervention(&self) -> bool {
        matches!(
            self,
            SongbirdError::Config { .. } | SongbirdError::Security { .. }
        )
    }
}

/// Result type for Songbird operations - returns `SongbirdResponse`
/// Songbird result type alias for convenience
/// Uses `SongbirdResponse` wrapper for all successful operations
pub type SongbirdResult<T> = std::result::Result<SongbirdResponse<T>, SongbirdError>;

/// Standard Result type using unified `SongbirdError` - EVOLVED APPROACH
/// This follows standard Rust patterns but uses our rich error type
/// We use a different name to avoid conflicts with std::result::Result
pub type SongbirdStdResult<T> = std::result::Result<T, SongbirdError>;

/// Specialized result types for common patterns - all use SongbirdError
pub type NetworkResult<T> = std::result::Result<T, SongbirdError>;
pub type ConfigResult<T> = std::result::Result<T, SongbirdError>;
pub type ServiceResult<T> = std::result::Result<T, SongbirdError>;
pub type DiscoveryResult<T> = std::result::Result<T, SongbirdError>;
pub type FederationResult<T> = std::result::Result<T, SongbirdError>;

/// For cases where we need to be explicit about error types
pub type StdResult<T, E> = Result<T, E>;

/// Helper function to create successful `SongbirdResponse`
pub fn success<T>(data: T) -> SongbirdResponse<T> {
    SongbirdResponse::success(data)
}

/// Helper function to create successful `SongbirdResponse`
/// This is a convenience function that wraps data in a successful response
#[must_use]
pub fn success_result<T>(data: T) -> SongbirdResponse<T> {
    SongbirdResponse::success(data)
}

/// Helper function to create error `SongbirdResult`
///
/// # Errors
/// Always returns the provided error as an Err variant
pub fn error_result<T>(error: SongbirdError) -> SongbirdResult<T> {
    Err(error)
}

// ============================================================================
// FROM IMPLEMENTATIONS - Enable automatic error conversions
// ============================================================================

impl From<String> for SongbirdError {
    fn from(msg: String) -> Self {
        SongbirdError::Unknown(msg)
    }
}

impl From<&str> for SongbirdError {
    fn from(msg: &str) -> Self {
        SongbirdError::Unknown(msg.to_string())
    }
}

impl From<std::io::Error> for SongbirdError {
    fn from(err: std::io::Error) -> Self {
        SongbirdError::Network {
            message: format!("IO error: {err}"),
            operation: Some("io_operation".to_string()),
            suggestion: Some("Check file permissions and system resources".to_string()),
        }
    }
}

impl From<serde_json::Error> for SongbirdError {
    fn from(err: serde_json::Error) -> Self {
        SongbirdError::validation_error(format!("JSON parsing error: {err}"))
    }
}

impl From<tokio::time::error::Elapsed> for SongbirdError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        SongbirdError::Network {
            message: format!("Operation timeout: {err}"),
            operation: Some("async operation".to_string()),
            suggestion: Some(
                "Increase timeout duration or check operation performance".to_string(),
            ),
        }
    }
}

impl From<std::net::AddrParseError> for SongbirdError {
    fn from(err: std::net::AddrParseError) -> Self {
        SongbirdError::Network {
            message: format!("Address parsing error: {err}"),
            operation: Some("address_parsing".to_string()),
            suggestion: Some("Check IP address and port format".to_string()),
        }
    }
}

// Note: reqwest::Error From implementation would go here if reqwest is added as dependency

// Note: From implementations for serde_yaml::Error
// are not included due to dependency constraints.
// These should be handled with manual error conversion in the calling code.
