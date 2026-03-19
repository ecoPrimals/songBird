//! Modern Concurrent Test Helpers
//!
//! Production-grade concurrent testing utilities evolved from `BearDog`'s patterns.
//!
//! ## Philosophy
//!
//! "Tests should mirror production concurrency patterns"
//!
//! - Event-driven synchronization (no sleep)
//! - Async-aware primitives
//! - Explicit readiness signaling
//! - Fast and reliable
//!
//! ## Evolution from `BearDog`
//!
//! This module implements the concurrent testing patterns proven by `BearDog`,
//! adapted for Songbird's architecture:
//!
//! - `ReadinessSignal` - Event-driven service startup
//! - `CompletionWaiter` - Async completion tracking
//! - `AsyncBarrier` - Coordination primitive
//! - `RetryPolicy` - Smart network polling
//! - `unique_unix_socket()` - Test isolation

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Notify, RwLock, Semaphore};
use tokio::time::{sleep, timeout};

// Re-export commonly used types for convenience
type Result<T, E = Box<dyn std::error::Error + Send + Sync>> = std::result::Result<T, E>;

#[allow(unused_imports)]
use tracing::{debug, warn};

/// Readiness signal for event-driven service startup
///
/// ## Pattern
///
/// ```no_run
/// # use songbird_test_utils::concurrent_helpers::ReadinessSignal;
/// # use std::sync::Arc;
/// # #[tokio::main]
/// # async fn main() {
/// let ready = Arc::new(ReadinessSignal::new());
/// let ready_clone = ready.clone();
///
/// // Service task
/// tokio::spawn(async move {
///     // ... service initialization ...
///     ready_clone.signal(); // ← Event-driven, no sleep!
/// });
///
/// // Test waits for readiness
/// ready.wait().await.unwrap(); // ← Explicit, fast
/// # }
/// ```
///
/// ## vs Sleep Anti-Pattern
///
/// Before (timing-based):
/// ```ignore
/// tokio::spawn(async { /* start service */ });
/// sleep(Duration::from_secs(1)).await; // ← Hope it's ready!
/// ```
///
/// After (event-driven):
/// ```ignore
/// let ready = Arc::new(ReadinessSignal::new());
/// tokio::spawn(async move { ready.signal(); });
/// ready.wait().await?; // ← Know it's ready!
/// ```
#[derive(Debug, Clone)]
pub struct ReadinessSignal {
    notify: Arc<Notify>,
    signaled: Arc<RwLock<bool>>,
}

impl ReadinessSignal {
    /// Create a new readiness signal
    #[must_use]
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            signaled: Arc::new(RwLock::new(false)),
        }
    }

    /// Signal that the service is ready
    ///
    /// Can be called multiple times safely.
    pub fn signal(&self) {
        let notify = self.notify.clone();
        let signaled = self.signaled.clone();

        tokio::spawn(async move {
            let mut s = signaled.write().await;
            if !*s {
                *s = true;
                notify.notify_waiters();
                debug!("Readiness signaled");
                drop(s);
            }
        });
    }

    /// Wait for the readiness signal with timeout
    ///
    /// Returns an error if the timeout expires before the signal.
    pub async fn wait(&self) -> Result<()> {
        self.wait_with_timeout(Duration::from_secs(30)).await
    }

    /// Wait for the readiness signal with custom timeout
    pub async fn wait_with_timeout(&self, duration: Duration) -> Result<()> {
        // Fast path: already signaled
        {
            let signaled = self.signaled.read().await;
            if *signaled {
                #[cfg(feature = "tracing")]
                debug!("Readiness already signaled (fast path)");
                return Ok(());
            }
        }

        // Slow path: wait for signal
        timeout(duration, self.notify.notified())
            .await
            .map_err(|_| "Readiness signal timeout".into())
            .map(|()| {
                #[cfg(feature = "tracing")]
                debug!("Readiness signal received");
            })
    }

    /// Check if already signaled (non-blocking)
    pub async fn is_ready(&self) -> bool {
        *self.signaled.read().await
    }

    /// Reset the signal (for reuse in tests)
    pub async fn reset(&self) {
        *self.signaled.write().await = false;
        debug!("Readiness signal reset");
    }
}

impl Default for ReadinessSignal {
    fn default() -> Self {
        Self::new()
    }
}

/// Completion waiter for tracking async task completion
///
/// ## Pattern
///
/// ```no_run
/// # use songbird_test_utils::concurrent_helpers::CompletionWaiter;
/// # use std::sync::Arc;
/// # #[tokio::main]
/// # async fn main() {
/// let waiter = Arc::new(CompletionWaiter::new(3)); // Wait for 3 tasks
///
/// for i in 0..3 {
///     let w = waiter.clone();
///     tokio::spawn(async move {
///         // ... do work ...
///         w.complete(); // ← Signal completion
///     });
/// }
///
/// waiter.wait_all().await.unwrap(); // ← All tasks complete!
/// # }
/// ```
#[derive(Debug)]
pub struct CompletionWaiter {
    remaining: Arc<RwLock<usize>>,
    notify: Arc<Notify>,
}

impl CompletionWaiter {
    /// Create a new completion waiter for N tasks
    #[must_use]
    pub fn new(count: usize) -> Self {
        Self {
            remaining: Arc::new(RwLock::new(count)),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Signal that one task has completed
    pub fn complete(&self) {
        let remaining = self.remaining.clone();
        let notify = self.notify.clone();

        tokio::spawn(async move {
            let mut r = remaining.write().await;
            if *r > 0 {
                *r -= 1;
                debug!("Task completed, {} remaining", *r);
                if *r == 0 {
                    notify.notify_waiters();
                }
            }
        });
    }

    /// Wait for all tasks to complete
    pub async fn wait_all(&self) -> Result<()> {
        self.wait_all_with_timeout(Duration::from_secs(60)).await
    }

    /// Wait for all tasks with custom timeout
    pub async fn wait_all_with_timeout(&self, duration: Duration) -> Result<()> {
        // Fast path: already complete
        {
            let remaining = self.remaining.read().await;
            if *remaining == 0 {
                #[cfg(feature = "tracing")]
                debug!("All tasks already complete (fast path)");
                return Ok(());
            }
        }

        // Slow path: wait for completion
        timeout(duration, async {
            loop {
                {
                    let remaining = self.remaining.read().await;
                    if *remaining == 0 {
                        break;
                    }
                }
                self.notify.notified().await;
            }
        })
        .await
        .map_err(|_| "Completion waiter timeout".into())
        .map(|()| {
            #[cfg(feature = "tracing")]
            debug!("All tasks complete");
        })
    }

    /// Get remaining task count
    pub async fn remaining(&self) -> usize {
        *self.remaining.read().await
    }
}

/// Async barrier for coordinating multiple tasks
///
/// ## Pattern
///
/// ```no_run
/// # use songbird_test_utils::concurrent_helpers::AsyncBarrier;
/// # use std::sync::Arc;
/// # #[tokio::main]
/// # async fn main() {
/// let barrier = Arc::new(AsyncBarrier::new(3)); // 3 tasks must arrive
///
/// for i in 0..3 {
///     let b = barrier.clone();
///     tokio::spawn(async move {
///         // ... independent work ...
///         b.wait().await.unwrap(); // ← Synchronize here
///         // ... coordinated work ...
///     });
/// }
/// # }
/// ```
#[derive(Debug)]
pub struct AsyncBarrier {
    count: usize,
    arrived: Arc<RwLock<usize>>,
    generation: Arc<RwLock<usize>>,
    notify: Arc<Notify>,
}

impl AsyncBarrier {
    /// Create a new barrier for N tasks
    #[must_use]
    pub fn new(count: usize) -> Self {
        Self {
            count,
            arrived: Arc::new(RwLock::new(0)),
            generation: Arc::new(RwLock::new(0)),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Wait at the barrier
    pub async fn wait(&self) -> Result<()> {
        self.wait_with_timeout(Duration::from_secs(30)).await
    }

    /// Wait at the barrier with custom timeout
    pub async fn wait_with_timeout(&self, duration: Duration) -> Result<()> {
        let current_gen = *self.generation.read().await;

        // Increment arrived count
        let should_notify = {
            let mut arrived = self.arrived.write().await;
            *arrived += 1;
            #[cfg(feature = "tracing")]
            debug!("Barrier: {} of {} arrived", *arrived, self.count);
            *arrived == self.count
        };

        if should_notify {
            // Last task to arrive: reset and notify all
            {
                let mut arrived = self.arrived.write().await;
                *arrived = 0;
            }
            {
                let mut gen = self.generation.write().await;
                *gen += 1;
            }
            self.notify.notify_waiters();
            #[cfg(feature = "tracing")]
            debug!("Barrier: all arrived, releasing");
            Ok(())
        } else {
            // Wait for last task
            timeout(duration, async {
                loop {
                    let gen = *self.generation.read().await;
                    if gen > current_gen {
                        break;
                    }
                    self.notify.notified().await;
                }
            })
            .await
            .map_err(|_| "Barrier wait timeout".into())
            .map(|()| {
                #[cfg(feature = "tracing")]
                debug!("Barrier: released");
            })
        }
    }
}

/// Retry policy for smart network polling
///
/// ## Pattern
///
/// ```rust,ignore
/// use songbird_test_utils::concurrent_helpers::RetryPolicy;
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let policy = RetryPolicy::default();
///
/// policy.retry_with_backoff(|| async {
///     // Try to connect to service
///     match try_connect().await {
///         Ok(conn) => Ok(conn),
///         Err(e) if e.is_retriable() => Err(e),
///         Err(e) => Err(e), // Fatal error
///     }
/// }).await?;
/// # Ok(())
/// # }
/// # async fn try_connect() -> Result<(), anyhow::Error> { Ok(()) }
/// ```
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    max_attempts: usize,
    initial_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
}

impl RetryPolicy {
    /// Create a new retry policy
    #[must_use]
    pub const fn new(
        max_attempts: usize,
        initial_delay: Duration,
        max_delay: Duration,
        backoff_multiplier: f64,
    ) -> Self {
        Self {
            max_attempts,
            initial_delay,
            max_delay,
            backoff_multiplier,
        }
    }

    /// Retry with exponential backoff
    pub async fn retry_with_backoff<F, Fut, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let mut delay = self.initial_delay;

        for attempt in 1..=self.max_attempts {
            match operation().await {
                Ok(result) => {
                    #[cfg(feature = "tracing")]
                    if attempt > 1 {
                        debug!("Operation succeeded on attempt {}", attempt);
                    }
                    return Ok(result);
                }
                Err(e) if attempt == self.max_attempts => {
                    #[cfg(feature = "tracing")]
                    warn!("Operation failed after {} attempts: {}", self.max_attempts, e);
                    return Err(e);
                }
                Err(e) => {
                    #[cfg(feature = "tracing")]
                    warn!("Attempt {} failed: {}, retrying in {:?}", attempt, e, delay);
                    sleep(delay).await;

                    // Exponential backoff
                    delay = Duration::from_secs_f64(
                        (delay.as_secs_f64() * self.backoff_multiplier)
                            .min(self.max_delay.as_secs_f64()),
                    );
                }
            }
        }

        unreachable!("Loop should have returned")
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(
            5,                          // max_attempts
            Duration::from_millis(100), // initial_delay
            Duration::from_secs(5),     // max_delay
            2.0,                        // backoff_multiplier
        )
    }
}

/// Generate a unique Unix socket path for test isolation
///
/// ## Pattern
///
/// ```no_run
/// # use songbird_test_utils::concurrent_helpers::unique_unix_socket;
/// # #[tokio::main]
/// # async fn main() {
/// let socket_path = unique_unix_socket();
/// // Use for test-specific IPC
/// // Automatically unique per test invocation
/// # }
/// ```
#[must_use]
pub fn unique_unix_socket() -> std::path::PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
    let pid = std::process::id();
    let timestamp =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros();

    std::env::temp_dir().join(format!("songbird-test-{}-{}-{}.sock", pid, timestamp, counter))
}

/// Semaphore-based concurrency limiter
///
/// ## Pattern
///
/// ```no_run
/// # use songbird_test_utils::concurrent_helpers::ConcurrencyLimiter;
/// # use std::sync::Arc;
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let limiter = Arc::new(ConcurrencyLimiter::new(5)); // Max 5 concurrent
///
/// for i in 0..100 {
///     let l = limiter.clone();
///     tokio::spawn(async move {
///         let _guard = l.acquire().await.unwrap();
///         // ... work (max 5 concurrent) ...
///     });
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ConcurrencyLimiter {
    semaphore: Arc<Semaphore>,
}

impl ConcurrencyLimiter {
    /// Create a new concurrency limiter
    #[must_use]
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    /// Acquire a permit (async)
    pub async fn acquire(&self) -> Result<tokio::sync::SemaphorePermit<'_>> {
        self.semaphore
            .acquire()
            .await
            .map_err(|e| format!("Failed to acquire semaphore permit: {}", e).into())
    }

    /// Try to acquire a permit (non-blocking)
    #[must_use]
    pub fn try_acquire(&self) -> Option<tokio::sync::SemaphorePermit<'_>> {
        self.semaphore.try_acquire().ok()
    }

    /// Get available permits
    #[must_use]
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_readiness_signal() {
        let ready = Arc::new(ReadinessSignal::new());
        let ready_clone = ready.clone();

        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            ready_clone.signal();
        });

        ready.wait().await.unwrap();
        assert!(ready.is_ready().await);
    }

    #[tokio::test]
    async fn test_completion_waiter() {
        let waiter = Arc::new(CompletionWaiter::new(3));

        for _ in 0..3 {
            let w = waiter.clone();
            tokio::spawn(async move {
                sleep(Duration::from_millis(50)).await;
                w.complete();
            });
        }

        waiter.wait_all().await.unwrap();
        assert_eq!(waiter.remaining().await, 0);
    }

    #[tokio::test]
    async fn test_async_barrier() {
        let barrier = Arc::new(AsyncBarrier::new(3));
        let completed = Arc::new(RwLock::new(0));

        for _ in 0..3 {
            let b = barrier.clone();
            let c = completed.clone();
            tokio::spawn(async move {
                sleep(Duration::from_millis(50)).await;
                b.wait().await.unwrap();
                let mut count = c.write().await;
                *count += 1;
            });
        }

        sleep(Duration::from_millis(200)).await;
        assert_eq!(*completed.read().await, 3);
    }

    #[tokio::test]
    async fn test_retry_policy() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let policy = RetryPolicy::default();
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_clone = attempts.clone();

        let result = policy
            .retry_with_backoff(move || {
                let attempts = attempts_clone.clone();
                async move {
                    let count = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                    if count < 3 {
                        Err(anyhow::anyhow!("Not yet"))
                    } else {
                        Ok("Success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_unique_unix_socket() {
        let socket1 = unique_unix_socket();
        let socket2 = unique_unix_socket();

        assert_ne!(socket1, socket2);
        assert!(socket1.to_string_lossy().contains("songbird-test"));
    }

    #[tokio::test]
    async fn test_concurrency_limiter() {
        let limiter = Arc::new(ConcurrencyLimiter::new(2));
        let concurrent = Arc::new(RwLock::new(0));
        let max_concurrent = Arc::new(RwLock::new(0));

        let mut handles = vec![];

        for _ in 0..10 {
            let l = limiter.clone();
            let c = concurrent.clone();
            let m = max_concurrent.clone();

            let handle = tokio::spawn(async move {
                let _guard = l.acquire().await.unwrap();

                {
                    let mut count = c.write().await;
                    *count += 1;
                    let mut max = m.write().await;
                    *max = (*max).max(*count);
                }

                sleep(Duration::from_millis(10)).await;

                {
                    let mut count = c.write().await;
                    *count -= 1;
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(*max_concurrent.read().await, 2);
    }
}
