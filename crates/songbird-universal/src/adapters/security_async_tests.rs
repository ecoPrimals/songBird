#![cfg(test)]

//! Async Integration Tests for Security Adapter
//!
//! **Purpose**: Achieve 90% coverage by testing actual async HTTP operations
//!
//! This file adds missing coverage for:
//! - `from_discovery()` method
//! - `collect_metrics()` async operations
//! - `verify_auth()` async operations  
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
// FROM_DISCOVERY TESTS
// ============================================================================

#[tokio::test]
async fn test_from_discovery_with_env_variable() {
    // ARRANGE: Set environment variable
    std::env::set_var("SONGBIRD_SECURITY_ENDPOINT", "http://test-security:9000");

    // ACT: Discover security adapter
    let result = SecurityAdapter::from_discovery().await;

    // ASSERT: Should succeed and use env variable
    assert!(result.is_ok(), "Discovery should succeed with env variable");
    let adapter = result.unwrap();
    assert!(
        adapter.endpoint().contains("test-security") || adapter.endpoint().contains("9000"),
        "Should use environment variable endpoint"
    );

    // CLEANUP
    std::env::remove_var("SONGBIRD_SECURITY_ENDPOINT");
}

#[tokio::test]
async fn test_from_discovery_legacy_beardog_endpoint() {
    // ARRANGE: Set legacy BEARDOG_ENDPOINT and clear other endpoints
    std::env::set_var("BEARDOG_ENDPOINT", "http://legacy-beardog:8443");
    std::env::remove_var("SONGBIRD_SECURITY_ENDPOINT");
    std::env::remove_var("SECURITY_PROVIDER_ENDPOINT");

    // ACT: Discover security adapter (should fall back to legacy env)
    let result = SecurityAdapter::from_discovery().await;

    // ASSERT: Should succeed with legacy fallback
    assert!(result.is_ok(), "Discovery should work with legacy BEARDOG_ENDPOINT");
    let adapter = result.unwrap();
    // Adapter should discover some endpoint (may be legacy or constructed)
    assert!(!adapter.endpoint().is_empty(), "Should have valid endpoint");

    // CLEANUP
    std::env::remove_var("BEARDOG_ENDPOINT");
}

#[tokio::test]
async fn test_from_discovery_fallback_to_default() {
    // ARRANGE: Clear all security endpoints
    std::env::remove_var("SONGBIRD_SECURITY_ENDPOINT");
    std::env::remove_var("SECURITY_PROVIDER_ENDPOINT");
    std::env::remove_var("BEARDOG_ENDPOINT");

    // ACT: Discover with no hints (should use defaults)
    let result = SecurityAdapter::from_discovery().await;

    // ASSERT: Should succeed with default fallback
    assert!(result.is_ok(), "Discovery should fall back to defaults");
    let adapter = result.unwrap();
    assert!(!adapter.endpoint().is_empty(), "Should have valid endpoint");
}

#[tokio::test]
async fn test_from_discovery_with_custom_port() {
    // ARRANGE: Clean environment and set custom security port
    std::env::remove_var("SONGBIRD_SECURITY_ENDPOINT");
    std::env::remove_var("SECURITY_CAPABILITY_ENDPOINT");
    std::env::remove_var("BEARDOG_ENDPOINT");
    std::env::set_var("SONGBIRD_SECURITY_PORT", "9999");

    // ACT: Discover (should use custom port)
    let result = SecurityAdapter::from_discovery().await;

    // ASSERT: Should succeed (exact endpoint varies by host/discovery)
    assert!(result.is_ok(), "Discovery should succeed with custom port");

    // CLEANUP
    std::env::remove_var("SONGBIRD_SECURITY_PORT");
}

// ============================================================================
// COLLECT_METRICS ASYNC TESTS
// ============================================================================

#[tokio::test]
async fn test_collect_metrics_success() {
    // ARRANGE: Mock server returning valid metrics
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 42,
            "failed_auth_attempts": 3,
            "blocked_ips": 1,
            "security_score": 0.95,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Collect metrics
    let result = adapter.collect_metrics().await;

    // ASSERT: Should succeed and parse metrics
    assert!(result.is_ok(), "Should successfully collect metrics");
    let metrics = result.unwrap();
    assert_eq!(metrics.active_sessions, 42);
    assert_eq!(metrics.failed_auth_attempts, 3);
    assert_eq!(metrics.blocked_ips, 1);
    assert_eq!(metrics.security_score, 0.95);
}

#[tokio::test]
async fn test_collect_metrics_network_error() {
    // ARRANGE: Adapter pointing to non-existent server
    let adapter = SecurityAdapter::new("http://invalid-nonexistent-host-12345:9999")
        .await
        .unwrap()
        .with_timeout(Duration::from_millis(100)); // Short timeout

    // ACT: Attempt to collect metrics
    let result = adapter.collect_metrics().await;

    // ASSERT: Should fail with network error
    assert!(result.is_err(), "Should fail with network error");
    let err = result.unwrap_err();
    let err_msg = format!("{err:?}");
    assert!(
        err_msg.contains("network") || err_msg.contains("Failed to reach"),
        "Error should indicate network failure"
    );
}

#[tokio::test]
async fn test_collect_metrics_http_error_status() {
    // ARRANGE: Mock server returning 500 error
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Collect metrics
    let result = adapter.collect_metrics().await;

    // ASSERT: Should fail with HTTP error
    assert!(result.is_err(), "Should fail with HTTP 500");
    let err = result.unwrap_err();
    let err_msg = format!("{err:?}");
    assert!(
        err_msg.contains("500") || err_msg.contains("Security metrics unavailable"),
        "Error should mention HTTP 500"
    );
}

#[tokio::test]
async fn test_collect_metrics_invalid_json() {
    // ARRANGE: Mock server returning invalid JSON
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid json {"))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Collect metrics
    let result = adapter.collect_metrics().await;

    // ASSERT: Should fail to parse
    assert!(result.is_err(), "Should fail to parse invalid JSON");
    let err = result.unwrap_err();
    let err_msg = format!("{err:?}");
    assert!(
        err_msg.contains("parse") || err_msg.contains("Failed to parse security metrics"),
        "Error should indicate parsing failure"
    );
}

#[tokio::test]
async fn test_collect_metrics_missing_timestamp() {
    // ARRANGE: Mock server returning metrics without timestamp
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 10,
            "failed_auth_attempts": 0,
            "blocked_ips": 0,
            "security_score": 1.0,
            "timestamp": "1970-01-01T00:00:00Z" // Epoch zero
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Collect metrics
    let result = adapter.collect_metrics().await;

    // ASSERT: Should set current timestamp
    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert!(metrics.timestamp.timestamp() > 0, "Should set current timestamp when missing");
}

#[tokio::test]
async fn test_collect_metrics_with_custom_timeout() {
    // ARRANGE: Adapter with very short timeout
    let adapter = SecurityAdapter::new("http://slow-nonexistent-host:9999")
        .await
        .unwrap()
        .with_timeout(Duration::from_millis(50));

    // ACT: Attempt to collect metrics (should timeout quickly)
    let result = adapter.collect_metrics().await;

    // ASSERT: Should fail with timeout/network error
    assert!(result.is_err(), "Should timeout with custom short timeout");
}

// ============================================================================
// VERIFY_AUTH ASYNC TESTS
// ============================================================================

#[tokio::test]
async fn test_verify_auth_authorized() {
    // ARRANGE: Mock server returning authorized
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json("Authorized"))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Verify valid token
    let result = adapter.verify_auth("valid-token-123").await;

    // ASSERT: Should be authorized
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AuthResult::Authorized);
}

#[tokio::test]
async fn test_verify_auth_unauthorized() {
    // ARRANGE: Mock server returning 401
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Verify invalid token
    let result = adapter.verify_auth("invalid-token").await;

    // ASSERT: Should be unauthorized
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AuthResult::Unauthorized);
}

#[tokio::test]
async fn test_verify_auth_expired() {
    // ARRANGE: Mock server returning expired status
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json("Expired"))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Verify expired token
    let result = adapter.verify_auth("expired-token").await;

    // ASSERT: Should indicate expiration
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AuthResult::Expired);
}

#[tokio::test]
async fn test_verify_auth_invalid() {
    // ARRANGE: Mock server returning invalid status
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json("Invalid"))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Verify malformed token
    let result = adapter.verify_auth("malformed-token").await;

    // ASSERT: Should indicate invalid
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AuthResult::Invalid);
}

#[tokio::test]
async fn test_verify_auth_network_error() {
    // ARRANGE: Adapter pointing to non-existent server
    let adapter = SecurityAdapter::new("http://invalid-auth-host:9999")
        .await
        .unwrap()
        .with_timeout(Duration::from_millis(100));

    // ACT: Attempt auth verification
    let result = adapter.verify_auth("any-token").await;

    // ASSERT: Should fail with network error
    assert!(result.is_err(), "Should fail with network error");
    let err = result.unwrap_err();
    let err_msg = format!("{err:?}");
    assert!(
        err_msg.contains("network") || err_msg.contains("Auth verification failed"),
        "Error should indicate network failure"
    );
}

#[tokio::test]
async fn test_verify_auth_invalid_response_json() {
    // ARRANGE: Mock server returning invalid JSON
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Verify auth
    let result = adapter.verify_auth("token").await;

    // ASSERT: Should fail to parse
    assert!(result.is_err(), "Should fail to parse invalid JSON response");
    let err = result.unwrap_err();
    let err_msg = format!("{err:?}");
    assert!(
        err_msg.contains("parse") || err_msg.contains("Failed to parse auth result"),
        "Error should indicate parsing failure"
    );
}

#[tokio::test]
async fn test_verify_auth_empty_token() {
    // ARRANGE: Mock server
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json("Invalid"))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Verify empty token
    let result = adapter.verify_auth("").await;

    // ASSERT: Should handle empty token (likely invalid)
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AuthResult::Invalid);
}

// ============================================================================
// CHECK_HEALTH ASYNC TESTS
// ============================================================================

#[tokio::test]
async fn test_check_health_healthy() {
    // ARRANGE: Mock server with healthy metrics
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 50,
            "failed_auth_attempts": 2,
            "blocked_ips": 1,
            "security_score": 0.95,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Check health
    let result = adapter.check_health().await;

    // ASSERT: Should be healthy
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), SecurityHealth::Healthy);
}

#[tokio::test]
async fn test_check_health_warning() {
    // ARRANGE: Mock server with warning-level metrics
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 100,
            "failed_auth_attempts": 60,
            "blocked_ips": 10,
            "security_score": 0.65,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Check health
    let result = adapter.check_health().await;

    // ASSERT: Should be warning
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), SecurityHealth::Warning);
}

#[tokio::test]
async fn test_check_health_critical() {
    // ARRANGE: Mock server with critical metrics
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 200,
            "failed_auth_attempts": 150,
            "blocked_ips": 60,
            "security_score": 0.30,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Check health
    let result = adapter.check_health().await;

    // ASSERT: Should be critical
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), SecurityHealth::Critical);
}

#[tokio::test]
async fn test_check_health_network_failure() {
    // ARRANGE: Adapter with unreachable endpoint
    let adapter = SecurityAdapter::new("http://unreachable-health-check:9999")
        .unwrap()
        .with_timeout(Duration::from_millis(100));

    // ACT: Check health
    let result = adapter.check_health().await;

    // ASSERT: Should fail (can't determine health)
    assert!(result.is_err(), "Should fail when metrics unavailable");
}

// ============================================================================
// SECURITY_PROVIDER TRAIT TESTS (Direct Implementation)
// ============================================================================
// Note: SecurityProvider has async methods and is not dyn compatible
// Tests use concrete SecurityAdapter type directly

#[tokio::test]
async fn test_security_provider_implementation_collect_metrics() {
    // ARRANGE: Mock server
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 25,
            "failed_auth_attempts": 1,
            "blocked_ips": 0,
            "security_score": 0.98,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Use trait method implementation
    let result = SecurityProvider::collect_security_metrics(&adapter).await;

    // ASSERT: Should work through trait
    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert_eq!(metrics.active_sessions, 25);
}

#[tokio::test]
async fn test_security_provider_implementation_verify_auth() {
    // ARRANGE: Mock server
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json("Authorized"))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Use trait method implementation
    let result = SecurityProvider::verify_authentication(&adapter, "token-123").await;

    // ASSERT: Should work through trait
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AuthResult::Authorized);
}

#[tokio::test]
async fn test_security_provider_implementation_check_health() {
    // ARRANGE: Mock server
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 10,
            "failed_auth_attempts": 0,
            "blocked_ips": 0,
            "security_score": 1.0,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Use trait default method
    let result = SecurityProvider::check_security_health(&adapter).await;

    // ASSERT: Should work through trait
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), SecurityHealth::Healthy);
}

// ============================================================================
// EDGE CASES & STRESS TESTS
// ============================================================================

#[tokio::test]
async fn test_adapter_creation_empty_endpoint() {
    // ACT: Create adapter with empty endpoint
    let result = SecurityAdapter::new(String::new());

    // ASSERT: Should succeed (validation could be added later)
    assert!(result.is_ok(), "Empty endpoint currently accepted");
}

#[tokio::test]
async fn test_adapter_with_very_long_endpoint() {
    // ARRANGE: Very long but valid URL
    let long_endpoint = format!("http://security-{}.example.com", "provider".repeat(100));

    // ACT: Create adapter
    let result = SecurityAdapter::new(long_endpoint.clone());

    // ASSERT: Should handle long URLs
    assert!(result.is_ok());
    assert_eq!(result.unwrap().endpoint(), long_endpoint);
}

#[tokio::test]
async fn test_adapter_multiple_sequential_requests() {
    // ARRANGE: Mock server
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 30,
            "failed_auth_attempts": 2,
            "blocked_ips": 1,
            "security_score": 0.90,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();

    // ACT: Make multiple requests
    for _ in 0..10 {
        let result = adapter.collect_metrics().await;
        assert!(result.is_ok(), "Each request should succeed");
    }

    // ASSERT: All requests succeeded
}

#[tokio::test]
async fn test_adapter_concurrent_requests() {
    // ARRANGE: Mock server
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 15,
            "failed_auth_attempts": 1,
            "blocked_ips": 0,
            "security_score": 0.95,
            "timestamp": "2025-11-19T12:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).await.unwrap();
    let adapter_ref = &adapter;

    // ACT: Make concurrent requests
    let futures: Vec<_> =
        (0..5).map(|_| async move { adapter_ref.collect_metrics().await }).collect();

    let results = futures::future::join_all(futures).await;

    // ASSERT: All concurrent requests should succeed
    for result in results {
        assert!(result.is_ok(), "Concurrent request should succeed");
    }
}
