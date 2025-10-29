//! Comprehensive tests for the service module
//!
//! These tests ensure complete coverage of service types and their methods.

use songbird_types::service::{
    AllowedValues, CanonicalServiceConfig, CanonicalServiceConfigParameter, CanonicalServiceInfo,
    CanonicalServiceStatus, CanonicalServiceType, ServiceMetrics,
};

// ==================== CanonicalServiceInfo Tests ====================

#[test]
fn test_service_info_new() {
    let info = CanonicalServiceInfo::new("test-service", "1.0.0");
    assert_eq!(info.name, "test-service");
    assert_eq!(info.version, "1.0.0");
    assert!(info.endpoints.is_empty());
    assert!(info.metadata.is_empty());
    assert!(info.dependencies.is_empty());
    assert!(info.capabilities.is_empty());
}

#[test]
fn test_service_info_default() {
    let info = CanonicalServiceInfo::default();
    assert_eq!(info.name, "unknown-service");
    assert_eq!(info.version, "0.1.0");
    assert_eq!(info.health_check_endpoint, Some("/health".to_string()));
    assert!(info.endpoints.is_empty());
    assert!(info.metadata.is_empty());
}

#[test]
fn test_service_info_with_endpoint() {
    let mut info = CanonicalServiceInfo::new("test", "1.0.0");
    info.with_endpoint("api", "http://localhost:8080/api");

    assert_eq!(info.endpoints.len(), 1);
    assert_eq!(info.endpoints.get("api"), Some(&"http://localhost:8080/api".to_string()));
}

#[test]
fn test_service_info_with_multiple_endpoints() {
    let mut info = CanonicalServiceInfo::new("test", "1.0.0");
    info.with_endpoint("api", "http://localhost:8080/api")
        .with_endpoint("grpc", "http://localhost:9090");

    assert_eq!(info.endpoints.len(), 2);
    assert!(info.endpoints.contains_key("api"));
    assert!(info.endpoints.contains_key("grpc"));
}

#[test]
fn test_service_info_with_metadata() {
    let mut info = CanonicalServiceInfo::new("test", "1.0.0");
    info.with_metadata("environment", "production");

    assert_eq!(info.metadata.len(), 1);
    assert_eq!(info.metadata.get("environment"), Some(&"production".to_string()));
}

#[test]
fn test_service_info_with_multiple_metadata() {
    let mut info = CanonicalServiceInfo::new("test", "1.0.0");
    info.with_metadata("environment", "production")
        .with_metadata("region", "us-east-1")
        .with_metadata("cluster", "primary");

    assert_eq!(info.metadata.len(), 3);
    assert_eq!(info.metadata.get("region"), Some(&"us-east-1".to_string()));
}

#[test]
fn test_service_info_with_capability() {
    let mut info = CanonicalServiceInfo::new("test", "1.0.0");
    info.with_capability("authentication");

    assert_eq!(info.capabilities.len(), 1);
    assert!(info.capabilities.contains(&"authentication".to_string()));
}

#[test]
fn test_service_info_with_multiple_capabilities() {
    let mut info = CanonicalServiceInfo::new("test", "1.0.0");
    info.with_capability("authentication")
        .with_capability("authorization")
        .with_capability("rate-limiting");

    assert_eq!(info.capabilities.len(), 3);
    assert!(info.capabilities.contains(&"rate-limiting".to_string()));
}

#[test]
fn test_service_info_with_dependency() {
    let mut info = CanonicalServiceInfo::new("test", "1.0.0");
    info.with_dependency("database");

    assert_eq!(info.dependencies.len(), 1);
    assert!(info.dependencies.contains(&"database".to_string()));
}

#[test]
fn test_service_info_with_multiple_dependencies() {
    let mut info = CanonicalServiceInfo::new("test", "1.0.0");
    info.with_dependency("database").with_dependency("cache").with_dependency("message-queue");

    assert_eq!(info.dependencies.len(), 3);
    assert!(info.dependencies.contains(&"cache".to_string()));
}

#[test]
fn test_service_info_with_description() {
    let mut info = CanonicalServiceInfo::new("test", "1.0.0");
    info.with_description("A test service for the ecosystem");

    assert_eq!(info.description, Some("A test service for the ecosystem".to_string()));
}

#[test]
fn test_service_info_builder_chain() {
    let mut info = CanonicalServiceInfo::new("comprehensive", "2.0.0");
    info.with_description("Comprehensive service")
        .with_endpoint("rest", "http://localhost:8080")
        .with_endpoint("grpc", "http://localhost:9090")
        .with_metadata("env", "prod")
        .with_capability("auth")
        .with_dependency("postgres");

    assert_eq!(info.name, "comprehensive");
    assert_eq!(info.version, "2.0.0");
    assert_eq!(info.endpoints.len(), 2);
    assert_eq!(info.metadata.len(), 1);
    assert_eq!(info.capabilities.len(), 1);
    assert_eq!(info.dependencies.len(), 1);
}

#[test]
fn test_service_info_clone() {
    let mut info = CanonicalServiceInfo::new("test", "1.0.0");
    info.with_endpoint("api", "http://localhost:8080");

    let cloned = info.clone();
    assert_eq!(cloned.name, info.name);
    assert_eq!(cloned.version, info.version);
    assert_eq!(cloned.endpoints.len(), info.endpoints.len());
}

#[test]
fn test_service_info_with_metrics() {
    let mut info = CanonicalServiceInfo::new("test", "1.0.0");
    info.metrics = Some(ServiceMetrics {
        request_count: 1000,
        error_count: 5,
        avg_response_time_ms: 45.5,
        uptime_seconds: 3600,
    });

    assert!(info.metrics.is_some());
    let metrics = info.metrics.unwrap();
    assert_eq!(metrics.request_count, 1000);
    assert_eq!(metrics.error_count, 5);
}

// ==================== ServiceMetrics Tests ====================

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
        request_count: 5000,
        error_count: 25,
        avg_response_time_ms: 125.75,
        uptime_seconds: 7200,
    };

    assert_eq!(metrics.request_count, 5000);
    assert_eq!(metrics.error_count, 25);
    assert!((metrics.avg_response_time_ms - 125.75).abs() < f64::EPSILON);
    assert_eq!(metrics.uptime_seconds, 7200);
}

#[test]
fn test_service_metrics_clone() {
    let metrics = ServiceMetrics {
        request_count: 100,
        error_count: 1,
        avg_response_time_ms: 50.0,
        uptime_seconds: 600,
    };

    let cloned = metrics.clone();
    assert_eq!(cloned.request_count, metrics.request_count);
    assert_eq!(cloned.error_count, metrics.error_count);
}

// ==================== CanonicalServiceType Tests ====================

#[test]
fn test_service_type_variants() {
    let types = vec![
        CanonicalServiceType::Web,
        CanonicalServiceType::Grpc,
        CanonicalServiceType::Database,
        CanonicalServiceType::MessageQueue,
        CanonicalServiceType::Cache,
        CanonicalServiceType::Auth,
        CanonicalServiceType::Storage,
        CanonicalServiceType::Compute,
        CanonicalServiceType::AI,
        CanonicalServiceType::Monitoring,
    ];

    assert_eq!(types.len(), 10);
}

#[test]
fn test_service_type_as_str() {
    assert_eq!(CanonicalServiceType::Web.as_str(), "web");
    assert_eq!(CanonicalServiceType::Grpc.as_str(), "grpc");
    assert_eq!(CanonicalServiceType::Database.as_str(), "database");
    assert_eq!(CanonicalServiceType::MessageQueue.as_str(), "message_queue");
    assert_eq!(CanonicalServiceType::Cache.as_str(), "cache");
    assert_eq!(CanonicalServiceType::Auth.as_str(), "auth");
    assert_eq!(CanonicalServiceType::Storage.as_str(), "storage");
    assert_eq!(CanonicalServiceType::Compute.as_str(), "compute");
    assert_eq!(CanonicalServiceType::AI.as_str(), "ai");
    assert_eq!(CanonicalServiceType::Monitoring.as_str(), "monitoring");
}

#[test]
fn test_service_type_custom() {
    let custom = CanonicalServiceType::Custom("blockchain".to_string());
    assert_eq!(custom.as_str(), "blockchain");
}

#[test]
fn test_service_type_default() {
    let default = CanonicalServiceType::default();
    match default {
        CanonicalServiceType::Custom(s) => assert_eq!(s, "unknown"),
        _ => panic!("Default should be Custom(unknown)"),
    }
}

#[test]
fn test_service_type_equality() {
    assert_eq!(CanonicalServiceType::Web, CanonicalServiceType::Web);
    assert_ne!(CanonicalServiceType::Web, CanonicalServiceType::Grpc);

    let custom1 = CanonicalServiceType::Custom("test".to_string());
    let custom2 = CanonicalServiceType::Custom("test".to_string());
    assert_eq!(custom1, custom2);
}

#[test]
fn test_service_type_clone() {
    let service_type = CanonicalServiceType::Database;
    let cloned = service_type.clone();
    assert_eq!(service_type, cloned);
}

#[test]
fn test_service_type_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(CanonicalServiceType::Web);
    set.insert(CanonicalServiceType::Web);
    set.insert(CanonicalServiceType::Database);

    assert_eq!(set.len(), 2);
}

// ==================== CanonicalServiceStatus Tests ====================

#[test]
fn test_service_status_variants() {
    let statuses = [
        CanonicalServiceStatus::Running,
        CanonicalServiceStatus::Starting,
        CanonicalServiceStatus::Stopping,
        CanonicalServiceStatus::Stopped,
        CanonicalServiceStatus::Error,
        CanonicalServiceStatus::Unknown,
    ];

    assert_eq!(statuses.len(), 6);
}

#[test]
fn test_service_status_default() {
    assert_eq!(CanonicalServiceStatus::default(), CanonicalServiceStatus::Unknown);
}

#[test]
fn test_service_status_equality() {
    assert_eq!(CanonicalServiceStatus::Running, CanonicalServiceStatus::Running);
    assert_ne!(CanonicalServiceStatus::Running, CanonicalServiceStatus::Stopped);
}

#[test]
fn test_service_status_copy() {
    let status = CanonicalServiceStatus::Running;
    let copied = status;
    assert_eq!(status, copied);
}

#[test]
fn test_service_status_clone() {
    let status = CanonicalServiceStatus::Starting;
    let cloned = status;
    assert_eq!(status, cloned);
}

// ==================== CanonicalServiceConfig Tests ====================

#[test]
fn test_service_config_default() {
    let config = CanonicalServiceConfig::default();
    assert_eq!(config.info.name, "unknown-service");
    assert_eq!(config.status, CanonicalServiceStatus::Unknown);
    assert!(config.config.is_empty());
    assert!(config.environment.is_empty());
}

#[test]
fn test_service_config_with_info() {
    let config = CanonicalServiceConfig {
        info: CanonicalServiceInfo::new("custom-service", "2.0.0"),
        ..Default::default()
    };

    assert_eq!(config.info.name, "custom-service");
    assert_eq!(config.info.version, "2.0.0");
}

#[test]
fn test_service_config_with_status() {
    let config = CanonicalServiceConfig {
        status: CanonicalServiceStatus::Running,
        ..Default::default()
    };

    assert_eq!(config.status, CanonicalServiceStatus::Running);
}

#[test]
fn test_service_config_with_type() {
    let config = CanonicalServiceConfig {
        service_type: CanonicalServiceType::Database,
        ..Default::default()
    };

    assert_eq!(config.service_type.as_str(), "database");
}

#[test]
fn test_service_config_with_parameters() {
    let mut config = CanonicalServiceConfig::default();

    let param = CanonicalServiceConfigParameter {
        name: "max_connections".to_string(),
        value: "100".to_string(),
        description: Some("Maximum number of connections".to_string()),
        required: true,
        default_value: Some("50".to_string()),
        allowed_values: AllowedValues::Range {
            min: 1.0,
            max: 1000.0,
        },
    };

    config.config.insert("max_connections".to_string(), param);

    assert_eq!(config.config.len(), 1);
    assert!(config.config.contains_key("max_connections"));
}

#[test]
fn test_service_config_with_environment() {
    let mut config = CanonicalServiceConfig::default();
    config.environment.insert("DATABASE_URL".to_string(), "postgres://localhost".to_string());
    config.environment.insert("REDIS_URL".to_string(), "redis://localhost".to_string());

    assert_eq!(config.environment.len(), 2);
    assert!(config.environment.contains_key("DATABASE_URL"));
}

#[test]
fn test_service_config_clone() {
    let config = CanonicalServiceConfig {
        status: CanonicalServiceStatus::Running,
        ..Default::default()
    };

    let cloned = config.clone();
    assert_eq!(cloned.status, config.status);
}

// ==================== CanonicalServiceConfigParameter Tests ====================

#[test]
fn test_config_parameter_required() {
    let param = CanonicalServiceConfigParameter {
        name: "api_key".to_string(),
        value: "secret".to_string(),
        description: Some("API key for authentication".to_string()),
        required: true,
        default_value: None,
        allowed_values: AllowedValues::Any,
    };

    assert_eq!(param.name, "api_key");
    assert!(param.required);
    assert!(param.default_value.is_none());
}

#[test]
fn test_config_parameter_optional_with_default() {
    let param = CanonicalServiceConfigParameter {
        name: "timeout".to_string(),
        value: "30".to_string(),
        description: Some("Request timeout in seconds".to_string()),
        required: false,
        default_value: Some("60".to_string()),
        allowed_values: AllowedValues::Range {
            min: 1.0,
            max: 300.0,
        },
    };

    assert!(!param.required);
    assert_eq!(param.default_value, Some("60".to_string()));
}

#[test]
fn test_config_parameter_clone() {
    let param = CanonicalServiceConfigParameter {
        name: "test".to_string(),
        value: "value".to_string(),
        description: None,
        required: false,
        default_value: None,
        allowed_values: AllowedValues::Any,
    };

    let cloned = param.clone();
    assert_eq!(cloned.name, param.name);
}

// ==================== AllowedValues Tests ====================

#[test]
fn test_allowed_values_any() {
    let allowed = AllowedValues::Any;
    match allowed {
        AllowedValues::Any => {} // Always valid
        _ => panic!("Should be Any variant"),
    }
}

#[test]
fn test_allowed_values_specific() {
    let allowed = AllowedValues::Specific(vec![
        "debug".to_string(),
        "info".to_string(),
        "warn".to_string(),
        "error".to_string(),
    ]);

    match allowed {
        AllowedValues::Specific(values) => {
            assert_eq!(values.len(), 4);
            assert!(values.contains(&"info".to_string()));
        }
        _ => panic!("Should be Specific variant"),
    }
}

#[test]
fn test_allowed_values_range() {
    let allowed = AllowedValues::Range {
        min: 0.0,
        max: 100.0,
    };

    match allowed {
        AllowedValues::Range {
            min,
            max,
        } => {
            assert!((min - 0.0).abs() < f64::EPSILON);
            assert!((max - 100.0).abs() < f64::EPSILON);
        }
        _ => panic!("Should be Range variant"),
    }
}

#[test]
fn test_allowed_values_pattern() {
    let allowed = AllowedValues::Pattern(r"^\d{3}-\d{3}-\d{4}$".to_string());

    match allowed {
        AllowedValues::Pattern(pattern) => {
            assert_eq!(pattern, r"^\d{3}-\d{3}-\d{4}$");
        }
        _ => panic!("Should be Pattern variant"),
    }
}

#[test]
fn test_allowed_values_default() {
    let allowed = AllowedValues::default();
    match allowed {
        AllowedValues::Any => {} // Always valid
        _ => panic!("Default should be Any"),
    }
}

#[test]
fn test_allowed_values_clone() {
    let allowed = AllowedValues::Range {
        min: 1.0,
        max: 10.0,
    };
    let cloned = allowed.clone();

    match (allowed, cloned) {
        (
            AllowedValues::Range {
                min: min1,
                max: max1,
            },
            AllowedValues::Range {
                min: min2,
                max: max2,
            },
        ) => {
            assert!((min1 - min2).abs() < f64::EPSILON);
            assert!((max1 - max2).abs() < f64::EPSILON);
        }
        _ => panic!("Both should be Range variants"),
    }
}

// ==================== Serialization Tests ====================

#[test]
fn test_service_info_serialization() {
    let info = CanonicalServiceInfo::new("test", "1.0.0");
    let json = serde_json::to_string(&info).expect("Failed to serialize");
    assert!(json.contains("test"));
    assert!(json.contains("1.0.0"));
}

#[test]
fn test_service_info_deserialization() {
    let json = r#"{"name":"test","version":"1.0.0","description":null,"endpoints":{},"metadata":{},"health_check_endpoint":null,"dependencies":[],"capabilities":[],"metrics":null}"#;
    let info: CanonicalServiceInfo = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(info.name, "test");
    assert_eq!(info.version, "1.0.0");
}

#[test]
fn test_service_type_serialization() {
    let service_type = CanonicalServiceType::Web;
    let json = serde_json::to_string(&service_type).expect("Failed to serialize");
    assert!(json.contains("Web"));
}

#[test]
fn test_service_status_serialization() {
    let status = CanonicalServiceStatus::Running;
    let json = serde_json::to_string(&status).expect("Failed to serialize");
    assert!(json.contains("Running"));
}

#[test]
fn test_service_config_serialization() {
    let config = CanonicalServiceConfig::default();
    let json = serde_json::to_string(&config).expect("Failed to serialize");
    assert!(json.contains("unknown-service"));
}

// ==================== Integration Tests ====================

#[test]
fn test_full_service_config_integration() {
    let mut config = CanonicalServiceConfig {
        info: CanonicalServiceInfo::new("api-gateway", "3.0.0"),
        ..Default::default()
    };

    // Set up service info
    config
        .info
        .with_description("Primary API Gateway")
        .with_endpoint("http", "http://localhost:8080")
        .with_endpoint("https", "https://api.example.com")
        .with_metadata("region", "us-west-2")
        .with_capability("rate-limiting")
        .with_dependency("auth-service");

    // Set service type and status
    config.service_type = CanonicalServiceType::Web;
    config.status = CanonicalServiceStatus::Running;

    // Add configuration parameter
    let timeout_param = CanonicalServiceConfigParameter {
        name: "request_timeout".to_string(),
        value: "30".to_string(),
        description: Some("Request timeout in seconds".to_string()),
        required: false,
        default_value: Some("60".to_string()),
        allowed_values: AllowedValues::Range {
            min: 1.0,
            max: 300.0,
        },
    };
    config.config.insert("request_timeout".to_string(), timeout_param);

    // Add environment variables
    config.environment.insert("LOG_LEVEL".to_string(), "info".to_string());

    // Verify the complete configuration
    assert_eq!(config.info.name, "api-gateway");
    assert_eq!(config.info.version, "3.0.0");
    assert_eq!(config.info.endpoints.len(), 2);
    assert_eq!(config.service_type.as_str(), "web");
    assert_eq!(config.status, CanonicalServiceStatus::Running);
    assert_eq!(config.config.len(), 1);
    assert_eq!(config.environment.len(), 1);
}

#[test]
fn test_service_with_metrics_integration() {
    let mut info = CanonicalServiceInfo::new("monitoring-service", "1.5.0");

    info.metrics = Some(ServiceMetrics {
        request_count: 10000,
        error_count: 42,
        avg_response_time_ms: 75.3,
        uptime_seconds: 86400,
    });

    assert!(info.metrics.is_some());
    let metrics = info.metrics.as_ref().unwrap();
    assert_eq!(metrics.request_count, 10000);
    assert_eq!(metrics.error_count, 42);
}
