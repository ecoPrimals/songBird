// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Comprehensive tests for the Health Check system
//!
//! Coverage goal: 0% → 80%+
//!
//! Tests all functionality of:
//! - `CanonicalHealthStatus` enum
//! - `CanonicalHealthCheck` struct
//! - `CanonicalHealthConfig` struct

use songbird_types::health::{CanonicalHealthCheck, CanonicalHealthConfig, CanonicalHealthStatus};

// ============================================================================
// HEALTH STATUS TESTS
// ============================================================================

#[test]
fn test_health_status_default() {
    let status = CanonicalHealthStatus::default();
    assert_eq!(status, CanonicalHealthStatus::Unknown);
}

#[test]
fn test_health_status_equality() {
    assert_eq!(CanonicalHealthStatus::Healthy, CanonicalHealthStatus::Healthy);
    assert_eq!(CanonicalHealthStatus::Degraded, CanonicalHealthStatus::Degraded);
    assert_eq!(CanonicalHealthStatus::Unhealthy, CanonicalHealthStatus::Unhealthy);
    assert_eq!(CanonicalHealthStatus::Unknown, CanonicalHealthStatus::Unknown);

    assert_ne!(CanonicalHealthStatus::Healthy, CanonicalHealthStatus::Degraded);
    assert_ne!(CanonicalHealthStatus::Healthy, CanonicalHealthStatus::Unhealthy);
}

#[test]
fn test_health_status_display() {
    assert_eq!(format!("{}", CanonicalHealthStatus::Healthy), "Healthy");
    assert_eq!(format!("{}", CanonicalHealthStatus::Degraded), "Degraded");
    assert_eq!(format!("{}", CanonicalHealthStatus::Unhealthy), "Unhealthy");
    assert_eq!(format!("{}", CanonicalHealthStatus::Unknown), "Unknown");
}

#[test]
fn test_health_status_debug() {
    let status = CanonicalHealthStatus::Healthy;
    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("Healthy"));
}

#[test]
fn test_health_status_clone() {
    let original = CanonicalHealthStatus::Healthy;
    let cloned = original;
    assert_eq!(original, cloned);
}

#[test]
fn test_health_status_serialization() {
    let status = CanonicalHealthStatus::Healthy;
    let json = serde_json::to_string(&status).expect("Should serialize");
    assert!(json.contains("Healthy"));

    let deserialized: CanonicalHealthStatus =
        serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(status, deserialized);
}

// ============================================================================
// HEALTH CHECK TESTS - Construction
// ============================================================================

#[test]
fn test_health_check_default() {
    let check = CanonicalHealthCheck::default();
    assert_eq!(check.status, CanonicalHealthStatus::Unknown);
    assert!(check.message.is_none());
    assert!(check.metrics.is_empty());
    assert!(check.components.is_empty());
}

#[test]
fn test_health_check_healthy() {
    let check = CanonicalHealthCheck::healthy();
    assert_eq!(check.status, CanonicalHealthStatus::Healthy);
    assert_eq!(check.message, Some("All systems operational".to_string()));
    assert!(check.metrics.is_empty());
    assert!(check.components.is_empty());
}

#[test]
fn test_health_check_degraded() {
    let check = CanonicalHealthCheck::degraded("Service slow");
    assert_eq!(check.status, CanonicalHealthStatus::Degraded);
    assert_eq!(check.message, Some("Service slow".to_string()));
    assert!(check.metrics.is_empty());
    assert!(check.components.is_empty());
}

#[test]
fn test_health_check_unhealthy() {
    let check = CanonicalHealthCheck::unhealthy("Database connection failed");
    assert_eq!(check.status, CanonicalHealthStatus::Unhealthy);
    assert_eq!(check.message, Some("Database connection failed".to_string()));
    assert!(check.metrics.is_empty());
    assert!(check.components.is_empty());
}

// ============================================================================
// HEALTH CHECK TESTS - Metrics
// ============================================================================

#[test]
fn test_health_check_with_single_metric() {
    let mut check = CanonicalHealthCheck::healthy();
    check.with_metric("cpu_usage", 45.5);

    assert_eq!(check.metrics.len(), 1);
    assert_eq!(check.metrics.get("cpu_usage"), Some(&45.5));
}

#[test]
fn test_health_check_with_multiple_metrics() {
    let mut check = CanonicalHealthCheck::healthy();
    check
        .with_metric("cpu_usage", 45.5)
        .with_metric("memory_usage", 78.2)
        .with_metric("disk_usage", 62.3);

    assert_eq!(check.metrics.len(), 3);
    assert_eq!(check.metrics.get("cpu_usage"), Some(&45.5));
    assert_eq!(check.metrics.get("memory_usage"), Some(&78.2));
    assert_eq!(check.metrics.get("disk_usage"), Some(&62.3));
}

#[test]
fn test_health_check_metric_chaining() {
    let mut check = CanonicalHealthCheck::degraded("High load");
    let result = check.with_metric("load_avg", 5.5);

    // Should return mutable reference for chaining
    assert_eq!(result.metrics.len(), 1);
}

#[test]
fn test_health_check_metric_update() {
    let mut check = CanonicalHealthCheck::healthy();
    check.with_metric("cpu_usage", 45.5);
    check.with_metric("cpu_usage", 75.0); // Update existing metric

    assert_eq!(check.metrics.len(), 1);
    assert_eq!(check.metrics.get("cpu_usage"), Some(&75.0));
}

// ============================================================================
// HEALTH CHECK TESTS - Components
// ============================================================================

#[test]
fn test_health_check_with_single_component() {
    let mut check = CanonicalHealthCheck::healthy();
    check.with_component("database", CanonicalHealthStatus::Healthy);

    assert_eq!(check.components.len(), 1);
    assert_eq!(check.components.get("database"), Some(&CanonicalHealthStatus::Healthy));
}

#[test]
fn test_health_check_with_multiple_components() {
    let mut check = CanonicalHealthCheck::degraded("Some services slow");
    check
        .with_component("database", CanonicalHealthStatus::Healthy)
        .with_component("cache", CanonicalHealthStatus::Degraded)
        .with_component("api", CanonicalHealthStatus::Healthy);

    assert_eq!(check.components.len(), 3);
    assert_eq!(check.components.get("database"), Some(&CanonicalHealthStatus::Healthy));
    assert_eq!(check.components.get("cache"), Some(&CanonicalHealthStatus::Degraded));
}

#[test]
fn test_health_check_component_chaining() {
    let mut check = CanonicalHealthCheck::healthy();
    let result = check.with_component("storage", CanonicalHealthStatus::Healthy);

    // Should return mutable reference for chaining
    assert_eq!(result.components.len(), 1);
}

#[test]
fn test_health_check_mixed_component_states() {
    let mut check = CanonicalHealthCheck::degraded("System partially degraded");
    check
        .with_component("service_a", CanonicalHealthStatus::Healthy)
        .with_component("service_b", CanonicalHealthStatus::Degraded)
        .with_component("service_c", CanonicalHealthStatus::Unhealthy)
        .with_component("service_d", CanonicalHealthStatus::Unknown);

    assert_eq!(check.components.len(), 4);
    assert_eq!(check.components.get("service_b"), Some(&CanonicalHealthStatus::Degraded));
    assert_eq!(check.components.get("service_c"), Some(&CanonicalHealthStatus::Unhealthy));
}

// ============================================================================
// HEALTH CHECK TESTS - Combined Metrics and Components
// ============================================================================

#[test]
fn test_health_check_with_metrics_and_components() {
    let mut check = CanonicalHealthCheck::healthy();
    check
        .with_metric("cpu_usage", 45.5)
        .with_metric("memory_usage", 78.2)
        .with_component("database", CanonicalHealthStatus::Healthy)
        .with_component("cache", CanonicalHealthStatus::Healthy);

    assert_eq!(check.metrics.len(), 2);
    assert_eq!(check.components.len(), 2);
    assert!(check.is_healthy());
}

#[test]
fn test_health_check_full_featured() {
    let mut check = CanonicalHealthCheck::degraded("Service experiencing issues");
    check
        .with_metric("response_time_ms", 1250.0)
        .with_metric("error_rate", 0.05)
        .with_metric("throughput_rps", 450.0)
        .with_component("frontend", CanonicalHealthStatus::Healthy)
        .with_component("backend", CanonicalHealthStatus::Degraded)
        .with_component("database", CanonicalHealthStatus::Healthy);

    assert_eq!(check.status, CanonicalHealthStatus::Degraded);
    assert!(check.message.is_some());
    assert_eq!(check.metrics.len(), 3);
    assert_eq!(check.components.len(), 3);
    assert!(!check.is_healthy());
}

// ============================================================================
// HEALTH CHECK TESTS - Status Checks
// ============================================================================

#[test]
fn test_is_healthy_true() {
    let check = CanonicalHealthCheck::healthy();
    assert!(check.is_healthy());
}

#[test]
fn test_is_healthy_false_degraded() {
    let check = CanonicalHealthCheck::degraded("Slow response");
    assert!(!check.is_healthy());
}

#[test]
fn test_is_healthy_false_unhealthy() {
    let check = CanonicalHealthCheck::unhealthy("Service down");
    assert!(!check.is_healthy());
}

#[test]
fn test_is_healthy_false_unknown() {
    let check = CanonicalHealthCheck::default();
    assert!(!check.is_healthy());
}

// ============================================================================
// HEALTH CHECK TESTS - Serialization
// ============================================================================

#[test]
fn test_health_check_serialization_simple() {
    let check = CanonicalHealthCheck::healthy();
    let json = serde_json::to_string(&check).expect("Should serialize");

    assert!(json.contains("Healthy"));
    assert!(json.contains("All systems operational"));
}

#[test]
fn test_health_check_serialization_with_metrics() {
    let mut check = CanonicalHealthCheck::healthy();
    check.with_metric("cpu", 45.5);

    let json = serde_json::to_string(&check).expect("Should serialize");
    let deserialized: CanonicalHealthCheck =
        serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.status, CanonicalHealthStatus::Healthy);
    assert_eq!(deserialized.metrics.get("cpu"), Some(&45.5));
}

#[test]
fn test_health_check_serialization_full() {
    let mut check = CanonicalHealthCheck::degraded("Test");
    check.with_metric("metric1", 100.0).with_component("comp1", CanonicalHealthStatus::Healthy);

    let json = serde_json::to_string_pretty(&check).expect("Should serialize");
    let deserialized: CanonicalHealthCheck =
        serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.status, check.status);
    assert_eq!(deserialized.message, check.message);
    assert_eq!(deserialized.metrics.len(), 1);
    assert_eq!(deserialized.components.len(), 1);
}

// ============================================================================
// HEALTH CHECK TESTS - Clone
// ============================================================================

#[test]
fn test_health_check_clone() {
    let mut original = CanonicalHealthCheck::healthy();
    original.with_metric("test", 42.0).with_component("service", CanonicalHealthStatus::Healthy);

    let cloned = original.clone();

    assert_eq!(cloned.status, original.status);
    assert_eq!(cloned.message, original.message);
    assert_eq!(cloned.metrics.len(), original.metrics.len());
    assert_eq!(cloned.components.len(), original.components.len());
}

// ============================================================================
// HEALTH CONFIG TESTS
// ============================================================================

#[test]
fn test_health_config_default() {
    let config = CanonicalHealthConfig::default();

    assert!(config.enabled);
    assert_eq!(config.endpoint, "/health");
    assert_eq!(config.check_interval_seconds, 30);
    assert_eq!(config.timeout_seconds, 5);
}

#[test]
fn test_health_config_custom() {
    let config = CanonicalHealthConfig {
        enabled: false,
        endpoint: "/status".to_string(),
        check_interval_seconds: 60,
        timeout_seconds: 10,
    };

    assert!(!config.enabled);
    assert_eq!(config.endpoint, "/status");
    assert_eq!(config.check_interval_seconds, 60);
    assert_eq!(config.timeout_seconds, 10);
}

#[test]
fn test_health_config_serialization() {
    let config = CanonicalHealthConfig::default();
    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: CanonicalHealthConfig =
        serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.enabled, config.enabled);
    assert_eq!(deserialized.endpoint, config.endpoint);
    assert_eq!(deserialized.check_interval_seconds, config.check_interval_seconds);
    assert_eq!(deserialized.timeout_seconds, config.timeout_seconds);
}

#[test]
fn test_health_config_clone() {
    let original = CanonicalHealthConfig::default();
    let cloned = original.clone();

    assert_eq!(cloned.enabled, original.enabled);
    assert_eq!(cloned.endpoint, original.endpoint);
}

// ============================================================================
// EDGE CASES AND INTEGRATION TESTS
// ============================================================================

#[test]
fn test_health_check_empty_message() {
    let check = CanonicalHealthCheck {
        status: CanonicalHealthStatus::Healthy,
        message: None,
        metrics: std::collections::HashMap::new(),
        components: std::collections::HashMap::new(),
    };

    assert!(check.message.is_none());
    assert!(check.is_healthy());
}

#[test]
fn test_health_check_large_metrics() {
    let mut check = CanonicalHealthCheck::healthy();

    // Add many metrics
    for i in 0..100 {
        check.with_metric(format!("metric_{}", i), f64::from(i));
    }

    assert_eq!(check.metrics.len(), 100);
}

#[test]
fn test_health_check_many_components() {
    let mut check = CanonicalHealthCheck::healthy();

    // Add many components
    for i in 0..50 {
        check.with_component(
            format!("service_{}", i),
            if i % 2 == 0 {
                CanonicalHealthStatus::Healthy
            } else {
                CanonicalHealthStatus::Degraded
            },
        );
    }

    assert_eq!(check.components.len(), 50);
}

#[test]
fn test_health_check_unicode_message() {
    let check = CanonicalHealthCheck::degraded("服务降级 🔥");
    assert_eq!(check.message, Some("服务降级 🔥".to_string()));
}

#[test]
fn test_health_check_long_message() {
    let long_message = "A".repeat(10000);
    let check = CanonicalHealthCheck::unhealthy(&long_message);
    assert_eq!(check.message, Some(long_message));
}

// ============================================================================
// REAL-WORLD SCENARIOS
// ============================================================================

#[test]
fn test_scenario_all_systems_operational() {
    let mut check = CanonicalHealthCheck::healthy();
    check
        .with_metric("cpu_usage", 25.0)
        .with_metric("memory_usage", 45.0)
        .with_metric("disk_usage", 60.0)
        .with_component("database", CanonicalHealthStatus::Healthy)
        .with_component("cache", CanonicalHealthStatus::Healthy)
        .with_component("api", CanonicalHealthStatus::Healthy);

    assert!(check.is_healthy());
    assert_eq!(check.status, CanonicalHealthStatus::Healthy);
}

#[test]
fn test_scenario_partial_degradation() {
    let mut check = CanonicalHealthCheck::degraded("Cache experiencing high load");
    check
        .with_metric("cache_hit_rate", 0.65) // Lower than normal 0.90
        .with_metric("cache_response_time_ms", 150.0) // Higher than normal 50ms
        .with_component("database", CanonicalHealthStatus::Healthy)
        .with_component("cache", CanonicalHealthStatus::Degraded)
        .with_component("api", CanonicalHealthStatus::Healthy);

    assert!(!check.is_healthy());
    assert_eq!(check.status, CanonicalHealthStatus::Degraded);
}

#[test]
fn test_scenario_critical_failure() {
    let mut check = CanonicalHealthCheck::unhealthy("Database connection lost");
    check
        .with_metric("failed_queries", 1500.0)
        .with_metric("connection_attempts", 25.0)
        .with_component("database", CanonicalHealthStatus::Unhealthy)
        .with_component("cache", CanonicalHealthStatus::Degraded)
        .with_component("api", CanonicalHealthStatus::Degraded);

    assert!(!check.is_healthy());
    assert_eq!(check.status, CanonicalHealthStatus::Unhealthy);
}
