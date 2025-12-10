#![cfg(test)]

//! Async Integration Tests for AI Adapter
//!
//! **Purpose**: Achieve 90% coverage (from 64.62%)

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
async fn test_from_discovery_with_env() {
    std::env::set_var("SONGBIRD_AI_ENDPOINT", "http://test-ai:9000");
    let result = AIAdapter::from_discovery().await;
    assert!(result.is_ok());
    std::env::remove_var("SONGBIRD_AI_ENDPOINT");
}

#[tokio::test]
async fn test_from_discovery_squirrel_fallback() {
    std::env::set_var("SQUIRREL_ENDPOINT", "http://legacy-squirrel:9010");
    std::env::remove_var("SONGBIRD_AI_ENDPOINT");
    let result = AIAdapter::from_discovery().await;
    assert!(result.is_ok());
    std::env::remove_var("SQUIRREL_ENDPOINT");
}

#[tokio::test]
async fn test_from_discovery_default() {
    std::env::remove_var("SONGBIRD_AI_ENDPOINT");
    std::env::remove_var("AI_PROVIDER_ENDPOINT");
    std::env::remove_var("SQUIRREL_ENDPOINT");
    let result = AIAdapter::from_discovery().await;
    assert!(result.is_ok());
}

// ============================================================================
// METRICS TESTS
// ============================================================================

#[tokio::test]
async fn test_collect_metrics_success() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/metrics/ai"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_models": 5,
            "total_requests": 10000u64,
            "avg_latency_ms": 250.0,
            "accuracy_score": 0.95,
            "gpu_utilization_percent": 60.0,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = AIAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.collect_metrics().await;
    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert_eq!(metrics.active_models, 5);
}

#[tokio::test]
async fn test_collect_metrics_network_error() {
    let adapter = AIAdapter::new("http://invalid:9999")
        .unwrap()
        .with_timeout(Duration::from_millis(100));
    let result = adapter.collect_metrics().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_collect_metrics_http_500() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/metrics/ai"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let adapter = AIAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.collect_metrics().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_collect_metrics_invalid_json() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/metrics/ai"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid"))
        .mount(&mock_server)
        .await;

    let adapter = AIAdapter::new(mock_server.uri()).unwrap();
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
        .and(path("/metrics/ai"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_models": 3,
            "total_requests": 5000u64,
            "avg_latency_ms": 200.0,
            "accuracy_score": 0.98,
            "gpu_utilization_percent": 50.0,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = AIAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.check_health().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AIHealth::Healthy);
}

#[tokio::test]
async fn test_check_health_degraded() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/metrics/ai"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_models": 10,
            "total_requests": 50000u64,
            "avg_latency_ms": 1500.0,
            "accuracy_score": 0.85,
            "gpu_utilization_percent": 95.0,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = AIAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.check_health().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AIHealth::Degraded);
}

#[tokio::test]
async fn test_check_health_overloaded() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/metrics/ai"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_models": 20,
            "total_requests": 100000u64,
            "avg_latency_ms": 3000.0,
            "accuracy_score": 0.70,
            "gpu_utilization_percent": 99.0,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = AIAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.check_health().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AIHealth::Overloaded);
}

// ============================================================================
// TRAIT TESTS
// ============================================================================

#[tokio::test]
async fn test_ai_provider_trait() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/metrics/ai"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_models": 2,
            "total_requests": 1000u64,
            "avg_latency_ms": 100.0,
            "accuracy_score": 0.99,
            "gpu_utilization_percent": 30.0,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = AIAdapter::new(mock_server.uri()).unwrap();
    let result = AIProvider::collect_ai_metrics(&adapter).await;
    assert!(result.is_ok());
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_metrics_calculations() {
    let metrics = AIMetrics {
        active_models: 5,
        total_requests: 10000,
        avg_latency_ms: 500.0,
        accuracy_score: 0.90,
        gpu_utilization_percent: 85.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_high_gpu_load());
    assert!(!metrics.is_high_latency());
    assert_eq!(metrics.health_status(), AIHealth::Healthy);
}

#[tokio::test]
async fn test_concurrent_requests() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/metrics/ai"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_models": 3,
            "total_requests": 5000u64,
            "avg_latency_ms": 200.0,
            "accuracy_score": 0.95,
            "gpu_utilization_percent": 60.0,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = AIAdapter::new(mock_server.uri()).unwrap();
    let adapter_ref = &adapter;

    let futures: Vec<_> =
        (0..5).map(|_| async move { adapter_ref.collect_metrics().await }).collect();

    let results = futures::future::join_all(futures).await;
    for result in results {
        assert!(result.is_ok());
    }
}
