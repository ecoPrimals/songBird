#![cfg(test)]

//! Async Integration Tests for Compute Adapter
//!
//! **Purpose**: Achieve 90% coverage (from 60.13%)
//!
//! Coverage targets:
//! - `new_from_discovery()` method
//! - `collect_metrics()` async operations
//! - `check_health()` async operations
//! - Error handling paths
//! - Network timeout scenarios

use super::*;
use std::time::Duration;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

// ============================================================================
// DISCOVERY TESTS
// ============================================================================

#[tokio::test]
async fn test_new_from_discovery_with_env() {
    std::env::set_var("SONGBIRD_COMPUTE_ENDPOINT", "http://test-compute:8080");

    let result = ComputeAdapter::new_from_discovery().await;

    assert!(result.is_ok());
    let adapter = result.unwrap();
    assert!(!adapter.endpoint().is_empty());

    std::env::remove_var("SONGBIRD_COMPUTE_ENDPOINT");
}

#[tokio::test]
async fn test_new_from_discovery_toadstool_fallback() {
    std::env::set_var("TOADSTOOL_ENDPOINT", "http://legacy-toadstool:9000");
    std::env::remove_var("SONGBIRD_COMPUTE_ENDPOINT");
    std::env::remove_var("COMPUTE_CAPABILITY_ENDPOINT");

    let result = ComputeAdapter::new_from_discovery().await;

    assert!(result.is_ok());
    let adapter = result.unwrap();
    assert!(!adapter.endpoint().is_empty());

    std::env::remove_var("TOADSTOOL_ENDPOINT");
}

#[tokio::test]
async fn test_new_from_discovery_default_fallback() {
    std::env::remove_var("SONGBIRD_COMPUTE_ENDPOINT");
    std::env::remove_var("COMPUTE_CAPABILITY_ENDPOINT");
    std::env::remove_var("TOADSTOOL_ENDPOINT");

    let result = ComputeAdapter::new_from_discovery().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_new_from_discovery_custom_port() {
    std::env::set_var("SONGBIRD_COMPUTE_PORT", "7777");
    std::env::remove_var("SONGBIRD_COMPUTE_ENDPOINT");

    let result = ComputeAdapter::new_from_discovery().await;

    assert!(result.is_ok());

    std::env::remove_var("SONGBIRD_COMPUTE_PORT");
}

// ============================================================================
// COLLECT_METRICS TESTS
// ============================================================================

#[tokio::test]
async fn test_collect_metrics_success() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/compute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cpu_usage_percent": 45.5,
            "memory_usage_bytes": 2_147_483_648u64,
            "memory_available_bytes": 2_147_483_648u64,
            "active_containers": 10,
            "queued_jobs": 2,
            "performance_score": 0.85,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = ComputeAdapter::new(mock_server.uri()).await.unwrap();
    let result = adapter.collect_metrics().await;

    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert_eq!(metrics.cpu_usage_percent, 45.5);
    assert_eq!(metrics.active_containers, 10);
    assert_eq!(metrics.queued_jobs, 2);
}

#[tokio::test]
async fn test_collect_metrics_network_error() {
    let adapter = ComputeAdapter::new("http://invalid-host-xyz:9999")
        .await
        .unwrap()
        .with_timeout(Duration::from_millis(100));

    let result = adapter.collect_metrics().await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_collect_metrics_http_500() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/compute"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let adapter = ComputeAdapter::new(mock_server.uri()).await.unwrap();
    let result = adapter.collect_metrics().await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_collect_metrics_invalid_json() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/compute"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
        .mount(&mock_server)
        .await;

    let adapter = ComputeAdapter::new(mock_server.uri()).await.unwrap();
    let result = adapter.collect_metrics().await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_collect_metrics_missing_timestamp() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/compute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cpu_usage_percent": 10.0,
            "memory_usage_bytes": 1_000_000_000u64,
            "memory_available_bytes": 3_000_000_000u64,
            "active_containers": 5,
            "queued_jobs": 0,
            "performance_score": 0.95,
            "timestamp": "1970-01-01T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = ComputeAdapter::new(mock_server.uri()).await.unwrap();
    let result = adapter.collect_metrics().await;

    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert!(metrics.timestamp.timestamp() > 0);
}

// ============================================================================
// HEALTH CHECK TESTS
// ============================================================================

#[tokio::test]
async fn test_check_health_healthy() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/compute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cpu_usage_percent": 30.0,
            "memory_usage_bytes": 1_000_000_000u64,
            "memory_available_bytes": 3_000_000_000u64,
            "active_containers": 5,
            "queued_jobs": 2,
            "performance_score": 0.9,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = ComputeAdapter::new(mock_server.uri()).await.unwrap();
    let result = adapter.check_health().await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), HealthStatus::Healthy);
}

#[tokio::test]
async fn test_check_health_degraded() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/compute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cpu_usage_percent": 85.0,
            "memory_usage_bytes": 3_500_000_000u64,
            "memory_available_bytes": 500_000_000u64,
            "active_containers": 20,
            "queued_jobs": 8,
            "performance_score": 0.6,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = ComputeAdapter::new(mock_server.uri()).await.unwrap();
    let result = adapter.check_health().await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), HealthStatus::Degraded);
}

#[tokio::test]
async fn test_check_health_unhealthy() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/compute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cpu_usage_percent": 98.0,
            "memory_usage_bytes": 3_900_000_000u64,
            "memory_available_bytes": 100_000_000u64,
            "active_containers": 50,
            "queued_jobs": 20,
            "performance_score": 0.1,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = ComputeAdapter::new(mock_server.uri()).await.unwrap();
    let result = adapter.check_health().await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), HealthStatus::Unhealthy);
}

#[tokio::test]
async fn test_check_health_network_failure() {
    let adapter = ComputeAdapter::new("http://unreachable:9999")
        .await
        .unwrap()
        .with_timeout(Duration::from_millis(100));

    let result = adapter.check_health().await;

    assert!(result.is_err());
}

// ============================================================================
// METRICS CALCULATIONS TESTS
// ============================================================================

#[tokio::test]
async fn test_metrics_calculations() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/compute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cpu_usage_percent": 50.0,
            "memory_usage_bytes": 2_000_000_000u64,
            "memory_available_bytes": 2_000_000_000u64,
            "active_containers": 10,
            "queued_jobs": 5,
            "performance_score": 0.8,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = ComputeAdapter::new(mock_server.uri()).await.unwrap();
    let result = adapter.collect_metrics().await;

    assert!(result.is_ok());
    let metrics = result.unwrap();

    assert_eq!(metrics.total_memory_bytes(), 4_000_000_000);
    assert_eq!(metrics.memory_usage_percent(), 50.0);
    assert!(!metrics.is_high_load());
}

#[tokio::test]
async fn test_metrics_high_load_detection() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/compute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cpu_usage_percent": 90.0,
            "memory_usage_bytes": 3_500_000_000u64,
            "memory_available_bytes": 500_000_000u64,
            "active_containers": 30,
            "queued_jobs": 15,
            "performance_score": 0.5,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = ComputeAdapter::new(mock_server.uri()).await.unwrap();
    let result = adapter.collect_metrics().await;

    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert!(metrics.is_high_load());
    assert_eq!(metrics.health_status(), HealthStatus::Degraded);
}

// ============================================================================
// TRAIT IMPLEMENTATION TESTS
// ============================================================================

#[tokio::test]
async fn test_compute_metrics_provider_trait() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/compute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cpu_usage_percent": 25.0,
            "memory_usage_bytes": 1_000_000_000u64,
            "memory_available_bytes": 3_000_000_000u64,
            "active_containers": 8,
            "queued_jobs": 3,
            "performance_score": 0.92,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = ComputeAdapter::new(mock_server.uri()).await.unwrap();
    let result = ComputeMetricsProvider::collect_compute_metrics(&adapter).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().active_containers, 8);
}

#[tokio::test]
async fn test_compute_metrics_provider_check_health() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/compute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cpu_usage_percent": 15.0,
            "memory_usage_bytes": 500_000_000u64,
            "memory_available_bytes": 3_500_000_000u64,
            "active_containers": 3,
            "queued_jobs": 1,
            "performance_score": 0.98,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = ComputeAdapter::new(mock_server.uri()).await.unwrap();
    let result = ComputeMetricsProvider::check_compute_health(&adapter).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), HealthStatus::Healthy);
}

// ============================================================================
// EDGE CASES & STRESS TESTS
// ============================================================================

#[tokio::test]
async fn test_adapter_empty_endpoint() {
    let result = ComputeAdapter::new(String::new());
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_adapter_very_long_endpoint() {
    let long_endpoint = format!("http://compute-{}.example.com", "provider".repeat(100));
    let result = ComputeAdapter::new(long_endpoint.clone());

    assert!(result.is_ok());
    assert_eq!(result.unwrap().endpoint(), long_endpoint);
}

#[tokio::test]
async fn test_adapter_multiple_sequential_requests() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/compute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cpu_usage_percent": 40.0,
            "memory_usage_bytes": 2_000_000_000u64,
            "memory_available_bytes": 2_000_000_000u64,
            "active_containers": 8,
            "queued_jobs": 3,
            "performance_score": 0.85,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = ComputeAdapter::new(mock_server.uri()).await.unwrap();

    for _ in 0..10 {
        let result = adapter.collect_metrics().await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_adapter_concurrent_requests() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/compute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "cpu_usage_percent": 35.0,
            "memory_usage_bytes": 1_500_000_000u64,
            "memory_available_bytes": 2_500_000_000u64,
            "active_containers": 7,
            "queued_jobs": 2,
            "performance_score": 0.88,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = ComputeAdapter::new(mock_server.uri()).await.unwrap();
    let adapter_ref = &adapter;

    let futures: Vec<_> =
        (0..5).map(|_| async move { adapter_ref.collect_metrics().await }).collect();

    let results = futures::future::join_all(futures).await;

    for result in results {
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_metrics_zero_memory_edge_case() {
    let metrics = ComputeMetrics {
        cpu_usage_percent: 50.0,
        memory_usage_bytes: 0,
        memory_available_bytes: 0,
        active_containers: 5,
        queued_jobs: 2,
        performance_score: 0.8,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.total_memory_bytes(), 0);
    assert_eq!(metrics.memory_usage_percent(), 0.0);
}

#[tokio::test]
async fn test_custom_timeout() {
    let adapter = ComputeAdapter::new("http://slow-host:9999")
        .unwrap()
        .with_timeout(Duration::from_millis(50));

    let result = adapter.collect_metrics().await;
    assert!(result.is_err());
}
