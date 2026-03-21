// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for capability system

#![cfg(test)]
#![expect(clippy::all, reason = "test assertions and harness ergonomics")]
#![expect(unused, reason = "test assertions and harness ergonomics")]

use super::*;
use std::collections::HashMap;

#[test]
fn test_adapter_creation() {
    let config = DiscoveryConfig::default();
    let _adapter = UniversalCapabilityAdapter::new(config);

    // Adapter should be created successfully (constructor doesn't panic)
    // Note: Internal fields are private and tested via public API methods
}

#[test]
fn test_discovery_config_default() {
    let config = DiscoveryConfig::default();

    assert_eq!(config.refresh_interval, std::time::Duration::from_secs(300));
    assert_eq!(config.discovery_timeout, std::time::Duration::from_secs(10));
    assert_eq!(config.max_concurrent_discoveries, 10);
    assert!(config.auto_discovery);
    assert!(!config.enable_network_discovery);
}

#[test]
fn test_discovery_config_custom() {
    let config = DiscoveryConfig {
        refresh_interval: std::time::Duration::from_secs(60),
        discovery_timeout: std::time::Duration::from_secs(5),
        max_concurrent_discoveries: 5,
        auto_discovery: false,
        enable_network_discovery: true,
        provider_endpoints: HashMap::new(),
    };

    assert_eq!(config.refresh_interval, std::time::Duration::from_secs(60));
    assert_eq!(config.discovery_timeout, std::time::Duration::from_secs(5));
    assert_eq!(config.max_concurrent_discoveries, 5);
    assert!(!config.auto_discovery);
    assert!(config.enable_network_discovery);
}

#[test]
fn test_capability_registry_default() {
    let registry = CapabilityRegistry::default();

    assert!(registry.primal_capabilities.is_empty());
    assert!(registry.capability_providers.is_empty());
    assert!(registry.last_updated.is_empty());
}

#[test]
fn test_capability_structure() {
    let capability = Capability {
        capability_type: "compute".to_string(),
        name: "container_runtime".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QoSMetrics::default(),
        available: true,
    };

    assert_eq!(capability.capability_type, "compute");
    assert_eq!(capability.name, "container_runtime");
    assert_eq!(capability.version, "1.0.0");
    assert!(capability.available);
}

#[test]
fn test_qos_metrics_default() {
    let metrics = QoSMetrics::default();

    assert!((metrics.latency_ms - 100.0).abs() < f64::EPSILON);
    assert!((metrics.throughput_ops_sec - 1000.0).abs() < f64::EPSILON);
    assert!((metrics.availability - 0.99).abs() < f64::EPSILON);
    assert!((metrics.reliability - 0.99).abs() < f64::EPSILON);
}

#[test]
fn test_resource_metrics_default() {
    let metrics = ResourceMetrics::default();

    assert!((metrics.cpu_percent - 10.0).abs() < f64::EPSILON);
    assert_eq!(metrics.memory_mb, 512);
    assert!((metrics.network_mbps - 10.0).abs() < f64::EPSILON);
    assert_eq!(metrics.storage_mb, 1024);
}

#[test]
fn test_connection_health_variants() {
    assert!(matches!(ConnectionHealth::Healthy, ConnectionHealth::Healthy));
    assert!(matches!(ConnectionHealth::Degraded, ConnectionHealth::Degraded));
    assert!(matches!(ConnectionHealth::Unhealthy, ConnectionHealth::Unhealthy));
    assert!(matches!(ConnectionHealth::Unknown, ConnectionHealth::Unknown));
}

#[test]
fn test_connection_health_equality() {
    assert_eq!(ConnectionHealth::Healthy, ConnectionHealth::Healthy);
    assert_ne!(ConnectionHealth::Healthy, ConnectionHealth::Degraded);
    assert_ne!(ConnectionHealth::Unhealthy, ConnectionHealth::Unknown);
}

#[test]
fn test_primal_type_variants() {
    assert!(matches!(PrimalType::Security, PrimalType::Security));
    assert!(matches!(PrimalType::Compute, PrimalType::Compute));
    assert!(matches!(PrimalType::Storage, PrimalType::Storage));
    assert!(matches!(PrimalType::AI, PrimalType::AI));
    assert!(matches!(PrimalType::Generic, PrimalType::Generic));
}

#[test]
fn test_primal_type_equality() {
    assert_eq!(PrimalType::Security, PrimalType::Security);
    assert_ne!(PrimalType::Compute, PrimalType::Storage);
    assert_ne!(PrimalType::AI, PrimalType::Generic);
}

#[test]
fn test_primal_connection_structure() {
    let connection = PrimalConnection {
        name: "test_primal".to_string(),
        primal_type: PrimalType::Compute,
        endpoint: "http://localhost:8080".to_string(),
        health: ConnectionHealth::Healthy,
        last_contact: chrono::Utc::now(),
        last_health_check: None,
        metadata: HashMap::new(),
    };

    assert_eq!(connection.name, "test_primal");
    assert_eq!(connection.primal_type, PrimalType::Compute);
    assert_eq!(connection.health, ConnectionHealth::Healthy);
}

#[test]
fn test_capability_error_network() {
    let error = CapabilityError::NetworkError("Connection failed".to_string());
    assert_eq!(error.to_string(), "Network error: Connection failed");
}

#[test]
fn test_capability_error_parse() {
    let error = CapabilityError::ParseError("Invalid JSON".to_string());
    assert_eq!(error.to_string(), "Parse error: Invalid JSON");
}

#[test]
fn test_capability_error_primal_not_found() {
    let error = CapabilityError::PrimalNotFound("test".to_string());
    assert_eq!(error.to_string(), "Primal not found: test");
}

#[test]
fn test_capability_error_capability_unavailable() {
    let error = CapabilityError::CapabilityUnavailable("encryption".to_string());
    assert_eq!(error.to_string(), "Capability unavailable: encryption");
}

#[test]
fn test_capability_equality() {
    let cap1 = Capability {
        capability_type: "compute".to_string(),
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QoSMetrics::default(),
        available: true,
    };

    let cap2 = Capability {
        capability_type: "compute".to_string(),
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QoSMetrics::default(),
        available: true,
    };

    assert_eq!(cap1, cap2);
}

#[test]
fn test_capability_inequality() {
    let cap1 = Capability {
        capability_type: "compute".to_string(),
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QoSMetrics::default(),
        available: true,
    };

    let cap2 = Capability {
        capability_type: "storage".to_string(),
        name: "test".to_string(),
        version: "1.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: QoSMetrics::default(),
        available: true,
    };

    assert_ne!(cap1, cap2);
}

#[test]
fn test_qos_metrics_equality() {
    let metrics1 = QoSMetrics::default();
    let metrics2 = QoSMetrics::default();

    assert_eq!(metrics1, metrics2);
}

#[test]
fn test_resource_metrics_equality() {
    let metrics1 = ResourceMetrics::default();
    let metrics2 = ResourceMetrics::default();

    assert_eq!(metrics1, metrics2);
}

#[test]
fn test_capability_with_custom_qos() {
    let qos = QoSMetrics {
        latency_ms: 50.0,
        throughput_ops_sec: 2000.0,
        availability: 0.999,
        reliability: 0.999,
        resource_usage: ResourceMetrics::default(),
    };

    let capability = Capability {
        capability_type: "compute".to_string(),
        name: "high_performance".to_string(),
        version: "2.0.0".to_string(),
        parameters: HashMap::new(),
        qos_metrics: qos.clone(),
        available: true,
    };

    assert_eq!(capability.qos_metrics, qos);
    assert!((capability.qos_metrics.latency_ms - 50.0).abs() < f64::EPSILON);
    assert!((capability.qos_metrics.throughput_ops_sec - 2000.0).abs() < f64::EPSILON);
}

#[test]
fn test_capability_with_parameters() {
    let mut parameters = HashMap::new();
    parameters.insert("max_memory".to_string(), serde_json::json!("4GB"));
    parameters.insert("cpu_cores".to_string(), serde_json::json!(4));

    let capability = Capability {
        capability_type: "compute".to_string(),
        name: "container".to_string(),
        version: "1.0.0".to_string(),
        parameters: parameters.clone(),
        qos_metrics: QoSMetrics::default(),
        available: true,
    };

    assert_eq!(capability.parameters.len(), 2);
    assert!(capability.parameters.contains_key("max_memory"));
    assert!(capability.parameters.contains_key("cpu_cores"));
}

#[tokio::test]
async fn test_get_active_connections_empty() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    let connections = adapter.get_all_connections().await;
    assert!(connections.is_empty());
}

#[tokio::test]
async fn test_find_capability_providers_empty() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    let providers = adapter.find_capability_providers("compute").await;
    assert!(providers.is_empty());
}

#[tokio::test]
async fn test_get_best_primal_for_capability_none() {
    let config = DiscoveryConfig::default();
    let adapter = UniversalCapabilityAdapter::new(config);

    let best = adapter.get_best_primal_for_capability("compute").await;
    assert!(best.is_none());
}
