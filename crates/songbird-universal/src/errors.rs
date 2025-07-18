//! Universal error types for ecosystem integration

use thiserror::Error;

/// Universal service error
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Service initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Service shutdown failed: {0}")]
    ShutdownFailed(String),

    #[error("Request timeout: {0}")]
    RequestTimeout(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Processing error: {0}")]
    ProcessingError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Capability error: {0}")]
    CapabilityError(#[from] CapabilityError),

    #[error("Security error: {0}")]
    SecurityError(#[from] SecurityError),

    #[error("Registry error: {0}")]
    RegistryError(#[from] RegistryError),

    #[error("Protocol error: {0}")]
    ProtocolError(#[from] ProtocolError),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Universal registry error
#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Service already exists: {0}")]
    ServiceAlreadyExists(String),

    #[error("Invalid registration: {0}")]
    InvalidRegistration(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Capability error: {0}")]
    CapabilityError(#[from] CapabilityError),

    #[error("Health check error: {0}")]
    HealthCheckError(String),

    #[error("Index error: {0}")]
    IndexError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Universal capability error
#[derive(Error, Debug)]
pub enum CapabilityError {
    #[error("Invalid capability: {0}")]
    InvalidCapability(String),

    #[error("Capability not supported: {0}")]
    CapabilityNotSupported(String),

    #[error("Capability requirement not met: {0}")]
    RequirementNotMet(String),

    #[error("Capability validation failed: {0}")]
    ValidationFailed(String),

    #[error("Capability conflict: {0}")]
    CapabilityConflict(String),

    #[error("Unknown capability: {0}")]
    UnknownCapability(String),
}

/// Universal load balancing error
#[derive(Error, Debug)]
pub enum LoadBalancingError {
    #[error("No healthy services available")]
    NoHealthyServices,

    #[error("No services with required capabilities")]
    NoCapableServices,

    #[error("No services available")]
    NoAvailableServices,

    #[error("No suitable service instance")]
    NoSuitableInstance,

    #[error("Unknown strategy: {0}")]
    UnknownStrategy(String),

    #[error("Strategy error: {0}")]
    StrategyError(String),

    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Metrics error: {0}")]
    MetricsError(#[from] MetricsError),
}

/// Universal discovery error
#[derive(Error, Debug)]
pub enum DiscoveryError {
    #[error("Backend error: {0}")]
    BackendError(String),

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Discovery timeout: {0}")]
    DiscoveryTimeout(String),

    #[error("Filter error: {0}")]
    FilterError(String),

    #[error("Watch error: {0}")]
    WatchError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Universal protocol error
#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Protocol not supported: {0}")]
    ProtocolNotSupported(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Request error: {0}")]
    RequestError(String),

    #[error("Response error: {0}")]
    ResponseError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Security error: {0}")]
    SecurityError(#[from] SecurityError),

    #[error("Invalid endpoint: {0}")]
    InvalidEndpoint(String),

    #[error("Protocol configuration error: {0}")]
    ConfigurationError(String),
}

/// Universal security error
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Token expired: {0}")]
    TokenExpired(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Key management error: {0}")]
    KeyManagementError(String),

    #[error("Security level insufficient: required {required}, provided {provided}")]
    SecurityLevelInsufficient { required: String, provided: String },

    #[error("Security configuration error: {0}")]
    ConfigurationError(String),
}

/// Universal metrics error
#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("Backend error: {0}")]
    BackendError(String),

    #[error("Collection error: {0}")]
    CollectionError(String),

    #[error("Invalid metric: {0}")]
    InvalidMetric(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Universal event error
#[derive(Error, Debug)]
pub enum EventError {
    #[error("Event processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Event serialization failed: {0}")]
    SerializationFailed(#[from] serde_json::Error),

    #[error("Event delivery failed: {0}")]
    DeliveryFailed(String),

    #[error("Event timeout: {0}")]
    EventTimeout(String),

    #[error("Invalid event: {0}")]
    InvalidEvent(String),

    #[error("Event handler not found: {0}")]
    HandlerNotFound(String),

    #[error("Event subscription error: {0}")]
    SubscriptionError(String),
}

/// Universal configuration error
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration not found: {0}")]
    ConfigNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Configuration parsing error: {0}")]
    ParsingError(String),

    #[error("Configuration validation error: {0}")]
    ValidationError(String),

    #[error("Configuration update error: {0}")]
    UpdateError(String),

    #[error("Configuration watch error: {0}")]
    WatchError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Universal orchestration error
#[derive(Error, Debug)]
pub enum OrchestrationError {
    #[error("Service discovery failed: {0}")]
    ServiceDiscoveryFailed(String),

    #[error("Load balancing failed: {0}")]
    LoadBalancingFailed(#[from] LoadBalancingError),

    #[error("Request routing failed: {0}")]
    RequestRoutingFailed(String),

    #[error("Coordination failed: {0}")]
    CoordinationFailed(String),

    #[error("Service registration failed: {0}")]
    ServiceRegistrationFailed(#[from] RegistryError),

    #[error("Protocol error: {0}")]
    ProtocolError(#[from] ProtocolError),

    #[error("Security error: {0}")]
    SecurityError(#[from] SecurityError),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

/// Universal coordination error
#[derive(Error, Debug)]
pub enum CoordinationError {
    #[error("Coordination timeout: {0}")]
    CoordinationTimeout(String),

    #[error("Participant not found: {0}")]
    ParticipantNotFound(String),

    #[error("Coordination step failed: {0}")]
    StepFailed(String),

    #[error("Invalid coordination: {0}")]
    InvalidCoordination(String),

    #[error("Event error: {0}")]
    EventError(#[from] EventError),

    #[error("Service error: {0}")]
    ServiceError(#[from] ServiceError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Result type aliases for convenience
pub type ServiceResult<T> = Result<T, ServiceError>;
pub type RegistryResult<T> = Result<T, RegistryError>;
pub type CapabilityResult<T> = Result<T, CapabilityError>;
pub type LoadBalancingResult<T> = Result<T, LoadBalancingError>;
pub type DiscoveryResult<T> = Result<T, DiscoveryError>;
pub type ProtocolResult<T> = Result<T, ProtocolError>;
pub type SecurityResult<T> = Result<T, SecurityError>;
pub type MetricsResult<T> = Result<T, MetricsError>;
pub type EventResult<T> = Result<T, EventError>;
pub type ConfigResult<T> = Result<T, ConfigError>;
pub type OrchestrationResult<T> = Result<T, OrchestrationError>;
pub type CoordinationResult<T> = Result<T, CoordinationError>;
