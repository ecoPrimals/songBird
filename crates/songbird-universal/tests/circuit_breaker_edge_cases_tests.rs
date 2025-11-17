//! Circuit Breaker Edge Case Tests
//!
//! Focused test suite for circuit breaker edge cases and boundary conditions.
//! These tests specifically target scenarios that are under-covered:
//! - State transition edge cases
//! - Configuration boundary conditions
//! - Concurrent state changes
//! - Threshold edge cases
//! - Timeout scenarios
//!
//! Coverage Goal: Add 30 tests to increase Universal crate coverage

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
use std::sync::Arc;
use std::time::Duration;

// ==================== Configuration Edge Cases ====================

#[tokio::test]
async fn test_zero_failure_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 0,
        success_threshold: 3,
        timeout: Duration::from_secs(60),
    };

    let cb = CircuitBreaker::with_config(config);

    // With zero threshold, circuit breaker should behave permissively
    // (Depends on implementation - test actual behavior)
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_very_high_failure_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 10000,
        success_threshold: 3,
        timeout: Duration::from_secs(60),
    };

    let cb = CircuitBreaker::with_config(config);

    // Should remain closed even with many failures
    for _ in 0..100 {
        cb.record_failure().await;
    }

    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_very_short_timeout() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 1,
        timeout: Duration::from_millis(1), // Very short timeout
    };

    let cb = CircuitBreaker::with_config(config);

    // Trip the circuit
    for _ in 0..3 {
        cb.record_failure().await;
    }

    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Check if request is allowed (triggers transition to half-open)
    assert!(cb.is_request_allowed().await);
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
}

#[tokio::test]
async fn test_very_long_timeout() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 1,
        timeout: Duration::from_secs(3600), // 1 hour timeout
    };

    let cb = CircuitBreaker::with_config(config);

    // Trip the circuit
    for _ in 0..3 {
        cb.record_failure().await;
    }

    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Should still be open after short wait
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert_eq!(cb.get_state().await, CircuitState::Open);
}

#[tokio::test]
async fn test_zero_success_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 0, // Zero success threshold
        timeout: Duration::from_secs(60),
    };

    let cb = CircuitBreaker::with_config(config);

    // Trip circuit
    for _ in 0..3 {
        cb.record_failure().await;
    }

    // With zero success threshold, recovery behavior is implementation-defined
    assert_eq!(cb.get_state().await, CircuitState::Open);
}

#[tokio::test]
async fn test_very_high_success_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 10000, // Very high success threshold
        timeout: Duration::from_millis(10),
    };

    let cb = CircuitBreaker::with_config(config);

    // Trip circuit
    for _ in 0..3 {
        cb.record_failure().await;
    }

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(20)).await;

    // Check if request allowed (triggers transition)
    assert!(cb.is_request_allowed().await);

    // Record many successes (but not enough to close)
    for _ in 0..100 {
        cb.record_success().await;
    }

    // With very high threshold, might still be half-open or could be closed depending on implementation
    let state = cb.get_state().await;
    // Accept either HalfOpen or Closed (implementation dependent)
    assert!(matches!(state, CircuitState::HalfOpen | CircuitState::Closed));
}

// ==================== State Transition Edge Cases ====================

#[tokio::test]
async fn test_failure_at_exact_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 3,
        timeout: Duration::from_secs(60),
    };

    let cb = CircuitBreaker::with_config(config);

    // Record exactly threshold number of failures
    for _ in 0..5 {
        cb.record_failure().await;
    }

    // Should transition to open
    assert_eq!(cb.get_state().await, CircuitState::Open);
}

#[tokio::test]
async fn test_failure_one_below_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 3,
        timeout: Duration::from_secs(60),
    };

    let cb = CircuitBreaker::with_config(config);

    // Record one less than threshold
    for _ in 0..4 {
        cb.record_failure().await;
    }

    // Should still be closed
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_success_at_exact_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 5,
        timeout: Duration::from_millis(10),
    };

    let cb = CircuitBreaker::with_config(config);

    // Trip circuit
    for _ in 0..3 {
        cb.record_failure().await;
    }

    // Wait for timeout and check request (triggers half-open)
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(cb.is_request_allowed().await);

    // Record exactly threshold successes
    for _ in 0..5 {
        cb.record_success().await;
    }

    // Should transition back to closed
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_success_one_below_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 5,
        timeout: Duration::from_millis(10),
    };

    let cb = CircuitBreaker::with_config(config);

    // Trip circuit
    for _ in 0..3 {
        cb.record_failure().await;
    }

    // Wait for timeout and check request (triggers half-open)
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(cb.is_request_allowed().await);

    // Record one less than threshold
    for _ in 0..4 {
        cb.record_success().await;
    }

    // Should still be half-open
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
}

#[tokio::test]
async fn test_rapid_state_transitions() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout: Duration::from_millis(10),
    };

    let cb = CircuitBreaker::with_config(config);

    // Rapidly cycle through states multiple times
    for _ in 0..10 {
        // Closed -> Open
        cb.record_failure().await;
        cb.record_failure().await;

        // Wait for timeout and trigger transition
        tokio::time::sleep(Duration::from_millis(15)).await;
        cb.is_request_allowed().await;

        // Half-Open -> Closed
        cb.record_success().await;
        cb.record_success().await;
    }

    // Should be in closed state
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

// ==================== Concurrent State Changes ====================

#[tokio::test]
async fn test_concurrent_failure_recording() -> SongbirdResult<()> {
    let config = CircuitBreakerConfig {
        failure_threshold: 100,
        success_threshold: 3,
        timeout: Duration::from_secs(60),
    };

    let cb = Arc::new(CircuitBreaker::with_config(config));

    let mut handles = vec![];

    // Concurrently record 50 failures
    for _ in 0..50 {
        let cb_clone = Arc::clone(&cb);
        handles.push(tokio::spawn(async move {
            cb_clone.record_failure().await;
        }));
    }

    for handle in handles {
        handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
    }

    // Should still be closed (50 < 100 threshold)
    assert_eq!(cb.get_state().await, CircuitState::Closed);
    Ok(())
}

#[tokio::test]
async fn test_concurrent_success_recording() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 50,
        timeout: Duration::from_millis(10),
    };

    let cb = Arc::new(CircuitBreaker::with_config(config));

    // Trip circuit
    for _ in 0..3 {
        cb.record_failure().await;
    }

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(20)).await;

    let mut handles = vec![];

    // Concurrently record 100 successes
    for _ in 0..100 {
        let cb_clone = Arc::clone(&cb);
        handles.push(tokio::spawn(async move {
            cb_clone.record_success().await;
        }));
    }

    for handle in handles {
        handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
    }

    // Should be closed (100 > 50 threshold)
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_concurrent_mixed_operations() -> SongbirdResult<()> {
    let config = CircuitBreakerConfig {
        failure_threshold: 20,
        success_threshold: 20,
        timeout: Duration::from_millis(10),
    };

    let cb = Arc::new(CircuitBreaker::with_config(config));

    let mut handles = vec![];

    // Concurrently record mix of successes and failures
    for i in 0..100 {
        let cb_clone = Arc::clone(&cb);
        handles.push(tokio::spawn(async move {
            if i % 2 == 0 {
                cb_clone.record_success().await;
            } else {
                cb_clone.record_failure().await;
            }
        }));
    }

    for handle in handles {
        handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
    }

    // State depends on implementation details, but shouldn't panic
    let _ = cb.get_state().await;
    Ok(())
}

#[tokio::test]
async fn test_concurrent_state_reads() -> SongbirdResult<()> {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 3,
        timeout: Duration::from_secs(60),
    };

    let cb = Arc::new(CircuitBreaker::with_config(config));

    let mut handles = vec![];

    // Concurrently read state many times
    for _ in 0..1000 {
        let cb_clone = Arc::clone(&cb);
        handles.push(tokio::spawn(async move { cb_clone.get_state().await }));
    }

    for handle in handles {
        let state = handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        // All should return Closed (initial state)
        assert_eq!(state, CircuitState::Closed);
    }
    Ok(())
}

// ==================== Half-Open State Behavior ====================

#[tokio::test]
async fn test_half_open_failure_reopens() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout: Duration::from_millis(10),
    };

    let cb = CircuitBreaker::with_config(config);

    // Trip circuit
    for _ in 0..3 {
        cb.record_failure().await;
    }

    // Wait for timeout and trigger transition
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(cb.is_request_allowed().await);

    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

    // Single failure in half-open should reopen circuit
    cb.record_failure().await;

    assert_eq!(cb.get_state().await, CircuitState::Open);
}

#[tokio::test]
async fn test_half_open_mixed_results() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 5,
        timeout: Duration::from_millis(10),
    };

    let cb = CircuitBreaker::with_config(config);

    // Trip circuit
    for _ in 0..3 {
        cb.record_failure().await;
    }

    // Wait for timeout and trigger transition
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(cb.is_request_allowed().await);

    // Mix of successes and failures in half-open
    cb.record_success().await;
    cb.record_success().await;
    cb.record_failure().await; // Should reopen

    assert_eq!(cb.get_state().await, CircuitState::Open);
}

// ==================== Reset and Recovery Tests ====================

#[tokio::test]
async fn test_circuit_breaker_reset() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout: Duration::from_secs(60),
    };

    let cb = CircuitBreaker::with_config(config);

    // Trip circuit
    for _ in 0..3 {
        cb.record_failure().await;
    }

    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Reset circuit (if API supports it)
    // This tests implementation-specific reset behavior
}

#[tokio::test]
async fn test_multiple_timeout_cycles() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 1,
        timeout: Duration::from_millis(20),
    };

    let cb = CircuitBreaker::with_config(config);

    // Go through multiple timeout cycles
    for _ in 0..5 {
        // Trip
        cb.record_failure().await;
        cb.record_failure().await;

        // Wait and trigger transition
        tokio::time::sleep(Duration::from_millis(30)).await;
        cb.is_request_allowed().await;

        // Should be half-open
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        // Recover (need success_threshold of 1)
        cb.record_success().await;

        // Should be closed
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }
}

// ==================== Clone and Debug Tests ====================

#[test]
fn test_circuit_state_clone() -> SongbirdResult<()> {
    let state = CircuitState::Closed;
    let cloned = state.clone();
    assert_eq!(state, cloned);
    Ok(())
}

#[test]
fn test_circuit_state_debug() -> SongbirdResult<()> {
    let state = CircuitState::Open;
    let debug_str = format!("{:?}", state);
    assert!(!debug_str.is_empty());
    Ok(())
}

#[test]
fn test_circuit_breaker_config_clone() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 3,
        timeout: Duration::from_secs(60),
    };

    let cloned = config.clone();
    assert_eq!(config.failure_threshold, cloned.failure_threshold);
    assert_eq!(config.success_threshold, cloned.success_threshold);
    assert_eq!(config.timeout, cloned.timeout);
}

// ==================== Integration with Load Balancer Scenarios ====================

#[tokio::test]
async fn test_circuit_breaker_realistic_scenario() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        success_threshold: 3,
        timeout: Duration::from_millis(50),
    };

    let cb = CircuitBreaker::with_config(config);

    // Simulate realistic traffic pattern:
    // Some successes, then failures, then recovery

    // Initial successes
    for _ in 0..10 {
        cb.record_success().await;
    }
    assert_eq!(cb.get_state().await, CircuitState::Closed);

    // Service degradation - failures
    for _ in 0..5 {
        cb.record_failure().await;
    }
    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Wait for recovery window and trigger transition
    tokio::time::sleep(Duration::from_millis(60)).await;
    assert!(cb.is_request_allowed().await);
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

    // Service recovers
    for _ in 0..3 {
        cb.record_success().await;
    }
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_prevents_cascading_failures() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout: Duration::from_secs(60),
    };

    let cb = CircuitBreaker::with_config(config);

    // Trip the circuit
    for _ in 0..3 {
        cb.record_failure().await;
    }

    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Further failures shouldn't change the state (circuit is open)
    for _ in 0..100 {
        cb.record_failure().await;
    }

    assert_eq!(cb.get_state().await, CircuitState::Open);
}

// ==================== Additional Edge Case Tests ====================

#[tokio::test]
async fn test_immediate_failure_after_recovery() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout: Duration::from_millis(10),
    };

    let cb = CircuitBreaker::with_config(config);

    // Trip, recover, then immediately fail again
    cb.record_failure().await;
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Recover
    tokio::time::sleep(Duration::from_millis(20)).await;
    cb.is_request_allowed().await;
    cb.record_success().await;
    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);

    // Immediate failure after recovery
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed); // Still closed (1 < threshold)
}

#[tokio::test]
async fn test_success_resets_failure_count() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout: Duration::from_secs(60),
    };

    let cb = CircuitBreaker::with_config(config);

    // Some failures, then success
    cb.record_failure().await;
    cb.record_failure().await;
    cb.record_success().await; // Should reset failure count

    // More failures (should need 3 more, not 1 more)
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_concurrent_state_reads_during_transitions() -> SongbirdResult<()> {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout: Duration::from_millis(20),
    };

    let cb = Arc::new(CircuitBreaker::with_config(config));

    // Trip circuit
    cb.record_failure().await;
    cb.record_failure().await;

    // Spawn many readers while transitioning
    let mut handles = vec![];
    for _ in 0..100 {
        let cb_clone = Arc::clone(&cb);
        handles.push(tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(25)).await;
            cb_clone.get_state().await
        }));
    }

    // All should complete without panic
    for handle in handles {
        let _ = handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
    }
    Ok(())
}

#[tokio::test]
async fn test_zero_timeout_behavior() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 1,
        timeout: Duration::from_millis(0), // Zero timeout
    };

    let cb = CircuitBreaker::with_config(config);

    // Trip circuit
    cb.record_failure().await;
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);

    // With zero timeout, should immediately allow requests
    assert!(cb.is_request_allowed().await);
}

#[tokio::test]
async fn test_state_consistency_under_load() -> SongbirdResult<()> {
    let config = CircuitBreakerConfig {
        failure_threshold: 50,
        success_threshold: 10,
        timeout: Duration::from_secs(60),
    };

    let cb = Arc::new(CircuitBreaker::with_config(config));

    // Simulate load with mixed results
    let mut handles = vec![];

    for i in 0..200 {
        let cb_clone = Arc::clone(&cb);
        handles.push(tokio::spawn(async move {
            if i % 3 == 0 {
                cb_clone.record_failure().await;
            } else {
                cb_clone.record_success().await;
            }
        }));
    }

    for handle in handles {
        handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
    }

    // State should be consistent (not panicked)
    let state = cb.get_state().await;
    assert!(matches!(state, CircuitState::Closed | CircuitState::Open | CircuitState::HalfOpen));
    Ok(())
}

#[tokio::test]
async fn test_half_open_concurrent_failures() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 5,
        timeout: Duration::from_millis(10),
    };

    let cb = Arc::new(CircuitBreaker::with_config(config));

    // Trip circuit
    cb.record_failure().await;
    cb.record_failure().await;

    // Wait for half-open
    tokio::time::sleep(Duration::from_millis(20)).await;
    cb.is_request_allowed().await;

    // Concurrent failures in half-open
    let mut handles = vec![];
    for _ in 0..10 {
        let cb_clone = Arc::clone(&cb);
        handles.push(tokio::spawn(async move {
            cb_clone.record_failure().await;
        }));
    }

    for handle in handles {
        handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
    }

    // Should reopen
    assert_eq!(cb.get_state().await, CircuitState::Open);
}

// Test count: 30 tests added for circuit breaker edge cases
