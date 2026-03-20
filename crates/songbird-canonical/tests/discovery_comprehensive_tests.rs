// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![expect(
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
    reason = "test assertions and harness ergonomics"
)]
// Allow unwrap/expect in tests - idiomatic for test code
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive tests for canonical service discovery types
//!
//! This test suite provides thorough coverage of the ServiceInfo structure
//! and related discovery patterns.

use songbird_canonical::discovery::ServiceInfo;
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;

// ========== ServiceInfo Creation Tests ==========

#[test]
fn test_service_info_basic_creation() {
    let service = ServiceInfo::new("test-service".to_string(), "192.168.1.100".to_string(), 8080);

    assert_eq!(service.name, "test-service");
    assert_eq!(service.address, "192.168.1.100");
    assert_eq!(service.port, 8080);
    assert!(service.metadata.is_empty());
}

#[test]
fn test_service_info_with_localhost() {
    let service = ServiceInfo::new("local-service".to_string(), "localhost".to_string(), 3000);

    assert_eq!(service.address, "localhost");
    assert_eq!(service.port, 3000);
}

#[test]
fn test_service_info_with_ipv6() {
    let service = ServiceInfo::new("ipv6-service".to_string(), "::1".to_string(), 9000);

    assert_eq!(service.address, "::1");
}

#[test]
fn test_service_info_with_domain() {
    let service = ServiceInfo::new("web-service".to_string(), "example.com".to_string(), 443);

    assert_eq!(service.address, "example.com");
    assert_eq!(service.port, 443);
}

#[test]
fn test_service_info_standard_ports() {
    let http = ServiceInfo::new("http".to_string(), "server".to_string(), 80);
    let https = ServiceInfo::new("https".to_string(), "server".to_string(), 443);
    let ssh = ServiceInfo::new("ssh".to_string(), "server".to_string(), 22);

    assert_eq!(http.port, 80);
    assert_eq!(https.port, 443);
    assert_eq!(ssh.port, 22);
}

#[test]
fn test_service_info_high_port_numbers() {
    let service =
        ServiceInfo::new("high-port-service".to_string(), "192.168.1.1".to_string(), 65535);

    assert_eq!(service.port, 65535);
}

#[test]
fn test_service_info_low_port_numbers() {
    let service = ServiceInfo::new("low-port-service".to_string(), "192.168.1.1".to_string(), 1024);

    assert_eq!(service.port, 1024);
}

// ========== ServiceInfo Metadata Tests ==========

#[test]
fn test_service_info_with_single_metadata() {
    let service = ServiceInfo::new("service".to_string(), "host".to_string(), 8080)
        .with_metadata("version".to_string(), "1.0.0".to_string());

    assert_eq!(service.metadata.len(), 1);
    assert_eq!(service.metadata.get("version"), Some(&"1.0.0".to_string()));
}

#[test]
fn test_service_info_with_multiple_metadata() -> SongbirdResult<()> {
    let service = ServiceInfo::new("service".to_string(), "host".to_string(), 8080)
        .with_metadata("version".to_string(), "1.0.0".to_string())
        .with_metadata("environment".to_string(), "production".to_string())
        .with_metadata("region".to_string(), "us-west-2".to_string());

    assert_eq!(service.metadata.len(), 3);
    assert_eq!(service.metadata.get("version"), Some(&"1.0.0".to_string()));
    assert_eq!(service.metadata.get("environment"), Some(&"production".to_string()));
    assert_eq!(service.metadata.get("region"), Some(&"us-west-2".to_string()));
    Ok(())
}

#[test]
fn test_service_info_metadata_overwrite() -> SongbirdResult<()> {
    let service = ServiceInfo::new("service".to_string(), "host".to_string(), 8080)
        .with_metadata("key".to_string(), "value1".to_string())
        .with_metadata("key".to_string(), "value2".to_string());

    // Second value should overwrite first
    assert_eq!(service.metadata.len(), 1);
    assert_eq!(service.metadata.get("key"), Some(&"value2".to_string()));
    Ok(())
}

#[test]
fn test_service_info_metadata_complex_values() -> SongbirdResult<()> {
    let service = ServiceInfo::new("service".to_string(), "host".to_string(), 8080)
        .with_metadata("json_config".to_string(), r#"{"key": "value"}"#.to_string())
        .with_metadata("list".to_string(), "[1, 2, 3]".to_string())
        .with_metadata("url".to_string(), "https://example.com/path?query=value".to_string());

    assert_eq!(service.metadata.len(), 3);
    assert!(
        service
            .metadata
            .get("json_config")
            .ok_or_else(|| SongbirdError::configuration(
                "Missing performance configuration".to_string()
            ))?
            .contains("key")
    );
    Ok(())
}

#[test]
fn test_service_info_metadata_empty_values() {
    let service = ServiceInfo::new("service".to_string(), "host".to_string(), 8080)
        .with_metadata("empty_key".to_string(), String::new());

    assert_eq!(service.metadata.len(), 1);
    assert_eq!(service.metadata.get("empty_key"), Some(&String::new()));
}

#[test]
fn test_service_info_metadata_special_characters() {
    let service = ServiceInfo::new("service".to_string(), "host".to_string(), 8080)
        .with_metadata("tag:production".to_string(), "enabled".to_string())
        .with_metadata("env.var".to_string(), "value".to_string())
        .with_metadata("key-with-dash".to_string(), "value".to_string());

    assert_eq!(service.metadata.len(), 3);
}

// ========== ServiceInfo Equality Tests ==========

#[test]
fn test_service_info_equality() {
    let service1 = ServiceInfo::new("service".to_string(), "host".to_string(), 8080);
    let service2 = ServiceInfo::new("service".to_string(), "host".to_string(), 8080);

    assert_eq!(service1, service2);
}

#[test]
fn test_service_info_inequality_name() {
    let service1 = ServiceInfo::new("service1".to_string(), "host".to_string(), 8080);
    let service2 = ServiceInfo::new("service2".to_string(), "host".to_string(), 8080);

    assert_ne!(service1, service2);
}

#[test]
fn test_service_info_inequality_address() {
    let service1 = ServiceInfo::new("service".to_string(), "host1".to_string(), 8080);
    let service2 = ServiceInfo::new("service".to_string(), "host2".to_string(), 8080);

    assert_ne!(service1, service2);
}

#[test]
fn test_service_info_inequality_port() {
    let service1 = ServiceInfo::new("service".to_string(), "host".to_string(), 8080);
    let service2 = ServiceInfo::new("service".to_string(), "host".to_string(), 9090);

    assert_ne!(service1, service2);
}

#[test]
fn test_service_info_inequality_metadata() {
    let service1 = ServiceInfo::new("service".to_string(), "host".to_string(), 8080)
        .with_metadata("key".to_string(), "value1".to_string());

    let service2 = ServiceInfo::new("service".to_string(), "host".to_string(), 8080)
        .with_metadata("key".to_string(), "value2".to_string());

    assert_ne!(service1, service2);
}

#[test]
fn test_service_info_equality_with_metadata() {
    let service1 = ServiceInfo::new("service".to_string(), "host".to_string(), 8080)
        .with_metadata("k1".to_string(), "v1".to_string())
        .with_metadata("k2".to_string(), "v2".to_string());

    let service2 = ServiceInfo::new("service".to_string(), "host".to_string(), 8080)
        .with_metadata("k1".to_string(), "v1".to_string())
        .with_metadata("k2".to_string(), "v2".to_string());

    assert_eq!(service1, service2);
}

// ========== ServiceInfo Clone Tests ==========

#[test]
fn test_service_info_clone() {
    let original = ServiceInfo::new("service".to_string(), "host".to_string(), 8080);

    let cloned = original.clone();

    assert_eq!(original, cloned);
    assert_eq!(original.name, cloned.name);
    assert_eq!(original.address, cloned.address);
    assert_eq!(original.port, cloned.port);
}

#[test]
fn test_service_info_clone_with_metadata() {
    let original = ServiceInfo::new("service".to_string(), "host".to_string(), 8080)
        .with_metadata("key1".to_string(), "value1".to_string())
        .with_metadata("key2".to_string(), "value2".to_string());

    let cloned = original.clone();

    assert_eq!(original, cloned);
    assert_eq!(original.metadata.len(), cloned.metadata.len());
    assert_eq!(original.metadata.get("key1"), cloned.metadata.get("key1"));
}

#[test]
fn test_service_info_clone_independence() -> SongbirdResult<()> {
    let mut original = ServiceInfo::new("service".to_string(), "host".to_string(), 8080);

    let cloned = original.clone();

    // Modify original
    original.metadata.insert("new_key".to_string(), "new_value".to_string());

    // Cloned should remain unchanged
    assert!(!cloned.metadata.contains_key("new_key"));
    assert_ne!(original, cloned);
    Ok(())
}

// ========== ServiceInfo Serialization Tests ==========

#[test]
fn test_service_info_serialization() -> SongbirdResult<()> {
    let service = ServiceInfo::new("test-service".to_string(), "192.168.1.1".to_string(), 8080);

    let json_str = serde_json::to_string(&service)?;
    assert!(json_str.contains("test-service"));
    assert!(json_str.contains("192.168.1.1"));
    assert!(json_str.contains("8080"));
    Ok(())
}

#[test]
fn test_service_info_serialization_with_metadata() -> SongbirdResult<()> {
    let service = ServiceInfo::new("service".to_string(), "host".to_string(), 8080)
        .with_metadata("version".to_string(), "1.0.0".to_string());

    let json_str = serde_json::to_string(&service)?;
    assert!(json_str.contains("version"));
    assert!(json_str.contains("1.0.0"));
    Ok(())
}

#[test]
fn test_service_info_deserialization() -> SongbirdResult<()> {
    let json_data = r#"{
        "name": "test-service",
        "address": "192.168.1.1",
        "port": 8080,
        "metadata": {}
    }"#;

    let result: Result<ServiceInfo, _> = serde_json::from_str(json_data);
    assert!(result.is_ok());

    let service = result.map_err(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(service.name, "test-service");
    assert_eq!(service.address, "192.168.1.1");
    assert_eq!(service.port, 8080);
    assert!(service.metadata.is_empty());
    Ok(())
}

#[test]
fn test_service_info_deserialization_with_metadata() -> SongbirdResult<()> {
    let json_data = r#"{
        "name": "service",
        "address": "host",
        "port": 8080,
        "metadata": {
            "version": "1.0.0",
            "environment": "production"
        }
    }"#;

    let result: Result<ServiceInfo, _> = serde_json::from_str(json_data);
    assert!(result.is_ok());

    let service = result.map_err(|_| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    assert_eq!(service.metadata.len(), 2);
    assert_eq!(service.metadata.get("version"), Some(&"1.0.0".to_string()));
    assert_eq!(service.metadata.get("environment"), Some(&"production".to_string()));
    Ok(())
}

#[test]
fn test_service_info_round_trip_serialization() -> SongbirdResult<()> {
    let original = ServiceInfo::new("service".to_string(), "192.168.1.100".to_string(), 9090)
        .with_metadata("key1".to_string(), "value1".to_string())
        .with_metadata("key2".to_string(), "value2".to_string());

    let serialized = serde_json::to_string(&original)
        .map_err(|_e| SongbirdError::configuration("Serialization failed".to_string()))?;
    let deserialized: ServiceInfo =
        serde_json::from_str(&serialized).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Parsing failed: {}", e),
            debug_info: None,
        })?;

    assert_eq!(original, deserialized);
    Ok(())
}

// ========== ServiceInfo Debug Tests ==========

#[test]
fn test_service_info_debug() -> SongbirdResult<()> {
    let service = ServiceInfo::new("debug-service".to_string(), "localhost".to_string(), 8080);

    let debug_str = format!("{:?}", service);
    assert!(debug_str.contains("debug-service"));
    assert!(debug_str.contains("localhost"));
    assert!(debug_str.contains("8080"));
    Ok(())
}

#[test]
fn test_service_info_debug_with_metadata() -> SongbirdResult<()> {
    let service = ServiceInfo::new("service".to_string(), "host".to_string(), 8080)
        .with_metadata("key".to_string(), "value".to_string());

    let debug_str = format!("{:?}", service);
    assert!(debug_str.contains("key"));
    assert!(debug_str.contains("value"));
    Ok(())
}

// ========== Integration & Workflow Tests ==========

#[test]
fn test_service_info_registration_workflow() {
    // Simulate a service registration workflow
    let service = ServiceInfo::new("api-gateway".to_string(), "api.example.com".to_string(), 443)
        .with_metadata("version".to_string(), "2.1.0".to_string())
        .with_metadata("environment".to_string(), "production".to_string())
        .with_metadata("health_check_path".to_string(), "/health".to_string())
        .with_metadata("protocol".to_string(), "https".to_string());

    assert_eq!(service.name, "api-gateway");
    assert_eq!(service.port, 443);
    assert_eq!(service.metadata.len(), 4);

    // Verify metadata is accessible
    assert!(service.metadata.contains_key("version"));
    assert!(service.metadata.contains_key("health_check_path"));
}

#[test]
fn test_service_info_update_scenario() {
    let mut service = ServiceInfo::new("service".to_string(), "old-host".to_string(), 8080);

    // Simulate updating service information
    service.address = "new-host".to_string();
    service.port = 9090;
    service.metadata.insert("updated_at".to_string(), "2025-11-01".to_string());

    assert_eq!(service.address, "new-host");
    assert_eq!(service.port, 9090);
    assert_eq!(service.metadata.get("updated_at"), Some(&"2025-11-01".to_string()));
}

#[test]
fn test_service_info_collection() {
    let services = [
        ServiceInfo::new("service1".to_string(), "host1".to_string(), 8001),
        ServiceInfo::new("service2".to_string(), "host2".to_string(), 8002),
        ServiceInfo::new("service3".to_string(), "host3".to_string(), 8003),
    ];

    assert_eq!(services.len(), 3);
    assert_eq!(services[0].name, "service1");
    assert_eq!(services[1].port, 8002);
    assert_eq!(services[2].address, "host3");
}

#[test]
fn test_service_info_hashmap_storage() {
    let mut service_registry: HashMap<String, ServiceInfo> = HashMap::new();

    let service1 = ServiceInfo::new("service1".to_string(), "host1".to_string(), 8001);
    let service2 = ServiceInfo::new("service2".to_string(), "host2".to_string(), 8002);

    service_registry.insert(service1.name.clone(), service1);
    service_registry.insert(service2.name.clone(), service2);

    assert_eq!(service_registry.len(), 2);
    assert!(service_registry.contains_key("service1"));
    assert!(service_registry.contains_key("service2"));
}

#[test]
fn test_service_info_filtering() {
    let services = [
        ServiceInfo::new("prod-service".to_string(), "prod.host".to_string(), 443),
        ServiceInfo::new("dev-service".to_string(), "dev.host".to_string(), 8080),
        ServiceInfo::new("test-service".to_string(), "test.host".to_string(), 8080),
    ];

    let production_services: Vec<&ServiceInfo> =
        services.iter().filter(|s| s.address.contains("prod")).collect();

    assert_eq!(production_services.len(), 1);
    assert_eq!(production_services[0].name, "prod-service");
}

#[test]
fn test_service_info_sorting() -> SongbirdResult<()> {
    let mut services = [
        ServiceInfo::new("zebra".to_string(), "host".to_string(), 8080),
        ServiceInfo::new("alpha".to_string(), "host".to_string(), 8080),
        ServiceInfo::new("beta".to_string(), "host".to_string(), 8080),
    ];

    services.sort_by(|a, b| a.name.cmp(&b.name));

    assert_eq!(services[0].name, "alpha");
    assert_eq!(services[1].name, "beta");
    assert_eq!(services[2].name, "zebra");
    Ok(())
}

#[test]
fn test_service_info_with_tags_metadata() -> SongbirdResult<()> {
    let service = ServiceInfo::new("tagged-service".to_string(), "host".to_string(), 8080)
        .with_metadata("tags".to_string(), "production,monitored,critical".to_string())
        .with_metadata("owner".to_string(), "platform-team".to_string());

    let tags = service
        .metadata
        .get("tags")
        .ok_or_else(|| SongbirdError::configuration("Missing tags metadata".to_string()))?;
    assert!(tags.contains("production"));
    assert!(tags.contains("monitored"));
    assert!(tags.contains("critical"));
    Ok(())
}

#[test]
fn test_service_info_microservices_pattern() {
    // Simulate a microservices architecture
    let api_gateway =
        ServiceInfo::new("api-gateway".to_string(), "gateway.internal".to_string(), 80)
            .with_metadata("role".to_string(), "gateway".to_string());

    let auth_service =
        ServiceInfo::new("auth-service".to_string(), "auth.internal".to_string(), 8001)
            .with_metadata("role".to_string(), "authentication".to_string())
            .with_metadata("capability".to_string(), "security".to_string());

    let data_service =
        ServiceInfo::new("data-service".to_string(), "data.internal".to_string(), 8002)
            .with_metadata("role".to_string(), "storage".to_string())
            .with_metadata("capability".to_string(), "storage".to_string());

    let services = [api_gateway, auth_service, data_service];

    // Verify services are properly structured
    assert_eq!(services.len(), 3);

    // Find services by capability
    let security_services: Vec<&ServiceInfo> = services
        .iter()
        .filter(|s| s.metadata.get("capability") == Some(&"security".to_string()))
        .collect();

    assert_eq!(security_services.len(), 1);
    assert_eq!(security_services[0].name, "auth-service");
}
