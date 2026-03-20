// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Coverage tests for songbird-types::adapters::canonical
//!
//! Tests the CanonicalUniversalAdapter, configs, enums, and supporting types.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

use songbird_types::adapters::canonical::*;
use songbird_types::traits::canonical::{
    Endpoint, HealthStatus as CanonicalHealthStatus, ProviderType as CanonicalProviderType,
    ServiceInfo as CanonicalServiceInfo, ServiceType,
};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

// ==================== CONFIG DEFAULTS ====================

#[test]
fn test_adapter_config_default() {
    let config = CanonicalAdapterConfig::default();
    assert!(config.retry.max_attempts > 0);
    assert!(config.timeouts.request_timeout > Duration::ZERO);
    assert!(config.health_check.interval > Duration::ZERO);
}

#[test]
fn test_discovery_config_default() {
    let config = CanonicalDiscoveryConfig::default();
    assert!(config.interval > Duration::ZERO);
    assert!(config.timeout > Duration::ZERO);
    assert!(config.max_services_per_capability > 0);
    assert!(config.service_ttl > Duration::ZERO);
}

#[test]
fn test_load_balancing_config_default() {
    let config = CanonicalLoadBalancingConfig::default();
    assert_eq!(config.strategy, CanonicalLoadBalancingStrategy::HealthAware);
    assert!(config.health_weight > 0.0);
    assert!(config.performance_weight > 0.0);
    assert!(config.availability_weight > 0.0);
}

#[test]
fn test_circuit_breaker_config_default() {
    let config = CanonicalCircuitBreakerConfig::default();
    assert!(config.failure_threshold > 0);
    assert!(config.success_threshold > 0);
    assert!(config.timeout > Duration::ZERO);
    assert!(config.reset_timeout > Duration::ZERO);
}

#[test]
fn test_retry_config_default() {
    let config = CanonicalRetryConfig::default();
    assert!(config.max_attempts > 0);
    assert!(config.initial_delay > Duration::ZERO);
    assert!(config.max_delay > Duration::ZERO);
    assert!(config.backoff_multiplier >= 1.0);
}

#[test]
fn test_timeout_config_default() {
    let config = CanonicalTimeoutConfig::default();
    assert!(config.request_timeout > Duration::ZERO);
    assert!(config.connection_timeout > Duration::ZERO);
    assert!(config.health_check_timeout > Duration::ZERO);
    assert!(config.discovery_timeout > Duration::ZERO);
}

#[test]
fn test_health_check_config_default() {
    let config = CanonicalHealthCheckConfig::default();
    assert!(config.interval > Duration::ZERO);
    assert!(config.timeout > Duration::ZERO);
    assert!(config.unhealthy_threshold > 0);
    assert!(config.healthy_threshold > 0);
}

#[test]
fn test_monitoring_config_default() {
    let config = CanonicalMonitoringConfig::default();
    assert!(config.enabled);
    assert!(config.collection_interval > Duration::ZERO);
    assert!(config.retention_period > Duration::ZERO);
    assert!(config.history_size > 0);
}

// ==================== CONFIG SERIALIZATION ====================

#[test]
fn test_adapter_config_serialization() {
    let config = CanonicalAdapterConfig::default();
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: CanonicalAdapterConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.retry.max_attempts, config.retry.max_attempts);
    assert_eq!(deserialized.load_balancing.strategy, config.load_balancing.strategy);
}

#[test]
fn test_circuit_breaker_config_serialization() {
    let config = CanonicalCircuitBreakerConfig {
        failure_threshold: 10,
        success_threshold: 3,
        timeout: Duration::from_secs(30),
        reset_timeout: Duration::from_secs(120),
    };
    let json = serde_json::to_string(&config).unwrap();
    let de: CanonicalCircuitBreakerConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(de.failure_threshold, 10);
    assert_eq!(de.success_threshold, 3);
}

// ==================== ENUM TYPES ====================

#[test]
fn test_load_balancing_strategy_variants() {
    let strategies = vec![
        CanonicalLoadBalancingStrategy::RoundRobin,
        CanonicalLoadBalancingStrategy::WeightedRoundRobin,
        CanonicalLoadBalancingStrategy::LeastConnections,
        CanonicalLoadBalancingStrategy::LeastResponseTime,
        CanonicalLoadBalancingStrategy::Random,
        CanonicalLoadBalancingStrategy::ConsistentHash,
        CanonicalLoadBalancingStrategy::HealthAware,
    ];
    for s in &strategies {
        let json = serde_json::to_string(s).unwrap();
        let de: CanonicalLoadBalancingStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(&de, s);
    }
    assert_ne!(strategies[0], strategies[1]);
}

#[test]
fn test_circuit_state_variants() {
    let states = vec![
        CanonicalCircuitState::Closed,
        CanonicalCircuitState::Open,
        CanonicalCircuitState::HalfOpen,
    ];
    for s in &states {
        let json = serde_json::to_string(s).unwrap();
        let de: CanonicalCircuitState = serde_json::from_str(&json).unwrap();
        assert_eq!(&de, s);
    }
}

#[test]
fn test_request_priority_ordering() {
    assert!(CanonicalRequestPriority::Low < CanonicalRequestPriority::Normal);
    assert!(CanonicalRequestPriority::Normal < CanonicalRequestPriority::High);
    assert!(CanonicalRequestPriority::High < CanonicalRequestPriority::Critical);
}

#[test]
fn test_request_priority_serialization() {
    let priorities = vec![
        CanonicalRequestPriority::Low,
        CanonicalRequestPriority::Normal,
        CanonicalRequestPriority::High,
        CanonicalRequestPriority::Critical,
    ];
    for p in &priorities {
        let json = serde_json::to_string(p).unwrap();
        let de: CanonicalRequestPriority = serde_json::from_str(&json).unwrap();
        assert_eq!(&de, p);
    }
}

// ==================== SUPPORTING TYPES ====================

#[test]
fn test_service_performance_default() {
    let perf = CanonicalServicePerformance::default();
    assert_eq!(perf.total_requests, 0);
    assert_eq!(perf.successful_requests, 0);
    assert_eq!(perf.failed_requests, 0);
    assert!(perf.success_rate >= 0.0);
}

#[test]
fn test_adapter_metrics_default() {
    let metrics = CanonicalAdapterMetrics::default();
    assert_eq!(metrics.total_requests, 0);
    assert_eq!(metrics.successful_requests, 0);
    assert_eq!(metrics.failed_requests, 0);
    assert_eq!(metrics.circuit_breaker_activations, 0);
    assert!(metrics.requests_by_capability.is_empty());
    assert!(metrics.requests_by_service_type.is_empty());
}

#[test]
fn test_adapter_request_serialization() {
    let mut metadata = HashMap::new();
    metadata.insert("key".to_string(), "value".to_string());
    let request = CanonicalAdapterRequest {
        id: "req-1".to_string(),
        capability: "compute".to_string(),
        payload: serde_json::json!({"task": "run"}),
        priority: CanonicalRequestPriority::High,
        timeout: Some(Duration::from_secs(30)),
        metadata,
    };
    let json = serde_json::to_string(&request).unwrap();
    let de: CanonicalAdapterRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(de.id, "req-1");
    assert_eq!(de.capability, "compute");
    assert_eq!(de.priority, CanonicalRequestPriority::High);
    assert!(de.timeout.is_some());
}

#[test]
fn test_adapter_response_serialization() {
    let response = CanonicalAdapterResponse {
        request_id: "req-1".to_string(),
        service_id: "svc-1".to_string(),
        payload: serde_json::json!({"result": "ok"}),
        metadata: HashMap::new(),
        processing_time: Duration::from_millis(42),
        performance_info: CanonicalServicePerformance::default(),
    };
    let json = serde_json::to_string(&response).unwrap();
    let de: CanonicalAdapterResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(de.request_id, "req-1");
    assert_eq!(de.service_id, "svc-1");
}

// ==================== PROTOCOL ROUTER ====================

#[test]
fn test_protocol_router_new() {
    let router = CanonicalProtocolRouter::new();
    let debug = format!("{router:?}");
    assert!(debug.contains("CanonicalProtocolRouter"));
    assert!(debug.contains("async registry"));
}

#[test]
fn test_protocol_router_default() {
    let router = CanonicalProtocolRouter::default();
    let debug = format!("{router:?}");
    assert!(debug.contains("http"));
}

// ==================== LOAD BALANCER ====================

#[test]
fn test_load_balancer_new() {
    let config = CanonicalLoadBalancingConfig::default();
    let lb = CanonicalLoadBalancer::new(config);
    let debug = format!("{lb:?}");
    assert!(debug.contains("CanonicalLoadBalancer"));
}

// ==================== CIRCUIT BREAKER ====================

#[tokio::test]
async fn test_circuit_breaker_new() {
    let config = CanonicalCircuitBreakerConfig::default();
    let cb = CanonicalCircuitBreaker::new(config);
    assert!(cb.can_execute("test-service").await);
}

#[tokio::test]
async fn test_circuit_breaker_success_recording() {
    let config = CanonicalCircuitBreakerConfig::default();
    let cb = CanonicalCircuitBreaker::new(config);
    cb.record_success("test-service").await;
    assert!(cb.can_execute("test-service").await);
}

#[tokio::test]
async fn test_circuit_breaker_failure_recording() {
    let config = CanonicalCircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 1,
        timeout: Duration::from_millis(50),
        reset_timeout: Duration::from_millis(50),
    };
    let cb = CanonicalCircuitBreaker::new(config);
    cb.record_failure("test-service").await;
    cb.record_failure("test-service").await;
    // After 2 failures (threshold=2), circuit should be open
    assert!(!cb.can_execute("test-service").await);
}

// ==================== ADAPTER (async) ====================

#[tokio::test]
async fn test_adapter_creation() {
    let config = CanonicalAdapterConfig::default();
    let adapter = CanonicalUniversalAdapter::new(config);
    let metrics = adapter.get_metrics().await;
    assert_eq!(metrics.total_requests, 0);
}

#[tokio::test]
async fn test_adapter_register_service() {
    let config = CanonicalAdapterConfig::default();
    let adapter = CanonicalUniversalAdapter::new(config);

    let service = CanonicalServiceInfo {
        id: "svc-1".to_string(),
        name: "Test Service".to_string(),
        service_type: ServiceType::Authentication,
        version: "1.0.0".to_string(),
        endpoints: vec![Endpoint {
            protocol: "http".to_string(),
            host: "localhost".to_string(),
            port: 8080,
            path: None,
            metadata: HashMap::new(),
        }],
        health: CanonicalHealthStatus::Healthy,
        metadata: HashMap::new(),
        tags: vec![],
        capabilities: vec!["auth".to_string()],
        last_updated: SystemTime::now(),
    };

    let result =
        adapter.register_service(service, vec!["auth".to_string(), "crypto".to_string()]).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_adapter_handle_request_no_services() {
    let config = CanonicalAdapterConfig::default();
    let adapter = CanonicalUniversalAdapter::new(config);

    let request = CanonicalAdapterRequest {
        id: "req-1".to_string(),
        capability: "nonexistent".to_string(),
        payload: serde_json::json!({}),
        priority: CanonicalRequestPriority::Normal,
        timeout: None,
        metadata: HashMap::new(),
    };

    let result = adapter.handle_request(request).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("No services found"));
}

#[tokio::test]
async fn test_adapter_health_check_all_empty() {
    let config = CanonicalAdapterConfig::default();
    let adapter = CanonicalUniversalAdapter::new(config);
    let results = adapter.health_check_all().await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_adapter_health_check_all_with_service() {
    let config = CanonicalAdapterConfig::default();
    let adapter = CanonicalUniversalAdapter::new(config);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        loop {
            if listener.accept().await.is_err() {
                break;
            }
        }
    });

    let service = CanonicalServiceInfo {
        id: "svc-health".to_string(),
        name: "Health Test".to_string(),
        service_type: ServiceType::WebService,
        version: "1.0.0".to_string(),
        endpoints: vec![Endpoint {
            protocol: "http".to_string(),
            host: "127.0.0.1".to_string(),
            port,
            path: None,
            metadata: HashMap::new(),
        }],
        health: CanonicalHealthStatus::Healthy,
        metadata: HashMap::new(),
        tags: vec![],
        capabilities: vec!["compute".to_string()],
        last_updated: SystemTime::now(),
    };
    adapter.register_service(service, vec!["compute".to_string()]).await.unwrap();

    let results = adapter.health_check_all().await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results["svc-health"], CanonicalHealthStatus::Healthy);
}

#[tokio::test]
async fn test_adapter_metrics_increment_on_request() {
    let config = CanonicalAdapterConfig::default();
    let adapter = CanonicalUniversalAdapter::new(config);

    let request = CanonicalAdapterRequest {
        id: "req-1".to_string(),
        capability: "nope".to_string(),
        payload: serde_json::json!({}),
        priority: CanonicalRequestPriority::Normal,
        timeout: None,
        metadata: HashMap::new(),
    };

    let _ = adapter.handle_request(request).await;
    let metrics = adapter.get_metrics().await;
    assert_eq!(metrics.total_requests, 1);
}

// ==================== REGISTERED SERVICE ====================

#[test]
fn test_registered_service_serialization() {
    let service = CanonicalRegisteredService {
        service: CanonicalServiceInfo {
            id: "svc-1".to_string(),
            name: "Test".to_string(),
            service_type: ServiceType::FileStorage,
            version: "0.1.0".to_string(),
            endpoints: vec![Endpoint {
                protocol: "http".to_string(),
                host: "localhost".to_string(),
                port: 8080,
                path: None,
                metadata: HashMap::new(),
            }],
            health: CanonicalHealthStatus::Healthy,
            metadata: HashMap::new(),
            tags: vec![],
            capabilities: vec![],
            last_updated: SystemTime::now(),
        },
        capabilities: vec!["storage".to_string()],
        provider_type: CanonicalProviderType::Service,
        registered_at: std::time::SystemTime::now(),
        last_health_check: None,
        performance: CanonicalServicePerformance::default(),
    };
    let json = serde_json::to_string(&service).unwrap();
    let de: CanonicalRegisteredService = serde_json::from_str(&json).unwrap();
    assert_eq!(de.service.id, "svc-1");
    assert_eq!(de.capabilities, vec!["storage".to_string()]);
}

// ==================== DEBUG IMPLS ====================

#[test]
fn test_debug_impls() {
    let config = CanonicalAdapterConfig::default();
    let debug_config = format!("{config:?}");
    assert!(debug_config.contains("CanonicalAdapterConfig"));

    let perf = CanonicalServicePerformance::default();
    let debug_perf = format!("{perf:?}");
    assert!(debug_perf.contains("CanonicalServicePerformance"));

    let metrics = CanonicalAdapterMetrics::default();
    let debug_metrics = format!("{metrics:?}");
    assert!(debug_metrics.contains("CanonicalAdapterMetrics"));
}
