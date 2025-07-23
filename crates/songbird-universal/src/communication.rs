//! Universal communication patterns for ecosystem integration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_config::AuthMethod;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{CapabilityRequirement, PrimalType, SecurityConfig, SecurityLevel, ServiceCapability};
// Remove problematic import - will fix hardcoded values in a different way
// use songbird_config::config::hardcoded_elimination::replace;

/// Universal request format for all ecosystem communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalRequest {
    /// Unique request identifier
    pub request_id: Uuid,

    /// Source service identifier
    pub source_service: String,

    /// Target service identifier
    pub target_service: String,

    /// Operation (agnostic string)
    pub operation: String,

    /// Payload (completely agnostic)
    pub payload: serde_json::Value,

    /// Security context
    pub security_context: SecurityContext,

    /// Request metadata (extensible)
    pub metadata: HashMap<String, serde_json::Value>,

    /// Request timestamp
    pub timestamp: DateTime<Utc>,

    /// Protocol version (for future compatibility)
    pub protocol_version: String,

    /// Required capabilities for this request
    pub required_capabilities: Vec<CapabilityRequirement>,

    /// Preferred primal type for routing
    pub preferred_primal_type: Option<PrimalType>,

    /// Load balancing strategy preference
    pub load_balancing_strategy: Option<String>,

    /// Request timeout
    pub timeout_ms: Option<u64>,

    /// Retry configuration
    pub retry_config: Option<crate::RetryConfig>,
}

impl UniversalRequest {
    pub fn new(
        source_service: String,
        target_service: String,
        operation: String,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            source_service,
            target_service,
            operation,
            payload,
            security_context: SecurityContext::default(),
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            protocol_version: "1.0".to_string(),
            required_capabilities: Vec::new(),
            preferred_primal_type: None,
            load_balancing_strategy: None,
            timeout_ms: None,
            retry_config: None,
        }
    }

    pub fn with_capabilities(mut self, capabilities: Vec<CapabilityRequirement>) -> Self {
        self.required_capabilities = capabilities;
        self
    }

    pub fn with_primal_preference(mut self, primal_type: PrimalType) -> Self {
        self.preferred_primal_type = Some(primal_type);
        self
    }

    pub fn with_load_balancing_strategy(mut self, strategy: String) -> Self {
        self.load_balancing_strategy = Some(strategy);
        self
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Universal response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalResponse {
    /// Request ID this response is for
    pub request_id: Uuid,

    /// Response status (standardized)
    pub status: ResponseStatus,

    /// Response payload (completely agnostic)
    pub payload: serde_json::Value,

    /// Response metadata (extensible)
    pub metadata: HashMap<String, serde_json::Value>,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,

    /// Protocol version
    pub protocol_version: String,

    /// Processing time in milliseconds
    pub processing_time_ms: Option<u64>,

    /// Service that handled the request
    pub handled_by: Option<String>,
}

impl UniversalResponse {
    pub fn success(request_id: Uuid, payload: serde_json::Value) -> Self {
        Self {
            request_id,
            status: ResponseStatus::Success,
            payload,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            protocol_version: "1.0".to_string(),
            processing_time_ms: None,
            handled_by: None,
        }
    }

    pub fn error(request_id: Uuid, code: String, message: String, retryable: bool) -> Self {
        Self {
            request_id,
            status: ResponseStatus::Error {
                code,
                message,
                retryable,
            },
            payload: serde_json::Value::Null,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            protocol_version: "1.0".to_string(),
            processing_time_ms: None,
            handled_by: None,
        }
    }

    pub fn with_processing_time(mut self, processing_time_ms: u64) -> Self {
        self.processing_time_ms = Some(processing_time_ms);
        self
    }

    pub fn with_handled_by(mut self, service: String) -> Self {
        self.handled_by = Some(service);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Error {
        code: String,
        message: String,
        retryable: bool,
    },
    Timeout,
    ServiceUnavailable,
    RateLimited,
}

/// Security context for all requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Authentication token
    pub auth_token: Option<String>,

    /// User/service identity
    pub identity: String,

    /// Permissions/capabilities
    pub permissions: Vec<String>,

    /// Security level required
    pub security_level: SecurityLevel,

    /// Custom security metadata
    pub custom_security: HashMap<String, serde_json::Value>,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            auth_token: None,
            identity: "anonymous".to_string(),
            permissions: Vec::new(),
            security_level: SecurityLevel::None,
            custom_security: HashMap::new(),
        }
    }
}

// SecurityLevel is defined in types.rs

/// Universal event for cross-primal coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalEvent {
    pub event_id: Uuid,
    pub event_type: String,
    pub source_service: String,
    pub target_services: Vec<String>,
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
    pub ttl_seconds: Option<u64>,
}

impl UniversalEvent {
    pub fn new(event_type: String, source_service: String, payload: serde_json::Value) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type,
            source_service,
            target_services: Vec::new(),
            payload,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
            correlation_id: None,
            ttl_seconds: None,
        }
    }

    pub fn with_targets(mut self, targets: Vec<String>) -> Self {
        self.target_services = targets;
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = Some(ttl_seconds);
        self
    }
}

/// Universal protocol characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCharacteristics {
    pub latency: ProtocolLatency,
    pub throughput: ProtocolThroughput,
    pub streaming: bool,
    pub bidirectional: bool,
    pub security: ProtocolSecurity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolLatency {
    Low,    // < 1ms
    Medium, // 1-10ms
    High,   // > 10ms
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolThroughput {
    Low,    // < 1MB/s
    Medium, // 1-100MB/s
    High,   // > 100MB/s
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolSecurity {
    None,
    Basic,
    Strong,
}

/// Universal service registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceRegistration {
    /// Service identification
    pub service: crate::ServiceIdentification,

    /// Primal type
    pub primal_type: PrimalType,

    /// Associated biome identifier (if applicable)
    pub biome_id: Option<String>,

    /// Service capabilities
    pub capabilities: Vec<ServiceCapability>,

    /// API endpoints
    pub endpoints: Vec<crate::ServiceEndpoint>,

    /// Resource requirements
    pub resource_requirements: crate::ResourceSpec,

    /// Security configuration
    pub security_config: SecurityConfig,

    /// Health check configuration
    pub health_check: crate::HealthCheckConfig,

    /// Extensible metadata for future primals
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Universal service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceConfig {
    /// Service identification
    pub service: crate::ServiceIdentification,

    /// Songbird integration settings
    pub songbird: SongbirdIntegrationConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Resource requirements
    pub resources: crate::ResourceSpec,

    /// Feature flags
    pub features: crate::FeatureFlags,

    /// Primal-specific configuration (completely agnostic)
    pub primal_config: HashMap<String, serde_json::Value>,

    /// Environment overrides
    pub environment: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdIntegrationConfig {
    /// Service mesh endpoints
    pub discovery_endpoint: String,
    pub registration_endpoint: String,
    pub health_endpoint: String,
    pub metrics_endpoint: String,

    /// Authentication
    pub auth_token: Option<String>,
    pub auth_method: AuthMethod,

    /// Retry configuration
    pub retry_config: crate::RetryConfig,

    /// Circuit breaker configuration
    pub circuit_breaker: crate::CircuitBreakerConfig,

    /// Load balancing preferences
    pub load_balancing: crate::LoadBalancingConfig,
}

impl Default for SongbirdIntegrationConfig {
    fn default() -> Self {
        Self {
            discovery_endpoint: "http://localhost:8080/discovery".to_string(),
            registration_endpoint: "http://localhost:8080/register".to_string(),
            health_endpoint: "http://localhost:8080/health".to_string(),
            metrics_endpoint: "http://localhost:8080/metrics".to_string(),
            auth_token: None,
            auth_method: AuthMethod::JWT,
            retry_config: crate::RetryConfig::default(),
            circuit_breaker: crate::CircuitBreakerConfig::default(),
            load_balancing: crate::LoadBalancingConfig::default(),
        }
    }
}
