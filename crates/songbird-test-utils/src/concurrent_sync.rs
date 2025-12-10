//! Modern Concurrent Synchronization Primitives for Testing
//!
//! This module provides robust, deterministic synchronization primitives that replace
//! `tokio::time::sleep()` with proper event-driven coordination. These primitives ensure
//! tests are:
//! - **Concurrent**: Multiple tests run in parallel without interference
//! - **Deterministic**: Tests complete as soon as conditions are met (no arbitrary waits)
//! - **Robust**: Race conditions are prevented through proper synchronization
//!
//! ## Anti-Pattern: Using sleep() in tests
//! ```rust,ignore
//! // ❌ BAD: Arbitrary wait, slow, flaky
//! tokio::time::sleep(Duration::from_millis(100)).await;
//! assert!(condition_is_met());
//! ```
//!
//! ## Correct Pattern: Event-driven synchronization
//! ```rust
//! # use songbird_test_utils::concurrent_sync::EventSignal;
//! # #[tokio::main]
//! # async fn main() {
//! // ✅ GOOD: Wait for actual event, fast, reliable
//! let signal = EventSignal::new();
//! let waiter = signal.clone();
//!
//! tokio::spawn(async move {
//!     // Do work
//!     waiter.notify().await;
//! });
//!
//! signal.wait().await; // Returns immediately when notified
//! # }
//! ```

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Barrier, Notify, RwLock};
use tokio::time::timeout;

/// Event signal for one-time notifications (replaces sleep for event waiting)
#[derive(Clone)]
pub struct EventSignal {
    notify: Arc<Notify>,
}

impl EventSignal {
    /// Create a new event signal
    #[must_use]
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
        }
    }

    /// Wait for the event to be signaled
    ///
    /// This blocks until `notify()` is called, or the timeout expires.
    ///
    /// # Example
    /// ```rust
    /// # use songbird_test_utils::concurrent_sync::EventSignal;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let signal = EventSignal::new();
    /// let waiter = signal.clone();
    ///
    /// tokio::spawn(async move {
    ///     // Simulate work
    ///     waiter.notify().await;
    /// });
    ///
    /// signal.wait().await; // Returns when notified
    /// # }
    /// ```
    pub async fn wait(&self) {
        self.notify.notified().await;
    }

    /// Wait for the event with a timeout
    ///
    /// # Errors
    ///
    /// Returns `Err(())` if the timeout expires before the event is signaled.
    pub async fn wait_timeout(&self, duration: Duration) -> Result<(), ()> {
        timeout(duration, self.notify.notified()).await.map_err(|_| ())
    }

    /// Signal the event (notify all waiters)
    #[allow(clippy::unused_async)] // Kept async for consistency with wait()
    pub async fn notify(&self) {
        self.notify.notify_waiters();
    }

    /// Signal exactly one waiter
    #[allow(clippy::unused_async)] // Kept async for consistency with wait()
    pub async fn notify_one(&self) {
        self.notify.notify_one();
    }
}

impl Default for EventSignal {
    fn default() -> Self {
        Self::new()
    }
}

/// State watcher - wait for specific state transitions
///
/// Replaces polling with `sleep()` with event-driven state watching
pub struct StateWatcher<T: Clone + PartialEq + Send + Sync> {
    state: Arc<RwLock<T>>,
    notify: Arc<Notify>,
}

impl<T: Clone + PartialEq + Send + Sync> StateWatcher<T> {
    /// Create a new state watcher with initial state
    #[must_use]
    pub fn new(initial: T) -> Self {
        Self {
            state: Arc::new(RwLock::new(initial)),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Get current state
    pub async fn get(&self) -> T {
        self.state.read().await.clone()
    }

    /// Set new state and notify waiters
    pub async fn set(&self, new_state: T) {
        let mut state = self.state.write().await;
        *state = new_state;
        drop(state);
        self.notify.notify_waiters();
    }

    /// Wait until state equals expected value
    ///
    /// # Example
    /// ```rust
    /// # use songbird_test_utils::concurrent_sync::StateWatcher;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let watcher = StateWatcher::new(0);
    /// let setter = watcher.clone();
    ///
    /// tokio::spawn(async move {
    ///     setter.set(42).await;
    /// });
    ///
    /// watcher.wait_for(42).await; // Returns immediately when state becomes 42
    /// assert_eq!(watcher.get().await, 42);
    /// # }
    /// ```
    pub async fn wait_for(&self, expected: T) {
        loop {
            {
                let current = self.state.read().await;
                if *current == expected {
                    return;
                }
            }
            self.notify.notified().await;
        }
    }

    /// Wait until predicate returns true
    pub async fn wait_until<F>(&self, predicate: F)
    where
        F: Fn(&T) -> bool,
    {
        loop {
            {
                let current = self.state.read().await;
                if predicate(&*current) {
                    return;
                }
            }
            self.notify.notified().await;
        }
    }

    /// Wait for state with timeout
    ///
    /// # Errors
    ///
    /// Returns `Err(())` if the timeout expires before the expected state is reached.
    pub async fn wait_for_timeout(&self, expected: T, duration: Duration) -> Result<(), ()> {
        timeout(duration, self.wait_for(expected)).await.map_err(|_| ())
    }
}

impl<T: Clone + PartialEq + Send + Sync> Clone for StateWatcher<T> {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            notify: Arc::clone(&self.notify),
        }
    }
}

/// Coordination barrier for synchronizing multiple tasks
///
/// Replaces complex `sleep()` coordination with deterministic barriers
pub struct CoordinationBarrier {
    barrier: Arc<Barrier>,
}

impl CoordinationBarrier {
    /// Create a new barrier for N tasks
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            barrier: Arc::new(Barrier::new(n)),
        }
    }

    /// Wait at the barrier until all tasks arrive
    ///
    /// # Example
    /// ```rust
    /// # use songbird_test_utils::concurrent_sync::CoordinationBarrier;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let barrier = CoordinationBarrier::new(3);
    ///
    /// for i in 0..3 {
    ///     let barrier = barrier.clone();
    ///     tokio::spawn(async move {
    ///         // Do independent work
    ///         barrier.wait().await; // All tasks synchronized here
    ///         // Continue together
    ///     });
    /// }
    /// # }
    /// ```
    pub async fn wait(&self) {
        self.barrier.wait().await;
    }
}

impl Clone for CoordinationBarrier {
    fn clone(&self) -> Self {
        Self {
            barrier: Arc::clone(&self.barrier),
        }
    }
}

/// Completion counter - track when N events have occurred
///
/// Replaces polling loops with `sleep()` with event-driven completion tracking
pub struct CompletionCounter {
    target: usize,
    count: Arc<RwLock<usize>>,
    notify: Arc<Notify>,
}

impl CompletionCounter {
    /// Create a counter that waits for N completions
    #[must_use]
    pub fn new(target: usize) -> Self {
        Self {
            target,
            count: Arc::new(RwLock::new(0)),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Increment completion count
    pub async fn increment(&self) {
        let mut count = self.count.write().await;
        *count += 1;
        drop(count);
        self.notify.notify_waiters();
    }

    /// Wait until target completions reached
    pub async fn wait_for_completion(&self) {
        loop {
            {
                let count = self.count.read().await;
                if *count >= self.target {
                    return;
                }
            }
            self.notify.notified().await;
        }
    }

    /// Get current count
    pub async fn current(&self) -> usize {
        *self.count.read().await
    }

    /// Wait with timeout
    ///
    /// # Errors
    ///
    /// Returns `Err(())` if the timeout expires before completion.
    pub async fn wait_with_timeout(&self, duration: Duration) -> Result<(), ()> {
        timeout(duration, self.wait_for_completion()).await.map_err(|_| ())
    }
}

impl Clone for CompletionCounter {
    fn clone(&self) -> Self {
        Self {
            target: self.target,
            count: Arc::clone(&self.count),
            notify: Arc::clone(&self.notify),
        }
    }
}

/// Condition variable - wait for condition with automatic notification
pub struct ConditionVariable<T: Clone + Send + Sync> {
    value: Arc<RwLock<T>>,
    notify: Arc<Notify>,
}

impl<T: Clone + Send + Sync> ConditionVariable<T> {
    /// Create new condition variable
    #[must_use]
    pub fn new(initial: T) -> Self {
        Self {
            value: Arc::new(RwLock::new(initial)),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Get current value
    pub async fn get(&self) -> T {
        self.value.read().await.clone()
    }

    /// Update value and notify all waiters
    pub async fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        let mut value = self.value.write().await;
        updater(&mut *value);
        drop(value);
        self.notify.notify_waiters();
    }

    /// Wait until condition is met
    pub async fn wait_for<F>(&self, condition: F)
    where
        F: Fn(&T) -> bool,
    {
        loop {
            {
                let value = self.value.read().await;
                if condition(&*value) {
                    return;
                }
            }
            self.notify.notified().await;
        }
    }
}

impl<T: Clone + Send + Sync> Clone for ConditionVariable<T> {
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
            notify: Arc::clone(&self.notify),
        }
    }
}

/// Message channel for test coordination
///
/// Type-safe event channel for test synchronization
pub struct TestChannel<T> {
    tx: mpsc::UnboundedSender<T>,
    rx: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<T>>>,
}

impl<T> TestChannel<T> {
    /// Create a new test channel
    #[must_use]
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            tx,
            rx: Arc::new(tokio::sync::Mutex::new(rx)),
        }
    }

    /// Send a message
    ///
    /// # Errors
    ///
    /// Returns an error if the receiver has been dropped.
    pub fn send(&self, msg: T) -> Result<(), mpsc::error::SendError<T>> {
        self.tx.send(msg)
    }

    /// Receive a message (blocking until available)
    pub async fn recv(&self) -> Option<T> {
        self.rx.lock().await.recv().await
    }

    /// Try to receive without blocking
    pub async fn try_recv(&self) -> Option<T> {
        self.rx.lock().await.try_recv().ok()
    }
}

impl<T> Default for TestChannel<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for TestChannel<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            rx: Arc::clone(&self.rx),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_signal() {
        let signal = EventSignal::new();
        let waiter = signal.clone();

        let handle = tokio::spawn(async move {
            waiter.notify().await;
        });

        signal.wait().await;
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_state_watcher() {
        let watcher = StateWatcher::new(0);
        let setter = watcher.clone();

        tokio::spawn(async move {
            setter.set(42).await;
        });

        watcher.wait_for(42).await;
        assert_eq!(watcher.get().await, 42);
    }

    #[tokio::test]
    async fn test_coordination_barrier() {
        let barrier = CoordinationBarrier::new(3);
        let mut handles = vec![];

        for _ in 0..3 {
            let barrier = barrier.clone();
            handles.push(tokio::spawn(async move {
                barrier.wait().await;
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_completion_counter() {
        let counter = CompletionCounter::new(5);

        for _ in 0..5 {
            let counter = counter.clone();
            tokio::spawn(async move {
                counter.increment().await;
            });
        }

        counter.wait_for_completion().await;
        assert_eq!(counter.current().await, 5);
    }
}
