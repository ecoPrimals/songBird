//! Comprehensive Types Tests for songbird-universal
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

//!
//! Tests for universal types module covering all data structures.

use songbird_universal::types::*;
use std::collections::HashMap;

// ============================================================================
// PRIMAL TYPE TESTS
// ============================================================================

#[test]
fn test_primal_type_new() {
    let primal = PrimalType::new("compute");
    assert_eq!(primal.category, "compute");
    assert_eq!(primal.subcategory, None);
    assert_eq!(primal.version, "1.0");
}

#[test]
fn test_primal_type_from_string() {
    let primal = PrimalType::from_string("storage");
    assert_eq!(primal.category, "storage");
    assert_eq!(primal.version, "1.0");
}

#[test]
fn test_primal_type_as_str() {
    let primal = PrimalType::new("ai");
    assert_eq!(primal.as_str(), "ai");
}

#[test]
fn test_primal_type_display() {
    let primal = PrimalType::new("security");
    assert_eq!(format!("{}", primal), "security");
}

#[test]
fn test_primal_type_equality() {
    let p1 = PrimalType::new("compute");
    let p2 = PrimalType::new("compute");
    let p3 = PrimalType::new("storage");

    assert_eq!(p1, p2);
    assert_ne!(p1, p3);
}

#[test]
fn test_primal_type_clone() {
    let p1 = PrimalType::new("network");
    let p2 = p1.clone();
    assert_eq!(p1, p2);
}

#[test]
fn test_primal_type_serialization() {
    let primal = PrimalType::new("compute");
    let json = serde_json::to_string(&primal).expect("Failed to serialize");
    let deserialized: PrimalType = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized, primal);
}

// ============================================================================
// SECURITY LEVEL TESTS
// ============================================================================

#[test]
fn test_security_level_all_variants() {
    let none = SecurityLevel::None;
    let basic = SecurityLevel::Basic;
    let standard = SecurityLevel::Standard;
    let high = SecurityLevel::High;
    let maximum = SecurityLevel::Maximum;

    assert_eq!(none, SecurityLevel::None);
    assert_eq!(basic, SecurityLevel::Basic);
    assert_eq!(standard, SecurityLevel::Standard);
    assert_eq!(high, SecurityLevel::High);
    assert_eq!(maximum, SecurityLevel::Maximum);
}

#[test]
fn test_security_level_default() {
    let default = SecurityLevel::default();
    assert_eq!(default, SecurityLevel::Standard);
}

#[test]
fn test_security_level_equality() {
    assert_eq!(SecurityLevel::High, SecurityLevel::High);
    assert_ne!(SecurityLevel::High, SecurityLevel::Basic);
}

#[test]
fn test_security_level_clone() {
    let level1 = SecurityLevel::Maximum;
    let level2 = level1.clone();
    assert_eq!(level1, level2);
}

#[test]
fn test_security_level_serialization() {
    let level = SecurityLevel::High;
    let json = serde_json::to_string(&level).expect("Failed to serialize");
    let deserialized: SecurityLevel = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized, level);
}

// ============================================================================
// QOS METRICS TESTS
// ============================================================================

#[test]
fn test_qos_metrics_default() {
    let qos = QosMetrics::default();
    assert_eq!(qos.latency_ms, None);
    assert_eq!(qos.throughput_ops_sec, None);
    assert_eq!(qos.availability, None);
    assert_eq!(qos.reliability, None);
}

#[test]
fn test_qos_metrics_with_values() {
    let qos = QosMetrics {
        latency_ms: Some(50.0),
        throughput_ops_sec: Some(1000.0),
        availability: Some(0.99),
        reliability: Some(0.999),
    };

    assert_eq!(qos.latency_ms, Some(50.0));
    assert_eq!(qos.throughput_ops_sec, Some(1000.0));
    assert_eq!(qos.availability, Some(0.99));
    assert_eq!(qos.reliability, Some(0.999));
}

#[test]
fn test_qos_metrics_clone() {
    let qos1 = QosMetrics {
        latency_ms: Some(100.0),
        throughput_ops_sec: Some(500.0),
        availability: Some(0.95),
        reliability: Some(0.98),
    };
    let qos2 = qos1.clone();

    assert_eq!(qos1.latency_ms, qos2.latency_ms);
}

#[test]
fn test_qos_metrics_serialization() {
    let qos = QosMetrics {
        latency_ms: Some(75.0),
        throughput_ops_sec: Some(800.0),
        availability: Some(0.97),
        reliability: Some(0.99),
    };

    let json = serde_json::to_string(&qos).expect("Failed to serialize");
    let deserialized: QosMetrics = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.latency_ms, qos.latency_ms);
}

// ============================================================================
// HEALTH STATUS TESTS
// ============================================================================

#[test]
fn test_health_status_all_variants() {
    let healthy = HealthStatus::Healthy;
    let degraded = HealthStatus::Degraded;
    let unhealthy = HealthStatus::Unhealthy;
    let unknown = HealthStatus::Unknown;

    assert_eq!(healthy, HealthStatus::Healthy);
    assert_eq!(degraded, HealthStatus::Degraded);
    assert_eq!(unhealthy, HealthStatus::Unhealthy);
    assert_eq!(unknown, HealthStatus::Unknown);
}

#[test]
fn test_health_status_default() {
    let default = HealthStatus::default();
    assert_eq!(default, HealthStatus::Unknown);
}

#[test]
fn test_health_status_equality() {
    assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
}

#[test]
fn test_health_status_clone() {
    let health1 = HealthStatus::Healthy;
    let health2 = health1.clone();
    assert_eq!(health1, health2);
}

#[test]
fn test_health_status_serialization() {
    let health = HealthStatus::Degraded;
    let json = serde_json::to_string(&health).expect("Failed to serialize");
    let deserialized: HealthStatus = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized, health);
}

// ============================================================================
// DISCOVERED CAPABILITY TESTS
// ============================================================================

#[test]
fn test_discovered_capability_creation() {
    let cap = DiscoveredCapability {
        name: "encryption".to_string(),
        version: "2.0.0".to_string(),
        description: "AES encryption capability".to_string(),
        provider: "security-primal".to_string(),
        endpoint: "https://security.example.com".to_string(),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Healthy,
    };

    assert_eq!(cap.name, "encryption");
    assert_eq!(cap.version, "2.0.0");
    assert_eq!(cap.provider, "security-primal");
}

#[test]
fn test_discovered_capability_clone() {
    let cap1 = DiscoveredCapability {
        name: "test-cap".to_string(),
        version: "1.0.0".to_string(),
        description: "Test capability".to_string(),
        provider: "test-provider".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Healthy,
    };

    let cap2 = cap1.clone();
    assert_eq!(cap1.name, cap2.name);
    assert_eq!(cap1.version, cap2.version);
}

#[test]
fn test_discovered_capability_serialization() {
    let cap = DiscoveredCapability {
        name: "storage".to_string(),
        version: "1.5.0".to_string(),
        description: "Object storage".to_string(),
        provider: "storage-primal".to_string(),
        endpoint: "https://storage.example.com".to_string(),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Healthy,
    };

    let json = serde_json::to_string(&cap).expect("Failed to serialize");
    let deserialized: DiscoveredCapability =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.name, cap.name);
}

// ============================================================================
// PRIMAL CAPABILITY TESTS
// ============================================================================

#[test]
fn test_primal_capability_creation() {
    let mut params = HashMap::new();
    params.insert("key".to_string(), serde_json::json!("value"));

    let cap = PrimalCapability {
        capability_type: "compute".to_string(),
        version: "1.0.0".to_string(),
        parameters: params,
        qos_metrics: QosMetrics::default(),
    };

    assert_eq!(cap.capability_type, "compute");
    assert_eq!(cap.version, "1.0.0");
    assert_eq!(cap.parameters.len(), 1);
}

#[test]
fn test_primal_capability_clone() {
    let cap1 = PrimalCapability {
        capability_type: "network".to_string(),
        version: "2.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QosMetrics::default(),
    };

    let cap2 = cap1.clone();
    assert_eq!(cap1.capability_type, cap2.capability_type);
}

#[test]
fn test_primal_capability_serialization() {
    let cap = PrimalCapability {
        capability_type: "storage".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QosMetrics::default(),
    };

    let json = serde_json::to_string(&cap).expect("Failed to serialize");
    let deserialized: PrimalCapability =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.capability_type, cap.capability_type);
}

// ============================================================================
// DISCOVERY FILTERS TESTS
// ============================================================================

#[test]
fn test_discovery_filters_default() {
    let filters = DiscoveryFilters::default();
    assert_eq!(filters.capability_types.len(), 0);
    assert_eq!(filters.security_levels.len(), 0);
    assert_eq!(filters.geographic_regions.len(), 0);
    assert!(filters.performance_requirements.is_none());
}

#[test]
fn test_discovery_filters_with_criteria() {
    let filters = DiscoveryFilters {
        capability_types: vec!["compute".to_string(), "storage".to_string()],
        security_levels: vec![SecurityLevel::High],
        geographic_regions: vec!["us-west".to_string()],
        performance_requirements: Some(QosMetrics::default()),
    };

    assert_eq!(filters.capability_types.len(), 2);
    assert_eq!(filters.security_levels.len(), 1);
    assert_eq!(filters.geographic_regions.len(), 1);
    assert!(filters.performance_requirements.is_some());
}

#[test]
fn test_discovery_filters_clone() {
    let filters1 = DiscoveryFilters {
        capability_types: vec!["ai".to_string()],
        security_levels: vec![SecurityLevel::Maximum],
        geographic_regions: vec!["eu-central".to_string()],
        performance_requirements: None,
    };

    let filters2 = filters1.clone();
    assert_eq!(filters1.capability_types, filters2.capability_types);
}

// ============================================================================
// SERVICE INFO TESTS
// ============================================================================

#[test]
fn test_service_info_creation() {
    let service = ServiceInfo {
        name: "test-service".to_string(),
        primal_type: PrimalType::new("compute"),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec![],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    assert_eq!(service.name, "test-service");
    assert_eq!(service.health, HealthStatus::Healthy);
}

#[test]
fn test_service_info_with_capabilities() {
    let cap = DiscoveredCapability {
        name: "test-cap".to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        provider: "test".to_string(),
        endpoint: "http://test".to_string(),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Healthy,
    };

    let service = ServiceInfo {
        name: "service-with-cap".to_string(),
        primal_type: PrimalType::new("ai"),
        endpoint: "http://ai:9000".to_string(),
        capabilities: vec![cap],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    assert_eq!(service.capabilities.len(), 1);
}

#[test]
fn test_service_info_serialization() {
    let service = ServiceInfo {
        name: "serialize-test".to_string(),
        primal_type: PrimalType::new("storage"),
        endpoint: "http://storage:8080".to_string(),
        capabilities: vec![],
        health: HealthStatus::Degraded,
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&service).expect("Failed to serialize");
    let deserialized: ServiceInfo = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.name, service.name);
}

// ============================================================================
// RESPONSE STATUS TESTS
// ============================================================================

#[test]
fn test_response_status_all_variants() {
    let success = ResponseStatus::Success;
    let partial = ResponseStatus::Partial;
    let failed = ResponseStatus::Failed;
    let timeout = ResponseStatus::Timeout;
    let not_found = ResponseStatus::NotFound;

    assert_eq!(success, ResponseStatus::Success);
    assert_eq!(partial, ResponseStatus::Partial);
    assert_eq!(failed, ResponseStatus::Failed);
    assert_eq!(timeout, ResponseStatus::Timeout);
    assert_eq!(not_found, ResponseStatus::NotFound);
}

#[test]
fn test_response_status_default() {
    let default = ResponseStatus::default();
    assert_eq!(default, ResponseStatus::Success);
}

#[test]
fn test_response_status_equality() {
    assert_eq!(ResponseStatus::Success, ResponseStatus::Success);
    assert_ne!(ResponseStatus::Success, ResponseStatus::Failed);
}

#[test]
fn test_response_status_serialization() {
    let status = ResponseStatus::Timeout;
    let json = serde_json::to_string(&status).expect("Failed to serialize");
    let deserialized: ResponseStatus = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized, status);
}

// ============================================================================
// UNIVERSAL REQUEST TESTS
// ============================================================================

#[test]
fn test_universal_request_creation() {
    let request = UniversalRequest {
        request_id: "req-001".to_string(),
        source: "client-1".to_string(),
        target: "service-1".to_string(),
        action: "query".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    assert_eq!(request.request_id, "req-001");
    assert_eq!(request.action, "query");
}

#[test]
fn test_universal_request_with_parameters() {
    let mut params = HashMap::new();
    params.insert("key".to_string(), serde_json::json!("value"));

    let request = UniversalRequest {
        request_id: "req-002".to_string(),
        source: "client-2".to_string(),
        target: "service-2".to_string(),
        action: "execute".to_string(),
        parameters: params,
        security_context: None,
    };

    assert_eq!(request.parameters.len(), 1);
}

#[test]
fn test_universal_request_serialization() {
    let request = UniversalRequest {
        request_id: "req-003".to_string(),
        source: "test-source".to_string(),
        target: "test-target".to_string(),
        action: "test-action".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let json = serde_json::to_string(&request).expect("Failed to serialize");
    let deserialized: UniversalRequest =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.request_id, request.request_id);
}

// ============================================================================
// UNIVERSAL RESPONSE TESTS
// ============================================================================

#[test]
fn test_universal_response_success() {
    let response = UniversalResponse {
        request_id: "req-001".to_string(),
        status: ResponseStatus::Success,
        data: Some(serde_json::json!({"result": "ok"})),
        error: None,
        metadata: HashMap::new(),
    };

    assert_eq!(response.status, ResponseStatus::Success);
    assert!(response.data.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_universal_response_error() {
    let response = UniversalResponse {
        request_id: "req-002".to_string(),
        status: ResponseStatus::Failed,
        data: None,
        error: Some("Operation failed".to_string()),
        metadata: HashMap::new(),
    };

    assert_eq!(response.status, ResponseStatus::Failed);
    assert!(response.error.is_some());
}

#[test]
fn test_universal_response_serialization() {
    let response = UniversalResponse {
        request_id: "req-003".to_string(),
        status: ResponseStatus::Success,
        data: None,
        error: None,
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&response).expect("Failed to serialize");
    let deserialized: UniversalResponse =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.request_id, response.request_id);
}

// ============================================================================
// SECURITY CONFIG TESTS
// ============================================================================

#[test]
fn test_security_config_creation() {
    let config = SecurityConfig {
        enabled: true,
        level: SecurityLevel::High,
        authentication_required: true,
        tls_enabled: true,
        certificate_path: Some("/path/to/cert.pem".to_string()),
    };

    assert!(config.enabled);
    assert_eq!(config.level, SecurityLevel::High);
    assert!(config.authentication_required);
}

#[test]
fn test_security_config_disabled() {
    let config = SecurityConfig {
        enabled: false,
        level: SecurityLevel::None,
        authentication_required: false,
        tls_enabled: false,
        certificate_path: None,
    };

    assert!(!config.enabled);
    assert_eq!(config.level, SecurityLevel::None);
}

#[test]
fn test_security_config_serialization() {
    let config = SecurityConfig {
        enabled: true,
        level: SecurityLevel::Maximum,
        authentication_required: true,
        tls_enabled: true,
        certificate_path: Some("/cert.pem".to_string()),
    };

    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: SecurityConfig = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.enabled, config.enabled);
}

// ============================================================================
// SECURITY CONTEXT TESTS
// ============================================================================

#[test]
fn test_security_context_creation() {
    let context = SecurityContext {
        user_id: Some("user-123".to_string()),
        session_id: "session-abc".to_string(),
        permissions: vec!["read".to_string(), "write".to_string()],
        security_level: SecurityLevel::High,
    };

    assert_eq!(context.user_id, Some("user-123".to_string()));
    assert_eq!(context.permissions.len(), 2);
}

#[test]
fn test_security_context_anonymous() {
    let context = SecurityContext {
        user_id: None,
        session_id: "anon-session".to_string(),
        permissions: vec!["read".to_string()],
        security_level: SecurityLevel::Basic,
    };

    assert!(context.user_id.is_none());
    assert_eq!(context.permissions.len(), 1);
}

#[test]
fn test_security_context_serialization() {
    let context = SecurityContext {
        user_id: Some("user-456".to_string()),
        session_id: "session-xyz".to_string(),
        permissions: vec!["admin".to_string()],
        security_level: SecurityLevel::Maximum,
    };

    let json = serde_json::to_string(&context).expect("Failed to serialize");
    let deserialized: SecurityContext = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.user_id, context.user_id);
}
