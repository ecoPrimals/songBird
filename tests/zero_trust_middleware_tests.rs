use songbird_gaming_bridge::SongbirdOrchestrator;
use songbird_gaming_bridge::config::NetworkConfig;
use std::collections::HashMap;
// Zero Trust Middleware Tests
//
// Comprehensive tests for zero trust security enforcement

use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::{HeaderMap, Method, Request, StatusCode};
use songbird_gaming_bridge::errors::Result;
use songbird_gaming_bridge::security::{
    authentication::{AuthenticationResult, Credentials},
    SecurityConfig, UserInfo, ZeroTrustConfig, ZeroTrustMiddleware,
};

/// Helper to create a test request
fn create_test_request(
    method: Method,
    path: &str,
    headers: Option<HeaderMap>,
) -> Request<Incoming> {
    let mut builder = Request::builder().method(method).uri(path);

    if let Some(header_map) = headers {
        for (name, value) in header_map.iter() {
            builder = builder.header(name, value);
        }
    }

    // Create a dummy body for testing
    builder.body(hyper::body::Incoming::default()).unwrap_or_default()
}

/// Helper to create test headers with authentication
fn create_auth_headers(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "authorization",
        format!("Bearer {}", token).parse().unwrap_or_default(),
    );
    headers
}

#[tokio::test]
async fn test_zero_trust_middleware_creation() {
    let config = ZeroTrustConfig::default();
    let security_config = SecurityConfig::default();

    let middleware = ZeroTrustMiddleware::new(config, security_config);
    assert!(
        middleware.is_ok(),
        "Zero trust middleware should be created successfully"
    );
}

#[tokio::test]
async fn test_exempt_paths_bypass() {
    let config = ZeroTrustConfig::default();
    let security_config = SecurityConfig::default();
    let middleware = ZeroTrustMiddleware::new(config, security_config).unwrap_or_default();

    // Test exempt paths
    let request = create_test_request(Method::GET, "/health", None);
    let response = middleware.process_request(request).await;

    assert!(
        response.is_ok(),
        "Health check should be exempt from zero trust"
    );
    let response = response.unwrap_or_default();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_missing_authentication_denied() {
    let config = ZeroTrustConfig::default();
    let security_config = SecurityConfig::default();
    let middleware = ZeroTrustMiddleware::new(config, security_config).unwrap_or_default();

    // Test request without authentication
    let request = create_test_request(Method::GET, "/api/users", None);
    let response = middleware.process_request(request).await;

    assert!(response.is_ok());
    let response = response.unwrap_or_default();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_invalid_token_denied() {
    let config = ZeroTrustConfig::default();
    let security_config = SecurityConfig::default();
    let middleware = ZeroTrustMiddleware::new(config, security_config).unwrap_or_default();

    // Test request with invalid token
    let headers = create_auth_headers("invalid_token");
    let request = create_test_request(Method::GET, "/api/users", Some(headers));
    let response = middleware.process_request(request).await;

    assert!(response.is_ok());
    let response = response.unwrap_or_default();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_rate_limiting() {
    let mut config = ZeroTrustConfig::default();
    config.max_auth_attempts = 2;
    config.auth_attempt_window = 60; // 1 minute

    let security_config = SecurityConfig::default();
    let middleware = ZeroTrustMiddleware::new(config, security_config).unwrap_or_default();

    // Make multiple failed authentication attempts
    for _ in 0..3 {
        let request = create_test_request(Method::GET, "/api/users", None);
        let _response = middleware.process_request(request).await;
    }

    // Fourth attempt should be rate limited
    let request = create_test_request(Method::GET, "/api/users", None);
    let response = middleware.process_request(request).await;

    assert!(response.is_ok());
    let response = response.unwrap_or_default();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_client_ip_extraction() {
    let config = ZeroTrustConfig::default();
    let security_config = SecurityConfig::default();
    let middleware = ZeroTrustMiddleware::new(config, security_config).unwrap_or_default();

    // Test with X-Forwarded-For header
    let mut headers = HeaderMap::new();
    headers.insert("x-forwarded-for", "192.168.1.100".parse().unwrap_or_default());

    let request = create_test_request(Method::GET, "/api/users", Some(headers));
    let response = middleware.process_request(request).await;

    // Should get unauthorized (not rate limited) for new IP
    assert!(response.is_ok());
    let response = response.unwrap_or_default();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_resource_type_determination() {
    let config = ZeroTrustConfig::default();
    let security_config = SecurityConfig::default();
    let middleware = ZeroTrustMiddleware::new(config, security_config).unwrap_or_default();

    // Test different path types
    let test_cases = vec![
        ("/api/users", "api"),
        ("/admin/config", "admin"),
        ("/user/profile", "user"),
        ("/service/status", "service"),
        ("/other/path", "general"),
    ];

    for (path, expected_type) in test_cases {
        let resource_type = middleware.determine_resource_type(path);
        assert_eq!(
            resource_type, expected_type,
            "Path {} should map to resource type {}",
            path, expected_type
        );
    }
}

#[tokio::test]
async fn test_http_method_to_action_mapping() {
    let config = ZeroTrustConfig::default();
    let security_config = SecurityConfig::default();
    let middleware = ZeroTrustMiddleware::new(config, security_config).unwrap_or_default();

    let test_cases = vec![
        (Method::GET, "read"),
        (Method::POST, "create"),
        (Method::PUT, "update"),
        (Method::PATCH, "update"),
        (Method::DELETE, "delete"),
        (Method::HEAD, "read"),
        (Method::OPTIONS, "options"),
    ];

    for (method, expected_action) in test_cases {
        let action = middleware.http_method_to_action(&method);
        assert_eq!(
            action, expected_action,
            "Method {:?} should map to action {}",
            method, expected_action
        );
    }
}

#[tokio::test]
async fn test_zero_trust_config_defaults() {
    let config = ZeroTrustConfig::default();

    assert!(
        config.enforce_authentication,
        "Authentication should be enforced by default"
    );
    assert!(
        config.enforce_authorization,
        "Authorization should be enforced by default"
    );
    assert!(
        !config.exempt_paths.is_empty(),
        "Should have some exempt paths by default"
    );
    assert!(
        config.exempt_paths.contains(&"/health".to_string()),
        "Health check should be exempt"
    );
    assert!(
        config.max_auth_attempts > 0,
        "Should have positive max auth attempts"
    );
    assert!(
        config.auth_attempt_window > 0,
        "Should have positive auth attempt window"
    );
    assert!(
        config.audit_all_requests,
        "Should audit all requests by default"
    );
}

#[tokio::test]
async fn test_multiple_auth_methods() {
    let config = ZeroTrustConfig::default();
    let security_config = SecurityConfig::default();
    let middleware = ZeroTrustMiddleware::new(config, security_config).unwrap_or_default();

    // Test API key authentication
    let mut headers = HeaderMap::new();
    headers.insert("x-api-key", "test-api-key".parse().unwrap_or_default());

    let request = create_test_request(Method::GET, "/api/users", Some(headers));
    let response = middleware.process_request(request).await;

    assert!(response.is_ok());
    let response = response.unwrap_or_default();
    // Should get unauthorized since we don't have a valid provider configured
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_authorization_enforcement() {
    let mut config = ZeroTrustConfig::default();
    config.enforce_authorization = true;

    let security_config = SecurityConfig::default();
    let middleware = ZeroTrustMiddleware::new(config, security_config).unwrap_or_default();

    // Test that even with valid authentication, authorization is checked
    let headers = create_auth_headers("valid_token");
    let request = create_test_request(Method::DELETE, "/admin/delete", Some(headers));
    let response = middleware.process_request(request).await;

    assert!(response.is_ok());
    let response = response.unwrap_or_default();
    // Should get unauthorized since our test doesn't have a real auth provider
    assert!(
        response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn test_error_response_format() {
    let config = ZeroTrustConfig::default();
    let security_config = SecurityConfig::default();
    let middleware = ZeroTrustMiddleware::new(config, security_config).unwrap_or_default();

    let request = create_test_request(Method::GET, "/api/users", None);
    let response = middleware.process_request(request).await.unwrap_or_default();

    // Check response headers
    assert_eq!(
        response.headers().get("content-type").unwrap_or_default(),
        "application/json"
    );
    assert_eq!(response.headers().get("x-zero-trust").unwrap_or_default(), "enforced");

    // Response should be JSON with error structure
    let body = response.body();
    let body_str = std::str::from_utf8(body).unwrap_or_default();
    assert!(body_str.contains("status"));
    assert!(body_str.contains("message"));
    assert!(body_str.contains("timestamp"));
}

#[tokio::test]
async fn test_custom_exempt_paths() {
    let mut config = ZeroTrustConfig::default();
    config.exempt_paths = vec!["/custom/health".to_string(), "/status".to_string()];

    let security_config = SecurityConfig::default();
    let middleware = ZeroTrustMiddleware::new(config, security_config).unwrap_or_default();

    // Test custom exempt path
    let request = create_test_request(Method::GET, "/custom/health", None);
    let response = middleware.process_request(request).await.unwrap_or_default();
    assert_eq!(response.status(), StatusCode::OK);

    // Test path that's not exempt
    let request = create_test_request(Method::GET, "/health", None);
    let response = middleware.process_request(request).await.unwrap_or_default();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
