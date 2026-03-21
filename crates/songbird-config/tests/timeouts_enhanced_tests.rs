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
    clippy::unnecessary_literal_unwrap,
    reason = "test assertions and harness ergonomics"
)]

//! Enhanced timeout configuration tests

use songbird_config::defaults::timeouts::*;
use std::time::Duration;
#[test]
fn test_standard_timeout_default() {
    let timeout = standard_timeout();
    assert_eq!(timeout, Duration::from_millis(5000));
}

#[test]
fn test_long_timeout_default() {
    let timeout = long_timeout();
    assert_eq!(timeout, Duration::from_millis(30000));
}

#[test]
fn test_request_timeout_default() {
    let timeout = request_timeout();
    assert_eq!(timeout, Duration::from_millis(30000));
}

#[test]
fn test_cache_expiry_default() {
    let expiry = cache_expiry();
    assert_eq!(expiry, Duration::from_millis(300_000));
}

#[test]
fn test_heartbeat_interval_default() {
    let interval = heartbeat_interval();
    assert_eq!(interval, Duration::from_millis(60000));
}

#[test]
fn test_discovery_timeout_default() {
    let timeout = discovery_timeout();
    assert_eq!(timeout, Duration::from_millis(5000));
}

#[test]
fn test_connection_timeout_default() {
    let timeout = connection_timeout();
    assert_eq!(timeout, Duration::from_millis(10000));
}

#[test]
fn test_retry_backoff_default() {
    let backoff = retry_backoff();
    assert_eq!(backoff, Duration::from_millis(1000));
}

#[test]
fn test_timeouts_are_positive() {
    assert!(standard_timeout().as_millis() > 0);
    assert!(long_timeout().as_millis() > 0);
    assert!(request_timeout().as_millis() > 0);
    assert!(cache_expiry().as_millis() > 0);
    assert!(heartbeat_interval().as_millis() > 0);
    assert!(discovery_timeout().as_millis() > 0);
    assert!(connection_timeout().as_millis() > 0);
    assert!(retry_backoff().as_millis() > 0);
}

#[test]
fn test_timeout_ordering() {
    // Long timeout should be >= standard timeout
    assert!(long_timeout() >= standard_timeout());
    // Cache expiry should be longer than most operations
    assert!(cache_expiry() >= long_timeout());
    // Heartbeat should be longer than discovery
    assert!(heartbeat_interval() >= discovery_timeout());
}

#[test]
fn test_standard_timeout_as_secs() {
    let timeout = standard_timeout();
    assert_eq!(timeout.as_secs(), 5);
}

#[test]
fn test_long_timeout_as_secs() {
    let timeout = long_timeout();
    assert_eq!(timeout.as_secs(), 30);
}

#[test]
fn test_cache_expiry_as_mins() {
    let expiry = cache_expiry();
    let mins = expiry.as_secs() / 60;
    assert_eq!(mins, 5); // 5 minutes
}

#[test]
fn test_heartbeat_interval_as_mins() {
    let interval = heartbeat_interval();
    let mins = interval.as_secs() / 60;
    assert_eq!(mins, 1); // 1 minute
}

#[test]
fn test_operation_timeout_custom() {
    let timeout = operation_timeout("TEST", Duration::from_secs(10));
    assert_eq!(timeout.as_secs(), 10);
}

#[test]
fn test_operation_timeout_various_defaults() {
    let timeouts = vec![
        (operation_timeout("OP1", Duration::from_secs(1)), 1),
        (operation_timeout("OP2", Duration::from_secs(5)), 5),
        (operation_timeout("OP3", Duration::from_secs(10)), 10),
        (operation_timeout("OP4", Duration::from_secs(30)), 30),
    ];
    for (timeout, expected_secs) in timeouts {
        assert_eq!(timeout.as_secs(), expected_secs);
    }
}

#[test]
fn test_all_timeouts_consistent() {
    // Call each timeout function twice and ensure they return the same value
    assert_eq!(standard_timeout(), standard_timeout());
    assert_eq!(long_timeout(), long_timeout());
    assert_eq!(request_timeout(), request_timeout());
    assert_eq!(cache_expiry(), cache_expiry());
    assert_eq!(heartbeat_interval(), heartbeat_interval());
    assert_eq!(discovery_timeout(), discovery_timeout());
    assert_eq!(connection_timeout(), connection_timeout());
    assert_eq!(retry_backoff(), retry_backoff());
}

#[test]
fn test_timeout_durations_in_millis() {
    assert_eq!(standard_timeout().as_millis(), 5_000);
    assert_eq!(long_timeout().as_millis(), 30_000);
    assert_eq!(request_timeout().as_millis(), 30_000);
    assert_eq!(cache_expiry().as_millis(), 300_000);
    assert_eq!(heartbeat_interval().as_millis(), 60_000);
    assert_eq!(discovery_timeout().as_millis(), 5_000);
    assert_eq!(connection_timeout().as_millis(), 10_000);
    assert_eq!(retry_backoff().as_millis(), 1_000);
}

#[test]
fn test_timeout_durations_in_micros() {
    assert_eq!(standard_timeout().as_micros(), 5_000_000);
    assert_eq!(long_timeout().as_micros(), 30_000_000);
}

#[test]
fn test_retry_backoff_reasonable() {
    let backoff = retry_backoff();
    // Should be at least 100ms but less than 10 seconds
    assert!(backoff.as_millis() >= 100);
    assert!(backoff.as_millis() <= 10_000);
}

#[test]
fn test_connection_timeout_reasonable() {
    let timeout = connection_timeout();
    // Should be between 1 and 60 seconds
    assert!(timeout.as_secs() >= 1);
    assert!(timeout.as_secs() <= 60);
}

#[test]
fn test_discovery_timeout_reasonable() {
    let timeout = discovery_timeout();
    // Should be between 1 and 30 seconds
    assert!(timeout.as_secs() <= 30);
}
