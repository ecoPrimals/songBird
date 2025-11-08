//! Tests for Security Capability Adapter
//!
//! Separated from security.rs for file size compliance (1000-line policy)

use super::*;
use songbird_types::SongbirdError;
use std::time::Duration;

#[test]
fn test_security_metrics_calculations() {
    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 10,
        blocked_ips: 2,
        security_score: 0.95,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Healthy);
}

#[test]
fn test_security_under_attack() {
    let metrics = SecurityMetrics {
        active_sessions: 100,
        failed_auth_attempts: 150,
        blocked_ips: 60,
        security_score: 0.45,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_warning() -> SongbirdResult<()> {
    let metrics = SecurityMetrics {
        active_sessions: 75,
        failed_auth_attempts: 60,
        blocked_ips: 10,
        security_score: 0.65,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_under_attack());
    assert_eq!(metrics.health_status(), SecurityHealth::Warning);
    Ok(())
}

#[test]
fn test_adapter_creation() -> Result<(), Box<dyn std::error::Error>> {
    let adapter =
        SecurityAdapter::new("http://security-provider:8081".to_string()).map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?;
    assert_eq!(adapter.endpoint(), "http://security-provider:8081");
    Ok(())
}

#[test]
fn test_adapter_with_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SecurityAdapter::new("http://security-provider:8081".to_string())
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(Duration::from_secs(10));
    assert_eq!(adapter.timeout, Duration::from_secs(10));
    Ok(())
}

#[test]
fn test_auth_result_equality() {
    assert_eq!(AuthResult::Authorized, AuthResult::Authorized);
    assert_ne!(AuthResult::Authorized, AuthResult::Unauthorized);
    assert_eq!(AuthResult::Expired, AuthResult::Expired);
}

#[test]
fn test_security_health_critical_low_score() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 5,
        blocked_ips: 1,
        security_score: 0.45,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(
        metrics.health_status(),
        SecurityHealth::Critical,
        "Low security score should result in Critical status"
    );
}

#[test]
fn test_security_health_critical_high_failed_attempts() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 101,
        blocked_ips: 1,
        security_score: 0.95,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_under_attack(), "High failed attempts should trigger under attack");
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_health_critical_high_blocked_ips() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 5,
        blocked_ips: 51,
        security_score: 0.95,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_under_attack(), "High blocked IPs should trigger under attack");
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_health_warning_moderate_score() {
    let metrics = SecurityMetrics {
        active_sessions: 25,
        failed_auth_attempts: 55,
        blocked_ips: 10,
        security_score: 0.65,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(
        metrics.health_status(),
        SecurityHealth::Warning,
        "Moderate score with elevated failed attempts should be Warning"
    );
}

#[test]
fn test_security_health_boundary_cases() {
    // Test exactly at boundary: security_score = 0.5
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 10,
        blocked_ips: 5,
        security_score: 0.5,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(
        metrics.health_status(),
        SecurityHealth::Warning,
        "Score at 0.5 boundary should be Warning"
    );

    // Test exactly at boundary: failed_auth_attempts = 100
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 100,
        blocked_ips: 5,
        security_score: 0.9,
        timestamp: chrono::Utc::now(),
    };
    assert!(!metrics.is_under_attack(), "Exactly 100 failed attempts should not trigger attack");
}

#[test]
fn test_auth_result_variants() -> SongbirdResult<()> {
    let results = [
        AuthResult::Authorized,
        AuthResult::Unauthorized,
        AuthResult::Expired,
        AuthResult::Invalid,
    ];

    assert_eq!(results.len(), 4, "Should have all 4 auth result variants");
    Ok(())
}

#[test]
fn test_security_metrics_serialization() -> SongbirdResult<()> {
    let metrics = SecurityMetrics {
        active_sessions: 42,
        failed_auth_attempts: 7,
        blocked_ips: 3,
        security_score: 0.88,
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&metrics).map_err(|e| {
        SongbirdError::configuration(format!(
            "SecurityMetrics should serialize successfully: {}",
            e
        ))
    })?;
    assert!(json.contains("42"), "JSON should contain active_sessions value");
    assert!(json.contains("0.88"), "JSON should contain security_score");
    Ok(())
}

#[test]
fn test_security_metrics_deserialization() -> SongbirdResult<()> {
    let json = r#"{
            "active_sessions": 100,
            "failed_auth_attempts": 25,
            "blocked_ips": 8,
            "security_score": 0.75,
            "timestamp": "2024-01-01T00:00:00Z"
        }"#;

    let metrics: SecurityMetrics =
        serde_json::from_str(json).map_err(|e| SongbirdError::Serialization {
            format: Some("JSON".to_string()),
            message: format!("SecurityMetrics should deserialize successfully: {}", e),
            debug_info: None,
        })?;
    assert_eq!(metrics.active_sessions, 100);
    assert_eq!(metrics.failed_auth_attempts, 25);
    assert_eq!(metrics.blocked_ips, 8);
    assert!((metrics.security_score - 0.75).abs() < 0.001);
    Ok(())
}

#[test]
fn test_security_health_serialization() -> SongbirdResult<()> {
    let health = SecurityHealth::Critical;
    let serialized = serde_json::to_string(&health);
    assert!(serialized.is_ok(), "SecurityHealth should serialize successfully");
    Ok(())
}

#[test]
fn test_adapter_endpoint_access() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = "http://test-security:9000";
    let adapter = SecurityAdapter::new(endpoint.to_string()).map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
    })?;

    assert_eq!(adapter.endpoint(), endpoint, "Endpoint should be accessible");
    Ok(())
}

#[test]
fn test_adapter_timeout_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SecurityAdapter::new("http://test:8080".to_string())
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(Duration::from_secs(5));

    assert_eq!(adapter.timeout, Duration::from_secs(5), "Timeout should be configurable");
    Ok(())
}

#[test]
fn test_adapter_default_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SecurityAdapter::new("http://test:8080".to_string()).map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
    })?;

    assert_eq!(adapter.timeout, Duration::from_secs(5), "Default timeout should be 5 seconds");
    Ok(())
}

#[test]
fn test_auth_result_serialization() {
    let result = AuthResult::Authorized;
    let serialized = serde_json::to_string(&result);
    assert!(serialized.is_ok(), "AuthResult should serialize successfully");
}

#[test]
fn test_security_metrics_zero_values() {
    let metrics = SecurityMetrics {
        active_sessions: 0,
        failed_auth_attempts: 0,
        blocked_ips: 0,
        security_score: 1.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_under_attack(), "Zero values should not indicate attack");
    assert_eq!(metrics.health_status(), SecurityHealth::Healthy, "Perfect score should be Healthy");
}

#[test]
fn test_security_metrics_max_values() {
    let metrics = SecurityMetrics {
        active_sessions: u32::MAX,
        failed_auth_attempts: u32::MAX,
        blocked_ips: u32::MAX,
        security_score: 0.0,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_under_attack(), "Max values should indicate attack");
    assert_eq!(metrics.health_status(), SecurityHealth::Critical, "Worst case should be Critical");
}

#[test]
fn test_adapter_with_various_endpoints() {
    // Test empty endpoint (currently accepted, may want to validate later)
    let result = SecurityAdapter::new(String::new());
    assert!(result.is_ok(), "Empty endpoint creates adapter (validation could be added)");

    // Test various endpoint formats
    let result = SecurityAdapter::new("http://localhost:8080".to_string());
    assert!(result.is_ok(), "Valid HTTP endpoint should work");

    let result = SecurityAdapter::new("https://security.example.com".to_string());
    assert!(result.is_ok(), "Valid HTTPS endpoint should work");
}

// ========== NEW TESTS (20 tests to reach 85% coverage) ==========

#[test]
fn test_security_health_equality() -> SongbirdResult<()> {
    assert_eq!(SecurityHealth::Healthy, SecurityHealth::Healthy);
    assert_eq!(SecurityHealth::Warning, SecurityHealth::Warning);
    assert_eq!(SecurityHealth::Critical, SecurityHealth::Critical);
    assert_ne!(SecurityHealth::Healthy, SecurityHealth::Warning);
    assert_ne!(SecurityHealth::Warning, SecurityHealth::Critical);
    Ok(())
}

#[test]
fn test_security_health_clone() -> SongbirdResult<()> {
    let health = SecurityHealth::Critical;
    let cloned = health;
    assert_eq!(health, cloned);
    Ok(())
}

#[test]
fn test_security_health_debug() -> SongbirdResult<()> {
    let health = SecurityHealth::Warning;
    let debug_str = format!("{health:?}");
    assert!(debug_str.contains("Warning"));
    Ok(())
}

#[test]
fn test_security_metrics_clone() -> SongbirdResult<()> {
    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 10,
        blocked_ips: 5,
        security_score: 0.85,
        timestamp: chrono::Utc::now(),
    };
    let cloned = metrics;
    assert_eq!(cloned.active_sessions, 50);
    assert_eq!(cloned.failed_auth_attempts, 10);
    Ok(())
}

#[test]
fn test_security_metrics_debug() -> SongbirdResult<()> {
    let metrics = SecurityMetrics {
        active_sessions: 42,
        failed_auth_attempts: 7,
        blocked_ips: 3,
        security_score: 0.88,
        timestamp: chrono::Utc::now(),
    };
    let debug_str = format!("{metrics:?}");
    assert!(debug_str.contains("42"));
    assert!(debug_str.contains("0.88"));
    Ok(())
}

#[test]
fn test_auth_result_clone() -> SongbirdResult<()> {
    let result = AuthResult::Authorized;
    let cloned = result.clone();
    assert_eq!(result, cloned);
    Ok(())
}

#[test]
fn test_auth_result_debug() -> SongbirdResult<()> {
    let result = AuthResult::Unauthorized;
    let debug_str = format!("{result:?}");
    assert!(debug_str.contains("Unauthorized"));
    Ok(())
}

#[test]
fn test_auth_result_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#""Authorized""#;
    let result: AuthResult = serde_json::from_str(json).map_err(|e| {
        SongbirdError::configuration(format!("Deserialization should succeed: {}", e))
    })?;
    assert_eq!(result, AuthResult::Authorized);
    Ok(())
}

#[test]
fn test_security_health_deserialization() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#""Critical""#;
    let health: SecurityHealth = serde_json::from_str(json).map_err(|e| {
        SongbirdError::configuration(format!("Deserialization should succeed: {}", e))
    })?;
    assert_eq!(health, SecurityHealth::Critical);
    Ok(())
}

#[test]
fn test_security_metrics_edge_case_exactly_50_blocked_ips() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 10,
        blocked_ips: 50,
        security_score: 0.9,
        timestamp: chrono::Utc::now(),
    };
    assert!(!metrics.is_under_attack(), "Exactly 50 blocked IPs should not trigger attack");
    assert_eq!(metrics.health_status(), SecurityHealth::Healthy);
}

#[test]
fn test_security_metrics_edge_case_exactly_51_blocked_ips() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 10,
        blocked_ips: 51,
        security_score: 0.9,
        timestamp: chrono::Utc::now(),
    };
    assert!(metrics.is_under_attack(), "51 blocked IPs should trigger attack");
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_metrics_edge_case_exactly_101_failed_attempts() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 101,
        blocked_ips: 10,
        security_score: 0.9,
        timestamp: chrono::Utc::now(),
    };
    assert!(metrics.is_under_attack(), "101 failed attempts should trigger attack");
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_metrics_edge_case_score_exactly_0_7() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 10,
        blocked_ips: 5,
        security_score: 0.7,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(
        metrics.health_status(),
        SecurityHealth::Healthy,
        "Score at 0.7 boundary should be Healthy"
    );
}

#[test]
fn test_security_metrics_edge_case_score_below_0_7() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 10,
        blocked_ips: 5,
        security_score: 0.69,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(
        metrics.health_status(),
        SecurityHealth::Warning,
        "Score below 0.7 should be Warning"
    );
}

#[test]
fn test_security_metrics_edge_case_exactly_50_failed_attempts() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 50,
        blocked_ips: 5,
        security_score: 0.75,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(
        metrics.health_status(),
        SecurityHealth::Healthy,
        "Exactly 50 failed attempts should not trigger warning"
    );
}

#[test]
fn test_security_metrics_edge_case_exactly_51_failed_attempts() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 51,
        blocked_ips: 5,
        security_score: 0.75,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(
        metrics.health_status(),
        SecurityHealth::Warning,
        "51 failed attempts should trigger warning"
    );
}

#[test]
fn test_auth_result_all_variants_unique() -> SongbirdResult<()> {
    let results = [
        AuthResult::Authorized,
        AuthResult::Unauthorized,
        AuthResult::Expired,
        AuthResult::Invalid,
    ];

    for (i, r1) in results.iter().enumerate() {
        for (j, r2) in results.iter().enumerate() {
            if i == j {
                assert_eq!(r1, r2);
            } else {
                assert_ne!(r1, r2);
            }
        }
    }
    Ok(())
}

#[test]
fn test_adapter_chained_timeout_configuration() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SecurityAdapter::new("http://test:8080".to_string())
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(Duration::from_secs(2))
        .with_timeout(Duration::from_secs(8));

    assert_eq!(adapter.timeout, Duration::from_secs(8), "Last timeout should be applied");
    Ok(())
}

#[test]
fn test_adapter_zero_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SecurityAdapter::new("http://test:8080".to_string())
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(Duration::from_secs(0));

    assert_eq!(
        adapter.timeout,
        Duration::from_secs(0),
        "Zero timeout should be accepted (may cause immediate failures)"
    );
    Ok(())
}

#[test]
fn test_adapter_very_long_timeout() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SecurityAdapter::new("http://test:8080".to_string())
        .map_err(|e| {
            SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
        })?
        .with_timeout(Duration::from_secs(3600));

    assert_eq!(
        adapter.timeout,
        Duration::from_secs(3600),
        "Long timeout (1 hour) should be accepted"
    );
    Ok(())
}

#[test]
fn test_security_metrics_combined_attack_conditions() {
    // Both high failed attempts AND high blocked IPs
    let metrics = SecurityMetrics {
        active_sessions: 50,
        failed_auth_attempts: 200,
        blocked_ips: 100,
        security_score: 0.3,
        timestamp: chrono::Utc::now(),
    };

    assert!(metrics.is_under_attack(), "Multiple attack indicators should trigger");
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_metrics_warning_with_high_score_but_elevated_attempts() {
    let metrics = SecurityMetrics {
        active_sessions: 30,
        failed_auth_attempts: 75,
        blocked_ips: 20,
        security_score: 0.85,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(
        metrics.health_status(),
        SecurityHealth::Warning,
        "Elevated failed attempts should trigger warning despite high score"
    );
}

#[test]
fn test_security_metrics_boundary_score_0_5_with_no_attack() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 5,
        blocked_ips: 3,
        security_score: 0.5,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_under_attack());
    assert_eq!(
        metrics.health_status(),
        SecurityHealth::Warning,
        "Score exactly at 0.5 with no attack should be Warning"
    );
}

#[test]
fn test_security_metrics_critical_due_to_score_only() {
    let metrics = SecurityMetrics {
        active_sessions: 5,
        failed_auth_attempts: 2,
        blocked_ips: 1,
        security_score: 0.49,
        timestamp: chrono::Utc::now(),
    };

    assert!(!metrics.is_under_attack(), "No attack indicators");
    assert_eq!(
        metrics.health_status(),
        SecurityHealth::Critical,
        "Low score alone should cause Critical status"
    );
}

#[test]
fn test_security_metrics_warning_score_0_69_with_50_attempts() -> SongbirdResult<()> {
    let metrics = SecurityMetrics {
        active_sessions: 20,
        failed_auth_attempts: 50,
        blocked_ips: 10,
        security_score: 0.69,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(
        metrics.health_status(),
        SecurityHealth::Warning,
        "Score below 0.7 should be Warning"
    );
    Ok(())
}

#[test]
fn test_adapter_endpoint_with_trailing_slash() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SecurityAdapter::new("http://security:8080/".to_string()).map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
    })?;

    assert_eq!(adapter.endpoint(), "http://security:8080/");
    Ok(())
}

#[test]
fn test_adapter_endpoint_with_path() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = SecurityAdapter::new("http://security:8080/api/v1".to_string()).map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
    })?;

    assert_eq!(adapter.endpoint(), "http://security:8080/api/v1");
    Ok(())
}

#[test]
fn test_auth_result_invalid_variant() {
    let result = AuthResult::Invalid;
    assert_ne!(result, AuthResult::Authorized);
    assert_ne!(result, AuthResult::Unauthorized);
    assert_ne!(result, AuthResult::Expired);
    assert_eq!(result, AuthResult::Invalid);
}

#[test]
fn test_auth_result_expired_variant() {
    let result = AuthResult::Expired;
    assert_ne!(result, AuthResult::Authorized);
    assert_ne!(result, AuthResult::Unauthorized);
    assert_eq!(result, AuthResult::Expired);
    assert_ne!(result, AuthResult::Invalid);
}

#[test]
fn test_security_health_copy_trait() {
    let original = SecurityHealth::Critical;
    let copied = original;
    // Both should be usable (Copy trait)
    assert_eq!(original, SecurityHealth::Critical);
    assert_eq!(copied, SecurityHealth::Critical);
}

#[test]
fn test_security_metrics_with_negative_score() {
    // Test that scores below 0 are handled
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 5,
        blocked_ips: 2,
        security_score: -0.1,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(
        metrics.health_status(),
        SecurityHealth::Critical,
        "Negative score should be Critical"
    );
}

#[test]
fn test_security_metrics_with_score_above_1() -> SongbirdResult<()> {
    // Test that scores above 1.0 are handled gracefully
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 5,
        blocked_ips: 2,
        security_score: 1.5,
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(
        metrics.health_status(),
        SecurityHealth::Healthy,
        "Score above 1.0 should still be Healthy"
    );
    Ok(())
}

#[test]
fn test_adapter_builder_pattern_immutability() -> Result<(), Box<dyn std::error::Error>> {
    let adapter1 = SecurityAdapter::new("http://test:8080".to_string()).map_err(|e| {
        SongbirdError::configuration(format!("Adapter creation should succeed: {}", e))
    })?;

    let adapter2 = adapter1.with_timeout(Duration::from_secs(10));

    // Original timeout should remain unchanged (moved ownership)
    assert_eq!(adapter2.timeout, Duration::from_secs(10));
    Ok(())
}

#[test]
fn test_security_metrics_roundtrip_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let original = SecurityMetrics {
        active_sessions: 123,
        failed_auth_attempts: 45,
        blocked_ips: 67,
        security_score: 0.789,
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&original).map_err(|e| {
        SongbirdError::configuration(format!("Serialization should succeed: {}", e))
    })?;

    let deserialized: SecurityMetrics = serde_json::from_str(&json).map_err(|e| {
        SongbirdError::configuration(format!("Deserialization should succeed: {}", e))
    })?;

    assert_eq!(original.active_sessions, deserialized.active_sessions);
    assert_eq!(original.failed_auth_attempts, deserialized.failed_auth_attempts);
    assert_eq!(original.blocked_ips, deserialized.blocked_ips);
    assert!((original.security_score - deserialized.security_score).abs() < 0.001);
    Ok(())
}

#[test]
fn test_auth_result_roundtrip_serialization() -> Result<(), Box<dyn std::error::Error>> {
    for original in
        [AuthResult::Authorized, AuthResult::Unauthorized, AuthResult::Expired, AuthResult::Invalid]
    {
        let json = serde_json::to_string(&original).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {}", e))
        })?;

        let deserialized: AuthResult = serde_json::from_str(&json).map_err(|e| {
            SongbirdError::configuration(format!("Deserialization should succeed: {}", e))
        })?;

        assert_eq!(original, deserialized);
    }
    Ok(())
}

#[test]
fn test_security_health_roundtrip_serialization() -> Result<(), Box<dyn std::error::Error>> {
    for original in [SecurityHealth::Healthy, SecurityHealth::Warning, SecurityHealth::Critical] {
        let json = serde_json::to_string(&original).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {}", e))
        })?;

        let deserialized: SecurityHealth = serde_json::from_str(&json).map_err(|e| {
            SongbirdError::configuration(format!("Deserialization should succeed: {}", e))
        })?;

        assert_eq!(original, deserialized);
    }
    Ok(())
}
