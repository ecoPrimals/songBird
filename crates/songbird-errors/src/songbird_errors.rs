//! Comprehensive error types for Songbird Orchestrator

use std::fmt;

#[derive(Debug, Clone)]
pub enum SongbirdError {
    Communication(String),
    Config { message: String, field: Option<String> },
    Configuration { field: String, message: String },
    Io { message: String },
    Network { service: String, message: String, details: Option<String> },
    Discovery { message: String, service: Option<String> },
    Service { service: String, message: String },
    LoadBalancer { message: String },
    Protocol { protocol: String, message: String },
    Auth { message: String, user: Option<String> },
    Authentication { provider: String, message: String },
    Gaming { message: String, protocol: Option<String> },
    Security { message: String, context: Option<String> },
    Validation { field: String, message: String },
    NotFound { resource: String, message: String },
    TunnelCreation(String),
    EncryptionFailed(String),
    DecryptionFailed(String),
    NetworkDetection(String),
    UnsupportedChannelType,
    Deployment { service: String, message: String },
    CompositionFailed(String),
    PluginNotFound(String),
    RateLimitExceeded(String),
    ExecutionFailed(String),
    ResourceExhausted { resource: String, message: String },
    CircuitBreakerOpen { service: String, message: String },
    CircuitBreakerFailure { service: String, message: String },
    RetryExhausted { attempts: u32, message: String },
}

impl fmt::Display for SongbirdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SongbirdError::Communication(msg) => write!(f, "Communication error: {}", msg),
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
            SongbirdError::Io { message } => write!(f, "IO error: {}", message),
            SongbirdError::Network { service, message, details } => {
                if let Some(details) = details {
                    write!(f, "Network error in service '{}': {}: {}", service, message, details)
                } else {
                    write!(f, "Network error in service '{}': {}", service, message)
                }
            }
            SongbirdError::Discovery { message, service } => {
                if let Some(service) = service {
                    write!(f, "Discovery error for service '{}': {}", service, message)
                } else {
                    write!(f, "Discovery error: {}", message)
                }
            }
            SongbirdError::Service { service, message } => {
                write!(f, "Service error [{}]: {}", service, message)
            }
            SongbirdError::LoadBalancer { message } => {
                write!(f, "Load balancer error: {}", message)
            }
            SongbirdError::Protocol { protocol, message } => {
                write!(f, "Protocol error [{}]: {}", protocol, message)
            }
            SongbirdError::Auth { message, user } => {
                if let Some(user) = user {
                    write!(f, "Authentication error for user '{}': {}", user, message)
                } else {
                    write!(f, "Authentication error: {}", message)
                }
            }
            SongbirdError::Authentication { provider, message } => {
                write!(f, "Authentication error from provider '{}': {}", provider, message)
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
            SongbirdError::Validation { field, message } => {
                write!(f, "Validation error for field '{}': {}", field, message)
            }
            SongbirdError::NotFound { resource, message } => {
                write!(f, "Resource '{}' not found: {}", resource, message)
            }
            SongbirdError::TunnelCreation(message) => {
                write!(f, "Tunnel creation error: {}", message)
            }
            SongbirdError::EncryptionFailed(message) => write!(f, "Encryption failed: {}", message),
            SongbirdError::DecryptionFailed(message) => write!(f, "Decryption failed: {}", message),
            SongbirdError::NetworkDetection(message) => {
                write!(f, "Network detection error: {}", message)
            }
            SongbirdError::UnsupportedChannelType => write!(f, "Unsupported channel type error"),
            SongbirdError::Deployment { service, message } => {
                write!(f, "Deployment error for service '{}': {}", service, message)
            }
            SongbirdError::CompositionFailed(message) => {
                write!(f, "Plugin composition failed: {}", message)
            }
            SongbirdError::PluginNotFound(message) => {
                write!(f, "Plugin not found: {}", message)
            }
            SongbirdError::RateLimitExceeded(message) => {
                write!(f, "Rate limit exceeded: {}", message)
            }
            SongbirdError::ExecutionFailed(message) => {
                write!(f, "Execution failed: {}", message)
            }
            SongbirdError::ResourceExhausted { resource, message } => {
                write!(f, "Resource exhausted [{}]: {}", resource, message)
            }
            SongbirdError::CircuitBreakerOpen { service, message } => {
                write!(f, "Circuit breaker open for service '{}': {}", service, message)
            }
            SongbirdError::CircuitBreakerFailure { service, message } => {
                write!(f, "Circuit breaker failure for service '{}': {}", service, message)
            }
            SongbirdError::RetryExhausted { attempts, message } => {
                write!(f, "Retry exhausted after {} attempts: {}", attempts, message)
            }
        }
    }
}

impl std::error::Error for SongbirdError {}

impl SongbirdError {
    /// Create a service error
    pub fn service_error(service_id: &str, message: String) -> Self {
        SongbirdError::Service {
            service: service_id.to_string(),
            message,
        }
    }

    /// Create a health check failed error
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

// From implementations for seamless error conversion
impl From<&str> for SongbirdError {
    fn from(msg: &str) -> Self {
        SongbirdError::Communication(msg.to_string())
    }
}

impl From<String> for SongbirdError {
    fn from(msg: String) -> Self {
        SongbirdError::Communication(msg)
    }
}

impl From<std::io::Error> for SongbirdError {
    fn from(err: std::io::Error) -> Self {
        SongbirdError::Io {
            message: err.to_string() 
        }
    }
}

impl From<std::net::AddrParseError> for SongbirdError {
    fn from(err: std::net::AddrParseError) -> Self {
        SongbirdError::Network {
            service: "addr_parser".to_string(),
            message: err.to_string(),
            details: None,
        }
    }
}

impl From<std::time::SystemTimeError> for SongbirdError {
    fn from(err: std::time::SystemTimeError) -> Self {
        SongbirdError::Io { 
            message: err.to_string() 
        }
    }
}

impl From<serde_json::Error> for SongbirdError {
    fn from(err: serde_json::Error) -> Self {
        SongbirdError::Network {
            service: "json_parser".to_string(),
            message: err.to_string(),
            details: None,
        }
    }
}

pub type Result<T> = std::result::Result<T, SongbirdError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_formatting() {
        let error = SongbirdError::Config {
            message: "Invalid value".to_string(),
            field: Some("timeout".to_string()),
        };
        assert!(error.to_string().contains("timeout"));
        assert!(error.to_string().contains("Invalid value"));
    }

    #[test]
    fn test_error_display_formatting_no_field() {
        let error = SongbirdError::Config {
            message: "Invalid configuration".to_string(),
            field: None,
        };
        assert!(error.to_string().contains("Invalid configuration"));
        assert!(!error.to_string().contains("field"));
    }

    #[test]
    fn test_service_error_creation() {
        let error = SongbirdError::service_error("test-service", "Test message".to_string());
        assert!(error.to_string().contains("test-service"));
        assert!(error.to_string().contains("Test message"));
    }

    #[test]
    fn test_health_check_failed_creation() {
        let error = SongbirdError::health_check_failed("db-service", "Timeout".to_string());
        assert!(error.to_string().contains("db-service"));
        assert!(error.to_string().contains("Health check failed"));
    }

    #[test]
    fn test_configuration_error_creation() {
        let error = SongbirdError::configuration_error("Invalid config".to_string());
        assert!(error.to_string().contains("Invalid config"));
    }

    #[test]
    fn test_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let error: SongbirdError = io_error.into();
        assert!(error.to_string().contains("File not found"));
    }

    #[test]
    fn test_from_str() {
        let error: SongbirdError = "Test error".into();
        assert!(error.to_string().contains("Test error"));
    }

    #[test]
    fn test_from_string() {
        let error: SongbirdError = "Test error".to_string().into();
        assert!(error.to_string().contains("Test error"));
    }

    #[test]
    fn test_from_addr_parse_error() {
        let addr_error = "invalid_address:port".parse::<std::net::SocketAddr>().unwrap_err();
        let error: SongbirdError = addr_error.into();
        assert!(error.to_string().contains("addr_parser"));
    }

    #[test]
    fn test_from_system_time_error() {
        let time_error = std::time::SystemTime::UNIX_EPOCH
            .duration_since(std::time::SystemTime::now())
            .unwrap_err();
        let error: SongbirdError = time_error.into();
        assert!(error.to_string().contains("IO error"));
    }

    #[test]
    fn test_from_json_error() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let error: SongbirdError = json_error.into();
        assert!(error.to_string().contains("json_parser"));
    }

    #[test]
    fn test_error_variants_coverage() {
        // Test that all major error variants can be created and displayed
        let errors = vec![
            SongbirdError::Communication("test".to_string()),
            SongbirdError::TunnelCreation("tunnel error".to_string()),
            SongbirdError::EncryptionFailed("encryption error".to_string()),
            SongbirdError::DecryptionFailed("decryption error".to_string()),
            SongbirdError::NetworkDetection("network error".to_string()),
            SongbirdError::UnsupportedChannelType,
            SongbirdError::CompositionFailed("composition error".to_string()),
            SongbirdError::PluginNotFound("plugin error".to_string()),
            SongbirdError::RateLimitExceeded("rate limit error".to_string()),
            SongbirdError::ExecutionFailed("execution error".to_string()),
        ];

        for error in errors {
            let display_str = error.to_string();
            assert!(!display_str.is_empty());
            // Each error should produce a meaningful error message
            assert!(display_str.contains("error") || display_str.contains("failed"));
        }
    }

    #[test]
    fn test_network_error_variants() {
        let error_with_details = SongbirdError::Network {
            service: "test-service".to_string(),
            message: "Connection failed".to_string(),
            details: Some("Timeout after 30s".to_string()),
        };
        let display = error_with_details.to_string();
        assert!(display.contains("test-service"));
        assert!(display.contains("Connection failed"));
        assert!(display.contains("Timeout after 30s"));

        let error_without_details = SongbirdError::Network {
            service: "test-service".to_string(),
            message: "Connection failed".to_string(),
            details: None,
        };
        let display = error_without_details.to_string();
        assert!(display.contains("test-service"));
        assert!(display.contains("Connection failed"));
        assert!(!display.contains("Timeout"));
    }

    #[test]
    fn test_discovery_error_variants() {
        let error_with_service = SongbirdError::Discovery {
            message: "Service not found".to_string(),
            service: Some("api-service".to_string()),
        };
        let display = error_with_service.to_string();
        assert!(display.contains("api-service"));
        assert!(display.contains("Service not found"));

        let error_without_service = SongbirdError::Discovery {
            message: "Discovery failed".to_string(),
            service: None,
        };
        let display = error_without_service.to_string();
        assert!(display.contains("Discovery failed"));
        assert!(!display.contains("api-service"));
    }

    #[test]
    fn test_auth_error_variants() {
        let error_with_user = SongbirdError::Auth {
            message: "Invalid credentials".to_string(),
            user: Some("testuser".to_string()),
        };
        let display = error_with_user.to_string();
        assert!(display.contains("testuser"));
        assert!(display.contains("Invalid credentials"));

        let error_without_user = SongbirdError::Auth {
            message: "Authentication failed".to_string(),
            user: None,
        };
        let display = error_without_user.to_string();
        assert!(display.contains("Authentication failed"));
        assert!(!display.contains("testuser"));
    }

    #[test]
    fn test_gaming_error_variants() {
        let error_with_protocol = SongbirdError::Gaming {
            message: "Protocol mismatch".to_string(),
            protocol: Some("IPX".to_string()),
        };
        let display = error_with_protocol.to_string();
        assert!(display.contains("IPX"));
        assert!(display.contains("Protocol mismatch"));

        let error_without_protocol = SongbirdError::Gaming {
            message: "Gaming error".to_string(),
            protocol: None,
        };
        let display = error_without_protocol.to_string();
        assert!(display.contains("Gaming error"));
        assert!(!display.contains("IPX"));
    }

    #[test]
    fn test_security_error_variants() {
        let error_with_context = SongbirdError::Security {
            message: "Access denied".to_string(),
            context: Some("admin_panel".to_string()),
        };
        let display = error_with_context.to_string();
        assert!(display.contains("admin_panel"));
        assert!(display.contains("Access denied"));

        let error_without_context = SongbirdError::Security {
            message: "Security violation".to_string(),
            context: None,
        };
        let display = error_without_context.to_string();
        assert!(display.contains("Security violation"));
        assert!(!display.contains("admin_panel"));
    }

    #[test]
    fn test_complex_error_variants() {
        let resource_exhausted = SongbirdError::ResourceExhausted {
            resource: "memory".to_string(),
            message: "Out of memory".to_string(),
        };
        assert!(resource_exhausted.to_string().contains("memory"));
        assert!(resource_exhausted.to_string().contains("Out of memory"));

        let circuit_breaker = SongbirdError::CircuitBreakerOpen {
            service: "payment-service".to_string(),
            message: "Too many failures".to_string(),
        };
        assert!(circuit_breaker.to_string().contains("payment-service"));
        assert!(circuit_breaker.to_string().contains("Too many failures"));

        let retry_exhausted = SongbirdError::RetryExhausted {
            attempts: 5,
            message: "Max retries reached".to_string(),
        };
        assert!(retry_exhausted.to_string().contains("5"));
        assert!(retry_exhausted.to_string().contains("Max retries reached"));
    }

    #[test]
    fn test_error_trait_implementation() {
        let error = SongbirdError::Communication("Test error".to_string());
        let error_trait: &dyn std::error::Error = &error;
        assert!(!error_trait.to_string().is_empty());
    }

    #[test]
    fn test_error_debug_formatting() {
        let error = SongbirdError::Config {
            message: "Debug test".to_string(),
            field: Some("test_field".to_string()),
        };
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("Debug test"));
        assert!(debug_str.contains("test_field"));
    }

    #[test]
    fn test_error_clone() {
        let error = SongbirdError::Communication("Clone test".to_string());
        let cloned = error.clone();
        assert_eq!(error.to_string(), cloned.to_string());
    }

    #[test]
    fn test_result_type_alias() {
        fn test_function() -> Result<String> {
            Ok("Success".to_string())
        }

        fn test_function_error() -> Result<String> {
            Err(SongbirdError::Communication("Error".to_string()))
        }

        assert!(test_function().is_ok());
        assert!(test_function_error().is_err());
    }
}

 