#![cfg(test)]

//! Async Integration Tests for Storage Adapter
//!
//! **Purpose**: Achieve 90% coverage (from 66.50%)

use super::*;
use std::time::Duration;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

// ============================================================================
// ADAPTER CONSTRUCTION TESTS (concurrent-safe, no env vars)
// ============================================================================

#[tokio::test]
async fn test_new_with_explicit_endpoint() {
    // ✅ Concurrent-safe: Uses explicit endpoint
    let result = StorageAdapter::new("http://test-storage:8082".to_string()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_new_with_legacy_endpoint() {
    // ✅ Concurrent-safe: Tests legacy-style endpoint
    let result = StorageAdapter::new("http://legacy-nestgate:8084".to_string()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_from_discovery_default() {
    // ✅ from_discovery always succeeds with defaults
    let result = StorageAdapter::from_discovery().await;
    assert!(result.is_ok());
}

// ============================================================================
// METRICS TESTS
// ============================================================================

#[tokio::test]
async fn test_collect_metrics_success() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/metrics/storage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total_capacity_bytes": 1_000_000_000_000u64,
            "used_bytes": 400_000_000_000u64,
            "available_bytes": 600_000_000_000u64,
            "object_count": 5000u64,
            "avg_read_latency_ms": 10.0,
            "avg_write_latency_ms": 20.0,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = StorageAdapter::new(mock_server.uri()).await.unwrap();
    let result = adapter.collect_metrics().await;
    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert_eq!(metrics.total_capacity_bytes, 1_000_000_000_000);
    assert_eq!(metrics.object_count, 5000);
}

#[tokio::test]
async fn test_collect_metrics_network_error() {
    let adapter = StorageAdapter::new("http://invalid:9999")
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
        .and(path("/metrics/storage"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let adapter = StorageAdapter::new(mock_server.uri()).await.unwrap();
    let result = adapter.collect_metrics().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_collect_metrics_invalid_json() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/metrics/storage"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid"))
        .mount(&mock_server)
        .await;

    let adapter = StorageAdapter::new(mock_server.uri()).await.unwrap();
    let result = adapter.collect_metrics().await;
    assert!(result.is_err());
}

// ============================================================================
// HEALTH TESTS
// ============================================================================

#[tokio::test]
async fn test_check_health_healthy() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/metrics/storage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total_capacity_bytes": 1_000_000_000_000u64,
            "used_bytes": 300_000_000_000u64,
            "available_bytes": 700_000_000_000u64,
            "object_count": 3000,
            "avg_read_latency_ms": 50.0,
            "avg_write_latency_ms": 50.0,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = StorageAdapter::new(mock_server.uri()).await.unwrap();
    let result = adapter.check_health().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), StorageHealth::Healthy);
}

#[tokio::test]
async fn test_check_health_degraded() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/metrics/storage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total_capacity_bytes": 1_000_000_000_000u64,
            "used_bytes": 870_000_000_000u64,
            "available_bytes": 130_000_000_000u64,
            "object_count": 8000u64,
            "avg_read_latency_ms": 50.0,
            "avg_write_latency_ms": 50.0,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = StorageAdapter::new(mock_server.uri()).await.unwrap();
    let result = adapter.check_health().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), StorageHealth::Warning);
}

#[tokio::test]
async fn test_check_health_critical() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/metrics/storage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total_capacity_bytes": 1_000_000_000_000u64,
            "used_bytes": 980_000_000_000u64,
            "available_bytes": 20_000_000_000u64,
            "object_count": 10000,
            "avg_read_latency_ms": 8000,
            "avg_write_latency_ms": 0.30,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = StorageAdapter::new(mock_server.uri()).await.unwrap();
    let result = adapter.check_health().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), StorageHealth::Critical);
}

// ============================================================================
// TRAIT TESTS
// ============================================================================

#[tokio::test]
async fn test_storage_provider_trait() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/metrics/storage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total_capacity_bytes": 1_000_000_000_000u64,
            "used_bytes": 200_000_000_000u64,
            "available_bytes": 800_000_000_000u64,
            "object_count": 2000,
            "avg_read_latency_ms": 1500,
            "avg_write_latency_ms": 0.98,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = StorageAdapter::new(mock_server.uri()).await.unwrap();
    let result = StorageProvider::collect_storage_metrics(&adapter).await;
    assert!(result.is_ok());
}

// ============================================================================
// METRICS CALCULATIONS
// ============================================================================

#[tokio::test]
async fn test_metrics_calculations() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 400_000_000_000,
        available_bytes: 600_000_000_000,
        object_count: 3000,
        avg_read_latency_ms: 50.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.usage_percent(), 40.0);
    assert!(!metrics.is_nearly_full());
    assert!(!metrics.is_nearly_full());
    assert_eq!(metrics.health_status(), StorageHealth::Healthy);
}

#[tokio::test]
async fn test_metrics_low_space_detection() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 920_000_000_000,
        available_bytes: 80_000_000_000,
        object_count: 4000,
        avg_read_latency_ms: 3000.0,
        avg_write_latency_ms: 0.70,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.usage_percent(), 92.0);
    assert!(metrics.is_nearly_full());
    // Usage is 92%, which triggers Warning (> 85%)
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

#[tokio::test]
async fn test_metrics_high_latency() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 1_000_000_000_000,
        used_bytes: 500_000_000_000,
        available_bytes: 500_000_000_000,
        object_count: 12000u64,
        avg_read_latency_ms: 150.0,
        avg_write_latency_ms: 50.0,
        timestamp: chrono::Utc::now(),
    };

    // High latency (read > 100ms) triggers Warning
    assert!(metrics.is_high_latency());
    assert_eq!(metrics.health_status(), StorageHealth::Warning);
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_concurrent_operations() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/metrics/storage"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "total_capacity_bytes": 1_000_000_000_000u64,
            "used_bytes": 300_000_000_000u64,
            "available_bytes": 700_000_000_000u64,
            "object_count": 3000,
            "avg_read_latency_ms": 2000,
            "avg_write_latency_ms": 0.90,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = StorageAdapter::new(mock_server.uri()).await.unwrap();
    let adapter_ref = &adapter;

    let futures: Vec<_> =
        (0..5).map(|_| async move { adapter_ref.collect_metrics().await }).collect();

    let results = futures::future::join_all(futures).await;
    for result in results {
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_metrics_zero_capacity_edge_case() {
    let metrics = StorageMetrics {
        total_capacity_bytes: 0,
        used_bytes: 0,
        available_bytes: 0,
        object_count: 0,
        avg_read_latency_ms: 0.0,
        avg_write_latency_ms: 0.0,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.usage_percent(), 0.0);
}
