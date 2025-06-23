/*!
 * Robustness and reliability patterns for Songbird Orchestrator
 *
 * This module provides advanced reliability and fault tolerance capabilities including:
 * - Circuit breaker pattern for fault isolation
 * - Exponential backoff retry logic
 * - Rate limiting and throttling
 * - Comprehensive configuration options
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::errors::SongbirdError;

/// Comprehensive robustness configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RobustnessConfig {
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,

    /// Retry configuration
    pub retry: RetryConfig,

    /// Rate limiting configuration
    pub rate_limiting: RateLimitConfig,

    /// Timeout configuration
    pub timeout: TimeoutConfig,

    /// Bulkhead configuration
    pub bulkhead: BulkheadConfig,

    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before opening circuit
    pub failure_threshold: u32,

    /// Recovery timeout in seconds
    pub recovery_timeout_seconds: u64,

    /// Success threshold for closing circuit
    pub success_threshold: u32,

    /// Minimum request threshold before circuit can open
    pub minimum_request_threshold: u32,

    /// Time window for failure rate calculation in seconds
    pub failure_rate_window_seconds: u64,

    /// Failure rate threshold (0.0 to 1.0)
    pub failure_rate_threshold: f64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout_seconds: 30,
            success_threshold: 3,
            minimum_request_threshold: 10,
            failure_rate_window_seconds: 60,
            failure_rate_threshold: 0.5,
        }
    }
}

/// Retry configuration with exponential backoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,

    /// Base delay between retries in milliseconds
    pub base_delay_ms: u64,

    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,

    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,

    /// Jitter factor (0.0 to 1.0) to add randomness
    pub jitter_factor: f64,

    /// Retryable error patterns
    pub retryable_errors: Vec<String>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            retryable_errors: vec![
                "timeout".to_string(),
                "connection_refused".to_string(),
                "service_unavailable".to_string(),
            ],
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per second
    pub max_requests_per_second: u32,

    /// Burst capacity
    pub burst_capacity: u32,

    /// Rate limiting window in seconds
    pub window_seconds: u64,

    /// Rate limiting strategy
    pub strategy: RateLimitStrategy,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_second: 100,
            burst_capacity: 200,
            window_seconds: 1,
            strategy: RateLimitStrategy::TokenBucket,
        }
    }
}

/// Rate limiting strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitStrategy {
    /// Token bucket algorithm
    TokenBucket,

    /// Sliding window log
    SlidingWindowLog,

    /// Fixed window counter
    FixedWindow,

    /// Sliding window counter
    SlidingWindowCounter,
}

/// Timeout configuration for various operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Default operation timeout in seconds
    pub default_timeout_seconds: u64,

    /// Health check timeout in seconds
    pub health_check_timeout_seconds: u64,

    /// Service startup timeout in seconds
    pub startup_timeout_seconds: u64,

    /// Service shutdown timeout in seconds
    pub shutdown_timeout_seconds: u64,

    /// Connection timeout in seconds
    pub connection_timeout_seconds: u64,

    /// Request timeout in seconds
    pub request_timeout_seconds: u64,

    /// Adaptive timeout enabled
    pub adaptive_timeout_enabled: bool,

    /// Adaptive timeout percentile (95th percentile recommended)
    pub adaptive_timeout_percentile: f64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default_timeout_seconds: 30,
            health_check_timeout_seconds: 5,
            startup_timeout_seconds: 60,
            shutdown_timeout_seconds: 30,
            connection_timeout_seconds: 10,
            request_timeout_seconds: 30,
            adaptive_timeout_enabled: true,
            adaptive_timeout_percentile: 0.95,
        }
    }
}

/// Bulkhead configuration for resource isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkheadConfig {
    /// Maximum concurrent operations per service
    pub max_concurrent_per_service: u32,

    /// Maximum concurrent operations globally
    pub max_concurrent_global: u32,

    /// Queue size for pending operations
    pub queue_size: u32,

    /// Resource pools per service type
    pub resource_pools: HashMap<String, u32>,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self {
            max_concurrent_per_service: 10,
            max_concurrent_global: 100,
            queue_size: 50,
            resource_pools: HashMap::new(),
        }
    }
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check interval in seconds
    pub interval_seconds: u64,

    /// Health check timeout in seconds
    pub timeout_seconds: u64,

    /// Failure threshold before marking unhealthy
    pub failure_threshold: u32,

    /// Success threshold for recovery
    pub success_threshold: u32,

    /// Enable adaptive health checks
    pub adaptive_enabled: bool,

    /// Health check strategies per service type
    pub strategies: HashMap<String, HealthCheckStrategy>,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 30,
            timeout_seconds: 5,
            failure_threshold: 3,
            success_threshold: 2,
            adaptive_enabled: true,
            strategies: HashMap::new(),
        }
    }
}

/// Health check strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckStrategy {
    /// Simple ping/pong
    Ping,

    /// HTTP endpoint check
    HttpEndpoint(String),

    /// Custom health check
    Custom(String),

    /// Composite health check
    Composite(Vec<HealthCheckStrategy>),
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Circuit is closed (normal operation)
    Closed,

    /// Circuit is open (failing fast)
    Open,

    /// Circuit is half-open (testing recovery)
    HalfOpen,
}

/// Circuit breaker implementation
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    /// Circuit breaker configuration
    config: CircuitBreakerConfig,

    /// Current state
    state: CircuitState,

    /// Failure count
    failure_count: u32,

    /// Success count (for half-open state)
    success_count: u32,

    /// Last failure time
    last_failure_time: Option<Instant>,

    /// Request count in current window
    request_count: u32,

    /// Window start time
    window_start: Instant,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            request_count: 0,
            window_start: Instant::now(),
        }
    }

    /// Check if the circuit allows requests
    pub fn can_execute(&mut self) -> bool {
        self.update_state();

        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => false,
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful execution
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.reset_failure_count();
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitState::Closed;
                    self.reset_failure_count();
                }
            }
            CircuitState::Open => {
                // Should not happen, but reset if it does
                self.reset_failure_count();
            }
        }
    }

    /// Record a failed execution
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        match self.state {
            CircuitState::Closed => {
                if self.should_open_circuit() {
                    self.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.success_count = 0;
            }
            CircuitState::Open => {
                // Already open, just update failure time
            }
        }
    }

    /// Get current circuit state
    pub fn get_state(&self) -> CircuitState {
        self.state.clone()
    }

    /// Get failure count
    pub fn get_failure_count(&self) -> u32 {
        self.failure_count
    }

    /// Update circuit state based on time and configuration
    fn update_state(&mut self) {
        match self.state {
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    let recovery_timeout =
                        Duration::from_secs(self.config.recovery_timeout_seconds);
                    if last_failure.elapsed() >= recovery_timeout {
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                    }
                }
            }
            _ => {
                // Update request window
                let window_duration = Duration::from_secs(self.config.failure_rate_window_seconds);
                if self.window_start.elapsed() >= window_duration {
                    self.request_count = 0;
                    self.window_start = Instant::now();
                }
            }
        }
    }

    /// Check if circuit should open based on failure rate
    fn should_open_circuit(&self) -> bool {
        if self.request_count < self.config.minimum_request_threshold {
            return false;
        }

        let failure_rate = self.failure_count as f64 / self.request_count as f64;
        failure_rate >= self.config.failure_rate_threshold
    }

    /// Reset failure count
    fn reset_failure_count(&mut self) {
        self.failure_count = 0;
        self.success_count = 0;
        self.last_failure_time = None;
    }
}

/// Retry executor with exponential backoff
#[derive(Debug)]
pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    /// Create a new retry executor
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute operation with retry logic
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, SongbirdError>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>
            + Send
            + Sync,
        E: std::error::Error + Send + Sync + 'static,
    {
        let mut _last_error = None;

        for attempt in 0..=self.config.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    _last_error = Some(err);

                    if attempt < self.config.max_retries {
                        let delay = self.calculate_delay(attempt);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(SongbirdError::Internal {
            message: format!(
                "Operation failed after {} retries",
                self.config.max_retries
            ),
        })
    }

    /// Calculate delay for retry attempt with exponential backoff and jitter
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.config.base_delay_ms as f64;
        let multiplier = self.config.backoff_multiplier;
        let jitter_factor = self.config.jitter_factor;

        // Calculate exponential backoff
        let delay_ms = base_delay * multiplier.powi(attempt as i32);

        // Apply maximum delay limit
        let delay_ms = delay_ms.min(self.config.max_delay_ms as f64);

        // Add jitter to avoid thundering herd
        let jitter = delay_ms * jitter_factor * (rand::random::<f64>() - 0.5) * 2.0;
        let final_delay_ms = (delay_ms + jitter).max(0.0);

        Duration::from_millis(final_delay_ms as u64)
    }
}

/// Rate limiter implementation
#[derive(Debug)]
pub struct RateLimiter {
    config: RateLimitConfig,
    tokens: Arc<RwLock<f64>>,
    last_refill: Arc<RwLock<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(config.burst_capacity as f64)),
            last_refill: Arc::new(RwLock::new(Instant::now())),
            config,
        }
    }

    /// Check if request is allowed
    pub async fn is_allowed(&self) -> bool {
        self.refill_tokens().await;

        let mut tokens = self.tokens.write().await;
        if *tokens >= 1.0 {
            *tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Refill tokens based on time elapsed
    async fn refill_tokens(&self) {
        let now = Instant::now();
        let mut last_refill = self.last_refill.write().await;
        let elapsed = now.duration_since(*last_refill);

        if elapsed >= Duration::from_millis(100) {
            // Refill every 100ms
            let tokens_to_add = elapsed.as_secs_f64() * self.config.max_requests_per_second as f64;

            let mut tokens = self.tokens.write().await;
            *tokens = (*tokens + tokens_to_add).min(self.config.burst_capacity as f64);
            *last_refill = now;
        }
    }
}

/// Robustness manager coordinating all reliability patterns
#[derive(Debug)]
pub struct RobustnessManager {
    config: RobustnessConfig,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    retry_executor: RetryExecutor,
    rate_limiter: RateLimiter,
}

impl RobustnessManager {
    /// Create a new robustness manager
    pub fn new(config: RobustnessConfig) -> Self {
        let retry_executor = RetryExecutor::new(config.retry.clone());
        let rate_limiter = RateLimiter::new(config.rate_limiting.clone());

        Self {
            config,
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            retry_executor,
            rate_limiter,
        }
    }

    /// Get or create circuit breaker for service
    pub async fn get_circuit_breaker(&self, service_id: &str) -> CircuitBreaker {
        let circuit_breakers = self.circuit_breakers.read().await;
        if let Some(cb) = circuit_breakers.get(service_id) {
            cb.clone()
        } else {
            drop(circuit_breakers);

            let mut circuit_breakers = self.circuit_breakers.write().await;
            let cb = CircuitBreaker::new(self.config.circuit_breaker.clone());
            circuit_breakers.insert(service_id.to_string(), cb.clone());
            cb
        }
    }

    /// Execute operation with full robustness patterns
    pub async fn execute_with_robustness<F, T, E>(
        &self,
        service_id: &str,
        operation: F,
    ) -> Result<T, SongbirdError>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>
            + Send
            + Sync,
        E: std::error::Error + Send + Sync + 'static,
    {
        // Check rate limiting
        if !self.rate_limiter.is_allowed().await {
            return Err(SongbirdError::RateLimit {
                message: "Rate limit exceeded".to_string(),
            });
        }

        // Check circuit breaker
        let mut circuit_breaker = self.get_circuit_breaker(service_id).await;
        if !circuit_breaker.can_execute() {
            return Err(SongbirdError::CircuitBreakerOpen {
                message: "Circuit breaker is open".to_string(),
                service_id: service_id.to_string(),
            });
        }

        // Execute with retry logic
        let result = self.retry_executor.execute(operation).await;

        // Update circuit breaker based on result
        match &result {
            Ok(_) => circuit_breaker.record_success(),
            Err(_) => circuit_breaker.record_failure(),
        }

        // Update circuit breaker in storage
        {
            let mut circuit_breakers = self.circuit_breakers.write().await;
            circuit_breakers.insert(service_id.to_string(), circuit_breaker);
        }

        result
    }

    /// Get robustness statistics
    pub async fn get_stats(&self) -> RobustnessStats {
        let circuit_breakers = self.circuit_breakers.read().await;
        let circuit_breaker_states: HashMap<String, CircuitState> = circuit_breakers
            .iter()
            .map(|(id, cb)| (id.clone(), cb.get_state()))
            .collect();

        RobustnessStats {
            circuit_breaker_states,
            total_circuit_breakers: circuit_breakers.len(),
            open_circuit_breakers: circuit_breakers
                .values()
                .filter(|cb| cb.get_state() == CircuitState::Open)
                .count(),
        }
    }
}

/// Robustness statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobustnessStats {
    /// Circuit breaker states by service ID
    pub circuit_breaker_states: HashMap<String, CircuitState>,

    /// Total number of circuit breakers
    pub total_circuit_breakers: usize,

    /// Number of open circuit breakers
    pub open_circuit_breakers: usize,
}
