// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive Service Module Tests
#![expect(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![expect(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![expect(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![expect(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![expect(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![expect(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![expect(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
#![expect(clippy::similar_names, reason = "test assertions and harness ergonomics")]
#![expect(clippy::too_many_lines, reason = "test assertions and harness ergonomics")]
#![expect(clippy::module_name_repetitions, reason = "test assertions and harness ergonomics")]
// Allow unwrap/expect in tests - idiomatic for test code
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    reason = "test assertions and harness ergonomics"
)]

//!
//! Tests for `songbird_types::service` module to achieve full coverage.

use songbird_test_utils::test_orchestrator_port;
use songbird_types::service::*;
use songbird_types::{SongbirdError, SongbirdResult};

// ============================================================================
// SERVICE INFO TESTS
// ============================================================================

#[test]
fn test_service_info_default() {
    let info = CanonicalServiceInfo::default();
    assert_eq!(info.name, "unknown-service");
    assert_eq!(info.version, "0.1.0");
    assert!(info.description.is_none());
    assert!(info.endpoints.is_empty());
    assert!(info.metadata.is_empty());
    assert_eq!(info.health_check_endpoint, Some("/health".to_string()));
    assert!(info.dependencies.is_empty());
    assert!(info.capabilities.is_empty());
    assert!(info.metrics.is_none());
}

#[test]
fn test_service_info_new() {
    let info = CanonicalServiceInfo::new("my-service", "1.0.0");
    assert_eq!(info.name, "my-service");
    assert_eq!(info.version, "1.0.0");
    assert_eq!(info.health_check_endpoint, Some("/health".to_string()));
}

#[test]
fn test_service_info_with_endpoint() {
    let mut info = CanonicalServiceInfo::new("api", "1.0");
    info.with_endpoint("http", format!("http://localhost:{}", test_orchestrator_port()));

    assert_eq!(
        info.endpoints.get("http"),
        Some(&format!("http://localhost:{}", test_orchestrator_port()))
    );
}

#[test]
fn test_service_info_with_multiple_endpoints() {
    let mut info = CanonicalServiceInfo::new("api", "1.0");
    info.with_endpoint("http", format!("http://localhost:{}", test_orchestrator_port()))
        .with_endpoint("grpc", "grpc://localhost:9090");

    assert_eq!(info.endpoints.len(), 2);
    assert!(info.endpoints.contains_key("http"));
    assert!(info.endpoints.contains_key("grpc"));
}

#[test]
fn test_service_info_with_metadata() {
    let mut info = CanonicalServiceInfo::new("service", "1.0");
    info.with_metadata("region", "us-west");

    assert_eq!(info.metadata.get("region"), Some(&"us-west".to_string()));
}

#[test]
fn test_service_info_with_multiple_metadata() {
    let mut info = CanonicalServiceInfo::new("service", "1.0");
    info.with_metadata("region", "us-west")
        .with_metadata("environment", "production")
        .with_metadata("tier", "critical");

    assert_eq!(info.metadata.len(), 3);
}

#[test]
fn test_service_info_with_capability() {
    let mut info = CanonicalServiceInfo::new("service", "1.0");
    info.with_capability("storage");

    assert_eq!(info.capabilities.len(), 1);
    assert_eq!(info.capabilities[0], "storage");
}

#[test]
fn test_service_info_with_multiple_capabilities() {
    let mut info = CanonicalServiceInfo::new("service", "1.0");
    info.with_capability("storage").with_capability("compute").with_capability("ai");

    assert_eq!(info.capabilities.len(), 3);
}

#[test]
fn test_service_info_with_dependency() {
    let mut info = CanonicalServiceInfo::new("service", "1.0");
    info.with_dependency("database");

    assert_eq!(info.dependencies.len(), 1);
    assert_eq!(info.dependencies[0], "database");
}

#[test]
fn test_service_info_with_multiple_dependencies() {
    let mut info = CanonicalServiceInfo::new("service", "1.0");
    info.with_dependency("database").with_dependency("cache").with_dependency("message-queue");

    assert_eq!(info.dependencies.len(), 3);
}

#[test]
fn test_service_info_with_description() {
    let mut info = CanonicalServiceInfo::new("service", "1.0");
    info.with_description("A comprehensive service");

    assert_eq!(info.description, Some("A comprehensive service".to_string()));
}

#[test]
fn test_service_info_builder_chain() {
    let mut info = CanonicalServiceInfo::new("full-service", "2.0.0");
    info.with_endpoint("api", "http://api.example.com")
        .with_metadata("region", "eu-central")
        .with_capability("storage")
        .with_capability("compute")
        .with_dependency("auth-service")
        .with_description("Full-featured service");

    assert_eq!(info.name, "full-service");
    assert_eq!(info.version, "2.0.0");
    assert_eq!(info.endpoints.len(), 1);
    assert_eq!(info.metadata.len(), 1);
    assert_eq!(info.capabilities.len(), 2);
    assert_eq!(info.dependencies.len(), 1);
    assert!(info.description.is_some());
}

#[test]
fn test_service_info_clone() -> SongbirdResult<()> {
    let info1 = CanonicalServiceInfo::new("service", "1.0");
    let info2 = info1.clone();
    assert_eq!(info1.name, info2.name);
    assert_eq!(info1.version, info2.version);
    Ok(())
}

#[test]
fn test_service_info_debug() -> SongbirdResult<()> {
    let info = CanonicalServiceInfo::new("service", "1.0");
    let debug_str = format!("{info:?}");
    assert!(debug_str.contains("CanonicalServiceInfo"));
    Ok(())
}

// ============================================================================
// SERVICE METRICS TESTS
// ============================================================================

#[test]
fn test_service_metrics_default() {
    let metrics = ServiceMetrics::default();
    assert_eq!(metrics.request_count, 0);
    assert_eq!(metrics.error_count, 0);
    assert!((metrics.avg_response_time_ms - 0.0).abs() < f64::EPSILON);
    assert_eq!(metrics.uptime_seconds, 0);
}

#[test]
fn test_service_metrics_creation() -> SongbirdResult<()> {
    let metrics = ServiceMetrics {
        request_count: 1000,
        error_count: 10,
        avg_response_time_ms: 45.5,
        uptime_seconds: 3600,
    };

    assert_eq!(metrics.request_count, 1000);
    assert_eq!(metrics.error_count, 10);
    assert!((metrics.avg_response_time_ms - 45.5).abs() < f64::EPSILON);
    assert_eq!(metrics.uptime_seconds, 3600);
    Ok(())
}

#[test]
fn test_service_metrics_clone() -> SongbirdResult<()> {
    let metrics1 = ServiceMetrics::default();
    let metrics2 = metrics1.clone();
    assert_eq!(metrics1.request_count, metrics2.request_count);
    Ok(())
}

#[test]
fn test_service_metrics_debug() -> SongbirdResult<()> {
    let metrics = ServiceMetrics::default();
    let debug_str = format!("{metrics:?}");
    assert!(debug_str.contains("ServiceMetrics"));
    Ok(())
}

#[test]
fn test_service_info_with_metrics() -> SongbirdResult<()> {
    let mut info = CanonicalServiceInfo::new("service", "1.0");
    info.metrics = Some(ServiceMetrics {
        request_count: 500,
        error_count: 5,
        avg_response_time_ms: 25.0,
        uptime_seconds: 7200,
    });

    assert!(info.metrics.is_some());
    let metrics = info.metrics.as_ref().ok_or_else(|| {
        SongbirdError::configuration("Metrics not found when expected".to_string())
    })?;
    assert_eq!(metrics.request_count, 500);
    Ok(())
}

// ============================================================================
// SERVICE TYPE TESTS
// ============================================================================

#[test]
fn test_service_type_default() {
    let service_type = CanonicalServiceType::default();
    assert!(matches!(service_type, CanonicalServiceType::Custom(_)));
}

#[test]
fn test_service_type_all_variants() {
    let web = CanonicalServiceType::Web;
    let tarpc = CanonicalServiceType::Tarpc;
    let database = CanonicalServiceType::Database;
    let mq = CanonicalServiceType::MessageQueue;
    let cache = CanonicalServiceType::Cache;
    let auth = CanonicalServiceType::Auth;
    let storage = CanonicalServiceType::Storage;
    let compute = CanonicalServiceType::Compute;
    let ai = CanonicalServiceType::AI;
    let monitoring = CanonicalServiceType::Monitoring;
    let custom = CanonicalServiceType::Custom("MyService".to_string());

    assert_eq!(web, CanonicalServiceType::Web);
    assert_eq!(tarpc, CanonicalServiceType::Tarpc);
    assert_eq!(database, CanonicalServiceType::Database);
    assert_eq!(mq, CanonicalServiceType::MessageQueue);
    assert_eq!(cache, CanonicalServiceType::Cache);
    assert_eq!(auth, CanonicalServiceType::Auth);
    assert_eq!(storage, CanonicalServiceType::Storage);
    assert_eq!(compute, CanonicalServiceType::Compute);
    assert_eq!(ai, CanonicalServiceType::AI);
    assert_eq!(monitoring, CanonicalServiceType::Monitoring);
    assert!(matches!(custom, CanonicalServiceType::Custom(_)));
}

#[test]
fn test_service_type_as_str() {
    assert_eq!(CanonicalServiceType::Web.as_str(), "web");
    assert_eq!(CanonicalServiceType::Tarpc.as_str(), "tarpc");
    assert_eq!(CanonicalServiceType::Database.as_str(), "database");
    assert_eq!(CanonicalServiceType::MessageQueue.as_str(), "message_queue");
    assert_eq!(CanonicalServiceType::Cache.as_str(), "cache");
    assert_eq!(CanonicalServiceType::Auth.as_str(), "auth");
    assert_eq!(CanonicalServiceType::Storage.as_str(), "storage");
    assert_eq!(CanonicalServiceType::Compute.as_str(), "compute");
    assert_eq!(CanonicalServiceType::AI.as_str(), "ai");
    assert_eq!(CanonicalServiceType::Monitoring.as_str(), "monitoring");
    assert_eq!(CanonicalServiceType::Custom("test".to_string()).as_str(), "test");
}

#[test]
fn test_service_type_equality() -> SongbirdResult<()> {
    assert_eq!(CanonicalServiceType::Web, CanonicalServiceType::Web);
    assert_ne!(CanonicalServiceType::Web, CanonicalServiceType::Tarpc);
    Ok(())
}

#[test]
fn test_service_type_clone() -> SongbirdResult<()> {
    let type1 = CanonicalServiceType::Storage;
    let type2 = type1.clone();
    assert_eq!(type1, type2);
    Ok(())
}

#[test]
fn test_service_type_debug() -> SongbirdResult<()> {
    let service_type = CanonicalServiceType::Web;
    let debug_str = format!("{service_type:?}");
    assert!(debug_str.contains("Web"));
    Ok(())
}

#[test]
fn test_service_type_hash() -> SongbirdResult<()> {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(CanonicalServiceType::Web);
    set.insert(CanonicalServiceType::Tarpc);
    set.insert(CanonicalServiceType::Web); // Duplicate

    assert_eq!(set.len(), 2); // Should only have 2 unique types
    Ok(())
}

// ============================================================================
// SERIALIZATION TESTS
// ============================================================================

#[test]
fn test_service_info_serialization() -> SongbirdResult<()> {
    let info = CanonicalServiceInfo::new("test-service", "1.0.0");
    let json = serde_json::to_string(&info)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: CanonicalServiceInfo = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Failed to deserialize: {}", e)))?;

    assert_eq!(deserialized.name, info.name);
    assert_eq!(deserialized.version, info.version);
    Ok(())
}

#[test]
fn test_service_metrics_serialization() -> SongbirdResult<()> {
    let metrics = ServiceMetrics::default();
    let json = serde_json::to_string(&metrics)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: ServiceMetrics = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Failed to deserialize: {}", e)))?;

    assert_eq!(deserialized.request_count, metrics.request_count);
    Ok(())
}

#[test]
fn test_service_type_serialization() -> SongbirdResult<()> {
    let service_type = CanonicalServiceType::Web;
    let json = serde_json::to_string(&service_type)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: CanonicalServiceType = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Failed to deserialize: {}", e)))?;

    assert_eq!(deserialized, service_type);
    Ok(())
}

#[test]
fn test_service_type_custom_serialization() -> SongbirdResult<()> {
    let service_type = CanonicalServiceType::Custom("MyCustomService".to_string());
    let json = serde_json::to_string(&service_type)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {}", e)))?;
    let deserialized: CanonicalServiceType = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Failed to deserialize: {}", e)))?;

    if let CanonicalServiceType::Custom(name) = deserialized {
        assert_eq!(name, "MyCustomService");
    } else {
        panic!("Expected Custom variant");
    }
    Ok(())
}

// ============================================================================
// SERVICE STATUS TESTS
// ============================================================================

#[test]
fn test_service_status_default() {
    let status = CanonicalServiceStatus::default();
    assert_eq!(status, CanonicalServiceStatus::Unknown);
}

#[test]
fn test_service_status_all_variants() -> SongbirdResult<()> {
    let running = CanonicalServiceStatus::Running;
    let starting = CanonicalServiceStatus::Starting;
    let stopping = CanonicalServiceStatus::Stopping;
    let stopped = CanonicalServiceStatus::Stopped;
    let error = CanonicalServiceStatus::Error;
    let unknown = CanonicalServiceStatus::Unknown;

    assert_eq!(running, CanonicalServiceStatus::Running);
    assert_eq!(starting, CanonicalServiceStatus::Starting);
    assert_eq!(stopping, CanonicalServiceStatus::Stopping);
    assert_eq!(stopped, CanonicalServiceStatus::Stopped);
    assert_eq!(error, CanonicalServiceStatus::Error);
    assert_eq!(unknown, CanonicalServiceStatus::Unknown);
    Ok(())
}

#[test]
fn test_service_status_equality() -> SongbirdResult<()> {
    assert_eq!(CanonicalServiceStatus::Running, CanonicalServiceStatus::Running);
    assert_ne!(CanonicalServiceStatus::Running, CanonicalServiceStatus::Stopped);
    Ok(())
}

#[test]
fn test_service_status_debug() -> SongbirdResult<()> {
    let status = CanonicalServiceStatus::Running;
    let debug_str = format!("{status:?}");
    assert!(debug_str.contains("Running"));
    Ok(())
}

#[test]
fn test_service_status_clone() {
    let status1 = CanonicalServiceStatus::Running;
    let status2 = status1;
    assert_eq!(status1, status2);
}

// ============================================================================
// ALLOWED VALUES TESTS
// ============================================================================

#[test]
fn test_allowed_values_default() {
    let allowed = AllowedValues::default();
    assert!(matches!(allowed, AllowedValues::Any));
}

#[test]
fn test_allowed_values_any() {
    let allowed = AllowedValues::Any;
    assert!(matches!(allowed, AllowedValues::Any));
}

#[test]
fn test_allowed_values_specific() {
    let allowed = AllowedValues::Specific(vec!["value1".to_string(), "value2".to_string()]);
    if let AllowedValues::Specific(values) = allowed {
        assert_eq!(values.len(), 2);
    } else {
        panic!("Expected Specific variant");
    }
}

#[test]
fn test_allowed_values_range() {
    let allowed = AllowedValues::Range {
        min: 0.0,
        max: 100.0,
    };
    if let AllowedValues::Range {
        min,
        max,
    } = allowed
    {
        assert!((min - 0.0).abs() < f64::EPSILON);
        assert!((max - 100.0).abs() < f64::EPSILON);
    } else {
        panic!("Expected Range variant");
    }
}

#[test]
fn test_allowed_values_pattern() -> SongbirdResult<()> {
    let allowed = AllowedValues::Pattern("[0-9]+".to_string());
    if let AllowedValues::Pattern(pattern) = allowed {
        assert_eq!(pattern, "[0-9]+");
    } else {
        panic!("Expected Pattern variant");
    }
    Ok(())
}

#[test]
fn test_allowed_values_clone() -> SongbirdResult<()> {
    let allowed1 = AllowedValues::Any;
    let allowed2 = allowed1;
    assert!(matches!(allowed2, AllowedValues::Any));
    Ok(())
}

#[test]
fn test_allowed_values_debug() -> SongbirdResult<()> {
    let allowed = AllowedValues::Any;
    let debug_str = format!("{allowed:?}");
    assert!(debug_str.contains("Any"));
    Ok(())
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_complete_service_registration() {
    let mut info = CanonicalServiceInfo::new("payment-service", "3.2.1");
    info.with_endpoint("api", "https://payment.example.com/api")
        .with_endpoint("admin", "https://payment.example.com/admin")
        .with_metadata("region", "us-east-1")
        .with_metadata("cluster", "prod-cluster-1")
        .with_capability("payment-processing")
        .with_capability("refunds")
        .with_capability("subscriptions")
        .with_dependency("auth-service")
        .with_dependency("notification-service")
        .with_description("Handles all payment operations");

    info.metrics = Some(ServiceMetrics {
        request_count: 10000,
        error_count: 50,
        avg_response_time_ms: 120.5,
        uptime_seconds: 86400,
    });

    // Verify complete setup
    assert_eq!(info.name, "payment-service");
    assert_eq!(info.version, "3.2.1");
    assert_eq!(info.endpoints.len(), 2);
    assert_eq!(info.metadata.len(), 2);
    assert_eq!(info.capabilities.len(), 3);
    assert_eq!(info.dependencies.len(), 2);
    assert!(info.description.is_some());
    assert!(info.metrics.is_some());
}

#[test]
fn test_service_lifecycle_status_transitions() {
    let mut status = CanonicalServiceStatus::default();
    assert_eq!(status, CanonicalServiceStatus::Unknown);

    status = CanonicalServiceStatus::Starting;
    assert_eq!(status, CanonicalServiceStatus::Starting);

    status = CanonicalServiceStatus::Running;
    assert_eq!(status, CanonicalServiceStatus::Running);

    status = CanonicalServiceStatus::Stopping;
    assert_eq!(status, CanonicalServiceStatus::Stopping);

    status = CanonicalServiceStatus::Stopped;
    assert_eq!(status, CanonicalServiceStatus::Stopped);
}

#[test]
fn test_service_types_in_registry() {
    let services = [
        (CanonicalServiceType::Web, "web-api"),
        (CanonicalServiceType::Database, "postgres"),
        (CanonicalServiceType::Cache, "redis"),
        (CanonicalServiceType::Storage, "s3"),
        (CanonicalServiceType::AI, "ml-service"),
    ];

    assert_eq!(services.len(), 5);
    assert!(services.iter().any(|(t, _)| t == &CanonicalServiceType::Web));
    assert!(services.iter().any(|(t, _)| t == &CanonicalServiceType::AI));
}
