/// Error System Alignment Module
///
/// This module provides seamless conversion and alignment between the various
/// error systems used across the Songbird ecosystem, ensuring consistent
/// error handling and proper error propagation.

use crate::errors::*;

/// Universal error conversion trait for seamless error handling
pub trait UniversalErrorConversion {
    /// Convert to the main SongbirdError type
    fn to_songbird_error(self) -> SongbirdError;

    /// Convert with additional context
    fn to_songbird_error_with_context(self, context: &str) -> SongbirdError;
}

// Implement conversions from Universal errors to SongbirdError
impl From<ServiceError> for SongbirdError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::ServiceNotFound(service) => {
                service_error!(&service, "Service not found")
            }
            ServiceError::ServiceUnavailable(msg) => service_error!("unknown", msg),
            ServiceError::InitializationFailed(msg) => {
                service_error!("unknown", format!("Initialization failed: {msg}"))
            }
            ServiceError::ShutdownFailed(msg) => {
                service_error!("unknown", format!("Shutdown failed: {msg}"))
            }
            ServiceError::RequestTimeout(msg) => {
                SongbirdError::network(format!("Request timeout: {msg}"))
            }
            ServiceError::InvalidRequest(msg) => {
                SongbirdError::validation_error(format!("Invalid request: {msg}"))
            }
            ServiceError::ProcessingError(msg) => {
                service_error!("unknown", format!("Processing error: {msg}"))
            }
            ServiceError::ConfigurationError(msg) => {
                SongbirdError::configuration_error(format!("Configuration error: {msg}"))
            }
            ServiceError::CapabilityError(err) => err.into(),
            ServiceError::SecurityError(err) => err.into(),
            ServiceError::RegistryError(err) => err.into(),
            ServiceError::ProtocolError(err) => err.into(),
            ServiceError::InternalError(msg) => {
                service_error!("unknown", format!("Internal error: {msg}"))
            }
        }
    }
}

impl From<RegistryError> for SongbirdError {
    fn from(err: RegistryError) -> Self {
        match err {
            RegistryError::ServiceNotFound(service) => {
                service_error!(&service, "Service not found in registry")
            }
            RegistryError::ServiceAlreadyExists(service) => {
                service_error!(&service, "Service already exists in registry")
            }
            RegistryError::InvalidRegistration(msg) => {
                SongbirdError::validation_error(format!("Invalid registration: {msg}"))
            }
            RegistryError::StorageError(msg) => {
                SongbirdError::io_error(format!("Registry storage error: {msg}"))
            }
            RegistryError::CapabilityError(err) => err.into(),
            RegistryError::HealthCheckError(msg) => {
                service_error!("unknown", format!("Health check failed: {msg}"))
            }
            RegistryError::IndexError(msg) => {
                service_error!("unknown", format!("Index error: {msg}"))
            }
            RegistryError::ValidationError(msg) => SongbirdError::validation_error(msg),
            RegistryError::SerializationError(err) => {
                SongbirdError::validation_error(format!("Serialization error: {err}"))
            }
            RegistryError::InternalError(msg) => {
                service_error!("unknown", format!("Registry internal error: {msg}"))
            }
        }
    }
}

impl From<CapabilityError> for SongbirdError {
    fn from(err: CapabilityError) -> Self {
        match err {
            CapabilityError::InvalidCapability(msg) => {
                SongbirdError::validation_error(format!("Invalid capability: {msg}"))
            }
            CapabilityError::CapabilityNotSupported(capability) => {
                SongbirdError::not_found_error(format!("Capability not supported: {capability}"))
            }
            CapabilityError::RequirementNotMet(msg) => {
                SongbirdError::validation_error(format!("Capability requirement not met: {msg}"))
            }
            CapabilityError::ValidationFailed(msg) => {
                SongbirdError::validation_error(format!("Capability validation failed: {msg}"))
            }
            CapabilityError::CapabilityConflict(msg) => {
                SongbirdError::validation_error(format!("Capability conflict: {msg}"))
            }
            CapabilityError::UnknownCapability(capability) => {
                SongbirdError::not_found_error(format!("Unknown capability: {capability}"))
            }
        }
    }
}

impl From<LoadBalancingError> for SongbirdError {
    fn from(err: LoadBalancingError) -> Self {
        match err {
            LoadBalancingError::NoHealthyServices => {
                service_error!("load_balancer", "No healthy services available")
            }
            LoadBalancingError::NoCapableServices => service_error!(
                "load_balancer",
                "No services with required capabilities",
            ),
            LoadBalancingError::NoAvailableServices => {
                service_error!("load_balancer", "No services available")
            }
            LoadBalancingError::NoSuitableInstance => {
                service_error!("load_balancer", "No suitable service instance")
            }
            LoadBalancingError::UnknownStrategy(strategy) => SongbirdError::configuration("strategy",
                format!("Unknown load balancing strategy: {strategy}"),
            ),
            LoadBalancingError::StrategyError(msg) => {
                service_error!("load_balancer", format!("Strategy error: {msg}"))
            }
            LoadBalancingError::HealthCheckFailed(msg) => {
                service_error!("load_balancer", format!("Health check failed: {msg}"))
            }
            LoadBalancingError::CircuitBreakerOpen(service) => {
                SongbirdError::circuit_breaker_error(&service, "Circuit breaker is open")
            }
            LoadBalancingError::ConfigurationError(msg) => {
                SongbirdError::configuration_error(format!("Load balancer configuration error: {msg}"))
            }
            LoadBalancingError::MetricsError(err) => err.into(),
        }
    }
}

impl From<DiscoveryError> for SongbirdError {
    fn from(err: DiscoveryError) -> Self {
        match err {
            DiscoveryError::BackendError(msg) => {
                SongbirdError::discovery_error(format!("Discovery backend error: {msg}"))
            }
            DiscoveryError::ServiceNotFound(service) => {
                service_error!(&service, "Service not found during discovery")
            }
            DiscoveryError::DiscoveryTimeout(msg) => {
                SongbirdError::discovery_error(format!("Discovery timeout: {msg}"))
            }
            DiscoveryError::FilterError(msg) => {
                SongbirdError::discovery_error(format!("Discovery filter error: {msg}"))
            }
            DiscoveryError::WatchError(msg) => {
                SongbirdError::discovery_error(format!("Discovery watch error: {msg}"))
            }
            DiscoveryError::ConnectionError(msg) => {
                SongbirdError::network(format!("Discovery connection error: {msg}"))
            }
            DiscoveryError::SerializationError(err) => {
                SongbirdError::validation_error(format!("Discovery serialization error: {err}"))
            }
            DiscoveryError::ConfigurationError(msg) => {
                SongbirdError::configuration_error(format!("Discovery configuration error: {msg}"))
            }
        }
    }
}

impl From<ProtocolError> for SongbirdError {
    fn from(err: ProtocolError) -> Self {
        match err {
            ProtocolError::ProtocolNotSupported(protocol) => {
                SongbirdError::protocol_error(format!("Protocol not supported: {protocol}"))
            }
            ProtocolError::ConnectionError(msg) => {
                SongbirdError::network(format!("Protocol connection error: {msg}"))
            }
            ProtocolError::SerializationError(err) => {
                SongbirdError::validation_error(format!("Protocol serialization error: {err}"))
            }
            ProtocolError::RequestError(msg) => {
                SongbirdError::protocol_error(format!("Protocol request error: {msg}"))
            }
            ProtocolError::ResponseError(msg) => {
                SongbirdError::protocol_error(format!("Protocol response error: {msg}"))
            }
            ProtocolError::TimeoutError(msg) => {
                SongbirdError::network(format!("Protocol timeout: {msg}"))
            }
            ProtocolError::SecurityError(err) => err.into(),
            ProtocolError::InvalidEndpoint(endpoint) => {
                SongbirdError::network(format!("Invalid protocol endpoint: {endpoint}"))
            }
            ProtocolError::ConfigurationError(msg) => {
                SongbirdError::configuration_error(format!("Protocol configuration error: {msg}"))
            }
        }
    }
}

impl From<SecurityError> for SongbirdError {
    fn from(err: SecurityError) -> Self {
        match err {
            SecurityError::AuthenticationFailed(msg) => {
                SongbirdError::auth_error(format!("Authentication failed: {msg}"))
            }
            SecurityError::AuthorizationFailed(msg) => {
                SongbirdError::auth_error(format!("Authorization failed: {msg}"))
            }
            SecurityError::InvalidToken(msg) => {
                SongbirdError::auth_error(format!("Invalid token: {msg}"))
            }
            SecurityError::TokenExpired(msg) => {
                SongbirdError::auth_error(format!("Token expired: {msg}"))
            }
            SecurityError::EncryptionError(msg) => SongbirdError::encryption_failed(msg),
            SecurityError::DecryptionError(msg) => SongbirdError::decryption_failed(msg),
            SecurityError::KeyManagementError(msg) => {
                SongbirdError::security(format!("Key management error: {msg}"))
            }
            SecurityError::SecurityLevelInsufficient { required, provided } => {
                SongbirdError::security(format!(
                    "Security level insufficient: required {required}, provided {provided}"
                ))
            }
            SecurityError::ConfigurationError(msg) => {
                SongbirdError::configuration_error(format!("Security configuration error: {msg}"))
            }
        }
    }
}

impl From<MetricsError> for SongbirdError {
    fn from(err: MetricsError) -> Self {
        match err {
            MetricsError::BackendError(msg) => {
                service_error!("metrics", format!("Metrics backend error: {msg}"))
            }
            MetricsError::CollectionError(msg) => {
                service_error!("metrics", format!("Metrics collection error: {msg}"))
            }
            MetricsError::InvalidMetric(msg) => {
                SongbirdError::validation_error(format!("Invalid metric: {msg}"))
            }
            MetricsError::StorageError(msg) => {
                SongbirdError::io_error(format!("Metrics storage error: {msg}"))
            }
            MetricsError::SerializationError(err) => {
                SongbirdError::validation_error(format!("Metrics serialization error: {err}"))
            }
            MetricsError::ConfigurationError(msg) => {
                SongbirdError::configuration_error(format!("Metrics configuration error: {msg}"))
            }
        }
    }
}

impl From<EventError> for SongbirdError {
    fn from(err: EventError) -> Self {
        match err {
            EventError::ProcessingFailed(msg) => {
                service_error!("events", format!("Event processing failed: {msg}"))
            }
            EventError::SerializationFailed(err) => {
                SongbirdError::validation_error(format!("Event serialization failed: {err}"))
            }
            EventError::DeliveryFailed(msg) => {
                SongbirdError::network(format!("Event delivery failed: {msg}"))
            }
            EventError::EventTimeout(msg) => {
                SongbirdError::network(format!("Event timeout: {msg}"))
            }
            EventError::InvalidEvent(msg) => {
                SongbirdError::validation_error(format!("Invalid event: {msg}"))
            }
            EventError::HandlerNotFound(handler) => {
                SongbirdError::not_found_error(format!("Event handler not found: {handler}"))
            }
            EventError::SubscriptionError(msg) => {
                service_error!("events", format!("Event subscription error: {msg}"))
            }
        }
    }
}

impl From<ConfigError> for SongbirdError {
    fn from(err: ConfigError) -> Self {
        match err {
            ConfigError::ConfigNotFound(path) => SongbirdError::configuration("config_path",
                format!("Configuration not found: {path}"),
            ),
            ConfigError::InvalidConfig(msg) => {
                SongbirdError::configuration_error(format!("Invalid configuration: {msg}"))
            }
            ConfigError::ParsingError(msg) => {
                SongbirdError::configuration_error(format!("Configuration parsing error: {msg}"))
            }
            ConfigError::ValidationError(msg) => {
                SongbirdError::validation_error(format!("Configuration validation error: {msg}"))
            }
            ConfigError::UpdateError(msg) => {
                SongbirdError::configuration_error(format!("Configuration update error: {msg}"))
            }
            ConfigError::WatchError(msg) => {
                SongbirdError::configuration_error(format!("Configuration watch error: {msg}"))
            }
            ConfigError::SerializationError(err) => {
                SongbirdError::validation_error(format!("Configuration serialization error: {err}"))
            }
        }
    }
}

impl From<OrchestrationError> for SongbirdError {
    fn from(err: OrchestrationError) -> Self {
        match err {
            OrchestrationError::ServiceDiscoveryFailed(msg) => {
                SongbirdError::discovery_error(format!("Service discovery failed: {msg}"))
            }
            OrchestrationError::LoadBalancingFailed(err) => err.into(),
            OrchestrationError::RequestRoutingFailed(msg) => service_error!(
                "orchestrator",
                format!("Request routing failed: {msg}"),
            ),
            OrchestrationError::CoordinationFailed(msg) => {
                service_error!("orchestrator", format!("Coordination failed: {msg}"))
            }
            OrchestrationError::ServiceRegistrationFailed(err) => err.into(),
            OrchestrationError::ProtocolError(err) => err.into(),
            OrchestrationError::SecurityError(err) => err.into(),
            OrchestrationError::ConfigurationError(msg) => {
                SongbirdError::configuration_error(format!("Orchestration configuration error: {msg}"))
            }
        }
    }
}

impl From<CoordinationError> for SongbirdError {
    fn from(err: CoordinationError) -> Self {
        match err {
            CoordinationError::CoordinationTimeout(msg) => {
                SongbirdError::network(format!("Coordination timeout: {msg}"))
            }
            CoordinationError::ParticipantNotFound(participant) => SongbirdError::not_found_error(
                format!("Coordination participant not found: {participant}"),
            ),
            CoordinationError::StepFailed(msg) => service_error!(
                "coordinator",
                format!("Coordination step failed: {msg}"),
            ),
            CoordinationError::InvalidCoordination(msg) => {
                SongbirdError::validation_error(format!("Invalid coordination: {msg}"))
            }
            CoordinationError::EventError(err) => err.into(),
            CoordinationError::ServiceError(err) => err.into(),
            CoordinationError::SerializationError(err) => {
                SongbirdError::validation_error(format!("Coordination serialization error: {err}"))
            }
        }
    }
}

// Implement the trait for all universal error types
impl UniversalErrorConversion for ServiceError {
    fn to_songbird_error(self) -> SongbirdError {
        self.into()
    }

    fn to_songbird_error_with_context(self, context: &str) -> SongbirdError {
        let base_error: SongbirdError = self.into();
        base_error.with_context(context)
    }
}

impl UniversalErrorConversion for RegistryError {
    fn to_songbird_error(self) -> SongbirdError {
        self.into()
    }

    fn to_songbird_error_with_context(self, context: &str) -> SongbirdError {
        let base_error: SongbirdError = self.into();
        base_error.with_context(context)
    }
}

impl UniversalErrorConversion for SecurityError {
    fn to_songbird_error(self) -> SongbirdError {
        self.into()
    }

    fn to_songbird_error_with_context(self, context: &str) -> SongbirdError {
        let base_error: SongbirdError = self.into();
        base_error.with_context(context)
    }
}

/// Helper macro for converting errors with context
#[macro_export]
macro_rules! convert_with_context {
    ($error:expr, $context:expr) => {
        $error.to_songbird_error_with_context($context)
    };
}

/// Helper function to create a safe error result from any universal error
pub fn safe_convert<E: UniversalErrorConversion>(error: E) -> SongbirdError {
    error.to_songbird_error()
}

/// Helper function to create a safe error result with context
pub fn safe_convert_with_context<E: UniversalErrorConversion>(
    error: E,
    context: &str,
) -> SongbirdError {
    error.to_songbird_error_with_context(context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_error_conversion() {
        let service_error = ServiceError::ServiceNotFound("test-service".to_string());
        let songbird_error: SongbirdError = service_error.into();

        assert!(songbird_error.is_service_error());
        assert_eq!(songbird_error.category(), "service");
    }

    #[test]
    fn test_security_error_conversion() {
        let security_error = SecurityError::AuthenticationFailed("Invalid credentials".to_string());
        let songbird_error: SongbirdError = security_error.into();

        assert!(songbird_error.is_auth_error());
        assert_eq!(songbird_error.category(), "authentication");
    }

    #[test]
    fn test_configuration_error_conversion() {
        let config_error = ConfigError::ConfigNotFound("/path/to/config".to_string());
        let songbird_error: SongbirdError = config_error.into();

        assert!(songbird_error.is_config_error());
        assert_eq!(songbird_error.category(), "configuration");
    }

    #[test]
    fn test_universal_error_conversion_trait() {
        let service_error = ServiceError::RequestTimeout("Request timed out".to_string());
        let songbird_error = service_error.to_songbird_error_with_context("HTTP request handling");

        assert!(songbird_error.is_network_error());
    }
}
