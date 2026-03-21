// SPDX-License-Identifier: AGPL-3.0-only
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

//! Basic Discovery Tests
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
//! Simple tests for the discovery module to improve test coverage
//! using the actual discovery API.
//!
//! NOTE: This file was reconstructed after extensive corruption.

use chrono::Utc;
use serde_json::Value;
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
fn test_discovery_config_default() {
    let config = CanonicalDiscoveryConfig::default();

    assert!(config.enable_network_scan);
    assert!(config.enable_environment_discovery);
    assert!(!config.enable_container_discovery);
    assert_eq!(config.timeout_seconds, 30);
    assert_eq!(config.health_check_interval, 60);
    assert_eq!(config.backend, "static");
}

/// Test discovery config with custom values
#[test]
fn test_discovery_config_custom() {
    let config = CanonicalDiscoveryConfig::static_config();

    assert!(config.enable_network_scan);
    assert_eq!(config.backend, "static");
}

/// Test consul config creation
#[test]
fn test_discovery_config_consul() {
    let config = CanonicalDiscoveryConfig::consul_config("http://localhost:8500".to_string());

    assert_eq!(config.backend, "service_discovery");
    assert_eq!(config.consul_url, Some("http://localhost:8500".to_string()));
}

/// Test service info creation
#[test]
fn test_service_info_creation() {
    let mut metadata = HashMap::new();
    metadata.insert("region".to_string(), Value::String("us-west-2".to_string()));
    metadata.insert("zone".to_string(), Value::String("a".to_string()));

    let endpoints = vec![ServiceEndpoint {
        path: "/api".to_string(),
        method: "GET".to_string(),
        description: Some("API endpoint".to_string()),
        parameters: vec![],
        response_schema: None,
        auth_required: false,
        rate_limit: None,
    }];

    let now = Utc::now();
    let service_info = ServiceInfo {
        service_id: "test-service-1".to_string(),
        name: "Test Service".to_string(),
        version: "1.0.0".to_string(),
        service_type: "web_service".to_string(),
        description: Some("A test service".to_string()),
        endpoints,
        health_check_endpoint: Some("/health".to_string()),
        metadata: metadata.clone(),
        tags: vec!["test".to_string(), "api".to_string()],
        dependencies: vec![],
        status: ServiceStatus::Running,
        created_at: now,
        updated_at: now,
        instance_id: "instance-1".to_string(),
        host: songbird_config::canonical::constants::network::DEFAULT_HOST.to_string(),
        port: 8080,
    };

    assert_eq!(service_info.service_id, "test-service-1");
    assert_eq!(service_info.name, "Test Service");
    assert_eq!(service_info.version, "1.0.0");
    assert_eq!(service_info.service_type, "web_service");
    assert_eq!(service_info.description, Some("A test service".to_string()));
    assert_eq!(service_info.endpoints.len(), 1);
    assert_eq!(service_info.health_check_endpoint, Some("/health".to_string()));
    assert_eq!(service_info.metadata.len(), 2);
    assert_eq!(service_info.tags.len(), 2);
    assert!(matches!(service_info.status, ServiceStatus::Running));
}

/// Test service endpoint creation
#[test]
fn test_service_endpoint_creation() {
    let endpoint = ServiceEndpoint {
        path: "/v1/api".to_string(),
        method: "POST".to_string(),
        description: Some("API endpoint".to_string()),
        parameters: vec![],
        response_schema: None,
        auth_required: true,
        rate_limit: None,
    };

    assert_eq!(endpoint.path, "/v1/api");
    assert_eq!(endpoint.method, "POST");
    assert!(endpoint.auth_required);
}

/// Test service status variants
#[test]
fn test_service_status_variants() {
    let statuses = [
        ServiceStatus::Starting,
        ServiceStatus::Running,
        ServiceStatus::Stopping,
        ServiceStatus::Stopped,
        ServiceStatus::Error,
        ServiceStatus::Maintenance,
    ];

    assert_eq!(statuses.len(), 6);
}

/// Test health status variants
#[test]
fn test_health_status_variants() {
    let statuses = [HealthStatus::Healthy, HealthStatus::Unhealthy, HealthStatus::Unknown];

    assert_eq!(statuses.len(), 3);
}

/// Test static service discovery creation
#[test]
fn test_static_discovery_creation() {
    let _discovery = StaticServiceDiscovery::new();
    // Just ensure it can be created
}

/// Test service info serialization
#[test]
fn test_service_info_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let mut metadata = HashMap::new();
    metadata.insert("env".to_string(), Value::String("test".to_string()));

    let endpoints = vec![ServiceEndpoint {
        path: "/health".to_string(),
        method: "GET".to_string(),
        description: Some("Health check endpoint".to_string()),
        parameters: vec![],
        response_schema: None,
        auth_required: false,
        rate_limit: None,
    }];

    let now = Utc::now();
    let service_info = ServiceInfo {
        service_id: "serialize-test".to_string(),
        name: "Serialize Test".to_string(),
        version: "0.1.0".to_string(),
        service_type: "other".to_string(),
        description: Some("Test serialization".to_string()),
        endpoints,
        health_check_endpoint: None,
        metadata: metadata.clone(),
        tags: vec!["serialize".to_string()],
        dependencies: vec![],
        status: ServiceStatus::Running,
        created_at: now,
        updated_at: now,
        instance_id: "test-instance".to_string(),
        host: songbird_config::canonical::constants::network::DEFAULT_HOST.to_string(),
        port: 8080,
    };

    // Test serialization
    let serialized = serde_json::to_string(&service_info)
        .map_err(|e| SongbirdError::configuration(format!("Failed to serialize: {e}")))?;
    assert!(!serialized.is_empty());
    assert!(serialized.contains("serialize-test"));
    assert!(serialized.contains("Serialize Test"));
    Ok(())
}

/// Test minimal service creation
#[test]
fn test_minimal_service() {
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

    assert_eq!(minimal_service.description, None);
    assert_eq!(minimal_service.endpoints.len(), 0);
    assert_eq!(minimal_service.tags.len(), 0);
}

/// Test metadata with different value types
#[test]
fn test_metadata_value_types() {
    let mut metadata = HashMap::new();
    metadata.insert("string".to_string(), Value::String("test".to_string()));
    metadata.insert("number".to_string(), Value::Number(serde_json::Number::from(42)));
    metadata.insert("boolean".to_string(), Value::Bool(true));
    metadata.insert("null".to_string(), Value::Null);

    let endpoints = vec![ServiceEndpoint {
        path: "/query".to_string(),
        method: "GET".to_string(),
        description: Some("Database query endpoint".to_string()),
        parameters: vec![],
        response_schema: None,
        auth_required: true,
        rate_limit: None,
    }];

    let now = Utc::now();
    let service_info = ServiceInfo {
        service_id: "metadata-test".to_string(),
        name: "Metadata Test".to_string(),
        version: "1.0.0".to_string(),
        service_type: "database".to_string(),
        description: Some("Testing metadata types".to_string()),
        endpoints,
        health_check_endpoint: Some("/ping".to_string()),
        metadata: metadata.clone(),
        tags: vec!["metadata".to_string(), "test".to_string()],
        dependencies: vec![],
        status: ServiceStatus::Running,
        created_at: now,
        updated_at: now,
        instance_id: "db-instance-1".to_string(),
        host: songbird_config::canonical::constants::network::DEFAULT_HOST.to_string(),
        port: 3000,
    };

    assert_eq!(service_info.metadata.len(), 4);
    assert!(service_info.metadata.contains_key("string"));
    assert!(service_info.metadata.contains_key("number"));
    assert!(service_info.metadata.contains_key("boolean"));
    assert!(service_info.metadata.contains_key("null"));
}
