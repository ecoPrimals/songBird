//! Comprehensive Integration Tests for All Adapters
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

//!
//! This test suite validates all four primal adapters (`ToadStool`, `BearDog`, `NestGate`, Squirrel)
//! using mock HTTP servers to simulate real primal interactions.

use songbird_universal::adapters::{AIAdapter, ComputeAdapter, SecurityAdapter, StorageAdapter};
use std::time::Duration;

// ============================================================================
// TOADSTOOL COMPUTE ADAPTER TESTS
// ============================================================================

#[tokio::test]
async fn test_toadstool_collect_metrics_success() {
    let mut server = mockito::Server::new_async().await;

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
                "timestamp": "2025-10-23T00:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).expect("Failed to create adapter");
    let metrics = adapter.collect_metrics().await.expect("Failed to collect metrics");

    assert_eq!(metrics.cpu_usage_percent, 45.5);
    assert_eq!(metrics.active_containers, 5);
    assert_eq!(metrics.queued_jobs, 2);
    assert!((metrics.performance_score - 0.85).abs() < 0.01);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_toadstool_collect_metrics_network_error() {
    let adapter = ComputeAdapter::new("http://nonexistent-host-12345.invalid:8080".to_string())
        .expect("Failed to create adapter");

    let result = adapter.collect_metrics().await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Failed to reach compute service"));
}

#[tokio::test]
async fn test_toadstool_check_health() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "cpu_usage_percent": 75.0,
                "memory_usage_bytes": 5000000000,
                "memory_available_bytes": 3000000000,
                "active_containers": 10,
                "queued_jobs": 5,
                "performance_score": 0.7,
                "timestamp": "2025-10-23T00:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url()).expect("Failed to create adapter");
    let health = adapter.check_health().await.expect("Failed to check health");

    // Should be healthy (CPU < 80%, Memory usage = 62.5%)
    assert_eq!(health, songbird_universal::adapters::compute::HealthStatus::Healthy);
}

#[tokio::test]
async fn test_toadstool_adapter_with_custom_timeout() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
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
                "performance_score": 0.95,
                "timestamp": "2025-10-23T00:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = ComputeAdapter::new(server.url())
        .expect("Failed to create adapter")
        .with_timeout(Duration::from_secs(2));

    let metrics = adapter.collect_metrics().await.expect("Failed to collect metrics");
    assert_eq!(metrics.cpu_usage_percent, 30.0);
}

// ============================================================================
// BEARDOG SECURITY ADAPTER TESTS
// ============================================================================

#[tokio::test]
async fn test_beardog_collect_metrics_success() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "active_sessions": 50,
                "failed_auth_attempts": 10,
                "blocked_ips": 2,
                "security_score": 0.95,
                "timestamp": "2025-10-23T00:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).expect("Failed to create adapter");
    let metrics = adapter.collect_metrics().await.expect("Failed to collect metrics");

    assert_eq!(metrics.active_sessions, 50);
    assert_eq!(metrics.failed_auth_attempts, 10);
    assert_eq!(metrics.blocked_ips, 2);
    assert!((metrics.security_score - 0.95).abs() < 0.01);
}

#[tokio::test]
async fn test_beardog_verify_auth_success() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/auth/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#""Authorized""#)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).expect("Failed to create adapter");
    let result = adapter.verify_auth("valid_token").await.expect("Failed to verify auth");

    assert_eq!(result, songbird_universal::adapters::security::AuthResult::Authorized);
}

#[tokio::test]
async fn test_beardog_verify_auth_unauthorized() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server.mock("POST", "/auth/verify").with_status(401).create_async().await;

    let adapter = SecurityAdapter::new(server.url()).expect("Failed to create adapter");
    let result = adapter.verify_auth("invalid_token").await.expect("Failed to verify auth");

    assert_eq!(result, songbird_universal::adapters::security::AuthResult::Unauthorized);
}

#[tokio::test]
async fn test_beardog_check_health() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "active_sessions": 100,
                "failed_auth_attempts": 5,
                "blocked_ips": 1,
                "security_score": 0.98,
                "timestamp": "2025-10-23T00:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).expect("Failed to create adapter");
    let health = adapter.check_health().await.expect("Failed to check health");

    assert_eq!(health, songbird_universal::adapters::security::SecurityHealth::Healthy);
}

// ============================================================================
// NESTGATE STORAGE ADAPTER TESTS
// ============================================================================

#[tokio::test]
async fn test_nestgate_collect_metrics_success() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "total_capacity_bytes": 1000000000000,
                "used_bytes": 250000000000,
                "available_bytes": 750000000000,
                "object_count": 1500,
                "avg_read_latency_ms": 15.0,
                "avg_write_latency_ms": 25.0,
                "timestamp": "2025-10-23T00:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).expect("Failed to create adapter");
    let metrics = adapter.collect_metrics().await.expect("Failed to collect metrics");

    assert_eq!(metrics.total_capacity_bytes, 1_000_000_000_000);
    assert_eq!(metrics.used_bytes, 250_000_000_000);
    assert_eq!(metrics.object_count, 1500);
    assert!((metrics.avg_read_latency_ms - 15.0).abs() < 0.1);
    assert!((metrics.avg_write_latency_ms - 25.0).abs() < 0.1);
}

#[tokio::test]
async fn test_nestgate_check_health() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "total_capacity_bytes": 1000000000000,
                "used_bytes": 500000000000,
                "available_bytes": 500000000000,
                "object_count": 10000,
                "avg_read_latency_ms": 20.0,
                "avg_write_latency_ms": 30.0,
                "timestamp": "2025-10-23T00:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).expect("Failed to create adapter");
    let health = adapter.check_health().await.expect("Failed to check health");

    // 50% usage should be healthy
    assert_eq!(health, songbird_universal::adapters::storage::StorageHealth::Healthy);
}

#[tokio::test]
async fn test_nestgate_high_latency_detection() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "total_capacity_bytes": 1000000000000,
                "used_bytes": 900000000000,
                "available_bytes": 100000000000,
                "object_count": 50000,
                "avg_read_latency_ms": 150.0,
                "avg_write_latency_ms": 250.0,
                "timestamp": "2025-10-23T00:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).expect("Failed to create adapter");
    let health = adapter.check_health().await.expect("Failed to check health");

    // 90% usage + high latency should be warning or critical
    assert_ne!(health, songbird_universal::adapters::storage::StorageHealth::Healthy);
}

// ============================================================================
// SQUIRREL AI ADAPTER TESTS
// ============================================================================

#[tokio::test]
async fn test_squirrel_collect_metrics_success() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "active_models": 3,
                "total_requests": 1000,
                "avg_latency_ms": 250.0,
                "accuracy_score": 0.92,
                "gpu_utilization_percent": 75.0,
                "timestamp": "2025-10-23T00:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).expect("Failed to create adapter");
    let metrics = adapter.collect_metrics().await.expect("Failed to collect metrics");

    assert_eq!(metrics.active_models, 3);
    assert_eq!(metrics.total_requests, 1000);
    assert!((metrics.avg_latency_ms - 250.0).abs() < 0.1);
    assert!((metrics.accuracy_score - 0.92).abs() < 0.01);
    assert!((metrics.gpu_utilization_percent - 75.0).abs() < 0.1);
}

#[tokio::test]
async fn test_squirrel_check_health() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "active_models": 5,
                "total_requests": 5000,
                "avg_latency_ms": 500.0,
                "accuracy_score": 0.95,
                "gpu_utilization_percent": 60.0,
                "timestamp": "2025-10-23T00:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).expect("Failed to create adapter");
    let health = adapter.check_health().await.expect("Failed to check health");

    // 60% GPU, 500ms latency should be healthy
    assert_eq!(health, songbird_universal::adapters::ai::AIHealth::Healthy);
}

#[tokio::test]
async fn test_squirrel_high_gpu_load() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "active_models": 10,
                "total_requests": 50000,
                "avg_latency_ms": 1500.0,
                "accuracy_score": 0.88,
                "gpu_utilization_percent": 99.0,
                "timestamp": "2025-10-23T00:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).expect("Failed to create adapter");
    let health = adapter.check_health().await.expect("Failed to check health");

    // 99% GPU + high latency should be overloaded
    assert_eq!(health, songbird_universal::adapters::ai::AIHealth::Overloaded);
}

// ============================================================================
// CROSS-ADAPTER ERROR HANDLING TESTS
// ============================================================================

#[tokio::test]
async fn test_all_adapters_handle_500_error() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(500)
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let toadstool = ComputeAdapter::new(server.url()).expect("Failed to create ToadStool adapter");
    let beardog = SecurityAdapter::new(server.url()).expect("Failed to create BearDog adapter");
    let nestgate = StorageAdapter::new(server.url()).expect("Failed to create NestGate adapter");
    let squirrel = AIAdapter::new(server.url()).expect("Failed to create Squirrel adapter");

    // All should return errors
    assert!(toadstool.collect_metrics().await.is_err());
    assert!(beardog.collect_metrics().await.is_err());
    assert!(nestgate.collect_metrics().await.is_err());
    assert!(squirrel.collect_metrics().await.is_err());
}

#[tokio::test]
async fn test_all_adapters_handle_invalid_json() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", mockito::Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("{invalid json}")
        .create_async()
        .await;

    let toadstool = ComputeAdapter::new(server.url()).expect("Failed to create ToadStool adapter");
    let beardog = SecurityAdapter::new(server.url()).expect("Failed to create BearDog adapter");
    let nestgate = StorageAdapter::new(server.url()).expect("Failed to create NestGate adapter");
    let squirrel = AIAdapter::new(server.url()).expect("Failed to create Squirrel adapter");

    // All should return parse errors
    assert!(toadstool.collect_metrics().await.is_err());
    assert!(beardog.collect_metrics().await.is_err());
    assert!(nestgate.collect_metrics().await.is_err());
    assert!(squirrel.collect_metrics().await.is_err());
}

// ============================================================================
// ADAPTER CONCURRENCY TESTS
// ============================================================================

#[tokio::test]
async fn test_concurrent_adapter_operations() {
    let mut server = mockito::Server::new_async().await;

    let _mock_compute = server
        .mock("GET", "/metrics/compute")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "cpu_usage_percent": 50.0,
                "memory_usage_bytes": 3000000000,
                "memory_available_bytes": 5000000000,
                "active_containers": 7,
                "queued_jobs": 3,
                "performance_score": 0.8,
                "timestamp": "2025-10-23T00:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let _mock_security = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "active_sessions": 75,
                "failed_auth_attempts": 15,
                "blocked_ips": 3,
                "security_score": 0.9,
                "timestamp": "2025-10-23T00:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let toadstool = ComputeAdapter::new(server.url()).expect("Failed to create ToadStool adapter");
    let beardog = SecurityAdapter::new(server.url()).expect("Failed to create BearDog adapter");

    // Run both adapters concurrently
    let (compute_result, security_result) =
        tokio::join!(toadstool.collect_metrics(), beardog.collect_metrics());

    assert!(compute_result.is_ok());
    assert!(security_result.is_ok());

    let compute_metrics = compute_result.unwrap();
    let security_metrics = security_result.unwrap();

    assert_eq!(compute_metrics.active_containers, 7);
    assert_eq!(security_metrics.active_sessions, 75);
}
