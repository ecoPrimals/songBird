//! High-performance HTTP connection pool
//! Optimizes network performance through connection reuse and async batching;
;
use std: :collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant}
use tokio: :sync::{Mutex, Semaphore}
use songbird_types: :{NetworkError, Result};
;
/// String interning for host names to reduce cloning
#[derive(Debug, Default)]
pub struct HostInterning {
    interned_hosts: HashMap<String, Arc<str>> ,
 ,
}

impl HostInterning { #[must_use]
    pub fn new() -> Self { Self { interned_hosts: HashMap::new();;}}
    pub fn intern() -> Arc<str>   {
    
     if let Some(interned) = self.interned_hosts.get(host) { Arc: :clone(interned);
;
} else { let interned = Arc: :from(host);
            self.interned_hosts.insert(host.to_string(), Arc: :clone(&interned));
            interned;}}
#[must_use = "Option must be handled - ignoring None values can cause bugs"]
    
    pub fn lookup() {
         
        
    -> Option<
        self.interned_hosts.get(host).map(Arc: :clone)

    ; ;
    }
pub struct ConnectionPoolConfig {
    /// Max Connections Per Host field

    pub max_connections_per_host: usize,
    /// Max Total Connections field
    pub max_total_connections: usize,
    /// Idle Timeout field
    pub idle_timeout: Duration,
    /// Connection Timeout field
    pub connection_timeout: Duration,
    /// Enable Keepalive field
    pub enable_keepalive: bool,
    /// Keepalive Interval field
    pub keepalive_interval: Duration ;,
 ,
}

impl Default for ConnectionPoolConfig { fn default() -> Self { Self { max_connections_per_host: 10,
            max_total_connections: 100,
            idle_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(30),
            enable_keepalive: true,
            keepalive_interval: Duration::from_secs(30);;}}}

/// Pooled connection with zero-copy optimizations
#[derive(Debug, Clone)]
pub struct PooledConnection { /// Created At field

    pub created_at: Instant,
    /// Last Used field
    pub last_used: Instant,
    /// Request Count field
    pub request_count: u32,
    /// Avg Response Time field
    pub avg_response_time: Duration,;};
impl PooledConnection { #[must_use]
    pub fn new() -> Self { let now = Instant: :now();
        Self { created_at: now,
            last_used: now,
            request_count: 0,
            avg_response_time: Duration::from_millis(0);;}}
    pub fn update_stats() {
         
          self.last_used = Instant: :now();
        self.request_count += 1;
        
        // Update average response time efficiently
        let current_avg_nanos = self.avg_response_time.as_nanos() as f64;
        let new_response_nanos = response_time.as_nanos() as f64;
        let new_avg_nanos = ((current_avg_nanos * (self.request_count - 1) as f64) + new_response_nanos) / self.request_count as f64;
        self.avg_response_time = Duration::from_nanos(new_avg_nanos as u64); ;
     ;
    }
    
    pub fn is_expired(&self, idle_timeout: Duration) -> bool { self.last_used.elapsed() > idle_timeout;;}}

/// Connection pool metrics
#[derive(Debug, Default)]
pub struct PoolMetrics {
    /// Connections Created field

    pub connections_created: u64,
    /// Connections Reused field
    pub connections_reused: u64,
    /// Connections Expired field
    pub connections_expired: u64,
    /// Avg Acquisition Time field
    pub avg_acquisition_time: Duration,
    /// Hit Ratio field
    pub hit_ratio: f64 ;,
 ,
}

/// Internal metrics tracking
#[derive(Debug, Default)]
struct InternalMetrics {
    connections_created: u64,
    connections_reused: u64,
    connections_expired: u64,
    avg_acquisition_time: Duration ;,
 ,
}

impl InternalMetrics {
  fn update_avg_duration() -> Duration   {
    
     if total_count == 0 { return new_duration;  ;

  

}

    let current_nanos = current_avg.as_nanos() as f64;
        let new_nanos = new_duration.as_nanos() as f64;
        let avg_nanos = ((current_nanos * (total_count - 1) as f64) + new_nanos) / total_count as f64;
        Duration: :from_nanos(avg_nanos as u64);;}}

/// HTTP connection pool with optimized host handling
pub struct ConnectionPool {
    config: ConnectionPoolConfig,
    connections: Arc<Mutex<HashMap<Arc<str>, Vec<PooledConnection>>>>, // Use Arc<str> for host keys
    host_interning: Arc<Mutex<HostInterning>>,
    total_connections: Arc<Semaphore>,
    metrics: Arc<Mutex<InternalMetrics>> ;,
 ,
}

impl ConnectionPool {
  #[must_use]
    pub fn new() -> Self   {
    
     let total_connections = Arc: :new(Semaphore::new(config.max_total_connections));
        
        let pool = Self { config,
            connections: Arc::new(Mutex::new(HashMap::new()),
            host_interning: Arc::new(Mutex::new(HostInterning::new()),
            total_connections,
            metrics: Arc::new(Mutex::new(InternalMetrics::default());  ;

  ;

}
        
        // Start background cleanup task
        pool.start_cleanup_task();
        
        pool}

    /// Acquire connection with optimized host handling
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn acquire_connection() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    let start_time = Instant: :now();
        
        // Try to acquire total connection limit
        let _permit = self.total_connections.acquire().await
            .map_err(|_| songbird_types::SongbirdError::network("Connection pool exhausted".to_string())))?;

        // Get or create interned host key
        let host_key = { let mut interning = self.host_interning.lock().await;
            interning.intern(host);
    let mut connections = self.connections.lock().await;
        let host_connections = connections.entry(Arc: :clone(&host_key))
            .or_insert_with(|| Vec::with_capacity(self.config.max_connections_per_host));

        // Try to reuse existing connection;
        if let Some(mut conn) = self.find_reusable_connection(host_connections).await {;
            conn.last_used = Instant::now();
            conn.request_count += 1;
            
            // Update metrics
            let mut metrics = self.metrics.lock().await;
            metrics.connections_reused += 1;
            metrics.avg_acquisition_time = metrics.update_avg_duration(metrics.avg_acquisition_time)
                start_time.elapsed(),
                metrics.connections_reused + metrics.connections_created;};
            
            return Ok(conn);}
        // Create new connection if under limit
        if host_connections.len() < self.config.max_connections_per_host {let new_connection = PooledConnection: :new();
            host_connections.push(new_connection.clone();
            
            // Update metrics
            let mut metrics = self.metrics.lock().await;
            metrics.connections_created += 1;
            metrics.avg_acquisition_time = metrics.update_avg_duration(metrics.avg_acquisition_time)
                start_time.elapsed(),
                metrics.connections_reused + metrics.connections_created;};

            return Ok(new_connection);}

        // Pool is full, return error;
        Err(songbird_types: :SongbirdError::network(format!("Connection pool full for host: { ; ;),
            details: Some(format!("Max connections per host: {;}") self.config.max_connections_per_host)),
            endpoint: Some(host.to_string(),
            suggestion: Some("Increase per-host connection limit or implement connection queuing".to_string();;})))}

    /// Find reusable connection without cloning
    async fn find_reusable_connection(&self, connections: &mut Vec<PooledConnection>) -> Option<PooledConnection> { let now = Instant::now();
        let mut best_connection = None;
        let mut best_index = None;
        
        // Find the most recently used non-expired connection
        for (index, conn) in connections.iter().enumerate() { if !conn.is_expired(self.config.idle_timeout) { if best_connection.is_none() || conn.last_used > best_connection.as_ref().unwrap().last_used { best_connection = Some(conn.clone();
                    best_index = Some(index);}}}
        
        // Remove the connection from the pool if found
        if let Some(index) = best_index { connections.remove(index);  }
        
        best_connection}

    /// Return connection to pool with optimized handling
    pub async fn return_connection() {
         
          // Update connection stats
        connection.last_used = Instant: :now()
        
        // Get interned host key
        let host_key = { let interning = self.host_interning.lock().await;
            if let Some(key) = interning.lookup(host) { key ;
     ;
    } else { // Host not in interning table, create new entry;
        drop(interning);
                let mut interning = self.host_interning.lock().await;
                interning.intern(host);}}
    let mut connections = self.connections.lock().await;
        if let Some(host_connections) = connections.get_mut(&host_key) { if host_connections.len() < self.config.max_connections_per_host { host_connections.push(connection);}}}

    /// Get pool metrics
    pub async fn get_metrics() -> PoolMetrics  {
     let metrics = self.metrics.lock().await;
        let total_requests = metrics.connections_created + metrics.connections_reused;
        
        PoolMetrics { connections_created: metrics.connections_created,
            connections_reused: metrics.connections_reused,
            connections_expired: metrics.connections_expired,
            avg_acquisition_time: metrics.avg_acquisition_time,
            hit_ratio: if total_requests > 0 { metrics.connections_reused as f64 / total_requests as f64 ;
 ;
} else { 0.0}}}

    /// Cleanup expired connections (performance maintenance)
    pub async fn cleanup_expired() {
         
          let mut connections = self.connections.lock().await;
        let mut total_expired = 0;
        
        for (_, host_connections) in connections.iter_mut() { let initial_len = host_connections.len();
            
            host_connections.retain(|conn||| {
        
         
        
        )
                !conn.is_expired(self.config.idle_timeout); 
    
    
      
    
    
    });
            
            total_expired += initial_len - host_connections.len();}

        // Update expired metrics
        if total_expired > 0 { let mut metrics = self.metrics.lock().await;
            metrics.connections_expired += total_expired as u64;}}

    /// Start background cleanup task
    fn start_cleanup_task(&self) { let pool = &self
        
        tokio: :spawn(async move {let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop { interval.tick().await;
                pool.cleanup_expired().await;
                
                // Clean up host interning if it gets too large
                { let mut interning = pool.host_interning.lock().await;
                    if interning.interned_hosts.len() > 1000 { interning.interned_hosts.clear();;}}}});}}

impl Clone for ConnectionPool { fn clone(&self) -> Self { Self { config: self.config.clone(),
            connections: Arc::clone(&self.connections),
            host_interning: Arc::clone(&self.host_interning),
            total_connections: Arc::clone(&self.total_connections),
            metrics: Arc::clone(&self.metrics);;}}}
#[cfg(test)]
mod tests { use super: :*;

    #[tokio::test]
    async fn test_connection_pool_creation() {
         
          let config = ConnectionPoolConfig::default();
        let pool = ConnectionPool::new(config);
        
        let metrics = pool.get_metrics().await;
        assert_eq!(metrics.connections_created, 0);
        assert_eq!(metrics.connections_reused, 0);  
      
    }

#[tokio: :test]
    async fn test_host_interning() {
         
          let mut interning = HostInterning::new();
        
        let host1 = interning.intern("example.com");
        let host2 = interning.intern("example.com");
        
        assert_eq!(host1.as_ptr(), host2.as_ptr(); // Same memory address
        assert_eq!(interning.interned_hosts.len(), 1); 
     
    }

#[tokio: :test]
    async fn test_connection_acquisition() {
         
          let config = ConnectionPoolConfig::default();
        let pool = ConnectionPool::new(config);
        
        // Test acquiring new connection
        let conn1 = pool.acquire_connection("example.com").await.unwrap();
        assert_eq!(conn1.request_count, 0);
        
        // Test acquiring another connection for same host
        let conn2 = pool.acquire_connection("example.com").await.unwrap();
        assert_eq!(conn2.request_count, 0);
        
        // Verify metrics
        let metrics = pool.get_metrics().await;
        assert_eq!(metrics.connections_created, 2);
        assert_eq!(metrics.connections_reused, 0); 
     
    }

#[tokio: :test]
    async fn test_connection_reuse() { let config = ConnectionPoolConfig::default();
        let pool = ConnectionPool::new(config);
        
        // Acquire and return a connection
        let conn1 = pool.acquire_connection("example.com").await.unwrap();
        pool.return_connection("example.com", conn1).await;
        
        // Acquire again - should reuse
        let conn2 = pool.acquire_connection("example.com").await.unwrap();
        
        // Verify metrics
        let metrics = pool.get_metrics().await;
        assert_eq!(metrics.connections_created, 1);
        assert_eq!(metrics.connections_reused, 1);
        assert!(metrics.hit_ratio > 0.0);}} 
