//! Event-driven test synchronization helpers
//!
//! **Philosophy**: "Sleeps in tests are technical debt. Events are the solution."
//!
//! This module provides utilities for writing truly concurrent, deterministic tests
//! without arbitrary sleep() calls that make tests slow and flaky.
//!
//! ## Patterns
//!
//! 1. **poll_until**: Poll a condition until true or timeout
//! 2. **poll_until_some**: Poll until Some value or timeout
//! 3. **poll_until_ok**: Poll until Ok value or timeout
//! 4. **poll_until_count**: Poll until collection reaches size
//! 5. **wait_for_condition**: Wait for async condition
//!
//! ## Usage
//!
//! ```rust,no_run
//! use common::sync_helpers::*;
//!
//! // Wait for peers to be discovered
//! let peers = poll_until_some(
//!     || {
//!         let p = listener.get_peers().await;
//!         if !p.is_empty() { Some(p) } else { None }
//!     },
//!     Duration::from_secs(5)
//! ).expect("Peers discovered");
//!
//! // Wait for service to be ready
//! assert!(wait_for_condition(
//!     || async { service.is_ready().await },
//!     Duration::from_secs(10)
//! ).await);
//! ```

use std::time::Duration;

/// Poll until condition is true or timeout
///
/// Uses cooperative yielding instead of blocking sleeps.
/// Returns immediately when condition becomes true.
///
/// # Arguments
///
/// * `check` - Closure that returns true when condition is met
/// * `timeout` - Maximum time to wait
///
/// # Returns
///
/// `true` if condition met, `false` if timeout
pub async fn poll_until<F>(check: F, timeout: Duration) -> bool
where
    F: Fn() -> bool,
{
    let start = tokio::time::Instant::now();
    loop {
        if check() {
            return true;
        }
        if start.elapsed() > timeout {
            return false;
        }
        // Cooperative yielding - allows other tasks to run
        tokio::task::yield_now().await;
    }
}

/// Poll until Some value or timeout
///
/// Returns the value as soon as it becomes available.
/// More efficient than sleep-based polling.
///
/// # Arguments
///
/// * `check` - Closure that returns Some(T) when value is ready
/// * `timeout` - Maximum time to wait
///
/// # Returns
///
/// `Some(T)` if value obtained, `None` if timeout
pub async fn poll_until_some<F, T>(mut check: F, timeout: Duration) -> Option<T>
where
    F: FnMut() -> Option<T>,
{
    let start = tokio::time::Instant::now();
    loop {
        if let Some(result) = check() {
            return Some(result);
        }
        if start.elapsed() > timeout {
            return None;
        }
        tokio::task::yield_now().await;
    }
}

/// Poll until Ok value or timeout
///
/// Useful for waiting for services to become available.
///
/// # Arguments
///
/// * `check` - Closure that returns Ok(T) when ready
/// * `timeout` - Maximum time to wait
///
/// # Returns
///
/// `Ok(T)` if successful, `Err` if timeout or error
pub async fn poll_until_ok<F, T, E>(mut check: F, timeout: Duration) -> Result<T, PollError<E>>
where
    F: FnMut() -> Result<T, E>,
{
    let start = tokio::time::Instant::now();
    let mut last_error = None;
    
    loop {
        match check() {
            Ok(result) => return Ok(result),
            Err(e) => last_error = Some(e),
        }
        
        if start.elapsed() > timeout {
            return Err(PollError::Timeout(last_error));
        }
        
        tokio::task::yield_now().await;
    }
}

/// Poll until collection reaches expected count
///
/// Optimized for waiting for peers, connections, etc.
///
/// # Arguments
///
/// * `get_count` - Closure that returns current count
/// * `expected` - Expected count
/// * `timeout` - Maximum time to wait
pub async fn poll_until_count<F>(get_count: F, expected: usize, timeout: Duration) -> bool
where
    F: Fn() -> usize,
{
    poll_until(|| get_count() >= expected, timeout).await
}

/// Poll until collection equals expected count (exact match)
pub async fn poll_until_eq<F>(get_count: F, expected: usize, timeout: Duration) -> bool
where
    F: Fn() -> usize,
{
    poll_until(|| get_count() == expected, timeout).await
}

/// Wait for async condition
///
/// For conditions that require async operations to check.
///
/// # Arguments
///
/// * `condition` - Async closure that returns true when condition met
/// * `timeout` - Maximum time to wait
pub async fn wait_for_condition<F, Fut>(condition: F, timeout: Duration) -> bool
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = tokio::time::Instant::now();
    loop {
        if condition().await {
            return true;
        }
        if start.elapsed() > timeout {
            return false;
        }
        tokio::task::yield_now().await;
    }
}

/// Poll error types
#[derive(Debug)]
pub enum PollError<E> {
    /// Timeout reached
    Timeout(Option<E>),
}

impl<E: std::fmt::Display> std::fmt::Display for PollError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PollError::Timeout(Some(e)) => write!(f, "Poll timeout (last error: {})", e),
            PollError::Timeout(None) => write!(f, "Poll timeout"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for PollError<E> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_poll_until_immediate() {
        // Condition already true
        let result = poll_until(|| true, Duration::from_secs(1)).await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_poll_until_eventual() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        // Spawn task that sets condition after 10ms
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            counter_clone.store(1, Ordering::SeqCst);
        });
        
        // Poll should succeed before timeout
        let result = poll_until(
            || counter.load(Ordering::SeqCst) == 1,
            Duration::from_secs(1)
        ).await;
        
        assert!(result);
    }

    #[tokio::test]
    async fn test_poll_until_timeout() {
        // Condition never true
        let result = poll_until(|| false, Duration::from_millis(50)).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_poll_until_some_immediate() {
        let result = poll_until_some(
            || Some(42),
            Duration::from_secs(1)
        ).await;
        
        assert_eq!(result, Some(42));
    }

    #[tokio::test]
    async fn test_poll_until_some_eventual() {
        let value = Arc::new(tokio::sync::RwLock::new(None));
        let value_clone = value.clone();
        
        // Spawn task that sets value
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            *value_clone.write().await = Some(42);
        });
        
        // Poll should get value
        let result = poll_until_some(
            || {
                let v = value.blocking_read();
                *v
            },
            Duration::from_secs(1)
        ).await;
        
        assert_eq!(result, Some(42));
    }

    #[tokio::test]
    async fn test_poll_until_count() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        // Spawn task that increments counter
        tokio::spawn(async move {
            for i in 1..=5 {
                tokio::time::sleep(Duration::from_millis(5)).await;
                counter_clone.store(i, Ordering::SeqCst);
            }
        });
        
        // Wait for count to reach 3
        let result = poll_until_count(
            || counter.load(Ordering::SeqCst),
            3,
            Duration::from_secs(1)
        ).await;
        
        assert!(result);
    }

    #[tokio::test]
    async fn test_wait_for_condition_async() {
        let ready = Arc::new(tokio::sync::RwLock::new(false));
        let ready_clone = ready.clone();
        
        // Spawn task that sets ready flag
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            *ready_clone.write().await = true;
        });
        
        // Wait for ready
        let result = wait_for_condition(
            || async {
                *ready.read().await
            },
            Duration::from_secs(1)
        ).await;
        
        assert!(result);
    }
}

