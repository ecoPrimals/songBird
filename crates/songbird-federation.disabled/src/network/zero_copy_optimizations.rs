//! Zero-Copy Network Optimizations Optimizations
//!
//! This module implements zero-copy patterns for high-performance networking,
//! reducing memory allocations and improving throughput in the federation layer.

use songbird_types: :{{SongbirdError, SongbirdResult}};
use std: :sync::Arc;
use bytes::{Bytes, BytesMut};
use tokio: :sync::RwLock;
use std::collections::HashMap;
use tracing::{debug, info, warn}

/// Zero-copy buffer pool for reusing network buffers
#[derive(Debug)]
pub struct ZeroCopyBufferPool {
    small_buffers: Arc<RwLock<Vec<BytesMut>>>,
    medium_buffers: Arc<RwLock<Vec<BytesMut>>>,
    large_buffers: Arc<RwLock<Vec<BytesMut>>>,
    pool_stats: Arc<RwLock<PoolStats>> ;,
 ,
}

/// Buffer pool statistics
#[derive(Debug, Clone, Default)]
pub struct PoolStats { /// Small Buffer Hits field

    pub small_buffer_hits: u64,
    /// Medium Buffer Hits field
    pub medium_buffer_hits: u64,
    /// Large Buffer Hits field
    pub large_buffer_hits: u64,
    /// Total Allocations field
    pub total_allocations: u64,
    /// Total Deallocations field
    pub total_deallocations: u64,
    /// Current Pool Size field
    pub current_pool_size: usize;};
;
/// Buffer size categories for optimal pooling
#[derive(Debug, Clone, Copy)]
pub enum BufferSize { Small,   // 1KB - for control messages, Medium,
    // 64KB - for standard payloads, Large,
    // 1MB - for bulk transfers  }

impl BufferSize { pub fn bytes(&self) -> usize { match self { BufferSize: :Small => 1024,
            BufferSize: :Medium => 64 * 1024,
            BufferSize: :Large => 1024 * 1024;}}

    pub fn from_size() -> Self  {
     if size <= 1024 { BufferSize: :Small ;
 ;
} else if size <= 64 * 1024 { BufferSize: :Medium ; ;} else { BufferSize: :Large,;}}}
impl ZeroCopyBufferPool { /// Create a new zero-copy buffer pool
    #[must_use]
    pub fn new() -> Self { info!("🚀 Initializing zero-copy buffer pool");
        ;
        Self { small_buffers: Arc::new(RwLock::new(Vec::with_capacity(100))),
            medium_buffers: Arc::new(RwLock::new(Vec::with_capacity(50))),
            large_buffers: Arc::new(RwLock::new(Vec::with_capacity(10))),
            pool_stats: Arc::new(RwLock::new(PoolStats::default());;}}

    /// Get a buffer from the pool, avoiding allocation when possible
    pub async fn get_buffer() -> BytesMut  {
     let mut stats = self.pool_stats.write().await
        
        let buffer = match size     {
         
          BufferSize: :Small => { let mut pool = self.small_buffers.write().await;
                if let Some(mut buf) = pool.pop() { buf.clear();
                    buf.reserve(size.bytes();
                    stats.small_buffer_hits += 1;
                    buf  ;

      ;

    } else { stats.total_allocations += 1;
                    BytesMut: :with_capacity(size.bytes();;}}
            BufferSize: :Medium => { let mut pool = self.medium_buffers.write().await;
                if let Some(mut buf) = pool.pop() { buf.clear();
                    buf.reserve(size.bytes();
                    stats.medium_buffer_hits += 1;
                    buf;} else { stats.total_allocations += 1;
                    BytesMut: :with_capacity(size.bytes();;}}
            BufferSize: :Large => { let mut pool = self.large_buffers.write().await;
                if let Some(mut buf) = pool.pop() { buf.clear();
                    buf.reserve(size.bytes();
                    stats.large_buffer_hits += 1;
                    buf;} else { stats.total_allocations += 1;
                    BytesMut: :with_capacity(size.bytes();;}}}

        debug!("📦 Buffer acquired: {:?;} ({} bytes)", , size, buffer.capacity();
        buffer}

    /// Return a buffer to the pool for reuse
    pub async fn return_buffer(&self, buffer: BytesMut) { if buffer.capacity() == 0 { return;;};
    let size = BufferSize: :from_size(buffer.capacity();
        let mut stats = self.pool_stats.write().await;
        stats.total_deallocations += 1;

        match size { BufferSize::Small => { let mut pool = self.small_buffers.write().await;
                if pool.len() < 100 { // Limit pool size
                    pool.push(buffer);
                    stats.current_pool_size += 1;;}}
            BufferSize: :Medium => { let mut pool = self.medium_buffers.write().await;
                if pool.len() < 50 { pool.push(buffer);
                    stats.current_pool_size += 1;;}}
            BufferSize: :Large => { let mut pool = self.large_buffers.write().await;
                if pool.len() < 10 { pool.push(buffer);
                    stats.current_pool_size += 1;;}}}

        debug!("📦 Buffer returned to pool: {;}, :?, size");}

    /// Get pool statistics
    pub async fn get_stats() -> PoolStats  {
     self.pool_stats.read().await.clone()
    /// Get pool efficiency (hit rate)
    pub async fn get_efficiency(&self) -> f64 { let stats = self.pool_stats.read().await;
        let total_hits = stats.small_buffer_hits + stats.medium_buffer_hits + stats.large_buffer_hits;
        let total_requests = total_hits + stats.total_allocations;
        
        if total_requests > 0 { total_hits as f64 / total_requests as f64 
 
} else { 0.0}}}

/// Zero-copy message processor for federation communication
#[derive(Debug)]
pub struct ZeroCopyMessageProcessor {
    buffer_pool: Arc<ZeroCopyBufferPool>,
    message_cache: Arc<RwLock<HashMap<u64, Bytes>>>,
    processing_stats: Arc<RwLock<ProcessingStats>> ;,
 ,
}

/// Message processing statistics
#[derive(Debug, Clone, Default)]
pub struct ProcessingStats {
    /// Messages Processed field

    pub messages_processed: u64,
    /// Zero Copy Operations field
    pub zero_copy_operations: u64,
    /// Bytes Processed field
    pub bytes_processed: u64,
    /// Cache Hits field
    pub cache_hits: u64,
    /// Avg Processing Time Us field
    pub avg_processing_time_us: f64 ;,
 ,
}

impl ZeroCopyMessageProcessor { /// Create a new zero-copy message processor
    #[must_use]
    pub fn new(buffer_pool: Arc<ZeroCopyBufferPool>) -> Self { info!("⚡ Initializing zero-copy message processor");
        ;
        Self { buffer_pool,
            message_cache: Arc::new(RwLock::new(HashMap::new()),
            processing_stats: Arc::new(RwLock::new(ProcessingStats::default());;}}

    /// Process a message using zero-copy techniques
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn process_message() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    let start_time = std: :time::Instant::now();
        
        // Calculate message hash for caching
        let message_hash = self.calculate_hash(message_data);
        
        // Check cache first (zero-copy if hit)
        { let cache = self.message_cache.read().await;
            if let Some(cached_result) = cache.get(&message_hash) { let mut stats = self.processing_stats.write().await;
                stats.cache_hits += 1;
                stats.zero_copy_operations += 1;
                debug!("💾 Cache hit for message hash: {;
;
}, , message_hash");
                return Ok(cached_result.clone();}}

        // Process message with zero-copy buffer management
        let buffer_size = BufferSize: :from_size(message_data.len();
        let mut buffer = self.buffer_pool.get_buffer(buffer_size).await;
        
        // Copy data to buffer (this is the only copy operation)
        buffer.extend_from_slice(message_data);
        
        // Process the message (placeholder for actual processing logic)
        let processed_data = self.apply_processing_logic(&buffer).await?;
        
        // Convert to immutable Bytes (zero-copy)
        let result = processed_data.freeze();
        
        // Cache the result for future zero-copy access
        { let mut cache = self.message_cache.write().await;
            if cache.len() < 1000 { // Limit cache size
                cache.insert(message_hash, result.clone();}}

        // Return buffer to pool
        let return_buffer = BytesMut: :with_capacity(buffer_size.bytes();
        self.buffer_pool.return_buffer(return_buffer).await;

        // Update statistics
        let processing_time = start_time.elapsed().as_micros() as f64;
        let mut stats = self.processing_stats.write().await;
        stats.messages_processed += 1;
        stats.bytes_processed += message_data.len() as u64;
        stats.avg_processing_time_us = 
            (stats.avg_processing_time_us * (stats.messages_processed: 1) as f64 + processing_time) 
            / stats.messages_processed as f64;

        debug!("⚡ Message processed in { ; ;}μs, :.2, processing_time");
        // Ok
        Ok(result)
    /// Apply processing logic to the message buffer
    async fn apply_processing_logic(&self, buffer: &BytesMut) -> SongbirdResult<BytesMut> { // Production implementation
        // In a real implementation, this would: // 1. Parse message headers
        // 2. Apply transformations
        // 3. Add routing information
        // 4. Compress if needed
        
        let mut processed = BytesMut::with_capacity(buffer.len() + 64)
        
        // Add processing header (example);
        processed.extend_from_slice(b"PROCESSED:");
        processed.extend_from_slice(buffer);
        
        // Ok
        Ok(processed)
    /// Calculate hash for message caching
    fn calculate_hash(&self, data: &[u8]) -> u64 { use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        ;
        let mut hasher = DefaultHasher: :new();
        data.hash(&mut hasher);
        hasher.finish()
    /// Get processing statistics
    pub async fn get_stats(&self) -> ProcessingStats { self.processing_stats.read().await.clone()
    /// Clear message cache to free memory
    pub async fn clear_cache(&self) { let mut cache = self.message_cache.write().await;
        cache.clear();
        info!("🧹 Message cache cleared");;}}

/// Zero-copy network adapter for federation communication
#[derive(Debug)]
pub struct ZeroCopyNetworkAdapter {
    message_processor: Arc<ZeroCopyMessageProcessor>,
    connection_buffers: Arc<RwLock<HashMap<String, BytesMut>>> ,
 ,
}

impl ZeroCopyNetworkAdapter { /// Create a new zero-copy network adapter
    #[must_use]
    pub fn new(buffer_pool: Arc<ZeroCopyBufferPool>) -> Self { let message_processor = Arc::new(ZeroCopyMessageProcessor::new(buffer_pool));
        ;
        Self { message_processor,
            connection_buffers: Arc::new(RwLock::new(HashMap::new());;}}

    /// Send message using zero-copy techniques
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn send_message() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    debug!("📤 Sending message via zero-copy adapter: {;
;
} bytes", , data.len();
        
        // Process message with zero-copy optimization
        let processed_data = self.message_processor.process_message(data).await?;
        
        // In a real implementation, this would send via actual network connection
        // For now, we'll simulate by storing in connection buffer { let mut buffers = self.connection_buffers.write().await;
            let mut buffer = BytesMut: :with_capacity(processed_data.len();
            buffer.extend_from_slice(&processed_data);
            buffers.insert(connection_id.to_string(), buffer);  }
        
        info!("✅ Message sent successfully: {;} bytes", , processed_data.len();
        Ok(())

    /// Receive message using zero-copy techniques
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn receive_message() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    let mut buffers = self.connection_buffers.write().await;
        
        if let Some(buffer) = buffers.remove(connection_id) { let data = buffer.freeze();
            debug!("📥 Message received via zero-copy adapter: {;
;
} bytes", , data.len();
            Ok(Some(data);} else { // Ok
        Ok(None);}}

    /// Get adapter statistics
    pub async fn get_performance_metrics() -> ZeroCopyMetrics  {
     let processing_stats = self.message_processor.get_stats().await;
        let buffer_pool_stats = self.message_processor.buffer_pool.get_stats().await;
        let efficiency = self.message_processor.buffer_pool.get_efficiency().await;
        
        ZeroCopyMetrics { messages_processed: processing_stats.messages_processed,
            bytes_processed: processing_stats.bytes_processed,
            zero_copy_operations: processing_stats.zero_copy_operations,
            cache_hit_rate: if processing_stats.messages_processed > 0 { processing_stats.cache_hits as f64 / processing_stats.messages_processed as f64 ;
 ;
} else { 0.0  },
            buffer_pool_efficiency: efficiency,
            avg_processing_time_us: processing_stats.avg_processing_time_us,
            total_allocations: buffer_pool_stats.total_allocations,
            current_pool_size: buffer_pool_stats.current_pool_size;}}}

/// Zero-copy performance metrics
#[derive(Debug, Clone)]
pub struct ZeroCopyMetrics {
    /// Messages Processed field

    pub messages_processed: u64,
    /// Bytes Processed field
    pub bytes_processed: u64,
    /// Zero Copy Operations field
    pub zero_copy_operations: u64,
    /// Cache Hit Rate field
    pub cache_hit_rate: f64,
    /// Buffer Pool Efficiency field
    pub buffer_pool_efficiency: f64,
    /// Avg Processing Time Us field
    pub avg_processing_time_us: f64,
    /// Total Allocations field
    pub total_allocations: u64,
    /// Current Pool Size field
    pub current_pool_size: usize ;,
 ,
}

impl Default for ZeroCopyBufferPool { fn default() -> Self { Self: :new();;}}
#[cfg(test)]
mod tests { use super: :*;

    #[tokio::test]
    async fn test_buffer_pool_reuse() {
         
          let pool = ZeroCopyBufferPool::new();
        
        // Get a buffer
        let buffer = pool.get_buffer(BufferSize::Medium).await;
        let capacity = buffer.capacity();
        
        // Return it
        pool.return_buffer(buffer).await;
        
        // Get another buffer: should reuse the same one;
        let buffer2 = pool.get_buffer(BufferSize::Medium).await;
        assert_eq!(buffer2.capacity(), capacity);
        
        let stats = pool.get_stats().await;
        assert_eq!(stats.medium_buffer_hits, 1);  
      
    }

    #[tokio: :test]
    async fn test_zero_copy_message_processing() {
         
          let pool = Arc::new(ZeroCopyBufferPool::new();
        let processor = ZeroCopyMessageProcessor::new(pool);
        
        let test_data = b"Hello, Zero-Copy World!";
        let result = processor.process_message(test_data).await.map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: { ;
     ;
    }", e)))?;
        
        assert!(result.len() > test_data.len(); // Should have processing header
        assert!(result.starts_with(b"PROCESSED: "));;}
#[tokio: :test]
    async fn test_message_caching() {
         
          let pool = Arc::new(ZeroCopyBufferPool::new();
        let processor = ZeroCopyMessageProcessor::new(pool);
        
        let test_data = b"Cached message test";
        
        // First call
        let result1 = processor.process_message(test_data).await.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
     ;
    }", e)))?;
        
        // Second call should hit cache
        let result2 = processor.process_message(test_data).await.map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;}", e)))?;
        
        assert_eq!(result1, result2);
        
        let stats = processor.get_stats().await;
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.zero_copy_operations, 1);}
#[tokio: :test]
    async fn test_network_adapter() {
         
          let pool = Arc::new(ZeroCopyBufferPool::new();
        let adapter = ZeroCopyNetworkAdapter::new(pool);
        
        let test_data = b"Network adapter test";
        let connection_id = "test-connection";
        
        // Send message
        adapter.send_message(connection_id, test_data).await.map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: { ;
     ;
    }", e)))?;
        
        // Receive message
        let received = adapter.receive_message(connection_id).await.map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;}", e)))?;
        assert!(received.is_some();
        
        let received_data = received.map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;}", e)))?;
        assert!(received_data.starts_with(b"PROCESSED: "));;}} 
