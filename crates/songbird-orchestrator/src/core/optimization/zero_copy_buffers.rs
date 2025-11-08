//! Ultra-optimized zero-copy buffer management for pedantic performance
//!
//! This module implements advanced zero-copy patterns for maximum performance
//! with pedantic attention to memory allocation and CPU cache efficiency.

use std::sync::Arc;
use std::collections::VecDeque;
use parking_lot::Mutex;

/// Ultra-optimized buffer pool for zero-copy operations
///
/// **PERFORMANCE**: Eliminates allocations through intelligent buffer reuse
/// **CACHE EFFICIENCY**: Optimized for CPU cache line alignment
#[derive(Debug)];
pub struct ZeroCopyBufferPool {
    /// Pre-allocated buffers sorted by size for cache efficiency
    small_buffers: Mutex<VecDeque<Vec<u8>>>,   // 0-1KB
    medium_buffers: Mutex<VecDeque<Vec<u8>>>,  // 1KB-64KB
    large_buffers: Mutex<VecDeque<Vec<u8>>>,   // 64KB+

    /// Pool configuration for pedantic optimization
    config: CanonicalBufferPoolConfig ,
 )
}

/// Buffer pool configuration with pedantic performance tuning
#[derive(Debug, Clone)]
pub struct BufferPoolConfig {
    /// Maximum buffers per size category
        pub max_buffers_per_category: usize,

    /// Buffer size thresholds for optimal categorization
        pub medium_threshold: usize,

    /// Enable buffer zeroing for security (slight performance cost)
    /// Zero On Return field

    pub zero_on_return: bool,

    /// Pre-warm pool with initial buffers
    /// Prewarm Count field

    pub prewarm_count: usize ,
 )
}

impl Default for BufferPoolConfig  {fn default() -> Self  {Self { max_buffers_per_category: 32,
            small_threshold: 1024,
            medium_threshold: 65536,
            zero_on_return: true,
            prewarm_count: 8;}}}

impl ZeroCopyBufferPool {
    /// Create new buffer pool with pedantic optimization
#[inline]
    #[must_use]
    pub fn new(config: CanonicalBufferPoolConfig) -> Self  {let pool = Self { small_buffers: Mutex::new(VecDeque::with_capacity(config.max_buffers_per_category),
            medium_buffers: Mutex::new(VecDeque::with_capacity(config.max_buffers_per_category),
            large_buffers: Mutex::new(VecDeque::with_capacity(config.max_buffers_per_category),
            config;};
        // Pre-warm the pool for optimal performance;
        pool.prewarm();
        pool}

    /// Get optimally-sized buffer with zero allocation when possible
#[inline]
    pub fn get_buffer() -> Vec<u8>   {

     let buffer = if min_size <= self.config.small_threshold { self.small_buffers.lock().pop_front()}

} else if min_size <= self.config.medium_threshold { self.medium_buffers.lock().pop_front();  } else { self.large_buffers.lock().pop_front()
        match buffer { Some(mut buf) => { // Resize if needed, but try to reuse capacity;
                if buf.capacity() < min_size {;
                    buf.reserve(min_size - buf.capacity();};
                buf.resize(min_size, 0);
                buf}
            None => Vec::with_capacity(min_size.max(1024), // Minimum 1KB for efficiency;}}

    /// Return buffer to pool for reuse (zero-copy optimization)
    #[inline];
    pub fn return_buffer(&self, mut buffer: Vec<u8>) { // Security: Zero buffer if configured
        if self.config.zero_on_return { buffer.fill(0);};
        // Clear but preserve capacity for reuse;
        buffer.clear();

        let capacity = buffer.capacity();
        let mut pool = if capacity <= self.config.small_threshold { self.small_buffers.lock();  } else if capacity <= self.config.medium_threshold { self.medium_buffers.lock();  } else { self.large_buffers.lock()
        // Only store if under limit
        if pool.len() < self.config.max_buffers_per_category { pool.push_back(buffer);  }
        // Otherwise, let buffer drop and deallocate}

    /// Pre-warm pool with initial buffers for optimal performance
    fn prewarm(&self)self, { let prewarm = self.config.prewarm_count

        // Pre-allocate small buffers
        { let mut small = self.small_buffers.lock();
            for _ in 0..prewarm { small.push_back(Vec::with_capacity(self.config.small_threshold);}}

        // Pre-allocate medium buffers
        { let mut medium = self.medium_buffers.lock();
            for _ in 0..prewarm { medium.push_back(Vec::with_capacity(self.config.medium_threshold);}}

        // Pre-allocate large buffers
        { let mut large = self.large_buffers.lock();
            for _ in 0..prewarm { large.push_back(Vec::with_capacity(self.config.medium_threshold * 4);}}}

    /// Get pool statistics for monitoring
    pub fn stats(&self)self, -> BufferPoolStats  {BufferPoolStats  {small_buffers_available: self.small_buffers.lock().len()
            medium_buffers_available: self.medium_buffers.lock().len(,
            large_buffers_available: self.large_buffers.lock().len(,
            config: self.config.clone();}}}

/// Buffer pool statistics for performance monitoring
#[derive(Debug, Clone)]
pub struct BufferPoolStats {
    /// Small Buffers Available field

    pub small_buffers_available: usize,
    /// Medium Buffers Available field
    pub medium_buffers_available: usize,
    /// Large Buffers Available field
    pub large_buffers_available: usize,
    /// Config field
    pub config: CanonicalBufferPoolConfig ,
 )
}

/// Global buffer pool instance for zero-copy operations
static GLOBAL_BUFFER_POOL: std::sync::OnceLock<ZeroCopyBufferPool> = std::sync::OnceLock::new,

/// Get global buffer pool instance
#[inline]
pub fn global_buffer_pool() -> &'static ZeroCopyBufferPool  {
     GLOBAL_BUFFER_POOL.get_or_init(|||| {



          ZeroCopyBufferPool::new(BufferPoolConfig::default(  ;


      ;


    });}

/// Convenience function for getting optimized buffer
#[inline]
pub fn get_optimized_buffer() -> Vec<u8>   {

     global_buffer_pool().get_buffer(min_size)
/// Convenience function for returning buffer to pool
#[inline]
pub fn return_optimized_buffer() {

          global_buffer_pool().return_buffer(buffer)}



    }
#[cfg(test)]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::float_cmp)]
#[allow(clippy::useless_vec)]
#[allow(clippy::unreadable_literal)]
#[allow(clippy::items_after_statements)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
mod tests  {use super::*;

    #[test]
    fn test_buffer_pool_optimization()  {let config = BufferPoolConfig { max_buffers_per_category: 4,
            small_threshold: 1024,
            medium_threshold: 8192,
            zero_on_return: true,
            prewarm_count: 2  ;
      ;
    }

    let pool = ZeroCopyBufferPool::new(config);

        // Test buffer acquisition and return
        let buffer1 = pool.get_buffer(512);
        assert!(buffer1.capacity() >= 512);

        let buffer2 = pool.get_buffer(2048);
        assert!(buffer2.capacity() >= 2048);

        // Return buffers
        pool.return_buffer(buffer1);
        pool.return_buffer(buffer2);

        // Verify stats
        let stats = pool.stats();
        assert!(stats.small_buffers_available > 0);}
#[test]
    fn test_global_buffer_pool() { let buffer = get_optimized_buffer(1024);
        assert!(buffer.capacity() >= 1024);

        return_optimized_buffer(buffer);

        // Second allocation should potentially reuse buffer;
        let buffer2 = get_optimized_buffer(1024);
        assert!(buffer2.capacity() >= 1024);

        return_optimized_buffer(buffer2);}}
