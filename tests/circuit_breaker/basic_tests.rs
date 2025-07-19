//! Basic circuit breaker functionality tests

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use songbird_errors::Result;
use songbird_network::communication::circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState,
};

/// Test helper to create a circuit breaker with custom config
pub fn create_test_circuit_breaker(config: CircuitBreakerConfig) -> CircuitBreaker {
    CircuitBreaker::new(config)
}

/// Test helper to create default circuit breaker
pub fn create_default_circuit_breaker() -> CircuitBreaker {
    CircuitBreaker::new(CircuitBreakerConfig::default())
}

/// Test helper to create fast circuit breaker for quick testing
pub fn create_fast_circuit_breaker() -> CircuitBreaker {
    CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_duration: Duration::from_millis(100),
    })
}

#[tokio::test]
async fn test_circuit_breaker_basic_functionality() -> Result<()> {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_duration: Duration::from_secs(1),
    };

    let circuit_breaker = CircuitBreaker::new(config);

    // Initially closed
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Closed
    ));

    // Record failures
    circuit_breaker.record_failure().await;
    circuit_breaker.record_failure().await;
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Closed
    ));

    // Third failure should open the circuit
    circuit_breaker.record_failure().await;
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Open
    ));

    // Should reject calls when open
    assert!(!circuit_breaker.should_allow_request().await);

    Ok(())
}

#[tokio::test]
async fn test_circuit_breaker_initial_state() {
    let circuit_breaker = create_default_circuit_breaker();

    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Closed
    ));
    assert!(circuit_breaker.should_allow_request().await);
}

#[tokio::test]
async fn test_circuit_breaker_closed_to_open_transition() {
    let circuit_breaker = create_test_circuit_breaker(CircuitBreakerConfig {
        failure_threshold: 2,
        success_threshold: 2,
        timeout_duration: Duration::from_secs(1),
    });

    // Record failures below threshold
    circuit_breaker.record_failure().await;
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Closed
    ));

    // Second failure should open the circuit
    circuit_breaker.record_failure().await;
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Open
    ));
}

#[tokio::test]
async fn test_circuit_breaker_open_state_behavior() {
    let circuit_breaker = create_fast_circuit_breaker();

    // Force open state
    for _ in 0..3 {
        circuit_breaker.record_failure().await;
    }

    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Open
    ));
    assert!(!circuit_breaker.should_allow_request().await);
}

#[tokio::test]
async fn test_circuit_breaker_timeout_transition_to_half_open() {
    let circuit_breaker = create_fast_circuit_breaker();

    // Force open state
    for _ in 0..3 {
        circuit_breaker.record_failure().await;
    }

    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Open
    ));

    // Wait for timeout
    sleep(Duration::from_millis(150)).await;

    // Should transition to half-open
    assert!(circuit_breaker.should_allow_request().await);
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::HalfOpen
    ));
}

#[tokio::test]
async fn test_circuit_breaker_success_count_reset() {
    let circuit_breaker = create_test_circuit_breaker(CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_duration: Duration::from_millis(100),
    });

    // Record one failure
    circuit_breaker.record_failure().await;

    // Record success (should reset failure count)
    circuit_breaker.record_success().await;

    // Now need 3 more failures to open
    circuit_breaker.record_failure().await;
    circuit_breaker.record_failure().await;
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Closed
    ));

    circuit_breaker.record_failure().await;
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Open
    ));
}

#[tokio::test]
async fn test_circuit_breaker_metrics_tracking() {
    let circuit_breaker = create_default_circuit_breaker();

    circuit_breaker.record_success().await;
    circuit_breaker.record_failure().await;
    circuit_breaker.record_success().await;

    let metrics = circuit_breaker.get_metrics().await;
    assert_eq!(metrics.total_requests, 3);
    assert_eq!(metrics.successful_requests, 2);
    assert_eq!(metrics.failed_requests, 1);
}

#[tokio::test]
async fn test_circuit_breaker_state_persistence() {
    let circuit_breaker = create_fast_circuit_breaker();

    // Force open state
    for _ in 0..3 {
        circuit_breaker.record_failure().await;
    }

    let state_before = circuit_breaker.get_state().await;
    
    // State should persist across calls
    let state_after = circuit_breaker.get_state().await;
    assert_eq!(std::mem::discriminant(&state_before), std::mem::discriminant(&state_after));
}

#[tokio::test]
async fn test_circuit_breaker_should_allow_request_closed() {
    let circuit_breaker = create_default_circuit_breaker();
    
    // In closed state, should always allow requests
    assert!(circuit_breaker.should_allow_request().await);
    assert!(circuit_breaker.should_allow_request().await);
    assert!(circuit_breaker.should_allow_request().await);
}

#[tokio::test]
async fn test_circuit_breaker_failure_threshold_exact() {
    let circuit_breaker = create_test_circuit_breaker(CircuitBreakerConfig {
        failure_threshold: 1,
        success_threshold: 1,
        timeout_duration: Duration::from_secs(1),
    });

    // Single failure should open circuit
    circuit_breaker.record_failure().await;
    assert!(matches!(
        circuit_breaker.get_state().await,
        CircuitState::Open
    ));
} 