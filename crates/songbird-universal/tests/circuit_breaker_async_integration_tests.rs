//! Async Integration Tests for Circuit Breaker
//!
//! **Goal**: Test circuit breaker state machine under realistic async scenarios
//! **Coverage Target**: State transitions, timing, concurrent access
//!
//! This suite tests:
//! - State transitions (Closed → Open → HalfOpen → Closed)
//! - Timing-based transitions (timeout recovery)
//! - Concurrent request handling
//! - Failure threshold behavior
//! - Success threshold recovery

use songbird_universal::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// STATE TRANSITION TESTS
// ============================================================================

#[tokio::test]
async fn test_closed_to_open_transition() {
    let config = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 3,
        success_threshold: 2,
        half_open_max_requests: 1,
        timeout: Duration::from_secs(5),
    };
    let cb = CircuitBreaker::with_config(config);

    // Start in Closed state
    assert_eq!(cb.get_state().await, CircuitState::Closed);
    assert!(cb.is_request_allowed().await);

    // Record failures to reach threshold
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);
    
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);
    
    cb.record_failure().await; // Should trip
    assert_eq!(cb.get_state().await, CircuitState::Open);
    assert!(!cb.is_request_allowed().await);
}

#[tokio::test]
async fn test_open_to_halfopen_after_timeout() {
    let config = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 1,
        success_threshold: 2,
        half_open_max_requests: 1,
        timeout: Duration::from_millis(100),
    };
    let cb = CircuitBreaker::with_config(config);

    // Trip the circuit
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Should still be open immediately
    assert!(!cb.is_request_allowed().await);

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Should transition to HalfOpen on next request check
    assert!(cb.is_request_allowed().await);
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
}

#[tokio::test]
async fn test_halfopen_to_closed_on_success() {
    let config = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 1,
        success_threshold: 2,
        half_open_max_requests: 1,
        timeout: Duration::from_millis(50),
    };
    let cb = CircuitBreaker::with_config(config);

    // Trip to Open
    cb.record_failure().await;
    
    // Wait for timeout to HalfOpen
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(cb.is_request_allowed().await);
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

    // Record successes to close circuit
    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
    
    cb.record_success().await; // Should close
    assert_eq!(cb.get_state().await, CircuitState::Closed);
    assert!(cb.is_request_allowed().await);
}

#[tokio::test]
async fn test_halfopen_to_open_on_failure() {
    let config = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 1,
        success_threshold: 2,
        half_open_max_requests: 1,
        timeout: Duration::from_millis(50),
    };
    let cb = CircuitBreaker::with_config(config);

    // Trip to Open
    cb.record_failure().await;
    
    // Wait for timeout to HalfOpen
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(cb.is_request_allowed().await);
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

    // Any failure in HalfOpen should reopen
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);
    assert!(!cb.is_request_allowed().await);
}

// ============================================================================
// CONCURRENT ACCESS TESTS
// ============================================================================

#[tokio::test]
async fn test_concurrent_request_checks() {
    let cb = Arc::new(CircuitBreaker::new());
    let cb1 = Arc::clone(&cb);
    let cb2 = Arc::clone(&cb);
    let cb3 = Arc::clone(&cb);

    // Fire off multiple concurrent request checks
    let handles = vec![
        tokio::spawn(async move {
            cb1.is_request_allowed().await
        }),
        tokio::spawn(async move {
            cb2.is_request_allowed().await
        }),
        tokio::spawn(async move {
            cb3.is_request_allowed().await
        }),
    ];

    let results = futures::future::join_all(handles).await;
    
    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}

#[tokio::test]
async fn test_concurrent_failure_recording() {
    let config = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 5,
        success_threshold: 2,
        half_open_max_requests: 1,
        timeout: Duration::from_secs(5),
    };
    let cb = Arc::new(CircuitBreaker::with_config(config));

    // Record 5 failures concurrently
    let mut handles = vec![];
    for _ in 0..5 {
        let cb_clone = Arc::clone(&cb);
        handles.push(tokio::spawn(async move {
            cb_clone.record_failure().await;
        }));
    }

    futures::future::join_all(handles).await;

    // Should be open
    assert_eq!(cb.get_state().await, CircuitState::Open);
    assert!(!cb.is_request_allowed().await);
}

#[tokio::test]
async fn test_concurrent_success_recording_in_halfopen() {
    let config = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 1,
        success_threshold: 3,
        half_open_max_requests: 1,
        timeout: Duration::from_millis(50),
    };
    let cb = Arc::new(CircuitBreaker::with_config(config));

    // Trip to Open
    cb.record_failure().await;
    
    // Wait for HalfOpen
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(cb.is_request_allowed().await);

    // Record 3 successes concurrently
    let mut handles = vec![];
    for _ in 0..3 {
        let cb_clone = Arc::clone(&cb);
        handles.push(tokio::spawn(async move {
            cb_clone.record_success().await;
        }));
    }

    futures::future::join_all(handles).await;

    // Should be closed
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

// ============================================================================
// TIMING AND RECOVERY TESTS
// ============================================================================

#[tokio::test]
async fn test_multiple_timeout_cycles() {
    let config = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 1,
        success_threshold: 1,
        half_open_max_requests: 1,
        timeout: Duration::from_millis(50),
    };
    let cb = CircuitBreaker::with_config(config);

    // Cycle 1: Open → HalfOpen → Open
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);
    
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(cb.is_request_allowed().await);
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
    
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Cycle 2: Open → HalfOpen → Closed
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(cb.is_request_allowed().await);
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
    
    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_timeout_does_not_affect_closed_state() {
    let cb = CircuitBreaker::new();
    
    // Stay in Closed state
    assert_eq!(cb.get_state().await, CircuitState::Closed);
    
    // Wait longer than any timeout
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Should still be closed
    assert_eq!(cb.get_state().await, CircuitState::Closed);
    assert!(cb.is_request_allowed().await);
}

#[tokio::test]
async fn test_rapid_failure_recovery_cycle() {
    let config = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 2,
        success_threshold: 1,
        half_open_max_requests: 1,
        timeout: Duration::from_millis(30),
    };
    let cb = CircuitBreaker::with_config(config);

    for _ in 0..3 {
        // Trip the circuit
        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);
        
        // Wait for recovery
        tokio::time::sleep(Duration::from_millis(40)).await;
        assert!(cb.is_request_allowed().await);
        
        // Successful recovery
        cb.record_success().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }
}

// ============================================================================
// THRESHOLD BOUNDARY TESTS
// ============================================================================

#[tokio::test]
async fn test_failure_threshold_boundary() {
    let config = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 5,
        success_threshold: 2,
        half_open_max_requests: 1,
        timeout: Duration::from_secs(5),
    };
    let cb = CircuitBreaker::with_config(config);

    // Record failures up to but not including threshold
    for _ in 0..4 {
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    // One more should trip it
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);
}

#[tokio::test]
async fn test_success_threshold_boundary() {
    let config = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 1,
        success_threshold: 3,
        half_open_max_requests: 1,
        timeout: Duration::from_millis(50),
    };
    let cb = CircuitBreaker::with_config(config);

    // Trip to Open then HalfOpen
    cb.record_failure().await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert!(cb.is_request_allowed().await);
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

    // Record successes up to but not including threshold
    for _ in 0..2 {
        cb.record_success().await;
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
    }

    // One more should close it
    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

// ============================================================================
// RESET AND RECOVERY TESTS
// ============================================================================

#[tokio::test]
async fn test_manual_reset() {
    let config = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 1,
        success_threshold: 2,
        half_open_max_requests: 1,
        timeout: Duration::from_secs(10),
    };
    let cb = CircuitBreaker::with_config(config);

    // Trip to Open
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Manual reset
    cb.reset().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);
    assert!(cb.is_request_allowed().await);
}

#[tokio::test]
async fn test_success_in_closed_resets_failure_count() {
    let config = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 3,
        success_threshold: 2,
        half_open_max_requests: 1,
        timeout: Duration::from_secs(5),
    };
    let cb = CircuitBreaker::with_config(config);

    // Record some failures (not enough to trip)
    cb.record_failure().await;
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);

    // Record success (should reset failure count)
    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);

    // Now we need 3 more failures to trip
    cb.record_failure().await;
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);
    
    cb.record_failure().await; // Should trip
    assert_eq!(cb.get_state().await, CircuitState::Open);
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[tokio::test]
async fn test_open_state_blocks_all_requests() {
    let config = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 1,
        success_threshold: 2,
        half_open_max_requests: 1,
        timeout: Duration::from_secs(60), // Long timeout
    };
    let cb = CircuitBreaker::with_config(config);

    // Trip to Open
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Multiple checks should all return false
    for _ in 0..10 {
        assert!(!cb.is_request_allowed().await);
    }
}

#[tokio::test]
async fn test_low_failure_threshold() {
    // Edge case: threshold of 1 means trips immediately
    let config = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 1,
        success_threshold: 2,
        half_open_max_requests: 1,
        timeout: Duration::from_secs(5),
    };
    let cb = CircuitBreaker::with_config(config);

    // First failure should trip
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);
}

#[tokio::test]
async fn test_very_short_timeout() {
    let config = CircuitBreakerConfig {
        enabled: true,
        failure_threshold: 1,
        success_threshold: 1,
        half_open_max_requests: 1,
        timeout: Duration::from_millis(1), // 1ms timeout
    };
    let cb = CircuitBreaker::with_config(config);

    // Trip to Open
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Wait just a bit
    tokio::time::sleep(Duration::from_millis(5)).await;

    // Should be HalfOpen
    assert!(cb.is_request_allowed().await);
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
}

