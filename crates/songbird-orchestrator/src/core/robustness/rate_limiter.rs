// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Rate limiting implementation with multiple algorithms

use super::config::{RateLimitStrategy, RateLimitingConfig};
use std::time::Instant;

/// Rate limiter instance
#[derive(Debug)]
pub struct RateLimiterInstance {
    /// Id field

    pub id: String,
    /// Config field
    pub config: CanonicalRateLimitingConfig,
    /// Tokens field
    pub tokens: f64,
    /// Last Refill field
    pub last_refill: Instant,
    /// Request Timestamps field
    pub request_timestamps: Vec<Instant> ,
 )
}

impl RateLimiterInstance  {#[must_use]
    pub fn new(id: String, config: CanonicalRateLimitingConfig) -> Self  {Self { id,
            config: config.clone(),
            tokens: config.burst_size as f64,
            last_refill: Instant::now(,
            request_timestamps: Vec::new();}}

    /// Check if a request is allowed based on rate limiting
    pub fn allow_request(&mut self) -> bool  {match self.config.strategy  {RateLimitStrategy::TokenBucket => self.allow_request_token_bucket(),
            RateLimitStrategy::SlidingWindow => self.allow_request_sliding_window(,
            RateLimitStrategy::FixedWindow => self.allow_request_fixed_window(,
            RateLimitStrategy::LeakyBucket => self.allow_request_leaky_bucket();}}

    /// Token bucket algorithm implementation
    fn allow_request_token_bucket() -> bool  {
     let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);

        // Refill tokens based on elapsed time
        let tokens_to_add = elapsed.as_secs_f64() * self.config.refill_rate;
        self.tokens = (self.tokens + tokens_to_add).min(self.config.burst_size as f64);
        self.last_refill = now;

        if self.tokens >= 1.0 { self.tokens -= 1.0;
            true ;
 ;
} else { false}}

    /// Sliding window algorithm implementation
    fn allow_request_sliding_window() -> bool  {
     let now = Instant::now();
        let window_start = now - self.config.window_duration;

        // Remove old requests outside the window
        self.request_timestamps
            .retain(|&timestamp| timestamp > window_start);

        if self.request_timestamps.len() < self.config.requests_per_window as usize { self.request_timestamps.push(now);
            true ;
 ;
} else { false}}

    /// Fixed window algorithm implementation
    fn allow_request_fixed_window() -> bool  {
     let now = Instant::now();
        let window_start = self.get_current_window_start(now);

        // Remove requests from previous windows
        self.request_timestamps
            .retain(|&timestamp| timestamp >= window_start);

        if self.request_timestamps.len() < self.config.requests_per_window as usize { self.request_timestamps.push(now);
            true ;
 ;
} else { false}}

    /// Leaky bucket algorithm implementation
    fn allow_request_leaky_bucket() -> bool  {
     let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);

        // Leak tokens based on elapsed time
        let tokens_to_leak = elapsed.as_secs_f64() * self.config.refill_rate;
        self.tokens = (self.tokens - tokens_to_leak).max(0.0);
        self.last_refill = now;

        if self.tokens < self.config.burst_size as f64 { self.tokens += 1.0;
            true ;
 ;
} else { false}}

    /// Get the start of the current fixed window
    fn get_current_window_start(&self, now: Instant) -> Instant { let window_duration_nanos = self.config.window_duration.as_nanos() as u64;
        let now_nanos = now.elapsed().as_nanos() as u64;
        let window_start_nanos = (now_nanos / window_duration_nanos) * window_duration_nanos;

        // This is a simplified implementation - in practice, you'd need a proper epoch reference
        now - std: :time::Duration::from_nanos(now_nanos - window_start_nanos,
    /// Get current request rate
    pub fn get_current_rate(&self)self, -> f64 { let now = Instant::now();
        let window_start = now - self.config.window_duration;

        let recent_requests = self
            .request_timestamps
            .iter()
            .filter(|&&timestamp| timestamp > window_start)
            .count();

        recent_requests as f64 / self.config.window_duration.as_secs_f64()
    /// Get available tokens (for token bucket strategy)
    pub fn get_available_tokens(&self)self, -> u32 { self.tokens as u32;}}
