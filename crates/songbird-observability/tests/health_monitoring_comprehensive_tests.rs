// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate,
    reason = "test assertions and harness ergonomics"
)]

//! Comprehensive Health Monitoring Tests
#![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![allow(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![allow(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![allow(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
#![allow(clippy::similar_names, reason = "test assertions and harness ergonomics")]
#![allow(clippy::too_many_lines, reason = "test assertions and harness ergonomics")]
#![allow(clippy::module_name_repetitions, reason = "test assertions and harness ergonomics")]
#![allow(clippy::uninlined_format_args, reason = "test assertions and harness ergonomics")]
#![allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
#![allow(clippy::useless_vec, reason = "test assertions and harness ergonomics")]
#![allow(clippy::unreadable_literal, reason = "test assertions and harness ergonomics")]
#![allow(clippy::items_after_statements, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_precision_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_possible_truncation, reason = "test assertions and harness ergonomics")]
#![allow(clippy::cast_sign_loss, reason = "test assertions and harness ergonomics")]
#![allow(clippy::needless_pass_by_value, reason = "test assertions and harness ergonomics")]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions and harness ergonomics"
)]

//!
//! Additional test coverage for health monitoring functionality

use songbird_observability::health::{
    HealthCheckResult, HealthChecker, HealthState, HealthStatus, HealthStatusDetails,
    HealthThresholds,
};
use std::collections::HashMap;

#[test]
fn test_health_state_equality() {
    assert_eq!(HealthState::Healthy, HealthState::Healthy);
    assert_eq!(HealthState::Degraded, HealthState::Degraded);
    assert_eq!(HealthState::Unhealthy, HealthState::Unhealthy);
    assert_eq!(HealthState::Critical, HealthState::Critical);
    assert_eq!(HealthState::Unknown, HealthState::Unknown);
    assert_eq!(HealthState::Maintenance, HealthState::Maintenance);
}

#[test]
fn test_health_state_inequality() {
    assert_ne!(HealthState::Healthy, HealthState::Degraded);
    assert_ne!(HealthState::Degraded, HealthState::Unhealthy);
    assert_ne!(HealthState::Unhealthy, HealthState::Critical);
    assert_ne!(HealthState::Critical, HealthState::Unknown);
    assert_ne!(HealthState::Unknown, HealthState::Maintenance);
}

#[test]
fn test_health_status_equality() {
    assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
    assert_eq!(HealthStatus::Degraded, HealthStatus::Degraded);
    assert_eq!(HealthStatus::Unhealthy, HealthStatus::Unhealthy);
}

#[test]
fn test_health_status_inequality() {
    assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
    assert_ne!(HealthStatus::Degraded, HealthStatus::Unhealthy);
    assert_ne!(HealthStatus::Unhealthy, HealthStatus::Healthy);
}

#[test]
fn test_health_checker_creation() {
    let _checker = HealthChecker::new();
    // HealthChecker created successfully
}

#[test]
fn test_health_checker_default() {
    let _checker = HealthChecker::default();
    // HealthChecker created successfully
}

#[test]
fn test_health_check_result_creation() {
    let result = HealthCheckResult {
        name: "test-check".to_string(),
        status: HealthStatus::Healthy,
        message: "All systems operational".to_string(),
        response_time_ms: 50,
    };

    assert_eq!(result.name, "test-check");
    assert_eq!(result.status, HealthStatus::Healthy);
    assert_eq!(result.message, "All systems operational");
    assert_eq!(result.response_time_ms, 50);
}

#[test]
fn test_health_check_result_with_different_statuses() {
    let healthy = HealthCheckResult {
        name: "healthy-check".to_string(),
        status: HealthStatus::Healthy,
        message: "OK".to_string(),
        response_time_ms: 10,
    };

    let degraded = HealthCheckResult {
        name: "degraded-check".to_string(),
        status: HealthStatus::Degraded,
        message: "Slow response".to_string(),
        response_time_ms: 500,
    };

    let unhealthy = HealthCheckResult {
        name: "unhealthy-check".to_string(),
        status: HealthStatus::Unhealthy,
        message: "Service unavailable".to_string(),
        response_time_ms: 5000,
    };

    assert_eq!(healthy.status, HealthStatus::Healthy);
    assert_eq!(degraded.status, HealthStatus::Degraded);
    assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
}

#[test]
fn test_health_status_details_creation() {
    let mut metadata = HashMap::new();
    metadata.insert("region".to_string(), serde_json::json!("us-west-1"));
    metadata.insert("version".to_string(), serde_json::json!("1.0.0"));

    let details = HealthStatusDetails {
        state: HealthState::Healthy,
        score: 0.95,
        checks_passed: 10,
        checks_failed: 0,
        last_updated: std::time::SystemTime::now(),
        metadata,
    };

    assert_eq!(details.state, HealthState::Healthy);
    assert!((details.score - 0.95).abs() < 0.001);
    assert_eq!(details.checks_passed, 10);
    assert_eq!(details.checks_failed, 0);
    assert_eq!(details.metadata.len(), 2);
}

#[test]
fn test_health_status_details_with_failures() {
    let details = HealthStatusDetails {
        state: HealthState::Degraded,
        score: 0.75,
        checks_passed: 7,
        checks_failed: 3,
        last_updated: std::time::SystemTime::now(),
        metadata: HashMap::new(),
    };

    assert_eq!(details.state, HealthState::Degraded);
    assert_eq!(details.checks_passed, 7);
    assert_eq!(details.checks_failed, 3);
    assert!((details.score - 0.75).abs() < 0.001);
}

#[test]
fn test_health_status_details_critical() {
    let details = HealthStatusDetails {
        state: HealthState::Critical,
        score: 0.25,
        checks_passed: 2,
        checks_failed: 8,
        last_updated: std::time::SystemTime::now(),
        metadata: HashMap::new(),
    };

    assert_eq!(details.state, HealthState::Critical);
    assert_eq!(details.checks_passed, 2);
    assert_eq!(details.checks_failed, 8);
    assert!(details.score < 0.5);
}

#[test]
fn test_health_thresholds_creation() {
    let thresholds = HealthThresholds {
        response_time_threshold: std::time::Duration::from_millis(500),
        error_rate_threshold: 0.05,
        cpu_threshold: 80.0,
        memory_threshold: 85.0,
        disk_threshold: 90.0,
        failure_count_threshold: 5,
    };

    assert_eq!(thresholds.response_time_threshold, std::time::Duration::from_millis(500));
    assert!((thresholds.error_rate_threshold - 0.05).abs() < 0.001);
    assert!((thresholds.cpu_threshold - 80.0).abs() < 0.001);
    assert!((thresholds.memory_threshold - 85.0).abs() < 0.001);
    assert!((thresholds.disk_threshold - 90.0).abs() < 0.001);
    assert_eq!(thresholds.failure_count_threshold, 5);
}

#[test]
fn test_health_thresholds_strict() {
    let strict = HealthThresholds {
        response_time_threshold: std::time::Duration::from_millis(100),
        error_rate_threshold: 0.01,
        cpu_threshold: 70.0,
        memory_threshold: 75.0,
        disk_threshold: 80.0,
        failure_count_threshold: 2,
    };

    assert!(strict.response_time_threshold < std::time::Duration::from_millis(500));
    assert!(strict.error_rate_threshold < 0.05);
    assert!(strict.cpu_threshold < 80.0);
    assert!(strict.memory_threshold < 85.0);
    assert!(strict.disk_threshold < 90.0);
    assert!(strict.failure_count_threshold < 5);
}

#[test]
fn test_health_thresholds_relaxed() {
    let relaxed = HealthThresholds {
        response_time_threshold: std::time::Duration::from_secs(2),
        error_rate_threshold: 0.10,
        cpu_threshold: 90.0,
        memory_threshold: 95.0,
        disk_threshold: 95.0,
        failure_count_threshold: 10,
    };

    assert!(relaxed.response_time_threshold > std::time::Duration::from_millis(500));
    assert!(relaxed.error_rate_threshold > 0.05);
    assert!(relaxed.cpu_threshold > 80.0);
    assert!(relaxed.memory_threshold > 85.0);
}

#[test]
fn test_health_check_result_fast_response() {
    let result = HealthCheckResult {
        name: "fast-api".to_string(),
        status: HealthStatus::Healthy,
        message: "Quick response".to_string(),
        response_time_ms: 5,
    };

    assert!(result.response_time_ms < 100);
    assert_eq!(result.status, HealthStatus::Healthy);
}

#[test]
fn test_health_check_result_slow_response() {
    let result = HealthCheckResult {
        name: "slow-api".to_string(),
        status: HealthStatus::Degraded,
        message: "Slow response detected".to_string(),
        response_time_ms: 1500,
    };

    assert!(result.response_time_ms > 1000);
    assert_eq!(result.status, HealthStatus::Degraded);
}

#[test]
fn test_health_check_result_timeout() {
    let result = HealthCheckResult {
        name: "timeout-api".to_string(),
        status: HealthStatus::Unhealthy,
        message: "Request timed out".to_string(),
        response_time_ms: 30_000,
    };

    assert!(result.response_time_ms > 10_000);
    assert_eq!(result.status, HealthStatus::Unhealthy);
}

#[test]
fn test_health_status_details_score_ranges() {
    let excellent = HealthStatusDetails {
        state: HealthState::Healthy,
        score: 1.0,
        checks_passed: 10,
        checks_failed: 0,
        last_updated: std::time::SystemTime::now(),
        metadata: HashMap::new(),
    };

    let good = HealthStatusDetails {
        state: HealthState::Healthy,
        score: 0.85,
        checks_passed: 9,
        checks_failed: 1,
        last_updated: std::time::SystemTime::now(),
        metadata: HashMap::new(),
    };

    let acceptable = HealthStatusDetails {
        state: HealthState::Degraded,
        score: 0.70,
        checks_passed: 7,
        checks_failed: 3,
        last_updated: std::time::SystemTime::now(),
        metadata: HashMap::new(),
    };

    assert!(excellent.score > good.score);
    assert!(good.score > acceptable.score);
}

#[test]
fn test_health_state_clone() {
    let state1 = HealthState::Healthy;
    let state2 = state1.clone();
    assert_eq!(state1, state2);
}

#[test]
fn test_health_status_clone() {
    let status1 = HealthStatus::Degraded;
    let status2 = status1.clone();
    assert_eq!(status1, status2);
}

#[test]
fn test_health_check_result_clone() {
    let result1 = HealthCheckResult {
        name: "test".to_string(),
        status: HealthStatus::Healthy,
        message: "OK".to_string(),
        response_time_ms: 100,
    };

    let result2 = result1.clone();
    assert_eq!(result1.name, result2.name);
    assert_eq!(result1.status, result2.status);
    assert_eq!(result1.response_time_ms, result2.response_time_ms);
}

#[test]
fn test_health_status_details_clone() {
    let details1 = HealthStatusDetails {
        state: HealthState::Healthy,
        score: 0.9,
        checks_passed: 9,
        checks_failed: 1,
        last_updated: std::time::SystemTime::now(),
        metadata: HashMap::new(),
    };

    let details2 = details1.clone();
    assert_eq!(details1.state, details2.state);
    assert!((details1.score - details2.score).abs() < 0.001);
}
