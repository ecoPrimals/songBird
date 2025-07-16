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
                    write!(f, "Configuration error in field '{field}': {message}")
                } else {
                    write!(f, "Configuration error: {message}")
                }
            }
            SongbirdError::Configuration { field, message } => {
                write!(f, "Configuration error in field '{field}': {message}")
            }
            SongbirdError::Network {
                service,
                message,
                details,
            } => {
                if let Some(details) = details {
                    write!(
                        f,
                        "Network error in service '{service}': {message}: {details}"
                    )
                } else {
                    write!(f, "Network error in service '{service}': {message}")
                }
            }
            SongbirdError::Communication(message) => write!(f, "Communication error: {message}"),
            SongbirdError::Discovery { message, service } => {
                if let Some(service) = service {
                    write!(f, "Discovery error for service '{service}': {message}")
                } else {
                    write!(f, "Discovery error: {message}")
                }
            }
            SongbirdError::Auth { message, user } => {
                if let Some(user) = user {
                    write!(f, "Authentication error for user '{user}': {message}")
                } else {
                    write!(f, "Authentication error: {message}")
                }
            }
            SongbirdError::Authentication { provider, message } => {
                write!(
                    f,
                    "Authentication error from provider '{provider}': {message}"
                )
            }
            SongbirdError::Gaming { message, protocol } => {
                if let Some(protocol) = protocol {
                    write!(f, "Gaming error for protocol '{protocol}': {message}")
                } else {
                    write!(f, "Gaming error: {message}")
                }
            }
            SongbirdError::Security { message, context } => {
                if let Some(context) = context {
                    write!(f, "Security error in context '{context}': {message}")
                } else {
                    write!(f, "Security error: {message}")
                }
            }
            SongbirdError::Protocol { protocol, message } => {
                write!(f, "Protocol error [{protocol}]: {message}")
            }
            SongbirdError::Service { service, message } => {
                write!(f, "Service error [{service}]: {message}")
            }
            SongbirdError::Validation { field, message } => {
                write!(f, "Validation error for field '{field}': {message}")
            }
            SongbirdError::NotFound { resource, message } => {
                write!(f, "Resource '{resource}' not found: {message}")
            }
            SongbirdError::Io { message } => write!(f, "IO error: {message}"),
            SongbirdError::LoadBalancer { message } => {
                write!(f, "Load balancer error: {message}")
            }
            SongbirdError::TunnelCreation(message) => {
                write!(f, "Tunnel creation error: {message}")
            }
            SongbirdError::EncryptionFailed(message) => write!(f, "Encryption failed: {message}"),
            SongbirdError::DecryptionFailed(message) => write!(f, "Decryption failed: {message}"),
            SongbirdError::NetworkDetection(message) => {
                write!(f, "Network detection error: {message}")
            }
            SongbirdError::UnsupportedChannelType => write!(f, "Unsupported channel type error"),
            SongbirdError::Deployment { service, message } => {
                write!(f, "Deployment error for service '{service}': {message}")
            }
            SongbirdError::CircuitBreakerOpen { message } => {
                write!(f, "Circuit breaker is open: {message}")
            }
            SongbirdError::CircuitBreakerFailure { message } => {
                write!(f, "Circuit breaker failure: {message}")
            }
            SongbirdError::RetryExhausted {
                attempts,
                last_error,
            } => {
                write!(
                    f,
                    "Retry exhausted after {attempts} attempts, last error: {last_error}"
                )
            }
            SongbirdError::RateLimitExceeded { message } => {
                write!(f, "Rate limit exceeded: {message}")
            }
            SongbirdError::ExecutionFailed { message } => {
                write!(f, "Execution failed: {message}")
            }
            SongbirdError::CompositionFailed(message) => {
                write!(f, "Plugin composition failed: {message}")
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
            message: format!("Health check failed: {message}"),
        }
    }

    /// Create a configuration error
    pub fn configuration_error(message: String) -> Self {
        SongbirdError::Config {
            message,
            field: None,
        }
    }

    /// Create a configuration error with basic fields
    pub fn config_error(message: String, field: Option<String>) -> Self {
        Self::Config { message, field }
    }

    /// Create a network error with basic fields
    pub fn network_error(service: String, message: String, details: Option<String>) -> Self {
        Self::Network {
            service,
            message,
            details,
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
            message: format!("Invalid address format: {err}"),
            details: None,
        }
    }
}
impl From<std::time::SystemTimeError> for SongbirdError {
    fn from(err: std::time::SystemTimeError) -> Self {
        SongbirdError::Config {
            message: format!("System time error: {err}"),
            field: Some("system_time".to_string()),
        }
    }
}
impl From<serde_json::Error> for SongbirdError {
    fn from(err: serde_json::Error) -> Self {
        SongbirdError::Config {
            message: format!("JSON serialization error: {err}"),
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

impl From<songbird_universal_primals::errors::PrimalError> for SongbirdError {
    fn from(error: songbird_universal_primals::errors::PrimalError) -> Self {
        match error {
            songbird_universal_primals::errors::PrimalError::Network(msg) => {
                SongbirdError::Network {
                    service: "primal".to_string(),
                    message: msg,
                    details: None,
                }
            }
            songbird_universal_primals::errors::PrimalError::Authentication(msg) => {
                SongbirdError::Auth {
                    message: msg,
                    user: None,
                }
            }
            songbird_universal_primals::errors::PrimalError::Authorization(msg) => {
                SongbirdError::Auth {
                    message: msg,
                    user: None,
                }
            }
            songbird_universal_primals::errors::PrimalError::Configuration(msg) => {
                SongbirdError::Config {
                    message: msg,
                    field: None,
                }
            }
            songbird_universal_primals::errors::PrimalError::Validation(msg) => {
                SongbirdError::Config {
                    message: msg,
                    field: None,
                }
            }
            songbird_universal_primals::errors::PrimalError::Timeout(msg) => {
                SongbirdError::Network {
                    service: "primal".to_string(),
                    message: format!("Timeout: {msg}"),
                    details: None,
                }
            }
            songbird_universal_primals::errors::PrimalError::ServiceUnavailable(msg) => {
                SongbirdError::Network {
                    service: "primal".to_string(),
                    message: format!("Service unavailable: {msg}"),
                    details: None,
                }
            }
            songbird_universal_primals::errors::PrimalError::InvalidRequest(msg) => {
                SongbirdError::Config {
                    message: format!("Invalid request: {msg}"),
                    field: None,
                }
            }
            songbird_universal_primals::errors::PrimalError::Internal(msg) => {
                SongbirdError::Network {
                    service: "primal".to_string(),
                    message: format!("Internal error: {msg}"),
                    details: None,
                }
            }
            songbird_universal_primals::errors::PrimalError::RateLimit(msg) => {
                SongbirdError::Network {
                    service: "primal".to_string(),
                    message: format!("Rate limit: {msg}"),
                    details: None,
                }
            }
            _ => SongbirdError::Network {
                service: "primal".to_string(),
                message: error.to_string(),
                details: None,
            },
        }
    }
}

impl From<SongbirdError> for hyper::StatusCode {
    fn from(error: SongbirdError) -> Self {
        match error {
            SongbirdError::Security { .. } => hyper::StatusCode::FORBIDDEN,
            SongbirdError::Protocol { .. } => hyper::StatusCode::BAD_REQUEST,
            SongbirdError::Service { .. } => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            SongbirdError::CompositionFailed(_) => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            SongbirdError::Communication(_) => hyper::StatusCode::BAD_GATEWAY,
            SongbirdError::Validation { .. } => hyper::StatusCode::BAD_REQUEST,
            SongbirdError::ExecutionFailed { .. } => hyper::StatusCode::REQUEST_TIMEOUT,
            SongbirdError::RateLimitExceeded { .. } => hyper::StatusCode::TOO_MANY_REQUESTS,
            SongbirdError::NotFound { .. } => hyper::StatusCode::NOT_FOUND,
            SongbirdError::Config { .. } => hyper::StatusCode::BAD_REQUEST,
            SongbirdError::Configuration { .. } => hyper::StatusCode::BAD_REQUEST,
            SongbirdError::Io { .. } => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            SongbirdError::Network { .. } => hyper::StatusCode::BAD_GATEWAY,
            SongbirdError::Discovery { .. } => hyper::StatusCode::BAD_REQUEST,
            SongbirdError::LoadBalancer { .. } => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            SongbirdError::Auth { .. } => hyper::StatusCode::UNAUTHORIZED,
            SongbirdError::Authentication { .. } => hyper::StatusCode::UNAUTHORIZED,
            SongbirdError::Gaming { .. } => hyper::StatusCode::BAD_REQUEST,
            SongbirdError::TunnelCreation(_) => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            SongbirdError::EncryptionFailed(_) => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            SongbirdError::DecryptionFailed(_) => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            SongbirdError::NetworkDetection(_) => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            SongbirdError::UnsupportedChannelType => hyper::StatusCode::BAD_REQUEST,
            SongbirdError::Deployment { .. } => hyper::StatusCode::INTERNAL_SERVER_ERROR,
            SongbirdError::CircuitBreakerOpen { .. } => hyper::StatusCode::SERVICE_UNAVAILABLE,
            SongbirdError::CircuitBreakerFailure { .. } => hyper::StatusCode::SERVICE_UNAVAILABLE,
            SongbirdError::RetryExhausted { .. } => hyper::StatusCode::REQUEST_TIMEOUT,
        }
    }
}

// Conversion from crate version to local version
impl From<songbird_errors::SongbirdError> for SongbirdError {
    fn from(error: songbird_errors::SongbirdError) -> Self {
        match error {
            songbird_errors::SongbirdError::Config { message, field, .. } => {
                SongbirdError::Config { message, field }
            }
            songbird_errors::SongbirdError::Network {
                service,
                message,
                details,
                ..
            } => SongbirdError::Network {
                service: service.unwrap_or_default(),
                message,
                details,
            },
            songbird_errors::SongbirdError::Communication(msg) => SongbirdError::Communication(msg),
            songbird_errors::SongbirdError::Service {
                service, message, ..
            } => SongbirdError::Service { service, message },
            songbird_errors::SongbirdError::Discovery {
                message, service, ..
            } => SongbirdError::Discovery { message, service },
            songbird_errors::SongbirdError::Auth { message, user, .. } => {
                SongbirdError::Auth { message, user }
            }
            songbird_errors::SongbirdError::Authentication {
                provider, message, ..
            } => SongbirdError::Authentication { provider, message },
            songbird_errors::SongbirdError::Gaming {
                message, protocol, ..
            } => SongbirdError::Gaming { message, protocol },
            songbird_errors::SongbirdError::Security {
                message, context, ..
            } => SongbirdError::Security { message, context },
            songbird_errors::SongbirdError::Protocol {
                protocol, message, ..
            } => SongbirdError::Protocol { protocol, message },
            songbird_errors::SongbirdError::Validation { field, message, .. } => {
                SongbirdError::Validation { field, message }
            }
            songbird_errors::SongbirdError::NotFound {
                resource, message, ..
            } => SongbirdError::NotFound { resource, message },
            songbird_errors::SongbirdError::Io { message, .. } => SongbirdError::Io { message },
            songbird_errors::SongbirdError::LoadBalancer { message, .. } => {
                SongbirdError::LoadBalancer { message }
            }
            songbird_errors::SongbirdError::Deployment {
                service, message, ..
            } => SongbirdError::Deployment { service, message },
            songbird_errors::SongbirdError::CircuitBreakerOpen { message, .. } => {
                SongbirdError::CircuitBreakerOpen { message }
            }
            songbird_errors::SongbirdError::CircuitBreakerFailure { message, .. } => {
                SongbirdError::CircuitBreakerFailure { message }
            }
            songbird_errors::SongbirdError::RetryExhausted { attempts, .. } => {
                SongbirdError::RetryExhausted {
                    attempts,
                    last_error: "Unknown error".to_string(),
                }
            }
            songbird_errors::SongbirdError::RateLimitExceeded { message, .. } => {
                SongbirdError::RateLimitExceeded { message }
            }
            songbird_errors::SongbirdError::ExecutionFailed { message, .. } => {
                SongbirdError::ExecutionFailed { message }
            }
            _ => SongbirdError::CompositionFailed("Unknown error type".to_string()),
        }
    }
}

// Conversion from local version to crate version
impl From<SongbirdError> for songbird_errors::SongbirdError {
    fn from(error: SongbirdError) -> Self {
        match error {
            SongbirdError::Config { message, field } => songbird_errors::SongbirdError::Config {
                message,
                field,
                suggestion: None,
                context: None,
            },
            SongbirdError::Network {
                service,
                message,
                details,
            } => songbird_errors::SongbirdError::Network {
                service: Some(service),
                message,
                details,
                endpoint: None,
                suggestion: None,
            },
            SongbirdError::Communication(msg) => songbird_errors::SongbirdError::Communication(msg),
            SongbirdError::Service { service, message } => {
                songbird_errors::SongbirdError::Service {
                    service,
                    message,
                    status: None,
                    suggestion: None,
                }
            }
            SongbirdError::Discovery { message, service } => {
                songbird_errors::SongbirdError::Discovery {
                    message,
                    service,
                    timeout: None,
                    suggestion: None,
                }
            }
            SongbirdError::Auth { message, user } => songbird_errors::SongbirdError::Auth {
                message,
                user,
                provider: None,
                suggestion: None,
            },
            SongbirdError::Authentication { provider, message } => {
                songbird_errors::SongbirdError::Authentication {
                    provider,
                    message,
                    suggestion: None,
                }
            }
            SongbirdError::Gaming { message, protocol } => songbird_errors::SongbirdError::Gaming {
                message,
                protocol,
                game: None,
                suggestion: None,
            },
            SongbirdError::Security { message, context } => {
                songbird_errors::SongbirdError::Security {
                    message,
                    context,
                    severity: None,
                    suggestion: None,
                }
            }
            SongbirdError::Protocol { protocol, message } => {
                songbird_errors::SongbirdError::Protocol {
                    protocol,
                    message,
                    version: None,
                    suggestion: None,
                }
            }
            SongbirdError::Validation { field, message } => {
                songbird_errors::SongbirdError::Validation {
                    field,
                    message,
                    value: None,
                    expected: None,
                    suggestion: None,
                }
            }
            SongbirdError::NotFound { resource, message } => {
                songbird_errors::SongbirdError::NotFound {
                    resource,
                    message,
                    searched_paths: None,
                    suggestion: None,
                }
            }
            SongbirdError::Io { message } => songbird_errors::SongbirdError::Io {
                message,
                path: None,
                operation: None,
                suggestion: None,
            },
            SongbirdError::LoadBalancer { message } => {
                songbird_errors::SongbirdError::LoadBalancer {
                    message,
                    backend: None,
                    suggestion: None,
                }
            }
            SongbirdError::Deployment { service, message } => {
                songbird_errors::SongbirdError::Deployment {
                    service,
                    message,
                    environment: None,
                    stage: None,
                    suggestion: None,
                }
            }
            SongbirdError::CircuitBreakerOpen { message } => {
                songbird_errors::SongbirdError::CircuitBreakerOpen {
                    service: "unknown".to_string(),
                    message,
                    failure_count: None,
                    suggestion: None,
                }
            }
            SongbirdError::CircuitBreakerFailure { message } => {
                songbird_errors::SongbirdError::CircuitBreakerFailure {
                    service: "unknown".to_string(),
                    message,
                    suggestion: None,
                }
            }
            SongbirdError::RetryExhausted {
                attempts,
                last_error,
            } => songbird_errors::SongbirdError::RetryExhausted {
                attempts,
                message: last_error,
                duration: None,
                suggestion: None,
            },
            SongbirdError::RateLimitExceeded { message } => {
                songbird_errors::SongbirdError::RateLimitExceeded {
                    message,
                    service: None,
                    limit: None,
                    suggestion: None,
                }
            }
            SongbirdError::ExecutionFailed { message } => {
                songbird_errors::SongbirdError::ExecutionFailed {
                    message,
                    command: None,
                    exit_code: None,
                    suggestion: None,
                }
            }
            _ => songbird_errors::SongbirdError::Generic("Unknown error type".to_string()),
        }
    }
}
