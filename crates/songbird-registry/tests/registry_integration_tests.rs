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

//! Integration tests for registry operations
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
//! These tests verify registry functionality in realistic scenarios.

use songbird_registry::types::HealthStatus;
use std::time::Duration;

#[test]
fn test_health_status_healthy_creation() {
    // ACT
    let status = HealthStatus::healthy();

    // ASSERT
    assert!(status.healthy);
    assert!((status.score - 1.0).abs() < f64::EPSILON);
    assert!(status.message.is_none());
}

#[test]
fn test_health_status_unhealthy_creation() {
    // ACT
    let status = HealthStatus::unhealthy("Service unavailable");

    // ASSERT
    assert!(!status.healthy);
    assert!(status.score.abs() < f64::EPSILON);
    assert!(status.message.is_some());
    assert_eq!(status.message.expect("test precondition"), "Service unavailable");
}

#[test]
fn test_health_status_degraded_creation() {
    // ACT
    let status = HealthStatus::degraded(0.7, "High latency");

    // ASSERT
    assert!(status.healthy); // 0.7 > 0.5
    assert!((status.score - 0.7).abs() < f64::EPSILON);
    assert!(status.message.is_some());
}

#[test]
fn test_health_status_degraded_unhealthy_threshold() {
    // ACT
    let status = HealthStatus::degraded(0.3, "Very degraded");

    // ASSERT
    assert!(!status.healthy); // 0.3 < 0.5
    assert!((status.score - 0.3).abs() < f64::EPSILON);
}

#[test]
fn test_health_status_with_response_time() {
    // ACT
    let status = HealthStatus::healthy().with_response_time(Duration::from_millis(50));

    // ASSERT
    assert_eq!(status.response_time, Duration::from_millis(50));
}

#[test]
fn test_health_status_multiple_response_times() {
    // Test various response times
    let fast = HealthStatus::healthy().with_response_time(Duration::from_millis(10));
    let medium = HealthStatus::healthy().with_response_time(Duration::from_millis(100));
    let slow = HealthStatus::healthy().with_response_time(Duration::from_millis(1000));

    assert!(fast.response_time < medium.response_time);
    assert!(medium.response_time < slow.response_time);
}

#[test]
fn test_health_status_score_clamping() {
    // Test that scores outside 0.0-1.0 are clamped
    let too_high = HealthStatus::degraded(1.5, "Above max");
    let too_low = HealthStatus::degraded(-0.5, "Below min");

    assert!((too_high.score - 1.0).abs() < f64::EPSILON);
    assert!(too_low.score.abs() < f64::EPSILON);
}

#[test]
fn test_health_status_builder_pattern() {
    // Test chaining with_response_time
    let status = HealthStatus::healthy().with_response_time(Duration::from_millis(25));

    assert!(status.healthy);
    assert_eq!(status.response_time, Duration::from_millis(25));
}

#[test]
fn test_health_status_degraded_edge_cases() {
    // Test boundary conditions for degraded health
    let exactly_half = HealthStatus::degraded(0.5, "Exactly 50%");
    let just_above = HealthStatus::degraded(0.51, "Just above 50%");
    let just_below = HealthStatus::degraded(0.49, "Just below 50%");

    assert!(!exactly_half.healthy); // 0.5 is not > 0.5
    assert!(just_above.healthy); // 0.51 > 0.5
    assert!(!just_below.healthy); // 0.49 < 0.5
}

#[test]
fn test_health_status_various_scores() {
    // Test health status at various score levels
    let perfect = HealthStatus::degraded(1.0, "Perfect");
    let good = HealthStatus::degraded(0.9, "Good");
    let fair = HealthStatus::degraded(0.6, "Fair");
    let poor = HealthStatus::degraded(0.4, "Poor");
    let critical = HealthStatus::degraded(0.1, "Critical");
    let failed = HealthStatus::degraded(0.0, "Failed");

    assert!(perfect.healthy);
    assert!(good.healthy);
    assert!(fair.healthy);
    assert!(!poor.healthy);
    assert!(!critical.healthy);
    assert!(!failed.healthy);
}

#[test]
fn test_health_status_with_long_message() {
    // Test with a longer message
    let long_message = "Service experiencing high latency due to network congestion. \
                       Response times have increased by 200%. Investigating root cause.";
    let status = HealthStatus::degraded(0.6, long_message);

    assert_eq!(status.message.expect("test precondition"), long_message);
}

#[test]
fn test_health_status_unhealthy_with_empty_message() {
    // Test unhealthy with empty string
    let status = HealthStatus::unhealthy("");

    assert!(!status.healthy);
    assert!(status.message.is_some());
}

#[test]
fn test_health_status_response_time_zero() {
    // Test that default response time is zero
    let status = HealthStatus::healthy();

    assert_eq!(status.response_time, Duration::from_millis(0));
}

#[test]
fn test_health_status_response_time_large() {
    // Test with large response time
    let status = HealthStatus::unhealthy("Timeout").with_response_time(Duration::from_secs(30));

    assert_eq!(status.response_time.as_secs(), 30);
}

#[test]
fn test_health_status_clone() {
    // Test that HealthStatus can be cloned
    let status1 = HealthStatus::healthy();
    let status2 = status1.clone();

    assert_eq!(status1.healthy, status2.healthy);
    // Allow float comparison in test context
    #[allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
    {
        assert_eq!(status1.score, status2.score);
    }
}

#[test]
fn test_health_status_degraded_sequence() {
    // Test a sequence of degrading health
    let statuses = vec![
        HealthStatus::degraded(0.9, "Minor issue"),
        HealthStatus::degraded(0.7, "Moderate issue"),
        HealthStatus::degraded(0.5, "Significant issue"),
        HealthStatus::degraded(0.3, "Major issue"),
        HealthStatus::degraded(0.1, "Critical issue"),
    ];

    for (i, status) in statuses.iter().enumerate() {
        if i < 3 {
            #[allow(clippy::float_cmp, reason = "test assertions and harness ergonomics")]
            {
                assert!(status.healthy || status.score == 0.5);
            }
        } else {
            assert!(!status.healthy);
        }
    }
}
