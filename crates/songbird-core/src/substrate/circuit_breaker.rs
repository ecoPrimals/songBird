//! Circuit breaker pattern for substrate resilience

use std::time::{Duration, Instant};

/// Circuit breaker for substrate resilience
#[derive(Debug)]
pub struct CircuitBreaker {
    pub failure_count: u32,
    pub failure_threshold: u32,
    pub last_failure: Option<Instant>,
    pub timeout: Duration,
    pub state: CircuitState,
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Default)]
pub enum CircuitState {
    #[default]
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    /// Create new circuit breaker
    pub fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_count: 0,
            failure_threshold,
            last_failure: None,
            timeout,
            state: CircuitState::Closed,
        }
    }

    /// Check if request is allowed
    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure {
                    if last_failure.elapsed() >= self.timeout {
                        self.state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    self.state = CircuitState::Closed;
                    true
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record successful request
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
        self.last_failure = None;
    }

    /// Record failed request
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }

    /// Get circuit breaker state
    pub fn get_state(&self) -> &CircuitState {
        &self.state
    }

    /// Reset circuit breaker to closed state
    pub fn reset(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
        self.last_failure = None;
    }

    /// Check if circuit breaker is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.state, CircuitState::Closed)
    }

    /// Get failure rate as percentage
    pub fn failure_rate(&self) -> f64 {
        if self.failure_threshold == 0 {
            0.0
        } else {
            (self.failure_count as f64 / self.failure_threshold as f64) * 100.0
        }
    }

    /// Get time until circuit can be retried (if open)
    pub fn retry_in(&self) -> Option<Duration> {
        if let (CircuitState::Open, Some(last_failure)) = (&self.state, self.last_failure) {
            let elapsed = last_failure.elapsed();
            if elapsed < self.timeout {
                Some(self.timeout - elapsed)
            } else {
                None // Can retry now
            }
        } else {
            None
        }
    }
}
