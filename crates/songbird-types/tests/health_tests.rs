//! Comprehensive tests for health check system
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

use songbird_types::health::*;

#[test]
fn test_health_status_default() {
    let status = CanonicalHealthStatus::default();
    assert_eq!(status, CanonicalHealthStatus::Unknown);
}

#[test]
fn test_health_status_display() {
    assert_eq!(CanonicalHealthStatus::Healthy.to_string(), "Healthy");
    assert_eq!(CanonicalHealthStatus::Degraded.to_string(), "Degraded");
    assert_eq!(CanonicalHealthStatus::Unhealthy.to_string(), "Unhealthy");
    assert_eq!(CanonicalHealthStatus::Unknown.to_string(), "Unknown");
}

#[test]
fn test_health_status_equality() {
    assert_eq!(CanonicalHealthStatus::Healthy, CanonicalHealthStatus::Healthy);
    assert_ne!(CanonicalHealthStatus::Healthy, CanonicalHealthStatus::Degraded);
    assert_ne!(CanonicalHealthStatus::Degraded, CanonicalHealthStatus::Unhealthy);
    assert_ne!(CanonicalHealthStatus::Unhealthy, CanonicalHealthStatus::Unknown);
}

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
    let check = CanonicalHealthCheck::degraded("Slow response times");
    assert_eq!(check.status, CanonicalHealthStatus::Degraded);
    assert_eq!(check.message, Some("Slow response times".to_string()));
}

#[test]
fn test_health_check_unhealthy() {
    let check = CanonicalHealthCheck::unhealthy("Database connection failed");
    assert_eq!(check.status, CanonicalHealthStatus::Unhealthy);
    assert_eq!(check.message, Some("Database connection failed".to_string()));
}

#[test]
fn test_health_check_unknown() {
    let check = CanonicalHealthCheck::default();
    assert_eq!(check.status, CanonicalHealthStatus::Unknown);
    assert!(check.message.is_none());
}

#[test]
fn test_health_check_with_metrics() {
    let mut check = CanonicalHealthCheck::healthy();
    check.metrics.insert("cpu_usage".to_string(), 45.5);
    check.metrics.insert("memory_usage".to_string(), 78.2);

    assert_eq!(check.metrics.len(), 2);
    assert_eq!(check.metrics.get("cpu_usage"), Some(&45.5));
    assert_eq!(check.metrics.get("memory_usage"), Some(&78.2));
}

#[test]
fn test_health_check_with_components() {
    let mut check = CanonicalHealthCheck::healthy();
    check.components.insert("database".to_string(), CanonicalHealthStatus::Healthy);
    check.components.insert("cache".to_string(), CanonicalHealthStatus::Degraded);

    assert_eq!(check.components.len(), 2);
    assert_eq!(check.components.get("database"), Some(&CanonicalHealthStatus::Healthy));
    assert_eq!(check.components.get("cache"), Some(&CanonicalHealthStatus::Degraded));
}

#[test]
fn test_health_check_with_custom_message() {
    let mut check = CanonicalHealthCheck::healthy();
    check.message = Some("System running smoothly".to_string());

    assert_eq!(check.message, Some("System running smoothly".to_string()));
}

#[test]
fn test_health_check_manual_construction() {
    let mut check = CanonicalHealthCheck::healthy();
    check.metrics.insert("latency_ms".to_string(), 12.5);
    check.components.insert("api".to_string(), CanonicalHealthStatus::Healthy);
    check.message = Some("All services operational".to_string());

    assert_eq!(check.metrics.len(), 1);
    assert_eq!(check.components.len(), 1);
    assert_eq!(check.message, Some("All services operational".to_string()));
}

#[test]
fn test_health_check_serialization() {
    let check = CanonicalHealthCheck::healthy();
    let json = serde_json::to_string(&check).expect("Should serialize");
    assert!(json.contains("Healthy"));

    let deserialized: CanonicalHealthCheck =
        serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized.status, CanonicalHealthStatus::Healthy);
}

#[test]
fn test_health_status_serialization() {
    let status = CanonicalHealthStatus::Healthy;
    let json = serde_json::to_string(&status).expect("Should serialize");
    let deserialized: CanonicalHealthStatus =
        serde_json::from_str(&json).expect("Should deserialize");
    assert_eq!(deserialized, CanonicalHealthStatus::Healthy);
}

#[test]
fn test_health_check_complex_scenario() {
    let mut check = CanonicalHealthCheck::degraded("High load detected");
    check.metrics.insert("cpu_usage".to_string(), 85.5);
    check.metrics.insert("memory_usage".to_string(), 92.3);
    check.metrics.insert("disk_usage".to_string(), 67.8);
    check.components.insert("web_server".to_string(), CanonicalHealthStatus::Healthy);
    check.components.insert("database".to_string(), CanonicalHealthStatus::Degraded);
    check.components.insert("cache".to_string(), CanonicalHealthStatus::Healthy);

    assert_eq!(check.status, CanonicalHealthStatus::Degraded);
    assert_eq!(check.metrics.len(), 3);
    assert_eq!(check.components.len(), 3);
    assert_eq!(check.message, Some("High load detected".to_string()));
}

#[test]
fn test_health_check_metric_overwrite() {
    let mut check = CanonicalHealthCheck::healthy();
    check.metrics.insert("cpu".to_string(), 50.0);
    check.metrics.insert("cpu".to_string(), 75.0); // Overwrite

    assert_eq!(check.metrics.get("cpu"), Some(&75.0));
    assert_eq!(check.metrics.len(), 1);
}

#[test]
fn test_health_check_component_overwrite() {
    let mut check = CanonicalHealthCheck::healthy();
    check.components.insert("service".to_string(), CanonicalHealthStatus::Healthy);
    check.components.insert("service".to_string(), CanonicalHealthStatus::Degraded); // Overwrite

    assert_eq!(check.components.get("service"), Some(&CanonicalHealthStatus::Degraded));
    assert_eq!(check.components.len(), 1);
}

#[test]
fn test_health_status_all_variants() {
    let variants = [
        CanonicalHealthStatus::Healthy,
        CanonicalHealthStatus::Degraded,
        CanonicalHealthStatus::Unhealthy,
        CanonicalHealthStatus::Unknown,
    ];

    // Ensure all variants are unique
    for (i, v1) in variants.iter().enumerate() {
        for (j, v2) in variants.iter().enumerate() {
            if i == j {
                assert_eq!(v1, v2);
            } else {
                assert_ne!(v1, v2);
            }
        }
    }
}

#[test]
fn test_health_check_empty_message() {
    let mut check = CanonicalHealthCheck::healthy();
    check.message = Some(String::new());
    assert_eq!(check.message, Some(String::new()));
}

#[test]
fn test_health_check_multiple_metrics() {
    let mut check = CanonicalHealthCheck::healthy();
    for i in 0..10 {
        check.metrics.insert(format!("metric_{i}"), f64::from(i));
    }
    assert_eq!(check.metrics.len(), 10);
}

#[test]
fn test_health_check_multiple_components() {
    let mut check = CanonicalHealthCheck::healthy();
    check.components.insert("component_1".to_string(), CanonicalHealthStatus::Healthy);
    check.components.insert("component_2".to_string(), CanonicalHealthStatus::Degraded);
    check.components.insert("component_3".to_string(), CanonicalHealthStatus::Unhealthy);
    check.components.insert("component_4".to_string(), CanonicalHealthStatus::Unknown);

    assert_eq!(check.components.len(), 4);
}

#[test]
fn test_health_check_clone() {
    let check = CanonicalHealthCheck::healthy();
    let cloned = check.clone();

    assert_eq!(check.status, cloned.status);
    assert_eq!(check.message, cloned.message);
}

#[test]
fn test_health_status_copy() {
    let status = CanonicalHealthStatus::Healthy;
    let copied = status;

    assert_eq!(status, copied);
}

#[test]
fn test_health_check_debug_format() {
    let check = CanonicalHealthCheck::healthy();
    let debug_str = format!("{check:?}");
    assert!(debug_str.contains("CanonicalHealthCheck"));
}

#[test]
fn test_health_status_debug_format() {
    let status = CanonicalHealthStatus::Healthy;
    let debug_str = format!("{status:?}");
    assert!(debug_str.contains("Healthy"));
}
