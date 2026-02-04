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
    /// This will only fail if the DegradationStrategy was constructed without a fallback,
    /// which is not possible through the public API (with_value or with_fn).
    pub async fn execute_with_fallback<F, Fut>(&self, operation: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        self.try_execute_with_fallback(operation)
            .await
            .expect("DegradationStrategy must be constructed with with_value or with_fn")
    }
}

#[cfg(test)]
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
}
