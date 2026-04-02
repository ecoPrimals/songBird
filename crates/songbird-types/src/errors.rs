// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Error Types and Handling
//!
//! **CANONICAL**: Centralized error handling for the entire Songbird ecosystem
//! **AI-FIRST**: Enhanced with automation hints and rich context for AI agents

use serde::{Deserialize, Serialize};
use std::fmt;

/// **CANONICAL**: Result type for all Songbird operations
pub type SongbirdResult<T> = Result<T, SongbirdError>;

/// Automation hint for AI agents to handle errors automatically
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationHint {
    /// Retry with exponential backoff
    RetryExponential {
        /// Maximum number of retry attempts
        max_attempts: u32,
        /// Base delay in milliseconds (doubles each retry)
        base_delay_ms: u64,
    },
    /// Retry with fixed interval
    RetryFixed {
        /// Maximum number of retry attempts
        max_attempts: u32,
        /// Fixed interval between retries in milliseconds
        interval_ms: u64,
    },
    /// Fallback to alternative service
    FallbackService {
        /// List of alternative service endpoints
        alternatives: Vec<String>,
    },
    /// Escalate to human intervention
    EscalateHuman {
        /// Urgency level
        urgency: Urgency,
    },
    /// Safe to ignore (non-critical error)
    Ignore,
    /// Circuit breaker open - stop retrying temporarily
    CircuitOpen {
        /// Seconds to wait before retrying
        retry_after_secs: u64,
    },
}

/// Error urgency level for human escalation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Urgency {
    /// Critical - immediate attention required
    Critical,
    /// High priority
    High,
    /// Medium priority
    Medium,
    /// Low priority
    Low,
}

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

    /// RPC-related errors (tarpc, JSON-RPC, etc.)
    #[error("RPC error: {message}")]
    Rpc {
        /// Error message
        message: String,
        /// RPC method that failed
        method: Option<String>,
        /// Error code from RPC response (if applicable)
        code: Option<i64>,
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

    /// Discovery-related errors
    #[error("Discovery error: {message}")]
    Discovery {
        /// Error message
        message: String,
        /// Discovery backend that failed
        backend: Option<String>,
        /// Retry strategy hint
        retry_strategy: Option<String>,
    },

    /// Service registry errors
    #[error("Registry error: {message}")]
    Registry {
        /// Error message
        message: String,
        /// Service name involved
        service_name: Option<String>,
        /// Operation being performed
        operation: String,
    },

    /// Load balancing errors
    #[error("Load balancing error: {message}")]
    LoadBalancing {
        /// Error message
        message: String,
        /// Number of available instances
        available_instances: usize,
        /// Strategy being used
        strategy: String,
    },

    /// Protocol-related errors
    #[error("Protocol error: {message}")]
    Protocol {
        /// Error message
        message: String,
        /// Expected protocol version
        expected_version: Option<String>,
        /// Actual protocol version
        actual_version: Option<String>,
    },

    /// Metrics collection errors
    #[error("Metrics error: {message}")]
    Metrics {
        /// Error message
        message: String,
        /// Metric name involved
        metric_name: Option<String>,
        /// Operation being performed
        operation: String,
    },

    /// Event processing errors
    #[error("Event error: {message}")]
    Event {
        /// Error message
        message: String,
        /// Event type
        event_type: Option<String>,
        /// Processing stage where error occurred
        processing_stage: Option<String>,
    },

    /// Feature, integration, or code path not yet available
    #[error("Not implemented: {feature}")]
    NotImplemented {
        /// Short identifier for the missing capability (e.g. `consul_discovery`, `btsp_bidirectional`)
        feature: String,
        /// Optional human-readable detail
        detail: Option<String>,
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

    /// Create a new discovery error
    pub fn discovery(message: impl Into<String>) -> Self {
        Self::Discovery {
            message: message.into(),
            backend: None,
            retry_strategy: None,
        }
    }

    /// Create a new registry error
    pub fn registry(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::Registry {
            message: message.into(),
            service_name: None,
            operation: operation.into(),
        }
    }

    /// Create a new load balancing error
    pub fn load_balancing(message: impl Into<String>, strategy: impl Into<String>) -> Self {
        Self::LoadBalancing {
            message: message.into(),
            available_instances: 0,
            strategy: strategy.into(),
        }
    }

    /// Create a new protocol error
    pub fn protocol(message: impl Into<String>) -> Self {
        Self::Protocol {
            message: message.into(),
            expected_version: None,
            actual_version: None,
        }
    }

    /// Create a new timeout error (network timeout)
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Network {
            message: message.into(),
            interface: None,
            suggestion: Some("Increase timeout duration or check network connectivity".to_string()),
        }
    }

    /// Create a new metrics error
    pub fn metrics(message: impl Into<String>, operation: impl Into<String>) -> Self {
        Self::Metrics {
            message: message.into(),
            metric_name: None,
            operation: operation.into(),
        }
    }

    /// Create a new event error
    pub fn event(message: impl Into<String>) -> Self {
        Self::Event {
            message: message.into(),
            event_type: None,
            processing_stage: None,
        }
    }

    /// Not implemented: use when a deliberate stub or future phase has no implementation yet.
    pub fn not_implemented(feature: impl Into<String>) -> Self {
        Self::NotImplemented {
            feature: feature.into(),
            detail: None,
        }
    }

    /// Not implemented with optional remediation or context.
    pub fn not_implemented_with_detail(
        feature: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self::NotImplemented {
            feature: feature.into(),
            detail: Some(detail.into()),
        }
    }

    /// Create a new validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
            field: None,
            suggestion: None,
        }
    }

    /// Create a new serialization error
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::Serialization {
            format: None,
            message: message.into(),
            debug_info: None,
        }
    }

    /// Create a new RPC error
    pub fn rpc(message: impl Into<String>) -> Self {
        Self::Rpc {
            message: message.into(),
            method: None,
            code: None,
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

// Note: From implementations for external crates (reqwest, tokio)
// are implemented in their respective modules where those dependencies are available

#[cfg(test)]
#[path = "errors_tests.rs"]
mod tests;
