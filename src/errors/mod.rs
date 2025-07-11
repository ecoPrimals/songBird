// Module imports
//! Error handling for Songbird Orchestrator

use std::error::Error as StdError;
use std::fmt;
pub mod validation;
/// Result type alias for Songbird operations
pub type Result<T> = std::result::Result<T, SongbirdError>;
/// Main error type for Songbird services
#[derive(Debug, Clone)]
pub enum SongbirdError {
    /// Configuration related errors
    Config {
        message: String,
        field: Option<String>,
    },
    /// Configuration related errors (alias)
    Configuration { field: String, message: String },
    /// Network communication errors
    Network {
        service: String,
        message: String,
        details: Option<String>,
    },
    /// Communication errors
    Communication(String),
    /// Service discovery errors
    Discovery {
        message: String,
        service: Option<String>,
    },
    /// Authentication and authorization errors
    Auth {
        message: String,
        user: Option<String>,
    },
    /// Authentication errors (alias)
    Authentication { provider: String, message: String },
    /// Gaming protocol errors
    Gaming {
        message: String,
        protocol: Option<String>,
    },
    /// Security related errors
    Security {
        message: String,
        context: Option<String>,
    },
    /// Protocol specific errors
    Protocol { protocol: String, message: String },
    /// Service errors
    Service { service: String, message: String },
    /// Validation errors
    Validation { field: String, message: String },
    /// Resource not found errors
    NotFound { resource: String, message: String },
    /// IO errors
    Io { message: String },
    /// Load balancer errors
    LoadBalancer { message: String },
    /// Tunnel creation errors
    TunnelCreation(String),
    /// Encryption errors
    EncryptionFailed(String),
    /// Decryption errors
    DecryptionFailed(String),
    /// Network detection errors
    NetworkDetection(String),
    /// Unsupported channel type errors
    UnsupportedChannelType,
    /// Deployment errors
    Deployment { service: String, message: String },
    /// Circuit breaker open errors
    CircuitBreakerOpen { message: String },
    /// Circuit breaker failure errors
    CircuitBreakerFailure { message: String },
    /// Retry exhausted errors
    RetryExhausted { attempts: u32, last_error: String },
    /// Rate limit exceeded errors
    RateLimitExceeded { message: String },
    /// Execution failed errors
    ExecutionFailed { message: String },
    /// Plugin composition failed errors
    CompositionFailed(String),
}
impl fmt::Display for SongbirdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SongbirdError::Config { message, field } => {
                if let Some(field) = field {
                    write!(f, "Configuration error in field '{}': {}", field, message)
                } else {
                    write!(f, "Configuration error: {}", message)
                }
            }
            SongbirdError::Configuration { field, message } => {
                write!(f, "Configuration error in field '{}': {}", field, message)
            }
            SongbirdError::Network {
                service,
                message,
                details,
            } => {
                if let Some(details) = details {
                    write!(
                        f,
                        "Network error in service '{}': {}: {}",
                        service, message, details
                    )
                } else {
                    write!(f, "Network error in service '{}': {}", service, message)
                }
            }
            SongbirdError::Communication(message) => write!(f, "Communication error: {}", message),
            SongbirdError::Discovery { message, service } => {
                if let Some(service) = service {
                    write!(f, "Discovery error for service '{}': {}", service, message)
                } else {
                    write!(f, "Discovery error: {}", message)
                }
            }
            SongbirdError::Auth { message, user } => {
                if let Some(user) = user {
                    write!(f, "Authentication error for user '{}': {}", user, message)
                } else {
                    write!(f, "Authentication error: {}", message)
                }
            }
            SongbirdError::Authentication { provider, message } => {
                write!(
                    f,
                    "Authentication error from provider '{}': {}",
                    provider, message
                )
            }
            SongbirdError::Gaming { message, protocol } => {
                if let Some(protocol) = protocol {
                    write!(f, "Gaming error for protocol '{}': {}", protocol, message)
                } else {
                    write!(f, "Gaming error: {}", message)
                }
            }
            SongbirdError::Security { message, context } => {
                if let Some(context) = context {
                    write!(f, "Security error in context '{}': {}", context, message)
                } else {
                    write!(f, "Security error: {}", message)
                }
            }
            SongbirdError::Protocol { protocol, message } => {
                write!(f, "Protocol error [{}]: {}", protocol, message)
            }
            SongbirdError::Service { service, message } => {
                write!(f, "Service error [{}]: {}", service, message)
            }
            SongbirdError::Validation { field, message } => {
                write!(f, "Validation error for field '{}': {}", field, message)
            }
            SongbirdError::NotFound { resource, message } => {
                write!(f, "Resource '{}' not found: {}", resource, message)
            }
            SongbirdError::Io { message } => write!(f, "IO error: {}", message),
            SongbirdError::LoadBalancer { message } => {
                write!(f, "Load balancer error: {}", message)
            }
            SongbirdError::TunnelCreation(message) => {
                write!(f, "Tunnel creation error: {}", message)
            }
            SongbirdError::EncryptionFailed(message) => write!(f, "Encryption failed: {}", message),
            SongbirdError::DecryptionFailed(message) => write!(f, "Decryption failed: {}", message),
            SongbirdError::NetworkDetection(message) => write!(f, "Network detection error: {}", message),
            SongbirdError::UnsupportedChannelType => write!(f, "Unsupported channel type error"),
            SongbirdError::Deployment { service, message } => {
                write!(f, "Deployment error for service '{}': {}", service, message)
            }
            SongbirdError::CircuitBreakerOpen { message } => {
                write!(f, "Circuit breaker is open: {}", message)
            }
            SongbirdError::CircuitBreakerFailure { message } => {
                write!(f, "Circuit breaker failure: {}", message)
            }
            SongbirdError::RetryExhausted { attempts, last_error } => {
                write!(f, "Retry exhausted after {} attempts, last error: {}", attempts, last_error)
            }
            SongbirdError::RateLimitExceeded { message } => {
                write!(f, "Rate limit exceeded: {}", message)
            }
            SongbirdError::ExecutionFailed { message } => {
                write!(f, "Execution failed: {}", message)
            }
            SongbirdError::CompositionFailed(message) => {
                write!(f, "Plugin composition failed: {}", message)
            }
        }
    }
}
impl std::error::Error for SongbirdError {}
// Helper methods for common error patterns in HPC system
impl SongbirdError {
    /// Create a service error
    pub fn service_error(service_id: &str, message: String) -> Self {
        SongbirdError::Service {
            service: service_id.to_string(),
            message,
        }
    }

    /// Create a health check failure error
    pub fn health_check_failed(service_id: &str, message: String) -> Self {
        SongbirdError::Service {
            service: service_id.to_string(),
            message: format!("Health check failed: {}", message),
        }
    }

    /// Create a configuration error
    pub fn configuration_error(message: String) -> Self {
        SongbirdError::Config {
            message,
            field: None,
        }
    }
}
// Implement From traits for common error types
impl From<&str> for SongbirdError {
    fn from(msg: &str) -> Self {
        SongbirdError::Protocol {
            protocol: "unknown".to_string(),
            message: msg.to_string(),
        }
    }
}
impl From<String> for SongbirdError {
    fn from(msg: String) -> Self {
        SongbirdError::Protocol {
            protocol: "unknown".to_string(),
            message: msg,
        }
    }
}
impl From<std::io::Error> for SongbirdError {
    fn from(err: std::io::Error) -> Self {
        SongbirdError::Io {
            message: err.to_string(),
        }
    }
}
impl From<std::net::AddrParseError> for SongbirdError {
    fn from(err: std::net::AddrParseError) -> Self {
        SongbirdError::Network {
            service: "unknown".to_string(),
            message: format!("Invalid address format: {}", err),
            details: None,
        }
    }
}
impl From<std::time::SystemTimeError> for SongbirdError {
    fn from(err: std::time::SystemTimeError) -> Self {
        SongbirdError::Config {
            message: format!("System time error: {}", err),
            field: Some("system_time".to_string()),
        }
    }
}
impl From<serde_json::Error> for SongbirdError {
    fn from(err: serde_json::Error) -> Self {
        SongbirdError::Config {
            message: format!("JSON serialization error: {}", err),
            field: Some("json".to_string()),
        }
    }
}
impl From<Box<dyn std::error::Error>> for SongbirdError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        SongbirdError::Network {
            service: "unknown".to_string(),
            message: err.to_string(),
            details: None,
        }
    }
}
impl From<Box<dyn std::error::Error + Send + Sync>> for SongbirdError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        SongbirdError::Network {
            service: "unknown".to_string(),
            message: err.to_string(),
            details: None,
        }
    }
}

// CLI and HTTP client error types would be defined here...
#[derive(Debug)]
pub struct CliError {
    pub message: String,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for CliError {}

impl From<CliError> for SongbirdError {
    fn from(err: CliError) -> Self {
        SongbirdError::Config {
            message: err.message,
            field: None,
        }
    }
}

#[derive(Debug)]
pub struct HyperClientError {
    pub message: String,
}

impl fmt::Display for HyperClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for HyperClientError {}

impl From<HyperClientError> for SongbirdError {
    fn from(err: HyperClientError) -> Self {
        SongbirdError::Config {
            message: err.message,
            field: None,
        }
    }
}
