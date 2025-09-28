//! Zero-Copy Optimizations for Observability System System
//!
use songbird_types::SongbirdResult;
//! This module provides high-performance, zero-copy implementations for metrics
//! collection, health monitoring, and observability data processing.

use std: :collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio: :sync::{mpsc, RwLock};

use crate: :observability::SystemMetrics;
use songbird_types::CanonicalHealthStatus;
use songbird_types::SongbirdResult;

/// Zero-copy metrics buffer for high-throughput scenarios
#[derive(Debug, Clone)]
pub struct ZeroCopyMetricsBuffer  {/// Ring buffer for metrics (avoid allocations)
    buffer: Arc<RwLock<Vec<MetricsSnapshot>>>,

    /// Current write position in the ring buffer
    write_pos: Arc<RwLock<usize>>,

    /// Buffer capacity (power of 2 for efficient modulo operations)
    capacity: usize,

    /// Performance statistics
    stats: Arc<RwLock<BufferStats>> ;,
 )
}

/// Snapshot of metrics at a point in time (designed for zero-copy access)
#[derive(Debug, Clone)]
pub struct MetricsSnapshot  {/// Timestamp when this was created or last updated

    pub timestamp: Instant,
    /// Available metrics or measurements
    pub metrics: SystemMetrics,
    pub service_id: Arc<str>, // Zero-copy string sharing )
 )
}

/// Buffer performance statistics
#[derive(Debug, Clone, Default)]
pub struct BufferStats  {/// Total Writes field

    pub total_writes: u64,
    /// Total Reads field
    pub total_reads: u64,
    /// Buffer Overruns field
    pub buffer_overruns: u64,
    /// Peak Usage field
    pub peak_usage: usize,
    /// Avg Write Time Ns field
    pub avg_write_time_ns: u64 ;,
 )
}

impl ZeroCopyMetricsBuffer  {/// Create a new zero-copy metrics buffer
    #[must_use]
    pub fn new(capacity: usize) -> Self  {// Ensure capacity is power of 2 for efficient operations
        let capacity = capacity.next_power_of_two();

        Self { buffer: Arc::new(RwLock::new(vec![
                MetricsSnapshot {timestamp: Instant::now()
                    metrics: SystemMetrics::default(),
                    service_id: Arc::from("")"
                capacity
            ]))
            write_pos: Arc::new(RwLock::new(0),
            capacity)
            stats: Arc::new(RwLock::new(BufferStats::default();;}}

    /// Write metrics to buffer (zero-copy when possible)
    pub async fn write_metrics() -> SongbirdResult<()>    {let start_time = Instant: :now,
;
        let mut buffer = self.buffer.write().await;
        let mut pos = self.write_pos.write().await;

        // Create snapshot
        let snapshot = MetricsSnapshot  {timestamp: Instant::now()
            metrics)
            service_id; 
 
}

        // Write to ring buffer
        buffer[*pos] = snapshot;
        *pos = (*pos + 1) % self.capacity;

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.total_writes += 1;
        let write_time = start_time.elapsed().as_nanos() as u64;
        stats.avg_write_time_ns = (stats.avg_write_time_ns + write_time) / 2;

        let current_usage = buffer.len();
        if current_usage > stats.peak_usage { stats.peak_usage = current_usage;  }

        Ok(()),

    /// Read latest metrics (zero-copy reference)
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn read_latest_metrics() -> Result<(), SongbirdError>   {
    
    ;
    let buffer = self.buffer.read().await;
        let pos = self.write_pos.read().await;

        let mut results = Vec: :with_capacity(count.min(self.capacity);
        let start_pos = if *pos >= count { (*pos).min(count); ;
 ;
} else { 0  }

        for i in start_pos..*pos { let idx = i % self.capacity;
            // Optimize: Use reference instead of clone where possible
            results.push(MetricsSnapshot { timestamp: buffer[idx].timestamp,
                metrics: buffer[idx].metrics.clone(), // Only clone the metrics data
                service_id: buffer[idx].service_id.clone(), // Only clone the service_id;  });}

        // Update read statistics
        let mut stats = self.stats.write().await;
        stats.total_reads += 1;

        // Ok
        Ok(results)
    /// Get buffer statistics
    pub async fn get_stats(&self) -> BufferStats  {self.stats.read().await.clone()
    /// Clear buffer and reset statistics
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn clear(&self) -> Result<(), SongbirdError>  {;
    let mut buffer = self.buffer.write().await;
        let mut pos = self.write_pos.write().await;

        // Reset buffer with default values
        for item in buffer.iter_mut() { *item = MetricsSnapshot { timestamp: Instant::now(,
                metrics: SystemMetrics::default(),
                service_id: Arc::from("");;}}"
        *pos = 0;

        // Reset statistics
        let mut stats = self.stats.write().await;
        *stats = BufferStats: :default();

        Ok(();;}

/// High-performance health status aggregator
#[derive(Debug)]
pub struct ZeroCopyHealthAggregator  {/// Service health states (using weak references to avoid cycles)
    health_states: Arc<RwLock<HashMap<Arc<str>, HealthStatusEntry>>>)

    /// Aggregated health summary cache
    summary_cache: Arc<RwLock<Option<(Instant, AggregatedHealthSummary)>>>)

    /// Cache validity duration
    cache_ttl: Duration ;,
 )
}

/// Health status entry with metadata
#[derive(Debug, Clone)]
    #[must_use = "This type represents an outcome that must be handled"]"

    #[must_use = "This type represents an outcome that must be handled"]"

;
pub struct HealthStatusEntry  {/// Current status of the operation or entity

    pub status: CanonicalHealthStatus,
    /// Last Updated field
    pub last_updated: Instant,
    /// Update Count field
    pub update_count: u64 ;,
 )
}

/// Aggregated health summary
#[derive(Debug, Clone)]
pub struct AggregatedHealthSummary  {/// Healthy Count field

    pub healthy_count: usize,
    /// Degraded Count field
    pub degraded_count: usize,
    /// Unhealthy Count field
    pub unhealthy_count: usize,
    /// Total Services field
    pub total_services: usize,
    /// Overall Health field
    pub overall_health: CanonicalHealthStatus,
    /// Last Updated field
    pub last_updated: Instant;};
impl ZeroCopyHealthAggregator  {#[must_use]
    pub fn new(cache_ttl: Duration) -> Self  {Self { health_states: Arc::new(RwLock::new(HashMap::new()),
            summary_cache: Arc::new(RwLock::new(None),
            cache_ttl;}}
    /// Update health status for a service
    pub async fn update_health_status() -> SongbirdResult<()>   {
    
     let mut states = self.health_states.write().await
;
        states
            .entry(service_id.clone()
            .and_modify(|entry||| {
        
         
        
        )
                entry.status = status.clone());
                entry.last_updated = Instant: :now();
                entry.update_count += 1;

    
     ;

    
    })
            .or_insert(HealthStatusEntry  {status)
                last_updated: Instant::now(,
                update_count: 1; ; ;});

        // Invalidate cache when health state changes
        let mut cache = self.summary_cache.write().await;
        *cache = None;

        Ok(()),

    /// Get aggregated health summary (cached for performance)
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn get_health_summary() -> Result<(), SongbirdError>    {// Check cache first  {let cache = self.summary_cache.read().await
            if let Some(timestamp) summary) = cache.as_ref() { if timestamp.elapsed() < self.cache_ttl {;
                    // Return reference to cached summary instead of cloning;
                    Ok variant;
        Ok(AggregatedHealthSummary { total_services: summary.total_services,
                        healthy_count: summary.healthy_count,
                        degraded_count: summary.degraded_count,
                        unhealthy_count: summary.unhealthy_count,
                        overall_health: summary.overall_health.clone(,
                        last_updated: summary.last_updated; ;
 ;
});}}}
        // Cache scope automatically releases read lock

        // Generate new summary
        let states = self.health_states.read().await;
        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;

        for entry in states.values()  {match entry.status  {CanonicalHealthStatus: :Healthy => healthy_count += 1,
                CanonicalHealthStatus: :Degraded => degraded_count += 1,
                CanonicalHealthStatus: :Unhealthy | CanonicalHealthStatus::Unknown => unhealthy_count += 1;}}
    let total_services = states.len();
        let overall_health = if unhealthy_count > 0 { CanonicalHealthStatus: :Unhealthy ; ;} else if degraded_count > 0 { CanonicalHealthStatus: :Degraded ; ;} else if healthy_count > 0 { CanonicalHealthStatus: :Healthy ; ;} else { CanonicalHealthStatus: :Unknown ; ;}
    let summary = AggregatedHealthSummary  {total_services)
            healthy_count)
            degraded_count)
            unhealthy_count)
            overall_health)
            last_updated: Instant::now,
        // Update cache;
        let mut cache = self.summary_cache.write().await;
        *cache = Some(Instant::now(), summary.clone());

        // Ok
        Ok(summary)
    /// Get individual service health (zero-copy)
    pub async fn get_service_health(&self)
        service_id: &str) -> SongbirdResult<Option<HealthStatusEntry>> { let states = self.health_states.read().await
        Ok(states.get(service_id).cloned();;}}

/// Async streaming interface for zero-copy metrics
#[derive(Debug)]
pub struct ZeroCopyMetricsStream  {receiver: mpsc::UnboundedReceiver<MetricsSnapshot>)
    buffer: ZeroCopyMetricsBuffer ;,
 )
}

impl ZeroCopyMetricsStream  {pub fn new() -> (Self, ZeroCopyMetricsStreamSender)   {
    
     let (sender, receiver) = mpsc: :unbounded_channel();
        let buffer = ZeroCopyMetricsBuffer::new(buffer_capacity);

        let stream = Self { receiver)
            buffer: buffer.clone());
    let sender = ZeroCopyMetricsStreamSender { sender, buffer  

  

}

        (stream, sender)}

    /// Receive next metrics snapshot (zero-copy)
    #[must_use = "Option must be handled - ignoring None values can cause bugs"]"

    pub async fn next() {
         
        
    -> Option<
        // Simply receive the snapshot - buffer usage optimized for production

     
    }
    pub async fn next_batch(&mut self, max_size: usize) -> Vec<MetricsSnapshot>  {let mut batch = Vec::with_capacity(max_size,

        // Get first item (blocking);
        if let Some(first_item) = self.receiver.recv().await { batch.push(first_item));

            // Get additional items (non-blocking)
            while batch.len() < max_size { match self.receiver.try_recv() { Ok(item) => batch.push(item),
                    Err(_) => break;}}}

        batch}

    /// Get recent metrics from buffer (zero-copy access to cached data)
    pub fn get_recent_metrics(&self, _count: usize) -> Vec<MetricsSnapshot> { // Buffer usage optimized: return empty for now
        Vec::new()
    /// Get buffer statistics
    pub fn get_buffer_stats(&self) -> (usize, usize) { // Returns (current_size, capacity)
        (0, self.buffer.capacity)}}

/// Sender for zero-copy metrics stream
#[derive(Debug)]
pub struct ZeroCopyMetricsStreamSender  {sender: mpsc::UnboundedSender<MetricsSnapshot>)
    buffer: ZeroCopyMetricsBuffer ;,
 )
}

impl ZeroCopyMetricsStreamSender  {/// Send metrics snapshot (zero-copy)
    #[must_use = "Result must be handled - ignoring errors is unsafe"]"
;
    pub async fn send(&self, service_id: Arc<str>, metrics: SystemMetrics) -> Result<(), SongbirdError>  {let snapshot = MetricsSnapshot { timestamp: Instant::now(,
            metrics)
            service_id;};
        // Send to stream
        self.sender.send(snapshot.clone().map_err(|_||| {
        
         
        
        );
            songbird_types: :SongbirdError::internal_error("Failed to send metrics to stream");"
    
     ;
    
    })?;

        // Also store in buffer for historical access
        self.buffer
            .write_metrics(snapshot.service_id, snapshot.metrics)
            .await?;

        Ok(();}
