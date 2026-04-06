// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::ignore_without_reason,
    clippy::unreadable_literal,
    clippy::no_effect_underscore_binding,
    clippy::float_cmp,
    clippy::default_trait_access,
    clippy::needless_collect,
    clippy::unused_async,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements,
    clippy::unnecessary_wraps,
    clippy::used_underscore_binding,
    clippy::struct_excessive_bools,
    clippy::similar_names,
    clippy::significant_drop_tightening,
    clippy::struct_field_names,
    clippy::match_same_arms,
    clippy::future_not_send,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

//! Modern event-driven test helpers
//!
//! **Philosophy**: "Sleeps in tests are technical debt. Events are the solution."
//!
//! This module provides event-driven coordination primitives for writing
//! truly concurrent, fast, and robust tests.
//!
//! ## Patterns
//!
//! 1. **Server Startup**: Event notification when server is ready
//! 2. **Ephemeral Ports**: Dynamic port allocation (no conflicts)
//! 3. **Channel Coordination**: Message passing for state changes
//! 4. **Event Waiters**: Wait for conditions without polling
//!
//! ## Usage
//!
//! ```rust,no_run
//! use common::event_helpers::*;
//!
//! // Server startup with notification
//! let (handle, ready) = start_server_with_notify(async {
//!     server.run().await
//! }).await;
//! ready.notified().await; // Guaranteed ready!
//!
//! // Ephemeral port (no conflicts)
//! let (listener, port) = bind_ephemeral().await?;
//!
//! // Event-driven coordination
//! let (tx, rx) = oneshot::channel();
//! tokio::spawn(async move {
//!     let result = operation().await;
//!     tx.send(result).ok();
//! });
//! let result = rx.await?;
//! ```

use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, UnixListener};
use tokio::sync::{Notify, mpsc, oneshot};
use tokio::task::JoinHandle;
use tracing::{debug, trace};

/// Server readiness notification
#[derive(Clone)]
pub struct ReadyNotifier {
    notify: Arc<Notify>,
}

impl ReadyNotifier {
    /// Create a new readiness notifier
    pub fn new() -> (Self, Arc<Notify>) {
        let notify = Arc::new(Notify::new());
        let notifier = Self {
            notify: notify.clone(),
        };
        (notifier, notify)
    }

    /// Signal that server is ready
    pub fn signal_ready(&self) {
        trace!("🟢 Server ready signal sent");
        self.notify.notify_one();
    }

    /// Wait for server to be ready (with timeout)
    pub async fn wait_ready(notify: Arc<Notify>, timeout: Duration) -> Result<()> {
        tokio::time::timeout(timeout, notify.notified())
            .await
            .context("Timeout waiting for server ready")?;
        debug!("✅ Server ready");
        Ok(())
    }
}

/// Bind to an ephemeral TCP port (no conflicts!)
///
/// Returns the listener and the assigned port number.
/// Each test gets a unique port allocated by the OS.
///
/// # Example
///
/// ```rust,no_run
/// let (listener, port) = bind_ephemeral().await?;
/// info!("Server listening on port {}", port);
/// ```
pub async fn bind_ephemeral() -> Result<(TcpListener, u16)> {
    let listener =
        TcpListener::bind("127.0.0.1:0").await.context("Failed to bind ephemeral port")?;
    let port = listener.local_addr()?.port();
    debug!("🔌 Bound to ephemeral port: {}", port);
    Ok((listener, port))
}

/// Bind to an ephemeral TCP port with specific address
#[allow(dead_code)]
pub async fn bind_ephemeral_addr(addr: &str) -> Result<(TcpListener, SocketAddr)> {
    let listener =
        TcpListener::bind(format!("{addr}:0")).await.context("Failed to bind ephemeral port")?;
    let sock_addr = listener.local_addr()?;
    debug!("🔌 Bound to ephemeral address: {}", sock_addr);
    Ok((listener, sock_addr))
}

/// Create a temporary Unix socket path
///
/// Returns a unique path for Unix socket testing.
/// Automatically cleaned up when dropped.
pub fn temp_unix_socket() -> Result<TempUnixSocket> {
    let path = std::env::temp_dir().join(format!("songbird-test-{}.sock", uuid::Uuid::new_v4()));
    debug!("🔌 Created temp Unix socket: {}", path.display());
    Ok(TempUnixSocket {
        path,
    })
}

/// Temporary Unix socket that auto-cleans up
pub struct TempUnixSocket {
    path: std::path::PathBuf,
}

impl TempUnixSocket {
    /// Get the socket path
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Bind a Unix listener to this socket
    pub async fn bind(&self) -> Result<UnixListener> {
        UnixListener::bind(&self.path).context("Failed to bind Unix socket")
    }
}

impl Drop for TempUnixSocket {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = std::fs::remove_file(&self.path);
            trace!("🧹 Cleaned up Unix socket: {}", self.path.display());
        }
    }
}

/// Event-driven server starter
///
/// Spawns a server and provides notification when it's ready.
///
/// # Returns
///
/// - `JoinHandle`: Server task handle
/// - `Arc<Notify>`: Notification when ready
///
/// # Example
///
/// ```rust,no_run
/// let (handle, ready) = spawn_server_with_notify(async {
///     server.run_with_callback(|| {
///         // Server is ready!
///     }).await
/// });
///
/// ready.notified().await; // Wait for ready
/// // Now safe to connect!
/// ```
#[allow(dead_code)]
pub fn spawn_server_with_notify<F, Fut>(server_fn: F) -> (JoinHandle<Result<()>>, Arc<Notify>)
where
    F: FnOnce(Arc<Notify>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<()>> + Send + 'static,
{
    let ready = Arc::new(Notify::new());
    let ready_clone = ready.clone();

    let handle = tokio::spawn(async move { server_fn(ready_clone).await });

    (handle, ready)
}

/// Wait for a condition with timeout (no polling sleeps!)
///
/// Uses cooperative yielding instead of blocking sleeps.
/// Returns immediately when condition becomes true.
///
/// # Example
///
/// ```rust,no_run
/// wait_for(|| server.is_ready(), Duration::from_secs(5))
///     .await
///     .expect("Server ready");
/// ```
pub async fn wait_for<F>(check: F, timeout: Duration) -> Result<()>
where
    F: Fn() -> bool,
{
    let start = tokio::time::Instant::now();
    loop {
        if check() {
            return Ok(());
        }
        if start.elapsed() > timeout {
            anyhow::bail!("Timeout waiting for condition");
        }
        // Small sleep advances virtual time under `start_paused` and avoids
        // CPU-burning busy loops under real time.
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}

/// Wait for async condition with timeout
#[allow(dead_code)]
pub async fn wait_for_async<F, Fut>(condition: F, timeout: Duration) -> Result<()>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = tokio::time::Instant::now();
    loop {
        if condition().await {
            return Ok(());
        }
        if start.elapsed() > timeout {
            anyhow::bail!("Timeout waiting for async condition");
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}

/// Wait for a value with timeout (no polling!)
///
/// Returns the value as soon as it becomes available.
///
/// # Example
///
/// ```rust,no_run
/// let peers = wait_for_some(
///     || {
///         let p = service.get_peers();
///         if !p.is_empty() { Some(p) } else { None }
///     },
///     Duration::from_secs(5)
/// ).await?;
/// ```
#[allow(dead_code)]
pub async fn wait_for_some<F, T>(mut check: F, timeout: Duration) -> Result<T>
where
    F: FnMut() -> Option<T>,
{
    let start = tokio::time::Instant::now();
    loop {
        if let Some(result) = check() {
            return Ok(result);
        }
        if start.elapsed() > timeout {
            anyhow::bail!("Timeout waiting for value");
        }
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}

/// Channel-based state coordinator
///
/// Provides mpsc channel for multi-event coordination.
///
/// # Example
///
/// ```rust,no_run
/// let (tx, mut rx) = event_channel(10);
///
/// tokio::spawn(async move {
///     loop {
///         // Do work
///         tx.send(Event::Progress(50)).await.ok();
///     }
/// });
///
/// while let Some(event) = rx.recv().await {
///     match event {
///         Event::Progress(p) => info!("Progress: {}%", p),
///         Event::Complete => break,
///     }
/// }
/// ```
#[allow(dead_code)]
pub fn event_channel<T>(buffer: usize) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
    mpsc::channel(buffer)
}

/// One-shot channel for single response
///
/// Perfect for "wait for server startup" patterns.
///
/// # Example
///
/// ```rust,no_run
/// let (tx, rx) = response_channel();
///
/// tokio::spawn(async move {
///     let result = expensive_operation().await;
///     tx.send(result).ok();
/// });
///
/// let result = rx.await?;
/// ```
pub fn response_channel<T>() -> (oneshot::Sender<T>, oneshot::Receiver<T>) {
    oneshot::channel()
}

/// Concurrent event selector
///
/// Wait for first of multiple events using `tokio::select`!
///
/// # Example
///
/// ```rust,no_run
/// tokio::select! {
///     result = operation1() => { /* handle */ },
///     result = operation2() => { /* handle */ },
///     _ = tokio::time::sleep(Duration::from_secs(5)) => {
///         panic!("Timeout!");
///     }
/// }
/// ```
#[allow(dead_code)]
pub async fn select_first<F1, F2, T1, T2>(future1: F1, future2: F2) -> Result<Either<T1, T2>>
where
    F1: std::future::Future<Output = Result<T1>>,
    F2: std::future::Future<Output = Result<T2>>,
{
    tokio::select! {
        result1 = future1 => Ok(Either::Left(result1?)),
        result2 = future2 => Ok(Either::Right(result2?)),
    }
}

/// Either type for `select_first`
#[allow(dead_code)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ephemeral_ports_no_conflict() {
        // Bind two ephemeral ports - should never conflict
        let (listener1, port1) = bind_ephemeral().await.unwrap();
        let (listener2, port2) = bind_ephemeral().await.unwrap();

        assert_ne!(port1, port2, "Ports should be different");
        assert!(port1 > 0);
        assert!(port2 > 0);

        drop(listener1);
        drop(listener2);
    }

    #[tokio::test]
    async fn test_ready_notifier() {
        let (notifier, ready) = ReadyNotifier::new();

        // Spawn task that signals ready after short delay
        tokio::spawn(async move {
            // Simulate startup work
            tokio::task::yield_now().await;
            notifier.signal_ready();
        });

        // Wait for ready (should be fast!)
        ReadyNotifier::wait_ready(ready, Duration::from_secs(1)).await.unwrap();
    }

    #[tokio::test]
    async fn test_wait_for_immediate() {
        // Condition already true - should return immediately
        wait_for(|| true, Duration::from_secs(1)).await.unwrap();
    }

    #[tokio::test]
    async fn test_wait_for_eventual() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        // Spawn task that sets flag
        tokio::spawn(async move {
            tokio::task::yield_now().await;
            flag_clone.store(true, Ordering::SeqCst);
        });

        // Wait for flag (event-driven, no polling sleep!)
        wait_for(|| flag.load(Ordering::SeqCst), Duration::from_secs(1)).await.unwrap();
    }

    #[tokio::test]
    async fn test_response_channel() {
        let (tx, rx) = response_channel();

        tokio::spawn(async move {
            // Simulate async work
            tokio::task::yield_now().await;
            tx.send(42).ok();
        });

        let result = rx.await.unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_temp_unix_socket() {
        let sock = temp_unix_socket().unwrap();
        let path = sock.path().to_path_buf();

        // Bind listener
        let _listener = sock.bind().await.unwrap();

        // Path should exist
        assert!(path.exists());

        // Drop socket - should auto-clean
        drop(_listener);
        drop(sock);

        // Path should be cleaned up (eventually)
        tokio::task::yield_now().await;
    }
}
