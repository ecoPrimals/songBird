// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Retry Policies
//!
//! Implements retry strategies with:
//! - Exponential backoff
//! - Jitter
//! - Max attempts
//! - Conditional retry based on error type

use super::{ErrorClass, classify_error};
use anyhow::Result;
use std::future::Future;
use std::time::Duration;
use tracing::{debug, warn};

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,

    /// Initial backoff duration
    pub initial_backoff: Duration,

    /// Maximum backoff duration
    pub max_backoff: Duration,

    /// Backoff multiplier (for exponential backoff)
    pub multiplier: f64,

    /// Enable jitter to prevent thundering herd
    pub jitter: bool,

    /// Which error classes should be retried
    pub retry_on: Vec<ErrorClass>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
            retry_on: vec![ErrorClass::Transient, ErrorClass::Timeout, ErrorClass::RateLimit],
        }
    }
}

impl RetryPolicy {
    /// Create a policy that retries all errors
    #[must_use]
    pub fn retry_all(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            retry_on: vec![
                ErrorClass::Transient,
                ErrorClass::Permanent,
                ErrorClass::RateLimit,
                ErrorClass::Timeout,
                ErrorClass::ResourceExhausted,
            ],
            ..Default::default()
        }
    }

    /// Create a policy that only retries transient errors
    #[must_use]
    pub fn transient_only(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            retry_on: vec![ErrorClass::Transient],
            ..Default::default()
        }
    }

    /// Execute a function with retry logic
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn execute<F, Fut, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut attempt = 0;
        let mut last_error = None;

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        debug!("Operation succeeded on attempt {}", attempt);
                    }
                    return Ok(result);
                }
                Err(err) => {
                    let error_class = classify_error(&err);

                    // Check if we should retry this error
                    if !self.retry_on.contains(&error_class) {
                        debug!("Error {:?} is not retryable", error_class);
                        return Err(err);
                    }

                    // Check if we've exhausted attempts
                    if attempt >= self.max_attempts {
                        warn!("Operation failed after {} attempts: {:?}", attempt, err);
                        return Err(last_error.unwrap_or(err));
                    }

                    // Calculate backoff
                    let backoff = self.calculate_backoff(attempt - 1);

                    debug!(
                        "Retrying after {:?} (attempt {}/{})",
                        backoff, attempt, self.max_attempts
                    );

                    // Wait before retrying
                    tokio::time::sleep(backoff).await;

                    last_error = Some(err);
                }
            }
        }
    }

    /// Calculate backoff duration for a given attempt
    fn calculate_backoff(&self, attempt: u32) -> Duration {
        let base_duration = self.initial_backoff.as_millis() as f64;
        let multiplier = self.multiplier.powi(attempt as i32);
        let mut duration_ms = base_duration * multiplier;

        // Apply max backoff
        duration_ms = duration_ms.min(self.max_backoff.as_millis() as f64);

        // Apply jitter (±25%)
        if self.jitter {
            let jitter_factor = 1.0 + fastrand::f64().mul_add(0.5, -0.25);
            duration_ms *= jitter_factor;
        }

        Duration::from_millis(duration_ms as u64)
    }

    /// Check if an error should be retried
    #[must_use]
    pub fn should_retry(&self, error: &anyhow::Error) -> bool {
        let error_class = classify_error(error);
        self.retry_on.contains(&error_class)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_success_on_first_attempt() {
        let policy = RetryPolicy::default();
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let result: Result<i32> = policy
            .execute(move || {
                let count = call_count_clone.clone();
                async move {
                    count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok::<_, anyhow::Error>(42)
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_success_on_second_attempt() {
        let policy = RetryPolicy::default();
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let result: Result<i32> = policy
            .execute(move || {
                let count = call_count_clone.clone();
                async move {
                    let current = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    if current == 1 {
                        anyhow::bail!("Transient error");
                    }
                    Ok::<_, anyhow::Error>(42)
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let policy = RetryPolicy {
            max_attempts: 2,
            initial_backoff: Duration::from_millis(1), // Fast for testing
            ..Default::default()
        };
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let result: Result<()> = policy
            .execute(move || {
                let count = call_count_clone.clone();
                async move {
                    count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    anyhow::bail!("Transient error")
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_retry_permanent_error() {
        let policy = RetryPolicy::default();
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let result: Result<()> = policy
            .execute(move || {
                let count = call_count_clone.clone();
                async move {
                    count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    anyhow::bail!("Resource not found") // Permanent error
                }
            })
            .await;

        // Should not retry permanent errors
        assert!(result.is_err());
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn test_backoff_calculation() {
        let policy = RetryPolicy {
            initial_backoff: Duration::from_millis(100),
            multiplier: 2.0,
            jitter: false,
            ..Default::default()
        };

        let backoff0 = policy.calculate_backoff(0);
        let backoff1 = policy.calculate_backoff(1);
        let backoff2 = policy.calculate_backoff(2);

        assert_eq!(backoff0.as_millis(), 100);
        assert_eq!(backoff1.as_millis(), 200);
        assert_eq!(backoff2.as_millis(), 400);
    }

    #[test]
    fn test_max_backoff() {
        let policy = RetryPolicy {
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(1),
            multiplier: 2.0,
            jitter: false,
            ..Default::default()
        };

        let backoff10 = policy.calculate_backoff(10);

        // Should be capped at max_backoff
        assert_eq!(backoff10, Duration::from_secs(1));
    }

    #[test]
    fn test_should_retry() {
        let policy = RetryPolicy::default();

        let transient_err = anyhow::anyhow!("Connection failed");
        assert!(policy.should_retry(&transient_err));

        let permanent_err = anyhow::anyhow!("Resource not found");
        assert!(!policy.should_retry(&permanent_err));
    }

    #[test]
    fn retry_all_includes_permanent_in_policy() {
        let p = RetryPolicy::retry_all(5);
        assert!(p.retry_on.contains(&ErrorClass::Permanent));
        assert_eq!(p.max_attempts, 5);
    }

    #[test]
    fn transient_only_policy() {
        let p = RetryPolicy::transient_only(3);
        assert_eq!(p.retry_on, vec![ErrorClass::Transient]);
        assert_eq!(p.max_attempts, 3);
    }

    #[test]
    fn backoff_jitter_nonzero() {
        let policy = RetryPolicy {
            initial_backoff: Duration::from_millis(10),
            multiplier: 2.0,
            jitter: true,
            ..Default::default()
        };
        let a = policy.calculate_backoff(0);
        let b = policy.calculate_backoff(0);
        // Jitter may differ between draws (not guaranteed unequal)
        assert!(a.as_millis() >= 7);
        assert!(a.as_millis() <= 13);
        assert!(b.as_millis() >= 7);
        assert!(b.as_millis() <= 13);
    }

    #[tokio::test]
    async fn exhausted_returns_last_transient_error() {
        let policy = RetryPolicy {
            max_attempts: 2,
            initial_backoff: Duration::from_millis(1),
            ..Default::default()
        };
        let err = policy
            .execute(|| async { Err::<i32, _>(anyhow::anyhow!("rate limited")) })
            .await
            .expect_err("should fail");
        assert!(err.to_string().contains("rate limited"));
    }

    #[test]
    fn should_retry_respects_custom_retry_on_list() {
        let mut p = RetryPolicy::default();
        p.retry_on = vec![ErrorClass::Permanent];
        let transient = anyhow::anyhow!("connection reset");
        assert!(!p.should_retry(&transient));
        let perm = anyhow::anyhow!("Resource not found");
        assert!(p.should_retry(&perm));
    }

    #[test]
    fn default_policy_clones_correctly() {
        let a = RetryPolicy::default();
        let b = a.clone();
        assert_eq!(a.max_attempts, b.max_attempts);
        assert_eq!(a.jitter, b.jitter);
    }

    #[test]
    fn backoff_attempt_zero_with_multiplier_one() {
        let policy = RetryPolicy {
            initial_backoff: Duration::from_millis(50),
            multiplier: 1.0,
            jitter: false,
            max_backoff: Duration::from_secs(60),
            ..Default::default()
        };
        assert_eq!(policy.calculate_backoff(0).as_millis(), 50);
        assert_eq!(policy.calculate_backoff(3).as_millis(), 50);
    }

    #[test]
    fn calculate_backoff_handles_large_attempt_index() {
        let policy = RetryPolicy {
            initial_backoff: Duration::from_millis(1),
            max_backoff: Duration::from_millis(500),
            multiplier: 2.0,
            jitter: false,
            ..Default::default()
        };
        assert_eq!(policy.calculate_backoff(100).as_millis(), 500);
    }

    #[tokio::test]
    async fn execute_stops_on_non_retryable_without_sleep_extra() {
        let policy = RetryPolicy {
            max_attempts: 5,
            initial_backoff: Duration::from_secs(60),
            ..Default::default()
        };
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c2 = calls.clone();
        let r = policy
            .execute(move || {
                let c = c2.clone();
                async move {
                    c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Err::<(), _>(anyhow::anyhow!("invalid request"))
                }
            })
            .await;
        assert!(r.is_err());
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn retry_all_covers_resource_exhausted_class() {
        let p = RetryPolicy::retry_all(4);
        assert!(p.retry_on.contains(&ErrorClass::ResourceExhausted));
    }

    #[test]
    fn transient_only_excludes_rate_limit() {
        let p = RetryPolicy::transient_only(2);
        let rate_err = anyhow::anyhow!("rate limit exceeded");
        assert!(!p.should_retry(&rate_err));
    }
}
