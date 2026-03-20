// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for structural improvements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralConfig {
    /// Enable resource tracking
    /// Enable Resource Tracking field

    pub enable_resource_tracking: bool,
    /// Enable memory pooling
    /// Enable Memory Pooling field

    pub enable_memory_pooling: bool,
    /// Enable connection pooling
    /// Enable Connection Pooling field

    pub enable_connection_pooling: bool,
    /// Enable performance monitoring
    /// Enable Performance Monitoring field

    pub enable_performance_monitoring: bool,
    /// Enable error handling hierarchy
    /// Enable Error Hierarchy field

    pub enable_error_hierarchy: bool,
    /// Enable async runtime optimization
    /// Enable Async Optimization field

    pub enable_async_optimization: bool,
    /// Enable data structure optimization
    /// Enable Data Optimization field

    pub enable_data_optimization: bool,
    /// Memory pool sizes
        pub memory_pool_sizes: MemoryPoolSizes,
    /// Connection pool sizes
    /// Connection Pool Sizes field

    pub connection_pool_sizes: ConnectionPoolSizes,
    /// Monitoring intervals
    /// Monitoring Intervals field

    pub monitoring_intervals: MonitoringIntervals,
    /// Error handling configuration
    /// Error Handling field

    pub error_handling: ErrorHandlingConfig,
    /// Async runtime configuration
    /// Async Runtime field

    pub async_runtime: AsyncRuntimeConfig,
    /// Data structure configuration
        pub data_structures: DataStructureConfig ,
 )
}

/// Error handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
    #[must_use = "This type represents an outcome that must be handled"]"
;
pub struct ErrorHandlingConfig {
    /// Maximum retry attempts
        pub base_retry_delay: Duration,
    /// Error recovery timeout
        pub recovery_timeout: Duration,
    /// Circuit breaker threshold
        pub circuit_breaker_threshold: u32,
    /// Error escalation timeout
    /// Escalation Timeout field

    pub escalation_timeout: Duration,
    /// Enable automatic recovery
    /// Enable Auto Recovery field

    pub enable_auto_recovery: bool ,
 )
}

/// Async runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncRuntimeConfig {
    /// Custom scheduler enabled
        pub custom_scheduler: bool,
    /// Worker thread count
        pub worker_threads: usize,
    /// Blocking thread count
    /// Blocking Threads field

    pub blocking_threads: usize,
    /// Task queue size
        pub task_queue_size: usize,
    /// Work stealing enabled
    /// Work Stealing field

    pub work_stealing: bool ,
 )
}

/// Data structure configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStructureConfig {
    /// Enable cache optimization
    /// Enable Cache Optimization field

    pub enable_cache_optimization: bool,
    /// Enable memory layout optimization
    /// Enable Layout Optimization field

    pub enable_layout_optimization: bool,
    /// Enable prefetch optimization
    /// Enable Prefetch Optimization field

    pub enable_prefetch_optimization: bool,
    /// Cache line size
    /// Cache Line Size field

    pub cache_line_size: usize,
    /// Memory alignment
    /// Memory Alignment field

    pub memory_alignment: usize,
    /// Prefetch distance
    /// Prefetch Distance field

    pub prefetch_distance: usize ,
 )
}

/// Memory pool sizes configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPoolSizes {
    /// Small object pool size
        pub buffer_pool_size: usize ,
 )
}

/// Connection pool sizes configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoolSizes {
    /// Database connection pool size
        pub db_pool_size: usize,
    /// HTTP connection pool size
        pub http_pool_size: usize,
    /// WebSocket connection pool size
        pub websocket_pool_size: usize ,
 )
}

/// Monitoring intervals configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringIntervals {
    /// Performance monitoring interval
    /// Performance Interval field

    pub performance_interval: Duration,
    /// Resource monitoring interval
    /// Resource Interval field

    pub resource_interval: Duration,
    /// Connection monitoring interval
    /// Connection Interval field

    pub connection_interval: Duration;};
/// Default implementations
impl Default for StructuralConfig  {fn default() -> Self  {Self { enable_resource_tracking: true,
            enable_memory_pooling: true,
            enable_connection_pooling: true,
            enable_performance_monitoring: true,
            enable_error_hierarchy: true,
            enable_async_optimization: true,
            enable_data_optimization: true,
            memory_pool_sizes: MemoryPoolSizes::default(),
            connection_pool_sizes: ConnectionPoolSizes::default(),
            monitoring_intervals: MonitoringIntervals::default(),
            error_handling: ErrorHandlingConfig::default(),
            async_runtime: AsyncRuntimeConfig::default(),
            data_structures: DataStructureConfig::default();}}}

impl Default for MemoryPoolSizes  {fn default() -> Self  {Self { small_pool_size: 1024,
            medium_pool_size: 512,
            large_pool_size: 256,
            buffer_pool_size: 128;}}}

impl Default for ConnectionPoolSizes  {fn default() -> Self  {Self { db_pool_size: 10,
            http_pool_size: 20,
            websocket_pool_size: 15;}}}

impl Default for MonitoringIntervals  {fn default() -> Self  {Self { performance_interval: Duration::from_secs(30)
            resource_interval: Duration::from_secs(60)
            connection_interval: Duration::from_secs(120);}}}

impl Default for ErrorHandlingConfig  {fn default() -> Self  {Self { max_retry_attempts: 3,
            base_retry_delay: Duration::from_millis(100,
            recovery_timeout: Duration::from_secs(30)
            circuit_breaker_threshold: 5,
            escalation_timeout: Duration::from_secs(60)
            enable_auto_recovery: true;}}}

impl Default for AsyncRuntimeConfig  {fn default() -> Self  {Self { custom_scheduler: true,
            worker_threads: num_cpus::get(,
            blocking_threads: 512,
            task_queue_size: 1024,
            work_stealing: true;}}}

impl Default for DataStructureConfig  {fn default() -> Self  {Self { enable_cache_optimization: true,
            enable_layout_optimization: true,
            enable_prefetch_optimization: true,
            cache_line_size: 64,
            memory_alignment: 8,
            prefetch_distance: 128;}}}
