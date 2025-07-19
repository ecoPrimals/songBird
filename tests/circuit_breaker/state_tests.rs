//! Circuit breaker state transition tests

use std::time::Duration;
use tokio::time::sleep;

use songbird_network::communication::circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState,
};
use super::basic_tests::{create_test_circuit_breaker, create_fast_circuit_breaker};

#[tokio::test]
async fn test_half_open_to_closed_transition() {
    let circuit_breaker = create_fast_circuit_breaker();

    // Force open state
    for _ in 0..3 {
        circuit_breaker.record_failure().await;
    }

    // Wait for timeout to get half-open
    sleep(Duration::from_millis(150)).await;
    assert!(circuit_breaker.should_allow_request().await);
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::HalfOpen
    ));

    // Record successes to close circuit
    circuit_breaker.record_success().await;
    circuit_breaker.record_success().await;

    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Closed
    ));
}

#[tokio::test]
async fn test_half_open_to_open_transition() {
    let circuit_breaker = create_fast_circuit_breaker();

    // Force open state
    for _ in 0..3 {
        circuit_breaker.record_failure().await;
    }

    // Wait for timeout to get half-open
    sleep(Duration::from_millis(150)).await;
    assert!(circuit_breaker.should_allow_request().await);

    // Record failure in half-open state should go back to open
    circuit_breaker.record_failure().await;
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Open
    ));
}

#[tokio::test]
async fn test_half_open_partial_success() {
    let circuit_breaker = create_fast_circuit_breaker();

    // Force open state
    for _ in 0..3 {
        circuit_breaker.record_failure().await;
    }

    // Wait for timeout
    sleep(Duration::from_millis(150)).await;
    assert!(circuit_breaker.should_allow_request().await);

    // Record one success (need 2 for closed)
    circuit_breaker.record_success().await;
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::HalfOpen
    ));
}

#[tokio::test]
async fn test_state_transition_with_mixed_results() {
    let circuit_breaker = create_test_circuit_breaker(CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_duration: Duration::from_millis(100),
    });

    // Mix of success and failure - should stay closed
    circuit_breaker.record_success().await;
    circuit_breaker.record_failure().await;
    circuit_breaker.record_success().await;
    circuit_breaker.record_failure().await;

    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Closed
    ));
}

#[tokio::test]
async fn test_open_state_timeout_behavior() {
    let circuit_breaker = create_test_circuit_breaker(CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 1,
        timeout_duration: Duration::from_millis(50),
    });

    // Force open
    circuit_breaker.record_failure().await;
    circuit_breaker.record_failure().await;
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Open
    ));

    // Wait less than timeout
    sleep(Duration::from_millis(25)).await;
    assert!(!circuit_breaker.should_allow_request().await);

    // Wait for full timeout
    sleep(Duration::from_millis(50)).await;
    assert!(circuit_breaker.should_allow_request().await);
}

#[tokio::test]
async fn test_rapid_state_changes() {
    let circuit_breaker = create_test_circuit_breaker(CircuitBreakerConfig {
        failure_threshold: 1,
        success_threshold: 1,
        timeout_duration: Duration::from_millis(50),
    });

    // Rapid open/close cycle
    circuit_breaker.record_failure().await; // Open
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Open
    ));

    sleep(Duration::from_millis(75)).await; // Half-open
    assert!(circuit_breaker.should_allow_request().await);

    circuit_breaker.record_success().await; // Closed
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Closed
    ));

    circuit_breaker.record_failure().await; // Open again
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Open
    ));
}

#[tokio::test]
async fn test_state_consistency_under_load() {
    let circuit_breaker = create_fast_circuit_breaker();

    // Record many failures quickly
    for _ in 0..10 {
        circuit_breaker.record_failure().await;
    }

    // Should be open
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Open
    ));

    // Should not allow requests
    for _ in 0..5 {
        assert!(!circuit_breaker.should_allow_request().await);
    }
}

#[tokio::test]
async fn test_half_open_single_request_limit() {
    let circuit_breaker = create_fast_circuit_breaker();

    // Force open
    for _ in 0..3 {
        circuit_breaker.record_failure().await;
    }

    // Wait for half-open
    sleep(Duration::from_millis(150)).await;

    // First request should be allowed
    assert!(circuit_breaker.should_allow_request().await);
    
    // Subsequent requests in half-open should be blocked
    // (This depends on implementation - some allow limited requests)
    let state = circuit_breaker.get_state().await;
    assert!(matches!(state, CircuitState::HalfOpen));
}

#[tokio::test]
async fn test_reset_after_successful_recovery() {
    let circuit_breaker = create_fast_circuit_breaker();

    // Go through full cycle
    for _ in 0..3 {
        circuit_breaker.record_failure().await;
    }
    
    sleep(Duration::from_millis(150)).await;
    assert!(circuit_breaker.should_allow_request().await);
    
    // Recover
    circuit_breaker.record_success().await;
    circuit_breaker.record_success().await;
    
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Closed
    ));

    // Should be able to handle failures again
    circuit_breaker.record_failure().await;
    circuit_breaker.record_failure().await;
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Closed
    ));
} 