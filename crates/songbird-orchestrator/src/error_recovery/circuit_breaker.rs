// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Circuit Breaker Pattern
//!
//! Prevents cascading failures by:
//! - Tracking failure rates
//! - Opening circuit when threshold exceeded
//! - Half-open state for recovery testing
//! - Automatic reset after cooldown

use anyhow::Result;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::Instant;
use tracing::{debug, warn};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,

    /// Circuit is open, requests are rejected
    Open,

    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening
    pub failure_threshold: u32,

    /// Success threshold in half-open state before closing
    pub success_threshold: u32,

    /// How long to wait before trying half-open
    pub timeout: Duration,

    /// Window size for tracking failure rate
    pub window_size: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: songbird_types::defaults::timeouts::DEFAULT_CIRCUIT_BREAKER_TIMEOUT,
            window_size: songbird_types::defaults::timeouts::DEFAULT_CIRCUIT_BREAKER_TIMEOUT,
        }
    }
}

/// Circuit breaker
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerState>>,
}

#[derive(Debug)]
struct CircuitBreakerState {
    current_state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    opened_at: Option<Instant>,
}

impl CircuitBreaker {
    #[must_use]
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState {
                current_state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
                opened_at: None,
            })),
        }
    }

    /// Execute an operation through the circuit breaker
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn call<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        // Check if circuit is open
        {
            let mut state = self.state.write().await;

            match state.current_state {
                CircuitState::Open => {
                    // Check if timeout has passed
                    if let Some(opened_at) = state.opened_at {
                        if opened_at.elapsed() >= self.config.timeout {
                            debug!("Circuit breaker transitioning to half-open");
                            state.current_state = CircuitState::HalfOpen;
                            state.success_count = 0;
                        } else {
                            anyhow::bail!("Circuit breaker is open");
                        }
                    }
                }
                CircuitState::HalfOpen => {
                    debug!("Circuit breaker is half-open, allowing test request");
                }
                CircuitState::Closed => {
                    // Normal operation
                }
            }
        }

        // Execute operation
        match operation().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(err) => {
                self.on_failure().await;
                Err(err)
            }
        }
    }

    /// Record a successful operation
    async fn on_success(&self) {
        let mut state = self.state.write().await;

        match state.current_state {
            CircuitState::Closed => {
                // Reset failure count on success
                state.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                state.success_count += 1;

                if state.success_count >= self.config.success_threshold {
                    debug!("Circuit breaker closing after {} successes", state.success_count);
                    state.current_state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                    state.opened_at = None;
                }
            }
            CircuitState::Open => {
                // Should not happen, but handle gracefully
            }
        }
    }

    /// Record a failed operation
    async fn on_failure(&self) {
        let mut state = self.state.write().await;
        let now = Instant::now();

        state.failure_count += 1;
        state.last_failure_time = Some(now);

        match state.current_state {
            CircuitState::Closed => {
                if state.failure_count >= self.config.failure_threshold {
                    warn!("Circuit breaker opening after {} failures", state.failure_count);
                    state.current_state = CircuitState::Open;
                    state.opened_at = Some(now);
                }
            }
            CircuitState::HalfOpen => {
                warn!("Circuit breaker reopening after failure in half-open state");
                state.current_state = CircuitState::Open;
                state.opened_at = Some(now);
                state.success_count = 0;
            }
            CircuitState::Open => {
                // Already open
            }
        }
    }

    /// Get current circuit state
    pub async fn get_state(&self) -> CircuitState {
        let state = self.state.read().await;
        state.current_state
    }

    /// Reset circuit breaker to closed state
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        state.current_state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
        state.last_failure_time = None;
        state.opened_at = None;
        debug!("Circuit breaker manually reset");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_closed_on_success() {
        let config = CircuitBreakerConfig::default();
        let cb = CircuitBreaker::new(config);

        let result = cb.call(|| async { Ok::<_, anyhow::Error>(42) }).await;

        assert!(result.is_ok());
        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        // Fail 3 times
        for _ in 0..3 {
            let _: Result<()> = cb.call(|| async { anyhow::bail!("Error") }).await;
        }

        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Next call should fail immediately
        let result = cb.call(|| async { Ok::<_, anyhow::Error>(42) }).await;
        assert!(result.is_err());
    }

    #[tokio::test(start_paused = true)]
    async fn test_circuit_half_open_after_timeout() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_millis(100),
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        for _ in 0..2 {
            let _: Result<()> = cb.call(|| async { anyhow::bail!("Error") }).await;
        }

        assert_eq!(cb.get_state().await, CircuitState::Open);

        tokio::time::advance(Duration::from_millis(150)).await;

        let _: Result<i32> = cb.call(|| async { Ok::<_, anyhow::Error>(42) }).await;

        let state = cb.get_state().await;
        assert!(state == CircuitState::HalfOpen || state == CircuitState::Closed);
    }

    #[tokio::test(start_paused = true)]
    async fn test_circuit_closes_after_successes_in_half_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(10),
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        for _ in 0..2 {
            let _: Result<()> = cb.call(|| async { anyhow::bail!("Error") }).await;
        }

        tokio::time::advance(Duration::from_millis(20)).await;

        let _: Result<i32> = cb.call(|| async { Ok::<_, anyhow::Error>(42) }).await;
        let _: Result<i32> = cb.call(|| async { Ok::<_, anyhow::Error>(42) }).await;

        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_manual_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        // Open circuit
        for _ in 0..2 {
            let _: Result<()> = cb.call(|| async { anyhow::bail!("Error") }).await;
        }

        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Manual reset
        cb.reset().await;

        assert_eq!(cb.get_state().await, CircuitState::Closed);
    }
}
