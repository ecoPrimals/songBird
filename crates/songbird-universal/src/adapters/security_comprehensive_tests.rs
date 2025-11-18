//! Comprehensive Tests for Security Adapter
//!
//! **Purpose**: Achieve 90% test coverage for security adapter (from 14.71%)
//!
//! This file adds missing test coverage for:
//! - AuthResult variants and operations
//! - SecurityMetrics boundary conditions
//! - SecurityHealth transitions
//! - Adapter configuration edge cases
//! - Error handling paths

use super::*;
use songbird_types::SongbirdError;

// ============================================================================
// AUTH_RESULT COMPREHENSIVE TESTS
// ============================================================================

#[test]
fn test_auth_result_all_variants() {
    // Test all Auth Result variants can be created
    let authorized = AuthResult::Authorized;
    let unauthorized = AuthResult::Unauthorized;
    let expired = AuthResult::Expired;
    let invalid = AuthResult::Invalid;

    // Verify they're distinct
    assert_ne!(authorized, unauthorized);
    assert_ne!(authorized, expired);
    assert_ne!(authorized, invalid);
    assert_ne!(unauthorized, expired);
    assert_ne!(unauthorized, invalid);
    assert_ne!(expired, invalid);
}

#[test]
fn test_auth_result_equality() {
    assert_eq!(AuthResult::Authorized, AuthResult::Authorized);
    assert_eq!(AuthResult::Unauthorized, AuthResult::Unauthorized);
    assert_eq!(AuthResult::Expired, AuthResult::Expired);
    assert_eq!(AuthResult::Invalid, AuthResult::Invalid);
}

#[test]
fn test_auth_result_clone() {
    let result = AuthResult::Authorized;
    let cloned = result.clone();
    assert_eq!(result, cloned);
}

#[test]
fn test_auth_result_debug() {
    let result = AuthResult::Expired;
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("Expired"));
}

#[test]
fn test_auth_result_serialization() {
    let result = AuthResult::Authorized;
    let json = serde_json::to_string(&result);
    assert!(json.is_ok(), "Should serialize");

    let result = AuthResult::Unauthorized;
    let json = serde_json::to_string(&result);
    assert!(json.is_ok(), "Should serialize");
}

#[test]
fn test_auth_result_deserialization() {
    let json = r#""Authorized""#;
    let result: Result<AuthResult, _> = serde_json::from_str(json);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AuthResult::Authorized);

    let json = r#""Invalid""#;
    let result: Result<AuthResult, _> = serde_json::from_str(json);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), AuthResult::Invalid);
}

// ============================================================================
// SECURITY_METRICS COMPREHENSIVE TESTS
// ============================================================================

#[test]
fn test_security_metrics_healthy_score() {
    let metrics = SecurityMetrics {
        active_sessions: 100,
        failed_auth_attempts: 5,
        blocked_ips: 1,
        security_score: 0.95,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), SecurityHealth::Healthy);
    assert!(!metrics.is_under_attack());
}

#[test]
fn test_security_metrics_warning_score() {
    let metrics = SecurityMetrics {
        active_sessions: 100,
        failed_auth_attempts: 60, // Over threshold
        blocked_ips: 5,
        security_score: 0.65,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), SecurityHealth::Warning);
    assert!(!metrics.is_under_attack());
}

#[test]
fn test_security_metrics_critical_low_score() {
    let metrics = SecurityMetrics {
        active_sessions: 100,
        failed_auth_attempts: 30,
        blocked_ips: 10,
        security_score: 0.4, // Below 0.5 threshold
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_metrics_critical_high_failed_auth() {
    let metrics = SecurityMetrics {
        active_sessions: 100,
        failed_auth_attempts: 150, // Over 100 threshold
        blocked_ips: 10,
        security_score: 0.8,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_metrics_critical_high_blocked_ips() {
    let metrics = SecurityMetrics {
        active_sessions: 100,
        failed_auth_attempts: 30,
        blocked_ips: 60, // Over 50 threshold
        security_score: 0.8,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_metrics_boundary_attack_threshold() {
    // Exactly at threshold (100 failed attempts)
    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 100,
        blocked_ips: 30,
        security_score: 0.9,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_under_attack(), "100 is not > 100");

    // Just over threshold (101 failed attempts)
    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 101,
        blocked_ips: 30,
        security_score: 0.9,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_under_attack(), "101 is > 100");
}

#[test]
fn test_security_metrics_boundary_blocked_ips() {
    // Exactly at threshold (50 blocked IPs)
    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 30,
        blocked_ips: 50,
        security_score: 0.9,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_under_attack(), "50 is not > 50");

    // Just over threshold (51 blocked IPs)
    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 30,
        blocked_ips: 51,
        security_score: 0.9,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_under_attack(), "51 is > 50");
}

#[test]
fn test_security_metrics_boundary_score_healthy() {
    // Just above warning threshold (0.7)
    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 40,
        blocked_ips: 5,
        security_score: 0.71,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), SecurityHealth::Healthy);
}

#[test]
fn test_security_metrics_boundary_score_warning() {
    // Just below healthy threshold but above critical (0.69)
    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 40,
        blocked_ips: 5,
        security_score: 0.69,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), SecurityHealth::Warning);
}

#[test]
fn test_security_metrics_boundary_score_critical() {
    // Just below warning threshold (0.49)
    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 40,
        blocked_ips: 5,
        security_score: 0.49,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_metrics_serialization() {
    let metrics = SecurityMetrics {
        active_sessions: 100,
        failed_auth_attempts: 50,
        blocked_ips: 10,
        security_score: 0.85,
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&metrics);
    assert!(json.is_ok(), "Metrics should serialize");
}

#[test]
fn test_security_metrics_deserialization() {
    let json = r#"{
        "active_sessions": 100,
        "failed_auth_attempts": 50,
        "blocked_ips": 10,
        "security_score": 0.85,
        "timestamp": "2025-11-18T12:00:00Z"
    }"#;

    let metrics: Result<SecurityMetrics, _> = serde_json::from_str(json);
    assert!(metrics.is_ok(), "Should deserialize valid metrics");

    let metrics = metrics.unwrap();
    assert_eq!(metrics.active_sessions, 100);
    assert_eq!(metrics.failed_auth_attempts, 50);
    assert_eq!(metrics.blocked_ips, 10);
    assert_eq!(metrics.security_score, 0.85);
}

#[test]
fn test_security_metrics_clone() {
    let metrics = SecurityMetrics {
        active_sessions: 75,
        failed_auth_attempts: 20,
        blocked_ips: 3,
        security_score: 0.92,
        timestamp: chrono::Utc::now(),
    };

    let cloned = metrics.clone();
    assert_eq!(metrics.active_sessions, cloned.active_sessions);
    assert_eq!(metrics.failed_auth_attempts, cloned.failed_auth_attempts);
    assert_eq!(metrics.blocked_ips, cloned.blocked_ips);
    assert_eq!(metrics.security_score, cloned.security_score);
}

// ============================================================================
// SECURITY_HEALTH COMPREHENSIVE TESTS
// ============================================================================

#[test]
fn test_security_health_all_variants() {
    let healthy = SecurityHealth::Healthy;
    let warning = SecurityHealth::Warning;
    let critical = SecurityHealth::Critical;

    // Verify they're distinct
    assert_ne!(healthy, warning);
    assert_ne!(healthy, critical);
    assert_ne!(warning, critical);
}

#[test]
fn test_security_health_serialization() {
    let health = SecurityHealth::Warning;
    let json = serde_json::to_string(&health);
    assert!(json.is_ok());
}

#[test]
fn test_security_health_deserialization() {
    let json = r#""Healthy""#;
    let health: Result<SecurityHealth, _> = serde_json::from_str(json);
    assert!(health.is_ok());
    assert_eq!(health.unwrap(), SecurityHealth::Healthy);
}

#[test]
fn test_security_health_copy() {
    let health = SecurityHealth::Critical;
    let copied = health; // Copy trait
    assert_eq!(health, copied);
}

// ============================================================================
// SECURITY_ADAPTER COMPREHENSIVE TESTS
// ============================================================================

#[test]
fn test_adapter_new_various_endpoints() {
    // Test different URL formats
    let endpoints = vec![
        "http://localhost:8081",
        "https://security.example.com",
        "http://192.168.1.100:9000",
        "https://sec-provider.internal:443",
    ];

    for endpoint in endpoints {
        let adapter = SecurityAdapter::new(endpoint.to_string());
        assert!(adapter.is_ok(), "Should create adapter for endpoint: {}", endpoint);
        assert_eq!(adapter.unwrap().endpoint(), endpoint);
    }
}

#[test]
fn test_adapter_with_timeout_builder() {
    let adapter = SecurityAdapter::new("http://test:8080".to_string())
        .unwrap()
        .with_timeout(Duration::from_secs(10));

    assert_eq!(adapter.timeout, Duration::from_secs(10));
}

#[test]
fn test_adapter_with_timeout_chaining() {
    let adapter = SecurityAdapter::new("http://test:8080".to_string())
        .unwrap()
        .with_timeout(Duration::from_secs(5))
        .with_timeout(Duration::from_millis(500));

    // Last timeout should win
    assert_eq!(adapter.timeout, Duration::from_millis(500));
}

#[test]
fn test_adapter_endpoint_preserved_after_timeout() {
    let endpoint = "http://original:8080";
    let adapter = SecurityAdapter::new(endpoint.to_string()).unwrap();

    let adapter2 = adapter.with_timeout(Duration::from_secs(10));

    // Endpoint should be preserved after transformation
    assert_eq!(adapter2.endpoint(), endpoint);
}

#[test]
fn test_adapter_creation_error_scenarios() {
    // These currently succeed, but document expected behavior

    // Empty endpoint (currently allowed, may want to validate later)
    let result = SecurityAdapter::new(String::new());
    assert!(result.is_ok(), "Empty endpoint currently allowed");

    // Invalid URL format (currently allowed, validated at request time)
    let result = SecurityAdapter::new("not-a-valid-url".to_string());
    assert!(result.is_ok(), "Invalid URL format allowed (validated at request)");
}

#[test]
fn test_adapter_default_timeout_value() {
    let adapter = SecurityAdapter::new("http://test:8080".to_string()).unwrap();
    assert_eq!(adapter.timeout, Duration::from_secs(5), "Default timeout should be 5 seconds");
}

#[test]
fn test_adapter_timeout_ranges() {
    let durations = vec![
        Duration::from_millis(100),
        Duration::from_secs(1),
        Duration::from_secs(30),
        Duration::from_secs(60),
    ];

    for duration in durations {
        let adapter =
            SecurityAdapter::new("http://test:8080".to_string()).unwrap().with_timeout(duration);

        assert_eq!(adapter.timeout, duration);
    }
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[test]
fn test_metrics_with_attack_and_critical_score() {
    // Both attack indicators and low score
    let metrics = SecurityMetrics {
        active_sessions: 200,
        failed_auth_attempts: 150, // Attack
        blocked_ips: 60,           // Attack
        security_score: 0.3,       // Critical
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_metrics_warning_with_moderate_attacks() {
    // Some attack indicators but not extreme
    let metrics = SecurityMetrics {
        active_sessions: 150,
        failed_auth_attempts: 60, // Warning level
        blocked_ips: 20,
        security_score: 0.65, // Warning level
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Warning);
}

#[test]
fn test_metrics_healthy_with_low_values() {
    let metrics = SecurityMetrics {
        active_sessions: 25,
        failed_auth_attempts: 2,
        blocked_ips: 0,
        security_score: 0.99,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Healthy);
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn test_security_score_exactly_0_5() {
    // Exactly at critical threshold
    let metrics = SecurityMetrics {
        active_sessions: 100,
        failed_auth_attempts: 30,
        blocked_ips: 10,
        security_score: 0.5, // Exactly 0.5
        timestamp: chrono::Utc::now(),
    };

    // 0.5 is not < 0.5, so should be warning
    assert_eq!(metrics.health_status(), SecurityHealth::Warning);
}

#[test]
fn test_security_score_exactly_0_7() {
    // Exactly at warning threshold
    let metrics = SecurityMetrics {
        active_sessions: 100,
        failed_auth_attempts: 30,
        blocked_ips: 10,
        security_score: 0.7, // Exactly 0.7
        timestamp: chrono::Utc::now(),
    };

    // 0.7 is not < 0.7, so should be healthy
    assert_eq!(metrics.health_status(), SecurityHealth::Healthy);
}

#[test]
fn test_failed_auth_exactly_50() {
    // Exactly at warning threshold for failed auth
    let metrics = SecurityMetrics {
        active_sessions: 100,
        failed_auth_attempts: 50, // Exactly 50
        blocked_ips: 10,
        security_score: 0.8,
        timestamp: chrono::Utc::now(),
    };

    // 50 is not > 50, so no warning from failed auth alone
    // But score is good, so should be healthy
    assert_eq!(metrics.health_status(), SecurityHealth::Healthy);
}

#[test]
fn test_failed_auth_exactly_51() {
    // Just over warning threshold
    let metrics = SecurityMetrics {
        active_sessions: 100,
        failed_auth_attempts: 51, // Just over 50
        blocked_ips: 10,
        security_score: 0.8,
        timestamp: chrono::Utc::now(),
    };

    // 51 > 50, triggers warning condition
    assert_eq!(metrics.health_status(), SecurityHealth::Warning);
}

#[test]
fn test_multiple_conditions_worst_wins() {
    // Multiple warning conditions
    let metrics = SecurityMetrics {
        active_sessions: 100,
        failed_auth_attempts: 60, // Warning
        blocked_ips: 25,
        security_score: 0.65, // Warning
        timestamp: chrono::Utc::now(),
    };

    // Should be warning (not healthy)
    assert_eq!(metrics.health_status(), SecurityHealth::Warning);

    // Add critical condition - should become critical
    let metrics = SecurityMetrics {
        active_sessions: 100,
        failed_auth_attempts: 150, // Critical (attack)
        blocked_ips: 25,
        security_score: 0.65, // Warning
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}
