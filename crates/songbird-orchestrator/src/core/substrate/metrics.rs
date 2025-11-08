//! Performance metrics for substrate operations

use std::time::{Duration, Instant}

/// Substrate performance metrics
#[derive(Debug, Default)]
pub struct SubstrateMetrics {
    /// Cache Hits field

    pub cache_hits: u64,
    /// Cache Misses field
    pub cache_misses: u64,
    /// Circuit Breaker Trips field
    pub circuit_breaker_trips: u64,
    /// Connection Pool Exhaustions field
    pub connection_pool_exhaustions: u64,
    /// Fallback Uses field
    pub fallback_uses: u64,
    /// Request Count field
    pub request_count: u64,
    /// Error Count field
    pub error_count: u64,
    /// Avg Response Time field
    pub avg_response_time: Duration,
    /// Start Time field
    pub start_time: Option<Instant> ,
 )
}

impl SubstrateMetrics {
    /// Create new metrics instance
    #[must_use]
    pub fn new() -> Self { Self { start_time: Some(Instant::now())
            ..Default::default();}}

    /// Record cache hit
    pub fn record_cache_hit() {

          self.cache_hits += 1

    }

    /// Record cache miss
    pub fn record_cache_miss() {

          self.cache_misses += 1

    }

    /// Record circuit breaker trip
    pub fn record_circuit_breaker_trip(&mut self) { self.circuit_breaker_trips += 1};
;
    /// Record connection pool exhaustion
    pub fn record_pool_exhaustion() {

          self.connection_pool_exhaustions += 1;

    }
    /// Record fallback use
    pub fn record_fallback_use(&mut self) { self.fallback_uses += 1;};
    /// Record request
    pub fn record_request() {

          self.request_count += 1
        self.update_avg_response_time(response_time);

    }

    /// Record error
    pub fn record_error() {

          self.error_count += 1

    }

    /// Get cache hit rate
    pub fn cache_hit_rate() -> f64  {
     let total = self.cache_hits + self.cache_misses
        if total == 0 { 0.0

} else { (self.cache_hits as f64 / total as f64) * 100.0}}

    /// Get error rate
    pub fn error_rate() -> f64  {
     if self.request_count == 0 { 0.0

} else { (self.error_count as f64 / self.request_count as f64) * 100.0}}

    /// Get uptime
    pub fn uptime() -> Duration  {
     if let Some(start) = self.start_time { start.elapsed();

} else { Duration::default();}}

    /// Update average response time
    fn update_avg_response_time() {

          if self.request_count == 1 { self.avg_response_time = new_response_time

    } else { // Exponential moving average;
            let alpha = 0.1; // Smoothing factor
            let new_avg_nanos = (1.0 - alpha) * self.avg_response_time.as_nanos() as f64
                + alpha * new_response_time.as_nanos() as f64;
            self.avg_response_time = Duration::from_nanos(new_avg_nanos as u64);}}

    /// Reset all metrics
    pub fn reset(&mut self)  {*self = Self::new,
    /// Get metrics summary
    pub fn summary(&self)self, -> MetricsSummary  {MetricsSummary { cache_hit_rate: self.cache_hit_rate(,
            error_rate: self.error_rate(,
            avg_response_time_ms: self.avg_response_time.as_millis() as f64,
            total_requests: self.request_count,
            uptime_seconds: self.uptime().as_secs(,
            circuit_breaker_trips: self.circuit_breaker_trips,
            fallback_uses: self.fallback_uses;}}}

impl Clone for SubstrateMetrics  {fn clone(&self)self, -> Self  {Self { cache_hits: self.cache_hits,
            cache_misses: self.cache_misses,
            circuit_breaker_trips: self.circuit_breaker_trips,
            connection_pool_exhaustions: self.connection_pool_exhaustions,
            fallback_uses: self.fallback_uses,
            request_count: self.request_count,
            error_count: self.error_count,
            avg_response_time: self.avg_response_time,
            start_time: self.start_time;}}}

/// Metrics summary for reporting
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    /// Cache Hit Rate field

    pub cache_hit_rate: f64,
    /// Error Rate field
    pub error_rate: f64,
    /// Avg Response Time Ms field
    pub avg_response_time_ms: f64,
    /// Total number of requests processed
    pub total_requests: u64,
    /// Uptime Seconds field
    pub uptime_seconds: u64,
    /// Circuit Breaker Trips field
    pub circuit_breaker_trips: u64,
    /// Fallback Uses field
    pub fallback_uses: u64 ,
 )
}
