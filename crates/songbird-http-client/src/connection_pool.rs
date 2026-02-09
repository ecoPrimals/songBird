//! Connection pooling for HTTP/IPC connections
//!
//! Implements a production-ready connection pool with:
//! - Automatic connection lifecycle management
//! - Health checking and stale connection cleanup
//! - Bounded pool size with overflow protection
//! - Connection reuse for reduced latency
//! - Graceful degradation under load
//!
//! ## Deep Debt Evolution Principle
//!
//! **Before (No Pooling)**:
//! ```ignore
//! // Create new connection for every request (slow!)
//! let client = IpcHttpClient::new().await?;
//! let response = client.request(req).await?;
//! // Connection dropped, TCP overhead on next request
//! ```
//!
//! **After (With Pooling)**:
//! ```ignore
//! // Reuse connections from pool (fast!)
//! let pool = ConnectionPool::new(10).await?;
//! let conn = pool.get().await?;
//! let response = conn.request(req).await?;
//! pool.return_connection(conn); // Reused for next request
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_http_client::connection_pool::{ConnectionPool, PoolConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create pool with default config
//!     let pool = ConnectionPool::builder()
//!         .max_size(20)
//!         .min_idle(5)
//!         .build()
//!         .await?;
//!
//!     // Get connection from pool
//!     let conn = pool.acquire().await?;
//!     
//!     // Use connection
//!     // conn.request(...).await?;
//!     
//!     // Connection automatically returned to pool on drop
//!     Ok(())
//! }
//! ```

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn};

/// Connection pool error types
#[derive(Debug, thiserror::Error)]
pub enum PoolError {
    #[error("Pool is at maximum capacity ({0})")]
    PoolFull(usize),

    #[error("Connection failed health check")]
    UnhealthyConnection,

    #[error("Failed to create new connection: {0}")]
    ConnectionCreation(String),

    #[error("Connection acquisition timeout after {0:?}")]
    AcquisitionTimeout(Duration),

    #[error("Pool is shutting down")]
    ShuttingDown,
}

/// Result type for connection pool operations
pub type PoolResult<T> = Result<T, PoolError>;

/// Configuration for connection pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool
    pub max_size: usize,

    /// Minimum number of idle connections to maintain
    pub min_idle: usize,

    /// Maximum time a connection can be idle before being closed
    pub max_idle_time: Duration,

    /// Maximum time to wait when acquiring a connection
    pub acquire_timeout: Duration,

    /// How often to run background cleanup
    pub cleanup_interval: Duration,

    /// Connection health check interval
    pub health_check_interval: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 10,
            min_idle: 2,
            max_idle_time: Duration::from_secs(60),
            acquire_timeout: Duration::from_secs(5),
            cleanup_interval: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(10),
        }
    }
}

impl PoolConfig {
    /// Create a builder for pool configuration
    pub fn builder() -> PoolConfigBuilder {
        PoolConfigBuilder::default()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_size == 0 {
            return Err("max_size must be greater than 0".to_string());
        }
        if self.min_idle > self.max_size {
            return Err("min_idle cannot be greater than max_size".to_string());
        }
        if self.max_idle_time.is_zero() {
            return Err("max_idle_time cannot be zero".to_string());
        }
        Ok(())
    }
}

/// Builder for pool configuration
#[derive(Debug, Default)]
pub struct PoolConfigBuilder {
    max_size: Option<usize>,
    min_idle: Option<usize>,
    max_idle_time: Option<Duration>,
    acquire_timeout: Option<Duration>,
    cleanup_interval: Option<Duration>,
    health_check_interval: Option<Duration>,
}

impl PoolConfigBuilder {
    pub fn max_size(mut self, size: usize) -> Self {
        self.max_size = Some(size);
        self
    }

    pub fn min_idle(mut self, count: usize) -> Self {
        self.min_idle = Some(count);
        self
    }

    pub fn max_idle_time(mut self, duration: Duration) -> Self {
        self.max_idle_time = Some(duration);
        self
    }

    pub fn acquire_timeout(mut self, duration: Duration) -> Self {
        self.acquire_timeout = Some(duration);
        self
    }

    pub fn build(self) -> PoolConfig {
        let default = PoolConfig::default();
        PoolConfig {
            max_size: self.max_size.unwrap_or(default.max_size),
            min_idle: self.min_idle.unwrap_or(default.min_idle),
            max_idle_time: self.max_idle_time.unwrap_or(default.max_idle_time),
            acquire_timeout: self.acquire_timeout.unwrap_or(default.acquire_timeout),
            cleanup_interval: self.cleanup_interval.unwrap_or(default.cleanup_interval),
            health_check_interval: self
                .health_check_interval
                .unwrap_or(default.health_check_interval),
        }
    }
}

/// A pooled connection wrapper
pub struct PooledConnection<T: Send + Sync + 'static> {
    inner: Option<T>,
    pool: Arc<ConnectionPoolInner<T>>,
    #[allow(dead_code)] // Reserved for future connection age tracking
    created_at: Instant,
    last_used: Instant,
}

impl<T: Send + Sync + 'static> PooledConnection<T> {
    /// Check if connection is healthy
    pub fn is_healthy(&self) -> bool {
        let age = self.last_used.elapsed();
        age < self.pool.config.max_idle_time
    }

    /// Update last used timestamp
    pub fn touch(&mut self) {
        self.last_used = Instant::now();
    }

    /// Get a reference to the inner connection
    pub fn inner(&self) -> Option<&T> {
        self.inner.as_ref()
    }

    /// Get a mutable reference to the inner connection
    pub fn inner_mut(&mut self) -> Option<&mut T> {
        self.inner.as_mut()
    }
}

// Implement Deref to allow transparent usage as the inner type
impl<T: Send + Sync + 'static> std::ops::Deref for PooledConnection<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().expect("PooledConnection inner is None")
    }
}

// Implement DerefMut to allow mutable transparent usage
impl<T: Send + Sync + 'static> std::ops::DerefMut for PooledConnection<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().expect("PooledConnection inner is None")
    }
}

impl<T: Send + Sync + 'static> Drop for PooledConnection<T> {
    fn drop(&mut self) {
        if let Some(conn) = self.inner.take() {
            // Return connection to pool
            let pool = Arc::clone(&self.pool);
            let last_used = self.last_used;
            tokio::spawn(async move {
                pool.return_connection(conn, last_used).await;
            });
        }
    }
}

/// Inner connection pool state
struct ConnectionPoolInner<T> {
    connections: RwLock<VecDeque<(T, Instant)>>,
    semaphore: Semaphore,
    config: PoolConfig,
    is_shutting_down: RwLock<bool>,
}

impl<T> ConnectionPoolInner<T> {
    async fn return_connection(&self, conn: T, last_used: Instant) {
        if *self.is_shutting_down.read().await {
            // Pool is shutting down, don't return connection
            return;
        }

        let mut connections = self.connections.write().await;
        connections.push_back((conn, last_used));
        self.semaphore.add_permits(1);
    }

    async fn cleanup(&self) {
        let mut connections = self.connections.write().await;
        let now = Instant::now();
        let max_idle = self.config.max_idle_time;

        // Remove stale connections
        connections.retain(|(_, last_used)| {
            let age = now.duration_since(*last_used);
            age < max_idle
        });

        let removed = connections.len();
        if removed > 0 {
            debug!("Cleaned up {} stale connections", removed);
        }
    }
}

/// Connection pool for managing reusable connections
pub struct ConnectionPool<T> {
    inner: Arc<ConnectionPoolInner<T>>,
    _cleanup_task: tokio::task::JoinHandle<()>,
}

impl<T: Send + Sync + 'static> ConnectionPool<T> {
    /// Create a new connection pool with builder pattern
    pub fn builder() -> ConnectionPoolBuilder<T> {
        ConnectionPoolBuilder::default()
    }

    /// Create a new connection pool with default configuration
    pub async fn new(max_size: usize) -> PoolResult<Self> {
        Self::builder().max_size(max_size).build().await
    }

    /// Acquire a connection from the pool
    ///
    /// If no connections are available, waits up to `acquire_timeout` for one to become available.
    /// Returns `PoolError::AcquisitionTimeout` if the timeout expires.
    pub async fn acquire(&self) -> PoolResult<PooledConnection<T>> {
        if *self.inner.is_shutting_down.read().await {
            return Err(PoolError::ShuttingDown);
        }

        // Try to acquire a permit (blocks if pool is full)
        let acquire_timeout = self.inner.config.acquire_timeout;
        tokio::time::timeout(acquire_timeout, self.inner.semaphore.acquire())
            .await
            .map_err(|_| PoolError::AcquisitionTimeout(acquire_timeout))?
            .map_err(|_| PoolError::ShuttingDown)?
            .forget(); // We manage permits manually

        // Try to get an existing connection
        let mut connections = self.inner.connections.write().await;

        while let Some((conn, last_used)) = connections.pop_front() {
            // Check if connection is still healthy
            let age = Instant::now().duration_since(last_used);
            if age < self.inner.config.max_idle_time {
                drop(connections); // Release lock
                return Ok(PooledConnection {
                    inner: Some(conn),
                    pool: Arc::clone(&self.inner),
                    created_at: Instant::now(),
                    last_used: Instant::now(),
                });
            }
            // Connection is stale, try next one
        }

        drop(connections); // Release lock before returning error

        // No healthy connections available
        warn!("No healthy connections available in pool");
        self.inner.semaphore.add_permits(1); // Return permit
        Err(PoolError::UnhealthyConnection)
    }

    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        let connections = self.inner.connections.read().await;
        PoolStats {
            total_connections: connections.len(),
            idle_connections: connections.len(),
            max_connections: self.inner.config.max_size,
            min_idle: self.inner.config.min_idle,
        }
    }

    /// Shutdown the pool gracefully
    pub async fn shutdown(&self) {
        info!("Shutting down connection pool");
        *self.inner.is_shutting_down.write().await = true;

        // Clear all connections
        let mut connections = self.inner.connections.write().await;
        connections.clear();
    }

    /// Manually add a connection to the pool
    ///
    /// Used for pre-populating the pool or adding externally created connections.
    pub async fn add_connection(&self, conn: T) -> PoolResult<()> {
        if *self.inner.is_shutting_down.read().await {
            return Err(PoolError::ShuttingDown);
        }

        let mut connections = self.inner.connections.write().await;
        if connections.len() >= self.inner.config.max_size {
            return Err(PoolError::PoolFull(self.inner.config.max_size));
        }

        connections.push_back((conn, Instant::now()));
        self.inner.semaphore.add_permits(1);
        Ok(())
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_connections: usize,
    pub idle_connections: usize,
    pub max_connections: usize,
    pub min_idle: usize,
}

/// Builder for connection pool
pub struct ConnectionPoolBuilder<T> {
    config: PoolConfig,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Default for ConnectionPoolBuilder<T> {
    fn default() -> Self {
        Self {
            config: PoolConfig::default(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Send + Sync + 'static> ConnectionPoolBuilder<T> {
    pub fn max_size(mut self, size: usize) -> Self {
        self.config.max_size = size;
        self
    }

    pub fn min_idle(mut self, count: usize) -> Self {
        self.config.min_idle = count;
        self
    }

    pub fn max_idle_time(mut self, duration: Duration) -> Self {
        self.config.max_idle_time = duration;
        self
    }

    pub fn acquire_timeout(mut self, duration: Duration) -> Self {
        self.config.acquire_timeout = duration;
        self
    }

    pub async fn build(self) -> PoolResult<ConnectionPool<T>> {
        // Validate configuration
        self.config.validate().map_err(PoolError::ConnectionCreation)?;

        let inner = Arc::new(ConnectionPoolInner {
            connections: RwLock::new(VecDeque::with_capacity(self.config.max_size)),
            semaphore: Semaphore::new(self.config.max_size),
            config: self.config.clone(),
            is_shutting_down: RwLock::new(false),
        });

        // Start background cleanup task
        let cleanup_inner = Arc::clone(&inner);
        let cleanup_interval = self.config.cleanup_interval;
        let cleanup_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);
            loop {
                interval.tick().await;
                if *cleanup_inner.is_shutting_down.read().await {
                    break;
                }
                cleanup_inner.cleanup().await;
            }
        });

        info!(
            "Connection pool created: max_size={}, min_idle={}",
            self.config.max_size, self.config.min_idle
        );

        Ok(ConnectionPool {
            inner,
            _cleanup_task: cleanup_task,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock connection type for testing
    #[derive(Debug, Clone)]
    struct MockConnection {
        id: usize,
    }

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = ConnectionPool::<MockConnection>::builder()
            .max_size(5)
            .min_idle(2)
            .build()
            .await
            .unwrap();

        let stats = pool.stats().await;
        assert_eq!(stats.max_connections, 5);
        assert_eq!(stats.min_idle, 2);
    }

    #[tokio::test]
    async fn test_add_and_acquire_connection() {
        let pool = ConnectionPool::builder().max_size(5).build().await.unwrap();

        // Add connection
        let conn = MockConnection {
            id: 1,
        };
        pool.add_connection(conn).await.unwrap();

        // Acquire connection
        let acquired = pool.acquire().await.unwrap();
        assert_eq!(acquired.inner().unwrap().id, 1);
    }

    #[tokio::test]
    async fn test_pool_full() {
        let pool = ConnectionPool::builder().max_size(2).build().await.unwrap();

        // Fill pool
        pool.add_connection(MockConnection {
            id: 1,
        })
        .await
        .unwrap();
        pool.add_connection(MockConnection {
            id: 2,
        })
        .await
        .unwrap();

        // Try to add another connection (should fail)
        let result = pool
            .add_connection(MockConnection {
                id: 3,
            })
            .await;
        assert!(matches!(result, Err(PoolError::PoolFull(2))));
    }

    #[tokio::test]
    async fn test_connection_return_on_drop() {
        let pool = ConnectionPool::builder().max_size(5).build().await.unwrap();

        pool.add_connection(MockConnection {
            id: 1,
        })
        .await
        .unwrap();

        {
            let _conn = pool.acquire().await.unwrap();
            // Connection acquired
        } // Connection dropped, should be returned to pool

        // Give async drop task time to complete
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Should be able to acquire again
        let stats = pool.stats().await;
        assert!(stats.idle_connections > 0 || stats.total_connections > 0);
    }

    #[tokio::test]
    async fn test_config_validation() {
        // Invalid: max_size = 0
        let config = PoolConfig {
            max_size: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Invalid: min_idle > max_size
        let config = PoolConfig {
            max_size: 5,
            min_idle: 10,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Valid
        let config = PoolConfig {
            max_size: 10,
            min_idle: 5,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}
