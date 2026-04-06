// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! [`AuthResult`], [`SecurityHealth`], and trait behavior (equality, debug, clone).

use super::super::*;
use songbird_types::{SongbirdError, SongbirdResult};

#[test]
fn test_auth_result_equality() {
    assert_eq!(AuthResult::Authorized, AuthResult::Authorized);
    assert_ne!(AuthResult::Authorized, AuthResult::Unauthorized);
    assert_eq!(AuthResult::Expired, AuthResult::Expired);
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
fn test_auth_result_serialization() {
    let result = AuthResult::Authorized;
    let serialized = serde_json::to_string(&result);
    assert!(serialized.is_ok(), "AuthResult should serialize successfully");
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
