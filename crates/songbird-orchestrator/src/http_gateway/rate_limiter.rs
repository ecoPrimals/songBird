// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Token bucket rate limiter for HTTP gateway
//!
//! **Philosophy**: Fast AND safe Rust with modern async patterns
//!
//! ## Design
//!
//! - Token bucket algorithm (industry standard)
//! - Per-client rate limiting (fair resource allocation)
//! - Synchronous state access (`std::sync::RwLock`) — guards never cross awaits
//! - Automatic token refill (time-based)
//!
//! ## Performance
//!
//! - Minimal lock contention per-client
//! - Minimal contention (per-client buckets)
//! - Zero-copy where possible
//!
//! **Created**: January 16, 2026

use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::Instant;
use tracing::debug;

/// Token bucket for rate limiting
#[derive(Debug, Clone)]
struct TokenBucket {
    /// Current number of tokens
    tokens: f64,

    /// Maximum number of tokens
    capacity: f64,

    /// Token refill rate (tokens per second)
    refill_rate: f64,

    /// Last refill time
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    fn new(capacity: usize, refill_duration: Duration) -> Self {
        let refill_rate = capacity as f64 / refill_duration.as_secs_f64();

        Self {
            tokens: capacity as f64,
            capacity: capacity as f64,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();

        // Calculate new tokens
        let new_tokens = elapsed.mul_add(self.refill_rate, self.tokens);
        self.tokens = new_tokens.min(self.capacity);

        self.last_refill = now;
    }

    /// Try to consume a token
    ///
    /// Returns `true` if token was consumed, `false` if no tokens available
    fn try_consume(&mut self) -> bool {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Get current token count
    fn available_tokens(&mut self) -> f64 {
        self.refill();
        self.tokens
    }
}

/// Rate limiter with per-client token buckets
///
/// **Philosophy**:
/// - Modern idiomatic Rust: async/await, `RwLock`
/// - Fast AND safe: non-blocking, thread-safe
/// - Zero hardcoding: configurable limits
#[derive(Clone)]
pub struct RateLimiter {
    /// Per-client token buckets
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,

    /// Default capacity (requests)
    default_capacity: usize,

    /// Default refill duration
    default_duration: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of requests allowed
    /// * `duration` - Time window for the capacity
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    /// # use songbird_orchestrator::http_gateway::RateLimiter;
    ///
    /// // 100 requests per minute
    /// let limiter = RateLimiter::new(100, Duration::from_secs(60));
    /// ```
    #[must_use]
    pub fn new(capacity: usize, duration: Duration) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            default_capacity: capacity,
            default_duration: duration,
        }
    }

    /// Check rate limit for a client
    ///
    /// # Arguments
    /// * `client_id` - Unique identifier for the client
    ///
    /// # Returns
    /// * `Ok(())` if request is allowed
    /// * `Err(...)` if rate limit exceeded
    ///
    /// # Philosophy
    /// - Fair: Per-client limits
    /// - Automatic: Token refill
    /// # Errors
    ///
    /// Returns an error if the rate limit is exceeded.
    #[expect(clippy::unused_async, reason = "async for API stability")]
    pub async fn check(&self, client_id: &str) -> Result<()> {
        let mut buckets = self.buckets.write().unwrap_or_else(std::sync::PoisonError::into_inner);

        let bucket = buckets.entry(client_id.to_string()).or_insert_with(|| {
            debug!("Creating new token bucket for client: {}", client_id);
            TokenBucket::new(self.default_capacity, self.default_duration)
        });

        if bucket.try_consume() {
            Ok(())
        } else {
            let available = bucket.available_tokens();
            Err(anyhow!(
                "Rate limit exceeded for client '{client_id}': {available:.2} tokens available"
            ))
        }
    }

    /// Get available tokens for a client
    ///
    /// # Arguments
    /// * `client_id` - Unique identifier for the client
    ///
    /// # Returns
    /// * Number of available tokens (can be fractional due to refill)
    #[expect(clippy::unused_async, reason = "async for API stability")]
    pub async fn available_tokens(&self, client_id: &str) -> f64 {
        let mut buckets = self.buckets.write().unwrap_or_else(std::sync::PoisonError::into_inner);

        buckets
            .entry(client_id.to_string())
            .or_insert_with(|| TokenBucket::new(self.default_capacity, self.default_duration))
            .available_tokens()
    }

    #[cfg(test)]
    #[expect(clippy::unused_async, reason = "async for API stability")]
    pub async fn reset(&self, client_id: &str) {
        let mut buckets = self.buckets.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        buckets.remove(client_id);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(10, Duration::from_secs(1));

        // Should allow 10 requests
        for i in 0..10 {
            assert!(limiter.check("test_client").await.is_ok(), "Request {i} should be allowed");
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(10, Duration::from_secs(1));

        // Consume all tokens
        for _ in 0..10 {
            limiter.check("test_client").await.unwrap();
        }

        // 11th request should fail
        assert!(limiter.check("test_client").await.is_err());
    }

    #[tokio::test(start_paused = true)]
    async fn test_rate_limiter_refills_tokens() {
        let limiter = RateLimiter::new(10, Duration::from_millis(100));

        for _ in 0..10 {
            limiter.check("test_client").await.unwrap();
        }

        assert!(limiter.check("test_client").await.is_err());

        tokio::time::advance(Duration::from_millis(150)).await;

        assert!(limiter.check("test_client").await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_per_client() {
        let limiter = RateLimiter::new(10, Duration::from_secs(1));

        // Client A consumes all tokens
        for _ in 0..10 {
            limiter.check("client_a").await.unwrap();
        }

        // Client A should be rate limited
        assert!(limiter.check("client_a").await.is_err());

        // Client B should still be allowed (separate bucket)
        assert!(limiter.check("client_b").await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_available_tokens() {
        let limiter = RateLimiter::new(10, Duration::from_secs(1));

        // Should start with 10 tokens
        let available = limiter.available_tokens("test_client").await;
        assert!((available - 10.0).abs() < 0.01);

        // Consume 5 tokens
        for _ in 0..5 {
            limiter.check("test_client").await.unwrap();
        }

        // Should have ~5 tokens left
        let available = limiter.available_tokens("test_client").await;
        assert!((available - 5.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_rate_limiter_reset_restores_capacity() {
        let limiter = RateLimiter::new(5, Duration::from_secs(1));
        for _ in 0..5 {
            limiter.check("client").await.unwrap();
        }
        assert!(limiter.check("client").await.is_err());
        limiter.reset("client").await;
        assert!(limiter.check("client").await.is_ok());
    }

    #[tokio::test(start_paused = true)]
    async fn test_rate_limiter_partial_refill() {
        let limiter = RateLimiter::new(100, Duration::from_millis(1000));
        for _ in 0..100 {
            limiter.check("c").await.unwrap();
        }
        assert!(limiter.check("c").await.is_err());

        tokio::time::advance(Duration::from_millis(500)).await;

        let tokens = limiter.available_tokens("c").await;
        assert!(tokens > 40.0 && tokens < 60.0, "expected ~50 tokens, got {tokens}");
        assert!(limiter.check("c").await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_error_message_includes_client_id() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));
        limiter.check("unique-client-xyz").await.unwrap();
        let err = limiter.check("unique-client-xyz").await.unwrap_err();
        assert!(err.to_string().contains("unique-client-xyz"));
    }

    #[tokio::test]
    async fn test_rate_limiter_capacity_one_allows_single_request() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));
        assert!(limiter.check("solo").await.is_ok());
        assert!(limiter.check("solo").await.is_err());
    }

    #[tokio::test]
    async fn test_available_tokens_creates_bucket_lazily() {
        let limiter = RateLimiter::new(7, Duration::from_secs(1));
        let tokens = limiter.available_tokens("new-client").await;
        assert!((tokens - 7.0).abs() < 0.01);
    }
}
