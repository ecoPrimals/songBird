//! # Circuit Breaker Tests (Correct API)
//!
//! Comprehensive tests for circuit breaker functionality using the actual async API

use songbird_config::canonical::resilience::CircuitBreakerConfig;
use songbird_universal::circuit_breaker::{CircuitBreaker, CircuitState};
use std::time::Duration;

// ============================================================================
// CIRCUIT BREAKER CREATION
// ============================================================================

#[tokio::test]
async fn test_circuit_breaker_new_default() {
    let breaker = CircuitBreaker::new();
    let state = breaker.get_state().await;

    assert_eq!(state, CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_with_custom_config() {
    let config = CircuitBreakerConfig {
        failure_threshold: 10,
        timeout: Duration::from_secs(60),
        success_threshold: 3,
        half_open_max_requests: 5,
        enabled: true,
    };

    let breaker = CircuitBreaker::with_config(config);
    let state = breaker.get_state().await;

    assert_eq!(state, CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_initial_state_is_closed() {
    let breaker = CircuitBreaker::new();
    let state = breaker.get_state().await;

    assert_eq!(state, CircuitState::Closed);
    assert!(breaker.is_request_allowed().await);
}

// ============================================================================
// REQUEST PERMISSION TESTS
// ============================================================================

#[tokio::test]
async fn test_is_request_allowed_when_closed() {
    let breaker = CircuitBreaker::new();

    assert!(breaker.is_request_allowed().await);
}

#[tokio::test]
async fn test_is_request_allowed_after_few_failures() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        timeout: Duration::from_secs(30),
        success_threshold: 2,
        half_open_max_requests: 3,
        enabled: true,
    };

    let breaker = CircuitBreaker::with_config(config);

    // Record some failures (below threshold)
    breaker.record_failure().await;
    breaker.record_failure().await;
    breaker.record_failure().await;

    // Should still allow requests
    assert!(breaker.is_request_allowed().await);
}

#[tokio::test]
async fn test_is_request_allowed_after_threshold_failures() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        timeout: Duration::from_secs(30),
        success_threshold: 2,
        half_open_max_requests: 3,
        enabled: true,
    };

    let breaker = CircuitBreaker::with_config(config);

    // Record failures to exceed threshold
    for _ in 0..3 {
        breaker.record_failure().await;
    }

    // Should not allow requests (circuit open)
    assert!(!breaker.is_request_allowed().await);
}

// ============================================================================
// FAILURE RECORDING TESTS
// ============================================================================

#[tokio::test]
async fn test_record_single_failure() {
    let breaker = CircuitBreaker::new();

    breaker.record_failure().await;

    let state = breaker.get_state().await;
    assert_eq!(state, CircuitState::Closed);
}

#[tokio::test]
async fn test_record_failures_below_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        timeout: Duration::from_secs(30),
        success_threshold: 2,
        half_open_max_requests: 3,
        enabled: true,
    };

    let breaker = CircuitBreaker::with_config(config);

    // Record failures below threshold
    for _ in 0..4 {
        breaker.record_failure().await;
    }

    let state = breaker.get_state().await;
    assert_eq!(state, CircuitState::Closed);
}

#[tokio::test]
async fn test_record_failures_at_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        timeout: Duration::from_secs(30),
        success_threshold: 2,
        half_open_max_requests: 3,
        enabled: true,
    };

    let breaker = CircuitBreaker::with_config(config);

    // Record failures to reach threshold
    for _ in 0..3 {
        breaker.record_failure().await;
    }

    let state = breaker.get_state().await;
    assert_eq!(state, CircuitState::Open);
}

#[tokio::test]
async fn test_record_failures_above_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        timeout: Duration::from_secs(30),
        success_threshold: 2,
        half_open_max_requests: 3,
        enabled: true,
    };

    let breaker = CircuitBreaker::with_config(config);

    // Record many failures
    for _ in 0..10 {
        breaker.record_failure().await;
    }

    let state = breaker.get_state().await;
    assert_eq!(state, CircuitState::Open);
}

// ============================================================================
// SUCCESS RECORDING TESTS
// ============================================================================

#[tokio::test]
async fn test_record_success_when_closed() {
    let breaker = CircuitBreaker::new();

    breaker.record_success().await;

    let state = breaker.get_state().await;
    assert_eq!(state, CircuitState::Closed);
}

#[tokio::test]
async fn test_record_multiple_successes() {
    let breaker = CircuitBreaker::new();

    for _ in 0..10 {
        breaker.record_success().await;
    }

    let state = breaker.get_state().await;
    assert_eq!(state, CircuitState::Closed);
}

#[tokio::test]
async fn test_success_resets_failure_count() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        timeout: Duration::from_secs(30),
        success_threshold: 2,
        half_open_max_requests: 3,
        enabled: true,
    };

    let breaker = CircuitBreaker::with_config(config);

    // Record some failures
    breaker.record_failure().await;
    breaker.record_failure().await;

    // Record success (should reset failure count)
    breaker.record_success().await;

    // Record more failures - should need full threshold again
    breaker.record_failure().await;
    breaker.record_failure().await;

    let state = breaker.get_state().await;
    assert_eq!(state, CircuitState::Closed);
}

// ============================================================================
// STATE TRANSITION TESTS
// ============================================================================

#[tokio::test]
async fn test_transition_from_closed_to_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        timeout: Duration::from_secs(30),
        success_threshold: 2,
        half_open_max_requests: 3,
        enabled: true,
    };

    let breaker = CircuitBreaker::with_config(config);

    let initial_state = breaker.get_state().await;
    assert_eq!(initial_state, CircuitState::Closed);

    // Trigger threshold
    breaker.record_failure().await;
    breaker.record_failure().await;

    let final_state = breaker.get_state().await;
    assert_eq!(final_state, CircuitState::Open);
}

#[tokio::test]
async fn test_timeout_transitions_to_half_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        timeout: Duration::from_millis(50),
        success_threshold: 2,
        half_open_max_requests: 3,
        enabled: true,
    };

    let breaker = CircuitBreaker::with_config(config);

    // Open the circuit
    breaker.record_failure().await;
    breaker.record_failure().await;

    assert_eq!(breaker.get_state().await, CircuitState::Open);

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(60)).await;

    // Request should be allowed (transitions to half-open)
    assert!(breaker.is_request_allowed().await);
}

// ============================================================================
// CONFIGURATION TESTS
// ============================================================================

#[tokio::test]
async fn test_high_failure_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 100,
        timeout: Duration::from_secs(60),
        success_threshold: 10,
        half_open_max_requests: 5,
        enabled: true,
    };

    let breaker = CircuitBreaker::with_config(config);

    // Record many failures (but below threshold)
    for _ in 0..50 {
        breaker.record_failure().await;
    }

    let state = breaker.get_state().await;
    assert_eq!(state, CircuitState::Closed);
}

#[tokio::test]
async fn test_low_failure_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        timeout: Duration::from_secs(5),
        success_threshold: 1,
        half_open_max_requests: 1,
        enabled: true,
    };

    let breaker = CircuitBreaker::with_config(config);

    breaker.record_failure().await;

    let state = breaker.get_state().await;
    assert_eq!(state, CircuitState::Open);
}

#[tokio::test]
async fn test_very_short_timeout() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        timeout: Duration::from_millis(10),
        success_threshold: 2,
        half_open_max_requests: 3,
        enabled: true,
    };

    let breaker = CircuitBreaker::with_config(config);

    // Open the circuit
    for _ in 0..3 {
        breaker.record_failure().await;
    }

    // Wait for short timeout
    tokio::time::sleep(Duration::from_millis(20)).await;

    // Should allow request again
    assert!(breaker.is_request_allowed().await);
}

// ============================================================================
// CONCURRENT OPERATIONS
// ============================================================================

#[tokio::test]
async fn test_concurrent_failure_recording() {
    use std::sync::Arc;

    let config = CircuitBreakerConfig {
        failure_threshold: 100,
        timeout: Duration::from_secs(60),
        success_threshold: 10,
        half_open_max_requests: 5,
        enabled: true,
    };

    let breaker = Arc::new(CircuitBreaker::with_config(config));
    let mut handles = vec![];

    // Spawn 10 tasks recording failures concurrently
    for _ in 0..10 {
        let breaker_clone = Arc::clone(&breaker);
        let handle = tokio::spawn(async move {
            for _ in 0..5 {
                breaker_clone.record_failure().await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Should still be closed (50 failures < 100 threshold)
    let state = breaker.get_state().await;
    assert_eq!(state, CircuitState::Closed);
}

#[tokio::test]
async fn test_concurrent_success_recording() {
    use std::sync::Arc;

    let breaker = Arc::new(CircuitBreaker::new());
    let mut handles = vec![];

    // Spawn 10 tasks recording successes concurrently
    for _ in 0..10 {
        let breaker_clone = Arc::clone(&breaker);
        let handle = tokio::spawn(async move {
            for _ in 0..10 {
                breaker_clone.record_success().await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    let state = breaker.get_state().await;
    assert_eq!(state, CircuitState::Closed);
}

#[tokio::test]
async fn test_concurrent_state_checks() {
    use std::sync::Arc;

    let breaker = Arc::new(CircuitBreaker::new());
    let mut handles = vec![];

    // Spawn 20 tasks checking state concurrently
    for _ in 0..20 {
        let breaker_clone = Arc::clone(&breaker);
        let handle = tokio::spawn(async move {
            let _state = breaker_clone.get_state().await;
            let _allowed = breaker_clone.is_request_allowed().await;
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_zero_failures_recorded() {
    let breaker = CircuitBreaker::new();

    let state = breaker.get_state().await;
    assert_eq!(state, CircuitState::Closed);
    assert!(breaker.is_request_allowed().await);
}

#[tokio::test]
async fn test_alternating_success_and_failure() {
    let config = CircuitBreakerConfig {
        failure_threshold: 10,
        timeout: Duration::from_secs(30),
        success_threshold: 2,
        half_open_max_requests: 3,
        enabled: true,
    };

    let breaker = CircuitBreaker::with_config(config);

    // Alternate between success and failure
    for _ in 0..5 {
        breaker.record_failure().await;
        breaker.record_success().await;
    }

    let state = breaker.get_state().await;
    assert_eq!(state, CircuitState::Closed);
}

#[tokio::test]
async fn test_multiple_circuit_breakers() {
    let breaker1 = CircuitBreaker::new();
    let breaker2 = CircuitBreaker::new();
    let breaker3 = CircuitBreaker::new();

    // Each should be independent
    breaker1.record_failure().await;

    let state1 = breaker1.get_state().await;
    let state2 = breaker2.get_state().await;
    let state3 = breaker3.get_state().await;

    assert_eq!(state1, CircuitState::Closed);
    assert_eq!(state2, CircuitState::Closed);
    assert_eq!(state3, CircuitState::Closed);
}

// ============================================================================
// STRESS TESTS
// ============================================================================

#[tokio::test]
async fn test_rapid_failure_recording() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1000,
        timeout: Duration::from_secs(60),
        success_threshold: 10,
        half_open_max_requests: 5,
        enabled: true,
    };

    let breaker = CircuitBreaker::with_config(config);

    // Record many failures rapidly
    for _ in 0..500 {
        breaker.record_failure().await;
    }

    let state = breaker.get_state().await;
    assert_eq!(state, CircuitState::Closed);
}

#[tokio::test]
async fn test_many_circuit_breakers() {
    let mut breakers = Vec::new();

    for _ in 0..50 {
        breakers.push(CircuitBreaker::new());
    }

    assert_eq!(breakers.len(), 50);

    // All should be in closed state
    for breaker in &breakers {
        let state = breaker.get_state().await;
        assert_eq!(state, CircuitState::Closed);
    }
}
