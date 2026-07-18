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

//! Async Integration Tests for AI Adapter
//!
//! **Goal**: Test async network methods with mock HTTP servers
//! **Coverage Target**: Move from 64% → ~88% overall coverage
//!
//! This suite tests:
//! - `from_discovery()` - async capability discovery
//! - `collect_metrics()` - HTTP GET /metrics/ai
//! - `check_health()` - integration of `collect_metrics`
//! - Network error handling
//! - Timeout behavior

use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_universal::adapters::ai::{AIAdapter, AIHealth};
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
    m.insert(CapabilityType::Ai, endpoint.clone());
    let resolver = CapabilityEndpointResolver::with_endpoint_overrides(m);

    let adapter = AIAdapter::from_discovery_with_resolver(resolver).await;
    assert!(adapter.is_ok(), "Should create adapter from injected resolver");

    let adapter = adapter.expect("test precondition");
    assert_eq!(adapter.endpoint(), &endpoint);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_from_discovery_matches_explicit_new() {
    let server = mockito::Server::new_async().await;
    let endpoint = server.url();
    let direct = AIAdapter::new(endpoint.clone()).await.expect("explicit new");
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
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_models": 8,
            "total_requests": 150000,
            "avg_latency_ms": 45.5,
            "accuracy_score": 0.94,
            "gpu_utilization_percent": 72.3,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).await.expect("test precondition");
    let metrics = adapter.collect_metrics().await;

    assert!(metrics.is_ok(), "Should collect metrics successfully");
    let metrics = metrics.expect("test precondition");
    assert_eq!(metrics.active_models, 8);
    assert_eq!(metrics.total_requests, 150_000);
    assert!((metrics.avg_latency_ms - 45.5).abs() < 0.01);
    assert!((metrics.accuracy_score - 0.94).abs() < 0.01);
    assert!((metrics.gpu_utilization_percent - 72.3).abs() < 0.01);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_sets_timestamp_if_missing() {
    let mut server = mockito::Server::new_async().await;

    // Mock response with zero/missing timestamp
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_models": 4,
            "total_requests": 50000,
            "avg_latency_ms": 30.0,
            "accuracy_score": 0.90,
            "gpu_utilization_percent": 55.0,
            "timestamp": "1970-01-01T00:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).await.expect("test precondition");
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
    let adapter = AIAdapter::new("http://192.0.2.1:1".to_string())
        .await
        .expect("test precondition")
        .with_timeout(std::time::Duration::from_millis(200));

    let result = adapter.collect_metrics().await;
    assert!(result.is_err(), "Should fail with network error");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_http_error_status() {
    let mut server = mockito::Server::new_async().await;

    // Mock 503 error response
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(503)
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).await.expect("test precondition");
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
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("not valid json")
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).await.expect("test precondition");
    let result = adapter.collect_metrics().await;

    assert!(result.is_err(), "Should fail with parse error");
    let err = result.expect_err("testing error case");
    assert!(err.to_string().contains("Failed to parse AI metrics"));

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_with_timeout() {
    let mut server = mockito::Server::new_async().await;

    // Mock slow response (will timeout). mockito runs the body callback on a worker thread with
    // real HTTP I/O — wall-clock delay is required (tokio `start_paused` does not apply here).
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body_from_request(|_| {
            std::thread::sleep(Duration::from_millis(250));
            r#"{"active_models":0,"total_requests":0,"avg_latency_ms":0.0,"accuracy_score":1.0,"gpu_utilization_percent":0.0,"timestamp":"2025-11-18T12:00:00Z"}"#.into()
        })
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url())
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

    // Mock healthy metrics (low latency, moderate GPU usage)
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_models": 5,
            "total_requests": 100000,
            "avg_latency_ms": 80.0,
            "accuracy_score": 0.92,
            "gpu_utilization_percent": 65.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).await.expect("test precondition");
    let health = adapter.check_health().await;

    assert!(health.is_ok());
    assert_eq!(health.expect("test precondition"), AIHealth::Healthy);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_degraded_high_latency() {
    let mut server = mockito::Server::new_async().await;

    // Mock degraded state (high latency > 1000ms)
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_models": 12,
            "total_requests": 250000,
            "avg_latency_ms": 1200.0,
            "accuracy_score": 0.88,
            "gpu_utilization_percent": 75.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).await.expect("test precondition");
    let health = adapter.check_health().await;

    assert!(health.is_ok());
    assert_eq!(health.expect("test precondition"), AIHealth::Degraded);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_degraded_high_gpu() {
    let mut server = mockito::Server::new_async().await;

    // Mock degraded state (high GPU utilization)
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_models": 10,
            "total_requests": 200000,
            "avg_latency_ms": 90.0,
            "accuracy_score": 0.91,
            "gpu_utilization_percent": 91.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).await.expect("test precondition");
    let health = adapter.check_health().await;

    assert!(health.is_ok());
    assert_eq!(health.expect("test precondition"), AIHealth::Degraded);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_overloaded() {
    let mut server = mockito::Server::new_async().await;

    // Mock overloaded state
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_models": 25,
            "total_requests": 500000,
            "avg_latency_ms": 320.0,
            "accuracy_score": 0.70,
            "gpu_utilization_percent": 98.5,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).await.expect("test precondition");
    let health = adapter.check_health().await;

    assert!(health.is_ok());
    assert_eq!(health.expect("test precondition"), AIHealth::Overloaded);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_network_error() {
    let adapter = AIAdapter::new("http://192.0.2.1:1".to_string())
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
async fn test_full_ai_workflow() {
    let mut server = mockito::Server::new_async().await;

    // Setup metrics endpoint
    let metrics_mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_models": 6,
            "total_requests": 120000,
            "avg_latency_ms": 75.0,
            "accuracy_score": 0.93,
            "gpu_utilization_percent": 70.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .expect(2) // Called twice
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).await.expect("test precondition");

    // 1. Check health
    let health = adapter.check_health().await;
    assert!(health.is_ok());
    assert_eq!(health.expect("test precondition"), AIHealth::Healthy);

    // 2. Get metrics directly
    let metrics = adapter.collect_metrics().await;
    assert!(metrics.is_ok());
    let metrics = metrics.expect("test precondition");
    assert_eq!(metrics.active_models, 6);

    metrics_mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_requests() {
    let mut server = mockito::Server::new_async().await;

    // Mock endpoint that can handle concurrent requests
    let mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_models": 7,
            "total_requests": 100000,
            "avg_latency_ms": 60.0,
            "accuracy_score": 0.92,
            "gpu_utilization_percent": 68.0,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .expect(3)
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).await.expect("test precondition");

    // Fire off 3 concurrent requests
    let futures =
        vec![adapter.collect_metrics(), adapter.collect_metrics(), adapter.collect_metrics()];

    let results = futures_util::future::join_all(futures).await;

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
        .mock("GET", "/metrics/ai")
        .with_status(503)
        .with_body("Service Unavailable")
        .expect(1)
        .create_async()
        .await;

    let adapter = AIAdapter::new(server.url()).await.expect("test precondition");

    // First attempt should fail
    let result1 = adapter.collect_metrics().await;
    assert!(result1.is_err());

    fail_mock.assert_async().await;
    fail_mock.remove_async().await;

    // Setup success mock
    let success_mock = server
        .mock("GET", "/metrics/ai")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_models": 5,
            "total_requests": 90000,
            "avg_latency_ms": 65.0,
            "accuracy_score": 0.91,
            "gpu_utilization_percent": 62.0,
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
