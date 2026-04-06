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

//! HTTP Mock Tests for Security Adapter
//!
//! **Purpose**: Test actual async HTTP methods with mocked responses.
//! This provides coverage for the HTTP client logic that was previously untested (85% uncovered).
//!
//! These tests use mockito to create mock HTTP servers and verify:
//! - HTTP request formatting
//! - Response parsing
//! - Error handling
//! - Timeout behavior
//! - Status code handling

use songbird_types::SongbirdResult;
use songbird_universal::adapters::security::{AuthResult, SecurityAdapter, SecurityHealth};
use std::time::Duration;

// ============================================================================
// COLLECT_METRICS TESTS
// ============================================================================

#[tokio::test]
async fn test_collect_metrics_success() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 50,
            "failed_auth_attempts": 10,
            "blocked_ips": 2,
            "security_score": 0.95,
            "timestamp": "2025-11-18T00:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let metrics = adapter.collect_metrics().await?;

    mock.assert_async().await;
    assert_eq!(metrics.active_sessions, 50);
    assert_eq!(metrics.failed_auth_attempts, 10);
    assert_eq!(metrics.blocked_ips, 2);
    assert_eq!(metrics.security_score, 0.95);

    Ok(())
}

#[tokio::test]
async fn test_collect_metrics_network_error() {
    let adapter = SecurityAdapter::new("http://nonexistent-host-12345:9999".to_string())
        .await
        .expect("test precondition")
        .with_timeout(Duration::from_millis(100));

    let result = adapter.collect_metrics().await;
    assert!(result.is_err());

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

#[tokio::test]
async fn test_collect_metrics_http_error() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server.mock("GET", "/metrics/security").with_status(500).create_async().await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let result = adapter.collect_metrics().await;

    assert!(result.is_err());
    let err = result.expect_err("testing error case");
    assert!(err.to_string().contains("HTTP 500"));

    Ok(())
}

#[tokio::test]
async fn test_collect_metrics_invalid_json() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_body("invalid json{]")
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let result = adapter.collect_metrics().await;

    assert!(result.is_err());
    let err = result.expect_err("testing error case");
    assert!(err.to_string().contains("Failed to parse security metrics"));

    Ok(())
}

#[tokio::test]
async fn test_collect_metrics_missing_timestamp() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 25,
            "failed_auth_attempts": 5,
            "blocked_ips": 1,
            "security_score": 0.85,
            "timestamp": "1970-01-01T00:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let metrics = adapter.collect_metrics().await?;

    // Timestamp should be set to current time when it's zero
    assert!(metrics.timestamp.timestamp() > 0);
    assert_eq!(metrics.active_sessions, 25);

    Ok(())
}

#[tokio::test]
async fn test_collect_metrics_high_values() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 1000,
            "failed_auth_attempts": 150,
            "blocked_ips": 60,
            "security_score": 0.35,
            "timestamp": "2025-11-18T00:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let metrics = adapter.collect_metrics().await?;

    assert!(metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);

    Ok(())
}

// ============================================================================
// VERIFY_AUTH TESTS
// ============================================================================

#[tokio::test]
async fn test_verify_auth_authorized() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/auth/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#""Authorized""#)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let result = adapter.verify_auth("valid_token_123").await?;

    assert_eq!(result, AuthResult::Authorized);

    Ok(())
}

#[tokio::test]
async fn test_verify_auth_unauthorized() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server.mock("POST", "/auth/verify").with_status(401).create_async().await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let result = adapter.verify_auth("invalid_token").await?;

    assert_eq!(result, AuthResult::Unauthorized);

    Ok(())
}

#[tokio::test]
async fn test_verify_auth_expired() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/auth/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#""Expired""#)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let result = adapter.verify_auth("expired_token").await?;

    assert_eq!(result, AuthResult::Expired);

    Ok(())
}

#[tokio::test]
async fn test_verify_auth_invalid() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/auth/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#""Invalid""#)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let result = adapter.verify_auth("malformed_token").await?;

    assert_eq!(result, AuthResult::Invalid);

    Ok(())
}

#[tokio::test]
async fn test_verify_auth_network_error() {
    let adapter = SecurityAdapter::new("http://nonexistent-host-67890:9999".to_string())
        .await
        .expect("test precondition")
        .with_timeout(Duration::from_millis(100));

    let result = adapter.verify_auth("any_token").await;
    assert!(result.is_err());

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

#[tokio::test]
async fn test_verify_auth_request_body() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/auth/verify")
        .match_header("content-type", "application/json")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "token": "test_token_abc"
        })))
        .with_status(200)
        .with_body(r#""Authorized""#)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let result = adapter.verify_auth("test_token_abc").await?;

    assert_eq!(result, AuthResult::Authorized);

    Ok(())
}

#[tokio::test]
async fn test_verify_auth_invalid_json_response() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/auth/verify")
        .with_status(200)
        .with_body("not valid json")
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let result = adapter.verify_auth("token").await;

    assert!(result.is_err());
    let err = result.expect_err("testing error case");
    assert!(err.to_string().contains("Failed to parse auth result"));

    Ok(())
}

// ============================================================================
// CHECK_HEALTH TESTS
// ============================================================================

#[tokio::test]
async fn test_check_health_healthy() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 50,
            "failed_auth_attempts": 5,
            "blocked_ips": 2,
            "security_score": 0.95,
            "timestamp": "2025-11-18T00:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let health = adapter.check_health().await?;

    assert_eq!(health, SecurityHealth::Healthy);

    Ok(())
}

#[tokio::test]
async fn test_check_health_warning() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 100,
            "failed_auth_attempts": 55,
            "blocked_ips": 10,
            "security_score": 0.68,
            "timestamp": "2025-11-18T00:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let health = adapter.check_health().await?;

    assert_eq!(health, SecurityHealth::Warning);

    Ok(())
}

#[tokio::test]
async fn test_check_health_critical() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 200,
            "failed_auth_attempts": 150,
            "blocked_ips": 60,
            "security_score": 0.35,
            "timestamp": "2025-11-18T00:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let health = adapter.check_health().await?;

    assert_eq!(health, SecurityHealth::Critical);

    Ok(())
}

#[tokio::test]
async fn test_check_health_network_failure() {
    let adapter = SecurityAdapter::new("http://nonexistent-host-11111:9999".to_string())
        .await
        .expect("test precondition")
        .with_timeout(Duration::from_millis(100));

    let result = adapter.check_health().await;
    assert!(result.is_err());
}

// ============================================================================
// TIMEOUT TESTS
// ============================================================================

#[tokio::test]
async fn test_collect_metrics_with_custom_timeout() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 30,
            "failed_auth_attempts": 3,
            "blocked_ips": 1,
            "security_score": 0.90,
            "timestamp": "2025-11-18T00:00:00Z"
        }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await?.with_timeout(Duration::from_secs(30));

    let metrics = adapter.collect_metrics().await?;
    assert_eq!(metrics.active_sessions, 30);

    Ok(())
}

// ============================================================================
// MULTIPLE REQUESTS TESTS
// ============================================================================

#[tokio::test]
async fn test_multiple_collect_metrics_calls() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "active_sessions": 40,
            "failed_auth_attempts": 8,
            "blocked_ips": 3,
            "security_score": 0.88,
            "timestamp": "2025-11-18T00:00:00Z"
        }"#,
        )
        .expect(3)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await?;

    // Call multiple times
    for _ in 0..3 {
        let metrics = adapter.collect_metrics().await?;
        assert_eq!(metrics.active_sessions, 40);
    }

    Ok(())
}

#[tokio::test]
async fn test_multiple_verify_auth_calls() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("POST", "/auth/verify")
        .with_status(200)
        .with_body(r#""Authorized""#)
        .expect(5)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).await?;

    // Verify multiple tokens
    for i in 0..5 {
        let result = adapter.verify_auth(&format!("token_{}", i)).await?;
        assert_eq!(result, AuthResult::Authorized);
    }

    Ok(())
}

// ============================================================================
// ERROR STATUS CODE TESTS
// ============================================================================

#[tokio::test]
async fn test_collect_metrics_404() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server.mock("GET", "/metrics/security").with_status(404).create_async().await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let result = adapter.collect_metrics().await;

    assert!(result.is_err());
    let err = result.expect_err("testing error case");
    assert!(err.to_string().contains("HTTP 404"));

    Ok(())
}

#[tokio::test]
async fn test_collect_metrics_503() -> SongbirdResult<()> {
    let mut server = mockito::Server::new_async().await;

    let _mock = server.mock("GET", "/metrics/security").with_status(503).create_async().await;

    let adapter = SecurityAdapter::new(server.url()).await?;
    let result = adapter.collect_metrics().await;

    assert!(result.is_err());
    let err = result.expect_err("testing error case");
    assert!(err.to_string().contains("HTTP 503"));

    Ok(())
}
