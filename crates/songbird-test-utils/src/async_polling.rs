//! Async State Polling - Modern replacement for sleep-based waiting
//!
//! This module provides ergonomic helpers for waiting on async state changes
//! without using sleep(). These are designed specifically for tests that need
//! to wait for eventual consistency.
//!
//! # Philosophy
//!
//! **Tests that sleep are flaky.** Instead, we:
//! - Poll for actual state changes
//! - Use cooperative yielding
//! - Have timeouts for safety
//! - Make intent crystal clear
//!
//! # Examples
//!
//! ```rust
//! use songbird_test_utils::async_polling::*;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Wait for a condition to become true
//! poll_until(Duration::from_secs(5), || async {
//!     // Check your condition
//!     let state = check_some_state().await;
//!     state == ExpectedState::Ready
//! }).await?;
//!
//! // Wait for something to exist
//! let item = poll_until_some(Duration::from_secs(5), || async {
//!     find_item_in_registry("my-item").await
//! }).await?;
//! # Ok(())
//! # }
//! # async fn check_some_state() -> ExpectedState { ExpectedState::Ready }
//! # #[derive(PartialEq)] enum ExpectedState { Ready }
//! # async fn find_item_in_registry(s: &str) -> Option<String> { Some("found".to_string()) }
//! ```

use std::future::Future;
use std::time::Duration;
use tokio::time::timeout;

/// Poll a condition until it returns true or timeout expires
///
/// This is the modern replacement for `loop { sleep(); check() }` patterns.
///
/// # Examples
///
/// ```rust
/// use songbird_test_utils::async_polling::poll_until;
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), String> {
/// // Wait for service to be healthy
/// poll_until(Duration::from_secs(5), || async {
///     check_health().await == "healthy"
/// }).await.map_err(|_| "Timeout waiting for health")?;
/// # Ok(())
/// # }
/// # async fn check_health() -> &'static str { "healthy" }
/// ```
///
/// # Errors
///
/// Returns `Err(())` if timeout expires before condition becomes true.
pub async fn poll_until<F, Fut>(duration: Duration, mut condition: F) -> Result<(), ()>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = bool>,
{
    timeout(duration, async {
        loop {
            if condition().await {
                return;
            }
            // Cooperative yield - let other tasks run
            tokio::task::yield_now().await;
        }
    })
    .await
    .map_err(|_| ())
}

/// Poll until a function returns Some(value) or timeout expires
///
/// This is perfect for waiting until an item appears in a registry,
/// a service becomes available, etc.
///
/// # Examples
///
/// ```rust
/// use songbird_test_utils::async_polling::poll_until_some;
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), String> {
/// // Wait for item to appear in registry
/// let item = poll_until_some(Duration::from_secs(5), || async {
///     find_in_registry("my-item").await
/// }).await.map_err(|_| "Item never appeared")?;
/// # Ok(())
/// # }
/// # async fn find_in_registry(s: &str) -> Option<String> { Some("found".to_string()) }
/// ```
///
/// # Errors
///
/// Returns `Err(())` if timeout expires before condition returns Some.
pub async fn poll_until_some<F, Fut, T>(duration: Duration, mut condition: F) -> Result<T, ()>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Option<T>>,
{
    timeout(duration, async {
        loop {
            if let Some(value) = condition().await {
                return value;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .map_err(|_| ())
}

/// Poll until a function returns Ok(value) or timeout expires
///
/// Useful for waiting on operations that might temporarily fail
/// but will eventually succeed.
///
/// # Examples
///
/// ```rust
/// use songbird_test_utils::async_polling::poll_until_ok;
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), String> {
/// // Wait for connection to succeed
/// let connection = poll_until_ok(Duration::from_secs(5), || async {
///     connect_to_service().await
/// }).await.map_err(|_| "Connection never succeeded")?;
/// # Ok(())
/// # }
/// # async fn connect_to_service() -> Result<String, String> { Ok("connected".to_string()) }
/// ```
///
/// # Errors
///
/// Returns `Err(())` if timeout expires before condition returns Ok.
pub async fn poll_until_ok<F, Fut, T, E>(duration: Duration, mut condition: F) -> Result<T, ()>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    timeout(duration, async {
        loop {
            if let Ok(value) = condition().await {
                return value;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .map_err(|_| ())
}

/// Poll a condition with custom interval between checks
///
/// Use this when you need to rate-limit polling (e.g., external API calls).
/// For most internal state polling, prefer `poll_until()` which uses `yield_now()`.
///
/// # Examples
///
/// ```rust
/// use songbird_test_utils::async_polling::poll_with_interval;
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), String> {
/// // Poll external API every 100ms
/// poll_with_interval(
///     Duration::from_secs(5),
///     Duration::from_millis(100),
///     || async {
///         check_external_api().await == "ready"
///     }
/// ).await.map_err(|_| "API never became ready")?;
/// # Ok(())
/// # }
/// # async fn check_external_api() -> &'static str { "ready" }
/// ```
///
/// # Errors
///
/// Returns `Err(())` if timeout expires before condition becomes true.
pub async fn poll_with_interval<F, Fut>(
    timeout_duration: Duration,
    interval: Duration,
    mut condition: F,
) -> Result<(), ()>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = bool>,
{
    timeout(timeout_duration, async {
        loop {
            if condition().await {
                return;
            }
            // Use sleep for actual rate limiting
            tokio::time::sleep(interval).await;
        }
    })
    .await
    .map_err(|_| ())
}

/// Poll until a collection reaches a certain size
///
/// Common pattern in tests - wait for N providers to register,
/// N tasks to complete, etc.
///
/// # Examples
///
/// ```rust,ignore
/// use songbird_test_utils::async_polling::poll_until_count;
/// use std::time::Duration;
///
/// // Wait for 3 providers to register
/// poll_until_count(Duration::from_secs(5), 3, || async {
///     get_registered_providers().await.len()
/// }).await.map_err(|_| "Never got 3 providers")?;
/// ```
///
/// # Errors
///
/// Returns `Err(())` if timeout expires before count is reached.
pub async fn poll_until_count<F, Fut>(
    duration: Duration,
    expected_count: usize,
    mut get_count: F,
) -> Result<(), ()>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = usize>,
{
    timeout(duration, async {
        loop {
            if get_count().await >= expected_count {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .map_err(|_| ())
}

/// Poll until a value equals expected
///
/// Syntactic sugar for the common pattern of waiting for a value to match.
///
/// # Examples
///
/// ```rust,ignore
/// use songbird_test_utils::async_polling::poll_until_eq;
/// use std::time::Duration;
///
/// // Wait for status to be "ready"
/// poll_until_eq(Duration::from_secs(5), "ready", || async {
///     get_status().await
/// }).await.map_err(|_| "Status never became ready")?;
/// ```
///
/// # Errors
///
/// Returns `Err(())` if timeout expires before value equals expected.
#[allow(clippy::future_not_send)]
pub async fn poll_until_eq<F, Fut, T>(
    duration: Duration,
    expected: T,
    mut get_value: F,
) -> Result<(), ()>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = T>,
    T: PartialEq,
{
    timeout(duration, async {
        loop {
            if get_value().await == expected {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_poll_until_success() {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&flag);

        // Set flag to true after a short delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            flag_clone.store(true, Ordering::SeqCst);
        });

        // Poll until flag is true
        let result = poll_until(Duration::from_secs(1), || {
            let flag = Arc::clone(&flag);
            async move { flag.load(Ordering::SeqCst) }
        })
        .await;

        assert!(result.is_ok(), "Should succeed when condition becomes true");
    }

    #[tokio::test]
    async fn test_poll_until_timeout() {
        // Condition that never becomes true
        let result = poll_until(Duration::from_millis(100), || async { false }).await;

        assert!(result.is_err(), "Should timeout when condition never becomes true");
    }

    #[tokio::test]
    async fn test_poll_until_some() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        // Increment counter in background
        tokio::spawn(async move {
            for i in 1..=10 {
                tokio::time::sleep(Duration::from_millis(10)).await;
                counter_clone.store(i, Ordering::SeqCst);
            }
        });

        // Poll until counter reaches 5
        let result = poll_until_some(Duration::from_secs(1), || {
            let counter = Arc::clone(&counter);
            async move {
                let value = counter.load(Ordering::SeqCst);
                if value >= 5 {
                    Some(value)
                } else {
                    None
                }
            }
        })
        .await;

        assert!(result.is_ok(), "Should succeed when value appears");
        assert!(result.unwrap() >= 5, "Should return the value");
    }

    #[tokio::test]
    async fn test_poll_until_count() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        // Increment counter to 3
        tokio::spawn(async move {
            for _ in 0..3 {
                tokio::time::sleep(Duration::from_millis(20)).await;
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        // Poll until count reaches 3
        let result = poll_until_count(Duration::from_secs(1), 3, || {
            let counter = Arc::clone(&counter);
            async move { counter.load(Ordering::SeqCst) }
        })
        .await;

        assert!(result.is_ok(), "Should succeed when count is reached");
    }

    #[tokio::test]
    async fn test_poll_until_eq() {
        let value = Arc::new(tokio::sync::RwLock::new("pending".to_string()));
        let value_clone = Arc::clone(&value);

        // Change value to "ready" after delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            *value_clone.write().await = "ready".to_string();
        });

        // Poll until value equals "ready"
        let result = poll_until_eq(Duration::from_secs(1), "ready".to_string(), || {
            let value = Arc::clone(&value);
            async move { value.read().await.clone() }
        })
        .await;

        assert!(result.is_ok(), "Should succeed when value matches");
    }
}
