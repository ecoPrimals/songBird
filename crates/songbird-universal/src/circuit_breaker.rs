//! Circuit Breaker Pattern for Adapter Resilience
//!
//! Prevents cascading failures by temporarily disabling failed endpoints.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Circuit is closed (normal operation)
    Closed,
    /// Circuit is open (failures detected, requests blocked)
    Open,
    /// Circuit is half-open (testing if service recovered)
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Duration to wait before entering half-open state
    pub timeout: Duration,
    /// Number of successful requests needed to close circuit from half-open
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout: Duration::from_secs(60),
            success_threshold: 2,
        }
    }
}

/// Circuit breaker for protecting against cascading failures
pub struct CircuitBreaker {
    /// Current state
    state: Arc<RwLock<CircuitState>>,
    /// Configuration
    config: CircuitBreakerConfig,
    /// Failure count in current state
    failure_count: Arc<RwLock<u32>>,
    /// Success count in half-open state
    success_count: Arc<RwLock<u32>>,
    /// Time when circuit was last opened
    last_failure_time: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default config
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(CircuitBreakerConfig::default())
    }

    /// Create a circuit breaker with custom config
    #[must_use]
    pub fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            config,
            failure_count: Arc::new(RwLock::new(0)),
            success_count: Arc::new(RwLock::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Check if a request is allowed
    ///
    /// Returns `true` if the circuit allows the request, `false` if it's open.
    pub async fn is_request_allowed(&self) -> bool {
        let state = *self.state.read().await;

        match state {
            CircuitState::Closed | CircuitState::HalfOpen => true,
            CircuitState::Open => {
                // Check if timeout has elapsed
                let last_failure = self.last_failure_time.read().await;
                if let Some(last) = *last_failure {
                    if last.elapsed() >= self.config.timeout {
                        // Transition to half-open
                        drop(last_failure);
                        self.transition_to_half_open().await;
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Record a successful request
    pub async fn record_success(&self) {
        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => {
                // Reset failure count
                *self.failure_count.write().await = 0;
            }
            CircuitState::HalfOpen => {
                // Increment success count
                let mut success = self.success_count.write().await;
                *success += 1;

                // Check if we should close the circuit
                if *success >= self.config.success_threshold {
                    drop(success);
                    self.transition_to_closed().await;
                }
            }
            CircuitState::Open => {
                // Should not happen, but reset if it does
                self.transition_to_closed().await;
            }
        }
    }

    /// Record a failed request
    pub async fn record_failure(&self) {
        let state = *self.state.read().await;

        match state {
            CircuitState::Closed => {
                // Increment failure count
                let mut failures = self.failure_count.write().await;
                *failures += 1;

                // Check if we should open the circuit
                if *failures >= self.config.failure_threshold {
                    drop(failures);
                    self.transition_to_open().await;
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state reopens the circuit
                self.transition_to_open().await;
            }
            CircuitState::Open => {
                // Update last failure time
                *self.last_failure_time.write().await = Some(Instant::now());
            }
        }
    }

    /// Get current state
    pub async fn get_state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Transition to open state
    async fn transition_to_open(&self) {
        *self.state.write().await = CircuitState::Open;
        *self.last_failure_time.write().await = Some(Instant::now());
        *self.failure_count.write().await = 0;
        *self.success_count.write().await = 0;
    }

    /// Transition to half-open state
    async fn transition_to_half_open(&self) {
        *self.state.write().await = CircuitState::HalfOpen;
        *self.success_count.write().await = 0;
    }

    /// Transition to closed state
    async fn transition_to_closed(&self) {
        *self.state.write().await = CircuitState::Closed;
        *self.failure_count.write().await = 0;
        *self.success_count.write().await = 0;
    }

    /// Reset the circuit breaker to closed state
    pub async fn reset(&self) {
        self.transition_to_closed().await;
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::new();
        assert_eq!(cb.get_state().await, CircuitState::Closed);
        assert!(cb.is_request_allowed().await);
    }

    #[tokio::test]
    async fn test_circuit_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config(config);

        // Record failures
        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        cb.record_failure().await; // Should open
        assert_eq!(cb.get_state().await, CircuitState::Open);
        assert!(!cb.is_request_allowed().await);
    }

    #[tokio::test]
    async fn test_circuit_half_open_after_timeout() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            timeout: Duration::from_millis(100),
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config(config);

        // Open the circuit
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should transition to half-open
        assert!(cb.is_request_allowed().await);
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);
    }

    #[tokio::test]
    async fn test_circuit_closes_after_successes() {
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
        assert!(cb.is_request_allowed().await);

        // Record successes
        cb.record_success().await;
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        cb.record_success().await; // Should close
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_half_open_reopens_on_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            timeout: Duration::from_millis(50),
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config(config);

        // Open the circuit
        cb.record_failure().await;

        // Transition to half-open
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(cb.is_request_allowed().await);
        assert_eq!(cb.get_state().await, CircuitState::HalfOpen);

        // Failure in half-open should reopen
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_success_in_closed_resets_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let cb = CircuitBreaker::with_config(config);

        // Record some failures
        cb.record_failure().await;
        cb.record_failure().await;

        // Success should reset failure count
        cb.record_success().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        // These failures shouldn't immediately open
        cb.record_failure().await;
        cb.record_failure().await;
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_reset() {
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
}
