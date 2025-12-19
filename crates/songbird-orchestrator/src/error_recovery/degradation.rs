//! Graceful Degradation
//!
//! Provides fallback strategies when primary operations fail.

use anyhow::Result;
use std::future::Future;

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

    pub async fn execute_with_fallback<F, Fut>(&self, operation: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        match operation().await {
            Ok(result) => result,
            Err(_) => {
                if let Some(ref value) = self.fallback_value {
                    value.clone()
                } else if let Some(ref f) = self.fallback_fn {
                    f()
                } else {
                    panic!("No fallback configured")
                }
            }
        }
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
