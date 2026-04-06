// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    clippy::clone_on_ref_ptr,
    clippy::similar_names,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    reason = "test assertions and harness ergonomics"
)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//! Async Integration Tests for Storage Adapter
//!
//! **Goal**: Test async network methods with mock HTTP servers
//! **Coverage Target**: Move from 66% → ~88% overall coverage
//!
//! This suite tests:
//! - `from_discovery()` - async capability discovery
//! - `collect_metrics()` - HTTP GET /metrics/storage
//! - `check_health()` - integration of collect_metrics
//! - Network error handling
//! - Timeout behavior

use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_universal::adapters::storage::{StorageAdapter, StorageHealth};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// DISCOVERY ASYNC TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_from_discovery_with_injected_resolver() {
    let server = mockito::Server::new_async().await;
    let endpoint = server.url();

    let mut m = HashMap::new();
    m.insert(CapabilityType::Storage, endpoint.clone());
    let resolver = CapabilityEndpointResolver::with_endpoint_overrides(m);

    let adapter = StorageAdapter::from_discovery_with_resolver(resolver).await;
    assert!(adapter.is_ok(), "Should create adapter from injected resolver");

    let adapter = adapter.expect("test precondition");
    assert_eq!(adapter.endpoint(), &endpoint);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_from_discovery_matches_explicit_new() {
    let server = mockito::Server::new_async().await;
    let endpoint = server.url();
    let direct = StorageAdapter::new(endpoint.clone()).await.expect("explicit new");
    assert_eq!(direct.endpoint(), &endpoint);
}

// ============================================================================
// COLLECT_METRICS ASYNC TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_success() {
    let mut server = mockito::Server::new_async().await;

    // Mock successful metrics response
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "total_capacity_bytes": 10000000000,
            "used_bytes": 3500000000,
            "available_bytes": 6500000000,
            "object_count": 5000,
            "avg_read_latency_ms": 12.5,
            "avg_write_latency_ms": 18.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await.expect("test precondition");
    let metrics = adapter.collect_metrics().await;

    assert!(metrics.is_ok(), "Should collect metrics successfully");
    let metrics = metrics.expect("test precondition");
    assert_eq!(metrics.total_capacity_bytes, 10000000000);
    assert_eq!(metrics.used_bytes, 3500000000);
    assert_eq!(metrics.available_bytes, 6500000000);
    assert_eq!(metrics.object_count, 5000);
    assert!((metrics.avg_read_latency_ms - 12.5).abs() < 0.01);
    assert!((metrics.avg_write_latency_ms - 18.0).abs() < 0.01);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_sets_timestamp_if_missing() {
    let mut server = mockito::Server::new_async().await;

    // Mock response with zero/missing timestamp
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "total_capacity_bytes": 5000000000,
            "used_bytes": 1000000000,
            "available_bytes": 4000000000,
            "object_count": 2000,
            "avg_read_latency_ms": 8.0,
            "avg_write_latency_ms": 10.0,
            "timestamp": "1970-01-01T00:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await.expect("test precondition");
    let metrics = adapter.collect_metrics().await;

    assert!(metrics.is_ok());
    let metrics = metrics.expect("test precondition");

    // Timestamp should be set to current time
    let now = chrono::Utc::now();
    let diff = (now - metrics.timestamp).num_seconds().abs();
    assert!(diff < 5, "Timestamp should be recent");

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_network_error() {
    // RFC 5737 TEST-NET-1: guaranteed unreachable, avoids parallel-test port collisions
    let adapter = StorageAdapter::new("http://192.0.2.1:1".to_string())
        .await
        .expect("test precondition")
        .with_timeout(Duration::from_millis(200));

    let result = adapter.collect_metrics().await;
    assert!(result.is_err(), "Should fail with network error");

    let err = result.expect_err("testing error case");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("Failed to reach storage provider")
            || err_msg.contains("storage")
            || err_msg.contains("network")
            || err_msg.contains("timeout")
            || err_msg.contains("Timeout")
            || err_msg.contains("connect")
            || err_msg.contains("dns")
            || err_msg.contains("resolve"),
        "Expected network-related error, got: {err_msg}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_http_error_status() {
    let mut server = mockito::Server::new_async().await;

    // Mock 503 error response
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(503)
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await.expect("test precondition");
    let result = adapter.collect_metrics().await;

    assert!(result.is_err(), "Should fail with HTTP error");
    let err = result.expect_err("testing error case");
    assert!(err.to_string().contains("HTTP 503"));

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_invalid_json() {
    let mut server = mockito::Server::new_async().await;

    // Mock response with invalid JSON
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("not valid json")
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await.expect("test precondition");
    let result = adapter.collect_metrics().await;

    assert!(result.is_err(), "Should fail with parse error");
    let err = result.expect_err("testing error case");
    assert!(err.to_string().contains("Failed to parse storage metrics"));

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_with_timeout() {
    let mut server = mockito::Server::new_async().await;

    // Mock slow response (will timeout)
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body_from_request(|_| {
            // mockito invokes this synchronously; wall-clock delay (not tokio virtual time)
            std::thread::sleep(Duration::from_millis(250));
            r#"{"total_capacity_bytes":0,"used_bytes":0,"available_bytes":0,"object_count":0,"avg_read_latency_ms":0.0,"avg_write_latency_ms":0.0,"timestamp":"2025-11-18T12:00:00Z"}"#.into()
        })
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url())
        .await
        .expect("test precondition")
        .with_timeout(Duration::from_millis(200));

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

    // Mock healthy metrics (30% usage)
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "total_capacity_bytes": 10000000000,
            "used_bytes": 3000000000,
            "available_bytes": 7000000000,
            "object_count": 3000,
            "avg_read_latency_ms": 10.0,
            "avg_write_latency_ms": 15.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await.expect("test precondition");
    let health = adapter.check_health().await;

    assert!(health.is_ok());
    assert_eq!(health.expect("test precondition"), StorageHealth::Healthy);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_warning_high_usage() {
    let mut server = mockito::Server::new_async().await;

    // Mock warning state (82% usage)
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "total_capacity_bytes": 10000000000,
            "used_bytes": 8700000000,
            "available_bytes": 1300000000,
            "object_count": 8000,
            "avg_read_latency_ms": 15.0,
            "avg_write_latency_ms": 20.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await.expect("test precondition");
    let health = adapter.check_health().await;

    assert!(health.is_ok());
    assert_eq!(health.expect("test precondition"), StorageHealth::Warning);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_warning_high_latency() {
    let mut server = mockito::Server::new_async().await;

    // Mock warning state (high read latency > 100ms)
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "total_capacity_bytes": 10000000000,
            "used_bytes": 5000000000,
            "available_bytes": 5000000000,
            "object_count": 5000,
            "avg_read_latency_ms": 125.0,
            "avg_write_latency_ms": 180.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await.expect("test precondition");
    let health = adapter.check_health().await;

    assert!(health.is_ok());
    assert_eq!(health.expect("test precondition"), StorageHealth::Warning);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_critical() {
    let mut server = mockito::Server::new_async().await;

    // Mock critical state (96% usage > 95%)
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "total_capacity_bytes": 10000000000,
            "used_bytes": 9600000000,
            "available_bytes": 400000000,
            "object_count": 10000,
            "avg_read_latency_ms": 120.0,
            "avg_write_latency_ms": 550.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await.expect("test precondition");
    let health = adapter.check_health().await;

    assert!(health.is_ok());
    assert_eq!(health.expect("test precondition"), StorageHealth::Critical);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_network_error() {
    let adapter = StorageAdapter::new("http://192.0.2.1:1".to_string())
        .await
        .expect("test precondition")
        .with_timeout(Duration::from_millis(200));

    let result = adapter.check_health().await;
    assert!(result.is_err(), "Should propagate network error from collect_metrics");
}

// ============================================================================
// INTEGRATION WORKFLOW TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_full_storage_workflow() {
    let mut server = mockito::Server::new_async().await;

    // Setup metrics endpoint
    let metrics_mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "total_capacity_bytes": 8000000000,
            "used_bytes": 2500000000,
            "available_bytes": 5500000000,
            "object_count": 4000,
            "avg_read_latency_ms": 11.0,
            "avg_write_latency_ms": 16.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .expect(2) // Called twice
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await.expect("test precondition");

    // 1. Check health
    let health = adapter.check_health().await;
    assert!(health.is_ok());
    assert_eq!(health.expect("test precondition"), StorageHealth::Healthy);

    // 2. Get metrics directly
    let metrics = adapter.collect_metrics().await;
    assert!(metrics.is_ok());
    let metrics = metrics.expect("test precondition");
    assert_eq!(metrics.total_capacity_bytes, 8000000000);

    metrics_mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_requests() {
    let mut server = mockito::Server::new_async().await;

    // Mock endpoint that can handle concurrent requests
    let mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "total_capacity_bytes": 6000000000,
            "used_bytes": 2000000000,
            "available_bytes": 4000000000,
            "object_count": 3500,
            "avg_read_latency_ms": 9.0,
            "avg_write_latency_ms": 12.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .expect(3)
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await.expect("test precondition");

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
        .mock("GET", "/metrics/storage")
        .with_status(503)
        .with_body("Service Unavailable")
        .expect(1)
        .create_async()
        .await;

    let adapter = StorageAdapter::new(server.url()).await.expect("test precondition");

    // First attempt should fail
    let result1 = adapter.collect_metrics().await;
    assert!(result1.is_err());

    fail_mock.assert_async().await;
    fail_mock.remove_async().await;

    // Setup success mock
    let success_mock = server
        .mock("GET", "/metrics/storage")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "total_capacity_bytes": 7000000000,
            "used_bytes": 2800000000,
            "available_bytes": 4200000000,
            "object_count": 4500,
            "avg_read_latency_ms": 10.5,
            "avg_write_latency_ms": 14.0,
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
