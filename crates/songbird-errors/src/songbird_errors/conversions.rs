use songbird_types::errors::SongbirdError;
use super::specific::*;

// From implementations for seamless error conversion
impl From<std::string::String> for SongbirdError {
    fn from(msg: String) -> Self {
        SongbirdError::Communication(msg)
    }
}

impl From<url::ParseError> for SongbirdError {
    fn from(err: url::ParseError) -> Self {
        SongbirdError::Communication(format!("URL parse error: {err}"))
    }
}

impl From<&str> for SongbirdError {
    fn from(msg: &str) -> Self {
        SongbirdError::Communication(msg.to_string())
    }
}

impl From<std::io::Error> for SongbirdError {
    fn from(err: std::io::Error) -> Self {
        SongbirdError::Io(Box::new(IoError {
            message: err.to_string(),
            operation: None,
            path: None,
        }))
    }
}

impl From<std::net::AddrParseError> for SongbirdError {
    fn from(err: std::net::AddrParseError) -> Self {
        SongbirdError::network(format!("Address parse error: {err),
            endpoint: None,
            port: None,
            protocol: None,
        }))
    }
}

impl From<std::time::SystemTimeError> for SongbirdError {
    fn from(err: std::time::SystemTimeError) -> Self {
        SongbirdError::Io(Box::new(IoError {
            message: format!("System time error: {err}"),
            operation: Some("time_calculation".to_string()),
            path: None,
        }))
    }
}

impl From<serde_json::Error> for SongbirdError {
    fn from(err: serde_json::Error) -> Self {
        SongbirdError::Validation(Box::new(ValidationError {
            message: format!("JSON serialization/deserialization error: {err}"),
            field: None,
            expected: None,
        }))
    }
}

// Additional From implementations for specific error types
impl From<IoError> for SongbirdError {
    fn from(err: IoError) -> Self {
        SongbirdError::Io(Box::new(err))
    }
}

impl From<NetworkError> for SongbirdError {
    fn from(err: NetworkError) -> Self {
        SongbirdError::Network(Box::new(err))
    }
}

impl From<DiscoveryError> for SongbirdError {
    fn from(err: DiscoveryError) -> Self {
        SongbirdError::Discovery(Box::new(err))
    }
}

impl From<ServiceError> for SongbirdError {
    fn from(err: ServiceError) -> Self {
        SongbirdError::Service(Box::new(err))
    }
}

impl From<ProtocolError> for SongbirdError {
    fn from(err: ProtocolError) -> Self {
        SongbirdError::Protocol(Box::new(err))
    }
}

impl From<AuthError> for SongbirdError {
    fn from(err: AuthError) -> Self {
        SongbirdError::Auth(Box::new(err))
    }
}

impl From<GamingError> for SongbirdError {
    fn from(err: GamingError) -> Self {
        SongbirdError::Gaming(Box::new(err))
    }
}

impl From<ValidationError> for SongbirdError {
    fn from(err: ValidationError) -> Self {
        SongbirdError::Validation(Box::new(err))
    }
}

impl From<NotFoundError> for SongbirdError {
    fn from(err: NotFoundError) -> Self {
        SongbirdError::NotFound(Box::new(err))
    }
}

impl From<DeploymentError> for SongbirdError {
    fn from(err: DeploymentError) -> Self {
        SongbirdError::Deployment(Box::new(err))
    }
}

impl From<PluginNotFoundError> for SongbirdError {
    fn from(err: PluginNotFoundError) -> Self {
        SongbirdError::PluginNotFound(Box::new(err))
    }
}

impl From<RateLimitError> for SongbirdError {
    fn from(err: RateLimitError) -> Self {
        SongbirdError::RateLimitExceeded(Box::new(err))
    }
}

impl From<ExecutionError> for SongbirdError {
    fn from(err: ExecutionError) -> Self {
        SongbirdError::ExecutionFailed(Box::new(err))
    }
}

impl From<ResourceExhaustedError> for SongbirdError {
    fn from(err: ResourceExhaustedError) -> Self {
        SongbirdError::ResourceExhausted(Box::new(err))
    }
}

impl From<CircuitBreakerError> for SongbirdError {
    fn from(err: CircuitBreakerError) -> Self {
        SongbirdError::CircuitBreakerOpen(Box::new(err))
    }
}

impl From<RetryError> for SongbirdError {
    fn from(err: RetryError) -> Self {
        SongbirdError::RetryExhausted(Box::new(err))
    }
}

// Convenience constructors for specific error types
impl IoError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            operation: None,
            path: None,
        }
    }

    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

impl NetworkError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            endpoint: None,
            port: None,
            protocol: None,
        }
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn with_protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocol = Some(protocol.into());
        self
    }
}

impl DiscoveryError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            service: None,
            timeout: None,
            suggestion: None,
        }
    }

    pub fn with_service(mut self, service: impl Into<String>) -> Self {
        self.service = Some(service.into());
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl ServiceError {
    pub fn new(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            message: message.into(),
            status: None,
            suggestion: None,
        }
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl ProtocolError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            protocol: None,
        }
    }

    pub fn with_protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocol = Some(protocol.into());
        self
    }
}

impl AuthError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            provider: None,
        }
    }

    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }
}

impl GamingError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            game: None,
        }
    }

    pub fn with_game(mut self, game: impl Into<String>) -> Self {
        self.game = Some(game.into());
        self
    }
}

impl ValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: None,
            expected: None,
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }
}

impl NotFoundError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            resource: None,
        }
    }

    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }
}

impl DeploymentError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            environment: None,
            component: None,
        }
    }

    pub fn with_environment(mut self, environment: impl Into<String>) -> Self {
        self.environment = Some(environment.into());
        self
    }

    pub fn with_component(mut self, component: impl Into<String>) -> Self {
        self.component = Some(component.into());
        self
    }
}

impl PluginNotFoundError {
    pub fn new(plugin: impl Into<String>) -> Self {
        Self {
            plugin: plugin.into(),
        }
    }
}

impl RateLimitError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            limit: None,
        }
    }

    pub fn with_limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl ExecutionError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            command: None,
        }
    }

    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }
}

impl ResourceExhaustedError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            resource: None,
            current: None,
        }
    }

    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    pub fn with_current(mut self, current: u64) -> Self {
        self.current = Some(current);
        self
    }
}

impl CircuitBreakerError {
    pub fn new(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            message: message.into(),
        }
    }
}

impl RetryError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            attempts: None,
        }
    }

    pub fn with_attempts(mut self, attempts: u32) -> Self {
        self.attempts = Some(attempts);
        self
    }
}
