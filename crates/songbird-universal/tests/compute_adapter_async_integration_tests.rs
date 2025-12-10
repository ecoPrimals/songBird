//! Async Integration Tests for Compute Adapter
//!
//! **Goal**: Test async network methods with mock HTTP servers
//! **Coverage Target**: Move from 60.13% → ~85% overall coverage
//!
//! This suite tests:
//! - `new_from_discovery()` - async capability discovery
//! - `collect_metrics()` - HTTP GET /metrics/compute
//! - `check_health()` - integration of collect_metrics
//! - Network error handling
//! - Timeout behavior

use songbird_universal::adapters::compute::{ComputeAdapter, HealthStatus as ComputeHealth};
use std::time::Duration;

// ============================================================================
// DISCOVERY ASYNC TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_new_from_discovery_with_env_variable() {
    let server = mockito::Server::new_async().await;
    let endpoint = server.url();

    // Set environment variable
    std::env::set_var("SONGBIRD_COMPUTE_ENDPOINT", &endpoint);

    // Should discover from env var
    let adapter = ComputeAdapter::new_from_discovery().await;
    assert!(adapter.is_ok(), "Should create adapter from env var");

    let adapter = adapter.unwrap();
    assert_eq!(adapter.endpoint(), &endpoint);

    // Cleanup
    std::env::remove_var("SONGBIRD_COMPUTE_ENDPOINT");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_new_from_discovery_with_legacy_env() {
    let server = mockito::Server::new_async().await;
    let endpoint = server.url();

    // Set legacy COMPUTE_PROVIDER_ENDPOINT
    std::env::set_var("COMPUTE_PROVIDER_ENDPOINT", &endpoint);

    // Should discover from legacy env var
    let adapter = ComputeAdapter::new_from_discovery().await;
    assert!(adapter.is_ok(), "Should create adapter from legacy env var");

    // Cleanup
    std::env::remove_var("COMPUTE_PROVIDER_ENDPOINT");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_new_from_discovery_fallback_to_default() {
    // Clear all env vars that might interfere
    std::env::remove_var("SONGBIRD_COMPUTE_ENDPOINT");
    std::env::remove_var("COMPUTE_PROVIDER_ENDPOINT");

    // Should fall back to default host:port
    let adapter = ComputeAdapter::new_from_discovery().await;
    assert!(adapter.is_ok(), "Should create adapter with fallback");

    let adapter = adapter.unwrap();
    // Should have some default endpoint
    assert!(!adapter.endpoint().is_empty());
}

// ============================================================================
// COLLECT_METRICS ASYNC TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_success() {
    let mut server = mockito::Server::new_async().await;

    // Mock successful metrics response
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "cpu_usage_percent": 45.5,
            "memory_usage_bytes": 2000000000,
            "memory_available_bytes": 6000000000,
            "active_containers": 5,
            "queued_jobs": 2,
            "performance_score": 0.85,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).unwrap();
    let metrics = adapter.collect_metrics().await;

    assert!(metrics.is_ok(), "Should collect metrics successfully");
    let metrics = metrics.unwrap();
    assert!((metrics.cpu_usage_percent - 45.5).abs() < 0.01);
    assert_eq!(metrics.memory_usage_bytes, 2000000000);
    assert_eq!(metrics.active_containers, 5);
    assert!((metrics.performance_score - 0.85).abs() < 0.01);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_sets_timestamp_if_missing() {
    let mut server = mockito::Server::new_async().await;

    // Mock response with zero/missing timestamp
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "cpu_usage_percent": 30.0,
            "memory_usage_bytes": 1000000000,
            "memory_available_bytes": 7000000000,
            "active_containers": 3,
            "queued_jobs": 1,
            "performance_score": 0.90,
            "timestamp": "1970-01-01T00:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).unwrap();
    let metrics = adapter.collect_metrics().await;

    assert!(metrics.is_ok());
    let metrics = metrics.unwrap();

    // Timestamp should be set to current time
    let now = chrono::Utc::now();
    let diff = (now - metrics.timestamp).num_seconds().abs();
    assert!(diff < 5, "Timestamp should be recent");

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_network_error() {
    // Use invalid endpoint
    let adapter = ComputeAdapter::new("http://localhost:1".to_string()).unwrap();

    let result = adapter.collect_metrics().await;
    assert!(result.is_err(), "Should fail with network error");

    let err = result.unwrap_err();
    assert!(err.to_string().contains("Failed to reach compute service"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_http_error_status() {
    let mut server = mockito::Server::new_async().await;

    // Mock 503 error response
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(503)
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).unwrap();
    let result = adapter.collect_metrics().await;

    assert!(result.is_err(), "Should fail with HTTP error");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("HTTP 503"));

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_invalid_json() {
    let mut server = mockito::Server::new_async().await;

    // Mock response with invalid JSON
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("not valid json")
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).unwrap();
    let result = adapter.collect_metrics().await;

    assert!(result.is_err(), "Should fail with parse error");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Failed to parse compute metrics"));

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_with_timeout() {
    let mut server = mockito::Server::new_async().await;

    // Mock slow response (will timeout)
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body_from_request(|_| {
            std::thread::sleep(Duration::from_secs(2));
            r#"{"cpu_usage_percent":0.0,"memory_usage_bytes":0,"memory_available_bytes":0,"active_containers":0,"queued_jobs":0,"performance_score":1.0,"timestamp":"2025-11-18T12:00:00Z"}"#.into()
        })
        .create_async()
        .await;

    let adapter =
        ComputeAdapter::new(server.url()).unwrap().with_timeout(Duration::from_millis(100)); // Very short timeout

    let result = adapter.collect_metrics().await;
    assert!(result.is_err(), "Should timeout");

    mock.assert_async().await;
}

// ============================================================================
// CHECK_HEALTH ASYNC TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_healthy() {
    let mut server = mockito::Server::new_async().await;

    // Mock healthy metrics
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "cpu_usage_percent": 40.0,
            "memory_usage_bytes": 2000000000,
            "memory_available_bytes": 6000000000,
            "active_containers": 4,
            "queued_jobs": 1,
            "performance_score": 0.88,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).unwrap();
    let health = adapter.check_health().await;

    assert!(health.is_ok());
    assert_eq!(health.unwrap(), ComputeHealth::Healthy);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_degraded_high_cpu() {
    let mut server = mockito::Server::new_async().await;

    // Mock metrics showing degraded state (high CPU)
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "cpu_usage_percent": 88.0,
            "memory_usage_bytes": 3000000000,
            "memory_available_bytes": 5000000000,
            "active_containers": 8,
            "queued_jobs": 5,
            "performance_score": 0.70,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).unwrap();
    let health = adapter.check_health().await;

    assert!(health.is_ok());
    assert_eq!(health.unwrap(), ComputeHealth::Degraded);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_degraded_high_memory() {
    let mut server = mockito::Server::new_async().await;

    // Mock metrics showing degraded state (high memory usage)
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "cpu_usage_percent": 60.0,
            "memory_usage_bytes": 7200000000,
            "memory_available_bytes": 800000000,
            "active_containers": 10,
            "queued_jobs": 3,
            "performance_score": 0.75,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).unwrap();
    let health = adapter.check_health().await;

    assert!(health.is_ok());
    // 7.2GB used out of 8GB total = 90% usage, should be degraded
    assert_eq!(health.unwrap(), ComputeHealth::Degraded);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_unhealthy() {
    let mut server = mockito::Server::new_async().await;

    // Mock metrics showing unhealthy state
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "cpu_usage_percent": 98.0,
            "memory_usage_bytes": 7800000000,
            "memory_available_bytes": 200000000,
            "active_containers": 20,
            "queued_jobs": 50,
            "performance_score": 0.25,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).unwrap();
    let health = adapter.check_health().await;

    assert!(health.is_ok());
    assert_eq!(health.unwrap(), ComputeHealth::Unhealthy);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_network_error() {
    let adapter = ComputeAdapter::new("http://localhost:1".to_string()).unwrap();

    let result = adapter.check_health().await;
    assert!(result.is_err(), "Should propagate network error from collect_metrics");
}

// ============================================================================
// INTEGRATION WORKFLOW TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_full_compute_workflow() {
    let mut server = mockito::Server::new_async().await;

    // Setup metrics endpoint
    let metrics_mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "cpu_usage_percent": 55.0,
            "memory_usage_bytes": 3000000000,
            "memory_available_bytes": 5000000000,
            "active_containers": 6,
            "queued_jobs": 2,
            "performance_score": 0.82,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .expect(2) // Called twice
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).unwrap();

    // 1. Check health
    let health = adapter.check_health().await;
    assert!(health.is_ok());
    assert_eq!(health.unwrap(), ComputeHealth::Healthy);

    // 2. Get metrics directly
    let metrics = adapter.collect_metrics().await;
    assert!(metrics.is_ok());
    let metrics = metrics.unwrap();
    assert_eq!(metrics.active_containers, 6);

    metrics_mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_requests() {
    let mut server = mockito::Server::new_async().await;

    // Mock endpoint that can handle concurrent requests
    let mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "cpu_usage_percent": 50.0,
            "memory_usage_bytes": 4000000000,
            "memory_available_bytes": 4000000000,
            "active_containers": 5,
            "queued_jobs": 2,
            "performance_score": 0.85,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .expect(3)
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).unwrap();

    // Fire off 3 concurrent requests
    let futures =
        vec![adapter.collect_metrics(), adapter.collect_metrics(), adapter.collect_metrics()];

    let results = futures::future::join_all(futures).await;

    // All should succeed
    for result in results {
        assert!(result.is_ok(), "Concurrent request should succeed");
    }

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_retry_on_transient_failure() {
    let mut server = mockito::Server::new_async().await;

    // First call fails
    let fail_mock = server
        .mock("GET", "/metrics/compute")
        .with_status(503)
        .with_body("Service Unavailable")
        .expect(1)
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).unwrap();

    // First attempt should fail
    let result1 = adapter.collect_metrics().await;
    assert!(result1.is_err());

    fail_mock.assert_async().await;
    fail_mock.remove_async().await;

    // Setup success mock
    let success_mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "cpu_usage_percent": 45.0,
            "memory_usage_bytes": 2000000000,
            "memory_available_bytes": 6000000000,
            "active_containers": 4,
            "queued_jobs": 1,
            "performance_score": 0.88,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    // Retry should succeed
    let result2 = adapter.collect_metrics().await;
    assert!(result2.is_ok(), "Retry should succeed");

    success_mock.assert_async().await;
}
