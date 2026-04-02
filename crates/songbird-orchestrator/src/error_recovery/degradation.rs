// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Graceful Degradation
//!
//! Provides fallback strategies when primary operations fail.

use anyhow::Result;
use std::future::Future;

/// Error returned when degradation strategy has no fallback configured
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoFallbackError;

impl std::fmt::Display for NoFallbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No fallback configured for degradation strategy")
    }
}

impl std::error::Error for NoFallbackError {}

/// Degradation strategy
pub struct DegradationStrategy<T> {
    fallback_value: Option<T>,
    fallback_fn: Option<Box<dyn Fn() -> T + Send + Sync>>,
}

impl<T: Clone> DegradationStrategy<T> {
    pub fn with_value(value: T) -> Self {
        Self {
            fallback_value: Some(value),
            fallback_fn: None,
        }
    }

    pub fn with_fn<F>(f: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            fallback_value: None,
            fallback_fn: Some(Box::new(f)),
        }
    }

    /// Execute with fallback, returning Result to handle missing fallback gracefully
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn try_execute_with_fallback<F, Fut>(
        &self,
        operation: F,
    ) -> std::result::Result<T, NoFallbackError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        match operation().await {
            Ok(result) => Ok(result),
            Err(_) => self.get_fallback(),
        }
    }

    /// Get fallback value if configured
    fn get_fallback(&self) -> std::result::Result<T, NoFallbackError> {
        if let Some(ref value) = self.fallback_value {
            Ok(value.clone())
        } else if let Some(ref f) = self.fallback_fn {
            Ok(f())
        } else {
            Err(NoFallbackError)
        }
    }

    /// Execute with fallback (convenience method - uses unreachable for properly constructed strategies)
    ///
    /// # Safety
    /// This will only fail if the `DegradationStrategy` was constructed without a fallback,
    /// which is not possible through the public API (`with_value` or `with_fn`).
    pub async fn execute_with_fallback<F, Fut>(&self, operation: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        #[allow(
            clippy::expect_used,
            reason = "intentional pattern; clippy false positive for this API"
        )] // invariant: strategy always has a fallback
        self.try_execute_with_fallback(operation)
            .await
            .expect("DegradationStrategy must be constructed with with_value or with_fn")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fallback_value() {
        let strategy = DegradationStrategy::with_value(42);

        let result = strategy.execute_with_fallback(|| async { anyhow::bail!("Error") }).await;

        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_fallback_fn() {
        let strategy = DegradationStrategy::with_fn(|| 99);

        let result = strategy.execute_with_fallback(|| async { anyhow::bail!("Error") }).await;

        assert_eq!(result, 99);
    }

    #[tokio::test]
    async fn test_primary_operation_succeeds() {
        let strategy = DegradationStrategy::with_value(42);

        let result = strategy.execute_with_fallback(|| async { Ok::<_, anyhow::Error>(100) }).await;

        assert_eq!(result, 100); // Primary succeeds, so 100 not fallback 42
    }

    #[tokio::test]
    async fn test_try_execute_with_fallback_success() {
        let strategy = DegradationStrategy::with_value("fallback".to_string());

        let result = strategy
            .try_execute_with_fallback(|| async { Ok::<_, anyhow::Error>("primary".to_string()) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "primary");
    }

    #[tokio::test]
    async fn test_try_execute_with_fallback_failure() {
        let strategy = DegradationStrategy::with_value("fallback".to_string());

        let result =
            strategy.try_execute_with_fallback(|| async { anyhow::bail!("Primary failed") }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "fallback");
    }

    #[tokio::test]
    async fn test_fallback_fn_with_string() {
        let strategy = DegradationStrategy::with_fn(|| "computed".to_string());

        let result = strategy.execute_with_fallback(|| async { anyhow::bail!("Error") }).await;

        assert_eq!(result, "computed");
    }

    #[tokio::test]
    async fn test_fallback_fn_called_each_time() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let counter = std::sync::Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let strategy = DegradationStrategy::with_fn(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            42
        });

        let _ = strategy.execute_with_fallback(|| async { anyhow::bail!("Error") }).await;
        let _ = strategy.execute_with_fallback(|| async { anyhow::bail!("Error") }).await;

        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_no_fallback_error_display() {
        let err = NoFallbackError;
        let display = format!("{err}");
        assert!(display.contains("No fallback"));
    }

    #[test]
    fn test_no_fallback_error_debug() {
        let err = NoFallbackError;
        let debug = format!("{err:?}");
        assert!(debug.contains("NoFallbackError"));
    }

    #[test]
    fn test_no_fallback_error_equality() {
        let err1 = NoFallbackError;
        let err2 = NoFallbackError;
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_no_fallback_error_clone() {
        let err = NoFallbackError;
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn no_fallback_error_source_trait() {
        use std::error::Error;
        let err: &(dyn Error + 'static) = &NoFallbackError;
        assert!(err.source().is_none());
    }
}
