//! Comprehensive Tests for Universal Types
//!
//! This test suite validates all type definitions in the songbird-universal types module.

use songbird_universal::types::*;
use std::collections::HashMap;

// ============================================================================
// PRIMAL TYPE TESTS
// ============================================================================

#[test]
fn test_primal_type_creation() {
    let primal = PrimalType::new("compute");

    assert_eq!(primal.category, "compute");
    assert_eq!(primal.subcategory, None);
    assert_eq!(primal.version, "1.0");
}

#[test]
fn test_primal_type_from_string() {
    let primal = PrimalType::from_string("ai");

    assert_eq!(primal.category, "ai");
    assert_eq!(primal.as_str(), "ai");
}

#[test]
fn test_primal_type_display() {
    let primal = PrimalType::new("storage");
    let display_string = format!("{}", primal);

    assert_eq!(display_string, "storage");
}

#[test]
fn test_primal_type_equality() {
    let primal1 = PrimalType::new("compute");
    let primal2 = PrimalType::new("compute");
    let primal3 = PrimalType::new("storage");

    assert_eq!(primal1, primal2);
    assert_ne!(primal1, primal3);
}

#[test]
fn test_primal_type_serialization() {
    let primal = PrimalType::new("ai");
    let json = serde_json::to_string(&primal).expect("Failed to serialize");
    let deserialized: PrimalType = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(primal, deserialized);
}

// ============================================================================
// SECURITY LEVEL TESTS
// ============================================================================

#[test]
fn test_security_level_default() {
    let default_level = SecurityLevel::default();
    assert_eq!(default_level, SecurityLevel::Standard);
}

#[test]
fn test_security_level_ordering() {
    let levels = vec![
        SecurityLevel::None,
        SecurityLevel::Basic,
        SecurityLevel::Standard,
        SecurityLevel::High,
        SecurityLevel::Maximum,
    ];

    // Ensure all levels are distinct
    for (i, level1) in levels.iter().enumerate() {
        for (j, level2) in levels.iter().enumerate() {
            if i == j {
                assert_eq!(level1, level2);
            } else {
                assert_ne!(level1, level2);
            }
        }
    }
}

#[test]
fn test_security_level_serialization() {
    let level = SecurityLevel::High;
    let json = serde_json::to_string(&level).expect("Failed to serialize");
    let deserialized: SecurityLevel = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(level, deserialized);
}

// ============================================================================
// QOS METRICS TESTS
// ============================================================================

#[test]
fn test_qos_metrics_default() {
    let metrics = QosMetrics::default();

    assert_eq!(metrics.latency_ms, None);
    assert_eq!(metrics.throughput_ops_sec, None);
    assert_eq!(metrics.availability, None);
    assert_eq!(metrics.reliability, None);
}

#[test]
fn test_qos_metrics_creation() {
    let metrics = QosMetrics {
        latency_ms: Some(50.0),
        throughput_ops_sec: Some(1000.0),
        availability: Some(0.999),
        reliability: Some(0.95),
    };

    assert_eq!(metrics.latency_ms, Some(50.0));
    assert_eq!(metrics.throughput_ops_sec, Some(1000.0));
    assert_eq!(metrics.availability, Some(0.999));
    assert_eq!(metrics.reliability, Some(0.95));
}

#[test]
fn test_qos_metrics_serialization() {
    let metrics = QosMetrics {
        latency_ms: Some(100.0),
        throughput_ops_sec: Some(500.0),
        availability: Some(0.99),
        reliability: Some(0.98),
    };

    let json = serde_json::to_string(&metrics).expect("Failed to serialize");
    let deserialized: QosMetrics = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(metrics.latency_ms, deserialized.latency_ms);
    assert_eq!(metrics.throughput_ops_sec, deserialized.throughput_ops_sec);
}

// ============================================================================
// HEALTH STATUS TESTS
// ============================================================================

#[test]
fn test_health_status_default() {
    let status = HealthStatus::default();
    assert_eq!(status, HealthStatus::Unknown);
}

#[test]
fn test_health_status_variants() {
    let healthy = HealthStatus::Healthy;
    let degraded = HealthStatus::Degraded;
    let unhealthy = HealthStatus::Unhealthy;
    let unknown = HealthStatus::Unknown;

    assert_ne!(healthy, degraded);
    assert_ne!(degraded, unhealthy);
    assert_ne!(unhealthy, unknown);
    assert_ne!(unknown, healthy);
}

#[test]
fn test_health_status_serialization() {
    let status = HealthStatus::Healthy;
    let json = serde_json::to_string(&status).expect("Failed to serialize");
    let deserialized: HealthStatus = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(status, deserialized);
}

// ============================================================================
// CAPABILITY TESTS
// ============================================================================

#[test]
fn test_capability_creation() {
    let capability = Capability {
        name: "inference".to_string(),
        version: "1.0".to_string(),
        description: "AI inference capability".to_string(),
        provider: "SquirrelAI".to_string(),
        endpoint: "http://localhost:8083".to_string(),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Healthy,
    };

    assert_eq!(capability.name, "inference");
    assert_eq!(capability.version, "1.0");
    assert_eq!(capability.provider, "SquirrelAI");
    assert_eq!(capability.health_status, HealthStatus::Healthy);
}

#[test]
fn test_capability_with_qos() {
    let qos = QosMetrics {
        latency_ms: Some(200.0),
        throughput_ops_sec: Some(100.0),
        availability: Some(0.999),
        reliability: Some(0.99),
    };

    let capability = Capability {
        name: "storage".to_string(),
        version: "2.0".to_string(),
        description: "Object storage".to_string(),
        provider: "NestGate".to_string(),
        endpoint: "http://localhost:8082".to_string(),
        qos_metrics: qos.clone(),
        health_status: HealthStatus::Healthy,
    };

    assert_eq!(capability.qos_metrics.latency_ms, Some(200.0));
    assert_eq!(capability.qos_metrics.availability, Some(0.999));
}

#[test]
fn test_capability_serialization() {
    let capability = Capability {
        name: "compute".to_string(),
        version: "1.0".to_string(),
        description: "Compute capability".to_string(),
        provider: "ToadStool".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Degraded,
    };

    let json = serde_json::to_string(&capability).expect("Failed to serialize");
    let deserialized: Capability = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(capability.name, deserialized.name);
    assert_eq!(capability.provider, deserialized.provider);
}

// ============================================================================
// PRIMAL CAPABILITY TESTS
// ============================================================================

#[test]
fn test_primal_capability_creation() {
    let mut parameters = HashMap::new();
    parameters.insert("model_type".to_string(), serde_json::json!("gpt-4"));
    parameters.insert("max_tokens".to_string(), serde_json::json!(2048));

    let capability = PrimalCapability {
        capability_type: "inference".to_string(),
        version: "1.0".to_string(),
        parameters,
        qos_metrics: QosMetrics::default(),
    };

    assert_eq!(capability.capability_type, "inference");
    assert_eq!(capability.parameters.len(), 2);
    assert_eq!(capability.parameters.get("model_type").unwrap(), "gpt-4");
}

#[test]
fn test_primal_capability_with_metrics() {
    let qos = QosMetrics {
        latency_ms: Some(300.0),
        throughput_ops_sec: Some(50.0),
        availability: Some(0.98),
        reliability: Some(0.97),
    };

    let capability = PrimalCapability {
        capability_type: "embedding".to_string(),
        version: "1.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: qos,
    };

    assert_eq!(capability.qos_metrics.latency_ms, Some(300.0));
    assert_eq!(capability.qos_metrics.reliability, Some(0.97));
}

// ============================================================================
// SERVICE INFO TESTS
// ============================================================================

#[test]
fn test_service_info_creation() {
    let primal_type = PrimalType::new("compute");
    let capability = Capability {
        name: "batch_processing".to_string(),
        version: "1.0".to_string(),
        description: "Batch compute".to_string(),
        provider: "ToadStool".to_string(),
        endpoint: "http://localhost:8080".to_string(),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Healthy,
    };

    let service = ServiceInfo {
        name: "compute-service-1".to_string(),
        primal_type,
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec![capability],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    assert_eq!(service.name, "compute-service-1");
    assert_eq!(service.capabilities.len(), 1);
    assert_eq!(service.health, HealthStatus::Healthy);
}

#[test]
fn test_service_info_multiple_capabilities() {
    let cap1 = Capability {
        name: "inference".to_string(),
        version: "1.0".to_string(),
        description: "AI inference".to_string(),
        provider: "SquirrelAI".to_string(),
        endpoint: "http://localhost:8083".to_string(),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Healthy,
    };

    let cap2 = Capability {
        name: "embedding".to_string(),
        version: "1.0".to_string(),
        description: "Text embedding".to_string(),
        provider: "SquirrelAI".to_string(),
        endpoint: "http://localhost:8083".to_string(),
        qos_metrics: QosMetrics::default(),
        health_status: HealthStatus::Healthy,
    };

    let service = ServiceInfo {
        name: "ai-service-1".to_string(),
        primal_type: PrimalType::new("ai"),
        endpoint: "http://localhost:8083".to_string(),
        capabilities: vec![cap1, cap2],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    assert_eq!(service.capabilities.len(), 2);
    assert_eq!(service.capabilities[0].name, "inference");
    assert_eq!(service.capabilities[1].name, "embedding");
}

#[test]
fn test_service_info_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("region".to_string(), "us-west-1".to_string());
    metadata.insert("datacenter".to_string(), "dc1".to_string());
    metadata.insert("version".to_string(), "2.1.0".to_string());

    let service = ServiceInfo {
        name: "storage-service".to_string(),
        primal_type: PrimalType::new("storage"),
        endpoint: "http://localhost:8082".to_string(),
        capabilities: vec![],
        health: HealthStatus::Healthy,
        metadata: metadata.clone(),
    };

    assert_eq!(service.metadata.len(), 3);
    assert_eq!(service.metadata.get("region").unwrap(), "us-west-1");
    assert_eq!(service.metadata.get("datacenter").unwrap(), "dc1");
}

// ============================================================================
// DISCOVERY FILTERS TESTS
// ============================================================================

#[test]
fn test_discovery_filters_default() {
    let filters = DiscoveryFilters::default();

    assert!(filters.capability_types.is_empty());
    assert!(filters.security_levels.is_empty());
    assert!(filters.geographic_regions.is_empty());
    assert!(filters.performance_requirements.is_none());
}

#[test]
fn test_discovery_filters_with_capabilities() {
    let filters = DiscoveryFilters {
        capability_types: vec![
            "inference".to_string(),
            "embedding".to_string(),
            "storage".to_string(),
        ],
        security_levels: vec![SecurityLevel::High, SecurityLevel::Maximum],
        geographic_regions: vec!["us-west".to_string(), "eu-central".to_string()],
        performance_requirements: None,
    };

    assert_eq!(filters.capability_types.len(), 3);
    assert_eq!(filters.security_levels.len(), 2);
    assert_eq!(filters.geographic_regions.len(), 2);
}

#[test]
fn test_discovery_filters_with_performance() {
    let qos = QosMetrics {
        latency_ms: Some(100.0),
        throughput_ops_sec: Some(1000.0),
        availability: Some(0.99),
        reliability: Some(0.95),
    };

    let filters = DiscoveryFilters {
        capability_types: vec![],
        security_levels: vec![],
        geographic_regions: vec![],
        performance_requirements: Some(qos),
    };

    assert!(filters.performance_requirements.is_some());
    let perf = filters.performance_requirements.unwrap();
    assert_eq!(perf.latency_ms, Some(100.0));
    assert_eq!(perf.availability, Some(0.99));
}

#[test]
fn test_discovery_filters_serialization() {
    let filters = DiscoveryFilters {
        capability_types: vec!["compute".to_string()],
        security_levels: vec![SecurityLevel::Standard],
        geographic_regions: vec!["us-east".to_string()],
        performance_requirements: None,
    };

    let json = serde_json::to_string(&filters).expect("Failed to serialize");
    let deserialized: DiscoveryFilters =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(filters.capability_types.len(), deserialized.capability_types.len());
    assert_eq!(filters.security_levels.len(), deserialized.security_levels.len());
}

// ============================================================================
// UNIVERSAL REQUEST/RESPONSE TESTS
// ============================================================================

#[test]
fn test_universal_request_creation() {
    let mut parameters = HashMap::new();
    parameters.insert("query".to_string(), serde_json::json!("test query"));
    parameters.insert("max_results".to_string(), serde_json::json!(10));

    let request = UniversalRequest {
        request_id: "req-001".to_string(),
        source: "test-client".to_string(),
        target: "test-service".to_string(),
        action: "search".to_string(),
        parameters,
        security_context: None,
    };

    assert_eq!(request.action, "search");
    assert_eq!(request.parameters.len(), 2);
    assert_eq!(request.parameters.get("query").unwrap(), "test query");
    assert_eq!(request.source, "test-client");
    assert_eq!(request.target, "test-service");
}

#[test]
fn test_universal_request_with_security_context() {
    let security_context = SecurityContext {
        user_id: Some("user-123".to_string()),
        session_id: "session-456".to_string(),
        permissions: vec!["read".to_string(), "write".to_string()],
        security_level: SecurityLevel::High,
    };

    let request = UniversalRequest {
        request_id: "req-002".to_string(),
        source: "secure-client".to_string(),
        target: "secure-service".to_string(),
        action: "process".to_string(),
        parameters: HashMap::new(),
        security_context: Some(security_context),
    };

    assert!(request.security_context.is_some());
    let ctx = request.security_context.unwrap();
    assert_eq!(ctx.user_id, Some("user-123".to_string()));
    assert_eq!(ctx.permissions.len(), 2);
    assert_eq!(ctx.security_level, SecurityLevel::High);
}

#[test]
fn test_universal_response_success() {
    let data = serde_json::json!({
        "result": "success",
        "count": 42
    });

    let response = UniversalResponse {
        request_id: "req-001".to_string(),
        status: ResponseStatus::Success,
        data: Some(data),
        error: None,
        metadata: HashMap::new(),
    };

    assert_eq!(response.status, ResponseStatus::Success);
    assert_eq!(response.error, None);
    assert!(response.data.is_some());

    let data_val = response.data.unwrap();
    assert_eq!(data_val.get("result").unwrap(), "success");
    assert_eq!(data_val.get("count").unwrap(), 42);
}

#[test]
fn test_universal_response_error() {
    let mut metadata = HashMap::new();
    metadata.insert("error_code".to_string(), "TIMEOUT".to_string());

    let response = UniversalResponse {
        request_id: "req-003".to_string(),
        status: ResponseStatus::Failed,
        data: None,
        error: Some("Operation failed: timeout".to_string()),
        metadata,
    };

    assert_eq!(response.status, ResponseStatus::Failed);
    assert!(response.error.is_some());
    assert_eq!(response.error.unwrap(), "Operation failed: timeout");
    assert_eq!(response.metadata.get("error_code").unwrap(), "TIMEOUT");
}

#[test]
fn test_universal_request_response_serialization() {
    let mut parameters = HashMap::new();
    parameters.insert("key".to_string(), serde_json::json!("value"));

    let request = UniversalRequest {
        request_id: "req-004".to_string(),
        source: "test".to_string(),
        target: "test".to_string(),
        action: "test".to_string(),
        parameters,
        security_context: None,
    };

    let json = serde_json::to_string(&request).expect("Failed to serialize request");
    let deserialized: UniversalRequest =
        serde_json::from_str(&json).expect("Failed to deserialize request");

    assert_eq!(request.action, deserialized.action);
    assert_eq!(request.parameters.len(), deserialized.parameters.len());
    assert_eq!(request.request_id, deserialized.request_id);
}

#[test]
fn test_response_status_variants() {
    let success = ResponseStatus::Success;
    let partial = ResponseStatus::Partial;
    let failed = ResponseStatus::Failed;
    let timeout = ResponseStatus::Timeout;
    let not_found = ResponseStatus::NotFound;

    assert_ne!(success, partial);
    assert_ne!(partial, failed);
    assert_ne!(failed, timeout);
    assert_ne!(timeout, not_found);
}

#[test]
fn test_response_status_default() {
    let status = ResponseStatus::default();
    assert_eq!(status, ResponseStatus::Success);
}

#[test]
fn test_security_context_creation() {
    let context = SecurityContext {
        user_id: Some("user-789".to_string()),
        session_id: "session-xyz".to_string(),
        permissions: vec![
            "read:data".to_string(),
            "write:data".to_string(),
            "admin:system".to_string(),
        ],
        security_level: SecurityLevel::Maximum,
    };

    assert_eq!(context.user_id, Some("user-789".to_string()));
    assert_eq!(context.session_id, "session-xyz");
    assert_eq!(context.permissions.len(), 3);
    assert!(context.permissions.contains(&"admin:system".to_string()));
    assert_eq!(context.security_level, SecurityLevel::Maximum);
}

// ============================================================================
// COMPLEX INTEGRATION TESTS
// ============================================================================

#[test]
fn test_complete_service_discovery_scenario() {
    // Create comprehensive service info
    let qos = QosMetrics {
        latency_ms: Some(50.0),
        throughput_ops_sec: Some(1000.0),
        availability: Some(0.999),
        reliability: Some(0.99),
    };

    let capability = Capability {
        name: "inference".to_string(),
        version: "2.0".to_string(),
        description: "High-performance AI inference".to_string(),
        provider: "SquirrelAI-Premium".to_string(),
        endpoint: "http://ai-service:8083".to_string(),
        qos_metrics: qos,
        health_status: HealthStatus::Healthy,
    };

    let mut metadata = HashMap::new();
    metadata.insert("region".to_string(), "us-west-1".to_string());
    metadata.insert("tier".to_string(), "premium".to_string());

    let service = ServiceInfo {
        name: "ai-premium-1".to_string(),
        primal_type: PrimalType::new("ai"),
        endpoint: "http://ai-service:8083".to_string(),
        capabilities: vec![capability],
        health: HealthStatus::Healthy,
        metadata,
    };

    // Validate the complete service info
    assert_eq!(service.name, "ai-premium-1");
    assert_eq!(service.primal_type.as_str(), "ai");
    assert_eq!(service.capabilities.len(), 1);
    assert_eq!(service.capabilities[0].name, "inference");
    assert_eq!(service.capabilities[0].version, "2.0");
    assert_eq!(service.capabilities[0].qos_metrics.latency_ms, Some(50.0));
    assert_eq!(service.health, HealthStatus::Healthy);
    assert_eq!(service.metadata.get("tier").unwrap(), "premium");
}

#[test]
fn test_filter_based_discovery() {
    // Create filters
    let perf_requirements = QosMetrics {
        latency_ms: Some(100.0),
        throughput_ops_sec: Some(500.0),
        availability: Some(0.99),
        reliability: Some(0.95),
    };

    let filters = DiscoveryFilters {
        capability_types: vec!["inference".to_string(), "embedding".to_string()],
        security_levels: vec![SecurityLevel::High, SecurityLevel::Maximum],
        geographic_regions: vec!["us-west".to_string()],
        performance_requirements: Some(perf_requirements),
    };

    // Validate filters
    assert_eq!(filters.capability_types.len(), 2);
    assert!(filters.capability_types.contains(&"inference".to_string()));
    assert!(filters.security_levels.contains(&SecurityLevel::High));
    assert!(filters.performance_requirements.is_some());

    let perf = filters.performance_requirements.unwrap();
    assert_eq!(perf.latency_ms, Some(100.0));
}
