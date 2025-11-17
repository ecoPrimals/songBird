//! Enhanced Circuit Breaker Tests
//!
//! Comprehensive tests for circuit breaker state transitions, failure handling,
//! recovery scenarios, and concurrent access patterns.

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Barrier;

// ==================== State Transition Tests ====================

#[tokio::test]
async fn test_initial_state_is_closed() {
    let cb = CircuitBreaker::new();
    assert_eq!(cb.get_state().await, CircuitState::Closed);
    assert!(cb.is_request_allowed().await);
}

#[tokio::test]
async fn test_transition_closed_to_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        timeout: Duration::from_secs(60),
        success_threshold: 2,
    };
    let cb = CircuitBreaker::with_config(config);

    // Initially closed
    assert_eq!(cb.get_state().await, CircuitState::Closed);

    // Record failures up to threshold
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);

    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);

    // Third failure should open the circuit
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);
    assert!(!cb.is_request_allowed().await);
}

#[tokio::test]
async fn test_transition_open_to_half_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        timeout: Duration::from_millis(100),
        success_threshold: 2,
    };
    let cb = CircuitBreaker::with_config(config);

    // Open the circuit
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Next request check should transition to half-open
    assert!(cb.is_request_allowed().await);
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
}

#[tokio::test]
async fn test_transition_half_open_to_closed() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        timeout: Duration::from_millis(50),
        success_threshold: 2,
    };
    let cb = CircuitBreaker::with_config(config);

    // Open the circuit
    cb.record_failure().await;

    // Wait and transition to half-open
    tokio::time::sleep(Duration::from_millis(100)).await;
    cb.is_request_allowed().await;
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

    // Record successful requests
    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_transition_half_open_back_to_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        timeout: Duration::from_millis(50),
        success_threshold: 2,
    };
    let cb = CircuitBreaker::with_config(config);

    // Open the circuit
    cb.record_failure().await;

    // Wait and transition to half-open
    tokio::time::sleep(Duration::from_millis(100)).await;
    cb.is_request_allowed().await;
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

    // Failure in half-open should reopen
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);
}

// ==================== Failure Threshold Tests ====================

#[tokio::test]
async fn test_failure_threshold_respected() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        ..Default::default()
    };
    let cb = CircuitBreaker::with_config(config);

    // Record failures below threshold
    for _ in 0..4 {
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    // Fifth failure should open
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);
}

#[tokio::test]
async fn test_success_resets_failure_count() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        ..Default::default()
    };
    let cb = CircuitBreaker::with_config(config);

    // Record some failures
    cb.record_failure().await;
    cb.record_failure().await;

    // Success should reset counter
    cb.record_success().await;

    // Should need 3 more failures to open
    cb.record_failure().await;
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);

    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);
}

#[tokio::test]
async fn test_single_failure_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        ..Default::default()
    };
    let cb = CircuitBreaker::with_config(config);

    // Single failure should open
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);
}

// ==================== Success Threshold Tests ====================

#[tokio::test]
async fn test_success_threshold_for_closing() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        timeout: Duration::from_millis(50),
        success_threshold: 3,
    };
    let cb = CircuitBreaker::with_config(config);

    // Open the circuit
    cb.record_failure().await;

    // Transition to half-open
    tokio::time::sleep(Duration::from_millis(100)).await;
    cb.is_request_allowed().await;

    // Record successes below threshold
    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

    // Third success should close
    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

// ==================== Timeout Tests ====================

#[tokio::test]
async fn test_timeout_duration_respected() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        timeout: Duration::from_millis(200),
        success_threshold: 1,
    };
    let cb = CircuitBreaker::with_config(config);

    // Open the circuit
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Check immediately - should still be open
    assert!(!cb.is_request_allowed().await);
    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(250)).await;

    // Now should transition to half-open
    assert!(cb.is_request_allowed().await);
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
}

#[tokio::test]
async fn test_very_short_timeout() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        timeout: Duration::from_millis(1),
        success_threshold: 1,
    };
    let cb = CircuitBreaker::with_config(config);

    // Open the circuit
    cb.record_failure().await;

    // Even very short timeout should work
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert!(cb.is_request_allowed().await);
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
}

// ==================== Request Blocking Tests ====================

#[tokio::test]
async fn test_requests_blocked_when_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        timeout: Duration::from_secs(10),
        ..Default::default()
    };
    let cb = CircuitBreaker::with_config(config);

    // Open the circuit
    cb.record_failure().await;

    // Multiple requests should all be blocked
    for _ in 0..10 {
        assert!(!cb.is_request_allowed().await);
    }
}

#[tokio::test]
async fn test_requests_allowed_when_closed() {
    let cb = CircuitBreaker::new();

    // All requests should be allowed
    for _ in 0..100 {
        assert!(cb.is_request_allowed().await);
    }
}

#[tokio::test]
async fn test_requests_allowed_in_half_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        timeout: Duration::from_millis(50),
        success_threshold: 2,
    };
    let cb = CircuitBreaker::with_config(config);

    // Open then transition to half-open
    cb.record_failure().await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    cb.is_request_allowed().await;

    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

    // Requests should be allowed in half-open
    assert!(cb.is_request_allowed().await);
}

// ==================== Reset Tests ====================

#[tokio::test]
async fn test_reset_from_open_state() {
    let cb = CircuitBreaker::new();

    // Open the circuit
    for _ in 0..5 {
        cb.record_failure().await;
    }
    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Reset
    cb.reset().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);
    assert!(cb.is_request_allowed().await);
}

#[tokio::test]
async fn test_reset_from_half_open_state() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        timeout: Duration::from_millis(50),
        ..Default::default()
    };
    let cb = CircuitBreaker::with_config(config);

    // Get to half-open
    cb.record_failure().await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    cb.is_request_allowed().await;
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

    // Reset
    cb.reset().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

// ==================== Concurrent Access Tests ====================

#[tokio::test]
async fn test_concurrent_failure_recording() -> SongbirdResult<()> {
    let config = CircuitBreakerConfig {
        failure_threshold: 10,
        ..Default::default()
    };
    let cb = Arc::new(CircuitBreaker::with_config(config));

    let mut handles = vec![];
    let barrier = Arc::new(Barrier::new(5));

    // Record failures concurrently
    for _ in 0..5 {
        let cb_clone = Arc::clone(&cb);
        let barrier_clone = Arc::clone(&barrier);

        handles.push(tokio::spawn(async move {
            barrier_clone.wait().await;
            cb_clone.record_failure().await;
        }));
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
    }

    // Circuit might or might not be open depending on race conditions
    // but should still be in a valid state
    let state = cb.get_state().await;
    assert!(matches!(state, CircuitState::Closed | CircuitState::Open));
    Ok(())
}

#[tokio::test]
async fn test_concurrent_success_recording() -> SongbirdResult<()> {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        timeout: Duration::from_millis(50),
        success_threshold: 10,
    };
    let cb = Arc::new(CircuitBreaker::with_config(config));

    // Open and transition to half-open
    cb.record_failure().await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    cb.is_request_allowed().await;

    let mut handles = vec![];

    // Record successes concurrently
    for _ in 0..5 {
        let cb_clone = Arc::clone(&cb);

        handles.push(tokio::spawn(async move {
            cb_clone.record_success().await;
        }));
    }

    // Wait for all
    for handle in handles {
        handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
    }

    // Should be in half-open or closed
    let state = cb.get_state().await;
    assert!(matches!(state, CircuitState::HalfOpen | CircuitState::Closed));
    Ok(())
}

#[tokio::test]
async fn test_concurrent_state_queries() -> SongbirdResult<()> {
    let cb = Arc::new(CircuitBreaker::new());
    let mut handles = vec![];

    // Query state concurrently
    for _ in 0..20 {
        let cb_clone = Arc::clone(&cb);

        handles.push(tokio::spawn(async move {
            cb_clone.get_state().await;
            cb_clone.is_request_allowed().await;
        }));
    }

    // All should complete without panic
    for handle in handles {
        handle.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
    }
    Ok(())
}

// ==================== Edge Cases ====================

#[tokio::test]
async fn test_default_config_values() {
    let config = CircuitBreakerConfig::default();

    assert_eq!(config.failure_threshold, 5);
    assert_eq!(config.timeout, Duration::from_secs(60));
    assert_eq!(config.success_threshold, 2);
}

#[tokio::test]
async fn test_default_circuit_breaker() {
    let cb = CircuitBreaker::default();
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_rapid_state_changes() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        timeout: Duration::from_millis(10),
        success_threshold: 1,
    };
    let cb = CircuitBreaker::with_config(config);

    // Rapid open/close cycle
    for _ in 0..5 {
        // Open
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Wait and transition to half-open
        tokio::time::sleep(Duration::from_millis(20)).await;
        cb.is_request_allowed().await;

        // Close
        cb.record_success().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }
}

// ==================== Integration Scenarios ====================

#[tokio::test]
async fn test_realistic_failure_recovery_scenario() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        timeout: Duration::from_millis(100),
        success_threshold: 2,
    };
    let cb = CircuitBreaker::with_config(config);

    // Normal operation
    cb.record_success().await;
    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);

    // Service degrades
    cb.record_failure().await;
    cb.record_failure().await;
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Open);

    // Wait for recovery period
    tokio::time::sleep(Duration::from_millis(150)).await;
    cb.is_request_allowed().await;
    assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

    // Service recovers
    cb.record_success().await;
    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);

    // Back to normal operation
    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}

#[tokio::test]
async fn test_intermittent_failure_pattern() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        ..Default::default()
    };
    let cb = CircuitBreaker::with_config(config);

    // Intermittent failures don't open circuit if successes reset counter
    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);

    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);

    cb.record_failure().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);

    cb.record_success().await;
    assert_eq!(cb.get_state().await, CircuitState::Closed);
}
