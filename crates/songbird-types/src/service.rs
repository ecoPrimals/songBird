//! Service Types and Metadata
//!
//! **CANONICAL**: Service definitions and metadata for the Songbird ecosystem

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// **CANONICAL**: Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalServiceInfo {
    /// Service name
    pub name: String,
    /// Service version
    pub version: String,
    /// Service description
    pub description: Option<String>,
    /// Service endpoints
    pub endpoints: HashMap<String, String>,
    /// Service metadata
    pub metadata: HashMap<String, String>,
    /// Health check endpoint
    pub health_check_endpoint: Option<String>,
    /// Service dependencies
    pub dependencies: Vec<String>,
    /// Service capabilities
    pub capabilities: Vec<String>,
    /// Service metrics
    pub metrics: Option<ServiceMetrics>,
}

impl Default for CanonicalServiceInfo {
    fn default() -> Self {
        Self {
            name: "unknown-service".to_string(),
            version: "0.1.0".to_string(),
            description: None,
            endpoints: HashMap::new()),
            metadata: HashMap::new()),
            health_check_endpoint: Some("/health".to_string()),
            dependencies: Vec::new(),
            capabilities: Vec::new(),
            metrics: None,
        }
    }
}

impl CanonicalServiceInfo {
    /// Create a new service info
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            ..Default::default()
        }
    }

    /// Add an endpoint
    pub fn with_endpoint(&mut self, name: impl Into<String>, url: impl Into<String>) -> &mut Self {
        self.endpoints.insert(name.into(), url.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Add a capability
    pub fn with_capability(&mut self, capability: impl Into<String>) -> &mut Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Add a dependency
    pub fn with_dependency(&mut self, dependency: impl Into<String>) -> &mut Self {
        self.dependencies.push(dependency.into());
        self
    }

    /// Set description
    pub fn with_description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());
        self
    }
}

/// Service metrics information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// Request count
    pub request_count: u64,
    /// Error count
    pub error_count: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Uptime in seconds
    pub uptime_seconds: u64,
}

impl Default for ServiceMetrics {
    fn default() -> Self {
        Self {
            request_count: 0,
            error_count: 0,
            avg_response_time_ms: 0.0,
            uptime_seconds: 0,
        }
    }
}

/// **CANONICAL**: Service type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CanonicalServiceType {
    /// Web service (HTTP/REST API)
    Web,
    /// gRPC service
    Grpc,
    /// Database service
    Database,
    /// Message queue service
    MessageQueue,
    /// Cache service
    Cache,
    /// Authentication service
    Auth,
    /// Storage service
    Storage,
    /// Compute service
    Compute,
    /// AI/ML service
    AI,
    /// Monitoring service
    Monitoring,
    /// Custom service type
    Custom(String),
}

impl Default for CanonicalServiceType {
    fn default() -> Self {
        Self::Custom("unknown".to_string())
    }
}

impl CanonicalServiceType {
    /// Get the service type as a string
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Web => "web",
            Self::Grpc => "grpc",
            Self::Database => "database",
            Self::MessageQueue => "message_queue",
            Self::Cache => "cache",
            Self::Auth => "auth",
            Self::Storage => "storage",
            Self::Compute => "compute",
            Self::AI => "ai",
            Self::Monitoring => "monitoring",
            Self::Custom(custom) => custom,
        }
    }
}

/// **CANONICAL**: Service status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalServiceStatus {
    /// Service is running and healthy
    Running,
    /// Service is starting up
    Starting,
    /// Service is stopping
    Stopping,
    /// Service is stopped
    Stopped,
    /// Service is in error state
    Error,
    /// Service status is unknown
    Unknown,
}

impl Default for CanonicalServiceStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

/// **CANONICAL**: Service configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CanonicalServiceConfig {
    /// Service information
    pub info: CanonicalServiceInfo,
    /// Service type
    pub service_type: CanonicalServiceType,
    /// Current status
    pub status: CanonicalServiceStatus,
    /// Configuration parameters
    pub config: HashMap<String, CanonicalServiceConfigParameter>,
    /// Environment variables
    pub environment: HashMap<String, String>,
}

/// Configuration parameter for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalServiceConfigParameter {
    /// Parameter name
    pub name: String,
    /// Parameter value
    pub value: String,
    /// Parameter description
    pub description: Option<String>,
    /// Whether the parameter is required
    pub required: bool,
    /// Default value
    pub default_value: Option<String>,
    /// Allowed values
    pub allowed_values: AllowedValues,
}

/// Allowed values for configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllowedValues {
    /// Any value is allowed
    Any,
    /// Only specific values are allowed
    Specific(Vec<String>),
    /// Range of numeric values
    Range { min: f64, max: f64 },
    /// Pattern-based validation
    Pattern(String),
}

impl Default for AllowedValues {
    fn default() -> Self {
        Self::Any
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_service_status_default() {
        assert_eq!(
            CanonicalServiceStatus::default(),
            CanonicalServiceStatus::Unknown
        );
    }

    #[test]
    fn test_allowed_values_default() {
        match AllowedValues::default() {
            AllowedValues::Any => {} // Always valid
            _ => panic!("Invalid value type"),
        }
    }

    #[test]
    fn test_allowed_values_range() {
        let range = AllowedValues::Range {
            min: 0.0,
            max: 100.0,
        };
        match range {
            AllowedValues::Range { min, max } => {
                assert!((min - 0.0).abs() < f64::EPSILON);
                assert!((max - 100.0).abs() < f64::EPSILON);
            }
            _ => panic!("Invalid percentage range"),
        }
    }
}
