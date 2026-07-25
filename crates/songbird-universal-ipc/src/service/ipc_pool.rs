// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! UDS connection pool for `capability.call` dispatch.
//!
//! Eliminates per-request connect/disconnect overhead when dispatching
//! JSON-RPC calls to local provider sockets. Connections are kept alive
//! and reused across requests to the same socket path.
//!
//! Pool size per socket is controlled by `SONGBIRD_IPC_POOL_SIZE` (default: 4).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::{debug, trace};

use songbird_types::IpcStream;

const DEFAULT_POOL_SIZE: usize = 4;
const MAX_IDLE_DURATION: Duration = Duration::from_secs(60);

struct PooledConnection {
    stream: IpcStream,
    idle_since: Instant,
}

/// Per-socket-path connection pool.
pub struct IpcConnectionPool {
    pools: Arc<Mutex<HashMap<String, Vec<PooledConnection>>>>,
    max_per_path: usize,
}

impl IpcConnectionPool {
    /// Create a new pool, reading `SONGBIRD_IPC_POOL_SIZE` for max connections per path.
    #[must_use]
    pub fn new() -> Self {
        let max_per_path = songbird_process_env::var("SONGBIRD_IPC_POOL_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_POOL_SIZE);

        debug!(max_per_path, "IPC connection pool initialized");

        Self {
            pools: Arc::new(Mutex::new(HashMap::new())),
            max_per_path,
        }
    }

    /// Acquire a connection to `socket_path`. Reuses an idle connection if
    /// available, otherwise creates a new one.
    ///
    /// # Errors
    ///
    /// Returns an error if connection cannot be established.
    pub async fn acquire(&self, socket_path: &str) -> Result<IpcStream, String> {
        let mut pools = self.pools.lock().await;

        if let Some(bucket) = pools.get_mut(socket_path) {
            while let Some(conn) = bucket.pop() {
                if conn.idle_since.elapsed() < MAX_IDLE_DURATION {
                    trace!(socket_path, "Reusing pooled IPC connection");
                    return Ok(conn.stream);
                }
                trace!(socket_path, "Discarding expired pooled connection");
            }
        }

        drop(pools);

        trace!(socket_path, "Creating new IPC connection");
        IpcStream::connect(socket_path)
            .await
            .map_err(|e| format!("Cannot connect to provider at {socket_path}: {e}"))
    }

    /// Return a connection to the pool for reuse. If the pool is full or the
    /// connection is considered broken, it is dropped silently.
    pub async fn release(&self, socket_path: &str, stream: IpcStream) {
        let mut pools = self.pools.lock().await;
        let bucket = pools.entry(socket_path.to_string()).or_default();

        if bucket.len() < self.max_per_path {
            bucket.push(PooledConnection {
                stream,
                idle_since: Instant::now(),
            });
            trace!(socket_path, pool_size = bucket.len(), "Connection returned to pool");
        } else {
            trace!(socket_path, "Pool full — dropping connection");
        }
    }

    /// Execute a JSON-RPC request over a pooled connection. On IO failure,
    /// retries once with a fresh connection.
    ///
    /// # Errors
    ///
    /// Returns an error if both the pooled attempt and retry fail.
    pub async fn execute_jsonrpc(
        &self,
        socket_path: &str,
        request_bytes: &[u8],
        timeout: Duration,
    ) -> Result<String, String> {
        let stream = tokio::time::timeout(timeout, self.acquire(socket_path))
            .await
            .map_err(|_| format!("Timeout acquiring connection to {socket_path}"))??;

        match self.try_execute(socket_path, stream, request_bytes, timeout).await {
            Ok((response, stream)) => {
                self.release(socket_path, stream).await;
                Ok(response)
            }
            Err(first_err) => {
                debug!(
                    socket_path,
                    error = %first_err,
                    "Pooled connection failed — retrying with fresh connection after backoff"
                );

                // Brief backoff to handle transient provider restarts (bearDog, etc.)
                tokio::time::sleep(Duration::from_millis(50)).await;

                let fresh = IpcStream::connect(socket_path)
                    .await
                    .map_err(|e| format!("Retry connect failed to {socket_path}: {e}"))?;

                match self.try_execute(socket_path, fresh, request_bytes, timeout).await {
                    Ok((response, stream)) => {
                        self.release(socket_path, stream).await;
                        Ok(response)
                    }
                    Err(retry_err) => Err(format!(
                        "Provider at {socket_path} unreachable after retry: {retry_err}"
                    )),
                }
            }
        }
    }

    async fn try_execute(
        &self,
        socket_path: &str,
        stream: IpcStream,
        request_bytes: &[u8],
        timeout: Duration,
    ) -> Result<(String, IpcStream), String> {
        let (reader, mut writer) = tokio::io::split(stream);

        tokio::time::timeout(timeout, writer.write_all(request_bytes))
            .await
            .map_err(|_| format!("Timeout writing to provider at {socket_path}"))?
            .map_err(|e| format!("Write error to provider: {e}"))?;

        let mut buf_reader = BufReader::new(reader);
        let mut response_line = String::new();
        tokio::time::timeout(timeout, buf_reader.read_line(&mut response_line))
            .await
            .map_err(|_| format!("Timeout reading from provider at {socket_path}"))?
            .map_err(|e| format!("Read error from provider: {e}"))?;

        if response_line.is_empty() {
            return Err(format!("Empty response from provider at {socket_path}"));
        }

        let stream = buf_reader.into_inner().unsplit(writer);
        Ok((response_line, stream))
    }

    /// Evict all idle connections older than [`MAX_IDLE_DURATION`].
    pub async fn evict_stale(&self) {
        let mut pools = self.pools.lock().await;
        let mut total_evicted = 0usize;

        for bucket in pools.values_mut() {
            let before = bucket.len();
            bucket.retain(|c| c.idle_since.elapsed() < MAX_IDLE_DURATION);
            total_evicted += before - bucket.len();
        }

        pools.retain(|_, bucket| !bucket.is_empty());

        if total_evicted > 0 {
            debug!(evicted = total_evicted, "Evicted stale IPC connections");
        }
    }
}

impl Default for IpcConnectionPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn default_pool_size() {
        let pool = IpcConnectionPool::new();
        assert_eq!(pool.max_per_path, DEFAULT_POOL_SIZE);
    }

    #[tokio::test]
    async fn evict_stale_clears_empty_buckets() {
        let pool = IpcConnectionPool::new();
        pool.evict_stale().await;
        let pools = pool.pools.lock().await;
        assert!(pools.is_empty());
    }
}
