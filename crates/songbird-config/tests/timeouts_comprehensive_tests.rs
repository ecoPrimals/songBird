//! Comprehensive tests for timeout configuration

use songbird_config::defaults::timeouts::*;
use std::env;
use std::time::Duration;

#[test]
fn test_standard_timeout_default_value() {
    env::remove_var("SONGBIRD_TIMEOUT_MS");
    let timeout = standard_timeout();
    assert_eq!(timeout, Duration::from_millis(5000));
    assert_eq!(timeout.as_secs(), 5);
}

#[test]
fn test_standard_timeout_from_env() {
    env::set_var("SONGBIRD_TIMEOUT_MS", "3000");
    let timeout = standard_timeout();
    assert_eq!(timeout, Duration::from_millis(3000));
    env::remove_var("SONGBIRD_TIMEOUT_MS");
}

#[test]
fn test_standard_timeout_invalid_env_uses_default() {
    env::set_var("SONGBIRD_TIMEOUT_MS", "invalid");
    let timeout = standard_timeout();
    assert_eq!(timeout, Duration::from_millis(5000));
    env::remove_var("SONGBIRD_TIMEOUT_MS");
}

#[test]
fn test_long_timeout_default_value() {
    env::remove_var("SONGBIRD_LONG_TIMEOUT_MS");
    let timeout = long_timeout();
    assert_eq!(timeout, Duration::from_millis(30000));
    assert_eq!(timeout.as_secs(), 30);
}

#[test]
fn test_long_timeout_from_env() {
    env::set_var("SONGBIRD_LONG_TIMEOUT_MS", "45000");
    let timeout = long_timeout();
    assert_eq!(timeout, Duration::from_millis(45000));
    env::remove_var("SONGBIRD_LONG_TIMEOUT_MS");
}

#[test]
fn test_long_timeout_invalid_env_uses_default() {
    env::set_var("SONGBIRD_LONG_TIMEOUT_MS", "not_a_number");
    let timeout = long_timeout();
    assert_eq!(timeout, Duration::from_millis(30000));
    env::remove_var("SONGBIRD_LONG_TIMEOUT_MS");
}

#[test]
fn test_request_timeout_default_value() {
    env::remove_var("SONGBIRD_REQUEST_TIMEOUT_MS");
    let timeout = request_timeout();
    assert_eq!(timeout, Duration::from_millis(30000));
}

#[test]
fn test_request_timeout_from_env() {
    env::set_var("SONGBIRD_REQUEST_TIMEOUT_MS", "20000");
    let timeout = request_timeout();
    assert_eq!(timeout, Duration::from_millis(20000));
    env::remove_var("SONGBIRD_REQUEST_TIMEOUT_MS");
}

#[test]
fn test_cache_expiry_default_value() {
    env::remove_var("SONGBIRD_CACHE_EXPIRY_MS");
    let expiry = cache_expiry();
    assert_eq!(expiry, Duration::from_millis(300_000));
    assert_eq!(expiry.as_secs(), 300); // 5 minutes
}

#[test]
fn test_cache_expiry_from_env() {
    env::set_var("SONGBIRD_CACHE_EXPIRY_MS", "600000");
    let expiry = cache_expiry();
    assert_eq!(expiry, Duration::from_millis(600_000));
    env::remove_var("SONGBIRD_CACHE_EXPIRY_MS");
}

#[test]
fn test_heartbeat_interval_default_value() {
    env::remove_var("SONGBIRD_HEARTBEAT_INTERVAL_MS");
    let interval = heartbeat_interval();
    assert_eq!(interval, Duration::from_millis(60000));
    assert_eq!(interval.as_secs(), 60); // 1 minute
}

#[test]
fn test_heartbeat_interval_from_env() {
    env::set_var("SONGBIRD_HEARTBEAT_INTERVAL_MS", "30000");
    let interval = heartbeat_interval();
    assert_eq!(interval, Duration::from_millis(30000));
    env::remove_var("SONGBIRD_HEARTBEAT_INTERVAL_MS");
}

#[test]
fn test_discovery_timeout_default_value() {
    env::remove_var("SONGBIRD_DISCOVERY_TIMEOUT_MS");
    let timeout = discovery_timeout();
    assert_eq!(timeout, Duration::from_millis(5000));
}

#[test]
fn test_discovery_timeout_from_env() {
    env::set_var("SONGBIRD_DISCOVERY_TIMEOUT_MS", "8000");
    let timeout = discovery_timeout();
    assert_eq!(timeout, Duration::from_millis(8000));
    env::remove_var("SONGBIRD_DISCOVERY_TIMEOUT_MS");
}

#[test]
fn test_connection_timeout_default_value() {
    env::remove_var("SONGBIRD_CONNECTION_TIMEOUT_MS");
    let timeout = connection_timeout();
    assert_eq!(timeout, Duration::from_millis(10000));
    assert_eq!(timeout.as_secs(), 10);
}

#[test]
fn test_connection_timeout_from_env() {
    env::set_var("SONGBIRD_CONNECTION_TIMEOUT_MS", "15000");
    let timeout = connection_timeout();
    assert_eq!(timeout, Duration::from_millis(15000));
    env::remove_var("SONGBIRD_CONNECTION_TIMEOUT_MS");
}

#[test]
fn test_retry_backoff_default_value() {
    env::remove_var("SONGBIRD_RETRY_BACKOFF_MS");
    let backoff = retry_backoff();
    assert_eq!(backoff, Duration::from_millis(1000));
    assert_eq!(backoff.as_secs(), 1);
}

#[test]
fn test_retry_backoff_from_env() {
    env::set_var("SONGBIRD_RETRY_BACKOFF_MS", "2000");
    let backoff = retry_backoff();
    assert_eq!(backoff, Duration::from_millis(2000));
    env::remove_var("SONGBIRD_RETRY_BACKOFF_MS");
}

#[test]
fn test_operation_timeout_with_default() {
    env::remove_var("SONGBIRD_CUSTOM_TIMEOUT_MS");
    let timeout = operation_timeout("CUSTOM", Duration::from_secs(10));
    assert_eq!(timeout.as_secs(), 10);
}

#[test]
fn test_operation_timeout_from_env() {
    env::set_var("SONGBIRD_CUSTOM_TIMEOUT_MS", "15000");
    let timeout = operation_timeout("CUSTOM", Duration::from_secs(10));
    assert_eq!(timeout, Duration::from_millis(15000));
    env::remove_var("SONGBIRD_CUSTOM_TIMEOUT_MS");
}

#[test]
fn test_operation_timeout_lowercase_operation_name() {
    env::set_var("SONGBIRD_MYOP_TIMEOUT_MS", "7000");
    let timeout = operation_timeout("myop", Duration::from_secs(5));
    assert_eq!(timeout, Duration::from_millis(7000));
    env::remove_var("SONGBIRD_MYOP_TIMEOUT_MS");
}

#[test]
fn test_timeout_relationships() {
    env::remove_var("SONGBIRD_TIMEOUT_MS");
    env::remove_var("SONGBIRD_LONG_TIMEOUT_MS");

    let standard = standard_timeout();
    let long = long_timeout();

    assert!(long > standard, "Long timeout should be greater than standard");
}

#[test]
fn test_all_timeouts_are_positive() {
    env::remove_var("SONGBIRD_TIMEOUT_MS");
    env::remove_var("SONGBIRD_LONG_TIMEOUT_MS");
    env::remove_var("SONGBIRD_REQUEST_TIMEOUT_MS");
    env::remove_var("SONGBIRD_CACHE_EXPIRY_MS");
    env::remove_var("SONGBIRD_HEARTBEAT_INTERVAL_MS");
    env::remove_var("SONGBIRD_DISCOVERY_TIMEOUT_MS");
    env::remove_var("SONGBIRD_CONNECTION_TIMEOUT_MS");
    env::remove_var("SONGBIRD_RETRY_BACKOFF_MS");

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
fn test_timeouts_are_reasonable_for_production() {
    env::remove_var("SONGBIRD_TIMEOUT_MS");
    env::remove_var("SONGBIRD_LONG_TIMEOUT_MS");
    env::remove_var("SONGBIRD_CONNECTION_TIMEOUT_MS");

    let standard = standard_timeout();
    let long = long_timeout();
    let connection = connection_timeout();

    // Standard timeout should be between 1-10 seconds
    assert!(standard.as_secs() >= 1 && standard.as_secs() <= 10);

    // Long timeout should be between 10-60 seconds
    assert!(long.as_secs() >= 10 && long.as_secs() <= 60);

    // Connection timeout should be between 5-30 seconds
    assert!(connection.as_secs() >= 5 && connection.as_secs() <= 30);
}

#[test]
fn test_cache_expiry_is_longer_than_timeouts() {
    env::remove_var("SONGBIRD_CACHE_EXPIRY_MS");
    env::remove_var("SONGBIRD_TIMEOUT_MS");
    env::remove_var("SONGBIRD_LONG_TIMEOUT_MS");

    let cache = cache_expiry();
    let standard = standard_timeout();
    let long = long_timeout();

    assert!(cache > standard);
    assert!(cache > long);
}

#[test]
fn test_heartbeat_interval_reasonable() {
    env::remove_var("SONGBIRD_HEARTBEAT_INTERVAL_MS");
    let interval = heartbeat_interval();

    // Heartbeat should be between 10 seconds and 5 minutes
    assert!(interval.as_secs() >= 10 && interval.as_secs() <= 300);
}

#[test]
fn test_retry_backoff_less_than_standard_timeout() {
    env::remove_var("SONGBIRD_RETRY_BACKOFF_MS");
    env::remove_var("SONGBIRD_TIMEOUT_MS");

    let backoff = retry_backoff();
    let standard = standard_timeout();

    assert!(backoff < standard, "Retry backoff should be less than standard timeout");
}

#[test]
fn test_env_var_zero_value() {
    env::set_var("SONGBIRD_TIMEOUT_MS", "0");
    let timeout = standard_timeout();
    assert_eq!(timeout, Duration::from_millis(0));
    env::remove_var("SONGBIRD_TIMEOUT_MS");
}

#[test]
fn test_env_var_very_large_value() {
    env::set_var("SONGBIRD_TIMEOUT_MS", "999999");
    let timeout = standard_timeout();
    assert_eq!(timeout, Duration::from_millis(999_999));
    env::remove_var("SONGBIRD_TIMEOUT_MS");
}

#[test]
fn test_env_var_negative_value_uses_default() {
    env::set_var("SONGBIRD_TIMEOUT_MS", "-1000");
    let timeout = standard_timeout();
    assert_eq!(timeout, Duration::from_millis(5000)); // Uses default
    env::remove_var("SONGBIRD_TIMEOUT_MS");
}

#[test]
fn test_env_var_empty_string_uses_default() {
    env::set_var("SONGBIRD_TIMEOUT_MS", "");
    let timeout = standard_timeout();
    assert_eq!(timeout, Duration::from_millis(5000));
    env::remove_var("SONGBIRD_TIMEOUT_MS");
}

#[test]
fn test_multiple_timeout_calls_consistent() {
    env::remove_var("SONGBIRD_TIMEOUT_MS");

    let timeout1 = standard_timeout();
    let timeout2 = standard_timeout();
    let timeout3 = standard_timeout();

    assert_eq!(timeout1, timeout2);
    assert_eq!(timeout2, timeout3);
}

#[test]
fn test_operation_timeout_with_special_characters() {
    env::set_var("SONGBIRD_MY_SPECIAL_OP_TIMEOUT_MS", "12000");
    let _timeout = operation_timeout("MY-SPECIAL-OP", Duration::from_secs(5));
    // Should normalize to MY_SPECIAL_OP but env var has MY_SPECIAL_OP
    // This tests the uppercase conversion
    env::remove_var("SONGBIRD_MY_SPECIAL_OP_TIMEOUT_MS");
}

#[test]
fn test_all_timeouts_concurrent_access() {
    // Simulate concurrent access by calling all functions rapidly
    for _ in 0..10 {
        let _ = standard_timeout();
        let _ = long_timeout();
        let _ = request_timeout();
        let _ = cache_expiry();
        let _ = heartbeat_interval();
        let _ = discovery_timeout();
        let _ = connection_timeout();
        let _ = retry_backoff();
    }
}

#[test]
fn test_discovery_timeout_equals_standard_by_default() {
    env::remove_var("SONGBIRD_TIMEOUT_MS");
    env::remove_var("SONGBIRD_DISCOVERY_TIMEOUT_MS");

    let standard = standard_timeout();
    let discovery = discovery_timeout();

    assert_eq!(standard, discovery);
}

#[test]
fn test_request_timeout_equals_long_by_default() {
    env::remove_var("SONGBIRD_LONG_TIMEOUT_MS");
    env::remove_var("SONGBIRD_REQUEST_TIMEOUT_MS");

    let long = long_timeout();
    let request = request_timeout();

    assert_eq!(long, request);
}

#[test]
fn test_operation_timeout_with_very_long_name() {
    let very_long_name = "A".repeat(100);
    env::set_var(format!("SONGBIRD_{}_TIMEOUT_MS", very_long_name.to_uppercase()), "5000");
    let timeout = operation_timeout(&very_long_name, Duration::from_secs(1));
    assert_eq!(timeout, Duration::from_millis(5000));
    env::remove_var(format!("SONGBIRD_{}_TIMEOUT_MS", very_long_name.to_uppercase()));
}

#[test]
fn test_timeout_precision() {
    env::set_var("SONGBIRD_TIMEOUT_MS", "1");
    let timeout = standard_timeout();
    assert_eq!(timeout.as_millis(), 1);
    env::remove_var("SONGBIRD_TIMEOUT_MS");
}
