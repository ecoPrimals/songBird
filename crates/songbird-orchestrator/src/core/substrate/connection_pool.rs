// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP connection pool for substrate clients

use songbird_http_client::IpcHttpClient;
use std::sync::Arc;

/// Connection pool for HTTP clients
#[derive(Debug, Clone)]
pub struct ConnectionPool {
    /// Pool field
    pub pool: Vec<Arc<IpcHttpClient>>,
    /// Pool Size field
    pub pool_size: usize,
    /// Number of currently active connections
    pub active_connections: usize,
}

impl ConnectionPool {
    /// Create new connection pool
    ///
    /// # Errors
    ///
    /// Returns error if IPC client initialization fails
    pub async fn new(pool_size: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let mut pool = Vec::with_capacity(pool_size);

        for _ in 0..pool_size {
            let client = IpcHttpClient::new().await?;
            pool.push(Arc::new(client));
        }

        Ok(Self {
            pool,
            pool_size,
            active_connections: 0,
        })
    }

    /// Get a client from the pool
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]
    pub fn get_client(&mut self) -> Option<Arc<IpcHttpClient>> {
        if self.active_connections < self.pool_size {
            self.active_connections += 1;
            self.pool.get(self.active_connections - 1).cloned()
        } else {
            // Pool exhausted - client must be created async outside pool
            None
        }
    }

    /// Return a client to the pool
    pub fn return_client(&mut self) {
        if self.active_connections > 0 {
            self.active_connections -= 1;
        }
    }

    /// Get pool utilization percentage
    pub fn utilization(&self) -> f64 {
        if self.pool_size == 0 {
            0.0
        } else {
            (self.active_connections as f64 / self.pool_size as f64) * 100.0
        }
    }

    /// Check if pool has available connections
    pub fn has_available(&self) -> bool {
        self.active_connections < self.pool_size
    }

    /// Get number of available connections
    pub fn available_count(&self) -> usize {
        self.pool_size.saturating_sub(self.active_connections)
    }

    /// Reset the connection pool
    pub fn reset(&mut self) {
        self.active_connections = 0;
    }
}
