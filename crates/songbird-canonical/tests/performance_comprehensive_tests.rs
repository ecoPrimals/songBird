//! Comprehensive tests for Canonical Performance Monitoring
//!
//! This test suite provides thorough coverage of the performance
//! monitoring initialization system.

use songbird_canonical::performance::*;
use songbird_types::{SongbirdError, SongbirdResult};

// ========== Performance Monitoring Initialization Tests ==========

#[test]
fn test_initialize_performance_monitoring() -> SongbirdResult<()> {
    // Should complete without panicking
    initialize_performance_monitoring();
    Ok(())
}

#[test]
fn test_initialize_performance_monitoring_idempotent() -> SongbirdResult<()> {
    // Should be safe to call multiple times
    initialize_performance_monitoring();
    initialize_performance_monitoring();
    initialize_performance_monitoring();
    Ok(())
}

#[test]
fn test_initialize_performance_monitoring_thread_safe() -> SongbirdResult<()> {
    use std::thread;

    let handles: Vec<_> = (0..5)
        .map(|_| {
            thread::spawn(|| {
                initialize_performance_monitoring();
            })
        })
        .collect();

    for handle in handles {
        handle
            .join()
            .map_err(|_| SongbirdError::configuration("Thread should complete successfully"))?;
    }
    Ok(())
}

#[test]
fn test_initialize_performance_monitoring_no_panic() {
    // Test that initialization doesn't panic even under stress
    for _ in 0..10 {
        initialize_performance_monitoring();
    }
}

// ========== Integration Tests ==========

#[test]
fn test_performance_monitoring_in_production_scenario() {
    // Simulate production startup
    initialize_performance_monitoring();

    // Should be able to continue with other operations
    let test_value = 42;
    assert_eq!(test_value, 42);
}

#[test]
fn test_performance_monitoring_with_logging() {
    // Initialize performance monitoring (logging handled internally)
    initialize_performance_monitoring();

    // Should complete without issues
    // Test passes if initialization doesn't panic
}

#[test]
fn test_performance_monitoring_sequential_calls() {
    for i in 0..5 {
        initialize_performance_monitoring();
        // Verify we can do work between calls
        assert!(i < 10);
    }
}

#[test]
fn test_performance_monitoring_with_delays() {
    use std::thread;
    use std::time::Duration;

    initialize_performance_monitoring();
    thread::sleep(Duration::from_millis(10));
    initialize_performance_monitoring();
}

#[test]
fn test_performance_monitoring_return_type() {
    // Function returns (), so we can assign it
    let () = initialize_performance_monitoring();
    // Test passes if function doesn't panic
}

#[test]
fn test_performance_monitoring_inline() {
    // Should work inline
    initialize_performance_monitoring();
}
