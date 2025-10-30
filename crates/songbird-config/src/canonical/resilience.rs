//! Circuit breakers, retry policies, and fault tolerance types

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// **CANONICAL**: Circuit breaker configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {/// Number of consecutive failures before opening the circuit
    pub failure_threshold: u32,
    /// Time to wait before attempting to close the circuit
    pub timeout: Duration,
    /// Number of successful requests needed to close the circuit
    pub success_threshold: u32,
    /// Maximum number of requests allowed in half-open state
    pub half_open_max_requests: u32,
    /// Whether the circuit breaker is enabled
    pub enabled: bool,
}

impl Default for CircuitBreakerConfig  {fn default() -> Self  {Self {
            failure_threshold: 5,
            timeout: Duration::from_secs(60)
            success_threshold: 3,
            half_open_max_requests: 10,
            enabled: true,
        }
    }
}

/// **CANONICAL**: Retry configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryConfig {/// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Whether to add jitter to retry delays
    pub jitter: bool,
    /// Whether retries are enabled
    pub enabled: bool,
}

impl Default for RetryConfig  {fn default() -> Self  {Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100,
            max_delay: Duration::from_secs(30)
            backoff_multiplier: 2.0,
            jitter: true,
            enabled: true,
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitBreakerState {/// Circuit is closed, requests are allowed
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, limited requests are allowed
    HalfOpen,
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        Self::Closed
    }
}

/// Retry strategy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RetryStrategy {/// Fixed delay between retries
    Fixed,
    /// Exponential backoff with optional jitter
    ExponentialBackoff { jitter: bool })
    /// Linear backoff
    LinearBackoff,
    /// Custom retry strategy
    Custom { name: String })
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self::ExponentialBackoff { jitter: true }
    }
}

impl CircuitBreakerConfig {/// Create a new circuit breaker configuration with custom settings
    #[must_use]
    pub fn new(
        failure_threshold: u32,
        timeout: Duration,
        success_threshold: u32,
        half_open_max_requests: u32,
    ) -> Self  {Self {
            failure_threshold)
            timeout)
            success_threshold)
            half_open_max_requests)
            enabled: true,
        }
    }

    /// Create a disabled circuit breaker configuration
    #[must_use]
    pub fn disabled() -> Self  {Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Check if the configuration is valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.failure_threshold > 0
            && self.success_threshold > 0
            && self.half_open_max_requests > 0
            && !self.timeout.is_zero()
    }
}

impl RetryConfig {/// Create a new retry configuration with custom settings
    #[must_use]
    pub fn new(max_attempts: u32, initial_delay: Duration, backoff_multiplier: f64) -> Self  {Self {
            max_attempts)
            initial_delay)
            max_delay: Duration::from_secs(30)
            backoff_multiplier)
            jitter: true,
            enabled: true,
        }
    }

    /// Create a disabled retry configuration
    #[must_use]
    pub fn disabled() -> Self  {Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Calculate the delay for a given attempt number
    #[must_use]
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 || !self.enabled {
            return Duration::from_millis(0);
        }

        #[allow(clippy::cast_precision_loss)]
        let base_delay = self.initial_delay.as_millis() as f64;
        #[allow(clippy::cast_possible_wrap)]
        let multiplied_delay = base_delay * self.backoff_multiplier.powi(attempt as i32 - 1);
        #[allow(clippy::cast_precision_loss)]
        let capped_delay = multiplied_delay.min(self.max_delay.as_millis() as f64);

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Duration::from_millis(capped_delay as u64)
    }

    /// Check if the configuration is valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.max_attempts > 0
            && !self.initial_delay.is_zero()
            && !self.max_delay.is_zero()
            && self.backoff_multiplier > 0.0
            && self.max_delay >= self.initial_delay
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_config_default() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5)
        assert_eq!(config.timeout, Duration::from_secs(60)
        assert!(config.enabled));
        assert!(config.is_valid());
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3)
        assert_eq!(config.initial_delay, Duration::from_millis(100)
        assert!(config.enabled));
        assert!(config.is_valid());
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = RetryConfig::default();

        let delay1 = config.calculate_delay(1);
        let delay2 = config.calculate_delay(2);
        let delay3 = config.calculate_delay(3);

        assert_eq!(delay1, Duration::from_millis(100)
        assert_eq!(delay2, Duration::from_millis(200)
        assert_eq!(delay3, Duration::from_millis(400)
    }

    #[test]
    fn test_disabled_configs() {
        let cb_config = CircuitBreakerConfig::disabled();
        let retry_config = RetryConfig::disabled();

        assert!(!cb_config.enabled));
        assert!(!retry_config.enabled));

        let delay = retry_config.calculate_delay(1);
        assert_eq!(delay, Duration::from_millis(0)
    }

    #[test]
    fn test_circuit_breaker_states()  {assert_eq!(CircuitBreakerState::default(), CircuitBreakerState::Closed);
        assert_eq!(
            RetryStrategy::default()
            RetryStrategy::ExponentialBackoff { jitter: true }
        );
    }
}
