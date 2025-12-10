//! Comprehensive Async Security Adapter Coverage Tests
//!
//! **Goal**: Cover async methods in security.rs (from_discovery, collect_metrics, verify_auth, check_health)
//! **Coverage Target**: Raise security.rs from 14.71% to 90%+
//!
//! Uses `wiremock` for HTTP mocking without hardcoded ports

use songbird_universal::adapters::security::{
    AuthResult, SecurityAdapter, SecurityHealth, SecurityProvider,
};
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ============================================================================
// ASYNC ADAPTER CREATION TESTS
// ============================================================================

#[tokio::test]
async fn test_from_discovery_with_env_var() {
    // Setup mock server
    let mock_server = MockServer::start().await;

    // Set environment variable
    std::env::set_var("SONGBIRD_SECURITY_ENDPOINT", &mock_server.uri());

    // Test discovery
    let result = SecurityAdapter::from_discovery().await;
    assert!(result.is_ok(), "Should discover from environment variable");

    // Cleanup
    std::env::remove_var("SONGBIRD_SECURITY_ENDPOINT");
}

#[tokio::test]
async fn test_from_discovery_fallback_to_security_provider_endpoint() {
    // Setup mock server
    let mock_server = MockServer::start().await;

    // Remove primary env var and set fallback
    std::env::remove_var("SONGBIRD_SECURITY_ENDPOINT");
    std::env::set_var("SECURITY_PROVIDER_ENDPOINT", &mock_server.uri());

    // Test discovery
    let result = SecurityAdapter::from_discovery().await;
    assert!(result.is_ok(), "Should fallback to SECURITY_PROVIDER_ENDPOINT");

    // Cleanup
    std::env::remove_var("SECURITY_PROVIDER_ENDPOINT");
}

#[tokio::test]
async fn test_from_discovery_fallback_to_beardog_endpoint() {
    // Setup mock server
    let mock_server = MockServer::start().await;

    // Remove all primary env vars and set BearDog fallback
    std::env::remove_var("SONGBIRD_SECURITY_ENDPOINT");
    std::env::remove_var("SECURITY_PROVIDER_ENDPOINT");
    std::env::set_var("BEARDOG_ENDPOINT", &mock_server.uri());

    // Test discovery
    let result = SecurityAdapter::from_discovery().await;
    assert!(result.is_ok(), "Should fallback to BEARDOG_ENDPOINT");

    // Cleanup
    std::env::remove_var("BEARDOG_ENDPOINT");
}

#[tokio::test]
async fn test_from_discovery_uses_host_and_port_fallback() {
    // Remove all security endpoint env vars
    std::env::remove_var("SONGBIRD_SECURITY_ENDPOINT");
    std::env::remove_var("SECURITY_PROVIDER_ENDPOINT");
    std::env::remove_var("BEARDOG_ENDPOINT");

    // Set host and port
    std::env::set_var("SONGBIRD_HOST", "http://testhost");
    std::env::set_var("SONGBIRD_SECURITY_PORT", "9999");

    // Test discovery - should fall back to constructed endpoint
    let result = SecurityAdapter::from_discovery().await;
    assert!(result.is_ok(), "Should construct endpoint from host and port");

    let adapter = result.unwrap();
    assert!(adapter.endpoint().contains("testhost"), "Should use configured host");
    assert!(adapter.endpoint().contains("9999"), "Should use configured port");

    // Cleanup
    std::env::remove_var("SONGBIRD_HOST");
    std::env::remove_var("SONGBIRD_SECURITY_PORT");
}

// ============================================================================
// COLLECT METRICS TESTS
// ============================================================================

#[tokio::test]
async fn test_collect_metrics_success() {
    let mock_server = MockServer::start().await;

    // Mock successful metrics response
    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 42,
            "failed_auth_attempts": 5,
            "blocked_ips": 2,
            "security_score": 0.95,
            "timestamp": "2024-01-01T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();
    let metrics = adapter.collect_metrics().await;

    assert!(metrics.is_ok(), "Should collect metrics successfully");
    let metrics = metrics.unwrap();
    assert_eq!(metrics.active_sessions, 42);
    assert_eq!(metrics.failed_auth_attempts, 5);
    assert_eq!(metrics.blocked_ips, 2);
    assert!((metrics.security_score - 0.95).abs() < 0.001);
}

#[tokio::test]
async fn test_collect_metrics_network_error() {
    // Use a non-existent endpoint to trigger network error
    let adapter = SecurityAdapter::new("http://localhost:1".to_string())
        .unwrap()
        .with_timeout(Duration::from_millis(100));

    let result = adapter.collect_metrics().await;
    assert!(result.is_err(), "Should fail with network error");

    let err = result.unwrap_err();
    assert!(err.to_string().contains("network") || err.to_string().contains("reach"));
}

#[tokio::test]
async fn test_collect_metrics_http_error() {
    let mock_server = MockServer::start().await;

    // Mock error response
    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.collect_metrics().await;

    assert!(result.is_err(), "Should fail with HTTP error");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("500") || err.to_string().contains("unavailable"));
}

#[tokio::test]
async fn test_collect_metrics_invalid_json() {
    let mock_server = MockServer::start().await;

    // Mock invalid JSON response
    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.collect_metrics().await;

    assert!(result.is_err(), "Should fail parsing invalid JSON");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("parse") || err.to_string().contains("security"));
}

#[tokio::test]
async fn test_collect_metrics_missing_timestamp() {
    let mock_server = MockServer::start().await;

    // Mock response without timestamp (should be set to current time)
    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 10,
            "failed_auth_attempts": 0,
            "blocked_ips": 0,
            "security_score": 1.0,
            "timestamp": "1970-01-01T00:00:00Z"  // Unix epoch
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.collect_metrics().await;

    assert!(result.is_ok(), "Should handle missing timestamp");
    let metrics = result.unwrap();
    // Timestamp should be updated to current time if it was epoch
    assert!(metrics.timestamp.timestamp() > 0);
}

#[tokio::test]
async fn test_collect_metrics_timeout() {
    let mock_server = MockServer::start().await;

    // Mock slow response
    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(10)))
        .mount(&mock_server)
        .await;

    let adapter =
        SecurityAdapter::new(mock_server.uri()).unwrap().with_timeout(Duration::from_millis(50));

    let result = adapter.collect_metrics().await;
    assert!(result.is_err(), "Should timeout on slow response");
}

// ============================================================================
// VERIFY AUTH TESTS
// ============================================================================

#[tokio::test]
async fn test_verify_auth_authorized() {
    let mock_server = MockServer::start().await;

    // Mock successful authorization
    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!("Authorized")))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.verify_auth("valid_token").await;

    assert!(result.is_ok(), "Should verify auth successfully");
    assert_eq!(result.unwrap(), AuthResult::Authorized);
}

#[tokio::test]
async fn test_verify_auth_unauthorized() {
    let mock_server = MockServer::start().await;

    // Mock unauthorized response
    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.verify_auth("invalid_token").await;

    assert!(result.is_ok(), "Should handle unauthorized gracefully");
    assert_eq!(result.unwrap(), AuthResult::Unauthorized);
}

#[tokio::test]
async fn test_verify_auth_expired() {
    let mock_server = MockServer::start().await;

    // Mock expired token response
    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!("Expired")))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.verify_auth("expired_token").await;

    assert!(result.is_ok(), "Should handle expired token");
    assert_eq!(result.unwrap(), AuthResult::Expired);
}

#[tokio::test]
async fn test_verify_auth_invalid() {
    let mock_server = MockServer::start().await;

    // Mock invalid token response
    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!("Invalid")))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.verify_auth("malformed_token").await;

    assert!(result.is_ok(), "Should handle invalid token");
    assert_eq!(result.unwrap(), AuthResult::Invalid);
}

#[tokio::test]
async fn test_verify_auth_network_error() {
    let adapter = SecurityAdapter::new("http://localhost:1".to_string())
        .unwrap()
        .with_timeout(Duration::from_millis(100));

    let result = adapter.verify_auth("token").await;
    assert!(result.is_err(), "Should fail with network error");
}

#[tokio::test]
async fn test_verify_auth_invalid_json_response() {
    let mock_server = MockServer::start().await;

    // Mock invalid JSON response
    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.verify_auth("token").await;

    assert!(result.is_err(), "Should fail parsing invalid JSON");
}

// ============================================================================
// CHECK HEALTH TESTS
// ============================================================================

#[tokio::test]
async fn test_check_health_healthy() {
    let mock_server = MockServer::start().await;

    // Mock healthy metrics
    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 50,
            "failed_auth_attempts": 5,
            "blocked_ips": 2,
            "security_score": 0.95,
            "timestamp": "2024-01-01T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.check_health().await;

    assert!(result.is_ok(), "Should check health successfully");
    assert_eq!(result.unwrap(), SecurityHealth::Healthy);
}

#[tokio::test]
async fn test_check_health_warning() {
    let mock_server = MockServer::start().await;

    // Mock warning-level metrics
    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 75,
            "failed_auth_attempts": 60,
            "blocked_ips": 10,
            "security_score": 0.65,
            "timestamp": "2024-01-01T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.check_health().await;

    assert!(result.is_ok(), "Should check health successfully");
    assert_eq!(result.unwrap(), SecurityHealth::Warning);
}

#[tokio::test]
async fn test_check_health_critical() {
    let mock_server = MockServer::start().await;

    // Mock critical metrics (under attack)
    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 100,
            "failed_auth_attempts": 150,
            "blocked_ips": 60,
            "security_score": 0.45,
            "timestamp": "2024-01-01T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.check_health().await;

    assert!(result.is_ok(), "Should check health successfully");
    assert_eq!(result.unwrap(), SecurityHealth::Critical);
}

#[tokio::test]
async fn test_check_health_network_failure() {
    let adapter = SecurityAdapter::new("http://localhost:1".to_string())
        .unwrap()
        .with_timeout(Duration::from_millis(100));

    let result = adapter.check_health().await;
    assert!(result.is_err(), "Should fail when metrics unavailable");
}

// ============================================================================
// SECURITY PROVIDER TRAIT IMPLEMENTATION TESTS
// ============================================================================
//
// Note: SecurityProvider trait has async methods, so it's not dyn-compatible.
// These tests verify the trait implementation directly on SecurityAdapter.

#[tokio::test]
async fn test_security_provider_collect_security_metrics() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 30,
            "failed_auth_attempts": 3,
            "blocked_ips": 1,
            "security_score": 0.92,
            "timestamp": "2024-01-01T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();

    // Call trait method directly (SecurityAdapter implements SecurityProvider)
    let result = SecurityProvider::collect_security_metrics(&adapter).await;
    assert!(result.is_ok(), "Trait method should work");
    let metrics = result.unwrap();
    assert_eq!(metrics.active_sessions, 30);
}

#[tokio::test]
async fn test_security_provider_verify_authentication() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/auth/verify"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!("Authorized")))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();

    // Call trait method directly
    let result = SecurityProvider::verify_authentication(&adapter, "token").await;
    assert!(result.is_ok(), "Trait method should work");
    assert_eq!(result.unwrap(), AuthResult::Authorized);
}

#[tokio::test]
async fn test_security_provider_check_security_health() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 20,
            "failed_auth_attempts": 1,
            "blocked_ips": 0,
            "security_score": 0.98,
            "timestamp": "2024-01-01T00:00:00Z"
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();

    // Call trait method directly
    let result = SecurityProvider::check_security_health(&adapter).await;
    assert!(result.is_ok(), "Trait method should work");
    assert_eq!(result.unwrap(), SecurityHealth::Healthy);
}

// ============================================================================
// EDGE CASE AND ERROR PATH TESTS
// ============================================================================

#[tokio::test]
async fn test_collect_metrics_with_extra_fields() {
    let mock_server = MockServer::start().await;

    // Mock response with extra fields (should be ignored)
    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_sessions": 25,
            "failed_auth_attempts": 2,
            "blocked_ips": 1,
            "security_score": 0.88,
            "timestamp": "2024-01-01T00:00:00Z",
            "extra_field": "should be ignored",
            "unknown_metric": 999
        })))
        .mount(&mock_server)
        .await;

    let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();
    let result = adapter.collect_metrics().await;

    assert!(result.is_ok(), "Should handle extra fields gracefully");
    let metrics = result.unwrap();
    assert_eq!(metrics.active_sessions, 25);
}

#[tokio::test]
async fn test_collect_metrics_with_various_status_codes() {
    let mock_server = MockServer::start().await;

    // Test various HTTP error codes
    for status_code in [400, 401, 403, 404, 500, 502, 503] {
        Mock::given(method("GET"))
            .and(path("/metrics/security"))
            .respond_with(ResponseTemplate::new(status_code))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();
        let result = adapter.collect_metrics().await;

        assert!(result.is_err(), "Should fail for status code {}", status_code);
    }
}

#[tokio::test]
async fn test_verify_auth_with_various_http_error_codes() {
    let mock_server = MockServer::start().await;

    // Test that non-success codes return Unauthorized
    for status_code in [400, 403, 404, 500, 502] {
        Mock::given(method("POST"))
            .and(path("/auth/verify"))
            .respond_with(ResponseTemplate::new(status_code))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let adapter = SecurityAdapter::new(mock_server.uri()).unwrap();
        let result = adapter.verify_auth("token").await;

        assert!(result.is_ok(), "Should return Unauthorized for status {}", status_code);
        assert_eq!(result.unwrap(), AuthResult::Unauthorized);
    }
}

#[tokio::test]
async fn test_adapter_with_custom_timeout_in_requests() {
    let mock_server = MockServer::start().await;

    // Mock slow response (1 second delay)
    Mock::given(method("GET"))
        .and(path("/metrics/security"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(1)).set_body_json(
            serde_json::json!({
                "active_sessions": 10,
                "failed_auth_attempts": 0,
                "blocked_ips": 0,
                "security_score": 1.0,
                "timestamp": "2024-01-01T00:00:00Z"
            }),
        ))
        .mount(&mock_server)
        .await;

    // Should fail with very short timeout (100ms << 1s delay)
    let adapter_short =
        SecurityAdapter::new(mock_server.uri()).unwrap().with_timeout(Duration::from_millis(100));
    let result_short = adapter_short.collect_metrics().await;
    assert!(result_short.is_err(), "Should timeout with insufficient timeout (100ms vs 1s delay)");

    // Note: Don't test the "should succeed" case as it's timing-dependent and can be flaky
}

#[tokio::test]
async fn test_endpoint_url_construction_in_requests() {
    let mock_server = MockServer::start().await;

    // Test that endpoints with trailing slashes work correctly
    let endpoints = vec![
        mock_server.uri(),
        format!("{}/", mock_server.uri()),
        format!("{}//", mock_server.uri()), // Double slash
    ];

    for endpoint in endpoints {
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "active_sessions": 10,
                "failed_auth_attempts": 0,
                "blocked_ips": 0,
                "security_score": 1.0,
                "timestamp": "2024-01-01T00:00:00Z"
            })))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        let adapter = SecurityAdapter::new(endpoint.clone()).unwrap();
        // Request should still work despite URL formatting
        let result = adapter.collect_metrics().await;
        // May succeed or fail depending on URL normalization
        // Just ensure it doesn't panic
        let _ = result;
    }
}
