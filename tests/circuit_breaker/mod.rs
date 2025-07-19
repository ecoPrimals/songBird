//! Comprehensive tests for circuit breaker implementation
//!
//! This test suite provides extensive coverage for the circuit breaker pattern,
//! including state transitions, failure/success thresholds, timeout behavior,
//! concurrent access, and edge cases.
//!
//! ## Refactored Test Architecture
//!
//! The circuit breaker tests are organized into focused modules:
//! - `basic_tests` - Basic functionality and helper functions
//! - `state_tests` - State transition testing (Closed → Open → Half-Open → Closed)
//! - `concurrent_tests` - Concurrency and performance testing
//! - `config_tests` - Configuration validation and edge cases

pub mod basic_tests;
pub mod state_tests;
pub mod concurrent_tests;
pub mod config_tests;

// Re-export commonly used test helpers
pub use basic_tests::{create_test_circuit_breaker, create_default_circuit_breaker, create_fast_circuit_breaker};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::time::Duration;
    use songbird_network::communication::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};

    #[tokio::test]
    async fn test_full_circuit_breaker_lifecycle() {
        let circuit_breaker = create_fast_circuit_breaker();

        // Start closed
        assert!(matches!(
            circuit_breaker.get_state().await,
            CircuitState::Closed
        ));

        // Force open
        for _ in 0..3 {
            circuit_breaker.record_failure().await;
        }
        assert!(matches!(
            circuit_breaker.get_state().await,
            CircuitState::Open
        ));

        // Wait for half-open
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(circuit_breaker.should_allow_request().await);

        // Return to closed
        circuit_breaker.record_success().await;
        circuit_breaker.record_success().await;
        assert!(matches!(
            circuit_breaker.get_state().await,
            CircuitState::Closed
        ));
    }

    #[tokio::test]
    async fn test_metrics_accuracy() {
        let circuit_breaker = create_default_circuit_breaker();

        // Record known pattern
        circuit_breaker.record_success().await;
        circuit_breaker.record_success().await;
        circuit_breaker.record_failure().await;
        circuit_breaker.record_success().await;

        let metrics = circuit_breaker.get_metrics().await;
        assert_eq!(metrics.total_requests, 4);
        assert_eq!(metrics.successful_requests, 3);
        assert_eq!(metrics.failed_requests, 1);
    }
} 