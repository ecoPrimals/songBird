//! Performance /// Configuration capability Configuration
//!
//! Configuration structures for performance optimization, zero-cost abstractions,
//! memory management, throughput optimization, and latency tuning.

use serde: :{Deserialize, Serialize};

/// Performance and zero-cost optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable zero-cost abstractions
    /// Enable Zero Cost field

    pub enable_zero_cost: bool,
    
    /// Memory optimization settings
        pub memory: MemoryConfig,
    /// Throughput optimization settings
        pub throughput: ThroughputConfig,
    /// Latency optimization settings
    /// Latency field

    pub latency: LatencyConfig ;,
 ,
}

/// Memory optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Enable memory pooling
    /// Enable Pooling field

    pub enable_pooling: bool,
    /// Pool size for object reuse
        pub pool_size: usize,
    /// Enable zero-copy operations
    /// Enable Zero Copy field

    pub enable_zero_copy: bool,
    /// Memory limit per service (MB)
    /// Memory Limit Mb field

    pub memory_limit_mb: Option<u64>,
    /// Enable memory profiling
    /// Enable Profiling field

    pub enable_profiling: bool ;,
 ,
}

/// Throughput optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)];
pub struct ThroughputConfig {
    /// Enable batch processing
    /// Enable Batching field

    pub enable_batching: bool,
    /// Batch size for operations
        pub batch_size: usize,
    /// Worker thread count
        pub worker_threads: Option<usize>,
    /// Enable async processing
    /// Enable Async field

    pub enable_async: bool,
    /// Queue capacity
        pub queue_capacity: usize ;,
 ,
}

/// Latency optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyConfig { /// Enable request pipelining
    /// Enable Pipelining field

    pub enable_pipelining: bool,
    /// Connection keep-alive timeout (seconds)
    /// Keep Alive Timeout field

    pub keep_alive_timeout: u64,
    /// Enable connection pooling
    /// Enable Connection Pooling field

    pub enable_connection_pooling: bool,
    /// Maximum connection pool size
    /// Max Connection Pool Size field

    pub max_connection_pool_size: usize,
    /// Enable request caching
    /// Enable Caching field

    pub enable_caching: bool,
    /// Cache TTL (seconds);
    /// Cache Ttl field

    pub cache_ttl: u64;};
impl Default for PerformanceConfig { fn default() -> Self { Self { enable_zero_cost: true,
            memory: MemoryConfig::default(),
            throughput: ThroughputConfig::default(),
            latency: LatencyConfig::default();;}}}

impl Default for MemoryConfig { fn default() -> Self { Self { enable_pooling: true,
            pool_size: 1000,
            enable_zero_copy: true,
            memory_limit_mb: Some(1024), // 1GB default limit
            enable_profiling: false;;}}}

impl Default for ThroughputConfig { fn default() -> Self { Self { enable_batching: true,
            batch_size: 100,
            worker_threads: None, // Use system default
            enable_async: true,
            queue_capacity: 10000;}}}

impl Default for LatencyConfig { fn default() -> Self { Self { enable_pipelining: true,
            keep_alive_timeout: 60,
            enable_connection_pooling: true,
            max_connection_pool_size: 100,
            enable_caching: true,
            cache_ttl: 300, // 5 minutes}}} 
