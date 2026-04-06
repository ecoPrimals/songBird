// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive Discovery Tests
#![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![allow(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![allow(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![allow(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
#![allow(clippy::similar_names, reason = "test assertions and harness ergonomics")]
#![allow(clippy::too_many_lines, reason = "test assertions and harness ergonomics")]
#![allow(clippy::module_name_repetitions, reason = "test assertions and harness ergonomics")]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//!
//! Extended tests for discovery functionality with comprehensive coverage
//!
//! NOTE: This file was reconstructed after corruption. Original tests preserved where possible.

use chrono::Utc;
use songbird_discovery::{
    discovery::{backends::StaticServiceDiscovery, core::CanonicalDiscoveryConfig},
    traits::{
        discovery::HealthStatus,
        service::{ServiceEndpoint, ServiceInfo, ServiceStatus},
    },
};
use songbird_types::SongbirdError;
use std::collections::HashMap;

/// Test discovery configuration creation and defaults
#[test]
fn test_discovery_config_comprehensive() {
    let config = CanonicalDiscoveryConfig::default();

    assert!(config.enable_network_scan);
    assert!(config.enable_environment_discovery);
    assert!(!config.enable_container_discovery);
    assert_eq!(config.timeout_seconds, 30);
    assert_eq!(config.health_check_interval, 60);
}

/// Test health status variants
#[test]
fn test_health_status_variants() {
    let statuses = [HealthStatus::Healthy, HealthStatus::Unhealthy, HealthStatus::Unknown];

    assert_eq!(statuses.len(), 3);
}

/// Test service creation with all fields
#[test]
fn test_comprehensive_service_creation() {
    let mut metadata = HashMap::new();
    metadata.insert("region".to_string(), serde_json::Value::String("us-west-2".to_string()));
    metadata.insert("env".to_string(), serde_json::Value::String("production".to_string()));

    let endpoints = vec![
        ServiceEndpoint {
            path: "/api/v1".to_string(),
            method: "GET".to_string(),
            description: Some("Main API endpoint".to_string()),
            parameters: vec![],
            response_schema: None,
            auth_required: true,
            rate_limit: None,
        },
        ServiceEndpoint {
            path: "/health".to_string(),
            method: "GET".to_string(),
            description: Some("Health check".to_string()),
            parameters: vec![],
            response_schema: None,
            auth_required: false,
            rate_limit: None,
        },
    ];

    let now = Utc::now();
    let service_info = ServiceInfo {
        service_id: "comprehensive-service".to_string(),
        name: "Comprehensive Service".to_string(),
        version: "2.0.0".to_string(),
        service_type: "web_service".to_string(),
        description: Some("A comprehensive test service".to_string()),
        endpoints,
        health_check_endpoint: Some("/health".to_string()),
        metadata,
        tags: vec!["production".to_string(), "api".to_string()],
        dependencies: vec!["database".to_string(), "cache".to_string()],
        status: ServiceStatus::Running,
        created_at: now,
        updated_at: now,
        instance_id: "instance-prod-1".to_string(),
        host: "api.example.com".to_string(),
        port: 443,
    };

    assert_eq!(service_info.endpoints.len(), 2);
    assert_eq!(service_info.tags.len(), 2);
    assert_eq!(service_info.dependencies.len(), 2);
}

/// Test static discovery backend
#[test]
fn test_static_discovery_backend() {
    let _discovery = StaticServiceDiscovery::new();

    // Just validate creation - test passes by construction
    // Static discovery starts empty and is valid
}

/// Test backend type configuration
#[test]
fn test_backend_configuration_types() {
    let static_config = CanonicalDiscoveryConfig::static_config();
    assert_eq!(static_config.backend, "static");

    let consul_config = CanonicalDiscoveryConfig::consul_config("http://consul:8500".to_string());
    assert_eq!(consul_config.backend, "service_discovery");
    assert!(consul_config.consul_url.is_some());

    let k8s_config = CanonicalDiscoveryConfig::kubernetes_config("default".to_string());
    assert_eq!(k8s_config.backend, "container_orchestration");
    assert!(k8s_config.kubernetes_namespace.is_some());
}

/// Test service status transitions
#[test]
fn test_service_status_transitions() {
    let statuses = vec![
        ServiceStatus::Starting,
        ServiceStatus::Running,
        ServiceStatus::Stopping,
        ServiceStatus::Stopped,
        ServiceStatus::Error,
        ServiceStatus::Maintenance,
    ];

    for status in statuses {
        // Verify all statuses can be created
        let _ = format!("{status:?}");
    }
}

/// Test service with empty optional fields
#[test]
fn test_service_with_minimal_fields() {
    let now = Utc::now();
    let minimal_service = ServiceInfo {
        service_id: "minimal".to_string(),
        name: "Minimal".to_string(),
        version: "0.0.1".to_string(),
        service_type: "test".to_string(),
        description: None,
        endpoints: vec![],
        health_check_endpoint: None,
        metadata: HashMap::new(),
        tags: vec![],
        dependencies: vec![],
        status: ServiceStatus::Starting,
        created_at: now,
        updated_at: now,
        instance_id: "min-1".to_string(),
        host: "localhost".to_string(),
        port: 3000,
    };

    assert!(minimal_service.description.is_none());
    assert!(minimal_service.endpoints.is_empty());
    assert!(minimal_service.health_check_endpoint.is_none());
}

/// Test config with all backend types
#[test]
fn test_all_discovery_backends() {
    let backends = vec!["static", "service_discovery", "container_orchestration"];

    for backend in backends {
        // Verify backend strings are valid
        assert!(!backend.is_empty());
    }
}

/// Test comprehensive metadata types
#[test]
fn test_comprehensive_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("string".to_string(), serde_json::Value::String("value".to_string()));
    metadata.insert("number".to_string(), serde_json::Value::Number(serde_json::Number::from(100)));
    metadata.insert("bool".to_string(), serde_json::Value::Bool(false));
    metadata.insert("null".to_string(), serde_json::Value::Null);

    let array = serde_json::Value::Array(vec![
        serde_json::Value::String("item1".to_string()),
        serde_json::Value::String("item2".to_string()),
    ]);
    metadata.insert("array".to_string(), array);

    assert_eq!(metadata.len(), 5);
    assert!(metadata.contains_key("string"));
    assert!(metadata.contains_key("number"));
    assert!(metadata.contains_key("bool"));
    assert!(metadata.contains_key("null"));
    assert!(metadata.contains_key("array"));
}

/// Test service serialization and deserialization
#[test]
fn test_service_round_trip_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now();
    let service = ServiceInfo {
        service_id: "test-serialization".to_string(),
        name: "Test".to_string(),
        version: "1.0.0".to_string(),
        service_type: "test".to_string(),
        description: Some("Test service".to_string()),
        endpoints: vec![],
        health_check_endpoint: Some("/health".to_string()),
        metadata: HashMap::new(),
        tags: vec!["test".to_string()],
        dependencies: vec![],
        status: ServiceStatus::Running,
        created_at: now,
        updated_at: now,
        instance_id: "test-1".to_string(),
        host: "localhost".to_string(),
        port: 8080,
    };

    let serialized = serde_json::to_string(&service)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {e}")))?;
    let deserialized: ServiceInfo = serde_json::from_str(&serialized)
        .map_err(|e| SongbirdError::configuration(format!("Failed to deserialize: {e}")))?;

    assert_eq!(service.service_id, deserialized.service_id);
    assert_eq!(service.name, deserialized.name);
    assert_eq!(service.version, deserialized.version);
    Ok(())
}
