//! HTTP connection pool for substrate clients

use std::time::Duration;

/// Connection pool for HTTP clients
#[derive(Debug)]
pub struct ConnectionPool {
    pub pool: Vec<reqwest::Client>,
    pub pool_size: usize,
    pub active_connections: usize,
}

impl ConnectionPool {
    /// Create new connection pool
    pub fn new(pool_size: usize) -> Self {
        let mut pool = Vec::with_capacity(pool_size);

        for _ in 0..pool_size {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .pool_max_idle_per_host(10)
                .pool_idle_timeout(Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());
            pool.push(client);
        }

        Self {
            pool,
            pool_size,
            active_connections: 0,
        }
    }

    /// Get a client from the pool
    pub fn get_client(&mut self) -> Option<reqwest::Client> {
        if self.active_connections < self.pool_size {
            self.active_connections += 1;
            self.pool.get(self.active_connections - 1).cloned()
        } else {
            // Pool exhausted, create new client
            reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .ok()
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
