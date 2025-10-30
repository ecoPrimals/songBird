//! Comprehensive tests for `BearDog` security adapter
//!
//! Tests for security metrics collection, authentication verification, and error handling.

use chrono::Utc;
use songbird_types::SongbirdError;
use songbird_universal::adapters::security::{
    AuthResult, SecurityAdapter, SecurityHealth, SecurityMetrics,
};
use std::time::Duration;

/// Helper to create test security metrics
fn create_test_metrics() -> SecurityMetrics {
    SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 10,
        blocked_ips: 2,
        security_score: 0.95,
        timestamp: Utc::now(),
    }
}

// ============================================================================
// ADAPTER CREATION TESTS
// ============================================================================

#[test]
fn test_beardog_adapter_new_success() {
    // Arrange & Act
    let adapter = SecurityAdapter::new("http://example.com:8081".to_string());

    // Assert
    assert!(adapter.is_ok());
    let adapter = adapter.unwrap();
    assert_eq!(adapter.endpoint(), "http://example.com:8081");
}

#[test]
fn test_beardog_adapter_endpoint_validation() {
    // Arrange & Act
    let adapter = SecurityAdapter::new("http://security-service".to_string());

    // Assert
    assert!(adapter.is_ok());
    let adapter = adapter.unwrap();
    assert_eq!(adapter.endpoint(), "http://security-service");
}

#[test]
fn test_beardog_adapter_with_timeout() {
    // Arrange
    let custom_timeout = Duration::from_secs(30);

    // Act
    let adapter = SecurityAdapter::new("http://example.com".to_string())
        .unwrap()
        .with_timeout(custom_timeout);

    // Assert
    assert_eq!(adapter.endpoint(), "http://example.com");
}

// ============================================================================
// METRICS COLLECTION SUCCESS TESTS
// ============================================================================

#[tokio::test]
async fn test_beardog_collect_metrics_success() {
    // Arrange
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
                "timestamp": "2025-10-27T20:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert_eq!(metrics.active_sessions, 50);
    assert_eq!(metrics.failed_auth_attempts, 10);
    assert_eq!(metrics.blocked_ips, 2);
    assert!((metrics.security_score - 0.95).abs() < 0.001);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_beardog_collect_metrics_url_formatting() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "active_sessions": 100,
                "failed_auth_attempts": 5,
                "blocked_ips": 1,
                "security_score": 0.98,
                "timestamp": "2025-10-27T20:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_ok());
    mock.assert_async().await;
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[tokio::test]
async fn test_beardog_collect_metrics_network_error() {
    // Arrange
    let adapter = SecurityAdapter::new("http://nonexistent-host-12345:9999".to_string())
        .unwrap()
        .with_timeout(Duration::from_millis(100));

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Network { .. }));
}

#[tokio::test]
async fn test_beardog_collect_metrics_timeout() {
    // Arrange
    let adapter = SecurityAdapter::new("http://10.255.255.1:9999".to_string())
        .unwrap()
        .with_timeout(Duration::from_millis(50));

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Network { .. }));
}

#[tokio::test]
async fn test_beardog_collect_metrics_server_error_500() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/security")
        .with_status(500)
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Security { .. }));
    assert!(error.to_string().contains("500"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_beardog_collect_metrics_server_error_503() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/security")
        .with_status(503)
        .with_body("Service Unavailable")
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Security { .. }));
    assert!(error.to_string().contains("503"));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_beardog_collect_metrics_invalid_json() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("not valid json {{{")
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Security { .. }));
    mock.assert_async().await;
}

#[tokio::test]
async fn test_beardog_collect_metrics_missing_fields() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"active_sessions": 50}"#) // Missing required fields
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.collect_metrics().await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Security { .. }));
    mock.assert_async().await;
}

// ============================================================================
// AUTHENTICATION TESTS
// ============================================================================

#[tokio::test]
async fn test_beardog_verify_auth_authorized() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/auth/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#""Authorized""#)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.verify_auth("valid_token").await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AuthResult::Authorized);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_beardog_verify_auth_unauthorized() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server.mock("POST", "/auth/verify").with_status(401).create_async().await;

    let adapter = SecurityAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.verify_auth("invalid_token").await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AuthResult::Unauthorized);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_beardog_verify_auth_expired() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/auth/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#""Expired""#)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.verify_auth("expired_token").await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AuthResult::Expired);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_beardog_verify_auth_invalid() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/auth/verify")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#""Invalid""#)
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.verify_auth("malformed_token").await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AuthResult::Invalid);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_beardog_verify_auth_network_error() {
    // Arrange
    let adapter = SecurityAdapter::new("http://nonexistent-host-12345:9999".to_string())
        .unwrap()
        .with_timeout(Duration::from_millis(100));

    // Act
    let result = adapter.verify_auth("token").await;

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, SongbirdError::Network { .. }));
}

// ============================================================================
// HEALTH CHECK TESTS
// ============================================================================

#[tokio::test]
async fn test_beardog_check_health_healthy() {
    // Arrange
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
                "timestamp": "2025-10-27T20:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.check_health().await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), SecurityHealth::Healthy);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_beardog_check_health_warning() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "active_sessions": 75,
                "failed_auth_attempts": 60,
                "blocked_ips": 10,
                "security_score": 0.65,
                "timestamp": "2025-10-27T20:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.check_health().await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), SecurityHealth::Warning);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_beardog_check_health_critical() {
    // Arrange
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/metrics/security")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "active_sessions": 100,
                "failed_auth_attempts": 150,
                "blocked_ips": 60,
                "security_score": 0.45,
                "timestamp": "2025-10-27T20:00:00Z"
            }"#,
        )
        .create_async()
        .await;

    let adapter = SecurityAdapter::new(server.url()).unwrap();

    // Act
    let result = adapter.check_health().await;

    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), SecurityHealth::Critical);
    mock.assert_async().await;
}

// ============================================================================
// SECURITY METRICS CALCULATION TESTS
// ============================================================================

#[test]
fn test_security_metrics_healthy() {
    // Arrange
    let metrics = create_test_metrics();

    // Act & Assert
    assert!(!metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Healthy);
}

#[test]
fn test_security_metrics_under_attack_failed_auth() {
    // Arrange
    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 150, // > 100
        blocked_ips: 10,
        security_score: 0.85,
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_metrics_under_attack_blocked_ips() {
    // Arrange
    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 20,
        blocked_ips: 60, // > 50
        security_score: 0.85,
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_metrics_critical_low_score() {
    // Arrange
    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 20,
        blocked_ips: 10,
        security_score: 0.45, // < 0.5
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(!metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_metrics_warning_score() {
    // Arrange
    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 30,
        blocked_ips: 10,
        security_score: 0.65, // < 0.7
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(!metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Warning);
}

#[test]
fn test_security_metrics_warning_failed_auth() {
    // Arrange
    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 60, // > 50
        blocked_ips: 10,
        security_score: 0.85,
        timestamp: Utc::now(),
    };

    // Act & Assert
    assert!(!metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Warning);
}

#[test]
fn test_auth_result_equality() {
    // Test all AuthResult variants
    assert_eq!(AuthResult::Authorized, AuthResult::Authorized);
    assert_eq!(AuthResult::Unauthorized, AuthResult::Unauthorized);
    assert_eq!(AuthResult::Expired, AuthResult::Expired);
    assert_eq!(AuthResult::Invalid, AuthResult::Invalid);

    // Test inequality
    assert_ne!(AuthResult::Authorized, AuthResult::Unauthorized);
    assert_ne!(AuthResult::Expired, AuthResult::Invalid);
}

#[test]
fn test_security_health_equality() {
    // Test all SecurityHealth variants
    assert_eq!(SecurityHealth::Healthy, SecurityHealth::Healthy);
    assert_eq!(SecurityHealth::Warning, SecurityHealth::Warning);
    assert_eq!(SecurityHealth::Critical, SecurityHealth::Critical);

    // Test inequality
    assert_ne!(SecurityHealth::Healthy, SecurityHealth::Warning);
    assert_ne!(SecurityHealth::Warning, SecurityHealth::Critical);
}
