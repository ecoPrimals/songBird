//! Comprehensive tests for primal and health types

use songbird_types::{
    CanonicalHealthCheck, CanonicalHealthStatus, CanonicalPrimalId, CanonicalPrimalType,
};
use std::collections::HashMap;

// ============================================================================
// PRIMAL TYPE TESTS
// ============================================================================

#[test]
fn test_primal_type_display() {
    assert_eq!(CanonicalPrimalType::Security.to_string(), "Security");
    assert_eq!(CanonicalPrimalType::Storage.to_string(), "Storage");
    assert_eq!(CanonicalPrimalType::Compute.to_string(), "Compute");
    assert_eq!(CanonicalPrimalType::Ai.to_string(), "AI");
    assert_eq!(CanonicalPrimalType::Orchestration.to_string(), "Orchestration");
    assert_eq!(CanonicalPrimalType::Federation.to_string(), "Federation");
    assert_eq!(CanonicalPrimalType::Discovery.to_string(), "Discovery");
    assert_eq!(CanonicalPrimalType::Registry.to_string(), "Registry");
    assert_eq!(CanonicalPrimalType::Observability.to_string(), "Observability");
}

#[test]
fn test_primal_type_unknown() {
    let custom = CanonicalPrimalType::Unknown("CustomPrimal".to_string());
    assert_eq!(custom.to_string(), "CustomPrimal");
}

#[test]
fn test_primal_type_default() {
    let default = CanonicalPrimalType::default();
    assert!(matches!(default, CanonicalPrimalType::Unknown(_)));
}

#[test]
fn test_primal_type_equality() {
    assert_eq!(CanonicalPrimalType::Security, CanonicalPrimalType::Security);
    assert_ne!(CanonicalPrimalType::Security, CanonicalPrimalType::Storage);
}

#[test]
fn test_primal_type_clone() {
    let primal = CanonicalPrimalType::Compute;
    let cloned = primal.clone();
    assert_eq!(primal, cloned);
}

#[test]
fn test_primal_type_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let primal = CanonicalPrimalType::Security;
    let json = serde_json::to_string(&primal)?;
    assert!(json.contains("Security"));

    let deserialized: CanonicalPrimalType = serde_json::from_str(&json)?;
    assert_eq!(primal, deserialized);
    Ok(())
}

// ============================================================================
// PRIMAL ID TESTS
// ============================================================================

#[test]
fn test_primal_id_creation() {
    let mut endpoints = HashMap::new();
    endpoints.insert("http".to_string(), "http://localhost:8080".to_string());

    let mut metadata = HashMap::new();
    metadata.insert("region".to_string(), "us-east-1".to_string());

    let primal_id = CanonicalPrimalId {
        primal_type: CanonicalPrimalType::Security,
        instance_id: "security-1".to_string(),
        version: "1.0.0".to_string(),
        endpoints,
        metadata,
    };

    assert_eq!(primal_id.primal_type, CanonicalPrimalType::Security);
    assert_eq!(primal_id.instance_id, "security-1");
    assert_eq!(primal_id.version, "1.0.0");
    assert_eq!(primal_id.endpoints.len(), 1);
    assert_eq!(primal_id.metadata.len(), 1);
}

#[test]
fn test_primal_id_default() {
    let primal_id = CanonicalPrimalId::default();
    assert_eq!(primal_id.instance_id, "default-instance");
    assert_eq!(primal_id.version, "0.1.0");
    assert!(primal_id.endpoints.is_empty());
    assert!(primal_id.metadata.is_empty());
}

#[test]
fn test_primal_id_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let primal_id = CanonicalPrimalId {
        primal_type: CanonicalPrimalType::Compute,
        instance_id: "compute-1".to_string(),
        version: "2.0.0".to_string(),
        endpoints: HashMap::new(),
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&primal_id)?;
    assert!(!json.is_empty());

    let deserialized: CanonicalPrimalId = serde_json::from_str(&json)?;
    assert_eq!(primal_id.primal_type, deserialized.primal_type);
    assert_eq!(primal_id.instance_id, deserialized.instance_id);
    Ok(())
}

#[test]
fn test_primal_id_with_multiple_endpoints() {
    let mut endpoints = HashMap::new();
    endpoints.insert("http".to_string(), "http://localhost:8080".to_string());
    endpoints.insert("grpc".to_string(), "grpc://localhost:9090".to_string());
    endpoints.insert("ws".to_string(), "ws://localhost:8081".to_string());

    let primal_id = CanonicalPrimalId {
        primal_type: CanonicalPrimalType::Ai,
        instance_id: "ai-1".to_string(),
        version: "1.5.0".to_string(),
        endpoints,
        metadata: HashMap::new(),
    };

    assert_eq!(primal_id.endpoints.len(), 3);
    assert!(primal_id.endpoints.contains_key("http"));
    assert!(primal_id.endpoints.contains_key("grpc"));
    assert!(primal_id.endpoints.contains_key("ws"));
}

// ============================================================================
// HEALTH STATUS TESTS
// ============================================================================

#[test]
fn test_health_status_display() {
    assert_eq!(CanonicalHealthStatus::Healthy.to_string(), "Healthy");
    assert_eq!(CanonicalHealthStatus::Degraded.to_string(), "Degraded");
    assert_eq!(CanonicalHealthStatus::Unhealthy.to_string(), "Unhealthy");
    assert_eq!(CanonicalHealthStatus::Unknown.to_string(), "Unknown");
}

#[test]
fn test_health_status_default() {
    let status = CanonicalHealthStatus::default();
    assert_eq!(status, CanonicalHealthStatus::Unknown);
}

#[test]
fn test_health_status_equality() {
    assert_eq!(CanonicalHealthStatus::Healthy, CanonicalHealthStatus::Healthy);
    assert_ne!(CanonicalHealthStatus::Healthy, CanonicalHealthStatus::Degraded);
}

#[test]
fn test_health_status_clone() {
    let status = CanonicalHealthStatus::Healthy;
    let cloned = status; // Copy trait allows direct assignment
    assert_eq!(status, cloned);
}

#[test]
fn test_health_status_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let status = CanonicalHealthStatus::Healthy;
    let json = serde_json::to_string(&status)?;
    assert!(!json.is_empty());

    let deserialized: CanonicalHealthStatus = serde_json::from_str(&json)?;
    assert_eq!(status, deserialized);
    Ok(())
}

// ============================================================================
// HEALTH CHECK TESTS
// ============================================================================

#[test]
fn test_health_check_healthy() {
    let health = CanonicalHealthCheck::healthy();
    assert_eq!(health.status, CanonicalHealthStatus::Healthy);
    assert!(health.message.is_some());
    assert_eq!(health.message.unwrap(), "All systems operational");
}

#[test]
fn test_health_check_degraded() {
    let health = CanonicalHealthCheck::degraded("High latency detected");
    assert_eq!(health.status, CanonicalHealthStatus::Degraded);
    assert!(health.message.is_some());
    assert!(health.message.unwrap().contains("High latency"));
}

#[test]
fn test_health_check_unhealthy() {
    let health = CanonicalHealthCheck::unhealthy("Service down");
    assert_eq!(health.status, CanonicalHealthStatus::Unhealthy);
    assert!(health.message.is_some());
    assert!(health.message.unwrap().contains("Service down"));
}

#[test]
fn test_health_check_default() {
    let health = CanonicalHealthCheck::default();
    assert_eq!(health.status, CanonicalHealthStatus::Unknown);
    assert!(health.message.is_none());
    assert!(health.metrics.is_empty());
    assert!(health.components.is_empty());
}

#[test]
fn test_health_check_with_metrics() {
    let mut metrics = HashMap::new();
    metrics.insert("cpu_usage".to_string(), 45.5);
    metrics.insert("memory_usage".to_string(), 62.3);
    metrics.insert("disk_usage".to_string(), 78.9);

    let health = CanonicalHealthCheck {
        status: CanonicalHealthStatus::Healthy,
        message: Some("System operational".to_string()),
        metrics,
        components: HashMap::new(),
    };

    assert_eq!(health.metrics.len(), 3);
    assert_eq!(health.metrics.get("cpu_usage"), Some(&45.5));
}

#[test]
fn test_health_check_with_components() {
    let mut components = HashMap::new();
    components.insert("database".to_string(), CanonicalHealthStatus::Healthy);
    components.insert("cache".to_string(), CanonicalHealthStatus::Degraded);
    components.insert("queue".to_string(), CanonicalHealthStatus::Healthy);

    let health = CanonicalHealthCheck {
        status: CanonicalHealthStatus::Degraded,
        message: Some("Cache performance degraded".to_string()),
        metrics: HashMap::new(),
        components,
    };

    assert_eq!(health.components.len(), 3);
    assert_eq!(health.components.get("database"), Some(&CanonicalHealthStatus::Healthy));
    assert_eq!(health.components.get("cache"), Some(&CanonicalHealthStatus::Degraded));
}

#[test]
fn test_health_check_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let health = CanonicalHealthCheck::healthy();
    let json = serde_json::to_string(&health)?;
    assert!(!json.is_empty());

    let deserialized: CanonicalHealthCheck = serde_json::from_str(&json)?;
    assert_eq!(health.status, deserialized.status);
    Ok(())
}

#[test]
fn test_health_check_complex_scenario() {
    let mut metrics = HashMap::new();
    metrics.insert("response_time_ms".to_string(), 150.0);
    metrics.insert("error_rate".to_string(), 0.02);
    metrics.insert("throughput".to_string(), 1000.0);

    let mut components = HashMap::new();
    components.insert("api".to_string(), CanonicalHealthStatus::Healthy);
    components.insert("database".to_string(), CanonicalHealthStatus::Degraded);
    components.insert("cache".to_string(), CanonicalHealthStatus::Healthy);
    components.insert("queue".to_string(), CanonicalHealthStatus::Healthy);

    let health = CanonicalHealthCheck {
        status: CanonicalHealthStatus::Degraded,
        message: Some("Database latency increased".to_string()),
        metrics,
        components,
    };

    // Overall status should be degraded due to database
    assert_eq!(health.status, CanonicalHealthStatus::Degraded);

    // All metrics should be present
    assert_eq!(health.metrics.len(), 3);

    // All components should be tracked
    assert_eq!(health.components.len(), 4);

    // Verify specific component states
    assert_eq!(health.components.get("database"), Some(&CanonicalHealthStatus::Degraded));
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_primal_with_health() {
    let primal_id = CanonicalPrimalId {
        primal_type: CanonicalPrimalType::Security,
        instance_id: "security-main".to_string(),
        version: "3.0.0".to_string(),
        endpoints: HashMap::new(),
        metadata: HashMap::new(),
    };

    let health = CanonicalHealthCheck::healthy();

    // In a real system, these would be associated
    assert_eq!(primal_id.primal_type, CanonicalPrimalType::Security);
    assert_eq!(health.status, CanonicalHealthStatus::Healthy);
}

#[test]
fn test_multiple_primals_different_health() {
    let primals = [
        (CanonicalPrimalType::Security, CanonicalHealthStatus::Healthy),
        (CanonicalPrimalType::Storage, CanonicalHealthStatus::Degraded),
        (CanonicalPrimalType::Compute, CanonicalHealthStatus::Healthy),
        (CanonicalPrimalType::Ai, CanonicalHealthStatus::Unhealthy),
    ];

    assert_eq!(primals.len(), 4);

    let healthy_count =
        primals.iter().filter(|(_, status)| *status == CanonicalHealthStatus::Healthy).count();

    assert_eq!(healthy_count, 2);
}
