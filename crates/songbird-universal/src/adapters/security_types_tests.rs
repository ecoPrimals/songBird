// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;

#[test]
fn test_security_metrics_is_under_attack_threshold_failed_auth() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 101,
        blocked_ips: 5,
        security_score: 0.8,
        timestamp: chrono::Utc::now(),
    };
    assert!(metrics.is_under_attack());
}

#[test]
fn test_security_metrics_is_under_attack_threshold_blocked_ips() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 10,
        blocked_ips: 51,
        security_score: 0.8,
        timestamp: chrono::Utc::now(),
    };
    assert!(metrics.is_under_attack());
}

#[test]
fn test_security_metrics_not_under_attack() {
    let metrics = SecurityMetrics::healthy();
    assert!(!metrics.is_under_attack());
}

#[test]
fn test_security_metrics_health_status_healthy() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 10,
        blocked_ips: 5,
        security_score: 0.9,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), SecurityHealth::Healthy);
}

#[test]
fn test_security_metrics_health_status_warning() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 60,
        blocked_ips: 10,
        security_score: 0.65,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), SecurityHealth::Warning);
}

#[test]
fn test_security_metrics_health_status_critical_low_score() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 10,
        blocked_ips: 5,
        security_score: 0.4,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_metrics_health_status_critical_under_attack() {
    let metrics = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 150,
        blocked_ips: 80,
        security_score: 0.9,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics.health_status(), SecurityHealth::Critical);
}

#[test]
fn test_security_health_equality() {
    assert_eq!(SecurityHealth::Healthy, SecurityHealth::Healthy);
    assert_ne!(SecurityHealth::Healthy, SecurityHealth::Warning);
    assert_ne!(SecurityHealth::Warning, SecurityHealth::Critical);
}

#[test]
fn test_auth_result_equality() {
    assert_eq!(AuthResult::Authorized, AuthResult::Authorized);
    assert_ne!(AuthResult::Authorized, AuthResult::Unauthorized);
    assert_ne!(AuthResult::Expired, AuthResult::Invalid);
}

#[test]
fn test_security_metrics_serialization() {
    let metrics = SecurityMetrics::healthy();
    let json = serde_json::to_string(&metrics).unwrap();
    assert!(json.contains("active_sessions"));
    assert!(json.contains("security_score"));
}
