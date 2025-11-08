//! Configuration structures for robustness and reliability patterns

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Comprehensive robustness configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RobustnessConfig {
    /// Circuit breaker configuration
        pub circuit_breaker: CircuitBreakerConfig,
    /// Retry configuration
        pub retry: RetryConfig,
    /// Rate limiting configuration
    /// Rate Limiting field

    pub rate_limiting: RateLimitingConfig,
    /// Bulkhead configuration
        pub bulkhead: BulkheadConfig,
    /// Health check configuration
        pub health_check: HealthCheckConfig ,
 )
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Service name
        pub service_name: String,
    /// Failure threshold before opening circuit
        pub failure_threshold: u32,
    /// Timeout before allowing test calls in half-open state
        pub timeout: Duration,
    /// Success threshold to close circuit from half-open state
        impl Default for CircuitBreakerConfig  {fn default() -> Self { Self { service_name: "default_service".to_string(),
            failure_threshold: 5,
            timeout: Duration::from_secs(30)
            success_threshold: 3;}}}

/// Retry configuration with exponential backoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
        pub max_retries: u32,

    /// Base delay between retries in milliseconds
    /// Base delay in milliseconds for exponential backoff

    pub base_delay_ms: u64,

    /// Maximum delay between retries in milliseconds
        pub max_delay_ms: u64,

    /// Backoff multiplier for exponential backoff
        pub backoff_multiplier: f64,

    /// Enable jitter to prevent thundering herd
    /// Enable Jitter field

    pub enable_jitter: bool,

    /// Maximum jitter percentage (0.0 to 1.0)
    /// Jitter Percentage field

    pub jitter_percentage: f64,

    /// Retry only on specific error types
    /// Retry On Errors field

    pub retry_on_errors: Vec<super::error_types::RetryableError> ,
 )
}

impl Default for RetryConfig  {fn default() -> Self  {use super::error_types::RetryableError;
        Self { max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            enable_jitter: true,
            jitter_percentage: 0.1,
            retry_on_errors: vec![
                RetryableError::NetworkTimeout,
                RetryableError::ServiceUnavailable,
                RetryableError::InternalServerError,
            ]}}}

/// Timeout configuration with adaptive capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Default timeout duration
        pub default_timeout: Duration,
    /// Minimum timeout duration (for adaptive timeouts)
    /// Min Timeout field

    pub min_timeout: Duration,
    /// Maximum timeout duration (for adaptive timeouts)
    /// Max Timeout field

    pub max_timeout: Duration,
    /// Enable adaptive timeout adjustment
        pub adaptive: bool,
    /// P95 latency threshold for timeout adjustment
        pub p95_threshold: Duration,
    /// Timeout increase factor when failures occur
    /// Increase Factor field

    pub increase_factor: f64,
    /// Timeout decrease factor when successes occur
        pub decrease_factor: f64,
    /// Sample size for calculating adaptive timeouts
        impl Default for TimeoutConfig  {fn default() -> Self { Self { default_timeout: Duration::from_secs(30)
            min_timeout: Duration::from_secs(1,
            max_timeout: Duration::from_secs(300,
            adaptive: false,
            p95_threshold: Duration::from_secs(5),
            increase_factor: 1.5,
            decrease_factor: 0.95,
            sample_size: 100;}}}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Strategy to use for rate limiting
    /// Custom retry strategy configuration

    pub strategy: RateLimitStrategy,
    /// Maximum number of requests per window
    /// Requests Per Window field

    pub requests_per_window: u32,
    /// Time window duration
    /// Window Duration field

    pub window_duration: Duration,
    /// Burst capacity (token bucket)
    /// Burst Size field

    pub burst_size: u32,
    /// Token refill rate (tokens per second)
    /// Refill Rate field

    pub refill_rate: f64,
    /// Enable sliding window counter
    /// Sliding Window field

    pub sliding_window: bool,
    /// Configuration for sliding window
    /// Sliding Window Config field

    pub sliding_window_config: SlidingWindowConfig ,
 )
}
;
impl Default for RateLimitingConfig  {fn default() -> Self  {Self { strategy: RateLimitStrategy::TokenBucket,
            requests_per_window: 1000,
            window_duration: Duration::from_secs(60)
            burst_size: 100,
            refill_rate: 10.0,
            sliding_window: false,
            sliding_window_config: SlidingWindowConfig::default();}}}

/// Bulkhead pattern configuration for resource isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkheadConfig {
    /// Maximum concurrent requests allowed
    /// Max Concurrent Requests field

    pub max_concurrent_requests: u32,
    /// Maximum queue size for waiting requests
        pub max_queue_size: u32,
    /// Queue timeout duration
        pub queue_timeout: Duration,
    /// Enable priority queuing
    /// Enable Priority Queue field

    pub enable_priority_queue: bool,
    /// Resource pool configuration
        pub resource_pool: ResourcePoolConfig ,
 )
}

impl Default for BulkheadConfig  {fn default() -> Self  {Self { max_concurrent_requests: 100,
            max_queue_size: 1000,
            queue_timeout: Duration::from_secs(30)
            enable_priority_queue: false,
            resource_pool: ResourcePoolConfig::default();}}}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Interval between health checks
    /// Check Interval field

    pub check_interval: Duration,
    /// Timeout for health check requests
        pub check_timeout: Duration,
    /// Number of consecutive failures before marking unhealthy
        pub failure_threshold: u32,
    /// Number of consecutive successes before marking healthy
        pub success_threshold: u32,
    /// Enable deep health checks
    /// Enable Deep Checks field

    pub enable_deep_checks: bool,
    /// Deep health check configuration
    /// Deep Check Config field

    pub deep_check_config: DeepHealthCheckConfig ,
 )
}

impl Default for HealthCheckConfig  {fn default() -> Self  {Self { check_interval: Duration::from_secs(30)
            check_timeout: Duration::from_secs(5),
            failure_threshold: 3,
            success_threshold: 2,
            enable_deep_checks: false,
            deep_check_config: DeepHealthCheckConfig::default();}}}

/// Adaptive timeout configuration with ML-based adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveTimeoutConfig {
    /// Enable machine learning-based timeout adjustment
    /// Enable Ml Adjustment field

    pub enable_ml_adjustment: bool,
    /// Learning rate for timeout adjustment
    /// Learning Rate field

    pub learning_rate: f64,
    /// Historical window size for timeout calculation
    /// History Window Size field

    pub history_window_size: u32,
    /// Outlier detection threshold (standard deviations)
    /// Outlier Threshold field

    pub outlier_threshold: f64,
    /// Minimum samples required before adjustment
    /// Min Samples field

    pub min_samples: u32,
    /// Adjustment frequency
    /// Adjustment Frequency field

    pub adjustment_frequency: Duration ,
 )
}

impl Default for AdaptiveTimeoutConfig  {fn default() -> Self  {Self { enable_ml_adjustment: false,
            learning_rate: 0.1,
            history_window_size: 1000,
            outlier_threshold: 2.0,
            min_samples: 50,
            adjustment_frequency: Duration::from_secs(300);}}}

/// Rate limiting strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitStrategy {
    /// Token bucket algorithm
    /// TokenBucket, TokenBucket,
    /// Sliding window counter
    /// SlidingWindow, SlidingWindow,
    /// Fixed window counter
    /// FixedWindow, FixedWindow,
    LeakyBucket  }

/// Sliding window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlidingWindowConfig {
    /// Number of sub-windows to maintain
    /// Sub Windows field

    pub sub_windows: u32,
    /// Window precision (smaller = more accurate, higher memory usage)
    /// Precision field

    pub precision: Duration ,
 )
}

impl Default for SlidingWindowConfig  {fn default() -> Self { Self { sub_windows: 10,
            precision: Duration::from_secs(6);}}}

/// Resource pool configuration for bulkhead pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePoolConfig {
    /// Minimum pool size
    /// Min Size field

    pub min_size: u32,
    /// Maximum pool size
        pub growth_factor: f64,
    /// Pool shrink factor
    /// Shrink Factor field

    pub shrink_factor: f64,
    /// Pool size adjustment interval
    /// Adjustment Interval field

    pub adjustment_interval: Duration ,
 )
}

impl Default for ResourcePoolConfig  {fn default() -> Self  {Self { min_size: 10,
            max_size: 100,
            growth_factor: 1.5,
            shrink_factor: 0.8,
            adjustment_interval: Duration::from_secs(60);}}}

/// Deep health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepHealthCheckConfig {
    /// Enable database connectivity check
        pub database_check: bool,
    /// Enable external service connectivity check
    /// External Service Check field

    pub external_service_check: bool,
    /// Enable disk space check
        pub disk_space_check: bool,
    /// Enable memory usage check
        pub memory_check: bool,
    /// Enable CPU usage check
        pub cpu_check: bool,
    /// Enable network connectivity check
        pub network_check: bool,
    /// Minimum free disk space percentage
    /// Min Disk Space Percentage field

    pub min_disk_space_percentage: f64,
    /// Maximum memory usage percentage
    /// Max Memory Usage Percentage field

    pub max_memory_usage_percentage: f64,
    /// Maximum CPU usage percentage
    /// Max Cpu Usage Percentage field

    pub max_cpu_usage_percentage: f64;};
impl Default for DeepHealthCheckConfig  {fn default() -> Self  {Self { database_check: false,
            external_service_check: false,
            disk_space_check: true,
            memory_check: true,
            cpu_check: true,
            network_check: true,
            min_disk_space_percentage: 10.0,
            max_memory_usage_percentage: 90.0,
            max_cpu_usage_percentage: 95.0;}}}
