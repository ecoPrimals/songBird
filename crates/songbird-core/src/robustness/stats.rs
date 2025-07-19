//! Statistics and monitoring structures for robustness patterns

use super::error_types::{CircuitBreakerState, HealthStatus};
use std::time::{Duration, Instant};

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: CircuitBreakerState,
    pub failure_count: u32,
    pub success_count: u32,
    pub total_requests: u64,
    pub failure_rate: f64,
    pub last_failure_time: Option<Instant>,
    pub last_success_time: Option<Instant>,
    pub state_change_time: Instant,
}

/// Retry statistics
#[derive(Debug, Clone)]
pub struct RetryStats {
    pub total_attempts: u64,
    pub total_retries: u64,
    pub success_rate: f64,
    pub average_attempts: f64,
    pub last_retry_time: Option<Instant>,
}

/// Rate limiting statistics
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    pub total_requests: u64,
    pub allowed_requests: u64,
    pub rejected_requests: u64,
    pub current_rate: f64,
    pub burst_capacity: u32,
    pub tokens_available: u32,
    pub last_refill_time: Instant,
}

/// Bulkhead statistics
#[derive(Debug, Clone)]
pub struct BulkheadStats {
    pub active_requests: u32,
    pub queued_requests: u32,
    pub total_requests: u64,
    pub rejected_requests: u64,
    pub average_queue_time: Duration,
    pub average_processing_time: Duration,
}

/// Health check statistics
#[derive(Debug, Clone)]
pub struct HealthCheckStats {
    pub total_checks: u64,
    pub successful_checks: u64,
    pub failed_checks: u64,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub last_check_time: Option<Instant>,
    pub last_check_duration: Option<Duration>,
    pub health_status: HealthStatus,
}

/// Combined robustness status
#[derive(Debug, Clone)]
pub struct RobustnessStatus {
    pub circuit_breakers: std::collections::HashMap<String, CircuitBreakerStats>,
    pub retry_stats: std::collections::HashMap<String, RetryStats>,
    pub rate_limit_stats: std::collections::HashMap<String, RateLimitStats>,
    pub bulkhead_stats: std::collections::HashMap<String, BulkheadStats>,
    pub health_check_stats: std::collections::HashMap<String, HealthCheckStats>,
} 