//! # Comprehensive Health Module Tests
//!
//! Complete test coverage for canonical health checking types and functionality

use songbird_types::health::{CanonicalHealthCheck, CanonicalHealthConfig, CanonicalHealthStatus};

// ============================================================================
// CanonicalHealthStatus Tests
// ============================================================================

#[test]
fn test_health_status_variants() {
    // Test all variants exist and are distinct
    let healthy = CanonicalHealthStatus::Healthy;
    let degraded = CanonicalHealthStatus::Degraded;
    let unhealthy = CanonicalHealthStatus::Unhealthy;
    let unknown = CanonicalHealthStatus::Unknown;

    assert!(matches!(healthy, CanonicalHealthStatus::Healthy));
    assert!(matches!(degraded, CanonicalHealthStatus::Degraded));
    assert!(matches!(unhealthy, CanonicalHealthStatus::Unhealthy));
    assert!(matches!(unknown, CanonicalHealthStatus::Unknown));
}

#[test]
fn test_health_status_default() {
    let status = CanonicalHealthStatus::default();
    assert_eq!(status, CanonicalHealthStatus::Unknown);
}

#[test]
fn test_health_status_equality() {
    let healthy1 = CanonicalHealthStatus::Healthy;
    let healthy2 = CanonicalHealthStatus::Healthy;
    let degraded = CanonicalHealthStatus::Degraded;

    assert_eq!(healthy1, healthy2);
    assert_ne!(healthy1, degraded);
}

#[test]
fn test_health_status_display() {
    assert_eq!(CanonicalHealthStatus::Healthy.to_string(), "Healthy");
    assert_eq!(CanonicalHealthStatus::Degraded.to_string(), "Degraded");
    assert_eq!(CanonicalHealthStatus::Unhealthy.to_string(), "Unhealthy");
    assert_eq!(CanonicalHealthStatus::Unknown.to_string(), "Unknown");
}

#[test]
fn test_health_status_clone() {
    let status = CanonicalHealthStatus::Healthy;
    let cloned = status;
    assert_eq!(status, cloned);
}

#[test]
fn test_health_status_debug() {
    let status = CanonicalHealthStatus::Healthy;
    let debug_string = format!("{status:?}");
    assert!(debug_string.contains("Healthy"));
}

#[test]
fn test_health_status_serialization() {
    let status = CanonicalHealthStatus::Healthy;
    let json = serde_json::to_string(&status).expect("Failed to serialize");
    let deserialized: CanonicalHealthStatus =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(status, deserialized);
}

// ============================================================================
// CanonicalHealthCheck Tests
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
    let check = CanonicalHealthCheck::degraded("Service under heavy load");
    assert_eq!(check.status, CanonicalHealthStatus::Degraded);
    assert_eq!(check.message, Some("Service under heavy load".to_string()));
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

#[test]
fn test_health_check_with_metric() {
    let mut check = CanonicalHealthCheck::healthy();
    check.with_metric("cpu_usage", 45.5);
    check.with_metric("memory_usage", 62.3);

    assert_eq!(check.metrics.len(), 2);
    assert_eq!(check.metrics.get("cpu_usage"), Some(&45.5));
    assert_eq!(check.metrics.get("memory_usage"), Some(&62.3));
}

#[test]
fn test_health_check_with_metric_chaining() {
    let mut check = CanonicalHealthCheck::healthy();
    check
        .with_metric("cpu_usage", 45.5)
        .with_metric("memory_usage", 62.3)
        .with_metric("disk_usage", 78.1);

    assert_eq!(check.metrics.len(), 3);
    assert!(check.metrics.contains_key("cpu_usage"));
    assert!(check.metrics.contains_key("memory_usage"));
    assert!(check.metrics.contains_key("disk_usage"));
}

#[test]
fn test_health_check_with_component() {
    let mut check = CanonicalHealthCheck::healthy();
    check.with_component("database", CanonicalHealthStatus::Healthy);
    check.with_component("cache", CanonicalHealthStatus::Degraded);

    assert_eq!(check.components.len(), 2);
    assert_eq!(check.components.get("database"), Some(&CanonicalHealthStatus::Healthy));
    assert_eq!(check.components.get("cache"), Some(&CanonicalHealthStatus::Degraded));
}

#[test]
fn test_health_check_with_component_chaining() {
    let mut check = CanonicalHealthCheck::healthy();
    check
        .with_component("database", CanonicalHealthStatus::Healthy)
        .with_component("cache", CanonicalHealthStatus::Degraded)
        .with_component("queue", CanonicalHealthStatus::Healthy);

    assert_eq!(check.components.len(), 3);
    assert!(check.components.contains_key("database"));
    assert!(check.components.contains_key("cache"));
    assert!(check.components.contains_key("queue"));
}

#[test]
fn test_health_check_mixed_chaining() {
    let mut check = CanonicalHealthCheck::healthy();
    check
        .with_metric("cpu_usage", 45.5)
        .with_component("database", CanonicalHealthStatus::Healthy)
        .with_metric("memory_usage", 62.3)
        .with_component("cache", CanonicalHealthStatus::Degraded);

    assert_eq!(check.metrics.len(), 2);
    assert_eq!(check.components.len(), 2);
}

#[test]
fn test_health_check_is_healthy() {
    let healthy_check = CanonicalHealthCheck::healthy();
    assert!(healthy_check.is_healthy());

    let degraded_check = CanonicalHealthCheck::degraded("Issue");
    assert!(!degraded_check.is_healthy());

    let unhealthy_check = CanonicalHealthCheck::unhealthy("Failure");
    assert!(!unhealthy_check.is_healthy());

    let unknown_check = CanonicalHealthCheck::default();
    assert!(!unknown_check.is_healthy());
}

#[test]
fn test_health_check_clone() {
    let mut original = CanonicalHealthCheck::healthy();
    original.with_metric("cpu", 50.0);
    original.with_component("db", CanonicalHealthStatus::Healthy);

    let cloned = original.clone();
    assert_eq!(cloned.status, original.status);
    assert_eq!(cloned.message, original.message);
    assert_eq!(cloned.metrics.len(), original.metrics.len());
    assert_eq!(cloned.components.len(), original.components.len());
}

#[test]
fn test_health_check_debug() {
    let check = CanonicalHealthCheck::healthy();
    let debug_string = format!("{check:?}");
    assert!(debug_string.contains("Healthy"));
}

#[test]
fn test_health_check_serialization() {
    let mut check = CanonicalHealthCheck::healthy();
    check.with_metric("cpu", 50.0);
    check.with_component("database", CanonicalHealthStatus::Healthy);

    let json = serde_json::to_string(&check).expect("Failed to serialize");
    let deserialized: CanonicalHealthCheck =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.status, check.status);
    assert_eq!(deserialized.message, check.message);
    assert_eq!(deserialized.metrics.get("cpu"), Some(&50.0));
    assert_eq!(deserialized.components.get("database"), Some(&CanonicalHealthStatus::Healthy));
}

// ============================================================================
// CanonicalHealthConfig Tests
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
fn test_health_config_clone() {
    let original = CanonicalHealthConfig::default();
    let cloned = original.clone();

    assert_eq!(cloned.enabled, original.enabled);
    assert_eq!(cloned.endpoint, original.endpoint);
    assert_eq!(cloned.check_interval_seconds, original.check_interval_seconds);
    assert_eq!(cloned.timeout_seconds, original.timeout_seconds);
}

#[test]
fn test_health_config_debug() {
    let config = CanonicalHealthConfig::default();
    let debug_string = format!("{config:?}");
    assert!(debug_string.contains("enabled"));
    assert!(debug_string.contains("endpoint"));
}

#[test]
fn test_health_config_serialization() {
    let config = CanonicalHealthConfig::default();
    let json = serde_json::to_string(&config).expect("Failed to serialize");
    let deserialized: CanonicalHealthConfig =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.enabled, config.enabled);
    assert_eq!(deserialized.endpoint, config.endpoint);
    assert_eq!(deserialized.check_interval_seconds, config.check_interval_seconds);
    assert_eq!(deserialized.timeout_seconds, config.timeout_seconds);
}

// ============================================================================
// Integration & Edge Case Tests
// ============================================================================

#[test]
fn test_comprehensive_health_check_scenario() {
    // Simulate a complete health check scenario
    let mut check = CanonicalHealthCheck::healthy();

    // Add system metrics
    check
        .with_metric("cpu_usage_percent", 45.2)
        .with_metric("memory_usage_percent", 67.8)
        .with_metric("disk_usage_percent", 54.3)
        .with_metric("network_latency_ms", 12.5);

    // Add component statuses
    check
        .with_component("database", CanonicalHealthStatus::Healthy)
        .with_component("cache", CanonicalHealthStatus::Healthy)
        .with_component("message_queue", CanonicalHealthStatus::Degraded)
        .with_component("external_api", CanonicalHealthStatus::Healthy);

    // Verify complete health check
    assert!(check.is_healthy());
    assert_eq!(check.metrics.len(), 4);
    assert_eq!(check.components.len(), 4);
    assert!(check.message.is_some());
}

#[test]
fn test_degraded_system_scenario() {
    let mut check = CanonicalHealthCheck::degraded("High memory usage detected");

    check
        .with_metric("memory_usage_percent", 92.5)
        .with_component("memory_subsystem", CanonicalHealthStatus::Degraded);

    assert!(!check.is_healthy());
    assert_eq!(check.status, CanonicalHealthStatus::Degraded);
    assert!(check.message.is_some());
}

#[test]
fn test_unhealthy_system_scenario() {
    let mut check = CanonicalHealthCheck::unhealthy("Critical database failure");

    check
        .with_component("database", CanonicalHealthStatus::Unhealthy)
        .with_component("cache", CanonicalHealthStatus::Degraded)
        .with_metric("error_rate_percent", 85.0);

    assert!(!check.is_healthy());
    assert_eq!(check.status, CanonicalHealthStatus::Unhealthy);
}

#[test]
fn test_empty_health_check() {
    let check = CanonicalHealthCheck::default();
    assert_eq!(check.status, CanonicalHealthStatus::Unknown);
    assert!(check.metrics.is_empty());
    assert!(check.components.is_empty());
    assert!(!check.is_healthy());
}

#[test]
fn test_metric_overwrite() {
    let mut check = CanonicalHealthCheck::healthy();
    check.with_metric("cpu", 50.0);
    check.with_metric("cpu", 60.0); // Overwrite

    assert_eq!(check.metrics.len(), 1);
    assert_eq!(check.metrics.get("cpu"), Some(&60.0));
}

#[test]
fn test_component_overwrite() {
    let mut check = CanonicalHealthCheck::healthy();
    check.with_component("database", CanonicalHealthStatus::Healthy);
    check.with_component("database", CanonicalHealthStatus::Degraded); // Overwrite

    assert_eq!(check.components.len(), 1);
    assert_eq!(check.components.get("database"), Some(&CanonicalHealthStatus::Degraded));
}

#[test]
fn test_health_config_disabled() {
    let config = CanonicalHealthConfig {
        enabled: false,
        ..Default::default()
    };

    assert!(!config.enabled);
    // When disabled, other settings should still be valid
    assert!(!config.endpoint.is_empty());
    assert!(config.check_interval_seconds > 0);
    assert!(config.timeout_seconds > 0);
}

#[test]
fn test_health_config_custom_endpoint() {
    let config = CanonicalHealthConfig {
        endpoint: "/api/v1/health/status".to_string(),
        ..Default::default()
    };

    assert_eq!(config.endpoint, "/api/v1/health/status");
}

#[test]
fn test_health_config_timing() {
    let config = CanonicalHealthConfig::default();

    // Timeout should be less than check interval
    assert!(config.timeout_seconds < config.check_interval_seconds);

    // Both should be reasonable values
    assert!(config.check_interval_seconds >= 1);
    assert!(config.timeout_seconds >= 1);
}
