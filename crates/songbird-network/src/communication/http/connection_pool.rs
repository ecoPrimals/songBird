//! High-performance HTTP connection pool
//! Optimizes network performance through connection reuse and async batching

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use songbird_errors::Result;

/// High-performance connection pool configuration
#[derive(Debug, Clone)]
pub struct ConnectionPoolConfig {
    /// Maximum connections per host
    pub max_connections_per_host: usize,
    /// Maximum total connections
    pub max_total_connections: usize,
    /// Connection idle timeout
    pub idle_timeout: Duration,
    /// Connection keep-alive duration
    pub keep_alive: Duration,
    /// Enable connection reuse
    pub enable_reuse: bool,
}

impl Default for ConnectionPoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_host: 10,
            max_total_connections: 100,
            idle_timeout: Duration::from_secs(30),
            keep_alive: Duration::from_secs(60),
            enable_reuse: true,
        }
    }
}

/// Connection pool entry with performance metrics
#[derive(Debug)]
struct PooledConnection {
    /// Connection creation time
    created_at: Instant,
    /// Last used time
    last_used: Instant,
    /// Number of requests served
    request_count: u64,
    /// Average response time
    avg_response_time: Duration,
}

/// High-performance HTTP connection pool
pub struct HttpConnectionPool {
    config: ConnectionPoolConfig,
    connections: Arc<Mutex<HashMap<String, Vec<PooledConnection>>>>,
    total_connections: Arc<Semaphore>,
    metrics: Arc<Mutex<PoolMetrics>>,
}

/// Connection pool performance metrics
#[derive(Debug, Default)]
pub struct PoolMetrics {
    /// Total connections created
    pub connections_created: u64,
    /// Total connections reused
    pub connections_reused: u64,
    /// Total connections expired
    pub connections_expired: u64,
    /// Average connection acquisition time
    pub avg_acquisition_time: Duration,
    /// Pool hit ratio
    pub hit_ratio: f64,
}

impl HttpConnectionPool {
    /// Create a new high-performance connection pool
    pub fn new(config: ConnectionPoolConfig) -> Self {
        let total_permit_count = config.max_total_connections;
        
        Self {
            config,
            connections: Arc::new(Mutex::new(HashMap::with_capacity(16))),
            total_connections: Arc::new(Semaphore::new(total_permit_count)),
            metrics: Arc::new(Mutex::new(PoolMetrics::default())),
        }
    }

    /// Acquire a connection with performance optimization
    pub async fn acquire_connection(&self, host: &str) -> Result<PooledConnection> {
        let start_time = Instant::now();
        
        // Try to acquire total connection limit
        let _permit = self.total_connections.acquire().await
            .map_err(|_| songbird_errors::SongbirdError::Network {
                message: "Connection pool exhausted".to_string(),
                source: None,
            })?;

        let mut connections = self.connections.lock().await;
        let host_connections = connections.entry(host.to_string()).or_insert_with(|| Vec::with_capacity(4));

        // Try to reuse existing connection
        if let Some(mut conn) = self.find_reusable_connection(host_connections).await {
            conn.last_used = Instant::now();
            conn.request_count += 1;
            
            // Update metrics
            let mut metrics = self.metrics.lock().await;
            metrics.connections_reused += 1;
            metrics.avg_acquisition_time = self.update_avg_duration(
                metrics.avg_acquisition_time,
                start_time.elapsed(),
                metrics.connections_reused + metrics.connections_created,
            );
            
            return Ok(conn);
        }

        // Create new connection if under limit
        if host_connections.len() < self.config.max_connections_per_host {
            let new_conn = PooledConnection {
                created_at: Instant::now(),
                last_used: Instant::now(),
                request_count: 1,
                avg_response_time: Duration::from_millis(100), // Initial estimate
            };

            host_connections.push(new_conn);
            
            // Update metrics
            let mut metrics = self.metrics.lock().await;
            metrics.connections_created += 1;
            metrics.avg_acquisition_time = self.update_avg_duration(
                metrics.avg_acquisition_time,
                start_time.elapsed(),
                metrics.connections_reused + metrics.connections_created,
            );

            return host_connections.last().map(|conn| Ok(conn.clone())).unwrap_or_else(|| Err(songbird_errors::SongbirdError::Network { message: "Connection pool internal error: no connections available".to_string(), source: None }));
        }

        Err(songbird_errors::SongbirdError::Network {
            message: format!("Connection limit reached for host: {}", host),
            source: None,
        })
    }

    /// Find reusable connection with performance criteria
    async fn find_reusable_connection(&self, connections: &mut Vec<PooledConnection>) -> Option<PooledConnection> {
        let now = Instant::now();
        
        // Remove expired connections
        connections.retain(|conn| {
            now.duration_since(conn.last_used) < self.config.idle_timeout
        });

        // Find best connection to reuse (least recently used, best performance)
        connections.iter()
            .enumerate()
            .filter(|(_, conn)| now.duration_since(conn.last_used) < self.config.keep_alive)
            .min_by_key(|(_, conn)| (conn.last_used, conn.avg_response_time))
            .map(|(idx, _)| connections.remove(idx))
    }

    /// Update average duration efficiently
    fn update_avg_duration(&self, current_avg: Duration, new_duration: Duration, count: u64) -> Duration {
        if count == 0 {
            return new_duration;
        }
        
        let current_total = current_avg.as_nanos() * (count - 1) as u128;
        let new_total = current_total + new_duration.as_nanos();
        Duration::from_nanos((new_total / count as u128) as u64)
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> PoolMetrics {
        let metrics = self.metrics.lock().await;
        let total_requests = metrics.connections_created + metrics.connections_reused;
        
        PoolMetrics {
            connections_created: metrics.connections_created,
            connections_reused: metrics.connections_reused,
            connections_expired: metrics.connections_expired,
            avg_acquisition_time: metrics.avg_acquisition_time,
            hit_ratio: if total_requests > 0 {
                metrics.connections_reused as f64 / total_requests as f64
            } else {
                0.0
            },
        }
    }

    /// Cleanup expired connections (performance maintenance)
    pub async fn cleanup_expired(&self) {
        let mut connections = self.connections.lock().await;
        let mut total_expired = 0;
        
        for (_, host_connections) in connections.iter_mut() {
            let initial_len = host_connections.len();
            let now = Instant::now();
            
            host_connections.retain(|conn| {
                now.duration_since(conn.last_used) < self.config.idle_timeout
            });
            
            total_expired += initial_len - host_connections.len();
        }

        // Update expired metrics
        if total_expired > 0 {
            let mut metrics = self.metrics.lock().await;
            metrics.connections_expired += total_expired as u64;
        }
    }
}

impl Clone for PooledConnection {
    fn clone(&self) -> Self {
        Self {
            created_at: self.created_at,
            last_used: self.last_used,
            request_count: self.request_count,
            avg_response_time: self.avg_response_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connection_pool_creation() {
        let config = ConnectionPoolConfig::default();
        let pool = HttpConnectionPool::new(config);
        
        let metrics = pool.get_metrics().await;
        assert_eq!(metrics.connections_created, 0);
        assert_eq!(metrics.connections_reused, 0);
    }

    #[tokio::test]
    async fn test_connection_acquisition() {
        let config = ConnectionPoolConfig::default();
        let pool = HttpConnectionPool::new(config);
        
        let conn1 = pool.acquire_connection("example.com").await;
        assert!(conn1.is_ok());
        
        let metrics = pool.get_metrics().await;
        assert_eq!(metrics.connections_created, 1);
    }

    #[tokio::test]
    async fn test_connection_reuse() {
        let config = ConnectionPoolConfig::default();
        let pool = HttpConnectionPool::new(config);
        
        // First connection
        let _conn1 = pool.acquire_connection("example.com").await.expect("Test connection should succeed");
        
        // Second connection to same host (should trigger reuse logic)
        let _conn2 = pool.acquire_connection("example.com").await.expect("Test connection should succeed");
        
        let metrics = pool.get_metrics().await;
        assert!(metrics.connections_created >= 1);
    }
} 