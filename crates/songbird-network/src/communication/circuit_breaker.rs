// Module imports
//! Circuit Breaker Implementation
//!
//! Provides fault tolerance by monitoring service failures and preventing
//! requests to failing services until they recover.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, requests are allowed
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is half-open, limited requests are allowed
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open the circuit
    pub failure_threshold: u32,
    /// Success threshold to close the circuit from half-open
    pub success_threshold: u32,
    /// Timeout duration before trying to close the circuit
    pub timeout_duration: std::time::Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout_duration: std::time::Duration::from_secs(60),
        }
    }
}

/// Circuit breaker implementation
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    failure_count: AtomicU64,
    success_count: AtomicU64,
    half_open_requests: AtomicU64,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            half_open_requests: AtomicU64::new(0),
            last_failure_time: Arc::new(RwLock::new(None)),
        }
    }

    pub fn should_allow_request(&self) -> bool {
        let state = *self.state.blocking_read();
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last_failure) = *self.last_failure_time.blocking_read() {
                    if last_failure.elapsed() > self.config.timeout_duration {
                        // Move to half-open state
                        *self.state.blocking_write() = CircuitState::HalfOpen;
                        tracing::info!("Circuit breaker moved to HALF_OPEN state");
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                let current_requests = self.half_open_requests.load(Ordering::Relaxed);
                current_requests < self.config.success_threshold as u64
            }
        }
    }

    pub fn record_success(&self) {
        let state = *self.state.blocking_read();
        match state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
                let success_count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if success_count >= self.config.success_threshold as u64 {
                    // Close the circuit
                    *self.state.blocking_write() = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                    tracing::info!("Circuit breaker moved to CLOSED state");
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
                tracing::warn!("Received success while circuit is OPEN");
            }
        }
    }

    pub fn record_failure(&self) {
        *self.last_failure_time.blocking_write() = Some(Instant::now());
        let state = *self.state.blocking_read();
        match state {
            CircuitState::Closed => {
                let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if failure_count >= self.config.failure_threshold as u64 {
                    // Open the circuit
                    *self.state.blocking_write() = CircuitState::Open;
                    tracing::warn!(
                        "Circuit breaker moved to OPEN state after {} failures",
                        failure_count
                    );
                }
            }
            CircuitState::HalfOpen => {
                // Failed in half-open, go back to open
                *self.state.blocking_write() = CircuitState::Open;
                self.failure_count.fetch_add(1, Ordering::Relaxed);
                tracing::warn!("Circuit breaker moved back to OPEN state from HALF_OPEN");
            }
            CircuitState::Open => {
                // Already open, just record the failure
                self.failure_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Get current circuit breaker state
    pub fn get_state(&self) -> CircuitState {
        *self.state.blocking_read()
    }

    /// Get circuit breaker statistics
    pub fn get_stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.get_state(),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
            half_open_requests: self.half_open_requests.load(Ordering::Relaxed),
            last_failure_time: self.last_failure_time.blocking_read().map(|instant| {
                chrono::Utc::now()
                    - chrono::Duration::from_std(instant.elapsed()).unwrap_or_default()
            }),
        }
    }

    /// Reset circuit breaker state
    pub fn reset(&self) {
        *self.state.blocking_write() = CircuitState::Closed;
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        self.half_open_requests.store(0, Ordering::Relaxed);
        *self.last_failure_time.blocking_write() = None;
        tracing::info!("Circuit breaker reset to CLOSED state");
    }
}

/// Circuit breaker statistics
#[derive(Debug)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: u64,
    pub success_count: u64,
    pub half_open_requests: u64,
    pub last_failure_time: Option<chrono::DateTime<chrono::Utc>>,
}
