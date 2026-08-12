// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Circuit Breaker pattern for fault tolerance
//!
//! Implements the circuit breaker pattern to prevent cascading failures
//! and provide graceful degradation under failure conditions.
//!
//! ## Deep Debt Evolution Principle
//!
//! **Before (No Protection)**:
//! ```ignore
//! // Repeated calls to failing service (cascading failure!)
//! for _ in 0..100 {
//!     let result = call_failing_service().await; // All fail, wasting resources
//! }
//! ```
//!
//! **After (Circuit Breaker)**:
//! ```ignore
//! let breaker = CircuitBreaker::new(5, Duration::from_secs(30));
//! for _ in 0..100 {
//!     match breaker.call(|| call_service()).await {
//!         Ok(result) => { /* Success */ },
//!         Err(CircuitBreakerError::Open) => {
//!             // Fail fast, don't waste resources
//!             use_fallback().await;
//!         },
//!         Err(e) => { /* Other error */ },
//!     }
//! }
//! ```
//!
//! ## States
//!
//! ```text
//! ┌─────────┐ failures < threshold  ┌──────────┐
//! │ Closed  │◄────────────────────── │ Half-Open│
//! │         │                        │          │
//! └────┬────┘                        └─────▲────┘
//!      │                                   │
//!      │ failures >= threshold             │
//!      │                                   │
//!      ▼                                   │
//! ┌─────────┐ timeout expires              │
//! │  Open   ├────────────────────────────► │
//! │         │                               │
//! └─────────┘                               │
//! ```
//!
//! - **Closed**: Normal operation, requests pass through
//! - **Open**: Too many failures, requests fail immediately
//! - **Half-Open**: Testing if service recovered
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_orchestrator::resilience::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let breaker = CircuitBreaker::builder()
//!         .failure_threshold(5)
//!         .timeout(Duration::from_secs(30))
//!         .build();
//!
//!     let result = breaker.call(|| async {
//!         Ok::<_, std::io::Error>(42)
//!     }).await?;
//!
//!     println!("Result: {}", result);
//!     Ok(())
//! }
//! ```

use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;

use songbird_types::{SongbirdError, SongbirdResult};
use std::sync::RwLock;
use tracing::{debug, info, warn};

/// Circuit breaker error types
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open (too many failures)")]
    Open,

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Operation timeout after {0:?}")]
    Timeout(Duration),
}

/// Result type for circuit breaker operations
pub type CircuitBreakerResult<T> = Result<T, CircuitBreakerError>;

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests pass through normally
    Closed {
        /// Number of consecutive failures
        failures: usize,
    },

    /// Circuit is open, requests fail immediately
    Open {
        /// When the circuit was opened
        opened_at: Instant,
    },

    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Configuration for circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: usize,

    /// How long to wait before transitioning from Open to Half-Open
    pub timeout: Duration,

    /// Success threshold in Half-Open state before closing
    pub success_threshold: usize,

    /// Optional timeout for individual operations
    pub operation_timeout: Option<Duration>,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            timeout: songbird_types::defaults::timeouts::DEFAULT_CIRCUIT_BREAKER_TIMEOUT,
            success_threshold: 2,
            operation_timeout: None,
        }
    }
}

impl CircuitBreakerConfig {
    #[must_use]
    pub fn builder() -> CircuitBreakerConfigBuilder {
        CircuitBreakerConfigBuilder::default()
    }

    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn validate(&self) -> SongbirdResult<()> {
        if self.failure_threshold == 0 {
            return Err(SongbirdError::configuration("failure_threshold must be greater than 0"));
        }
        if self.success_threshold == 0 {
            return Err(SongbirdError::configuration("success_threshold must be greater than 0"));
        }
        if self.timeout.is_zero() {
            return Err(SongbirdError::configuration("timeout cannot be zero"));
        }
        Ok(())
    }
}

/// Builder for circuit breaker configuration
#[derive(Debug, Default)]
pub struct CircuitBreakerConfigBuilder {
    failure_threshold: Option<usize>,
    timeout: Option<Duration>,
    success_threshold: Option<usize>,
    operation_timeout: Option<Duration>,
}

impl CircuitBreakerConfigBuilder {
    #[must_use]
    pub const fn failure_threshold(mut self, threshold: usize) -> Self {
        self.failure_threshold = Some(threshold);
        self
    }

    #[must_use]
    pub const fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    #[must_use]
    pub const fn success_threshold(mut self, threshold: usize) -> Self {
        self.success_threshold = Some(threshold);
        self
    }

    #[must_use]
    pub const fn operation_timeout(mut self, duration: Duration) -> Self {
        self.operation_timeout = Some(duration);
        self
    }

    #[must_use]
    pub fn build(self) -> CircuitBreakerConfig {
        let default = CircuitBreakerConfig::default();
        CircuitBreakerConfig {
            failure_threshold: self.failure_threshold.unwrap_or(default.failure_threshold),
            timeout: self.timeout.unwrap_or(default.timeout),
            success_threshold: self.success_threshold.unwrap_or(default.success_threshold),
            operation_timeout: self.operation_timeout.or(default.operation_timeout),
        }
    }
}

/// Circuit breaker for fault tolerance
pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitState>>,
    config: CircuitBreakerConfig,
    successes_in_half_open: Arc<RwLock<usize>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with configuration
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn new(config: CircuitBreakerConfig) -> SongbirdResult<Self> {
        config.validate()?;

        Ok(Self {
            state: Arc::new(RwLock::new(CircuitState::Closed {
                failures: 0,
            })),
            config,
            successes_in_half_open: Arc::new(RwLock::new(0)),
        })
    }

    /// Create a circuit breaker with builder pattern
    #[must_use]
    pub fn builder() -> CircuitBreakerBuilder {
        CircuitBreakerBuilder::default()
    }

    /// Get the current state of the circuit breaker
    pub async fn state(&self) -> CircuitState {
        self.state.read().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }

    /// Execute an async operation with circuit breaker protection
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use songbird_orchestrator::resilience::circuit_breaker::CircuitBreaker;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let breaker = CircuitBreaker::builder().build().unwrap();
    ///
    /// let result = breaker.call(|| async {
    ///     // Your async operation
    ///     Ok::<_, std::io::Error>(42)
    /// }).await;
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn call<F, Fut, T, E>(&self, operation: F) -> CircuitBreakerResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::error::Error + Send + Sync + 'static,
    {
        // Check current state and transition if needed
        let current_state = {
            let mut state = self.state.write().unwrap_or_else(std::sync::PoisonError::into_inner);
            match *state {
                CircuitState::Open {
                    opened_at,
                } => {
                    // Check if timeout has expired
                    if opened_at.elapsed() >= self.config.timeout {
                        debug!("Circuit breaker transitioning from Open to Half-Open");
                        *state = CircuitState::HalfOpen;
                        *self
                            .successes_in_half_open
                            .write()
                            .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
                        CircuitState::HalfOpen
                    } else {
                        // Still open, reject request
                        return Err(CircuitBreakerError::Open);
                    }
                }
                ref s => s.clone(),
            }
        };

        // Execute operation
        let result = if let Some(timeout) = self.config.operation_timeout {
            match tokio::time::timeout(timeout, operation()).await {
                Ok(r) => r.map_err(|e| CircuitBreakerError::OperationFailed(e.to_string())),
                Err(_) => Err(CircuitBreakerError::Timeout(timeout)),
            }
        } else {
            operation().await.map_err(|e| CircuitBreakerError::OperationFailed(e.to_string()))
        };

        // Update state based on result
        match result {
            Ok(value) => {
                self.on_success(current_state).await;
                Ok(value)
            }
            Err(e) => {
                self.on_failure(current_state).await;
                Err(e)
            }
        }
    }

    /// Handle successful operation
    async fn on_success(&self, current_state: CircuitState) {
        match current_state {
            CircuitState::Closed {
                ..
            } => {
                // Reset failure count on success
                *self.state.write().unwrap_or_else(std::sync::PoisonError::into_inner) =
                    CircuitState::Closed {
                        failures: 0,
                    };
            }
            CircuitState::HalfOpen => {
                // Increment success count
                let mut successes = self
                    .successes_in_half_open
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                *successes += 1;

                // If enough successes, close the circuit
                if *successes >= self.config.success_threshold {
                    info!("Circuit breaker transitioning from Half-Open to Closed (recovered)");
                    *self.state.write().unwrap_or_else(std::sync::PoisonError::into_inner) =
                        CircuitState::Closed {
                            failures: 0,
                        };
                }
            }
            CircuitState::Open {
                ..
            } => {
                // Should not happen, but handle gracefully
                warn!("Received success in Open state (unexpected)");
            }
        }
    }

    /// Handle failed operation
    async fn on_failure(&self, current_state: CircuitState) {
        match current_state {
            CircuitState::Closed {
                failures,
            } => {
                let new_failures = failures + 1;
                if new_failures >= self.config.failure_threshold {
                    // Open the circuit
                    warn!(
                        "Circuit breaker opening (failures: {} >= threshold: {})",
                        new_failures, self.config.failure_threshold
                    );
                    *self.state.write().unwrap_or_else(std::sync::PoisonError::into_inner) =
                        CircuitState::Open {
                            opened_at: Instant::now(),
                        };
                } else {
                    // Increment failure count
                    *self.state.write().unwrap_or_else(std::sync::PoisonError::into_inner) =
                        CircuitState::Closed {
                            failures: new_failures,
                        };
                }
            }
            CircuitState::HalfOpen => {
                // Failure in half-open state, reopen circuit
                warn!("Circuit breaker reopening (failed in Half-Open state)");
                *self.state.write().unwrap_or_else(std::sync::PoisonError::into_inner) =
                    CircuitState::Open {
                        opened_at: Instant::now(),
                    };
            }
            CircuitState::Open {
                ..
            } => {
                // Already open, no state change needed
            }
        }
    }

    /// Manually reset the circuit breaker to closed state
    ///
    /// Use with caution - typically you want automatic recovery via Half-Open state.
    pub async fn reset(&self) {
        info!("Circuit breaker manually reset to Closed state");
        *self.state.write().unwrap_or_else(std::sync::PoisonError::into_inner) =
            CircuitState::Closed {
                failures: 0,
            };
        *self.successes_in_half_open.write().unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
    }

    /// Get circuit breaker statistics
    pub async fn stats(&self) -> CircuitBreakerStats {
        let state = self.state.read().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
        let current_failures = match &state {
            CircuitState::Closed {
                failures,
            } => *failures,
            _ => 0,
        };
        CircuitBreakerStats {
            state,
            failure_threshold: self.config.failure_threshold,
            success_threshold: self.config.success_threshold,
            timeout: self.config.timeout,
            current_failures,
            current_successes: *self
                .successes_in_half_open
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_threshold: usize,
    pub success_threshold: usize,
    pub timeout: Duration,
    pub current_failures: usize,
    pub current_successes: usize,
}

/// Builder for circuit breaker
#[derive(Debug, Default)]
pub struct CircuitBreakerBuilder {
    config: CircuitBreakerConfigBuilder,
}

impl CircuitBreakerBuilder {
    #[must_use]
    pub const fn failure_threshold(mut self, threshold: usize) -> Self {
        self.config = self.config.failure_threshold(threshold);
        self
    }

    #[must_use]
    pub const fn timeout(mut self, duration: Duration) -> Self {
        self.config = self.config.timeout(duration);
        self
    }

    #[must_use]
    pub const fn success_threshold(mut self, threshold: usize) -> Self {
        self.config = self.config.success_threshold(threshold);
        self
    }

    #[must_use]
    pub const fn operation_timeout(mut self, duration: Duration) -> Self {
        self.config = self.config.operation_timeout(duration);
        self
    }

    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub fn build(self) -> SongbirdResult<CircuitBreaker> {
        let config = self.config.build();
        CircuitBreaker::new(config)
    }
}

#[cfg(test)]
#[path = "circuit_breaker_tests.rs"]
mod tests;
