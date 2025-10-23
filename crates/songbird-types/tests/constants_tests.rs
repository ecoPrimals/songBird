//! Tests for constants module

use songbird_types::constants::*;

#[test]
fn test_default_service_timeout_exists() {
    // Test that DEFAULT_SERVICE_TIMEOUT_SECONDS is defined
    let timeout = CanonicalDiscoveryDefaults::DEFAULT_SERVICE_TIMEOUT_SECONDS;
    assert!(timeout > 0);
}

#[test]
fn test_default_retry_attempts_exists() {
    // Test that DEFAULT_RETRY_ATTEMPTS is defined
    let retries = CanonicalPerformanceDefaults::DEFAULT_RETRY_ATTEMPTS;
    assert!(retries > 0);
}

#[test]
fn test_default_batch_size_exists() {
    // Test that DEFAULT_BATCH_SIZE is defined
    let buffer_size = CanonicalPerformanceDefaults::DEFAULT_BATCH_SIZE;
    assert!(buffer_size > 0);
}

// Note: Compile-time constant reasonableness checks are documented in the constants module.
// Runtime assertions on constants would be optimized out by the compiler.
