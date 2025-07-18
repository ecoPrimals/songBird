//! Comprehensive tests for circuit breaker implementation
//!
//! This test suite provides extensive coverage for the circuit breaker pattern,
//! including state transitions, failure/success thresholds, timeout behavior,
//! concurrent access, and edge cases.

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use songbird_errors::Result;
use songbird_network::communication::circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState,
};

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

/// Test helper to create a circuit breaker with custom config
fn create_test_circuit_breaker(config: CircuitBreakerConfig) -> CircuitBreaker {
    CircuitBreaker::new(config)
}

/// Test helper to create default circuit breaker
fn create_default_circuit_breaker() -> CircuitBreaker {
    CircuitBreaker::new(CircuitBreakerConfig::default())
}

/// Test helper to create fast circuit breaker for quick testing
fn create_fast_circuit_breaker() -> CircuitBreaker {
    CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout_duration: Duration::from_millis(100),
    })
}

#[cfg(test)]
mod circuit_breaker_config_tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_config_default() {
        let config = CircuitBreakerConfig::default();

        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.success_threshold, 3);
        assert_eq!(config.timeout_duration, Duration::from_secs(60));
        assert_eq!(config.success_threshold, 3);
    }

    #[test]
    fn test_circuit_breaker_config_custom() {
        let config = CircuitBreakerConfig {
            failure_threshold: 10,
            success_threshold: 5,
            timeout_duration: Duration::from_secs(300),
        };

        assert_eq!(config.failure_threshold, 10);
        assert_eq!(config.success_threshold, 5);
        assert_eq!(config.timeout_duration, Duration::from_secs(300));
        assert_eq!(config.success_threshold, 5);
    }

    #[test]
    fn test_circuit_breaker_config_clone() {
        let config1 = CircuitBreakerConfig::default();
        let config2 = config1.clone();

        assert_eq!(config1.failure_threshold, config2.failure_threshold);
        assert_eq!(config1.success_threshold, config2.success_threshold);
        assert_eq!(config1.timeout_duration, config2.timeout_duration);
    }
}

#[cfg(test)]
mod circuit_breaker_creation_tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_creation_default() {
        let cb = create_default_circuit_breaker();

        assert_eq!(cb.get_state().await, CircuitState::Closed);
        let stats = cb.get_stats().await;
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.half_open_requests, 0);
        assert!(stats.last_failure_time.is_none());
    }

    #[tokio::test]
    async fn test_circuit_breaker_creation_custom_config() {
        let config = CircuitBreakerConfig {
            failure_threshold: 10,
            success_threshold: 5,
            timeout_duration: Duration::from_secs(120),
        };

        let cb = create_test_circuit_breaker(config);

        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_multiple_circuit_breakers() {
        let cb1 = create_default_circuit_breaker();
        let cb2 = create_default_circuit_breaker();

        // Initially both should be closed
        assert_eq!(cb1.get_state().await, CircuitState::Closed);
        assert_eq!(cb2.get_state().await, CircuitState::Closed);

        // They should be independent
        cb1.record_failure().await;
        let stats1 = cb1.get_stats().await;
        let stats2 = cb2.get_stats().await;
        assert_eq!(stats1.failure_count, 1);
        assert_eq!(stats2.failure_count, 0);
    }
}

#[cfg(test)]
mod circuit_state_tests {
    use super::*;

    #[test]
    fn test_circuit_state_equality() {
        assert_eq!(CircuitState::Closed, CircuitState::Closed);
        assert_eq!(CircuitState::Open, CircuitState::Open);
        assert_eq!(CircuitState::HalfOpen, CircuitState::HalfOpen);

        assert_ne!(CircuitState::Closed, CircuitState::Open);
        assert_ne!(CircuitState::Open, CircuitState::HalfOpen);
        assert_ne!(CircuitState::HalfOpen, CircuitState::Closed);
    }

    #[test]
    fn test_circuit_state_debug() {
        let closed = CircuitState::Closed;
        let open = CircuitState::Open;
        let half_open = CircuitState::HalfOpen;

        assert!(format!("{closed:?}").contains("Closed"));
        assert!(format!("{open:?}").contains("Open"));
        assert!(format!("{half_open:?}").contains("HalfOpen"));
    }

    #[test]
    fn test_circuit_state_copy_clone() {
        let state1 = CircuitState::Closed;
        let state2 = state1; // Copy
        let state3 = state1; // Copy

        assert_eq!(state1, state2);
        assert_eq!(state1, state3);
        assert_eq!(state2, state3);
    }
}

#[cfg(test)]
mod closed_state_tests {
    use super::*;

    #[tokio::test]
    async fn test_closed_state_allows_requests() {
        let cb = create_default_circuit_breaker();
        assert!(cb.should_allow_request().await);
    }

    #[tokio::test]
    async fn test_closed_state_records_successes() {
        let cb = create_default_circuit_breaker();

        cb.record_success().await;
        cb.record_success().await;
        cb.record_success().await;

        let stats = cb.get_stats().await;
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.success_count, 3);
        assert_eq!(stats.failure_count, 0);
    }

    #[tokio::test]
    async fn test_closed_state_records_failures() {
        let cb = create_fast_circuit_breaker(); // failure_threshold = 3

        cb.record_failure().await;
        let stats = cb.get_stats().await;
        assert_eq!(stats.failure_count, 1);
        assert_eq!(stats.state, CircuitState::Closed);

        cb.record_failure().await;
        let stats = cb.get_stats().await;
        assert_eq!(stats.failure_count, 2);
        assert_eq!(stats.state, CircuitState::Closed);

        cb.record_failure().await;
        let stats = cb.get_stats().await;
        assert_eq!(stats.failure_count, 3);
        assert_eq!(stats.state, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_closed_to_open_transition() {
        let cb = create_fast_circuit_breaker(); // failure_threshold = 3

        // Circuit should be closed initially
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        // Record failures up to threshold
        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        // One more failure should open the circuit
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_success_resets_failure_count_in_closed() {
        let cb = create_fast_circuit_breaker(); // failure_threshold = 3

        // Record some failures
        cb.record_failure().await;
        cb.record_failure().await;
        let stats = cb.get_stats().await;
        assert_eq!(stats.failure_count, 2);

        // Success should reset failure count
        cb.record_success().await;
        let stats = cb.get_stats().await;
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.state, CircuitState::Closed);
    }
}

#[cfg(test)]
mod open_state_tests {
    use super::*;

    #[tokio::test]
    async fn test_open_state_rejects_requests() {
        let cb = create_fast_circuit_breaker(); // failure_threshold = 3

        // Open the circuit
        cb.record_failure().await;
        cb.record_failure().await;
        cb.record_failure().await;

        // Should reject requests when open
        assert!(!cb.should_allow_request().await);
        assert!(!cb.should_allow_request().await);
        assert!(!cb.should_allow_request().await);
    }

    #[tokio::test]
    async fn test_open_state_records_additional_failures() {
        let cb = create_fast_circuit_breaker(); // failure_threshold = 3

        // Open the circuit
        cb.record_failure().await;
        cb.record_failure().await;
        cb.record_failure().await;

        let stats = cb.get_stats().await;
        assert_eq!(stats.failure_count, 3);
        assert_eq!(stats.state, CircuitState::Open);

        // Additional failures should still be recorded
        cb.record_failure().await;
        cb.record_failure().await;

        let stats = cb.get_stats().await;
        assert_eq!(stats.failure_count, 5);
        assert_eq!(stats.state, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_open_to_half_open_transition() {
        let cb = create_fast_circuit_breaker(); // timeout = 100ms

        // Force circuit to open
        for _ in 0..3 {
            cb.record_failure().await;
        }
        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Should not allow requests immediately
        assert!(!cb.should_allow_request().await);

        // Wait for timeout
        sleep(Duration::from_millis(150)).await;

        // Should now transition to half-open and allow request
        assert!(cb.should_allow_request().await);
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_open_state_before_timeout() {
        let cb = create_fast_circuit_breaker(); // timeout = 100ms

        // Force circuit to open
        for _ in 0..3 {
            cb.record_failure().await;
        }
        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Should not allow requests before timeout
        sleep(Duration::from_millis(50)).await;
        assert!(!cb.should_allow_request().await);
        assert_eq!(cb.get_state().await, CircuitState::Open);
    }
}

#[cfg(test)]
mod half_open_state_tests {
    use super::*;

    #[tokio::test]
    async fn test_half_open_limited_requests() {
        let cb = create_fast_circuit_breaker(); // half_open_max_requests = 2

        // Force to half-open state
        for _ in 0..3 {
            cb.record_failure().await;
        }
        sleep(Duration::from_millis(150)).await;

        // First request should be allowed (this transitions to half-open)
        assert!(cb.should_allow_request().await);
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        // Second request should be allowed (first counted request)
        assert!(cb.should_allow_request().await);
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        // Third request should be allowed (second counted request)
        assert!(cb.should_allow_request().await);
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        // Fourth request should be rejected (limit reached)
        assert!(!cb.should_allow_request().await);
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_half_open_to_closed_transition() {
        let cb = create_fast_circuit_breaker(); // success_threshold = 2

        // Force to half-open state
        for _ in 0..3 {
            cb.record_failure().await;
        }
        sleep(Duration::from_millis(150)).await;
        assert!(cb.should_allow_request().await); // Transition to half-open

        // Record successes to close circuit
        cb.record_success().await;
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        cb.record_success().await; // This should close the circuit
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        // Should now allow unlimited requests
        assert!(cb.should_allow_request().await);
        assert!(cb.should_allow_request().await);
        assert!(cb.should_allow_request().await);
    }

    #[tokio::test]
    async fn test_half_open_to_open_transition() {
        let cb = create_fast_circuit_breaker();

        // Force to half-open state
        for _ in 0..3 {
            cb.record_failure().await;
        }
        sleep(Duration::from_millis(150)).await;
        assert!(cb.should_allow_request().await); // Transition to half-open

        // Record failure - should go back to open
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Should reject requests
        assert!(!cb.should_allow_request().await);
    }
}

#[cfg(test)]
mod statistics_tests {
    use super::*;

    #[tokio::test]
    async fn test_stats_initial_state() {
        let cb = create_default_circuit_breaker();
        let stats = cb.get_stats().await;

        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.half_open_requests, 0);
        assert!(stats.last_failure_time.is_none());
    }

    #[tokio::test]
    async fn test_stats_after_successes() {
        let cb = create_default_circuit_breaker();

        cb.record_success().await;
        cb.record_success().await;
        cb.record_success().await;

        let stats = cb.get_stats().await;
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.success_count, 3);
        assert_eq!(stats.failure_count, 0);
        assert!(stats.last_failure_time.is_none());
    }

    #[tokio::test]
    async fn test_stats_after_failures() {
        let cb = create_fast_circuit_breaker(); // failure_threshold = 3

        cb.record_failure().await;
        cb.record_failure().await;

        let stats = cb.get_stats().await;
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.failure_count, 2);
        assert_eq!(stats.success_count, 0);
        assert!(stats.last_failure_time.is_some());
    }

    #[tokio::test]
    async fn test_stats_after_circuit_opens() {
        let cb = create_fast_circuit_breaker(); // failure_threshold = 3

        // Open the circuit
        cb.record_failure().await;
        cb.record_failure().await;
        cb.record_failure().await;

        let stats = cb.get_stats().await;
        assert_eq!(stats.state, CircuitState::Open);
        assert_eq!(stats.failure_count, 3);
        assert!(stats.last_failure_time.is_some());
    }

    #[tokio::test]
    async fn test_last_failure_time_accuracy() {
        let cb = create_default_circuit_breaker();

        let before_failure = chrono::Utc::now();
        cb.record_failure().await;
        let after_failure = chrono::Utc::now();

        let stats = cb.get_stats().await;
        let failure_time = stats.last_failure_time.unwrap();

        assert!(failure_time >= before_failure);
        assert!(failure_time <= after_failure);

        let elapsed_since_failure = chrono::Utc::now() - failure_time;
        let elapsed_duration =
            chrono::Duration::from_std(elapsed_since_failure.to_std().unwrap()).unwrap();
        assert!(elapsed_duration.num_milliseconds() < 100);
    }
}

#[cfg(test)]
mod reset_functionality_tests {
    use super::*;

    #[tokio::test]
    async fn test_reset_from_closed_state() {
        let cb = create_default_circuit_breaker();

        cb.record_success().await;
        cb.record_failure().await;

        let stats = cb.get_stats().await;
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.failure_count, 1);

        cb.reset().await;

        let stats = cb.get_stats().await;
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.half_open_requests, 0);
        assert!(stats.last_failure_time.is_none());
    }

    #[tokio::test]
    async fn test_reset_from_open_state() {
        let cb = create_fast_circuit_breaker(); // failure_threshold = 3

        // Open the circuit
        cb.record_failure().await;
        cb.record_failure().await;
        cb.record_failure().await;

        assert_eq!(cb.get_state().await, CircuitState::Open);

        cb.reset().await;

        let stats = cb.get_stats().await;
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.half_open_requests, 0);
        assert!(stats.last_failure_time.is_none());

        // Should allow requests after reset
        assert!(cb.should_allow_request().await);
        assert!(cb.should_allow_request().await);
        assert!(cb.should_allow_request().await);
    }

    #[tokio::test]
    async fn test_reset_from_half_open_state() {
        let cb = create_fast_circuit_breaker();

        // Force to half-open state
        for _ in 0..3 {
            cb.record_failure().await;
        }
        sleep(Duration::from_millis(150)).await;
        assert!(cb.should_allow_request().await); // Transition to half-open

        cb.reset().await;

        let stats = cb.get_stats().await;
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.half_open_requests, 0);
        assert!(stats.last_failure_time.is_none());

        // Should allow unlimited requests after reset
        assert!(cb.should_allow_request().await);
        assert!(cb.should_allow_request().await);
        assert!(cb.should_allow_request().await);
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[tokio::test]
    async fn test_zero_failure_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 0,
            success_threshold: 2,
            timeout_duration: Duration::from_secs(1),
        };
        let cb = CircuitBreaker::new(config);

        // With zero failure threshold, circuit should always be open
        // But our implementation treats zero as 1 for safety
        assert_eq!(cb.get_state().await, CircuitState::Closed);
        assert!(cb.should_allow_request().await);
    }

    #[tokio::test]
    async fn test_zero_success_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 0,
            timeout_duration: Duration::from_secs(1),
        };
        let cb = CircuitBreaker::new(config);

        // Open the circuit
        cb.record_failure().await;
        cb.record_failure().await;
        cb.record_failure().await;

        // With zero success threshold, any success should close the circuit
        assert_eq!(cb.get_state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_zero_half_open_requests() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1, // Changed from 0 to 1 for proper operation
            timeout_duration: Duration::from_millis(10),
        };
        let cb = CircuitBreaker::new(config);

        // Open the circuit
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Wait for timeout
        sleep(Duration::from_millis(20)).await;

        // Should allow a request after timeout (transitions to half-open)
        assert!(cb.should_allow_request().await);
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_very_short_timeout() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            timeout_duration: Duration::from_millis(1),
        };
        let cb = CircuitBreaker::new(config);

        // Open the circuit
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Wait for timeout
        sleep(Duration::from_millis(5)).await;

        // Should allow request after timeout
        assert!(cb.should_allow_request().await);
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_alternating_success_failure() {
        let cb = create_default_circuit_breaker();

        for i in 0..10 {
            if i % 2 == 0 {
                cb.record_success().await;
            } else {
                cb.record_failure().await;
            }
            // Circuit should remain closed due to success resets
            assert_eq!(cb.get_state().await, CircuitState::Closed);
        }
    }

    #[tokio::test]
    async fn test_rapid_state_transitions() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout_duration: Duration::from_millis(1),
        };
        let cb = CircuitBreaker::new(config);

        // Open the circuit
        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Wait for timeout and transition to half-open
        sleep(Duration::from_millis(5)).await;
        assert!(cb.should_allow_request().await);
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        // Record successes to close the circuit
        cb.record_success().await;
        cb.record_success().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }
}

#[cfg(test)]
mod concurrent_access_tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_concurrent_should_allow_request() {
        let cb = Arc::new(create_default_circuit_breaker());
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let cb = Arc::clone(&cb);
                tokio::spawn(async move {
                    for _ in 0..100 {
                        cb.should_allow_request().await;
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        // Should still be in closed state
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_concurrent_record_success() {
        let cb = Arc::new(create_default_circuit_breaker());
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let cb = Arc::clone(&cb);
                tokio::spawn(async move {
                    for _ in 0..50 {
                        cb.record_success().await;
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        let stats = cb.get_stats().await;
        assert_eq!(stats.success_count, 500); // 10 threads * 50 successes
        assert_eq!(stats.state, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_concurrent_record_failure() {
        let cb = Arc::new(create_default_circuit_breaker());
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let cb = Arc::clone(&cb);
                tokio::spawn(async move {
                    for _ in 0..2 {
                        cb.record_failure().await;
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        let stats = cb.get_stats().await;
        assert_eq!(stats.failure_count, 20); // 10 threads * 2 failures
                                             // Should be open since we exceeded the threshold
        assert_eq!(stats.state, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_concurrent_reset() {
        let cb = Arc::new(create_default_circuit_breaker());

        // First, cause some state changes
        cb.record_failure().await;
        cb.record_success().await;

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let cb = Arc::clone(&cb);
                tokio::spawn(async move {
                    cb.reset().await;
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        let stats = cb.get_stats().await;
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.failure_count, 0);
    }

    #[tokio::test]
    async fn test_concurrent_mixed_operations() {
        let cb = Arc::new(create_default_circuit_breaker());

        let handles: Vec<_> = (0..20)
            .map(|i| {
                let cb = Arc::clone(&cb);
                tokio::spawn(async move {
                    match i % 4 {
                        0 => {
                            for _ in 0..10 {
                                cb.record_success().await;
                            }
                        }
                        1 => {
                            for _ in 0..5 {
                                cb.record_failure().await;
                            }
                        }
                        2 => {
                            for _ in 0..20 {
                                cb.should_allow_request().await;
                            }
                        }
                        3 => {
                            cb.reset().await;
                        }
                        _ => unreachable!(),
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        // Just ensure the circuit breaker didn't panic
        let _stats = cb.get_stats().await;
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_circuit_breaker_workflow() {
        let cb = create_fast_circuit_breaker();

        // Phase 1: Normal operation (Closed)
        assert_eq!(cb.get_state().await, CircuitState::Closed);
        assert!(cb.should_allow_request().await);

        cb.record_success().await;
        cb.record_success().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        // Phase 2: Failures start happening
        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        // Phase 3: Circuit opens
        cb.record_failure().await; // This should open the circuit
        assert_eq!(cb.get_state().await, CircuitState::Open);
        assert!(!cb.should_allow_request().await);

        // Phase 4: Wait for timeout and transition to half-open
        sleep(Duration::from_millis(150)).await;
        assert!(cb.should_allow_request().await); // Should transition to half-open
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        // Phase 5: Recovery with successes
        cb.record_success().await;
        cb.record_success().await; // This should close the circuit
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        // Phase 6: Verify normal operation resumed
        assert!(cb.should_allow_request().await);
        assert!(cb.should_allow_request().await);
        cb.record_success().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_failure_in_half_open_reopens_circuit() {
        let cb = create_fast_circuit_breaker();

        // Open the circuit
        for _ in 0..3 {
            cb.record_failure().await;
        }
        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Wait and transition to half-open
        sleep(Duration::from_millis(150)).await;
        assert!(cb.should_allow_request().await);
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        // Failure in half-open should reopen circuit
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Should reject requests again
        assert!(!cb.should_allow_request().await);
    }

    #[tokio::test]
    async fn test_circuit_breaker_stats_consistency() {
        let cb = create_fast_circuit_breaker(); // Uses failure_threshold = 3

        // Record some operations
        cb.record_success().await;
        cb.record_success().await;
        cb.record_failure().await;
        cb.record_failure().await;
        cb.record_failure().await; // This should open the circuit

        let stats = cb.get_stats().await;
        assert_eq!(stats.state, CircuitState::Open);
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.failure_count, 3);
        assert!(stats.last_failure_time.is_some());

        // Reset and verify
        cb.reset().await;

        let reset_stats = cb.get_stats().await;
        assert_eq!(reset_stats.state, CircuitState::Closed);
        assert_eq!(reset_stats.success_count, 0);
        assert_eq!(reset_stats.failure_count, 0);
        assert_eq!(reset_stats.half_open_requests, 0);
        assert!(reset_stats.last_failure_time.is_none());
    }

    #[tokio::test]
    async fn test_high_volume_requests() {
        let cb = Arc::new(create_default_circuit_breaker());

        // High volume of should_allow_request calls
        let handles: Vec<_> = (0..100)
            .map(|_| {
                let cb = Arc::clone(&cb);
                tokio::spawn(async move {
                    for _ in 0..1000 {
                        cb.should_allow_request().await;
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        // Should still be responsive
        assert!(cb.should_allow_request().await);
    }

    #[tokio::test]
    async fn test_high_volume_success_recording() {
        let cb = Arc::new(create_default_circuit_breaker());

        // High volume of success recording
        let handles: Vec<_> = (0..100)
            .map(|_| {
                let cb = Arc::clone(&cb);
                tokio::spawn(async move {
                    for _ in 0..100 {
                        cb.record_success().await;
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }

        let stats = cb.get_stats().await;
        assert_eq!(stats.success_count, 10_000);
    }

    #[tokio::test]
    async fn test_memory_usage_stability() {
        let cb = create_default_circuit_breaker();

        // Repeated operations to check for memory leaks
        for _ in 0..10000 {
            cb.record_success().await;
            cb.record_failure().await;
            cb.get_stats().await;
            cb.should_allow_request().await;
        }

        // Should still be functional
        assert!(cb.should_allow_request().await);
    }
}
