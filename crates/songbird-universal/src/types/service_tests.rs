// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for service types module

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;
use serde::Serialize;
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;

fn assert_json_stable_roundtrip<T>(v: &T)
where
    T: Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
{
    let json = serde_json::to_string(v).expect("serialize");
    let back: T = serde_json::from_str(&json).expect("deserialize");
    let json2 = serde_json::to_string(&back).expect("re-serialize");
    assert_eq!(json, json2, "roundtrip changed JSON representation");
}

fn sample_discovered_capability() -> DiscoveredCapability {
    DiscoveredCapability {
        name: "compute".to_string(),
        version: "1.0".to_string(),
        description: "test cap".to_string(),
        provider: "p1".to_string(),
        endpoint: "http://localhost:8080/cap".to_string(),
        qos_metrics: QosMetrics {
            latency_ms: Some(12.0),
            throughput_ops_sec: Some(100.0),
            availability: Some(0.99),
            reliability: Some(0.99),
        },
        health_status: HealthStatus::Healthy,
    }
}

#[test]
fn test_service_info_creation() {
    let service = ServiceInfo {
        name: "test-service".to_string(),
        primal_type: PrimalType::new("compute"),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec![],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    assert_eq!(service.name, "test-service");
    assert_eq!(service.health, HealthStatus::Healthy);
    assert!(service.capabilities.is_empty());
}

#[test]
fn test_service_event_creation() {
    let mut details = HashMap::new();
    details.insert("key".to_string(), serde_json::json!("value"));

    let event = ServiceEvent {
        service_name: "test-service".to_string(),
        event_type: "started".to_string(),
        timestamp: chrono::Utc::now(),
        details,
    };

    assert_eq!(event.service_name, "test-service");
    assert_eq!(event.event_type, "started");
}

#[test]
fn test_registered_service_creation() {
    let service_info = ServiceInfo {
        name: "registered-service".to_string(),
        primal_type: PrimalType::new("storage"),
        endpoint: "http://localhost:9000".to_string(),
        capabilities: vec![],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let registered = RegisteredService {
        id: "service-123".to_string(),
        service_info,
        registration_time: chrono::Utc::now(),
        last_heartbeat: None,
    };

    assert_eq!(registered.id, "service-123");
    assert!(registered.last_heartbeat.is_none());
}

#[test]
fn test_service_identification_creation() {
    let ident = ServiceIdentification {
        service_id: "svc-001".to_string(),
        service_name: "my-service".to_string(),
        version: "1.0.0".to_string(),
        instance_id: "inst-001".to_string(),
    };

    assert_eq!(ident.service_id, "svc-001");
    assert_eq!(ident.version, "1.0.0");
}

#[test]
fn test_service_endpoint_default() {
    let endpoint = ServiceEndpoint::default();
    assert_eq!(endpoint.protocol, "http");
    assert!(!endpoint.tls_enabled);
    assert_eq!(endpoint.port, songbird_config::defaults::ports::orchestrator_port());
}

#[test]
fn test_service_endpoint_custom() {
    let endpoint = ServiceEndpoint {
        url: "https://example.com".to_string(),
        protocol: "https".to_string(),
        tls_enabled: true,
        port: 443,
    };

    assert_eq!(endpoint.url, "https://example.com");
    assert!(endpoint.tls_enabled);
    assert_eq!(endpoint.port, 443);
}

#[test]
fn test_service_endpoint_equality() {
    let endpoint1 = ServiceEndpoint {
        url: "http://localhost:8080".to_string(),
        protocol: "http".to_string(),
        tls_enabled: false,
        port: 8080,
    };

    let endpoint2 = endpoint1.clone();
    assert_eq!(endpoint1, endpoint2);
}

#[test]
fn test_service_health_creation() {
    let health = ServiceHealth {
        status: HealthStatus::Healthy,
        last_check: chrono::Utc::now(),
        consecutive_successes: 5,
        consecutive_failures: 0,
        recent_qos: None,
    };

    assert_eq!(health.status, HealthStatus::Healthy);
    assert_eq!(health.consecutive_successes, 5);
    assert_eq!(health.consecutive_failures, 0);
}

#[test]
fn test_service_health_with_qos() -> SongbirdResult<()> {
    let qos = QosMetrics {
        latency_ms: Some(25.0),
        throughput_ops_sec: Some(500.0),
        availability: Some(0.999),
        reliability: Some(0.98),
    };

    let health = ServiceHealth {
        status: HealthStatus::Healthy,
        last_check: chrono::Utc::now(),
        consecutive_successes: 3,
        consecutive_failures: 0,
        recent_qos: Some(qos),
    };

    assert!(health.recent_qos.is_some());
    assert_eq!(
        health
            .recent_qos
            .ok_or_else(|| SongbirdError::configuration(
                "recent_qos should be present".to_string()
            ))?
            .latency_ms,
        Some(25.0)
    );
    Ok(())
}

#[test]
fn test_resource_spec_creation() {
    let spec = ResourceSpec {
        cpu_cores: Some(4.0),
        memory_mb: Some(8192),
        disk_mb: Some(100_000),
        network_mbps: Some(1000.0),
    };

    assert_eq!(spec.cpu_cores, Some(4.0));
    assert_eq!(spec.memory_mb, Some(8192));
}

#[test]
fn test_resource_spec_optional_fields() {
    let spec = ResourceSpec {
        cpu_cores: Some(2.0),
        memory_mb: None,
        disk_mb: None,
        network_mbps: Some(100.0),
    };

    assert!(spec.memory_mb.is_none());
    assert!(spec.disk_mb.is_none());
}

#[test]
fn test_service_info_clone() {
    let service = ServiceInfo {
        name: "test".to_string(),
        primal_type: PrimalType::new("ai"),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec![],
        health: HealthStatus::Degraded,
        metadata: HashMap::new(),
    };

    let cloned = service.clone();
    assert_eq!(service.name, cloned.name);
    assert_eq!(service.health, cloned.health);
}

#[test]
fn test_registered_service_with_heartbeat() {
    let now = chrono::Utc::now();
    let service_info = ServiceInfo {
        name: "test".to_string(),
        primal_type: PrimalType::new("compute"),
        endpoint: "http://localhost:8080".to_string(),
        capabilities: vec![],
        health: HealthStatus::Healthy,
        metadata: HashMap::new(),
    };

    let registered = RegisteredService {
        id: "test-id".to_string(),
        service_info,
        registration_time: now,
        last_heartbeat: Some(now),
    };

    assert!(registered.last_heartbeat.is_some());
}

#[test]
fn test_service_info_serialization_roundtrip() {
    let s = ServiceInfo {
        name: "svc-a".to_string(),
        primal_type: PrimalType::new("compute"),
        endpoint: "http://localhost:9000".to_string(),
        capabilities: vec![sample_discovered_capability()],
        health: HealthStatus::Degraded,
        metadata: HashMap::from([("k".to_string(), "v".to_string())]),
    };
    assert_json_stable_roundtrip(&s);
    assert!(format!("{s:?}").contains("ServiceInfo"));
}

#[test]
fn test_service_event_serialization_roundtrip() {
    let e = ServiceEvent {
        service_name: "e-svc".to_string(),
        event_type: "stopped".to_string(),
        timestamp: chrono::DateTime::parse_from_rfc3339("2024-01-02T15:04:05Z")
            .expect("ts")
            .with_timezone(&chrono::Utc),
        details: HashMap::from([("code".to_string(), serde_json::json!(42))]),
    };
    assert_json_stable_roundtrip(&e);
}

#[test]
fn test_registered_service_serialization_roundtrip() {
    let service_info = ServiceInfo {
        name: "reg".to_string(),
        primal_type: PrimalType::new("storage"),
        endpoint: "http://x".to_string(),
        capabilities: vec![],
        health: HealthStatus::Unknown,
        metadata: HashMap::new(),
    };
    let r = RegisteredService {
        id: "id-1".to_string(),
        service_info,
        registration_time: chrono::DateTime::parse_from_rfc3339("2024-06-01T00:00:00Z")
            .expect("ts")
            .with_timezone(&chrono::Utc),
        last_heartbeat: None,
    };
    assert_json_stable_roundtrip(&r);
}

#[test]
fn test_service_identification_serialization_roundtrip() {
    let i = ServiceIdentification {
        service_id: "sid".to_string(),
        service_name: "nice".to_string(),
        version: "2.1.0".to_string(),
        instance_id: "i1".to_string(),
    };
    assert_json_stable_roundtrip(&i);
}

#[test]
fn test_service_endpoint_serialization_roundtrip() {
    let e = ServiceEndpoint {
        url: "https://h.example:8443/x".to_string(),
        protocol: "https".to_string(),
        tls_enabled: true,
        port: 8443,
    };
    assert_json_stable_roundtrip(&e);
}

#[test]
fn test_service_health_serialization_roundtrip() {
    let h = ServiceHealth {
        status: HealthStatus::Unhealthy,
        last_check: chrono::Utc::now(),
        consecutive_successes: 0,
        consecutive_failures: 3,
        recent_qos: Some(QosMetrics {
            latency_ms: Some(500.0),
            throughput_ops_sec: None,
            availability: None,
            reliability: None,
        }),
    };
    assert_json_stable_roundtrip(&h);
}

#[test]
fn test_resource_spec_serialization_roundtrip() {
    let r = ResourceSpec {
        cpu_cores: Some(8.0),
        memory_mb: Some(16_384),
        disk_mb: Some(100),
        network_mbps: Some(10.5),
    };
    assert_json_stable_roundtrip(&r);
}
