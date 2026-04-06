// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Extreme values, edge boundaries, and combined attack indicators for [`SecurityMetrics`].

use super::super::*;
use songbird_types::SongbirdResult;

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

// ========== NEW TESTS (20 tests to reach 85% coverage) ==========

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
