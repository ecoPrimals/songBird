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

//! Async Integration Tests for Security Adapter
//!
//! **Goal**: Test async network methods with mock HTTP servers
//! **Coverage Target**: Move from 14.71% → ~85% overall coverage
//!
//! This suite tests:
//! - `from_discovery()` - async capability discovery
//! - `collect_metrics()` - HTTP GET /metrics/security
//! - `verify_auth()` - HTTP POST /auth/verify
//! - `check_health()` - integration of collect_metrics
//! - Network error handling
//! - Timeout behavior
//! - Retry and fallback mechanisms

use songbird_config::capability_endpoints::{CapabilityEndpointResolver, CapabilityType};
use songbird_universal::adapters::security::{AuthResult, SecurityAdapter, SecurityHealth};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// FROM_DISCOVERY ASYNC TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_from_discovery_with_injected_resolver() {
    let server = mockito::Server::new_async().await;
    let endpoint = server.url();

    let mut m = HashMap::new();
    m.insert(CapabilityType::Security, endpoint.clone());
    let resolver = CapabilityEndpointResolver::with_endpoint_overrides(m);

    let adapter = SecurityAdapter::from_discovery_with_resolver(resolver).await;
    assert!(adapter.is_ok(), "Should create adapter from injected resolver");

    let adapter = adapter.expect("test precondition");
    assert_eq!(adapter.endpoint(), &endpoint);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_from_discovery_matches_explicit_new() {
    let server = mockito::Server::new_async().await;
    let endpoint = server.url();
    let direct = SecurityAdapter::new(endpoint.clone()).await.expect("explicit new");
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
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 100,
            "failed_auth_attempts": 5,
            "blocked_ips": 2,
            "security_score": 0.95,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await.expect("test precondition");
    let metrics = adapter.collect_metrics().await;

    assert!(metrics.is_ok(), "Should collect metrics successfully");
    let metrics = metrics.expect("test precondition");
    assert_eq!(metrics.active_sessions, 100);
    assert_eq!(metrics.failed_auth_attempts, 5);
    assert_eq!(metrics.blocked_ips, 2);
    assert!((metrics.security_score - 0.95).abs() < 0.01);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_sets_timestamp_if_missing() {
    let mut server = mockito::Server::new_async().await;

    // Mock response with zero/missing timestamp
    let mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 50,
            "failed_auth_attempts": 3,
            "blocked_ips": 1,
            "security_score": 0.90,
            "timestamp": "1970-01-01T00:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await.expect("test precondition");
    let metrics = adapter.collect_metrics().await;

    assert!(metrics.is_ok());
    let metrics = metrics.expect("test precondition");

    // Timestamp should be set to current time (not epoch zero)
    let now = chrono::Utc::now();
    let diff = (now - metrics.timestamp).num_seconds().abs();
    assert!(diff < 5, "Timestamp should be recent (within 5 seconds)");

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_network_error() {
    // RFC 5737 TEST-NET-1: guaranteed unreachable, avoids parallel-test port collisions
    let adapter = SecurityAdapter::new("http://192.0.2.1:1".to_string())
        .await
        .expect("test precondition")
        .with_timeout(Duration::from_millis(200));

    let result = adapter.collect_metrics().await;
    assert!(result.is_err(), "Should fail with network error");

    let err = result.expect_err("testing error case");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("Failed to reach")
            || err_msg.contains("security")
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

    // Mock 500 error response
    let mock = server
        .mock("GET", "/metrics/security")
        .with_status(500)
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await.expect("test precondition");
    let result = adapter.collect_metrics().await;

    assert!(result.is_err(), "Should fail with HTTP error");
    let err = result.expect_err("testing error case");
    assert!(err.to_string().contains("HTTP 500"));

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_invalid_json() {
    let mut server = mockito::Server::new_async().await;

    // Mock response with invalid JSON
    let mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("not valid json")
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await.expect("test precondition");
    let result = adapter.collect_metrics().await;

    assert!(result.is_err(), "Should fail with parse error");
    let err = result.expect_err("testing error case");
    assert!(err.to_string().contains("Failed to parse security metrics"));

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_collect_metrics_with_timeout() {
    let mut server = mockito::Server::new_async().await;

    // Mock slow response (will timeout)
    let mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body_from_request(|_| {
            // Note: mockito's with_body_from_request runs in sync context
            // Using thread::sleep here is necessary for mockito compatibility
            // The key improvement is the reduced timeout (100ms vs 1000ms)
            std::thread::sleep(Duration::from_millis(150)); // Just over timeout
            r#"{"active_sessions":0,"failed_auth_attempts":0,"blocked_ips":0,"security_score":1.0,"timestamp":"2025-11-18T12:00:00Z"}"#.into()
        })
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url())
        .await
        .expect("test precondition")
        .with_timeout(Duration::from_millis(100)); // Very short timeout

    let result = adapter.collect_metrics().await;
    assert!(result.is_err(), "Should timeout");

    mock.assert_async().await;
}

// ============================================================================
// VERIFY_AUTH ASYNC TESTS
// ============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_verify_auth_authorized() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/auth/verify")
        .match_header("content-type", "application/json")
        .match_body(mockito::Matcher::JsonString(r#"{"token":"valid-token"}"#.to_string()))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#""Authorized""#)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await.expect("test precondition");
    let result = adapter.verify_auth("valid-token").await;

    assert!(result.is_ok());
    assert_eq!(result.expect("test precondition"), AuthResult::Authorized);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_verify_auth_unauthorized() {
    let mut server = mockito::Server::new_async().await;

    // Return 401 for invalid token
    let mock = server.mock("POST", "/auth/verify").with_status(401).create_async().await;

    let adapter = SecurityAdapter::new(server.url()).await.expect("test precondition");
    let result = adapter.verify_auth("invalid-token").await;

    assert!(result.is_ok());
    assert_eq!(result.expect("test precondition"), AuthResult::Unauthorized);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_verify_auth_expired() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/auth/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#""Expired""#)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await.expect("test precondition");
    let result = adapter.verify_auth("expired-token").await;

    assert!(result.is_ok());
    assert_eq!(result.expect("test precondition"), AuthResult::Expired);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_verify_auth_invalid() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/auth/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#""Invalid""#)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await.expect("test precondition");
    let result = adapter.verify_auth("malformed-token").await;

    assert!(result.is_ok());
    assert_eq!(result.expect("test precondition"), AuthResult::Invalid);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_verify_auth_network_error() {
    let adapter = SecurityAdapter::new("http://192.0.2.1:1".to_string())
        .await
        .expect("test precondition")
        .with_timeout(Duration::from_millis(200));

    let result = adapter.verify_auth("some-token").await;
    assert!(result.is_err(), "Should fail with network error");

    let err = result.expect_err("testing error case");
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("Auth verification failed")
            || err_msg.contains("Failed to reach")
            || err_msg.contains("security")
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
async fn test_verify_auth_invalid_json_response() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("POST", "/auth/verify")
        .with_status(200)
        .with_body("not valid json")
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await.expect("test precondition");
    let result = adapter.verify_auth("some-token").await;

    assert!(result.is_err(), "Should fail with parse error");
    let err = result.expect_err("testing error case");
    assert!(err.to_string().contains("Failed to parse auth result"));

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
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 100,
            "failed_auth_attempts": 3,
            "blocked_ips": 1,
            "security_score": 0.95,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await.expect("test precondition");
    let health = adapter.check_health().await;

    assert!(health.is_ok());
    assert_eq!(health.expect("test precondition"), SecurityHealth::Healthy);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_warning() {
    let mut server = mockito::Server::new_async().await;

    // Mock metrics showing warning state (score between 0.5 and 0.8)
    let mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 50,
            "failed_auth_attempts": 35,
            "blocked_ips": 8,
            "security_score": 0.65,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await.expect("test precondition");
    let health = adapter.check_health().await;

    assert!(health.is_ok());
    assert_eq!(health.expect("test precondition"), SecurityHealth::Warning);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_critical() {
    let mut server = mockito::Server::new_async().await;

    // Mock metrics showing critical state (many failed attempts)
    let mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 50,
            "failed_auth_attempts": 150,
            "blocked_ips": 25,
            "security_score": 0.45,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await.expect("test precondition");
    let health = adapter.check_health().await;

    assert!(health.is_ok());
    assert_eq!(health.expect("test precondition"), SecurityHealth::Critical);

    mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_check_health_network_error() {
    let adapter = SecurityAdapter::new("http://192.0.2.1:1".to_string())
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
async fn test_full_security_workflow() {
    let mut server = mockito::Server::new_async().await;

    // Setup all endpoints
    let metrics_mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 80,
            "failed_auth_attempts": 5,
            "blocked_ips": 2,
            "security_score": 0.92,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .expect(2) // Called twice
        .create_async()
        .await;

    let auth_mock = server
        .mock("POST", "/auth/verify")
        .with_status(200)
        .with_body(r#""Authorized""#)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await.expect("test precondition");

    // 1. Check health
    let health = adapter.check_health().await;
    assert!(health.is_ok());
    assert_eq!(health.expect("test precondition"), SecurityHealth::Healthy);

    // 2. Verify auth
    let auth = adapter.verify_auth("test-token").await;
    assert!(auth.is_ok());
    assert_eq!(auth.expect("test precondition"), AuthResult::Authorized);

    // 3. Collect metrics
    let metrics = adapter.collect_metrics().await;
    assert!(metrics.is_ok());
    let metrics = metrics.expect("test precondition");
    assert_eq!(metrics.active_sessions, 80);

    metrics_mock.assert_async().await;
    auth_mock.assert_async().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_requests() {
    let mut server = mockito::Server::new_async().await;

    // Mock endpoint that can handle concurrent requests
    let mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 100,
            "failed_auth_attempts": 5,
            "blocked_ips": 2,
            "security_score": 0.95,
            "timestamp": "2025-11-18T12:00:00Z"
        }"#,
        )
        .expect(3)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await.expect("test precondition");

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

    // First call fails, second succeeds
    let fail_mock = server
        .mock("GET", "/metrics/security")
        .with_status(503)
        .with_body("Service Unavailable")
        .expect(1)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await.expect("test precondition");

    // First attempt should fail
    let result1 = adapter.collect_metrics().await;
    assert!(result1.is_err());

    fail_mock.assert_async().await;
    fail_mock.remove_async().await;

    // Setup success mock
    let success_mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 50,
            "failed_auth_attempts": 2,
            "blocked_ips": 1,
            "security_score": 0.93,
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
