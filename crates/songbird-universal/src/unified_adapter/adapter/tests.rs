// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]
#![allow(clippy::expect_used, reason = "test assertions")]

use super::*;
use crate::capabilities::{Capability, QoSMetrics, ResourceMetrics};
use crate::types::{DiscoveredCapability, PrimalType, QosMetrics};
use crate::types::{HealthStatus, ServiceInfo};
use crate::unified_adapter::{
    CapabilityRegistry, RegistryStats, ServiceConnection, UnifiedAdapterConfig,
    UniversalAdapterError,
};
use songbird_types::{SongbirdError, SongbirdResult};
use std::collections::HashMap;
use std::sync::Arc;

fn create_test_qos_metrics() -> QosMetrics {
    QosMetrics {
        latency_ms: Some(10.0),
        throughput_ops_sec: Some(1000.0),
        availability: Some(0.99),
        reliability: Some(0.99),
    }
}

fn create_test_qos_metrics_capability() -> QoSMetrics {
    QoSMetrics {
        latency_ms: 10.0,
        throughput_ops_sec: 1000.0,
        availability: 0.99,
        reliability: 0.99,
        resource_usage: ResourceMetrics {
            cpu_percent: 50.0,
            memory_mb: 512,
            network_mbps: 100.0,
            storage_mb: 1024,
        },
    }
}

fn create_test_discovered_capability(
    name: &str,
    endpoint: &str,
    provider: &str,
) -> DiscoveredCapability {
    DiscoveredCapability {
        name: name.to_string(),
        version: "1.0".to_string(),
        description: format!("{name} capability"),
        provider: provider.to_string(),
        endpoint: endpoint.to_string(),
        qos_metrics: create_test_qos_metrics(),
        health_status: HealthStatus::Healthy,
    }
}

#[test]
fn test_unified_adapter_creation() {
    let adapter = UnifiedUniversalAdapter::new();
    assert_eq!(adapter.config.discovery_endpoints.len(), 2);
    assert!(adapter.config.auto_discovery);
}

#[test]
fn test_capability_registry_default() {
    let registry = CapabilityRegistry::default();
    assert!(registry.service_capabilities.is_empty());
    assert!(registry.capability_providers.is_empty());
    assert!(registry.service_info.is_empty());
    assert!(registry.last_updated.is_empty());
}

#[test]
fn test_unified_adapter_config_default() {
    let config = UnifiedAdapterConfig::default();
    assert_eq!(config.discovery_endpoints.len(), 2);
    assert!(config.auto_discovery);
    assert_eq!(config.discovery_timeout, std::time::Duration::from_secs(30));
    assert_eq!(config.health_check_interval, std::time::Duration::from_secs(60));
    assert_eq!(config.max_concurrent_requests, 100);
}

#[tokio::test]
async fn test_discover_services_empty_endpoints() -> SongbirdResult<()> {
    let adapter = UnifiedUniversalAdapter::new();
    let services = adapter.discover_services().await.map_err(|e| {
        SongbirdError::configuration(format!(
            "Failed to discover services from empty registry: {}",
            e
        ))
    })?;
    assert!(services.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_find_capability_providers_empty_registry() -> SongbirdResult<()> {
    let adapter = UnifiedUniversalAdapter::new();
    let providers = adapter.find_capability_providers("compute").await.map_err(|e| {
        SongbirdError::configuration(format!(
            "Failed to find capability providers from empty registry: {}",
            e
        ))
    })?;
    assert!(providers.is_empty());
    Ok(())
}

#[test]
fn test_universal_adapter_error_display() {
    let err = UniversalAdapterError::MissingCapability;
    assert_eq!(err.to_string(), "Missing required capability");

    let err = UniversalAdapterError::NoProvidersAvailable("compute".to_string());
    assert_eq!(err.to_string(), "No providers available for capability: compute");

    let err = UniversalAdapterError::NetworkError("timeout".to_string());
    assert_eq!(err.to_string(), "Network error: timeout");

    let err = UniversalAdapterError::ParseError("invalid json".to_string());
    assert_eq!(err.to_string(), "Parse error: invalid json");

    let err = UniversalAdapterError::DiscoveryError("failed".to_string());
    assert_eq!(err.to_string(), "Discovery error: failed");

    let err = UniversalAdapterError::ServiceError("500".to_string());
    assert_eq!(err.to_string(), "Service error: 500");
}

#[test]
fn test_registry_stats_creation() {
    let stats = RegistryStats {
        total_services: 5,
        total_capabilities: 10,
        healthy_services: 4,
    };

    assert_eq!(stats.total_services, 5);
    assert_eq!(stats.total_capabilities, 10);
    assert_eq!(stats.healthy_services, 4);
}

#[tokio::test]
async fn test_concurrent_registry_access() {
    let adapter = Arc::new(UnifiedUniversalAdapter::new());
    let adapter1 = Arc::clone(&adapter);
    let adapter2 = Arc::clone(&adapter);

    let task1 = tokio::spawn(async move {
        let _ = adapter1.find_capability_providers("compute").await;
    });

    let task2 = tokio::spawn(async move {
        let _ = adapter2.find_capability_providers("storage").await;
    });

    let _ = tokio::join!(task1, task2);
}

#[test]
fn test_health_status_equality() {
    assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
}

#[tokio::test]
async fn test_route_request_missing_capability_type() {
    let adapter = UnifiedUniversalAdapter::new();
    let request = crate::types::UniversalRequest {
        request_id: "test-1".to_string(),
        source: "test-source".to_string(),
        target: "test-target".to_string(),
        action: "test".to_string(),
        parameters: HashMap::new(),
        security_context: None,
    };

    let result = adapter.route_request(request).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), UniversalAdapterError::MissingCapability));
}

#[tokio::test]
async fn test_route_request_no_providers() {
    let adapter = UnifiedUniversalAdapter::new();
    let mut parameters = HashMap::new();
    parameters.insert(
        "capability_type".to_string(),
        serde_json::Value::String("nonexistent".to_string()),
    );
    let request = crate::types::UniversalRequest {
        request_id: "test-2".to_string(),
        source: "test-source".to_string(),
        target: "test-target".to_string(),
        action: "test".to_string(),
        parameters,
        security_context: None,
    };

    let result = adapter.route_request(request).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        UniversalAdapterError::NoProvidersAvailable(cap) => {
            assert_eq!(cap, "nonexistent");
        }
        _ => panic!("Expected NoProvidersAvailable error"),
    }
}

#[tokio::test]
async fn test_get_registry_stats_empty() {
    let adapter = UnifiedUniversalAdapter::new();
    let stats = adapter.get_registry_stats().await;

    assert_eq!(stats.total_services, 0);
    assert_eq!(stats.total_capabilities, 0);
    assert_eq!(stats.healthy_services, 0);
}

#[tokio::test]
async fn test_get_registry_stats_with_services() {
    let adapter = UnifiedUniversalAdapter::new();

    {
        let mut registry = adapter.capability_registry.write().await;

        let service1 = ServiceInfo {
            name: "service1".to_string(),
            primal_type: PrimalType::new("compute"),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec![create_test_discovered_capability(
                "compute",
                "http://localhost:8080",
                "service1",
            )],
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };

        let service2 = ServiceInfo {
            name: "service2".to_string(),
            primal_type: PrimalType::new("storage"),
            endpoint: "http://localhost:8081".to_string(),
            capabilities: vec![create_test_discovered_capability(
                "storage",
                "http://localhost:8081",
                "service2",
            )],
            health: HealthStatus::Degraded,
            metadata: HashMap::new(),
        };

        registry.service_info.insert("service1".to_string(), service1);
        registry.service_info.insert("service2".to_string(), service2);
        registry.capability_providers.insert("compute".to_string(), vec!["service1".to_string()]);
        registry.capability_providers.insert("storage".to_string(), vec!["service2".to_string()]);
    }

    let stats = adapter.get_registry_stats().await;
    assert_eq!(stats.total_services, 2);
    assert_eq!(stats.total_capabilities, 2);
    assert_eq!(stats.healthy_services, 1);
}

#[tokio::test]
async fn test_find_capability_providers_with_data() -> SongbirdResult<()> {
    let adapter = UnifiedUniversalAdapter::new();

    {
        let mut registry = adapter.capability_registry.write().await;

        let service = ServiceInfo {
            name: "compute-service".to_string(),
            primal_type: PrimalType::new("compute"),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec![create_test_discovered_capability(
                "compute",
                "http://localhost:8080",
                "compute-service",
            )],
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };

        registry.service_info.insert("compute-service".to_string(), service);
        registry
            .capability_providers
            .insert("compute".to_string(), vec!["compute-service".to_string()]);
    }

    let providers = adapter.find_capability_providers("compute").await.map_err(|e| {
        SongbirdError::configuration(format!(
            "Failed to find capability providers with test data: {}",
            e
        ))
    })?;
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].name, "compute-service");
    Ok(())
}

#[tokio::test]
async fn test_find_capability_providers_multiple() -> SongbirdResult<()> {
    let adapter = UnifiedUniversalAdapter::new();

    {
        let mut registry = adapter.capability_registry.write().await;

        let service1 = ServiceInfo {
            name: "compute-1".to_string(),
            primal_type: PrimalType::new("compute"),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec![create_test_discovered_capability(
                "compute",
                "http://localhost:8080",
                "compute-1",
            )],
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };

        let service2 = ServiceInfo {
            name: "compute-2".to_string(),
            primal_type: PrimalType::new("compute"),
            endpoint: "http://localhost:8081".to_string(),
            capabilities: vec![create_test_discovered_capability(
                "compute",
                "http://localhost:8081",
                "compute-2",
            )],
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };

        registry.service_info.insert("compute-1".to_string(), service1);
        registry.service_info.insert("compute-2".to_string(), service2);
        registry
            .capability_providers
            .insert("compute".to_string(), vec!["compute-1".to_string(), "compute-2".to_string()]);
    }

    let providers = adapter.find_capability_providers("compute").await.map_err(|e| {
        SongbirdError::configuration(format!("Failed to find multiple capability providers: {}", e))
    })?;
    assert_eq!(providers.len(), 2);
    Ok(())
}

#[test]
fn test_adapter_with_custom_config() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: std::time::Duration::from_secs(10),
        health_check_interval: std::time::Duration::from_secs(30),
        max_concurrent_requests: 50,
        auto_discovery: false,
        discovery_endpoints: vec!["http://custom:9000".to_string()],
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);
    assert_eq!(adapter.config.discovery_timeout, std::time::Duration::from_secs(10));
    assert_eq!(adapter.config.max_concurrent_requests, 50);
    assert!(!adapter.config.auto_discovery);
    assert_eq!(adapter.config.discovery_endpoints.len(), 1);
}

#[test]
fn test_service_connection_creation() {
    let connection = ServiceConnection {
        endpoint: "http://localhost:8080".to_string(),
        health: HealthStatus::Healthy,
        metrics: std::collections::HashMap::new(),
        last_contact: chrono::Utc::now(),
    };

    assert_eq!(connection.endpoint, "http://localhost:8080");
    assert_eq!(connection.health, HealthStatus::Healthy);
    assert!(connection.metrics.is_empty());
}

#[test]
fn test_registry_stats_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let stats = RegistryStats {
        total_services: 10,
        total_capabilities: 20,
        healthy_services: 8,
    };

    let json = serde_json::to_string(&stats).map_err(|e| SongbirdError::Serialization {
        format: Some("JSON".to_string()),
        message: format!("Serialization failed: {}", e),
        debug_info: None,
    })?;
    assert!(json.contains("total_services"));
    assert!(json.contains("\"10\"") || json.contains("10"));

    let deserialized: RegistryStats =
        serde_json::from_str(&json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("Parsing failed: {}", e),
            debug_info: None,
        })?;
    assert_eq!(deserialized.total_services, 10);
    assert_eq!(deserialized.total_capabilities, 20);
    assert_eq!(deserialized.healthy_services, 8);
    Ok(())
}

#[tokio::test]
async fn test_capability_registry_indexing() -> SongbirdResult<()> {
    let adapter = UnifiedUniversalAdapter::new();

    {
        let mut registry = adapter.capability_registry.write().await;

        let capabilities = vec![
            create_test_discovered_capability(
                "compute",
                "http://localhost:8080",
                "multi-cap-service",
            ),
            create_test_discovered_capability(
                "storage",
                "http://localhost:8080",
                "multi-cap-service",
            ),
        ];

        let service = ServiceInfo {
            name: "multi-cap-service".to_string(),
            primal_type: PrimalType::new("generic"),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: capabilities.clone(),
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };

        registry.service_info.insert(service.name.clone(), service.clone());
        let simple_caps: Vec<Capability> = capabilities
            .iter()
            .map(|dc| Capability {
                capability_type: dc.name.clone(),
                name: dc.name.clone(),
                version: dc.version.clone(),
                parameters: HashMap::new(),
                qos_metrics: create_test_qos_metrics_capability(),
                available: true,
            })
            .collect();
        registry.service_capabilities.insert(service.name.clone(), simple_caps);

        for capability in &service.capabilities {
            registry
                .capability_providers
                .entry(capability.name.clone())
                .or_insert_with(Vec::new)
                .push(service.name.clone());
        }
    }

    let compute_providers = adapter.find_capability_providers("compute").await.map_err(|e| {
        SongbirdError::configuration(format!(
            "Failed to find compute providers in capability registry: {}",
            e
        ))
    })?;
    let storage_providers = adapter.find_capability_providers("storage").await.map_err(|e| {
        SongbirdError::configuration(format!(
            "Failed to find storage providers in capability registry: {}",
            e
        ))
    })?;

    assert_eq!(compute_providers.len(), 1);
    assert_eq!(storage_providers.len(), 1);
    assert_eq!(compute_providers[0].name, "multi-cap-service");
    assert_eq!(storage_providers[0].name, "multi-cap-service");
    Ok(())
}

#[test]
fn test_error_types_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<UniversalAdapterError>();
}

#[tokio::test]
async fn test_adapter_is_clonable() {
    let adapter1 = UnifiedUniversalAdapter::new();
    let adapter2 = adapter1.clone();

    let stats1 = adapter1.get_registry_stats().await;
    let stats2 = adapter2.get_registry_stats().await;

    assert_eq!(stats1.total_services, stats2.total_services);
}

#[tokio::test]
async fn test_concurrent_write_operations() {
    let adapter = Arc::new(UnifiedUniversalAdapter::new());
    let adapter1 = Arc::clone(&adapter);
    let adapter2 = Arc::clone(&adapter);

    let task1 = tokio::spawn(async move {
        let mut registry = adapter1.capability_registry.write().await;
        let service = ServiceInfo {
            name: "service1".to_string(),
            primal_type: PrimalType::new("generic"),
            endpoint: "http://localhost:8080".to_string(),
            capabilities: vec![],
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };
        registry.service_info.insert("service1".to_string(), service);
    });

    let task2 = tokio::spawn(async move {
        let mut registry = adapter2.capability_registry.write().await;
        let service = ServiceInfo {
            name: "service2".to_string(),
            primal_type: PrimalType::new("generic"),
            endpoint: "http://localhost:8081".to_string(),
            capabilities: vec![],
            health: HealthStatus::Healthy,
            metadata: HashMap::new(),
        };
        registry.service_info.insert("service2".to_string(), service);
    });

    let _ = tokio::join!(task1, task2);

    let stats = adapter.get_registry_stats().await;
    assert_eq!(stats.total_services, 2);
}
