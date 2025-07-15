//! Comprehensive tests for circuit breaker implementation
//!
//! This test suite provides extensive coverage for the circuit breaker pattern,
//! including state transitions, failure/success thresholds, timeout behavior,
//! concurrent access, and edge cases.

use std::time::Duration;
use tokio::time::sleep;

use songbird_errors::Result;
use songbird_lib::communication::circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitState,
};

#[tokio::test]
async fn test_circuit_breaker_basic_functionality() -> Result<()> {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        timeout: Duration::from_secs(1),
        window_size: Duration::from_secs(60),
        half_open_max_requests: 2,
    };

    let circuit_breaker = CircuitBreaker::new(config);

    // Initially closed
    assert!(matches!(circuit_breaker.get_state(), CircuitState::Closed));

    // Record failures
    circuit_breaker.record_failure();
    circuit_breaker.record_failure();
    assert!(matches!(circuit_breaker.get_state(), CircuitState::Closed));

    // Third failure should open the circuit
    circuit_breaker.record_failure();
    assert!(matches!(circuit_breaker.get_state(), CircuitState::Open));

    // Should reject calls when open
    assert!(!circuit_breaker.should_allow_request());

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
        timeout: Duration::from_millis(100),
        window_size: Duration::from_millis(500),
        half_open_max_requests: 2,
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
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.window_size, Duration::from_secs(60));
        assert_eq!(config.half_open_max_requests, 3);
    }

    #[test]
    fn test_circuit_breaker_config_custom() {
        let config = CircuitBreakerConfig {
            failure_threshold: 10,
            success_threshold: 5,
            timeout: Duration::from_secs(120),
            window_size: Duration::from_secs(300),
            half_open_max_requests: 5,
        };

        assert_eq!(config.failure_threshold, 10);
        assert_eq!(config.success_threshold, 5);
        assert_eq!(config.timeout, Duration::from_secs(120));
        assert_eq!(config.window_size, Duration::from_secs(300));
        assert_eq!(config.half_open_max_requests, 5);
    }

    #[test]
    fn test_circuit_breaker_config_clone() {
        let config1 = CircuitBreakerConfig::default();
        let config2 = config1.clone();

        assert_eq!(config1.failure_threshold, config2.failure_threshold);
        assert_eq!(config1.success_threshold, config2.success_threshold);
        assert_eq!(config1.timeout, config2.timeout);
        assert_eq!(config1.window_size, config2.window_size);
        assert_eq!(
            config1.half_open_max_requests,
            config2.half_open_max_requests
        );
    }
}

#[cfg(test)]
mod circuit_breaker_creation_tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_creation_default() {
        let cb = create_default_circuit_breaker();

        assert_eq!(cb.get_state(), CircuitState::Closed);

        let stats = cb.get_stats();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.half_open_requests, 0);
        assert!(stats.last_failure_time.is_none());
    }

    #[test]
    fn test_circuit_breaker_creation_custom_config() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_millis(50),
            window_size: Duration::from_millis(100),
            half_open_max_requests: 1,
        };

        let cb = create_test_circuit_breaker(config);
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_multiple_circuit_breakers() {
        let cb1 = create_default_circuit_breaker();
        let cb2 = create_default_circuit_breaker();

        assert_eq!(cb1.get_state(), CircuitState::Closed);
        assert_eq!(cb2.get_state(), CircuitState::Closed);

        // They should be independent
        cb1.record_failure();
        assert_eq!(cb1.get_stats().failure_count, 1);
        assert_eq!(cb2.get_stats().failure_count, 0);
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

    #[test]
    fn test_closed_state_allows_requests() {
        let cb = create_default_circuit_breaker();

        assert_eq!(cb.get_state(), CircuitState::Closed);
        assert!(cb.should_allow_request());
        assert!(cb.should_allow_request());
        assert!(cb.should_allow_request());
    }

    #[test]
    fn test_closed_state_records_successes() {
        let cb = create_default_circuit_breaker();

        cb.record_success();
        cb.record_success();
        cb.record_success();

        let stats = cb.get_stats();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.success_count, 3);
        assert_eq!(stats.failure_count, 0);
    }

    #[test]
    fn test_closed_state_records_failures() {
        let cb = create_fast_circuit_breaker(); // failure_threshold = 3

        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Closed);
        assert_eq!(cb.get_stats().failure_count, 1);

        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Closed);
        assert_eq!(cb.get_stats().failure_count, 2);
    }

    #[test]
    fn test_closed_to_open_transition() {
        let cb = create_fast_circuit_breaker(); // failure_threshold = 3

        // Record failures up to threshold
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Closed);

        // This should trigger the transition to Open
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Open);

        let stats = cb.get_stats();
        assert_eq!(stats.failure_count, 3);
        assert!(stats.last_failure_time.is_some());
    }

    #[test]
    fn test_success_resets_failure_count_in_closed() {
        let cb = create_fast_circuit_breaker(); // failure_threshold = 3

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.get_stats().failure_count, 2);

        // Success should reset failure count
        cb.record_success();
        assert_eq!(cb.get_stats().failure_count, 0);
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }
}

#[cfg(test)]
mod open_state_tests {
    use super::*;

    #[test]
    fn test_open_state_rejects_requests() {
        let cb = create_fast_circuit_breaker();

        // Force circuit to open
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.get_state(), CircuitState::Open);

        // Should reject requests
        assert!(!cb.should_allow_request());
        assert!(!cb.should_allow_request());
        assert!(!cb.should_allow_request());
    }

    #[test]
    fn test_open_state_records_additional_failures() {
        let cb = create_fast_circuit_breaker();

        // Force circuit to open
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.get_state(), CircuitState::Open);

        // Record additional failures
        cb.record_failure();
        cb.record_failure();

        let stats = cb.get_stats();
        assert_eq!(stats.state, CircuitState::Open);
        assert_eq!(stats.failure_count, 5);
    }

    #[tokio::test]
    async fn test_open_to_half_open_transition() {
        let cb = create_fast_circuit_breaker(); // timeout = 100ms

        // Force circuit to open
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.get_state(), CircuitState::Open);

        // Should not allow requests immediately
        assert!(!cb.should_allow_request());

        // Wait for timeout
        sleep(Duration::from_millis(150)).await;

        // Should now transition to half-open and allow request
        assert!(cb.should_allow_request());
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_open_state_before_timeout() {
        let cb = create_fast_circuit_breaker(); // timeout = 100ms

        // Force circuit to open
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.get_state(), CircuitState::Open);

        // Should not allow requests before timeout
        sleep(Duration::from_millis(50)).await;
        assert!(!cb.should_allow_request());
        assert_eq!(cb.get_state(), CircuitState::Open);
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
            cb.record_failure();
        }
        sleep(Duration::from_millis(150)).await;

        // First request should be allowed (this transitions to half-open)
        assert!(cb.should_allow_request());
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);

        // Second request should be allowed (first counted request)
        assert!(cb.should_allow_request());
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);

        // Third request should be allowed (second counted request)
        assert!(cb.should_allow_request());
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);

        // Fourth request should be rejected (limit reached)
        assert!(!cb.should_allow_request());
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_half_open_to_closed_transition() {
        let cb = create_fast_circuit_breaker(); // success_threshold = 2

        // Force to half-open state
        for _ in 0..3 {
            cb.record_failure();
        }
        sleep(Duration::from_millis(150)).await;
        assert!(cb.should_allow_request()); // Transition to half-open

        // Record successes to close circuit
        cb.record_success();
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);

        cb.record_success(); // This should close the circuit
        assert_eq!(cb.get_state(), CircuitState::Closed);

        // Should now allow unlimited requests
        assert!(cb.should_allow_request());
        assert!(cb.should_allow_request());
        assert!(cb.should_allow_request());
    }

    #[tokio::test]
    async fn test_half_open_to_open_transition() {
        let cb = create_fast_circuit_breaker();

        // Force to half-open state
        for _ in 0..3 {
            cb.record_failure();
        }
        sleep(Duration::from_millis(150)).await;
        assert!(cb.should_allow_request()); // Transition to half-open

        // Record failure - should go back to open
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Open);

        // Should reject requests
        assert!(!cb.should_allow_request());
    }
}

#[cfg(test)]
mod statistics_tests {
    use super::*;

    #[test]
    fn test_stats_initial_state() {
        let cb = create_default_circuit_breaker();
        let stats = cb.get_stats();

        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.half_open_requests, 0);
        assert!(stats.last_failure_time.is_none());
    }

    #[test]
    fn test_stats_after_successes() {
        let cb = create_default_circuit_breaker();

        cb.record_success();
        cb.record_success();
        cb.record_success();

        let stats = cb.get_stats();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.success_count, 3);
        assert_eq!(stats.half_open_requests, 0);
        assert!(stats.last_failure_time.is_none());
    }

    #[test]
    fn test_stats_after_failures() {
        let cb = create_default_circuit_breaker();

        cb.record_failure();
        cb.record_failure();

        let stats = cb.get_stats();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.failure_count, 2);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.half_open_requests, 0);
        assert!(stats.last_failure_time.is_some());
    }

    #[test]
    fn test_stats_after_circuit_opens() {
        let cb = create_fast_circuit_breaker();

        // Force circuit to open
        for _ in 0..3 {
            cb.record_failure();
        }

        let stats = cb.get_stats();
        assert_eq!(stats.state, CircuitState::Open);
        assert_eq!(stats.failure_count, 3);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.half_open_requests, 0);
        assert!(stats.last_failure_time.is_some());
    }

    #[test]
    fn test_last_failure_time_accuracy() {
        let cb = create_default_circuit_breaker();
        let before = std::time::Instant::now();

        cb.record_failure();

        let after = std::time::Instant::now();
        let stats = cb.get_stats();

        assert!(stats.last_failure_time.is_some());
        // The failure time should be between before and after
        let failure_time = stats.last_failure_time.unwrap();
        let now = chrono::Utc::now();
        let elapsed_since_failure = now - failure_time;
        let elapsed_duration = elapsed_since_failure.to_std().unwrap();

        assert!(elapsed_duration <= after - before + Duration::from_millis(10));
        // Small buffer for timing
    }
}

#[cfg(test)]
mod reset_functionality_tests {
    use super::*;

    #[test]
    fn test_reset_from_closed_state() {
        let cb = create_default_circuit_breaker();

        cb.record_success();
        cb.record_failure();

        assert_eq!(cb.get_stats().success_count, 1);
        assert_eq!(cb.get_stats().failure_count, 1);

        cb.reset();

        let stats = cb.get_stats();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.half_open_requests, 0);
        assert!(stats.last_failure_time.is_none());
    }

    #[test]
    fn test_reset_from_open_state() {
        let cb = create_fast_circuit_breaker();

        // Force circuit to open
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.get_state(), CircuitState::Open);

        cb.reset();

        let stats = cb.get_stats();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.half_open_requests, 0);
        assert!(stats.last_failure_time.is_none());

        // Should allow requests after reset
        assert!(cb.should_allow_request());
    }

    #[tokio::test]
    async fn test_reset_from_half_open_state() {
        let cb = create_fast_circuit_breaker();

        // Force to half-open state
        for _ in 0..3 {
            cb.record_failure();
        }
        sleep(Duration::from_millis(150)).await;
        assert!(cb.should_allow_request()); // Transition to half-open

        cb.reset();

        let stats = cb.get_stats();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.half_open_requests, 0);
        assert!(stats.last_failure_time.is_none());

        // Should allow unlimited requests after reset
        assert!(cb.should_allow_request());
        assert!(cb.should_allow_request());
        assert!(cb.should_allow_request());
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_zero_failure_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 0,
            success_threshold: 1,
            timeout: Duration::from_millis(100),
            window_size: Duration::from_millis(500),
            half_open_max_requests: 1,
        };

        let cb = create_test_circuit_breaker(config);

        // Even with zero threshold, should handle gracefully
        cb.record_failure();
        // Behavior may vary - just ensure it doesn't panic
        let _state = cb.get_state();
    }

    #[test]
    fn test_zero_success_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 0,
            timeout: Duration::from_millis(100),
            window_size: Duration::from_millis(500),
            half_open_max_requests: 1,
        };

        let cb = create_test_circuit_breaker(config);

        // Force to open then half-open
        cb.record_failure();
        // Behavior may vary - just ensure it doesn't panic
        let _state = cb.get_state();
    }

    #[test]
    fn test_zero_half_open_requests() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            timeout: Duration::from_millis(100),
            window_size: Duration::from_millis(500),
            half_open_max_requests: 0,
        };

        let cb = create_test_circuit_breaker(config);

        cb.record_failure();
        // Should handle zero half-open requests gracefully
        let _allowed = cb.should_allow_request();
    }

    #[test]
    fn test_very_short_timeout() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 1,
            timeout: Duration::from_nanos(1),
            window_size: Duration::from_millis(500),
            half_open_max_requests: 1,
        };

        let cb = create_test_circuit_breaker(config);

        cb.record_failure();
        // Even with very short timeout, should not panic
        let _allowed = cb.should_allow_request();
    }

    #[test]
    fn test_alternating_success_failure() {
        let cb = create_fast_circuit_breaker();

        for i in 0..10 {
            if i % 2 == 0 {
                cb.record_success();
            } else {
                cb.record_failure();
            }

            // Should handle alternating pattern gracefully
            let _state = cb.get_state();
            let _allowed = cb.should_allow_request();
        }
    }

    #[test]
    fn test_rapid_state_transitions() {
        let cb = create_fast_circuit_breaker();

        // Rapid failures to open
        for _ in 0..3 {
            cb.record_failure();
        }

        // Reset and repeat multiple times
        for _ in 0..5 {
            cb.reset();
            for _ in 0..3 {
                cb.record_failure();
            }
            assert_eq!(cb.get_state(), CircuitState::Open);
        }
    }
}

#[cfg(test)]
mod concurrent_access_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_concurrent_should_allow_request() {
        let cb = Arc::new(create_default_circuit_breaker());
        let mut handles = vec![];

        for _ in 0..10 {
            let cb_clone = Arc::clone(&cb);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let _allowed = cb_clone.should_allow_request();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Should complete without panic
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }

    #[test]
    fn test_concurrent_record_success() {
        let cb = Arc::new(create_default_circuit_breaker());
        let mut handles = vec![];

        for _ in 0..10 {
            let cb_clone = Arc::clone(&cb);
            let handle = thread::spawn(move || {
                for _ in 0..50 {
                    cb_clone.record_success();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = cb.get_stats();
        assert_eq!(stats.success_count, 500); // 10 threads * 50 successes
        assert_eq!(stats.state, CircuitState::Closed);
    }

    #[test]
    fn test_concurrent_record_failure() {
        let cb = Arc::new(create_default_circuit_breaker());
        let mut handles = vec![];

        for _ in 0..10 {
            let cb_clone = Arc::clone(&cb);
            let handle = thread::spawn(move || {
                for _ in 0..2 {
                    cb_clone.record_failure();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = cb.get_stats();
        assert_eq!(stats.failure_count, 20); // 10 threads * 2 failures
                                             // Circuit should be open due to high failure count
        assert_eq!(stats.state, CircuitState::Open);
    }

    #[test]
    fn test_concurrent_reset() {
        let cb = Arc::new(create_default_circuit_breaker());
        let mut handles = vec![];

        // First, generate some state
        cb.record_failure();
        cb.record_success();

        for _ in 0..5 {
            let cb_clone = Arc::clone(&cb);
            let handle = thread::spawn(move || {
                cb_clone.reset();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let stats = cb.get_stats();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.failure_count, 0);
    }

    #[test]
    fn test_concurrent_mixed_operations() {
        let cb = Arc::new(create_fast_circuit_breaker());
        let mut handles = vec![];

        // Mix of different operations
        for i in 0..8 {
            let cb_clone = Arc::clone(&cb);
            let handle = thread::spawn(move || match i % 4 {
                0 => {
                    for _ in 0..10 {
                        cb_clone.record_success();
                    }
                }
                1 => {
                    for _ in 0..5 {
                        cb_clone.record_failure();
                    }
                }
                2 => {
                    for _ in 0..20 {
                        let _allowed = cb_clone.should_allow_request();
                    }
                }
                3 => {
                    let _stats = cb_clone.get_stats();
                }
                _ => unreachable!(),
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Should complete without panic and have consistent state
        let _final_stats = cb.get_stats();
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_circuit_breaker_workflow() {
        let cb = create_fast_circuit_breaker();

        // Phase 1: Normal operation (Closed)
        assert_eq!(cb.get_state(), CircuitState::Closed);
        assert!(cb.should_allow_request());

        cb.record_success();
        cb.record_success();
        assert_eq!(cb.get_state(), CircuitState::Closed);

        // Phase 2: Failures start happening
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Closed);

        // Phase 3: Circuit opens
        cb.record_failure(); // This should open the circuit
        assert_eq!(cb.get_state(), CircuitState::Open);
        assert!(!cb.should_allow_request());

        // Phase 4: Wait for timeout and transition to half-open
        sleep(Duration::from_millis(150)).await;
        assert!(cb.should_allow_request()); // Should transition to half-open
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);

        // Phase 5: Recovery with successes
        cb.record_success();
        cb.record_success(); // This should close the circuit
        assert_eq!(cb.get_state(), CircuitState::Closed);

        // Phase 6: Verify normal operation resumed
        assert!(cb.should_allow_request());
        assert!(cb.should_allow_request());
        cb.record_success();
        assert_eq!(cb.get_state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_failure_in_half_open_reopens_circuit() {
        let cb = create_fast_circuit_breaker();

        // Open the circuit
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.get_state(), CircuitState::Open);

        // Wait and transition to half-open
        sleep(Duration::from_millis(150)).await;
        assert!(cb.should_allow_request());
        assert_eq!(cb.get_state(), CircuitState::HalfOpen);

        // Failure in half-open should reopen circuit
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitState::Open);

        // Should reject requests again
        assert!(!cb.should_allow_request());
    }

    #[test]
    fn test_circuit_breaker_stats_consistency() {
        let cb = create_fast_circuit_breaker();

        // Record some operations
        cb.record_success();
        cb.record_success();
        cb.record_failure();
        cb.record_failure();
        cb.record_failure(); // Should open circuit

        let stats = cb.get_stats();

        // Verify stats consistency
        assert_eq!(stats.state, CircuitState::Open);
        assert_eq!(stats.success_count, 2);
        assert_eq!(stats.failure_count, 3);
        assert!(stats.last_failure_time.is_some());

        // Reset and verify
        cb.reset();
        let reset_stats = cb.get_stats();

        assert_eq!(reset_stats.state, CircuitState::Closed);
        assert_eq!(reset_stats.success_count, 0);
        assert_eq!(reset_stats.failure_count, 0);
        assert_eq!(reset_stats.half_open_requests, 0);
        assert!(reset_stats.last_failure_time.is_none());
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_high_volume_requests() {
        let cb = create_default_circuit_breaker();

        let start = std::time::Instant::now();
        for _ in 0..10_000 {
            let _allowed = cb.should_allow_request();
        }
        let elapsed = start.elapsed();

        // Should complete quickly (less than 100ms for 10k requests)
        assert!(elapsed < Duration::from_millis(100));
    }

    #[test]
    fn test_high_volume_success_recording() {
        let cb = create_default_circuit_breaker();

        let start = std::time::Instant::now();
        for _ in 0..10_000 {
            cb.record_success();
        }
        let elapsed = start.elapsed();

        // Should complete quickly
        assert!(elapsed < Duration::from_millis(100));
        assert_eq!(cb.get_stats().success_count, 10_000);
    }

    #[test]
    fn test_memory_usage_stability() {
        let cb = create_default_circuit_breaker();

        // Generate lots of operations
        for i in 0..1000 {
            if i % 10 == 0 {
                cb.record_failure();
            } else {
                cb.record_success();
            }

            if i % 100 == 0 {
                cb.reset();
            }
        }

        // Should still be functional
        assert!(cb.should_allow_request());
        let _stats = cb.get_stats();
    }
}
