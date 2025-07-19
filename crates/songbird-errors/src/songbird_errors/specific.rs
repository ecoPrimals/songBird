use std::fmt;

/// IO-related error
#[derive(Debug, Clone)]
pub struct IoError {
    pub message: String,
    pub operation: Option<String>,
    pub path: Option<String>,
}

/// Network-related error
#[derive(Debug, Clone)]
pub struct NetworkError {
    pub message: String,
    pub endpoint: Option<String>,
    pub port: Option<u16>,
    pub protocol: Option<String>,
}

/// Service discovery error
#[derive(Debug, Clone)]
pub struct DiscoveryError {
    pub message: String,
    pub service: Option<String>,
    pub timeout: Option<u64>,
    pub suggestion: Option<String>,
}

/// Service-related error
#[derive(Debug, Clone)]
pub struct ServiceError {
    pub service: String,
    pub message: String,
    pub status: Option<String>,
    pub suggestion: Option<String>,
}

/// Protocol-related error
#[derive(Debug, Clone)]
pub struct ProtocolError {
    pub message: String,
    pub protocol: Option<String>,
}

/// Authentication error
#[derive(Debug, Clone)]
pub struct AuthError {
    pub message: String,
    pub provider: Option<String>,
}

/// Gaming-related error
#[derive(Debug, Clone)]
pub struct GamingError {
    pub message: String,
    pub game: Option<String>,
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
    pub field: Option<String>,
    pub expected: Option<String>,
}

/// Not found error
#[derive(Debug, Clone)]
pub struct NotFoundError {
    pub message: String,
    pub resource: Option<String>,
}

/// Deployment error
#[derive(Debug, Clone)]
pub struct DeploymentError {
    pub message: String,
    pub environment: Option<String>,
    pub component: Option<String>,
}

/// Plugin not found error
#[derive(Debug, Clone)]
pub struct PluginNotFoundError {
    pub plugin: String,
}

/// Rate limit error
#[derive(Debug, Clone)]
pub struct RateLimitError {
    pub message: String,
    pub limit: Option<u64>,
}

/// Execution error
#[derive(Debug, Clone)]
pub struct ExecutionError {
    pub message: String,
    pub command: Option<String>,
}

/// Resource exhausted error
#[derive(Debug, Clone)]
pub struct ResourceExhaustedError {
    pub message: String,
    pub resource: Option<String>,
    pub current: Option<u64>,
}

/// Circuit breaker error
#[derive(Debug, Clone)]
pub struct CircuitBreakerError {
    pub service: String,
    pub message: String,
}

/// Retry error
#[derive(Debug, Clone)]
pub struct RetryError {
    pub message: String,
    pub attempts: Option<u32>,
}

// Display implementations for all error types
impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(operation) = &self.operation {
            write!(f, " (operation: {operation})")?;
        }
        if let Some(path) = &self.path {
            write!(f, " (path: {path})")?;
        }
        Ok(())
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(endpoint) = &self.endpoint {
            write!(f, " (endpoint: {endpoint})")?;
        }
        if let Some(port) = &self.port {
            write!(f, " (port: {port})")?;
        }
        if let Some(protocol) = &self.protocol {
            write!(f, " (protocol: {protocol})")?;
        }
        Ok(())
    }
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(service) = &self.service {
            write!(f, " (service: {service})")?;
        }
        if let Some(timeout) = &self.timeout {
            write!(f, " (timeout: {timeout}s)")?;
        }
        if let Some(suggestion) = &self.suggestion {
            write!(f, " - Suggestion: {suggestion}")?;
        }
        Ok(())
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Service '{}': {}", self.service, self.message)?;
        if let Some(status) = &self.status {
            write!(f, " (status: {status})")?;
        }
        if let Some(suggestion) = &self.suggestion {
            write!(f, " - Suggestion: {suggestion}")?;
        }
        Ok(())
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(protocol) = &self.protocol {
            write!(f, " (protocol: {protocol})")?;
        }
        Ok(())
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(provider) = &self.provider {
            write!(f, " (provider: {provider})")?;
        }
        Ok(())
    }
}

impl fmt::Display for GamingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(game) = &self.game {
            write!(f, " (game: {game})")?;
        }
        Ok(())
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(field) = &self.field {
            write!(f, " (field: {field})")?;
        }
        if let Some(expected) = &self.expected {
            write!(f, " (expected: {expected})")?;
        }
        Ok(())
    }
}

impl fmt::Display for NotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(resource) = &self.resource {
            write!(f, " (resource: {resource})")?;
        }
        Ok(())
    }
}

impl fmt::Display for DeploymentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(environment) = &self.environment {
            write!(f, " (environment: {environment})")?;
        }
        if let Some(component) = &self.component {
            write!(f, " (component: {component})")?;
        }
        Ok(())
    }
}

impl fmt::Display for PluginNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Plugin '{}' not found", self.plugin)
    }
}

impl fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(limit) = &self.limit {
            write!(f, " (limit: {limit})")?;
        }
        Ok(())
    }
}

impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(command) = &self.command {
            write!(f, " (command: {command})")?;
        }
        Ok(())
    }
}

impl fmt::Display for ResourceExhaustedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(resource) = &self.resource {
            write!(f, " (resource: {resource})")?;
        }
        if let Some(current) = &self.current {
            write!(f, " (current: {current})")?;
        }
        Ok(())
    }
}

impl fmt::Display for CircuitBreakerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Circuit breaker for service '{}': {}",
            self.service, self.message
        )
    }
}

impl fmt::Display for RetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(attempts) = &self.attempts {
            write!(f, " (attempts: {attempts})")?;
        }
        Ok(())
    }
}
