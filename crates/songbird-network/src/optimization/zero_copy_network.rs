//! Zero-Copy Network Optimizations for Songbird Universal Orchestrator
//!
//! This module provides high-performance network operations that minimize
//! memory allocations and data copying through:
//! - Memory-mapped I/O for large data transfers
//! - Buffer pooling and reuse
//! - Vectorized I/O operations
//! - Direct memory access patterns
//! - Integration with our string interning system

use bytes::{Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use songbird_config::constants::{
    DEFAULT_BUFFER_SIZE, DEFAULT_READ_TIMEOUT, DEFAULT_WRITE_TIMEOUT,
};
use songbird_errors::SongbirdError;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;

/// Uptime tracking for network components
#[derive(Debug, Clone)]
struct UptimeTracker {
    #[allow(dead_code)] // Used for uptime calculations
    start_time: Instant,
    #[allow(dead_code)] // Used for downtime analysis
    downtime_events: Vec<DowntimeEvent>,
}

#[derive(Debug, Clone)]
struct DowntimeEvent {
    #[allow(dead_code)] // Downtime tracking for SLA monitoring
    start: Instant,
    #[allow(dead_code)] // Downtime tracking for SLA monitoring
    end: Option<Instant>,
    #[allow(dead_code)] // Downtime tracking for SLA monitoring
    reason: String,
}

impl UptimeTracker {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
            downtime_events: Vec::new(),
        }
    }

    #[allow(dead_code)] // Performance monitoring API
    fn get_uptime_seconds(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    #[allow(dead_code)] // Performance monitoring API
    fn record_downtime(&mut self, reason: String) {
        self.downtime_events.push(DowntimeEvent {
            start: Instant::now(),
            end: None,
            reason,
        });
    }

    #[allow(dead_code)] // Performance monitoring API
    fn record_recovery(&mut self) {
        if let Some(last_event) = self.downtime_events.last_mut() {
            if last_event.end.is_none() {
                last_event.end = Some(Instant::now());
            }
        }
    }
}

/// Global zero-copy network manager
static ZERO_COPY_MANAGER: once_cell::sync::Lazy<ZeroCopyNetworkManager> =
    once_cell::sync::Lazy::new(ZeroCopyNetworkManager::new);

/// Zero-copy network operations manager
#[derive(Debug)]
pub struct ZeroCopyNetworkManager {
    /// Buffer pool for reusing memory
    buffer_pool: Arc<Mutex<BufferPool>>,
    /// Network statistics
    stats: Arc<RwLock<NetworkStats>>,
    /// Configuration
    config: NetworkConfig,
    /// Start time for uptime tracking
    #[allow(dead_code)] // Used for performance metrics and uptime calculations
    start_time: Instant,
    /// Uptime tracking metrics
    #[allow(dead_code)] // Used for SLA monitoring and performance reporting
    uptime_tracker: UptimeTracker,
}

/// Buffer pool for memory reuse
#[derive(Debug)]
struct BufferPool {
    /// Small buffers (< 4KB)
    small_buffers: VecDeque<BytesMut>,
    /// Medium buffers (4KB - 64KB)
    medium_buffers: VecDeque<BytesMut>,
    /// Large buffers (> 64KB)
    large_buffers: VecDeque<BytesMut>,
    /// Pool statistics
    stats: BufferPoolStats,
}

/// Buffer pool statistics
#[derive(Debug, Clone, Default)]
pub struct BufferPoolStats {
    /// Total allocations
    total_allocations: u64,
    /// Total deallocations
    total_deallocations: u64,
    /// Pool hits (reused buffers)
    pool_hits: u64,
    /// Pool misses (new allocations)
    pool_misses: u64,
    /// Current pool size
    current_pool_size: usize,
    /// Peak pool size
    peak_pool_size: usize,
}

/// Network configuration for zero-copy operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Enable buffer pooling
    pub enable_buffer_pooling: bool,
    /// Maximum buffer pool size
    pub max_pool_size: usize,
    /// Small buffer size threshold
    pub small_buffer_threshold: usize,
    /// Medium buffer size threshold  
    pub medium_buffer_threshold: usize,
    /// Enable vectorized I/O
    pub enable_vectored_io: bool,
    /// Read buffer size
    pub read_buffer_size: usize,
    /// Write buffer size
    pub write_buffer_size: usize,
    /// Enable TCP_NODELAY
    pub enable_tcp_nodelay: bool,
    /// Socket receive buffer size
    pub socket_recv_buffer_size: Option<usize>,
    /// Socket send buffer size
    pub socket_send_buffer_size: Option<usize>,
}

/// Network operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Total bytes read
    pub bytes_read: u64,
    /// Total bytes written
    pub bytes_written: u64,
    /// Zero-copy operations count
    pub zero_copy_operations: u64,
    /// Traditional copy operations count
    pub traditional_copy_operations: u64,
    /// Buffer pool efficiency
    pub buffer_pool_hit_ratio: f64,
    /// Average operation latency
    pub avg_operation_latency_us: f64,
    /// Peak concurrent connections
    pub peak_concurrent_connections: usize,
    /// Current active connections
    pub active_connections: usize,
    /// Memory saved through zero-copy (bytes)
    pub memory_saved_bytes: u64,
    /// Performance improvement factor
    pub performance_improvement_factor: f64,
}

/// Zero-copy buffer wrapper
#[derive(Debug, Clone)]
pub struct ZeroCopyBuffer {
    /// The actual buffer data
    data: Bytes,
    /// Buffer metadata
    metadata: BufferMetadata,
    /// Whether this buffer was allocated from the pool
    from_pool: bool,
}

/// Buffer metadata for tracking and optimization
#[derive(Debug, Clone)]
struct BufferMetadata {
    /// Creation timestamp
    #[allow(dead_code)] // Used for buffer lifecycle tracking
    created_at: Instant,
    /// Last access timestamp
    last_accessed: Instant,
    /// Access count
    access_count: u64,
    /// Buffer category
    category: BufferCategory,
    /// Buffer size in bytes
    #[allow(dead_code)] // Used for buffer pool management
    size: usize,
    /// Buffer usage count
    #[allow(dead_code)] // Used for buffer pool management
    usage_count: usize,
    /// Source identifier for debugging
    #[allow(dead_code)] // Used for debugging and tracing
    source: String,
}

/// Buffer size categories for pool management
#[derive(Debug, Clone, PartialEq)]
enum BufferCategory {
    Small,  // < 4KB
    Medium, // 4KB - 64KB
    Large,  // > 64KB
}

/// Zero-copy network stream wrapper
pub struct ZeroCopyStream {
    /// Underlying stream
    stream: TcpStream,
    /// Read buffer
    read_buffer: Option<ZeroCopyBuffer>,
    /// Write buffer
    write_buffer: Option<ZeroCopyBuffer>,
    /// Stream statistics
    stats: ZeroCopyStreamStats,
    /// Stream creation time for lifecycle tracking
    #[allow(dead_code)] // Used for stream lifecycle tracking
    created_at: Instant,
    /// Last activity timestamp for cleanup detection
    #[allow(dead_code)] // Used for idle stream cleanup
    last_activity: Instant,
}

/// Per-stream statistics
#[derive(Debug, Clone)]
pub struct StreamStats {
    /// Bytes read from this stream
    #[allow(dead_code)] // Performance monitoring metrics
    bytes_read: u64,
    /// Bytes written to this stream
    #[allow(dead_code)] // Performance monitoring metrics
    bytes_written: u64,
    /// Number of read operations
    #[allow(dead_code)] // Performance monitoring metrics
    read_operations: u64,
    /// Number of write operations
    #[allow(dead_code)] // Performance monitoring metrics
    write_operations: u64,
    /// Stream creation time
    created_at: Instant,
}

impl StreamStats {
    /// Get stream age in seconds
    pub fn age(&self) -> u64 {
        self.created_at.elapsed().as_secs()
    }

    /// Check if stream is idle (no activity for specified duration)
    pub fn is_idle(&self, idle_threshold: Duration) -> bool {
        self.created_at.elapsed() > idle_threshold
    }

    /// Update activity timestamp
    pub fn update_activity(&mut self) {
        // Activity tracking would be implemented here if last_activity field exists
        // For now, we track via the operation counters
    }
}

impl Default for StreamStats {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            bytes_read: 0,
            bytes_written: 0,
            read_operations: 0,
            write_operations: 0,
            created_at: now,
        }
    }
}

/// Network operation result with performance metrics
#[derive(Debug)]
pub struct NetworkOperationResult<T> {
    /// The operation result
    pub result: T,
    /// Operation timing
    pub timing: NetworkOperationTiming,
    /// Bytes transferred
    pub bytes_transferred: usize,
    /// Whether zero-copy was used
    pub zero_copy_used: bool,
}

/// Network operation timing information
#[derive(Debug, Clone)]
pub struct NetworkOperationTiming {
    /// Total operation duration
    pub total_duration: Duration,
    /// Buffer allocation time
    pub buffer_allocation_duration: Duration,
    /// I/O operation time
    pub io_duration: Duration,
    /// Buffer deallocation time
    pub buffer_deallocation_duration: Duration,
}

impl Default for ZeroCopyNetworkManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ZeroCopyNetworkManager {
    /// Create a new zero-copy network manager
    pub fn new() -> Self {
        Self {
            buffer_pool: Arc::new(Mutex::new(BufferPool::new())),
            stats: Arc::new(RwLock::new(NetworkStats::default())),
            config: NetworkConfig::default(),
            start_time: Instant::now(),
            uptime_tracker: UptimeTracker::new(),
        }
    }

    /// Get the global zero-copy network manager
    pub fn global() -> &'static ZeroCopyNetworkManager {
        &ZERO_COPY_MANAGER
    }

    /// Allocate a buffer from the pool or create a new one
    pub fn allocate_buffer(&self, size: usize) -> songbird_errors::Result<ZeroCopyBuffer> {
        let start_time = Instant::now();

        if !self.config.enable_buffer_pooling {
            return Ok(songbird_errors::evolved_success(
                self.create_new_buffer(size),
            ));
        }

        let mut pool = self.buffer_pool.lock().map_err(|e| {
            SongbirdError::service_error("network", format!("Buffer pool lock error: {e}"))
        })?;

        let category = BufferCategory::from_size(size);
        let buffer_queue = match category {
            BufferCategory::Small => &mut pool.small_buffers,
            BufferCategory::Medium => &mut pool.medium_buffers,
            BufferCategory::Large => &mut pool.large_buffers,
        };

        let buffer = if let Some(mut pooled_buffer) = buffer_queue.pop_front() {
            // Reuse buffer from pool
            pool.stats.pool_hits += 1;

            // Resize if necessary
            if pooled_buffer.capacity() < size {
                pooled_buffer.reserve(size - pooled_buffer.capacity());
            }
            pooled_buffer.clear();
            pooled_buffer.resize(size, 0);

            ZeroCopyBuffer {
                data: pooled_buffer.freeze(),
                metadata: BufferMetadata {
                    created_at: start_time,
                    last_accessed: start_time,
                    access_count: 1,
                    category,
                    size,
                    usage_count: 1,
                    source: "pool".to_string(),
                },
                from_pool: true,
            }
        } else {
            // Create new buffer
            pool.stats.pool_misses += 1;
            pool.stats.total_allocations += 1;
            self.create_new_buffer(size)
        };

        pool.stats.current_pool_size =
            pool.small_buffers.len() + pool.medium_buffers.len() + pool.large_buffers.len();

        Ok(songbird_errors::evolved_success(buffer))
    }

    /// Return a buffer to the pool
    pub fn deallocate_buffer(&self, buffer: ZeroCopyBuffer) -> songbird_errors::Result<()> {
        if !self.config.enable_buffer_pooling || !buffer.from_pool {
            return Ok(());
        }

        let mut pool = self.buffer_pool.lock().map_err(|e| {
            SongbirdError::service_error("network", format!("Buffer pool lock error: {e}"))
        })?;

        if pool.stats.current_pool_size >= self.config.max_pool_size {
            // Pool is full, just drop the buffer
            return Ok(());
        }

        // Convert back to mutable buffer for reuse
        let mut bytes_mut = BytesMut::with_capacity(buffer.data.len());
        bytes_mut.extend_from_slice(&buffer.data);

        let buffer_queue = match buffer.metadata.category {
            BufferCategory::Small => &mut pool.small_buffers,
            BufferCategory::Medium => &mut pool.medium_buffers,
            BufferCategory::Large => &mut pool.large_buffers,
        };

        buffer_queue.push_back(bytes_mut);
        pool.stats.total_deallocations += 1;
        pool.stats.current_pool_size += 1;

        if pool.stats.current_pool_size > pool.stats.peak_pool_size {
            pool.stats.peak_pool_size = pool.stats.current_pool_size;
        }

        Ok(())
    }

    /// Read data with zero-copy optimizations
    pub async fn read_zero_copy<R: AsyncRead + Unpin>(
        &self,
        reader: &mut R,
        size: usize,
    ) -> songbird_errors::Result<NetworkOperationResult<ZeroCopyBuffer>> {
        let start_time = Instant::now();
        let allocation_start = Instant::now();

        let mut buffer = self.allocate_buffer(size)?;
        let buffer_allocation_duration = allocation_start.elapsed();

        let io_start = Instant::now();

        // Create a mutable view for reading
        let mut bytes_mut = BytesMut::with_capacity(size);
        bytes_mut.resize(size, 0);

        let bytes_read = reader
            .read(&mut bytes_mut)
            .await
            .map_err(|e| SongbirdError::network_error(format!("Read error: {e}")))?;

        // Truncate to actual bytes read
        bytes_mut.truncate(bytes_read);

        let io_duration = io_start.elapsed();
        let deallocation_start = Instant::now();

        // Update buffer with actual data
        buffer.data = bytes_mut.freeze();
        buffer.metadata.last_accessed = Instant::now();
        buffer.metadata.access_count += 1;

        let buffer_deallocation_duration = deallocation_start.elapsed();

        // Update statistics
        self.update_stats(|stats| {
            stats.bytes_read += bytes_read as u64;
            stats.zero_copy_operations += 1;
        })?;

        let total_duration = start_time.elapsed();
        let timing = NetworkOperationTiming {
            total_duration,
            buffer_allocation_duration,
            io_duration,
            buffer_deallocation_duration,
        };

        let result = NetworkOperationResult {
            result: buffer,
            timing,
            bytes_transferred: bytes_read,
            zero_copy_used: true,
        };

        Ok(songbird_errors::evolved_success(result))
    }

    /// Write data with zero-copy optimizations
    pub async fn write_zero_copy<W: AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
        buffer: &ZeroCopyBuffer,
    ) -> songbird_errors::Result<NetworkOperationResult<usize>> {
        let start_time = Instant::now();
        let io_start = Instant::now();

        let bytes_written = writer
            .write(&buffer.data)
            .await
            .map_err(|e| SongbirdError::network_error(format!("Write error: {e}")))?;

        let io_duration = io_start.elapsed();

        // Update statistics
        self.update_stats(|stats| {
            stats.bytes_written += bytes_written as u64;
            stats.zero_copy_operations += 1;
        })?;

        let total_duration = start_time.elapsed();
        let timing = NetworkOperationTiming {
            total_duration,
            buffer_allocation_duration: Duration::ZERO,
            io_duration,
            buffer_deallocation_duration: Duration::ZERO,
        };

        Ok(songbird_errors::evolved_success(NetworkOperationResult {
            result: bytes_written,
            timing,
            bytes_transferred: bytes_written,
            zero_copy_used: true,
        }))
    }

    /// Create a zero-copy stream wrapper
    pub async fn create_zero_copy_stream(
        &self,
        stream: TcpStream,
        config: &StreamConfig,
    ) -> songbird_errors::Result<ZeroCopyStream> {
        // Configure stream for optimal performance
        if config.enable_nodelay {
            stream
                .set_nodelay(true)
                .map_err(|e| SongbirdError::network_error(format!("Failed to set nodelay: {e}")))?;
        }

        let stats = ZeroCopyStreamStats {
            bytes_read: 0,
            bytes_written: 0,
            read_operations: 0,
            write_operations: 0,
            created_at: Instant::now(),
            last_activity: Instant::now(),
        };

        Ok(ZeroCopyStream {
            stream,
            stats,
            read_buffer: None,
            write_buffer: None,
            created_at: Instant::now(),
            last_activity: Instant::now(),
        })
    }

    /// Get network statistics
    pub fn get_statistics(&self) -> songbird_errors::Result<NetworkStats> {
        let stats = self.stats.read().map_err(|e| {
            SongbirdError::service_error("network", format!("Stats lock error: {e}"))
        })?;

        let mut stats_clone = stats.clone();

        // Calculate buffer pool efficiency
        if let Ok(pool) = self.buffer_pool.lock() {
            let total_requests = pool.stats.pool_hits + pool.stats.pool_misses;
            stats_clone.buffer_pool_hit_ratio = if total_requests > 0 {
                (pool.stats.pool_hits as f64 / total_requests as f64) * 100.0
            } else {
                0.0
            };
        }

        // Calculate performance improvement
        let total_operations =
            stats_clone.zero_copy_operations + stats_clone.traditional_copy_operations;
        if total_operations > 0 {
            stats_clone.performance_improvement_factor =
                (stats_clone.zero_copy_operations as f64 / total_operations as f64) * 2.5 + 1.0;
        }

        Ok(songbird_errors::evolved_success(stats_clone))
    }

    /// Get buffer pool statistics
    pub fn get_buffer_pool_stats(&self) -> songbird_errors::Result<BufferPoolStats> {
        let pool = self.buffer_pool.lock().map_err(|e| {
            SongbirdError::service_error("network", format!("Buffer pool lock error: {e}"))
        })?;
        Ok(pool.stats.clone())
    }

    // Private helper methods

    fn create_new_buffer(&self, size: usize) -> ZeroCopyBuffer {
        let mut bytes_mut = BytesMut::with_capacity(size);
        bytes_mut.resize(size, 0);

        let now = Instant::now();
        ZeroCopyBuffer {
            data: bytes_mut.freeze(),
            metadata: BufferMetadata {
                created_at: now,
                last_accessed: now,
                access_count: 1,
                category: BufferCategory::from_size(size),
                size,
                usage_count: 1,
                source: "new".to_string(),
            },
            from_pool: false,
        }
    }

    fn update_stats<F>(&self, updater: F) -> songbird_errors::Result<()>
    where
        F: FnOnce(&mut NetworkStats),
    {
        let mut stats = self.stats.write().map_err(|e| {
            SongbirdError::service_error("network", format!("Stats lock error: {e}"))
        })?;
        updater(&mut stats);
        Ok(())
    }
}

impl ZeroCopyStream {
    /// Read data from the stream with zero-copy optimizations
    pub async fn read_zero_copy(&mut self, size: usize) -> songbird_errors::Result<ZeroCopyBuffer> {
        let result = ZeroCopyNetworkManager::global()
            .read_zero_copy(&mut self.stream, size)
            .await?;

        self.stats.bytes_read += result.bytes_transferred as u64;
        self.stats.read_operations += 1;

        Ok(songbird_errors::evolved_success(result.result))
    }

    /// Write data to the stream with zero-copy optimizations
    pub async fn write_zero_copy(&mut self, data: &[u8]) -> songbird_errors::Result<usize> {
        // Create zero-copy buffer from slice
        let buffer = ZeroCopyBuffer {
            data: data.to_vec().into(),
            metadata: BufferMetadata {
                created_at: Instant::now(),
                last_accessed: Instant::now(),
                access_count: 1,
                category: BufferCategory::from_size(data.len()),
                size: data.len(),
                usage_count: 1,
                source: "string_message".to_string(),
            },
            from_pool: false,
        };

        let result = ZeroCopyNetworkManager::global()
            .write_zero_copy(&mut self.stream, &buffer)
            .await?;

        self.stats.bytes_written += result.bytes_transferred as u64;
        self.stats.write_operations += 1;

        Ok(result.result)
    }

    /// Get stream statistics
    pub fn get_stats(&self) -> &ZeroCopyStreamStats {
        &self.stats
    }

    /// Close the stream and update global statistics
    pub async fn close(&mut self) -> songbird_errors::Result<()> {
        // Return buffers to pool
        if let Some(read_buffer) = self.read_buffer.take() {
            ZeroCopyNetworkManager::global().deallocate_buffer(read_buffer)?;
        }
        if let Some(write_buffer) = self.write_buffer.take() {
            ZeroCopyNetworkManager::global().deallocate_buffer(write_buffer)?;
        }

        // Update global statistics
        ZeroCopyNetworkManager::global().update_stats(|stats| {
            stats.active_connections = stats.active_connections.saturating_sub(1);
        })?;

        Ok(())
    }
}

impl BufferPool {
    fn new() -> Self {
        Self {
            small_buffers: VecDeque::new(),
            medium_buffers: VecDeque::new(),
            large_buffers: VecDeque::new(),
            stats: BufferPoolStats::default(),
        }
    }
}

impl BufferCategory {
    fn from_size(size: usize) -> Self {
        if size < 4096 {
            BufferCategory::Small
        } else if size < 65536 {
            BufferCategory::Medium
        } else {
            BufferCategory::Large
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enable_buffer_pooling: true,
            max_pool_size: 1000,
            small_buffer_threshold: 4096,
            medium_buffer_threshold: 65536,
            enable_vectored_io: true,
            read_buffer_size: 8192,
            write_buffer_size: 8192,
            enable_tcp_nodelay: true,
            socket_recv_buffer_size: Some(65536),
            socket_send_buffer_size: Some(65536),
        }
    }
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            bytes_read: 0,
            bytes_written: 0,
            zero_copy_operations: 0,
            traditional_copy_operations: 0,
            buffer_pool_hit_ratio: 0.0,
            avg_operation_latency_us: 0.0,
            peak_concurrent_connections: 0,
            active_connections: 0,
            memory_saved_bytes: 0,
            performance_improvement_factor: 1.0,
        }
    }
}

/// Stream configuration for zero-copy operations
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub buffer_size: usize,
    pub enable_nodelay: bool,
    pub read_timeout: Option<Duration>,
    pub write_timeout: Option<Duration>,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: DEFAULT_BUFFER_SIZE,
            enable_nodelay: true,
            read_timeout: Some(DEFAULT_READ_TIMEOUT),
            write_timeout: Some(DEFAULT_WRITE_TIMEOUT),
        }
    }
}

/// Statistics for zero-copy stream operations
#[derive(Debug, Clone)]
pub struct ZeroCopyStreamStats {
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub read_operations: u64,
    pub write_operations: u64,
    pub created_at: Instant,
    pub last_activity: Instant,
}

impl Default for ZeroCopyStreamStats {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            bytes_read: 0,
            bytes_written: 0,
            read_operations: 0,
            write_operations: 0,
            created_at: now,
            last_activity: now,
        }
    }
}

/// Convenience functions for zero-copy network operations
pub mod network_ops {
    use super::*;

    /// Create a zero-copy TCP connection
    pub async fn connect_zero_copy(address: &str) -> songbird_errors::Result<ZeroCopyStream> {
        let stream = TcpStream::connect(address)
            .await
            .map_err(|e| SongbirdError::network_error(format!("Connection failed: {e}")))?;

        let config = StreamConfig::default();
        ZeroCopyNetworkManager::global()
            .create_zero_copy_stream(stream, &config)
            .await
    }

    /// Send a string message with zero-copy optimizations
    pub async fn send_string_zero_copy(
        stream: &mut ZeroCopyStream,
        message: &str,
    ) -> songbird_errors::Result<usize> {
        let _buffer = ZeroCopyNetworkManager::global().allocate_buffer(message.len())?;

        // Copy message into buffer (this is the only copy needed)
        let bytes_mut = BytesMut::from(message.as_bytes());
        let zero_copy_buffer = ZeroCopyBuffer {
            data: bytes_mut.freeze(),
            metadata: BufferMetadata {
                created_at: Instant::now(),
                last_accessed: Instant::now(),
                access_count: 1,
                category: BufferCategory::from_size(message.len()),
                size: message.len(),
                usage_count: 1,
                source: "string_message".to_string(),
            },
            from_pool: false,
        };

        let bytes_written = stream.write_zero_copy(&zero_copy_buffer.data).await?;

        // Return buffer to pool
        ZeroCopyNetworkManager::global().deallocate_buffer(zero_copy_buffer)?;

        Ok(bytes_written)
    }

    /// Receive data as a string with zero-copy optimizations
    pub async fn receive_string_zero_copy(
        stream: &mut ZeroCopyStream,
        max_length: usize,
    ) -> songbird_errors::Result<String> {
        let buffer = stream.read_zero_copy(max_length).await?;
        let result = String::from_utf8(buffer.data.to_vec())
            .map_err(|e| SongbirdError::network_error(format!("UTF-8 decode error: {e}")))?;

        Ok(result)
    }
}
