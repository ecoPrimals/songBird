// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Timing protection for Dark Forest compliance
//!
//! Prevents timing-based side-channel attacks by:
//! - Adding random delays
//! - Padding operations to constant time
//! - Using constant-time crypto (delegated to `security provider`)

use crate::error::Result;
use rand::Rng;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::debug;

/// Timing protector
#[derive(Debug)]
pub struct TimingProtector {
    /// Target duration for protected operations
    target_duration: Duration,

    /// Maximum random delay
    max_random_delay: Duration,

    /// Start time (for constant-time padding)
    start_time: Option<Instant>,
}

impl TimingProtector {
    /// Create new timing protector
    #[must_use]
    pub const fn new(target_duration: Duration, max_random_delay: Duration) -> Self {
        Self {
            target_duration,
            max_random_delay,
            start_time: None,
        }
    }

    /// Start timing protection (marks operation start)
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        debug!("Timing protection started");
    }

    /// Apply random delay (before operation)
    pub async fn random_delay(&self) {
        let delay_ms = rand::thread_rng().gen_range(0..self.max_random_delay.as_millis());
        let delay = Duration::from_millis(u64::try_from(delay_ms).unwrap_or(u64::MAX));

        debug!("Applying random delay: {:?}", delay);
        sleep(delay).await;
    }

    /// Pad to constant time (after operation)
    ///
    /// Sleeps remaining time to reach target duration
    ///
    /// # Errors
    ///
    /// Does not return errors; always succeeds.
    pub async fn pad_to_constant_time(&self) -> Result<()> {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed();

            if elapsed < self.target_duration {
                let remaining = self.target_duration.checked_sub(elapsed).unwrap_or_default();
                debug!("Padding to constant time: {:?} remaining", remaining);
                sleep(remaining).await;
            } else {
                debug!(
                    "Operation exceeded target duration: {:?} > {:?}",
                    elapsed, self.target_duration
                );
            }
        }

        Ok(())
    }

    /// Full protected operation wrapper
    ///
    /// Applies random delay before + constant-time padding after
    ///
    /// # Errors
    ///
    /// Returns an error if the inner operation or timing padding fails.
    pub async fn protect<F, T>(&mut self, f: F) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        // Random delay before
        self.random_delay().await;

        // Mark start
        self.start();

        // Execute operation
        let result = f.await?;

        // Pad to constant time
        self.pad_to_constant_time().await?;

        Ok(result)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::error::NfcError;

    #[tokio::test]
    async fn test_timing_protection() {
        let mut protector =
            TimingProtector::new(Duration::from_secs(2), Duration::from_millis(100));

        let start = Instant::now();

        protector
            .protect(async {
                // Fast operation
                sleep(Duration::from_millis(100)).await;
                Ok::<_, crate::error::NfcError>(())
            })
            .await
            .unwrap();

        let elapsed = start.elapsed();

        // Should be padded to at least target duration
        assert!(elapsed >= Duration::from_secs(2));
    }

    #[tokio::test]
    async fn pad_without_start_returns_ok_without_sleeping() {
        let protector = TimingProtector::new(Duration::from_secs(10), Duration::from_millis(1));
        let start = Instant::now();
        protector.pad_to_constant_time().await.expect("pad should succeed when no start mark");
        assert!(
            start.elapsed() < Duration::from_millis(50),
            "without start_time, pad_to_constant_time should not wait for target duration"
        );
    }

    #[tokio::test]
    async fn protect_propagates_inner_error_without_padding_success_path() {
        let mut protector = TimingProtector::new(Duration::from_secs(60), Duration::from_millis(1));
        let err = protector
            .protect(async { Err::<(), NfcError>(NfcError::Timeout) })
            .await
            .expect_err("inner error should surface");
        assert_eq!(err.to_string(), NfcError::Timeout.to_string());
    }

    #[tokio::test]
    async fn start_marks_operation_without_panicking() {
        let mut protector =
            TimingProtector::new(Duration::from_millis(100), Duration::from_millis(2));
        protector.start();
        protector.pad_to_constant_time().await.expect("pad after start");
    }
}
