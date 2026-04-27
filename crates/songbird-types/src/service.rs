// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

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
            endpoints: HashMap::new(),
            metadata: HashMap::new(),
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
    Tarpc,
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
            Self::Tarpc => "tarpc",
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
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
    #[default]
    Unknown,
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum AllowedValues {
    /// Any value is allowed
    #[default]
    Any,
    /// Only specific values are allowed
    Specific(Vec<String>),
    /// Range of numeric values
    Range {
        /// Inclusive lower bound for accepted numeric parameters.
        min: f64,
        /// Inclusive upper bound for accepted numeric parameters.
        max: f64,
    },
    /// Pattern-based validation
    Pattern(String),
}

#[cfg(test)]
#[allow(
    clippy::uninlined_format_args,
    clippy::float_cmp,
    clippy::useless_vec,
    clippy::unreadable_literal,
    clippy::items_after_statements,
    clippy::cast_precision_loss,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "test module"
)]
mod tests {

    use super::*;
    use crate::SongbirdError;

    #[test]
    fn test_canonical_service_status_default() {
        assert_eq!(CanonicalServiceStatus::default(), CanonicalServiceStatus::Unknown);
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
            AllowedValues::Range {
                min,
                max,
            } => {
                assert!((min - 0.0).abs() < f64::EPSILON);
                assert!((max - 100.0).abs() < f64::EPSILON);
            }
            _ => panic!("Invalid percentage range"),
        }
    }

    #[test]
    fn test_canonical_service_info_default() {
        let info = CanonicalServiceInfo::default();
        assert_eq!(info.name, "unknown-service");
        assert_eq!(info.version, "0.1.0");
        assert_eq!(info.description, None);
        assert!(info.endpoints.is_empty());
        assert!(info.metadata.is_empty());
        assert_eq!(info.health_check_endpoint, Some("/health".to_string()));
        assert!(info.dependencies.is_empty());
        assert!(info.capabilities.is_empty());
        assert!(info.metrics.is_none());
    }

    #[test]
    fn test_canonical_service_info_new() {
        let info = CanonicalServiceInfo::new("my-service", "1.0.0");
        assert_eq!(info.name, "my-service");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.description, None);
        assert_eq!(info.health_check_endpoint, Some("/health".to_string()));
    }

    #[test]
    fn test_canonical_service_info_with_endpoint() {
        let mut info = CanonicalServiceInfo::new("my-service", "1.0.0");
        info.with_endpoint("api", "http://localhost:8080");

        assert_eq!(info.endpoints.len(), 1);
        assert_eq!(info.endpoints.get("api"), Some(&"http://localhost:8080".to_string()));
    }

    #[test]
    fn test_canonical_service_info_with_multiple_endpoints() {
        let mut info = CanonicalServiceInfo::new("my-service", "1.0.0");
        info.with_endpoint("api", "http://localhost:8080")
            .with_endpoint("admin", "http://localhost:8081");

        assert_eq!(info.endpoints.len(), 2);
        assert_eq!(info.endpoints.get("api"), Some(&"http://localhost:8080".to_string()));
        assert_eq!(info.endpoints.get("admin"), Some(&"http://localhost:8081".to_string()));
    }

    #[test]
    fn test_canonical_service_info_with_metadata() {
        let mut info = CanonicalServiceInfo::new("my-service", "1.0.0");
        info.with_metadata("env", "production");

        assert_eq!(info.metadata.len(), 1);
        assert_eq!(info.metadata.get("env"), Some(&"production".to_string()));
    }

    #[test]
    fn test_canonical_service_info_with_multiple_metadata() {
        let mut info = CanonicalServiceInfo::new("my-service", "1.0.0");
        info.with_metadata("env", "production").with_metadata("region", "us-east-1");

        assert_eq!(info.metadata.len(), 2);
        assert_eq!(info.metadata.get("env"), Some(&"production".to_string()));
        assert_eq!(info.metadata.get("region"), Some(&"us-east-1".to_string()));
    }

    #[test]
    fn test_canonical_service_info_with_capability() {
        let mut info = CanonicalServiceInfo::new("my-service", "1.0.0");
        info.with_capability("compute");

        assert_eq!(info.capabilities.len(), 1);
        assert_eq!(info.capabilities[0], "compute");
    }

    #[test]
    fn test_canonical_service_info_with_multiple_capabilities() {
        let mut info = CanonicalServiceInfo::new("my-service", "1.0.0");
        info.with_capability("compute").with_capability("storage");

        assert_eq!(info.capabilities.len(), 2);
        assert_eq!(info.capabilities[0], "compute");
        assert_eq!(info.capabilities[1], "storage");
    }

    #[test]
    fn test_canonical_service_info_with_dependency() {
        let mut info = CanonicalServiceInfo::new("my-service", "1.0.0");
        info.with_dependency("database");

        assert_eq!(info.dependencies.len(), 1);
        assert_eq!(info.dependencies[0], "database");
    }

    #[test]
    fn test_canonical_service_info_with_multiple_dependencies() {
        let mut info = CanonicalServiceInfo::new("my-service", "1.0.0");
        info.with_dependency("database").with_dependency("cache");

        assert_eq!(info.dependencies.len(), 2);
        assert_eq!(info.dependencies[0], "database");
        assert_eq!(info.dependencies[1], "cache");
    }

    #[test]
    fn test_canonical_service_info_with_description() {
        let mut info = CanonicalServiceInfo::new("my-service", "1.0.0");
        info.with_description("A test service");

        assert_eq!(info.description, Some("A test service".to_string()));
    }

    #[test]
    fn test_canonical_service_info_builder_chaining() {
        let mut info = CanonicalServiceInfo::new("my-service", "1.0.0");
        info.with_endpoint("api", "http://localhost:8080")
            .with_metadata("env", "production")
            .with_capability("compute")
            .with_dependency("database")
            .with_description("A comprehensive service");

        assert_eq!(info.endpoints.len(), 1);
        assert_eq!(info.metadata.len(), 1);
        assert_eq!(info.capabilities.len(), 1);
        assert_eq!(info.dependencies.len(), 1);
        assert_eq!(info.description, Some("A comprehensive service".to_string()));
    }

    #[test]
    fn test_service_metrics_default() {
        let metrics = ServiceMetrics::default();
        assert_eq!(metrics.request_count, 0);
        assert_eq!(metrics.error_count, 0);
        assert!((metrics.avg_response_time_ms - 0.0).abs() < f64::EPSILON);
        assert_eq!(metrics.uptime_seconds, 0);
    }

    #[test]
    fn test_service_metrics_creation() {
        let metrics = ServiceMetrics {
            request_count: 1000,
            error_count: 10,
            avg_response_time_ms: 50.5,
            uptime_seconds: 3600,
        };

        assert_eq!(metrics.request_count, 1000);
        assert_eq!(metrics.error_count, 10);
        assert!((metrics.avg_response_time_ms - 50.5).abs() < f64::EPSILON);
        assert_eq!(metrics.uptime_seconds, 3600);
    }

    #[test]
    fn test_canonical_service_type_default() {
        let service_type = CanonicalServiceType::default();
        assert_eq!(service_type, CanonicalServiceType::Custom("unknown".to_string()));
    }

    #[test]
    fn test_canonical_service_type_as_str_web() {
        assert_eq!(CanonicalServiceType::Web.as_str(), "web");
    }

    #[test]
    fn test_canonical_service_type_as_str_grpc() {
        assert_eq!(CanonicalServiceType::Tarpc.as_str(), "tarpc");
    }

    #[test]
    fn test_canonical_service_type_as_str_database() {
        assert_eq!(CanonicalServiceType::Database.as_str(), "database");
    }

    #[test]
    fn test_canonical_service_type_as_str_message_queue() {
        assert_eq!(CanonicalServiceType::MessageQueue.as_str(), "message_queue");
    }

    #[test]
    fn test_canonical_service_type_as_str_cache() {
        assert_eq!(CanonicalServiceType::Cache.as_str(), "cache");
    }

    #[test]
    fn test_canonical_service_type_as_str_auth() {
        assert_eq!(CanonicalServiceType::Auth.as_str(), "auth");
    }

    #[test]
    fn test_canonical_service_type_as_str_storage() {
        assert_eq!(CanonicalServiceType::Storage.as_str(), "storage");
    }

    #[test]
    fn test_canonical_service_type_as_str_compute() {
        assert_eq!(CanonicalServiceType::Compute.as_str(), "compute");
    }

    #[test]
    fn test_canonical_service_type_as_str_ai() {
        assert_eq!(CanonicalServiceType::AI.as_str(), "ai");
    }

    #[test]
    fn test_canonical_service_type_as_str_monitoring() {
        assert_eq!(CanonicalServiceType::Monitoring.as_str(), "monitoring");
    }

    #[test]
    fn test_canonical_service_type_as_str_custom() {
        let custom = CanonicalServiceType::Custom("my-custom-type".to_string());
        assert_eq!(custom.as_str(), "my-custom-type");
    }

    #[test]
    fn test_canonical_service_type_equality() {
        assert_eq!(CanonicalServiceType::Web, CanonicalServiceType::Web);
        assert_eq!(CanonicalServiceType::Tarpc, CanonicalServiceType::Tarpc);
        assert_ne!(CanonicalServiceType::Web, CanonicalServiceType::Tarpc);

        let custom1 = CanonicalServiceType::Custom("type1".to_string());
        let custom2 = CanonicalServiceType::Custom("type1".to_string());
        let custom3 = CanonicalServiceType::Custom("type2".to_string());
        assert_eq!(custom1, custom2);
        assert_ne!(custom1, custom3);
    }

    #[test]
    fn test_canonical_service_status_variants() {
        assert_eq!(CanonicalServiceStatus::Running, CanonicalServiceStatus::Running);
        assert_eq!(CanonicalServiceStatus::Starting, CanonicalServiceStatus::Starting);
        assert_eq!(CanonicalServiceStatus::Stopping, CanonicalServiceStatus::Stopping);
        assert_eq!(CanonicalServiceStatus::Stopped, CanonicalServiceStatus::Stopped);
        assert_eq!(CanonicalServiceStatus::Error, CanonicalServiceStatus::Error);
        assert_eq!(CanonicalServiceStatus::Unknown, CanonicalServiceStatus::Unknown);

        assert_ne!(CanonicalServiceStatus::Running, CanonicalServiceStatus::Stopped);
    }

    #[test]
    fn test_canonical_service_config_default() {
        let config = CanonicalServiceConfig::default();
        assert_eq!(config.info.name, "unknown-service");
        assert_eq!(config.service_type, CanonicalServiceType::Custom("unknown".to_string()));
        assert_eq!(config.status, CanonicalServiceStatus::Unknown);
        assert!(config.config.is_empty());
        assert!(config.environment.is_empty());
    }

    #[test]
    fn test_canonical_service_config_creation() {
        let info = CanonicalServiceInfo::new("test-service", "1.0.0");
        let config = CanonicalServiceConfig {
            info,
            service_type: CanonicalServiceType::Web,
            status: CanonicalServiceStatus::Running,
            config: HashMap::new(),
            environment: HashMap::new(),
        };

        assert_eq!(config.info.name, "test-service");
        assert_eq!(config.service_type, CanonicalServiceType::Web);
        assert_eq!(config.status, CanonicalServiceStatus::Running);
    }

    #[test]
    fn test_canonical_service_config_parameter_creation() {
        let param = CanonicalServiceConfigParameter {
            name: "port".to_string(),
            value: "8080".to_string(),
            description: Some("HTTP port".to_string()),
            required: true,
            default_value: Some("8080".to_string()),
            allowed_values: AllowedValues::Range {
                min: 1024.0,
                max: 65535.0,
            },
        };

        assert_eq!(param.name, "port");
        assert_eq!(param.value, "8080");
        assert_eq!(param.description, Some("HTTP port".to_string()));
        assert!(param.required);
        assert_eq!(param.default_value, Some("8080".to_string()));
    }

    #[test]
    fn test_allowed_values_any() {
        let allowed = AllowedValues::Any;
        match allowed {
            AllowedValues::Any => {} // Expected
            _ => panic!("Expected Any variant"),
        }
    }

    #[test]
    fn test_allowed_values_specific() {
        let allowed = AllowedValues::Specific(vec!["value1".to_string(), "value2".to_string()]);
        match allowed {
            AllowedValues::Specific(values) => {
                assert_eq!(values.len(), 2);
                assert_eq!(values[0], "value1");
                assert_eq!(values[1], "value2");
            }
            _ => panic!("Expected Specific variant"),
        }
    }

    #[test]
    fn test_allowed_values_pattern() {
        let allowed = AllowedValues::Pattern("^[a-z]+$".to_string());
        match allowed {
            AllowedValues::Pattern(pattern) => {
                assert_eq!(pattern, "^[a-z]+$");
            }
            _ => panic!("Expected Pattern variant"),
        }
    }

    #[test]
    fn test_canonical_service_info_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let info = CanonicalServiceInfo::new("test-service", "1.0.0");
        let json = serde_json::to_string(&info)
            .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {}", e)))?;
        assert!(json.contains("test-service"));
        assert!(json.contains("1.0.0"));
        Ok(())
    }

    #[test]
    fn test_service_metrics_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let metrics = ServiceMetrics {
            request_count: 100,
            error_count: 5,
            avg_response_time_ms: 25.5,
            uptime_seconds: 1800,
        };
        let json = serde_json::to_string(&metrics)
            .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {}", e)))?;
        assert!(json.contains("100"));
        assert!(json.contains("25.5"));
        Ok(())
    }

    #[test]
    fn test_canonical_service_type_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let service_type = CanonicalServiceType::Web;
        let json = serde_json::to_string(&service_type)
            .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {}", e)))?;
        assert!(json.contains("Web"));
        Ok(())
    }

    #[test]
    fn test_canonical_service_status_serialization() -> Result<(), Box<dyn std::error::Error>> {
        let status = CanonicalServiceStatus::Running;
        let json = serde_json::to_string(&status)
            .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {}", e)))?;
        assert!(json.contains("Running"));
        Ok(())
    }

    fn assert_json_roundtrip<T>(v: &T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        use serde_json::Value;
        let a: Value = serde_json::to_value(v).unwrap();
        let back: T = serde_json::from_value(a.clone()).unwrap();
        assert_eq!(serde_json::to_value(&back).unwrap(), a);
    }

    #[test]
    fn serde_json_roundtrip_canonical_service_info() {
        assert_json_roundtrip(&CanonicalServiceInfo::new("svc", "2.0.0"));
    }

    #[test]
    fn serde_json_roundtrip_service_metrics() {
        assert_json_roundtrip(&ServiceMetrics::default());
    }

    #[test]
    fn serde_json_roundtrip_canonical_service_type_variants() {
        assert_json_roundtrip(&CanonicalServiceType::MessageQueue);
        assert_json_roundtrip(&CanonicalServiceType::Custom("x".to_string()));
    }

    #[test]
    fn serde_json_roundtrip_canonical_service_status_variants() {
        assert_json_roundtrip(&CanonicalServiceStatus::Stopped);
    }

    #[test]
    fn serde_json_roundtrip_canonical_service_config() {
        assert_json_roundtrip(&CanonicalServiceConfig::default());
    }

    #[test]
    fn serde_json_roundtrip_canonical_service_config_parameter_and_allowed_values() {
        let param = CanonicalServiceConfigParameter {
            name: "p".to_string(),
            value: "v".to_string(),
            description: None,
            required: true,
            default_value: None,
            allowed_values: AllowedValues::Specific(vec!["a".to_string()]),
        };
        assert_json_roundtrip(&param);
        assert_json_roundtrip(&AllowedValues::Any);
        assert_json_roundtrip(&AllowedValues::Range {
            min: 0.0,
            max: 1.0,
        });
        assert_json_roundtrip(&AllowedValues::Pattern(".*".to_string()));
    }

    #[test]
    fn with_endpoint_overwrites_duplicate_key() {
        let mut info = CanonicalServiceInfo::new("svc", "1.0");
        info.with_endpoint("api", "http://a:1");
        info.with_endpoint("api", "http://a:2");
        assert_eq!(info.endpoints.get("api").unwrap(), "http://a:2");
        assert_eq!(info.endpoints.len(), 1);
    }

    #[test]
    fn with_metadata_overwrites_duplicate_key() {
        let mut info = CanonicalServiceInfo::new("svc", "1.0");
        info.with_metadata("env", "dev");
        info.with_metadata("env", "prod");
        assert_eq!(info.metadata.get("env").unwrap(), "prod");
        assert_eq!(info.metadata.len(), 1);
    }

    #[test]
    fn with_capability_and_dependency_allow_duplicates() {
        let mut info = CanonicalServiceInfo::new("svc", "1.0");
        info.with_capability("compute");
        info.with_capability("compute");
        assert_eq!(info.capabilities.len(), 2);

        info.with_dependency("dep-a");
        info.with_dependency("dep-a");
        assert_eq!(info.dependencies.len(), 2);
    }

    #[test]
    fn service_type_as_str_custom_returns_inner() {
        let custom = CanonicalServiceType::Custom("my_service".to_string());
        assert_eq!(custom.as_str(), "my_service");
    }
}
