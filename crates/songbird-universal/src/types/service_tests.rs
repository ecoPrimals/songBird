// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for service types module

use super::*;
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;

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
