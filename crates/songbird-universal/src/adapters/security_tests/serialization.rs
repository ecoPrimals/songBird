// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! JSON serialization and round-trip tests for security adapter types.

use super::super::*;
use songbird_types::{SongbirdError, SongbirdResult};

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
        SongbirdError::configuration(format!("SecurityMetrics should serialize successfully: {e}"))
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
            message: format!("SecurityMetrics should deserialize successfully: {e}"),
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
fn test_security_metrics_roundtrip_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let original = SecurityMetrics {
        active_sessions: 123,
        failed_auth_attempts: 45,
        blocked_ips: 67,
        security_score: 0.789,
        timestamp: chrono::Utc::now(),
    };

    let json = serde_json::to_string(&original)
        .map_err(|e| SongbirdError::configuration(format!("Serialization should succeed: {e}")))?;

    let deserialized: SecurityMetrics = serde_json::from_str(&json).map_err(|e| {
        SongbirdError::configuration(format!("Deserialization should succeed: {e}"))
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
            SongbirdError::configuration(format!("Serialization should succeed: {e}"))
        })?;

        let deserialized: AuthResult = serde_json::from_str(&json).map_err(|e| {
            SongbirdError::configuration(format!("Deserialization should succeed: {e}"))
        })?;

        assert_eq!(original, deserialized);
    }
    Ok(())
}

#[test]
fn test_security_health_roundtrip_serialization() -> Result<(), Box<dyn std::error::Error>> {
    for original in [SecurityHealth::Healthy, SecurityHealth::Warning, SecurityHealth::Critical] {
        let json = serde_json::to_string(&original).map_err(|e| {
            SongbirdError::configuration(format!("Serialization should succeed: {e}"))
        })?;

        let deserialized: SecurityHealth = serde_json::from_str(&json).map_err(|e| {
            SongbirdError::configuration(format!("Deserialization should succeed: {e}"))
        })?;

        assert_eq!(original, deserialized);
    }
    Ok(())
}
