// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for Canonical Service Discovery
//!
//! Comprehensive test coverage for service discovery structures.

use super::discovery::*;
use songbird_types::{SongbirdError, SongbirdResult};

// ============================================================================
// ServiceInfo Tests
// ============================================================================

#[test]
fn test_service_info_new() {
    let service = ServiceInfo::new("test-service".to_string(), "192.168.1.100".to_string(), 8080);

    assert_eq!(service.name, "test-service");
    assert_eq!(service.address, "192.168.1.100");
    assert_eq!(service.port, 8080);
    assert!(service.metadata.is_empty());
}

#[test]
fn test_service_info_with_metadata() {
    let service = ServiceInfo::new("api-service".to_string(), "10.0.0.1".to_string(), 9090)
        .with_metadata("version".to_string(), "1.0.0".to_string())
        .with_metadata("region".to_string(), "us-west-2".to_string());

    assert_eq!(service.metadata.len(), 2);
    assert_eq!(service.metadata.get("version"), Some(&"1.0.0".to_string()));
    assert_eq!(service.metadata.get("region"), Some(&"us-west-2".to_string()));
}

#[test]
fn test_service_info_clone() -> SongbirdResult<()> {
    let service = ServiceInfo::new("clone-test".to_string(), "localhost".to_string(), 3000);
    let cloned = service.clone();

    assert_eq!(service.name, cloned.name);
    assert_eq!(service.address, cloned.address);
    assert_eq!(service.port, cloned.port);
    Ok(())
}

#[test]
fn test_service_info_equality() -> SongbirdResult<()> {
    let service1 = ServiceInfo::new("service".to_string(), "127.0.0.1".to_string(), 8080);
    let service2 = ServiceInfo::new("service".to_string(), "127.0.0.1".to_string(), 8080);
    let service3 = ServiceInfo::new("other".to_string(), "127.0.0.1".to_string(), 8080);

    assert_eq!(service1, service2);
    assert_ne!(service1, service3);
    Ok(())
}

#[test]
fn test_service_info_serialization() -> SongbirdResult<()> {
    let service = ServiceInfo::new("serializable".to_string(), "10.1.1.1".to_string(), 5000)
        .with_metadata("env".to_string(), "production".to_string());

    let json = serde_json::to_string(&service)
        .map_err(|e| SongbirdError::configuration(format!("Serialization failed: {e}")))?;
    let deserialized: ServiceInfo = serde_json::from_str(&json)
        .map_err(|e| SongbirdError::configuration(format!("Deserialization failed: {e}")))?;

    assert_eq!(service, deserialized);
    Ok(())
}

#[test]
fn test_service_info_different_ports() {
    let service_http = ServiceInfo::new("web".to_string(), "example.com".to_string(), 80);
    let service_https = ServiceInfo::new("web".to_string(), "example.com".to_string(), 443);

    assert_ne!(service_http, service_https);
    assert_eq!(service_http.name, service_https.name);
    assert_eq!(service_http.address, service_https.address);
}

#[test]
fn test_service_info_multiple_metadata() -> SongbirdResult<()> {
    let service = ServiceInfo::new("metadata-test".to_string(), "192.168.1.1".to_string(), 8080)
        .with_metadata("key1".to_string(), "value1".to_string())
        .with_metadata("key2".to_string(), "value2".to_string())
        .with_metadata("key3".to_string(), "value3".to_string())
        .with_metadata("key4".to_string(), "value4".to_string());

    assert_eq!(service.metadata.len(), 4);
    Ok(())
}

#[test]
fn test_service_info_metadata_override() -> SongbirdResult<()> {
    let service = ServiceInfo::new("override-test".to_string(), "localhost".to_string(), 3000)
        .with_metadata("key".to_string(), "original".to_string())
        .with_metadata("key".to_string(), "updated".to_string());

    assert_eq!(service.metadata.get("key"), Some(&"updated".to_string()));
    Ok(())
}

#[test]
fn test_service_info_debug_format() -> SongbirdResult<()> {
    let service = ServiceInfo::new("debug-test".to_string(), "127.0.0.1".to_string(), 8080);

    let debug_str = format!("{service:?}");
    assert!(debug_str.contains("debug-test"));
    assert!(debug_str.contains("127.0.0.1"));
    assert!(debug_str.contains("8080"));
    Ok(())
}

#[test]
fn test_service_info_localhost_variations() {
    let ipv4 = ServiceInfo::new("svc".to_string(), "127.0.0.1".to_string(), 8080);
    let ipv6 = ServiceInfo::new("svc".to_string(), "::1".to_string(), 8080);
    let hostname = ServiceInfo::new("svc".to_string(), "localhost".to_string(), 8080);

    assert_ne!(ipv4, ipv6);
    assert_ne!(ipv4, hostname);
    assert_ne!(ipv6, hostname);
}

#[test]
fn test_service_info_high_port_numbers() {
    let service = ServiceInfo::new("high-port".to_string(), "10.0.0.1".to_string(), 65535);

    assert_eq!(service.port, 65535);
}

#[test]
fn test_service_info_empty_metadata_initially() {
    let service = ServiceInfo::new("test".to_string(), "addr".to_string(), 1234);

    assert!(service.metadata.is_empty());
    assert_eq!(service.metadata.len(), 0);
}
