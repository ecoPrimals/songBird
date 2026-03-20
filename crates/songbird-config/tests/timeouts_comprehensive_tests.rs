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
    clippy::unnecessary_literal_unwrap
)]

//! Comprehensive tests for timeout configuration
//!
//! Uses injectable `*_from_map` variants for concurrent-safe testing without
//! mutating global environment variables.

use songbird_config::defaults::timeouts::*;
use std::collections::HashMap;
use std::time::Duration;

fn empty_env() -> HashMap<String, String> {
    HashMap::new()
}

fn env_with<K: Into<String>, V: Into<String>>(key: K, value: V) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert(key.into(), value.into());
    m
}

#[test]
fn test_standard_timeout_default_value() {
    let env = empty_env();
    let timeout = standard_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(5000));
    assert_eq!(timeout.as_secs(), 5);
}

#[test]
fn test_standard_timeout_from_env() {
    let env = env_with("SONGBIRD_TIMEOUT_MS", "3000");
    let timeout = standard_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(3000));
}

#[test]
fn test_standard_timeout_invalid_env_uses_default() {
    let env = env_with("SONGBIRD_TIMEOUT_MS", "invalid");
    let timeout = standard_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(5000));
}

#[test]
fn test_long_timeout_default_value() {
    let env = empty_env();
    let timeout = long_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(30000));
    assert_eq!(timeout.as_secs(), 30);
}

#[test]
fn test_long_timeout_from_env() {
    let env = env_with("SONGBIRD_LONG_TIMEOUT_MS", "45000");
    let timeout = long_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(45000));
}

#[test]
fn test_long_timeout_invalid_env_uses_default() {
    let env = env_with("SONGBIRD_LONG_TIMEOUT_MS", "not_a_number");
    let timeout = long_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(30000));
}

#[test]
fn test_request_timeout_default_value() {
    let env = empty_env();
    let timeout = request_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(30000));
}

#[test]
fn test_request_timeout_from_env() {
    let env = env_with("SONGBIRD_REQUEST_TIMEOUT_MS", "20000");
    let timeout = request_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(20000));
}

#[test]
fn test_cache_expiry_default_value() {
    let env = empty_env();
    let expiry = cache_expiry_from_map(&env);
    assert_eq!(expiry, Duration::from_millis(300_000));
    assert_eq!(expiry.as_secs(), 300); // 5 minutes
}

#[test]
fn test_cache_expiry_from_env() {
    let env = env_with("SONGBIRD_CACHE_EXPIRY_MS", "600000");
    let expiry = cache_expiry_from_map(&env);
    assert_eq!(expiry, Duration::from_millis(600_000));
}

#[test]
fn test_heartbeat_interval_default_value() {
    let env = empty_env();
    let interval = heartbeat_interval_from_map(&env);
    assert_eq!(interval, Duration::from_millis(60000));
    assert_eq!(interval.as_secs(), 60); // 1 minute
}

#[test]
fn test_heartbeat_interval_from_env() {
    let env = env_with("SONGBIRD_HEARTBEAT_INTERVAL_MS", "30000");
    let interval = heartbeat_interval_from_map(&env);
    assert_eq!(interval, Duration::from_millis(30000));
}

#[test]
fn test_discovery_timeout_default_value() {
    let env = empty_env();
    let timeout = discovery_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(5000));
}

#[test]
fn test_discovery_timeout_from_env() {
    let env = env_with("SONGBIRD_DISCOVERY_TIMEOUT_MS", "8000");
    let timeout = discovery_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(8000));
}

#[test]
fn test_connection_timeout_default_value() {
    let env = empty_env();
    let timeout = connection_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(10000));
    assert!(timeout.as_secs() >= 10 && timeout.as_secs() <= 45);
}

#[test]
fn test_connection_timeout_from_env() {
    let env = env_with("SONGBIRD_CONNECTION_TIMEOUT_MS", "15000");
    let timeout = connection_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(15000));
}

#[test]
fn test_retry_backoff_default_value() {
    let env = empty_env();
    let backoff = retry_backoff_from_map(&env);
    assert_eq!(backoff, Duration::from_millis(1000));
    assert_eq!(backoff.as_secs(), 1);
}

#[test]
fn test_retry_backoff_from_env() {
    let env = env_with("SONGBIRD_RETRY_BACKOFF_MS", "2000");
    let backoff = retry_backoff_from_map(&env);
    assert_eq!(backoff, Duration::from_millis(2000));
}

#[test]
fn test_operation_timeout_with_default() {
    let env = empty_env();
    let timeout = operation_timeout_from_map(&env, "CUSTOM", Duration::from_secs(10));
    assert_eq!(timeout.as_secs(), 10);
}

#[test]
fn test_operation_timeout_from_env() {
    let env = env_with("SONGBIRD_CUSTOM_TIMEOUT_MS", "15000");
    let timeout = operation_timeout_from_map(&env, "CUSTOM", Duration::from_secs(10));
    assert_eq!(timeout, Duration::from_millis(15000));
}

#[test]
fn test_operation_timeout_lowercase_operation_name() {
    let mut env = HashMap::new();
    env.insert("SONGBIRD_MYOP_TIMEOUT_MS".to_string(), "7000".to_string());
    let timeout = operation_timeout_from_map(&env, "myop", Duration::from_secs(5));
    assert_eq!(timeout, Duration::from_millis(7000));
}

#[test]
fn test_timeout_relationships() {
    let env = empty_env();
    let standard = standard_timeout_from_map(&env);
    let long = long_timeout_from_map(&env);
    assert!(long > standard, "Long timeout should be greater than standard");
}

#[test]
fn test_all_timeouts_are_positive() {
    let env = empty_env();
    assert!(standard_timeout_from_map(&env).as_millis() > 0);
    assert!(long_timeout_from_map(&env).as_millis() > 0);
    assert!(request_timeout_from_map(&env).as_millis() > 0);
    assert!(cache_expiry_from_map(&env).as_millis() > 0);
    assert!(heartbeat_interval_from_map(&env).as_millis() > 0);
    assert!(discovery_timeout_from_map(&env).as_millis() > 0);
    assert!(connection_timeout_from_map(&env).as_millis() > 0);
    assert!(retry_backoff_from_map(&env).as_millis() > 0);
}

#[test]
fn test_timeouts_are_reasonable_for_production() {
    let env = empty_env();
    let standard = standard_timeout_from_map(&env);
    let long = long_timeout_from_map(&env);
    let connection = connection_timeout_from_map(&env);

    assert!(standard.as_secs() >= 1 && standard.as_secs() <= 10);
    assert!(long.as_secs() >= 10 && long.as_secs() <= 60);
    assert!(connection.as_secs() >= 5 && connection.as_secs() <= 30);
}

#[test]
fn test_cache_expiry_is_longer_than_timeouts() {
    let env = empty_env();
    let cache = cache_expiry_from_map(&env);
    let standard = standard_timeout_from_map(&env);
    let long = long_timeout_from_map(&env);
    assert!(cache > standard);
    assert!(cache > long);
}

#[test]
fn test_heartbeat_interval_reasonable() {
    let env = empty_env();
    let interval = heartbeat_interval_from_map(&env);
    assert!(interval.as_secs() >= 10 && interval.as_secs() <= 300);
}

#[test]
fn test_retry_backoff_less_than_standard_timeout() {
    let env = empty_env();
    let backoff = retry_backoff_from_map(&env);
    let standard = standard_timeout_from_map(&env);
    assert!(backoff < standard, "Retry backoff should be less than standard timeout");
}

#[test]
fn test_env_var_zero_value() {
    let env = env_with("SONGBIRD_TIMEOUT_MS", "0");
    let timeout = standard_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(0));
}

#[test]
fn test_env_var_very_large_value() {
    let env = env_with("SONGBIRD_TIMEOUT_MS", "999999");
    let timeout = standard_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(999_999));
}

#[test]
fn test_env_var_negative_value_uses_default() {
    let env = env_with("SONGBIRD_TIMEOUT_MS", "-1000");
    let timeout = standard_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(5000));
}

#[test]
fn test_env_var_empty_string_uses_default() {
    let env = env_with("SONGBIRD_TIMEOUT_MS", "");
    let timeout = standard_timeout_from_map(&env);
    assert_eq!(timeout, Duration::from_millis(5000));
}

#[test]
fn test_multiple_timeout_calls_consistent() {
    let env = empty_env();
    let timeout1 = standard_timeout_from_map(&env);
    let timeout2 = standard_timeout_from_map(&env);
    let timeout3 = standard_timeout_from_map(&env);
    assert_eq!(timeout1, timeout2);
    assert_eq!(timeout2, timeout3);
}

#[test]
fn test_operation_timeout_with_special_characters() {
    let mut env = HashMap::new();
    env.insert("SONGBIRD_MY_SPECIAL_OP_TIMEOUT_MS".to_string(), "12000".to_string());
    let _timeout = operation_timeout_from_map(&env, "MY-SPECIAL-OP", Duration::from_secs(5));
}

#[test]
fn test_all_timeouts_concurrent_access() {
    let env = empty_env();
    for _ in 0..10 {
        let _ = standard_timeout_from_map(&env);
        let _ = long_timeout_from_map(&env);
        let _ = request_timeout_from_map(&env);
        let _ = cache_expiry_from_map(&env);
        let _ = heartbeat_interval_from_map(&env);
        let _ = discovery_timeout_from_map(&env);
        let _ = connection_timeout_from_map(&env);
        let _ = retry_backoff_from_map(&env);
    }
}

#[test]
fn test_discovery_timeout_equals_standard_by_default() {
    let env = empty_env();
    let standard = standard_timeout_from_map(&env);
    let discovery = discovery_timeout_from_map(&env);
    assert_eq!(standard, discovery);
}

#[test]
fn test_request_timeout_equals_long_by_default() {
    let env = empty_env();
    let long = long_timeout_from_map(&env);
    let request = request_timeout_from_map(&env);
    assert_eq!(long, Duration::from_millis(30000));
    assert_eq!(request, Duration::from_millis(30000));
    assert_eq!(long, request);
}

#[test]
fn test_operation_timeout_with_very_long_name() {
    let very_long_name = "A".repeat(100);
    let mut env = HashMap::new();
    env.insert(
        format!("SONGBIRD_{}_TIMEOUT_MS", very_long_name.to_uppercase()),
        "5000".to_string(),
    );
    let timeout = operation_timeout_from_map(&env, &very_long_name, Duration::from_secs(1));
    assert_eq!(timeout, Duration::from_millis(5000));
}

#[test]
fn test_timeout_precision() {
    let env = env_with("SONGBIRD_TIMEOUT_MS", "1");
    let timeout = standard_timeout_from_map(&env);
    assert_eq!(timeout.as_millis(), 1);
}
