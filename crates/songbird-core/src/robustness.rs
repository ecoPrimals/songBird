//! Robustness Module
//!
//! Provides circuit breakers, retry mechanisms, and fault tolerance

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use songbird_errors::{Result, SongbirdError};

/// Circuit breaker for fault tolerance
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerState>>,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout: Duration,
    pub max_failures: u32,
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            max_failures: 10,
            half_open_max_calls: 3,
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
    pub window_size: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 100,
            burst_size: 10,
            window_size: Duration::from_secs(1),
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone)]
enum CircuitBreakerState {
    Closed {
        failure_count: u32,
    },
    Open {
        opened_at: Instant,
    },
    HalfOpen {
        success_count: u32,
        failure_count: u32,
    },
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed {
                failure_count: 0,
            })),
        }
    }

    /// Execute a function with circuit breaker protection
    pub async fn call<F, T, E>(&self, func: F) -> Result<T>
    where
        F: std::future::Future<Output = std::result::Result<T, E>>,
        E: std::fmt::Display,
    {
        // Check if circuit breaker allows the call
        if !self.can_execute().await {
            return Err(SongbirdError::CircuitBreakerOpen {
                service: "unknown".to_string(),
                message: "Circuit breaker is open".to_string(),
                failure_count: Some(self.get_failure_count().await),
                suggestion: Some(
                    "Wait for the circuit breaker to reset or check service health".to_string(),
                ),
            });
        }

        // Execute the function
        match func.await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(e) => {
                self.record_failure().await;
                Err(SongbirdError::CircuitBreakerFailure {
                    service: "unknown".to_string(),
                    message: format!("Circuit breaker protected call failed: {e}"),
                    suggestion: Some(
                        "Check service health and consider adjusting circuit breaker thresholds"
                            .to_string(),
                    ),
                })
            }
        }
    }

    /// Check if the circuit breaker allows execution
    async fn can_execute(&self) -> bool {
        let state = self.state.read().await;
        match &*state {
            CircuitBreakerState::Closed { .. } => true,
            CircuitBreakerState::Open { opened_at } => {
                // Check if timeout has passed
                opened_at.elapsed() >= self.config.timeout
            }
            CircuitBreakerState::HalfOpen { .. } => true,
        }
    }

    /// Get the current failure count
    async fn get_failure_count(&self) -> u32 {
        let state = self.state.read().await;
        match &*state {
            CircuitBreakerState::Closed { failure_count } => *failure_count,
            CircuitBreakerState::Open { .. } => self.config.failure_threshold,
            CircuitBreakerState::HalfOpen { failure_count, .. } => *failure_count,
        }
    }

    /// Record a successful execution
    async fn record_success(&self) {
        let mut state = self.state.write().await;
        match &*state {
            CircuitBreakerState::Closed { .. } => {
                // Reset failure count on success
                *state = CircuitBreakerState::Closed { failure_count: 0 };
            }
            CircuitBreakerState::HalfOpen { success_count, .. } => {
                let new_success_count = success_count + 1;
                if new_success_count >= self.config.success_threshold {
                    // Close the circuit breaker
                    *state = CircuitBreakerState::Closed { failure_count: 0 };
                } else {
                    *state = CircuitBreakerState::HalfOpen {
                        success_count: new_success_count,
                        failure_count: 0,
                    };
                }
            }
            CircuitBreakerState::Open { .. } => {
                // Transition to half-open
                *state = CircuitBreakerState::HalfOpen {
                    success_count: 1,
                    failure_count: 0,
                };
            }
        }
    }

    /// Record a failed execution
    async fn record_failure(&self) {
        let mut state = self.state.write().await;
        match &*state {
            CircuitBreakerState::Closed { failure_count } => {
                let new_failure_count = failure_count + 1;
                if new_failure_count >= self.config.failure_threshold {
                    // Open the circuit breaker
                    *state = CircuitBreakerState::Open {
                        opened_at: Instant::now(),
                    };
                } else {
                    *state = CircuitBreakerState::Closed {
                        failure_count: new_failure_count,
                    };
                }
            }
            CircuitBreakerState::HalfOpen { failure_count, .. } => {
                let new_failure_count = failure_count + 1;
                if new_failure_count >= self.config.failure_threshold {
                    // Open the circuit breaker
                    *state = CircuitBreakerState::Open {
                        opened_at: Instant::now(),
                    };
                } else {
                    *state = CircuitBreakerState::HalfOpen {
                        success_count: 0,
                        failure_count: new_failure_count,
                    };
                }
            }
            CircuitBreakerState::Open { .. } => {
                // Already open, do nothing
            }
        }
    }

    /// Get current state information
    pub async fn get_state_info(&self) -> String {
        let state = self.state.read().await;
        match &*state {
            CircuitBreakerState::Closed { failure_count } => {
                format!("Closed (failures: {failure_count})")
            }
            CircuitBreakerState::Open { opened_at } => {
                format!(
                    "Open (opened {} seconds ago)",
                    opened_at.elapsed().as_secs()
                )
            }
            CircuitBreakerState::HalfOpen {
                success_count,
                failure_count,
            } => {
                format!("Half-Open (successes: {success_count}, failures: {failure_count})")
            }
        }
    }
}

/// Retry mechanism with exponential backoff
pub struct RetryMechanism {
    config: RetryConfig,
}

impl RetryMechanism {
    /// Create a new retry mechanism
    #[must_use]
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute a function with retry logic
    ///
    /// # Errors
    ///
    /// Returns an error if the retry operation fails after all attempts or
    /// if the provided function consistently returns errors.
    pub async fn retry<F, T, E>(&self, mut func: F) -> Result<T>
    where
        F: FnMut() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = std::result::Result<T, E>> + Send>,
        >,
        E: std::fmt::Display,
    {
        let mut attempt = 0;
        let mut delay = self.config.initial_delay;

        loop {
            attempt += 1;

            match func().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt >= self.config.max_attempts {
                        return Err(SongbirdError::RetryExhausted {
                            attempts: attempt,
                            message: e.to_string(),
                            duration: Some(format!("{}ms", self.config.initial_delay.as_millis())),
                            suggestion: Some(
                                "Increase retry attempts or check service availability".to_string(),
                            ),
                        });
                    }

                    // Calculate next delay with exponential backoff
                    if attempt > 1 {
                        let next_delay = Duration::from_millis(
                            (delay
                                .as_millis()
                                .saturating_mul((self.config.backoff_multiplier * 1000.0) as u128)
                                / 1000)
                                .min(u128::from(u64::MAX)) as u64,
                        );

                        // Add jitter (up to 25% of the delay)
                        let jitter_ms = fastrand::u64(
                            0..=u64::try_from(next_delay.as_millis()).unwrap_or(u64::MAX) / 4,
                        );
                        let delay_with_jitter = Duration::from_millis(
                            u64::try_from(next_delay.as_millis())
                                .unwrap_or(u64::MAX)
                                .saturating_add(jitter_ms),
                        );

                        // Cap at max delay
                        if delay_with_jitter > self.config.max_delay {
                            delay = self.config.max_delay;
                        } else {
                            delay = delay_with_jitter;
                        }
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }
    }
}

/// Rate limiter using token bucket algorithm
pub struct RateLimiter {
    config: RateLimitConfig,
    tokens: Arc<RwLock<f64>>,
    last_refill: Arc<RwLock<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    #[must_use]
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(f64::from(config.burst_size))),
            last_refill: Arc::new(RwLock::new(std::time::Instant::now())),
            config,
        }
    }

    /// Check if a request is allowed
    pub async fn allow_request(&self) -> bool {
        self.refill_tokens().await;

        let mut tokens = self.tokens.write().await;
        if *tokens >= 1.0 {
            *tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Get the rate limit value
    pub fn get_rate_limit(&self) -> u64 {
        self.config.requests_per_second as u64
    }

    /// Refill tokens based on time elapsed
    async fn refill_tokens(&self) {
        let now = Instant::now();
        let mut last_refill = self.last_refill.write().await;
        let elapsed = now.duration_since(*last_refill);

        if elapsed >= Duration::from_millis(10) {
            let tokens_to_add = elapsed.as_secs_f64() * f64::from(self.config.requests_per_second);
            let mut tokens = self.tokens.write().await;
            *tokens = (*tokens + tokens_to_add).min(f64::from(self.config.burst_size));
            *last_refill = now;
        }
    }

    /// Get current token count
    pub async fn get_available_tokens(&self) -> f64 {
        self.refill_tokens().await;
        *self.tokens.read().await
    }
}

/// Robustness manager that combines all fault tolerance mechanisms
pub struct RobustnessManager {
    circuit_breaker: Option<CircuitBreaker>,
    retry_mechanism: Option<RetryMechanism>,
    rate_limiter: Option<RateLimiter>,
}

impl RobustnessManager {
    /// Create a new robustness manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            circuit_breaker: None,
            retry_mechanism: None,
            rate_limiter: None,
        }
    }

    /// Add circuit breaker protection
    #[must_use]
    pub fn with_circuit_breaker(mut self, config: CircuitBreakerConfig) -> Self {
        self.circuit_breaker = Some(CircuitBreaker::new(config));
        self
    }

    /// Add retry mechanism
    #[must_use]
    pub fn with_retry(mut self, config: RetryConfig) -> Self {
        self.retry_mechanism = Some(RetryMechanism::new(config));
        self
    }

    /// Add rate limiting
    #[must_use]
    pub fn with_rate_limiting(mut self, config: RateLimitConfig) -> Self {
        self.rate_limiter = Some(RateLimiter::new(config));
        self
    }

    /// Execute a function with robustness mechanisms applied
    ///
    /// # Errors
    ///
    /// Returns an error if the execution fails after applying all configured
    /// robustness mechanisms (circuit breaker, retry, rate limiting).
    pub async fn execute<F, T, E>(&self, func: F) -> Result<T>
    where
        F: std::future::Future<Output = std::result::Result<T, E>> + Send,
        E: std::fmt::Display + Send,
    {
        // Check rate limit first
        if let Some(rate_limiter) = &self.rate_limiter {
            if !rate_limiter.allow_request().await {
                return Err(SongbirdError::RateLimitExceeded {
                    message: "Rate limit exceeded".to_string(),
                    service: Some("robustness_manager".to_string()),
                    limit: Some(rate_limiter.get_rate_limit()),
                    suggestion: Some("Wait before retrying or increase rate limit".to_string()),
                });
            }
        }

        // Apply circuit breaker if configured
        if let Some(circuit_breaker) = &self.circuit_breaker {
            circuit_breaker.call(func).await
        } else {
            // Execute directly if no circuit breaker
            match func.await {
                Ok(result) => Ok(result),
                Err(e) => Err(SongbirdError::ExecutionFailed {
                    message: e.to_string(),
                    command: Some("robustness_protected_call".to_string()),
                    exit_code: None,
                    suggestion: Some("Check the underlying service or operation".to_string()),
                }),
            }
        }
    }

    /// Get status information for all components
    pub async fn get_status(&self) -> RobustnessStatus {
        let circuit_breaker_status = if let Some(cb) = &self.circuit_breaker {
            Some(cb.get_state_info().await)
        } else {
            None
        };

        let available_tokens = if let Some(rl) = &self.rate_limiter {
            Some(rl.get_available_tokens().await)
        } else {
            None
        };

        RobustnessStatus {
            circuit_breaker_status,
            retry_enabled: self.retry_mechanism.is_some(),
            rate_limiter_tokens: available_tokens,
        }
    }
}

impl Default for RobustnessManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Status information for robustness components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobustnessStatus {
    pub circuit_breaker_status: Option<String>,
    pub retry_enabled: bool,
    pub rate_limiter_tokens: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_circuit_breaker_creation() {
        let config = CircuitBreakerConfig::default();
        let cb = CircuitBreaker::new(config);
        let status = cb.get_state_info().await;
        assert!(status.contains("Closed"));
    }

    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let config = CircuitBreakerConfig::default();
        let cb = CircuitBreaker::new(config);

        let result = cb.call(async { Ok::<i32, String>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        // First failure
        let result = cb
            .call(async { Err::<i32, String>("error".to_string()) })
            .await;
        assert!(result.is_err());

        // Second failure should open the circuit
        let result = cb
            .call(async { Err::<i32, String>("error".to_string()) })
            .await;
        assert!(result.is_err());

        // Third call should be rejected by open circuit
        let result = cb.call(async { Ok::<i32, String>(42) }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_mechanism() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1),
            ..Default::default()
        };
        let retry = RetryMechanism::new(config);

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let result = retry
            .retry(move || {
                let counter = Arc::clone(&counter_clone);
                Box::pin(async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err("not yet")
                    } else {
                        Ok(42)
                    }
                })
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimitConfig {
            requests_per_second: 2,
            burst_size: 2,
            window_size: Duration::from_secs(1),
        };
        let rate_limiter = RateLimiter::new(config);

        // First two requests should be allowed
        assert!(rate_limiter.allow_request().await);
        assert!(rate_limiter.allow_request().await);

        // Third request should be denied
        assert!(!rate_limiter.allow_request().await);
    }

    #[tokio::test]
    async fn test_robustness_manager() {
        let manager = RobustnessManager::new()
            .with_circuit_breaker(CircuitBreakerConfig::default())
            .with_retry(RetryConfig::default())
            .with_rate_limiting(RateLimitConfig::default());

        let result = manager.execute(async { Ok::<i32, String>(42) }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_robustness_status() {
        let manager = RobustnessManager::new()
            .with_circuit_breaker(CircuitBreakerConfig::default())
            .with_rate_limiting(RateLimitConfig::default());

        let status = manager.get_status().await;
        assert!(status.circuit_breaker_status.is_some());
        assert!(!status.retry_enabled);
        assert!(status.rate_limiter_tokens.is_some());
    }

    #[test]
    fn test_config_defaults() {
        let cb_config = CircuitBreakerConfig::default();
        assert_eq!(cb_config.failure_threshold, 5);
        assert_eq!(cb_config.success_threshold, 3);

        let retry_config = RetryConfig::default();
        assert_eq!(retry_config.max_attempts, 3);
        assert_eq!(retry_config.backoff_multiplier, 2.0);

        let rate_limit_config = RateLimitConfig::default();
        assert_eq!(rate_limit_config.requests_per_second, 100);
        assert_eq!(rate_limit_config.burst_size, 10);
    }
}
