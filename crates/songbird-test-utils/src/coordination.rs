//! Modern test coordination utilities
//!
//! These utilities replace `sleep()` calls with proper event-driven synchronization.
//! Philosophy: "Test issues WILL BE production issues" - if tests need sleeps,
//! production code has race conditions.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, Barrier as TokioBarrier, Notify};

/// Modern test barrier for synchronizing multiple async tasks
///
/// Replaces: `tokio::time::sleep()` in multi-task coordination
///
/// # Example
/// ```
/// # use songbird_test_utils::coordination::TestBarrier;
/// # tokio_test::block_on(async {
/// let barrier = TestBarrier::new(3);
///
/// let b1 = barrier.clone();
/// let task1 = tokio::spawn(async move {
///     // Do work...
///     b1.wait().await; // Synchronize with other tasks
///     // Continue...
/// });
///
/// let b2 = barrier.clone();
/// let task2 = tokio::spawn(async move {
///     b2.wait().await;
/// });
///
/// barrier.wait().await; // Main task waits too
/// # });
/// ```
#[derive(Clone)]
pub struct TestBarrier {
    inner: Arc<TokioBarrier>,
}

impl TestBarrier {
    /// Create a new barrier for `n` tasks
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            inner: Arc::new(TokioBarrier::new(n)),
        }
    }

    /// Wait for all tasks to reach the barrier
    pub async fn wait(&self) -> tokio::sync::BarrierWaitResult {
        self.inner.wait().await
    }
}

/// Event-driven test signal for one-time notifications
///
/// Replaces: `tokio::time::sleep()` waiting for events
///
/// # Example
/// ```
/// # use songbird_test_utils::coordination::TestSignal;
/// # tokio_test::block_on(async {
/// let (signal, waiter) = TestSignal::new();
///
/// let task = tokio::spawn(async move {
///     waiter.wait().await;
///     // Event received!
/// });
///
/// // Do work...
/// signal.signal();
/// task.await.unwrap();
/// # });
/// ```
pub struct TestSignal {
    tx: oneshot::Sender<()>,
}

/// Receiver side of test coordination signal
///
/// Waits for the corresponding `TestSignal` to be triggered.
/// Used to coordinate async test operations.
pub struct TestWaiter {
    rx: oneshot::Receiver<()>,
}

impl TestSignal {
    /// Create a new signal/waiter pair
    #[must_use]
    pub fn new() -> (Self, TestWaiter) {
        let (tx, rx) = oneshot::channel();
        (
            Self {
                tx,
            },
            TestWaiter {
                rx,
            },
        )
    }

    /// Send the signal (consumes self)
    pub fn signal(self) {
        let _ = self.tx.send(());
    }
}

impl TestWaiter {
    /// Wait for the signal
    pub async fn wait(self) {
        let _ = self.rx.await;
    }

    /// Wait for the signal with timeout
    ///
    /// # Errors
    ///
    /// Returns `tokio::time::error::Elapsed` if the timeout is reached before
    /// the signal is received.
    pub async fn wait_timeout(self, timeout: Duration) -> Result<(), tokio::time::error::Elapsed> {
        tokio::time::timeout(timeout, self.rx).await.map(|_| ())
    }
}

// Note: No Default impl - must use TestSignal::new() to get both signal and waiter

/// Wait group for waiting on N concurrent tasks
///
/// Replaces: `tokio::time::sleep()` waiting for task completion
///
/// # Example
/// ```
/// # use songbird_test_utils::coordination::TestWaitGroup;
/// # tokio_test::block_on(async {
/// let wg = TestWaitGroup::new();
///
/// for _ in 0..10 {
///     wg.add(1);
///     let wg_clone = wg.clone();
///     tokio::spawn(async move {
///         // Do work...
///         wg_clone.done();
///     });
/// }
///
/// wg.wait().await; // Wait for all 10 tasks
/// # });
/// ```
#[derive(Clone)]
pub struct TestWaitGroup {
    count: Arc<AtomicUsize>,
    notify: Arc<Notify>,
}

impl TestWaitGroup {
    /// Create a new wait group
    #[must_use]
    pub fn new() -> Self {
        Self {
            count: Arc::new(AtomicUsize::new(0)),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Add `delta` to the wait group counter
    pub fn add(&self, delta: usize) {
        self.count.fetch_add(delta, Ordering::SeqCst);
    }

    /// Mark one task as done (decrements counter)
    pub fn done(&self) {
        let prev = self.count.fetch_sub(1, Ordering::SeqCst);
        if prev == 1 {
            // Last task done, notify all waiters
            self.notify.notify_waiters();
        }
    }

    /// Wait for all tasks to complete
    pub async fn wait(&self) {
        loop {
            let count = self.count.load(Ordering::SeqCst);
            if count == 0 {
                return;
            }
            self.notify.notified().await;
        }
    }

    /// Wait for all tasks with timeout
    ///
    /// # Errors
    ///
    /// Returns `tokio::time::error::Elapsed` if the timeout is reached before
    /// all tasks complete.
    pub async fn wait_timeout(&self, timeout: Duration) -> Result<(), tokio::time::error::Elapsed> {
        tokio::time::timeout(timeout, self.wait()).await
    }

    /// Get current count
    #[must_use]
    pub fn count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
}

impl Default for TestWaitGroup {
    fn default() -> Self {
        Self::new()
    }
}

/// Eventually assertion - retry until condition is true
///
/// Replaces: `tokio::time::sleep()` + polling in tests
///
/// # Example
/// ```
/// # use songbird_test_utils::coordination::eventually;
/// # use std::time::Duration;
/// # use std::sync::Arc;
/// # use std::sync::atomic::{AtomicBool, Ordering};
/// # tokio_test::block_on(async {
/// let flag = Arc::new(AtomicBool::new(false));
/// let flag_clone = flag.clone();
///
/// tokio::spawn(async move {
///     // Simulate async work
///     tokio::task::yield_now().await;
///     flag_clone.store(true, Ordering::SeqCst);
/// });
///
/// eventually(Duration::from_secs(1), || {
///     flag.load(Ordering::SeqCst)
/// }).await.expect("flag should be set");
/// # });
/// ```
///
/// # Errors
///
/// Returns an error message if the condition is not met within the timeout duration.
pub async fn eventually<F>(timeout: Duration, mut condition: F) -> Result<(), String>
where
    F: FnMut() -> bool,
{
    let start = tokio::time::Instant::now();

    while start.elapsed() < timeout {
        if condition() {
            return Ok(());
        }

        // Yield to allow other tasks to make progress
        tokio::task::yield_now().await;

        // Small delay to prevent busy-waiting
        // This is the ONLY acceptable sleep - for preventing CPU spin
        tokio::time::sleep(Duration::from_micros(100)).await;
    }

    Err(format!("Condition not met within {:?}", timeout))
}

/// Async eventually with async condition
///
/// # Errors
///
/// Returns an error message if the condition is not met within the timeout duration.
pub async fn eventually_async<F, Fut>(timeout: Duration, mut condition: F) -> Result<(), String>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = tokio::time::Instant::now();

    while start.elapsed() < timeout {
        if condition().await {
            return Ok(());
        }

        tokio::task::yield_now().await;
        tokio::time::sleep(Duration::from_micros(100)).await;
    }

    Err(format!("Async condition not met within {:?}", timeout))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    #[tokio::test]
    async fn test_barrier_synchronizes_tasks() {
        let barrier = TestBarrier::new(3);
        let counter = Arc::new(AtomicUsize::new(0));

        let tasks: Vec<_> = (0..3)
            .map(|_| {
                let b = barrier.clone();
                let c = counter.clone();
                tokio::spawn(async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    b.wait().await;
                    c.load(Ordering::SeqCst)
                })
            })
            .collect();

        for task in tasks {
            let count = task.await.unwrap();
            assert_eq!(count, 3, "All tasks should have incremented before barrier");
        }
    }

    #[tokio::test]
    async fn test_signal_notifies_waiter() {
        let (signal, waiter) = TestSignal::new();

        let task = tokio::spawn(async move {
            waiter.wait().await;
            42
        });

        signal.signal();
        let result = task.await.unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_wait_group_tracks_tasks() {
        let wg = TestWaitGroup::new();
        let completed = Arc::new(AtomicUsize::new(0));

        for _ in 0..10 {
            wg.add(1);
            let wg_clone = wg.clone();
            let completed_clone = completed.clone();
            tokio::spawn(async move {
                completed_clone.fetch_add(1, Ordering::SeqCst);
                wg_clone.done();
            });
        }

        wg.wait().await;
        assert_eq!(completed.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn test_eventually_waits_for_condition() {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        tokio::spawn(async move {
            tokio::task::yield_now().await;
            flag_clone.store(true, Ordering::SeqCst);
        });

        eventually(Duration::from_secs(1), || flag.load(Ordering::SeqCst))
            .await
            .expect("flag should be set");
    }

    #[tokio::test]
    async fn test_eventually_times_out() {
        let result = eventually(Duration::from_millis(10), || false).await;
        assert!(result.is_err());
    }
}
