// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Core [`SecurityMetrics`] health classification and boundary tests.

use super::super::*;
use songbird_types::SongbirdResult;

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
fn test_security_health_boundary_conditions() {
    // Test just below critical boundary - score 0.49 (< 0.5)
    let metrics_critical = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 10,
        blocked_ips: 5,
        security_score: 0.49,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics_critical.health_status(), SecurityHealth::Critical);

    // Test exactly at boundary - score 0.5 (not < 0.5, so Warning)
    let metrics_boundary = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 10,
        blocked_ips: 5,
        security_score: 0.5,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics_boundary.health_status(), SecurityHealth::Warning);

    // Test just below warning boundary - score 0.69 (< 0.7)
    let metrics_warning = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 10,
        blocked_ips: 5,
        security_score: 0.69,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics_warning.health_status(), SecurityHealth::Warning);

    // Test exactly at warning boundary - score 0.7 (not < 0.7, so Healthy)
    let metrics_healthy = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 10,
        blocked_ips: 5,
        security_score: 0.7,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics_healthy.health_status(), SecurityHealth::Healthy);
}

#[test]
fn test_security_health_failed_attempts_boundary() {
    // Test exactly at failed attempts boundary - 50
    let metrics_boundary = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 50,
        blocked_ips: 5,
        security_score: 0.75,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics_boundary.health_status(), SecurityHealth::Healthy);

    // Test just over boundary - 51
    let metrics_over = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 51,
        blocked_ips: 5,
        security_score: 0.75,
        timestamp: chrono::Utc::now(),
    };
    assert_eq!(metrics_over.health_status(), SecurityHealth::Warning);
}

#[test]
fn test_security_under_attack_boundary_blocked_ips() {
    // Test exactly at blocked IPs boundary - 50
    let metrics_boundary = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 10,
        blocked_ips: 50,
        security_score: 0.95,
        timestamp: chrono::Utc::now(),
    };
    assert!(!metrics_boundary.is_under_attack());

    // Test just over boundary - 51
    let metrics_over = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 10,
        blocked_ips: 51,
        security_score: 0.95,
        timestamp: chrono::Utc::now(),
    };
    assert!(metrics_over.is_under_attack());
}

#[test]
fn test_security_under_attack_boundary_failed_auth() {
    // Test exactly at failed auth boundary - 100
    let metrics_boundary = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 100,
        blocked_ips: 5,
        security_score: 0.95,
        timestamp: chrono::Utc::now(),
    };
    assert!(!metrics_boundary.is_under_attack());

    // Test just over boundary - 101
    let metrics_over = SecurityMetrics {
        active_sessions: 10,
        failed_auth_attempts: 101,
        blocked_ips: 5,
        security_score: 0.95,
        timestamp: chrono::Utc::now(),
    };
    assert!(metrics_over.is_under_attack());
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
