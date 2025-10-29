//! Comprehensive Health Module Tests
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
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]

//!
//! Tests for `songbird_types::health` module to achieve full coverage.

use songbird_types::health::*;

// ============================================================================
// HEALTH STATUS TESTS
// ============================================================================

#[test]
fn test_health_status_default() {
    let status = CanonicalHealthStatus::default();
    assert_eq!(status, CanonicalHealthStatus::Unknown);
}

#[test]
fn test_health_status_display() {
    assert_eq!(format!("{}", CanonicalHealthStatus::Healthy), "Healthy");
    assert_eq!(format!("{}", CanonicalHealthStatus::Degraded), "Degraded");
    assert_eq!(format!("{}", CanonicalHealthStatus::Unhealthy), "Unhealthy");
    assert_eq!(format!("{}", CanonicalHealthStatus::Unknown), "Unknown");
}

#[test]
fn test_health_status_equality() {
    assert_eq!(CanonicalHealthStatus::Healthy, CanonicalHealthStatus::Healthy);
    assert_ne!(CanonicalHealthStatus::Healthy, CanonicalHealthStatus::Degraded);
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
    let debug_str = format!("{status:?}");
    assert!(debug_str.contains("Healthy"));
}

// ============================================================================
// HEALTH CHECK TESTS
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
    assert!(check.message.is_some());
    assert_eq!(check.message.as_deref(), Some("All systems operational"));
    assert!(check.is_healthy());
}

#[test]
fn test_health_check_degraded() {
    let check = CanonicalHealthCheck::degraded("Service slow");
    assert_eq!(check.status, CanonicalHealthStatus::Degraded);
    assert_eq!(check.message.as_deref(), Some("Service slow"));
    assert!(!check.is_healthy());
}

#[test]
fn test_health_check_unhealthy() {
    let check = CanonicalHealthCheck::unhealthy("Service down");
    assert_eq!(check.status, CanonicalHealthStatus::Unhealthy);
    assert_eq!(check.message.as_deref(), Some("Service down"));
    assert!(!check.is_healthy());
}

#[test]
fn test_health_check_with_metric() {
    let mut check = CanonicalHealthCheck::healthy();
    check.with_metric("cpu_usage", 75.5);
    check.with_metric("memory_usage", 60.2);

    assert_eq!(check.metrics.get("cpu_usage"), Some(&75.5));
    assert_eq!(check.metrics.get("memory_usage"), Some(&60.2));
}

#[test]
fn test_health_check_with_component() {
    let mut check = CanonicalHealthCheck::healthy();
    check.with_component("database", CanonicalHealthStatus::Healthy);
    check.with_component("cache", CanonicalHealthStatus::Degraded);

    assert_eq!(check.components.get("database"), Some(&CanonicalHealthStatus::Healthy));
    assert_eq!(check.components.get("cache"), Some(&CanonicalHealthStatus::Degraded));
}

#[test]
fn test_health_check_chained_builder() {
    let mut check = CanonicalHealthCheck::healthy();
    check
        .with_metric("latency_ms", 45.2)
        .with_metric("throughput", 1000.0)
        .with_component("api", CanonicalHealthStatus::Healthy)
        .with_component("storage", CanonicalHealthStatus::Degraded);

    assert_eq!(check.metrics.len(), 2);
    assert_eq!(check.components.len(), 2);
    assert!(check.is_healthy());
}

#[test]
fn test_health_check_is_healthy() {
    let healthy = CanonicalHealthCheck::healthy();
    assert!(healthy.is_healthy());

    let degraded = CanonicalHealthCheck::degraded("Test");
    assert!(!degraded.is_healthy());

    let unhealthy = CanonicalHealthCheck::unhealthy("Test");
    assert!(!unhealthy.is_healthy());

    let unknown = CanonicalHealthCheck::default();
    assert!(!unknown.is_healthy());
}

#[test]
fn test_health_check_clone() {
    let original = CanonicalHealthCheck::healthy();
    let cloned = original.clone();
    assert_eq!(cloned.status, original.status);
    assert_eq!(cloned.message, original.message);
}

#[test]
fn test_health_check_debug() {
    let check = CanonicalHealthCheck::healthy();
    let debug_str = format!("{check:?}");
    assert!(debug_str.contains("Healthy"));
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
        endpoint: "/api/health".to_string(),
        check_interval_seconds: 60,
        timeout_seconds: 10,
    };

    assert!(!config.enabled);
    assert_eq!(config.endpoint, "/api/health");
    assert_eq!(config.check_interval_seconds, 60);
    assert_eq!(config.timeout_seconds, 10);
}

#[test]
fn test_health_config_clone() {
    let config = CanonicalHealthConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.enabled, config.enabled);
    assert_eq!(cloned.endpoint, config.endpoint);
}

#[test]
fn test_health_config_debug() {
    let config = CanonicalHealthConfig::default();
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("enabled"));
}

// ============================================================================
// SERIALIZATION TESTS
// ============================================================================

#[test]
fn test_health_status_serialization() {
    let status = CanonicalHealthStatus::Healthy;
    let json = serde_json::to_string(&status).expect("Failed to serialize");
    assert!(json.contains("Healthy"));

    let deserialized: CanonicalHealthStatus =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized, status);
}

#[test]
fn test_health_check_serialization() {
    let check = CanonicalHealthCheck::healthy();
    let json = serde_json::to_string(&check).expect("Failed to serialize");
    assert!(json.contains("Healthy"));

    let deserialized: CanonicalHealthCheck =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.status, check.status);
}

#[test]
fn test_health_config_serialization() {
    let config = CanonicalHealthConfig::default();
    let json = serde_json::to_string(&config).expect("Failed to serialize");
    assert!(json.contains("health"));

    let deserialized: CanonicalHealthConfig =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.enabled, config.enabled);
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_complex_health_check_scenario() {
    let mut check = CanonicalHealthCheck::degraded("Database latency high");

    check
        .with_metric("db_latency_ms", 250.0)
        .with_metric("cpu_usage", 45.0)
        .with_metric("memory_usage", 78.5)
        .with_component("api", CanonicalHealthStatus::Healthy)
        .with_component("database", CanonicalHealthStatus::Degraded)
        .with_component("cache", CanonicalHealthStatus::Healthy);

    assert_eq!(check.status, CanonicalHealthStatus::Degraded);
    assert_eq!(check.metrics.len(), 3);
    assert_eq!(check.components.len(), 3);
    assert!(!check.is_healthy());
}

#[test]
fn test_health_check_with_multiple_metrics() {
    let mut check = CanonicalHealthCheck::healthy();

    for i in 0..10 {
        check.with_metric(format!("metric_{i}"), f64::from(i) * 10.0);
    }

    assert_eq!(check.metrics.len(), 10);
    assert_eq!(check.metrics.get("metric_5"), Some(&50.0));
}

#[test]
fn test_health_config_disabled_scenario() {
    let config = CanonicalHealthConfig {
        enabled: false,
        ..Default::default()
    };

    assert!(!config.enabled);
    // Even when disabled, other fields should have sensible defaults
    assert!(!config.endpoint.is_empty());
    assert!(config.check_interval_seconds > 0);
}
