//! Comprehensive tests for Canonical Service Configuration
//!
//! This test suite provides thorough coverage of service configuration
//! types and utilities.

use songbird_config::canonical::service::*;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;

// ========== ServiceConfig Tests ==========

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
    metadata.insert("region".to_string(), "us-west-2".to_string());

    let config = ServiceConfig {
        name: "my-service".to_string(),
        address: "192.168.1.1".to_string(),
        port: 9000,
        metadata,
        health_check: None,
    };

    assert_eq!(config.name, "my-service");
    assert_eq!(config.port, 9000);
    assert_eq!(config.metadata.len(), 1);
    assert!(config.health_check.is_none());
}

#[test]
fn test_service_config_with_health_check() -> SongbirdResult<()> {
    let health_check = HealthCheckConfig {
        endpoint: "/api/health".to_string(),
        interval: 60,
        timeout: 10,
    };

    let config = ServiceConfig {
        health_check: Some(health_check),
        ..ServiceConfig::default()
    };

    assert!(config.health_check.is_some());
    let hc = config.health_check.ok_or_else(|| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;
    assert_eq!(hc.endpoint, "/api/health");
    assert_eq!(hc.interval, 60);
    Ok(())
}

#[test]
fn test_service_config_clone() -> SongbirdResult<()> {
    let original = ServiceConfig::default();
    let cloned = original.clone();

    assert_eq!(original.name, cloned.name);
    assert_eq!(original.port, cloned.port);
    Ok(())
}

#[test]
fn test_service_config_debug() -> SongbirdResult<()> {
    let config = ServiceConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("ServiceConfig"));
    Ok(())
}

#[test]
fn test_service_config_serialization() -> SongbirdResult<()> {
    let config = ServiceConfig::default();
    let json = serde_json::to_string(&config);
    assert!(json.is_ok());
    Ok(())
}

#[test]
fn test_service_config_deserialization() -> SongbirdResult<()> {
    let json = r#"{
        "name": "test-service",
        "address": "127.0.0.1",
        "port": 3000,
        "metadata": {},
        "health_check": null
    }"#;

    let result: Result<ServiceConfig, _> = serde_json::from_str(json);
    assert!(result.is_ok());

    let config = result.ok_or_else(|| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;
    assert_eq!(config.name, "test-service");
    assert_eq!(config.port, 3000);
    Ok(())
}

// ========== ServiceInfo Tests ==========

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
    metadata.insert("version".to_string(), "1.0.0".to_string());

    let info = ServiceInfo {
        service_id: "svc-123".to_string(),
        name: "api-service".to_string(),
        host: "api.example.com".to_string(),
        port: 443,
        metadata,
    };

    assert_eq!(info.service_id, "svc-123");
    assert_eq!(info.name, "api-service");
    assert_eq!(info.port, 443);
    assert_eq!(info.metadata.len(), 1);
}

#[test]
fn test_service_info_clone() {
    let original = ServiceInfo::default();
    let cloned = original.clone();

    assert_eq!(original.service_id, cloned.service_id);
    assert_eq!(original.host, cloned.host);
}

#[test]
fn test_service_info_equality() -> SongbirdResult<()> {
    let info1 = ServiceInfo::default();
    let info2 = ServiceInfo::default();

    assert_eq!(info1, info2);
    Ok(())
}

#[test]
fn test_service_info_inequality() -> SongbirdResult<()> {
    let info1 = ServiceInfo::default();
    let info2 = ServiceInfo {
        port: 9000,
        ..ServiceInfo::default()
    };

    assert_ne!(info1, info2);
    Ok(())
}

#[test]
fn test_service_info_debug() -> SongbirdResult<()> {
    let info = ServiceInfo::default();
    let debug_str = format!("{:?}", info);
    assert!(debug_str.contains("ServiceInfo"));
    Ok(())
}

#[test]
fn test_service_info_serialization() {
    let info = ServiceInfo::default();
    let json = serde_json::to_string(&info);
    assert!(json.is_ok());
}

#[test]
fn test_service_info_deserialization() {
    let json = r#"{
        "service_id": "123",
        "name": "my-service",
        "host": "localhost",
        "port": 8080,
        "metadata": {}
    }"#;

    let result: Result<ServiceInfo, _> = serde_json::from_str(json);
    assert!(result.is_ok());
}

// ========== HealthCheckConfig Tests ==========

#[test]
fn test_health_check_config_default() {
    let config = HealthCheckConfig::default();

    assert_eq!(config.endpoint, "/health");
    assert_eq!(config.interval, 30);
    assert_eq!(config.timeout, 5);
}

#[test]
fn test_health_check_config_custom() -> SongbirdResult<()> {
    let config = HealthCheckConfig {
        endpoint: "/api/v1/health".to_string(),
        interval: 60,
        timeout: 10,
    };

    assert_eq!(config.endpoint, "/api/v1/health");
    assert_eq!(config.interval, 60);
    assert_eq!(config.timeout, 10);
    Ok(())
}

#[test]
fn test_health_check_config_clone() -> SongbirdResult<()> {
    let original = HealthCheckConfig::default();
    let cloned = original.clone();

    assert_eq!(original.endpoint, cloned.endpoint);
    assert_eq!(original.interval, cloned.interval);
    Ok(())
}

#[test]
fn test_health_check_config_debug() -> SongbirdResult<()> {
    let config = HealthCheckConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("HealthCheckConfig"));
    Ok(())
}

#[test]
fn test_health_check_config_serialization() {
    let config = HealthCheckConfig::default();
    let json = serde_json::to_string(&config);
    assert!(json.is_ok());
}

#[test]
fn test_health_check_config_deserialization() {
    let json = r#"{
        "endpoint": "/health",
        "interval": 30,
        "timeout": 5
    }"#;

    let result: Result<HealthCheckConfig, _> = serde_json::from_str(json);
    assert!(result.is_ok());
}

// ========== Integration Tests ==========

#[test]
fn test_service_config_with_complex_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "2.1.0".to_string());
    metadata.insert("region".to_string(), "eu-central-1".to_string());
    metadata.insert("environment".to_string(), "production".to_string());
    metadata.insert("team".to_string(), "platform".to_string());

    let config = ServiceConfig {
        name: "prod-api".to_string(),
        address: "10.0.0.100".to_string(),
        port: 8443,
        metadata,
        health_check: Some(HealthCheckConfig::default()),
    };

    assert_eq!(config.metadata.len(), 4);
    assert_eq!(config.metadata.get("version"), Some(&"2.1.0".to_string()));
    assert_eq!(config.metadata.get("environment"), Some(&"production".to_string()));
}

#[test]
fn test_service_info_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("protocol".to_string(), "https".to_string());
    metadata.insert("tls".to_string(), "enabled".to_string());

    let info = ServiceInfo {
        service_id: "secure-svc".to_string(),
        name: "secure-service".to_string(),
        host: "secure.example.com".to_string(),
        port: 443,
        metadata,
    };

    assert_eq!(info.metadata.len(), 2);
    assert_eq!(info.metadata.get("protocol"), Some(&"https".to_string()));
}

#[test]
fn test_health_check_with_short_interval() {
    let config = HealthCheckConfig {
        endpoint: "/ping".to_string(),
        interval: 5,
        timeout: 1,
    };

    assert_eq!(config.interval, 5);
    assert_eq!(config.timeout, 1);
}

#[test]
fn test_health_check_with_long_interval() -> SongbirdResult<()> {
    let config = HealthCheckConfig {
        endpoint: "/health/detailed".to_string(),
        interval: 300,
        timeout: 30,
    };

    assert_eq!(config.interval, 300);
    assert_eq!(config.timeout, 30);
    Ok(())
}

#[test]
fn test_service_config_round_trip_serialization() -> SongbirdResult<()> {
    let original = ServiceConfig {
        name: "test".to_string(),
        address: "192.168.1.1".to_string(),
        port: 9000,
        metadata: HashMap::new(),
        health_check: Some(HealthCheckConfig::default()),
    };

    let json = serde_json::to_string(&original)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {}", e)))?;
    let deserialized: ServiceConfig =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Parsing failed: {}", e),
            debug_info: None,
        })?;

    assert_eq!(original.name, deserialized.name);
    assert_eq!(original.port, deserialized.port);
    Ok(())
}

#[test]
fn test_service_info_round_trip_serialization() -> SongbirdResult<()> {
    let original = ServiceInfo {
        service_id: "123".to_string(),
        name: "test".to_string(),
        host: "localhost".to_string(),
        port: 8080,
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&original).map_err(|e| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;
    let deserialized: ServiceInfo =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Parsing failed: {}", e),
            debug_info: None,
        })?;

    assert_eq!(original, deserialized);
    Ok(())
}

#[test]
fn test_service_config_with_ipv4() {
    let config = ServiceConfig {
        address: "127.0.0.1".to_string(),
        ..ServiceConfig::default()
    };

    assert_eq!(config.address, "127.0.0.1");
}

#[test]
fn test_service_config_with_ipv6() {
    let config = ServiceConfig {
        address: "::1".to_string(),
        ..ServiceConfig::default()
    };

    assert_eq!(config.address, "::1");
}

#[test]
fn test_service_config_with_domain() {
    let config = ServiceConfig {
        address: "api.example.com".to_string(),
        ..ServiceConfig::default()
    };

    assert_eq!(config.address, "api.example.com");
}

#[test]
fn test_service_info_with_standard_ports() {
    let web_http = ServiceInfo {
        port: 80,
        ..ServiceInfo::default()
    };

    let web_https = ServiceInfo {
        port: 443,
        ..ServiceInfo::default()
    };

    assert_eq!(web_http.port, 80);
    assert_eq!(web_https.port, 443);
}

#[test]
fn test_service_info_with_high_port() -> SongbirdResult<()> {
    let info = ServiceInfo {
        port: 65000,
        ..ServiceInfo::default()
    };

    assert_eq!(info.port, 65000);
    Ok(())
}

#[test]
fn test_health_check_with_custom_path() -> SongbirdResult<()> {
    let config = HealthCheckConfig {
        endpoint: "/api/v2/status/health".to_string(),
        ..HealthCheckConfig::default()
    };

    assert_eq!(config.endpoint, "/api/v2/status/health");
    Ok(())
}

#[test]
fn test_service_config_json_format() -> SongbirdResult<()> {
    let config = ServiceConfig::default();
    let json = serde_json::to_string_pretty(&config).map_err(|e| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    assert!(json.contains("name"));
    assert!(json.contains("address"));
    assert!(json.contains("port"));
    Ok(())
}

#[test]
fn test_service_info_json_format() -> SongbirdResult<()> {
    let info = ServiceInfo::default();
    let json = serde_json::to_string_pretty(&info).map_err(|e| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;

    assert!(json.contains("service_id"));
    assert!(json.contains("name"));
    assert!(json.contains("host"));
    Ok(())
}

#[test]
fn test_service_config_with_empty_metadata() {
    let config = ServiceConfig {
        metadata: HashMap::new(),
        ..ServiceConfig::default()
    };

    assert!(config.metadata.is_empty());
}

#[test]
fn test_service_info_with_empty_metadata() {
    let info = ServiceInfo {
        metadata: HashMap::new(),
        ..ServiceInfo::default()
    };

    assert!(info.metadata.is_empty());
}

#[test]
fn test_service_config_with_many_metadata_entries() {
    let mut metadata = HashMap::new();
    for i in 0..100 {
        metadata.insert(format!("key-{}", i), format!("value-{}", i));
    }

    let config = ServiceConfig {
        metadata,
        ..ServiceConfig::default()
    };

    assert_eq!(config.metadata.len(), 100);
}

#[test]
fn test_health_check_zero_timeout() {
    let config = HealthCheckConfig {
        timeout: 0,
        ..HealthCheckConfig::default()
    };

    assert_eq!(config.timeout, 0);
}

#[test]
fn test_health_check_large_values() {
    let config = HealthCheckConfig {
        interval: u64::MAX,
        timeout: u64::MAX,
        ..HealthCheckConfig::default()
    };

    assert_eq!(config.interval, u64::MAX);
    assert_eq!(config.timeout, u64::MAX);
}
