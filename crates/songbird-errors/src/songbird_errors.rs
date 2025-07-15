//! Comprehensive error types for Songbird Orchestrator

use std::fmt;

#[derive(Debug, Clone)]
pub enum SongbirdError {
    Communication(String),
    Config {
        message: String,
        field: Option<String>,
        suggestion: Option<String>,
        context: Option<String>,
    },
    ConfigField {
        field: String,
        message: String,
        suggestion: Option<String>,
    },
    Configuration {
        field: String,
        message: String,
        suggestion: Option<String>,
    },
    Io {
        message: String,
        path: Option<String>,
        operation: Option<String>,
        suggestion: Option<String>,
    },
    Network {
        service: Option<String>,
        message: String,
        details: Option<String>,
        endpoint: Option<String>,
        suggestion: Option<String>,
    },
    Discovery {
        message: String,
        service: Option<String>,
        timeout: Option<u64>,
        suggestion: Option<String>,
    },
    Service {
        service: String,
        message: String,
        status: Option<String>,
        suggestion: Option<String>,
    },
    LoadBalancer {
        message: String,
        backend: Option<String>,
        suggestion: Option<String>,
    },
    Protocol {
        protocol: String,
        message: String,
        version: Option<String>,
        suggestion: Option<String>,
    },
    Auth {
        message: String,
        user: Option<String>,
        provider: Option<String>,
        suggestion: Option<String>,
    },
    Authentication {
        provider: String,
        message: String,
        suggestion: Option<String>,
    },
    AuthenticationProvider {
        provider: String,
        message: String,
        suggestion: Option<String>,
    },
    Gaming {
        message: String,
        protocol: Option<String>,
        game: Option<String>,
        suggestion: Option<String>,
    },
    Security {
        message: String,
        context: Option<String>,
        severity: Option<String>,
        suggestion: Option<String>,
    },
    Validation {
        field: String,
        message: String,
        value: Option<String>,
        expected: Option<String>,
        suggestion: Option<String>,
    },
    NotFound {
        resource: String,
        message: String,
        searched_paths: Option<Vec<String>>,
        suggestion: Option<String>,
    },
    TunnelCreation {
        message: String,
        tunnel_type: Option<String>,
        endpoint: Option<String>,
        suggestion: Option<String>,
    },
    EncryptionFailed {
        message: String,
        algorithm: Option<String>,
        suggestion: Option<String>,
    },
    DecryptionFailed {
        message: String,
        algorithm: Option<String>,
        suggestion: Option<String>,
    },
    NetworkDetection {
        message: String,
        interface: Option<String>,
        suggestion: Option<String>,
    },
    UnsupportedChannelType {
        channel_type: Option<String>,
        suggestion: Option<String>,
    },
    Deployment {
        service: String,
        message: String,
        environment: Option<String>,
        stage: Option<String>,
        suggestion: Option<String>,
    },
    PluginComposition {
        message: String,
        plugin: Option<String>,
        suggestion: Option<String>,
    },
    PluginNotFound {
        plugin: String,
        searched_paths: Option<Vec<String>>,
        suggestion: Option<String>,
    },
    RateLimitExceeded {
        message: String,
        service: Option<String>,
        limit: Option<u64>,
        suggestion: Option<String>,
    },
    ExecutionFailed {
        message: String,
        command: Option<String>,
        exit_code: Option<i32>,
        suggestion: Option<String>,
    },
    ResourceExhausted {
        resource: String,
        message: String,
        current_usage: Option<String>,
        limit: Option<String>,
        suggestion: Option<String>,
    },
    CircuitBreakerOpen {
        service: String,
        message: String,
        failure_count: Option<u32>,
        suggestion: Option<String>,
    },
    CircuitBreakerFailure {
        service: String,
        message: String,
        suggestion: Option<String>,
    },
    RetryExhausted {
        attempts: u32,
        message: String,
        duration: Option<String>,
        suggestion: Option<String>,
    },
    Generic(String),
}

impl fmt::Display for SongbirdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Communication(msg) => write!(f, "Communication error: {msg}"),
            Self::Config {
                field: Some(field),
                message,
                suggestion,
                context,
            } => {
                write!(f, "Configuration error in field '{field}': {message}")?;
                if let Some(ctx) = context {
                    write!(f, " (Context: {ctx})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Config {
                field: None,
                message,
                suggestion,
                context,
            } => {
                write!(f, "Configuration error: {message}")?;
                if let Some(ctx) = context {
                    write!(f, " (Context: {ctx})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::ConfigField {
                field,
                message,
                suggestion,
            }
            | Self::Configuration {
                field,
                message,
                suggestion,
            } => {
                write!(f, "Configuration error in field '{field}': {message}")?;
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Io {
                message,
                path,
                operation,
                suggestion,
            } => {
                write!(f, "IO error: {message}")?;
                if let Some(p) = path {
                    write!(f, " (Path: {p})")?;
                }
                if let Some(op) = operation {
                    write!(f, " (Operation: {op})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Network {
                service: Some(service),
                message,
                details,
                endpoint,
                suggestion,
            } => {
                write!(f, "Network error in service '{service}': {message}")?;
                if let Some(ep) = endpoint {
                    write!(f, " (Endpoint: {ep})")?;
                }
                if let Some(det) = details {
                    write!(f, " (Details: {det})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Network {
                service: None,
                message,
                details,
                endpoint,
                suggestion,
            } => {
                write!(f, "Network error: {message}")?;
                if let Some(ep) = endpoint {
                    write!(f, " (Endpoint: {ep})")?;
                }
                if let Some(det) = details {
                    write!(f, " (Details: {det})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Discovery {
                service: Some(service),
                message,
                timeout,
                suggestion,
            } => {
                write!(f, "Discovery error for service '{service}': {message}")?;
                if let Some(t) = timeout {
                    write!(f, " (Timeout: {t}s)")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Discovery {
                service: None,
                message,
                timeout,
                suggestion,
            } => {
                write!(f, "Discovery error: {message}")?;
                if let Some(t) = timeout {
                    write!(f, " (Timeout: {t}s)")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Service {
                service,
                message,
                status,
                suggestion,
            } => {
                write!(f, "Service error [{service}]: {message}")?;
                if let Some(st) = status {
                    write!(f, " (Status: {st})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::LoadBalancer {
                message,
                backend,
                suggestion,
            } => {
                write!(f, "Load balancer error: {message}")?;
                if let Some(be) = backend {
                    write!(f, " (Backend: {be})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Protocol {
                protocol,
                message,
                version,
                suggestion,
            } => {
                write!(f, "Protocol error [{protocol}]: {message}")?;
                if let Some(v) = version {
                    write!(f, " (Version: {v})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Auth {
                message,
                user: Some(user),
                provider,
                suggestion,
            } => {
                write!(f, "Authentication error for user '{user}': {message}")?;
                if let Some(prov) = provider {
                    write!(f, " (Provider: {prov})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Auth {
                message,
                user: None,
                provider,
                suggestion,
            } => {
                write!(f, "Authentication error: {message}")?;
                if let Some(prov) = provider {
                    write!(f, " (Provider: {prov})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Authentication {
                provider,
                message,
                suggestion,
            }
            | Self::AuthenticationProvider {
                provider,
                message,
                suggestion,
            } => {
                write!(
                    f,
                    "Authentication error from provider '{provider}': {message}"
                )?;
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Gaming {
                protocol: Some(protocol),
                message,
                game,
                suggestion,
            } => {
                write!(f, "Gaming error for protocol '{protocol}': {message}")?;
                if let Some(g) = game {
                    write!(f, " (Game: {g})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Gaming {
                protocol: None,
                message,
                game,
                suggestion,
            } => {
                write!(f, "Gaming error: {message}")?;
                if let Some(g) = game {
                    write!(f, " (Game: {g})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Security {
                context: Some(context),
                message,
                severity,
                suggestion,
            } => {
                write!(f, "Security error in context '{context}': {message}")?;
                if let Some(sev) = severity {
                    write!(f, " (Severity: {sev})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Security {
                context: None,
                message,
                severity,
                suggestion,
            } => {
                write!(f, "Security error: {message}")?;
                if let Some(sev) = severity {
                    write!(f, " (Severity: {sev})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Validation {
                field,
                message,
                value,
                expected,
                suggestion,
            } => {
                write!(f, "Validation error for field '{field}': {message}")?;
                if let Some(v) = value {
                    write!(f, " (Value: {v})")?;
                }
                if let Some(exp) = expected {
                    write!(f, " (Expected: {exp})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::NotFound {
                resource,
                message,
                searched_paths,
                suggestion,
            } => {
                write!(f, "Resource '{resource}' not found: {message}")?;
                if let Some(paths) = searched_paths {
                    write!(f, " (Searched: {})", paths.join(", "))?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::TunnelCreation {
                message,
                tunnel_type,
                endpoint,
                suggestion,
            } => {
                write!(f, "Tunnel creation error: {message}")?;
                if let Some(tt) = tunnel_type {
                    write!(f, " (Type: {tt})")?;
                }
                if let Some(ep) = endpoint {
                    write!(f, " (Endpoint: {ep})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::EncryptionFailed {
                message,
                algorithm,
                suggestion,
            } => {
                write!(f, "Encryption failed: {message}")?;
                if let Some(alg) = algorithm {
                    write!(f, " (Algorithm: {alg})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::DecryptionFailed {
                message,
                algorithm,
                suggestion,
            } => {
                write!(f, "Decryption failed: {message}")?;
                if let Some(alg) = algorithm {
                    write!(f, " (Algorithm: {alg})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::NetworkDetection {
                message,
                interface,
                suggestion,
            } => {
                write!(f, "Network detection error: {message}")?;
                if let Some(iface) = interface {
                    write!(f, " (Interface: {iface})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::UnsupportedChannelType {
                channel_type,
                suggestion,
            } => {
                write!(f, "Unsupported channel type error")?;
                if let Some(ct) = channel_type {
                    write!(f, " (Type: {ct})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Deployment {
                service,
                message,
                environment,
                stage,
                suggestion,
            } => {
                write!(f, "Deployment error for service '{service}': {message}")?;
                if let Some(env) = environment {
                    write!(f, " (Environment: {env})")?;
                }
                if let Some(st) = stage {
                    write!(f, " (Stage: {st})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::PluginComposition {
                message,
                plugin,
                suggestion,
            } => {
                write!(f, "Plugin composition failed: {message}")?;
                if let Some(p) = plugin {
                    write!(f, " (Plugin: {p})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::PluginNotFound {
                plugin,
                searched_paths,
                suggestion,
            } => {
                write!(f, "Plugin not found: {plugin}")?;
                if let Some(paths) = searched_paths {
                    write!(f, " (Searched: {})", paths.join(", "))?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::RateLimitExceeded {
                message,
                service,
                limit,
                suggestion,
            } => {
                write!(f, "Rate limit exceeded: {message}")?;
                if let Some(svc) = service {
                    write!(f, " (Service: {svc})")?;
                }
                if let Some(l) = limit {
                    write!(f, " (Limit: {l}/s)")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::ExecutionFailed {
                message,
                command,
                exit_code,
                suggestion,
            } => {
                write!(f, "Execution failed: {message}")?;
                if let Some(cmd) = command {
                    write!(f, " (Command: {cmd})")?;
                }
                if let Some(code) = exit_code {
                    write!(f, " (Exit code: {code})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::ResourceExhausted {
                resource,
                message,
                current_usage,
                limit,
                suggestion,
            } => {
                write!(f, "Resource exhausted [{resource}]: {message}")?;
                if let Some(usage) = current_usage {
                    write!(f, " (Usage: {usage})")?;
                }
                if let Some(l) = limit {
                    write!(f, " (Limit: {l})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::CircuitBreakerOpen {
                service,
                message,
                failure_count,
                suggestion,
            } => {
                write!(f, "Circuit breaker open for service '{service}': {message}")?;
                if let Some(count) = failure_count {
                    write!(f, " (Failures: {count})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::CircuitBreakerFailure {
                service,
                message,
                suggestion,
            } => {
                write!(
                    f,
                    "Circuit breaker failure for service '{service}': {message}"
                )?;
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::RetryExhausted {
                attempts,
                message,
                duration,
                suggestion,
            } => {
                write!(f, "Retry exhausted after {attempts} attempts: {message}")?;
                if let Some(dur) = duration {
                    write!(f, " (Duration: {dur})")?;
                }
                if let Some(sug) = suggestion {
                    write!(f, " [Suggestion: {sug}]")?;
                }
                Ok(())
            }
            Self::Generic(msg) => write!(f, "Generic error: {msg}"),
        }
    }
}

impl std::error::Error for SongbirdError {}

impl SongbirdError {
    /// Create a service error with context
    #[must_use]
    pub fn service_error(service_id: &str, message: String) -> Self {
        Self::Service {
            service: service_id.to_string(),
            message,
            status: None,
            suggestion: Some("Check service logs and configuration".to_string()),
        }
    }

    /// Create a service error with status and suggestion
    #[must_use]
    pub fn service_error_with_status(service_id: &str, message: String, status: &str) -> Self {
        Self::Service {
            service: service_id.to_string(),
            message,
            status: Some(status.to_string()),
            suggestion: Some(format!(
                "Service is {status}. Check service health and restart if needed"
            )),
        }
    }

    /// Create a health check failed error
    #[must_use]
    pub fn health_check_failed(service_id: &str, message: &str) -> Self {
        Self::Service {
            service: service_id.to_string(),
            message: format!("Health check failed: {message}"),
            status: Some("unhealthy".to_string()),
            suggestion: Some(
                "Check service status with 'songbird status --detailed' and restart if needed"
                    .to_string(),
            ),
        }
    }

    /// Create a configuration error with suggestion
    #[must_use]
    pub fn config_error(field: &str, message: &str, suggestion: &str) -> Self {
        Self::Config {
            field: Some(field.to_string()),
            message: message.to_string(),
            suggestion: Some(suggestion.to_string()),
            context: None,
        }
    }

    /// Create a configuration error with context
    #[must_use]
    pub fn config_error_with_context(
        field: &str,
        message: &str,
        context: &str,
        suggestion: &str,
    ) -> Self {
        Self::Config {
            field: Some(field.to_string()),
            message: message.to_string(),
            suggestion: Some(suggestion.to_string()),
            context: Some(context.to_string()),
        }
    }

    /// Create a network error with endpoint and suggestion
    #[must_use]
    pub fn network_error(service: &str, message: &str, endpoint: &str) -> Self {
        Self::Network {
            service: Some(service.to_string()),
            message: message.to_string(),
            details: None,
            endpoint: Some(endpoint.to_string()),
            suggestion: Some("Check network connectivity and endpoint configuration".to_string()),
        }
    }

    /// Create a validation error with expected value
    #[must_use]
    pub fn validation_error(field: &str, message: &str, value: &str, expected: &str) -> Self {
        Self::Validation {
            field: field.to_string(),
            message: message.to_string(),
            value: Some(value.to_string()),
            expected: Some(expected.to_string()),
            suggestion: Some(format!(
                "Update {field} to match expected format: {expected}"
            )),
        }
    }

    /// Create a resource not found error with search paths
    #[must_use]
    pub fn resource_not_found(resource: &str, message: &str, searched_paths: Vec<String>) -> Self {
        Self::NotFound {
            resource: resource.to_string(),
            message: message.to_string(),
            searched_paths: Some(searched_paths),
            suggestion: Some("Check the resource path and ensure it exists".to_string()),
        }
    }

    /// Create a security error with severity
    #[must_use]
    pub fn security_error(context: &str, message: &str, severity: &str) -> Self {
        Self::Security {
            context: Some(context.to_string()),
            message: message.to_string(),
            severity: Some(severity.to_string()),
            suggestion: Some(
                "Review security configuration and apply recommended fixes".to_string(),
            ),
        }
    }

    /// Create an execution error with command and exit code
    #[must_use]
    pub fn execution_error(message: &str, command: &str, exit_code: i32) -> Self {
        Self::ExecutionFailed {
            message: message.to_string(),
            command: Some(command.to_string()),
            exit_code: Some(exit_code),
            suggestion: Some("Check command syntax and system requirements".to_string()),
        }
    }

    /// Create a legacy configuration error for backward compatibility
    #[must_use]
    pub const fn configuration_error(message: String) -> Self {
        Self::Config {
            message,
            field: None,
            suggestion: None,
            context: None,
        }
    }

    /// Get the suggestion for recovery, if available
    #[must_use]
    pub fn get_suggestion(&self) -> Option<&str> {
        match self {
            Self::Config { suggestion, .. }
            | Self::ConfigField { suggestion, .. }
            | Self::Configuration { suggestion, .. }
            | Self::Io { suggestion, .. }
            | Self::Network { suggestion, .. }
            | Self::Discovery { suggestion, .. }
            | Self::Service { suggestion, .. }
            | Self::LoadBalancer { suggestion, .. }
            | Self::Protocol { suggestion, .. }
            | Self::Auth { suggestion, .. }
            | Self::Authentication { suggestion, .. }
            | Self::AuthenticationProvider { suggestion, .. }
            | Self::Gaming { suggestion, .. }
            | Self::Security { suggestion, .. }
            | Self::Validation { suggestion, .. }
            | Self::NotFound { suggestion, .. }
            | Self::TunnelCreation { suggestion, .. }
            | Self::EncryptionFailed { suggestion, .. }
            | Self::DecryptionFailed { suggestion, .. }
            | Self::NetworkDetection { suggestion, .. }
            | Self::UnsupportedChannelType { suggestion, .. }
            | Self::Deployment { suggestion, .. }
            | Self::PluginComposition { suggestion, .. }
            | Self::PluginNotFound { suggestion, .. }
            | Self::RateLimitExceeded { suggestion, .. }
            | Self::ExecutionFailed { suggestion, .. }
            | Self::ResourceExhausted { suggestion, .. }
            | Self::CircuitBreakerOpen { suggestion, .. }
            | Self::CircuitBreakerFailure { suggestion, .. }
            | Self::RetryExhausted { suggestion, .. } => suggestion.as_deref(),
            _ => None,
        }
    }

    /// Get the severity level of the error
    #[must_use]
    pub fn get_severity(&self) -> &str {
        match self {
            Self::Security {
                severity: Some(sev),
                ..
            } => sev,
            Self::Security { .. } => "medium",
            Self::Config { .. } | Self::ConfigField { .. } | Self::Configuration { .. } => "high",
            Self::Auth { .. }
            | Self::Authentication { .. }
            | Self::AuthenticationProvider { .. } => "high",
            Self::EncryptionFailed { .. } | Self::DecryptionFailed { .. } => "critical",
            Self::ResourceExhausted { .. } => "critical",
            Self::CircuitBreakerOpen { .. } | Self::CircuitBreakerFailure { .. } => "high",
            _ => "medium",
        }
    }

    /// Check if this error is recoverable
    #[must_use]
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::Network { .. }
            | Self::Discovery { .. }
            | Self::Service { .. }
            | Self::LoadBalancer { .. }
            | Self::RateLimitExceeded { .. }
            | Self::CircuitBreakerOpen { .. }
            | Self::RetryExhausted { .. } => true,
            Self::Config { .. }
            | Self::ConfigField { .. }
            | Self::Configuration { .. }
            | Self::Validation { .. } => false,
            _ => true,
        }
    }
}

// From implementations for seamless error conversion
impl From<std::string::String> for SongbirdError {
    fn from(message: String) -> Self {
        SongbirdError::Generic(message)
    }
}

impl From<url::ParseError> for SongbirdError {
    fn from(error: url::ParseError) -> Self {
        SongbirdError::Generic(format!("URL parse error: {}", error))
    }
}

impl From<&str> for SongbirdError {
    fn from(message: &str) -> Self {
        SongbirdError::Generic(message.to_string())
    }
}

impl From<std::io::Error> for SongbirdError {
    fn from(err: std::io::Error) -> Self {
        Self::Io {
            message: err.to_string(),
            path: None,
            operation: None,
            suggestion: Some("Check file permissions and disk space".to_string()),
        }
    }
}

impl From<std::net::AddrParseError> for SongbirdError {
    fn from(err: std::net::AddrParseError) -> Self {
        Self::Network {
            service: None,
            message: format!("Address parsing error: {err}"),
            details: None,
            endpoint: None,
            suggestion: Some("Check address format (e.g., 127.0.0.1:8080)".to_string()),
        }
    }
}

impl From<std::time::SystemTimeError> for SongbirdError {
    fn from(err: std::time::SystemTimeError) -> Self {
        Self::Io {
            message: format!("System time error: {err}"),
            path: None,
            operation: Some("time_calculation".to_string()),
            suggestion: Some("Check system time and timezone settings".to_string()),
        }
    }
}

impl From<serde_json::Error> for SongbirdError {
    fn from(err: serde_json::Error) -> Self {
        Self::Network {
            service: None,
            message: format!("JSON serialization error: {err}"),
            details: None,
            endpoint: None,
            suggestion: Some("Check JSON format and data types".to_string()),
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
            suggestion: Some("Check configuration file".to_string()),
            context: Some("config.toml".to_string()),
        };
        assert!(error.to_string().contains("timeout"));
        assert!(error.to_string().contains("Invalid value"));
        assert!(error.to_string().contains("Check configuration file"));
        assert!(error.to_string().contains("config.toml"));
    }

    #[test]
    fn test_error_display_formatting_no_field() {
        let error = SongbirdError::Config {
            message: "Invalid configuration".to_string(),
            field: None,
            suggestion: Some("Review configuration file".to_string()),
            context: None,
        };
        assert!(error.to_string().contains("Invalid configuration"));
        assert!(!error.to_string().contains("field"));
        assert!(error.to_string().contains("Review configuration file"));
    }

    #[test]
    fn test_service_error_creation() {
        let error = SongbirdError::service_error("test-service", "Test message".to_string());
        assert!(error.to_string().contains("test-service"));
        assert!(error.to_string().contains("Test message"));
        assert!(error
            .to_string()
            .contains("Check service logs and configuration"));
    }

    #[test]
    fn test_service_error_with_status_creation() {
        let error = SongbirdError::service_error_with_status(
            "test-service",
            "Service is unhealthy".to_string(),
            "unhealthy",
        );
        assert!(error.to_string().contains("test-service"));
        assert!(error.to_string().contains("Service is unhealthy"));
        assert!(error
            .to_string()
            .contains("unhealthy"));
    }

    #[test]
    fn test_health_check_failed_creation() {
        let error = SongbirdError::health_check_failed("db-service", "Timeout");
        assert!(error.to_string().contains("db-service"));
        assert!(error.to_string().contains("Health check failed"));
        assert!(error.to_string().contains(
            "Check service status with 'songbird status --detailed' and restart if needed"
        ));
    }

    #[test]
    fn test_configuration_error_creation() {
        let error = SongbirdError::configuration_error("Invalid config".to_string());
        assert!(error.to_string().contains("Invalid config"));
    }

    #[test]
    fn test_config_error_creation() {
        let error =
            SongbirdError::config_error("timeout", "Invalid value", "Check configuration file");
        assert!(error.to_string().contains("timeout"));
        assert!(error.to_string().contains("Invalid value"));
        assert!(error.to_string().contains("Check configuration file"));
    }

    #[test]
    fn test_config_error_with_context_creation() {
        let error = SongbirdError::config_error_with_context(
            "timeout",
            "Invalid value",
            "config.toml",
            "Check configuration file",
        );
        assert!(error.to_string().contains("timeout"));
        assert!(error.to_string().contains("Invalid value"));
        assert!(error.to_string().contains("config.toml"));
        assert!(error.to_string().contains("Check configuration file"));
    }

    #[test]
    fn test_network_error_creation() {
        let error = SongbirdError::network_error(
            "api-service",
            "Connection failed",
            "https://api.example.com/v1",
        );
        assert!(error.to_string().contains("api-service"));
        assert!(error.to_string().contains("Connection failed"));
        assert!(error.to_string().contains("https://api.example.com/v1"));
        assert!(error
            .to_string()
            .contains("Check network connectivity and endpoint configuration"));
    }

    #[test]
    fn test_validation_error_creation() {
        let error =
            SongbirdError::validation_error("port", "Invalid port number", "8080", "1024-65535");
        assert!(error.to_string().contains("port"));
        assert!(error.to_string().contains("Invalid port number"));
        assert!(error.to_string().contains("8080"));
        assert!(error.to_string().contains("1024-65535"));
        assert!(error
            .to_string()
            .contains("Update port to match expected format: 1024-65535"));
    }

    #[test]
    fn test_resource_not_found_creation() {
        let error = SongbirdError::resource_not_found(
            "config.toml",
            "File not found",
            vec![
                "/etc/songbird/config.toml".to_string(),
                "/home/user/.config/songbird/config.toml".to_string(),
            ],
        );
        assert!(error.to_string().contains("config.toml"));
        assert!(error.to_string().contains("File not found"));
        assert!(error.to_string().contains("/etc/songbird/config.toml"));
        assert!(error
            .to_string()
            .contains("/home/user/.config/songbird/config.toml"));
        assert!(error
            .to_string()
            .contains("Check the resource path and ensure it exists"));
    }

    #[test]
    fn test_security_error_creation() {
        let error = SongbirdError::security_error("admin_panel", "Access denied", "high");
        assert!(error.to_string().contains("admin_panel"));
        assert!(error.to_string().contains("Access denied"));
        assert!(error.to_string().contains("high"));
        assert!(error
            .to_string()
            .contains("Review security configuration and apply recommended fixes"));
    }

    #[test]
    fn test_execution_error_creation() {
        let error = SongbirdError::execution_error("Command failed", "ls -l", 1);
        assert!(error.to_string().contains("Command failed"));
        assert!(error.to_string().contains("ls -l"));
        assert!(error.to_string().contains("1"));
        assert!(error
            .to_string()
            .contains("Check command syntax and system requirements"));
    }

    #[test]
    fn test_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let error: SongbirdError = io_error.into();
        assert!(error.to_string().contains("File not found"));
        assert!(error
            .to_string()
            .contains("Check file permissions and disk space"));
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
        let addr_error = "invalid_address:port"
            .parse::<std::net::SocketAddr>()
            .unwrap_err();
        let error: SongbirdError = addr_error.into();
        assert!(error.to_string().contains("Address parsing error"));
        assert!(error
            .to_string()
            .contains("Check address format (e.g., 127.0.0.1:8080)"));
    }

    #[test]
    fn test_from_system_time_error() {
        let time_error = std::time::SystemTime::UNIX_EPOCH
            .duration_since(std::time::SystemTime::now())
            .unwrap_err();
        let error: SongbirdError = time_error.into();
        assert!(error.to_string().contains("System time error"));
        assert!(error
            .to_string()
            .contains("Check system time and timezone settings"));
    }

    #[test]
    fn test_from_json_error() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let error: SongbirdError = json_error.into();
        assert!(error.to_string().contains("JSON serialization error"));
        assert!(error
            .to_string()
            .contains("Check JSON format and data types"));
    }

    #[test]
    fn test_error_variants_coverage() {
        // Test that all major error variants can be created and displayed
        let errors = vec![
            SongbirdError::Communication("test".to_string()),
            SongbirdError::TunnelCreation {
                message: "tunnel error".to_string(),
                tunnel_type: Some("ssh".to_string()),
                endpoint: Some("localhost:22".to_string()),
                suggestion: Some("Check SSH connection and tunnel configuration".to_string()),
            },
            SongbirdError::EncryptionFailed {
                message: "encryption error".to_string(),
                algorithm: Some("AES-256".to_string()),
                suggestion: Some("Check encryption key and algorithm".to_string()),
            },
            SongbirdError::DecryptionFailed {
                message: "decryption error".to_string(),
                algorithm: Some("AES-256".to_string()),
                suggestion: Some("Check decryption key and algorithm".to_string()),
            },
            SongbirdError::NetworkDetection {
                message: "network error".to_string(),
                interface: Some("eth0".to_string()),
                suggestion: Some("Check network interface and configuration".to_string()),
            },
            SongbirdError::UnsupportedChannelType {
                channel_type: Some("unknown".to_string()),
                suggestion: Some("Check channel type and configuration".to_string()),
            },
            SongbirdError::PluginComposition {
                message: "composition error".to_string(),
                plugin: Some("plugin_a".to_string()),
                suggestion: Some("Check plugin dependencies and configuration".to_string()),
            },
            SongbirdError::PluginNotFound {
                plugin: "plugin_b".to_string(),
                searched_paths: Some(vec![
                    "/usr/local/lib/songbird/plugins/plugin_b.so".to_string(),
                    "/home/user/.config/songbird/plugins/plugin_b.so".to_string(),
                ]),
                suggestion: Some("Check plugin path and ensure it exists".to_string()),
            },
            SongbirdError::RateLimitExceeded {
                message: "rate limit error".to_string(),
                service: Some("api-service".to_string()),
                limit: Some(100),
                suggestion: Some("Increase rate limit or optimize requests".to_string()),
            },
            SongbirdError::ExecutionFailed {
                message: "execution error".to_string(),
                command: Some("npm install".to_string()),
                exit_code: Some(1),
                suggestion: Some("Check npm installation and dependencies".to_string()),
            },
        ];

        for error in errors {
            let display_str = error.to_string();
            assert!(!display_str.is_empty());
            // Each error should produce a meaningful error message
            assert!(display_str.contains("error") || display_str.contains("failed"));
            assert!(display_str.contains("Suggestion:"));
        }
    }

    #[test]
    fn test_network_error_variants() {
        let error_with_details = SongbirdError::Network {
            service: Some("test-service".to_string()),
            message: "Connection failed".to_string(),
            details: Some("Timeout after 30s".to_string()),
            endpoint: Some("https://api.example.com/v1".to_string()),
            suggestion: Some("Check network connectivity and endpoint configuration".to_string()),
        };
        let display = error_with_details.to_string();
        assert!(display.contains("test-service"));
        assert!(display.contains("Connection failed"));
        assert!(display.contains("Timeout after 30s"));
        assert!(display.contains("https://api.example.com/v1"));
        assert!(display.contains("Check network connectivity and endpoint configuration"));

        let error_without_details = SongbirdError::Network {
            service: Some("test-service".to_string()),
            message: "Connection failed".to_string(),
            details: None,
            endpoint: Some("https://api.example.com/v1".to_string()),
            suggestion: Some("Check network connectivity and endpoint configuration".to_string()),
        };
        let display = error_without_details.to_string();
        assert!(display.contains("test-service"));
        assert!(display.contains("Connection failed"));
        assert!(display.contains("https://api.example.com/v1"));
        assert!(display.contains("Check network connectivity and endpoint configuration"));
    }

    #[test]
    fn test_discovery_error_variants() {
        let error_with_service = SongbirdError::Discovery {
            message: "Service not found".to_string(),
            service: Some("api-service".to_string()),
            timeout: Some(10),
            suggestion: Some("Check service availability and endpoint".to_string()),
        };
        let display = error_with_service.to_string();
        assert!(display.contains("api-service"));
        assert!(display.contains("Service not found"));
        assert!(display.contains("10"));
        assert!(display.contains("Check service availability and endpoint"));

        let error_without_service = SongbirdError::Discovery {
            message: "Discovery failed".to_string(),
            service: None,
            timeout: Some(5),
            suggestion: Some("Check service discovery mechanism".to_string()),
        };
        let display = error_without_service.to_string();
        assert!(display.contains("Discovery failed"));
        assert!(!display.contains("api-service"));
        assert!(display.contains("5"));
        assert!(display.contains("Check service discovery mechanism"));
    }

    #[test]
    fn test_auth_error_variants() {
        let error_with_user = SongbirdError::Auth {
            message: "Invalid credentials".to_string(),
            user: Some("testuser".to_string()),
            provider: Some("local".to_string()),
            suggestion: Some("Check user credentials and authentication provider".to_string()),
        };
        let display = error_with_user.to_string();
        assert!(display.contains("testuser"));
        assert!(display.contains("Invalid credentials"));
        assert!(display.contains("local"));
        assert!(display.contains("Check user credentials and authentication provider"));

        let error_without_user = SongbirdError::Auth {
            message: "Authentication failed".to_string(),
            user: None,
            provider: Some("local".to_string()),
            suggestion: Some("Check authentication provider configuration".to_string()),
        };
        let display = error_without_user.to_string();
        assert!(display.contains("Authentication failed"));
        assert!(!display.contains("testuser"));
        assert!(display.contains("local"));
        assert!(display.contains("Check authentication provider configuration"));
    }

    #[test]
    fn test_gaming_error_variants() {
        let error_with_protocol = SongbirdError::Gaming {
            message: "Protocol mismatch".to_string(),
            protocol: Some("IPX".to_string()),
            game: Some("Counter-Strike".to_string()),
            suggestion: Some("Check game version and protocol compatibility".to_string()),
        };
        let display = error_with_protocol.to_string();
        assert!(display.contains("IPX"));
        assert!(display.contains("Protocol mismatch"));
        assert!(display.contains("Counter-Strike"));
        assert!(display.contains("Check game version and protocol compatibility"));

        let error_without_protocol = SongbirdError::Gaming {
            message: "Gaming error".to_string(),
            protocol: None,
            game: Some("Dota 2".to_string()),
            suggestion: Some("Check game installation and dependencies".to_string()),
        };
        let display = error_without_protocol.to_string();
        assert!(display.contains("Gaming error"));
        assert!(!display.contains("IPX"));
        assert!(display.contains("Dota 2"));
        assert!(display.contains("Check game installation and dependencies"));
    }

    #[test]
    fn test_security_error_variants() {
        let error_with_context = SongbirdError::Security {
            message: "Access denied".to_string(),
            context: Some("admin_panel".to_string()),
            severity: Some("high".to_string()),
            suggestion: Some(
                "Review security configuration and apply recommended fixes".to_string(),
            ),
        };
        let display = error_with_context.to_string();
        assert!(display.contains("admin_panel"));
        assert!(display.contains("Access denied"));
        assert!(display.contains("high"));
        assert!(display.contains("Review security configuration and apply recommended fixes"));

        let error_without_context = SongbirdError::Security {
            message: "Security violation".to_string(),
            context: None,
            severity: Some("medium".to_string()),
            suggestion: Some(
                "Review security configuration and apply recommended fixes".to_string(),
            ),
        };
        let display = error_without_context.to_string();
        assert!(display.contains("Security violation"));
        assert!(!display.contains("admin_panel"));
        assert!(display.contains("medium"));
        assert!(display.contains("Review security configuration and apply recommended fixes"));
    }

    #[test]
    fn test_complex_error_variants() {
        let resource_exhausted = SongbirdError::ResourceExhausted {
            resource: "memory".to_string(),
            message: "Out of memory".to_string(),
            current_usage: Some("90%".to_string()),
            limit: Some("100%".to_string()),
            suggestion: Some("Check system resources and optimize applications".to_string()),
        };
        assert!(resource_exhausted.to_string().contains("memory"));
        assert!(resource_exhausted.to_string().contains("Out of memory"));
        assert!(resource_exhausted.to_string().contains("90%"));
        assert!(resource_exhausted.to_string().contains("100%"));
        assert!(resource_exhausted
            .to_string()
            .contains("Check system resources and optimize applications"));

        let circuit_breaker = SongbirdError::CircuitBreakerOpen {
            service: "payment-service".to_string(),
            message: "Too many failures".to_string(),
            failure_count: Some(5),
            suggestion: Some("Check service health and restart if needed".to_string()),
        };
        assert!(circuit_breaker.to_string().contains("payment-service"));
        assert!(circuit_breaker.to_string().contains("Too many failures"));
        assert!(circuit_breaker.to_string().contains("5"));
        assert!(circuit_breaker
            .to_string()
            .contains("Check service health and restart if needed"));

        let retry_exhausted = SongbirdError::RetryExhausted {
            attempts: 5,
            message: "Max retries reached".to_string(),
            duration: Some("10s".to_string()),
            suggestion: Some("Check network connectivity and retry strategy".to_string()),
        };
        assert!(retry_exhausted.to_string().contains('5'));
        assert!(retry_exhausted.to_string().contains("Max retries reached"));
        assert!(retry_exhausted.to_string().contains("10s"));
        assert!(retry_exhausted
            .to_string()
            .contains("Check network connectivity and retry strategy"));
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
            suggestion: Some("Debug suggestion".to_string()),
            context: Some("debug_context".to_string()),
        };
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("Config"));
        assert!(debug_str.contains("Debug test"));
        assert!(debug_str.contains("test_field"));
        assert!(debug_str.contains("Debug suggestion"));
        assert!(debug_str.contains("debug_context"));
    }

    #[test]
    fn test_error_clone() {
        let error = SongbirdError::Communication("Clone test".to_string());
        let cloned = error.clone();
        assert_eq!(error.to_string(), cloned.to_string());
    }

    #[test]
    fn test_result_type_alias() {
        fn test_function() -> String {
            "Success".to_string()
        }

        fn test_function_error() -> Result<String> {
            Err(SongbirdError::Communication("Error".to_string()))
        }

        assert_eq!(test_function(), "Success");
        assert!(test_function_error().is_err());
    }
}
