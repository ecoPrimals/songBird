/// Error constructor methods for SongbirdError
///
/// This module contains all the convenient constructor methods for creating
/// different types of SongbirdError instances.

use super::core::SongbirdError;
use super::specific::*;

impl SongbirdError {
    /// Create a new communication error
    pub fn communication(msg: impl Into<String>) -> Self {
        SongbirdError::Communication(msg.into())
    }

    /// Create a new configuration error
    pub fn config(message: impl Into<String>) -> Self {
        SongbirdError::Config {
            message: message.into(),
            field: None,
            suggestion: None,
            context: None,
        }
    }

    /// Create a new configuration error with field
    pub fn config_field(field: impl Into<String>, message: impl Into<String>) -> Self {
        SongbirdError::ConfigField {
            field: field.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Create a new IO error
    pub fn io_error(message: impl Into<String>) -> Self {
        SongbirdError::Io(Box::new(IoError {
            message: message.into(),
            operation: None,
            path: None,
        }))
    }

    /// Create a new network error
    pub fn network_error(message: impl Into<String>) -> Self {
        SongbirdError::Network(Box::new(NetworkError {
            message: message.into(),
            endpoint: None,
            port: None,
            protocol: None,
        }))
    }

    /// Create a connection limit exceeded error
    pub fn connection_limit_exceeded(max_connections: usize) -> Self {
        SongbirdError::Network(Box::new(NetworkError {
            message: format!("Connection limit exceeded: {max_connections} maximum connections"),
            endpoint: None,
            port: None,
            protocol: Some("RPC".to_string()),
        }))
    }

    /// Create a connection not found error
    pub fn connection_not_found(connection_id: impl Into<String>) -> Self {
        SongbirdError::Network(Box::new(NetworkError {
            message: format!("Connection not found: {}", connection_id.into()),
            endpoint: None,
            port: None,
            protocol: Some("RPC".to_string()),
        }))
    }

    /// Create a new service error
    pub fn service_error(service: impl Into<String>, message: impl Into<String>) -> Self {
        SongbirdError::Service(Box::new(ServiceError {
            service: service.into(),
            message: message.into(),
            status: None,
            suggestion: None,
        }))
    }

    /// Create a new discovery error
    pub fn discovery_error(message: impl Into<String>) -> Self {
        SongbirdError::Discovery(Box::new(DiscoveryError {
            message: message.into(),
            service: None,
            timeout: None,
            suggestion: None,
        }))
    }

    /// Create a new protocol error
    pub fn protocol_error(message: impl Into<String>) -> Self {
        SongbirdError::Protocol(Box::new(ProtocolError {
            message: message.into(),
            protocol: None,
        }))
    }

    /// Create a new authentication error
    pub fn auth_error(message: impl Into<String>) -> Self {
        SongbirdError::Auth(Box::new(AuthError {
            message: message.into(),
            provider: None,
        }))
    }

    /// Create a new gaming error
    pub fn gaming_error(message: impl Into<String>) -> Self {
        SongbirdError::Gaming(Box::new(GamingError {
            message: message.into(),
            game: None,
        }))
    }

    /// Create a new validation error
    pub fn validation_error(message: impl Into<String>) -> Self {
        SongbirdError::Validation(Box::new(ValidationError {
            message: message.into(),
            field: None,
            expected: None,
        }))
    }

    /// Create a new internal error (generic internal failure)
    pub fn internal_error(message: impl Into<String>) -> Self {
        SongbirdError::Communication(message.into())
    }

    /// Create a new operation error (operation failure)
    pub fn operation_error(message: impl Into<String>) -> Self {
        SongbirdError::Communication(message.into())
    }



    /// Create a new not found error
    pub fn not_found_error(message: impl Into<String>) -> Self {
        SongbirdError::NotFound(Box::new(NotFoundError {
            message: message.into(),
            resource: None,
        }))
    }

    /// Create a new deployment error
    pub fn deployment_error(message: impl Into<String>) -> Self {
        SongbirdError::Deployment(Box::new(DeploymentError {
            message: message.into(),
            environment: None,
            component: None,
        }))
    }

    /// Create a new plugin not found error
    pub fn plugin_not_found_error(plugin: impl Into<String>) -> Self {
        SongbirdError::PluginNotFound(Box::new(PluginNotFoundError {
            plugin: plugin.into(),
        }))
    }

    /// Create a new rate limit error
    pub fn rate_limit_error(message: impl Into<String>) -> Self {
        SongbirdError::RateLimitExceeded(Box::new(RateLimitError {
            message: message.into(),
            limit: None,
        }))
    }

    /// Create a new execution error
    pub fn execution_error(message: impl Into<String>) -> Self {
        SongbirdError::ExecutionFailed(Box::new(ExecutionError {
            message: message.into(),
            command: None,
        }))
    }

    /// Create a new resource exhausted error
    pub fn resource_exhausted_error(message: impl Into<String>) -> Self {
        SongbirdError::ResourceExhausted(Box::new(ResourceExhaustedError {
            message: message.into(),
            resource: None,
            current: None,
        }))
    }

    /// Create a new circuit breaker error
    pub fn circuit_breaker_error(service: impl Into<String>, message: impl Into<String>) -> Self {
        SongbirdError::CircuitBreakerOpen(Box::new(CircuitBreakerError {
            service: service.into(),
            message: message.into(),
        }))
    }

    /// Create a new retry exhausted error
    pub fn retry_exhausted_error(message: impl Into<String>) -> Self {
        SongbirdError::RetryExhausted(Box::new(RetryError {
            message: message.into(),
            attempts: None,
        }))
    }

    /// Create a security error
    pub fn security_error(message: impl Into<String>) -> Self {
        SongbirdError::Security {
            message: message.into(),
            context: None,
            severity: None,
            suggestion: None,
        }
    }

    /// Create a tunnel creation error
    pub fn tunnel_creation_error(message: impl Into<String>) -> Self {
        SongbirdError::TunnelCreation {
            message: message.into(),
            tunnel_type: None,
            endpoint: None,
            suggestion: None,
        }
    }

    /// Create an encryption failed error
    pub fn encryption_failed(message: impl Into<String>) -> Self {
        SongbirdError::EncryptionFailed {
            message: message.into(),
            algorithm: None,
            suggestion: None,
        }
    }

    /// Create a decryption failed error
    pub fn decryption_failed(message: impl Into<String>) -> Self {
        SongbirdError::DecryptionFailed {
            message: message.into(),
            algorithm: None,
            suggestion: None,
        }
    }

    /// Create a network detection error
    pub fn network_detection_error(message: impl Into<String>) -> Self {
        SongbirdError::NetworkDetection {
            message: message.into(),
            interface: None,
            suggestion: None,
        }
    }

    /// Create an unsupported channel type error
    pub fn unsupported_channel_type(channel_type: impl Into<String>) -> Self {
        SongbirdError::UnsupportedChannelType {
            channel_type: Some(channel_type.into()),
            suggestion: None,
        }
    }

    /// Create a plugin composition error
    pub fn plugin_composition_error(message: impl Into<String>) -> Self {
        SongbirdError::PluginComposition {
            message: message.into(),
            plugin: None,
            suggestion: None,
        }
    }

    /// Create a load balancer error
    pub fn load_balancer_error(message: impl Into<String>) -> Self {
        SongbirdError::LoadBalancer {
            message: message.into(),
            backend: None,
            suggestion: None,
        }
    }

    /// Create a circuit breaker failure error
    pub fn circuit_breaker_failure(service: impl Into<String>, message: impl Into<String>) -> Self {
        SongbirdError::CircuitBreakerFailure {
            service: service.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Create a bulkhead full error
    pub fn bulkhead_full(bulkhead_id: impl Into<String>, message: impl Into<String>) -> Self {
        SongbirdError::BulkheadFull {
            bulkhead_id: bulkhead_id.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Create an authentication error with provider
    pub fn authentication_error(provider: impl Into<String>, message: impl Into<String>) -> Self {
        SongbirdError::Authentication {
            provider: provider.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Create an authentication provider error
    pub fn authentication_provider_error(
        provider: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        SongbirdError::AuthenticationProvider {
            provider: provider.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Add a suggestion to the error
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        match &mut self {
            SongbirdError::Config { suggestion: s, .. } => *s = Some(suggestion.into()),
            SongbirdError::ConfigField { suggestion: s, .. } => *s = Some(suggestion.into()),
            SongbirdError::Configuration { suggestion: s, .. } => *s = Some(suggestion.into()),
            SongbirdError::LoadBalancer { suggestion: s, .. } => *s = Some(suggestion.into()),
            SongbirdError::Authentication { suggestion: s, .. } => *s = Some(suggestion.into()),
            SongbirdError::AuthenticationProvider { suggestion: s, .. } => {
                *s = Some(suggestion.into())
            }
            SongbirdError::Security { suggestion: s, .. } => *s = Some(suggestion.into()),
            SongbirdError::TunnelCreation { suggestion: s, .. } => *s = Some(suggestion.into()),
            SongbirdError::EncryptionFailed { suggestion: s, .. } => *s = Some(suggestion.into()),
            SongbirdError::DecryptionFailed { suggestion: s, .. } => *s = Some(suggestion.into()),
            SongbirdError::NetworkDetection { suggestion: s, .. } => *s = Some(suggestion.into()),
            SongbirdError::UnsupportedChannelType { suggestion: s, .. } => {
                *s = Some(suggestion.into())
            }
            SongbirdError::PluginComposition { suggestion: s, .. } => *s = Some(suggestion.into()),
            SongbirdError::CircuitBreakerFailure { suggestion: s, .. } => {
                *s = Some(suggestion.into())
            }
            SongbirdError::BulkheadFull { suggestion: s, .. } => *s = Some(suggestion.into()),
            _ => {} // Other variants don't support suggestions
        }
        self
    }

    /// Add context to the error
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        match &mut self {
            SongbirdError::Config { context: c, .. } => *c = Some(context.into()),
            SongbirdError::Security { context: c, .. } => *c = Some(context.into()),
            _ => {} // Other variants don't support context
        }
        self
    }

    /// Add a field to the error
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        if let SongbirdError::Config { field: f, .. } = &mut self {
            *f = Some(field.into());
        } // Other variants don't support field
        self
    }

    /// Add a severity to the error
    pub fn with_severity(mut self, severity: impl Into<String>) -> Self {
        if let SongbirdError::Security { severity: s, .. } = &mut self {
            *s = Some(severity.into());
        } // Other variants don't support severity
        self
    }

    /// Check if the error is a network error
    pub fn is_network_error(&self) -> bool {
        matches!(
            self,
            SongbirdError::Network(_) | SongbirdError::NetworkDetection { .. }
        )
    }

    /// Check if the error is a configuration error
    pub fn is_config_error(&self) -> bool {
        matches!(
            self,
            SongbirdError::Config { .. }
                | SongbirdError::ConfigField { .. }
                | SongbirdError::Configuration { .. }
        )
    }

    /// Check if the error is a security error
    pub fn is_security_error(&self) -> bool {
        matches!(
            self,
            SongbirdError::Security { .. }
                | SongbirdError::EncryptionFailed { .. }
                | SongbirdError::DecryptionFailed { .. }
        )
    }

    /// Check if the error is an authentication error
    pub fn is_auth_error(&self) -> bool {
        matches!(
            self,
            SongbirdError::Auth(_)
                | SongbirdError::Authentication { .. }
                | SongbirdError::AuthenticationProvider { .. }
        )
    }

    /// Check if the error is a service error
    pub fn is_service_error(&self) -> bool {
        matches!(self, SongbirdError::Service(_))
    }

    /// Check if the error is a circuit breaker error
    pub fn is_circuit_breaker_error(&self) -> bool {
        matches!(
            self,
            SongbirdError::CircuitBreakerOpen(_) | SongbirdError::CircuitBreakerFailure { .. }
        )
    }

    /// Check if the error is a rate limit error
    pub fn is_rate_limit_error(&self) -> bool {
        matches!(self, SongbirdError::RateLimitExceeded(_))
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            SongbirdError::Network(_)
                | SongbirdError::Service(_)
                | SongbirdError::CircuitBreakerOpen(_)
                | SongbirdError::CircuitBreakerFailure { .. }
                | SongbirdError::RateLimitExceeded(_)
                | SongbirdError::RetryExhausted(_)
                | SongbirdError::ResourceExhausted(_)
                | SongbirdError::BulkheadFull { .. }
        )
    }

    /// Get the error category
    pub fn category(&self) -> &'static str {
        match self {
            SongbirdError::Communication(_) => "communication",
            SongbirdError::Config { .. }
            | SongbirdError::ConfigField { .. }
            | SongbirdError::Configuration { .. } => "configuration",
            SongbirdError::Io(_) => "io",
            SongbirdError::Network(_) | SongbirdError::NetworkDetection { .. } => "network",
            SongbirdError::Discovery(_) => "discovery",
            SongbirdError::Service(_) => "service",
            SongbirdError::LoadBalancer { .. } => "load_balancer",
            SongbirdError::Protocol(_) => "protocol",
            SongbirdError::Auth(_)
            | SongbirdError::Authentication { .. }
            | SongbirdError::AuthenticationProvider { .. } => "authentication",
            SongbirdError::Gaming(_) => "gaming",
            SongbirdError::Security { .. }
            | SongbirdError::EncryptionFailed { .. }
            | SongbirdError::DecryptionFailed { .. } => "security",
            SongbirdError::Validation(_) => "validation",
            SongbirdError::NotFound(_) => "not_found",
            SongbirdError::TunnelCreation { .. } => "tunnel",
            SongbirdError::UnsupportedChannelType { .. } => "channel",
            SongbirdError::Deployment(_) => "deployment",
            SongbirdError::PluginComposition { .. } | SongbirdError::PluginNotFound(_) => "plugin",
            SongbirdError::RateLimitExceeded(_) => "rate_limit",
            SongbirdError::ExecutionFailed(_) => "execution",
            SongbirdError::ResourceExhausted(_) => "resource",
            SongbirdError::CircuitBreakerOpen(_) | SongbirdError::CircuitBreakerFailure { .. } => {
                "circuit_breaker"
            }
            SongbirdError::RetryExhausted(_) => "retry",
            SongbirdError::BulkheadFull { .. } => "bulkhead",
            SongbirdError::Unknown { .. } => "unknown",
            SongbirdError::BulkheadNotFound { .. } => "bulkhead",
            SongbirdError::HealthCheckerNotFound { .. } => "health_checker",
            SongbirdError::HealthCheckTimeout { .. } => "health_checker",
        }
    }
}
