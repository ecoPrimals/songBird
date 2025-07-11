// Module imports
//! Circuit Breaker Implementation
//!
//! Provides fault tolerance by monitoring service failures and preventing
//! requests to failing services until they recover.

use parking_lot::RwLock as ParkingRwLock;
use serde::{Serialize, Deserialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Failing - reject all requests
    HalfOpen,  // Testing - allow limited requests
}
/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before opening circuit
    pub failure_threshold: u32,
    /// Success threshold to close circuit from half-open
    pub success_threshold: u32,
    /// Timeout before moving from open to half-open
    pub timeout: Duration,
    /// Window size for tracking failures
    pub window_size: Duration,
    /// Maximum number of requests allowed in half-open state
    pub half_open_max_requests: u32,
}
impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            window_size: Duration::from_secs(60),
            half_open_max_requests: 3,
        }
    }
}
/// Circuit breaker implementation
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<ParkingRwLock<CircuitState>>,
    failure_count: Arc<AtomicU64>,
    success_count: Arc<AtomicU64>,
    last_failure_time: Arc<ParkingRwLock<Option<Instant>>>,
    half_open_requests: Arc<AtomicU64>,
}
impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(ParkingRwLock::new(CircuitState::Closed)),
            failure_count: Arc::new(AtomicU64::new(0)),
            success_count: Arc::new(AtomicU64::new(0)),
            last_failure_time: Arc::new(ParkingRwLock::new(None)),
            half_open_requests: Arc::new(AtomicU64::new(0)),
        }
    }
    /// Check if request should be allowed through the circuit breaker
    pub fn should_allow_request(&self) -> bool {
        let state = *self.state.read();
        
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has passed
                if let Some(last_failure) = *self.last_failure_time.read() {
                    if last_failure.elapsed() >= self.config.timeout {
                        // Move to half-open state
                        *self.state.write() = CircuitState::HalfOpen;
                        self.half_open_requests.store(0, Ordering::Relaxed);
                        self.success_count.store(0, Ordering::Relaxed);
                        tracing::info!("Circuit breaker moved to HALF_OPEN state");
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                let current_requests = self.half_open_requests.load(Ordering::Relaxed);
                if current_requests < self.config.half_open_max_requests as u64 {
                    self.half_open_requests.fetch_add(1, Ordering::Relaxed);
                    true
                } else {
                    false
                }
            }
        }
    }
    /// Record a successful request
    pub fn record_success(&self) {
        let state = *self.state.read();
        
        match state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
                self.success_count.fetch_add(1, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                let success_count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if success_count >= self.config.success_threshold as u64 {
                    // Close the circuit
                    *self.state.write() = CircuitState::Closed;
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
    /// Record a failed request
    pub fn record_failure(&self) {
        *self.last_failure_time.write() = Some(Instant::now());
        let state = *self.state.read();
        
        match state {
            CircuitState::Closed => {
                let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if failure_count >= self.config.failure_threshold as u64 {
                    // Open the circuit
                    *self.state.write() = CircuitState::Open;
                    tracing::warn!("Circuit breaker moved to OPEN state after {} failures", failure_count);
                }
            }
            CircuitState::HalfOpen => {
                // Failed in half-open, go back to open
                *self.state.write() = CircuitState::Open;
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
        *self.state.read()
    }
    /// Get circuit breaker statistics
    pub fn get_stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.get_state(),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
            half_open_requests: self.half_open_requests.load(Ordering::Relaxed),
            last_failure_time: self.last_failure_time.read()
                .map(|instant| chrono::Utc::now() - chrono::Duration::from_std(instant.elapsed()).unwrap_or_default()),
        }
    }
    /// Reset circuit breaker state
    pub fn reset(&self) {
        *self.state.write() = CircuitState::Closed;
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        self.half_open_requests.store(0, Ordering::Relaxed);
        *self.last_failure_time.write() = None;
        tracing::info!("Circuit breaker reset to CLOSED state");
    }
}
/// Circuit breaker statistics
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: u64,
    pub success_count: u64,
    pub half_open_requests: u64,
    pub last_failure_time: Option<chrono::DateTime<chrono::Utc>>,
}

impl std::fmt::Debug for CircuitBreaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CircuitBreaker")
            .field("state", &self.get_state())
            .field("failure_count", &self.failure_count.load(Ordering::Relaxed))
            .field("success_count", &self.success_count.load(Ordering::Relaxed))
            .finish()
    }
} 
