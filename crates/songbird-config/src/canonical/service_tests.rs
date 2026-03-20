// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Comprehensive tests for canonical service configuration
//!
//! Phase 3 Test Coverage Expansion - Week 1
//! Target: 0% → 100% coverage for service.rs (25 lines)

use super::*;

// =============================================================================
// SERVICE CONFIG TESTS
// =============================================================================

#[test]
fn test_service_config_default() {
    let config = ServiceConfig::default();

    assert_eq!(config.name, "default-service");
    assert_eq!(config.address, "localhost");
    assert_eq!(config.port, 8080);
    assert!(config.metadata.is_empty());
    assert!(config.health_check.is_some());
}

#[test]
fn test_service_config_custom() {
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "1.0.0".to_string());
    metadata.insert("environment".to_string(), "production".to_string());

    let config = ServiceConfig {
        name: "custom-service".to_string(),
        address: "0.0.0.0".to_string(),
        port: 9000,
        metadata,
        health_check: None,
    };

    assert_eq!(config.name, "custom-service");
    assert_eq!(config.address, "0.0.0.0");
    assert_eq!(config.port, 9000);
    assert_eq!(config.metadata.len(), 2);
    assert!(config.health_check.is_none());
}

#[test]
fn test_service_config_clone() {
    let config = ServiceConfig::default();
    let cloned = config.clone();

    assert_eq!(config.name, cloned.name);
    assert_eq!(config.address, cloned.address);
    assert_eq!(config.port, cloned.port);
}

#[test]
fn test_service_config_debug() {
    let config = ServiceConfig::default();
    let debug_str = format!("{config:?}");

    assert!(debug_str.contains("ServiceConfig"));
    assert!(debug_str.contains("default-service"));
}

#[test]
fn test_service_config_serialization() {
    let config = ServiceConfig::default();

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: ServiceConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.name, deserialized.name);
    assert_eq!(config.port, deserialized.port);
}

// =============================================================================
// SERVICE INFO TESTS
// =============================================================================

#[test]
fn test_service_info_default() {
    let info = ServiceInfo::default();

    assert_eq!(info.service_id, "default-service");
    assert_eq!(info.name, "default-service");
    assert_eq!(info.host, "localhost");
    assert_eq!(info.port, 8080);
    assert!(info.metadata.is_empty());
}

#[test]
fn test_service_info_custom() {
    let mut metadata = HashMap::new();
    metadata.insert("region".to_string(), "us-west-2".to_string());

    let info = ServiceInfo {
        service_id: "srv-123".to_string(),
        name: "api-service".to_string(),
        host: "api.example.com".to_string(),
        port: 443,
        metadata,
    };

    assert_eq!(info.service_id, "srv-123");
    assert_eq!(info.name, "api-service");
    assert_eq!(info.host, "api.example.com");
    assert_eq!(info.port, 443);
    assert_eq!(info.metadata.len(), 1);
}

#[test]
fn test_service_info_equality() {
    let info1 = ServiceInfo::default();
    let info2 = ServiceInfo::default();

    assert_eq!(info1, info2);
}

#[test]
fn test_service_info_clone() {
    let info = ServiceInfo::default();
    let cloned = info.clone();

    assert_eq!(info, cloned);
}

#[test]
fn test_service_info_serialization() {
    let info = ServiceInfo::default();

    let json = serde_json::to_string(&info).expect("Should serialize");
    let deserialized: ServiceInfo = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(info, deserialized);
}

// =============================================================================
// HEALTH CHECK CONFIG TESTS
// =============================================================================

#[test]
fn test_health_check_config_default() {
    let config = HealthCheckConfig::default();

    assert_eq!(config.endpoint, "/health");
    assert_eq!(config.interval, 30);
    assert_eq!(config.timeout, 5);
}

#[test]
fn test_health_check_config_custom() {
    let config = HealthCheckConfig {
        endpoint: "/api/health".to_string(),
        interval: 60,
        timeout: 10,
    };

    assert_eq!(config.endpoint, "/api/health");
    assert_eq!(config.interval, 60);
    assert_eq!(config.timeout, 10);
}

#[test]
fn test_health_check_config_timeout_less_than_interval() {
    let config = HealthCheckConfig {
        endpoint: "/health".to_string(),
        interval: 30,
        timeout: 5,
    };

    assert!(config.timeout < config.interval, "Timeout should be less than interval");
}

#[test]
fn test_health_check_config_clone() {
    let config = HealthCheckConfig::default();
    let cloned = config.clone();

    assert_eq!(config.endpoint, cloned.endpoint);
    assert_eq!(config.interval, cloned.interval);
    assert_eq!(config.timeout, cloned.timeout);
}

#[test]
fn test_health_check_config_serialization() {
    let config = HealthCheckConfig::default();

    let json = serde_json::to_string(&config).expect("Should serialize");
    let deserialized: HealthCheckConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(config.endpoint, deserialized.endpoint);
    assert_eq!(config.interval, deserialized.interval);
}

// =============================================================================
// TYPE ALIAS TESTS
// =============================================================================

#[test]
fn test_canonical_service_info_alias() {
    let info: CanonicalServiceInfo = ServiceInfo::default();
    assert_eq!(info.service_id, "default-service");
}

#[test]
fn test_universal_service_info_alias() {
    let info: UniversalServiceInfo = ServiceInfo::default();
    assert_eq!(info.name, "default-service");
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[test]
fn test_service_config_with_health_check() {
    let health_check = HealthCheckConfig {
        endpoint: "/api/health".to_string(),
        interval: 15,
        timeout: 3,
    };

    let config = ServiceConfig {
        name: "monitored-service".to_string(),
        address: "0.0.0.0".to_string(),
        port: 8080,
        metadata: HashMap::new(),
        health_check: Some(health_check),
    };

    let hc = config.health_check.as_ref().unwrap();
    assert_eq!(hc.endpoint, "/api/health");
    assert_eq!(hc.interval, 15);
    assert_eq!(hc.timeout, 3);
}

#[test]
fn test_service_config_without_health_check() {
    let config = ServiceConfig {
        name: "unmonitored-service".to_string(),
        address: "localhost".to_string(),
        port: 8080,
        metadata: HashMap::new(),
        health_check: None,
    };

    assert!(config.health_check.is_none());
}

#[test]
fn test_service_info_with_rich_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "2.0.0".to_string());
    metadata.insert("environment".to_string(), "staging".to_string());
    metadata.insert("region".to_string(), "eu-west-1".to_string());
    metadata.insert("datacenter".to_string(), "dc1".to_string());

    let info = ServiceInfo {
        service_id: "srv-456".to_string(),
        name: "data-service".to_string(),
        host: "data.example.com".to_string(),
        port: 5432,
        metadata,
    };

    assert_eq!(info.metadata.len(), 4);
    assert_eq!(info.metadata.get("version"), Some(&"2.0.0".to_string()));
    assert_eq!(info.metadata.get("environment"), Some(&"staging".to_string()));
}

#[test]
fn test_service_config_round_trip_serialization() {
    let mut metadata = HashMap::new();
    metadata.insert("key".to_string(), "value".to_string());

    let original = ServiceConfig {
        name: "test-service".to_string(),
        address: "127.0.0.1".to_string(),
        port: 3000,
        metadata,
        health_check: Some(HealthCheckConfig {
            endpoint: "/status".to_string(),
            interval: 20,
            timeout: 4,
        }),
    };

    let json = serde_json::to_string(&original).expect("Should serialize");
    let deserialized: ServiceConfig = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(original.name, deserialized.name);
    assert_eq!(original.address, deserialized.address);
    assert_eq!(original.port, deserialized.port);
    assert_eq!(original.metadata.len(), deserialized.metadata.len());
    assert_eq!(
        original.health_check.as_ref().unwrap().endpoint,
        deserialized.health_check.as_ref().unwrap().endpoint
    );
}
