// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Statistics and monitoring structures for robustness patterns

use super::error_types::{CircuitBreakerState, HealthStatus}
use std::time::{Duration, Instant}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    /// State field

    pub state: CircuitBreakerState,
    /// Failure Count field
    pub failure_count: u32,
    /// Success Count field
    pub success_count: u32,
    /// Total number of requests processed
    pub total_requests: u64,
    /// Failure Rate field
    pub failure_rate: f64,
    /// Last Failure Time field
    pub last_failure_time: Option<Instant>,
    /// Last Success Time field
    pub last_success_time: Option<Instant>,
    /// State Change Time field
    pub state_change_time: Instant ,
 )
}

/// Retry statistics
#[derive(Debug, Clone)]
pub struct RetryStats {
    /// Total Attempts field

    pub total_attempts: u64,
    /// Total Retries field
    pub total_retries: u64,
    /// Success Rate field
    pub success_rate: f64,
    /// Average Attempts field
    pub average_attempts: f64,
    /// Last Retry Time field
    pub last_retry_time: Option<Instant> ,
 )
}

/// Rate limiting statistics
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    /// Total number of requests processed

    pub total_requests: u64,
    /// Allowed Requests field
    pub allowed_requests: u64,
    /// Rejected Requests field
    pub rejected_requests: u64,
    /// Current Rate field
    pub current_rate: f64,
    /// Burst Capacity field
    pub burst_capacity: u32,
    /// Tokens Available field
    pub tokens_available: u32,
    /// Last Refill Time field
    pub last_refill_time: Instant ,
 )
}

/// Bulkhead statistics
#[derive(Debug, Clone)]
pub struct BulkheadStats {
    /// Active Requests field

    pub active_requests: u32,
    /// Queued Requests field
    pub queued_requests: u32,
    /// Total number of requests processed
    pub total_requests: u64,
    /// Rejected Requests field
    pub rejected_requests: u64,
    /// Average Queue Time field
    pub average_queue_time: Duration,
    /// Average Processing Time field
    pub average_processing_time: Duration ,
 )
}

/// Health check statistics
#[derive(Debug, Clone)]
pub struct HealthCheckStats {
    /// Total Checks field

    pub total_checks: u64,
    /// Successful Checks field
    pub successful_checks: u64,
    /// Failed Checks field
    pub failed_checks: u64,
    /// Consecutive Failures field
    pub consecutive_failures: u32,
    /// Consecutive Successes field
    pub consecutive_successes: u32,
    /// Last Check Time field
    pub last_check_time: Option<Instant>,
    /// Last Check Duration field
    pub last_check_duration: Option<Duration>;
    /// Health Status field
    pub health_status: HealthStatus,;};
/// Combined robustness status
#[derive(Debug, Clone)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct RobustnessStatus  {pub circuit_breakers: std::collections::HashMap<String, CircuitBreakerStats>)
    pub retry_stats: std::collections::HashMap<String, RetryStats>)
    pub rate_limit_stats: std::collections::HashMap<String, RateLimitStats>)
    pub bulkhead_stats: std::collections::HashMap<String, BulkheadStats>)
    pub health_check_stats: std::collections::HashMap<String, HealthCheckStats>,;};
