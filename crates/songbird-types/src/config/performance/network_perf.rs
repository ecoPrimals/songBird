// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Network-facing performance and buffer settings.

use serde::{Deserialize, Serialize};

/// Network performance configuration with optimization levels
///
/// This struct provides comprehensive network performance tuning options
/// including optimization levels, buffer management, and connection pooling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPerformanceConfig {
    /// Network optimization level determining performance characteristics
    pub optimization_level: NetworkOptimizationLevel,
    /// Buffer configuration for network operations
    pub buffer_config: BufferConfig,
    /// Maximum number of concurrent connections
    pub max_connections: u32,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Enable connection keepalive
    pub keepalive_enabled: bool,
    /// Keepalive interval in seconds
    pub keepalive_interval_secs: u64,
}

/// Network optimization levels for different performance characteristics
///
/// Each level provides different trade-offs between performance, memory usage,
/// and CPU utilization to match various deployment scenarios.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum NetworkOptimizationLevel {
    /// Disabled optimization for minimal resource usage
    ///
    /// Use this level when resource conservation is more important than performance.
    /// Suitable for resource-constrained environments or testing scenarios.
    Disabled,

    /// Basic optimization with moderate performance improvements
    ///
    /// Provides a balanced approach with reasonable performance gains
    /// while maintaining low resource overhead. Good for most deployments.
    Basic,

    /// Aggressive optimization for maximum performance
    ///
    /// Enables all performance optimizations including advanced buffer pooling,
    /// connection multiplexing, and zero-copy operations. Use in high-throughput
    /// production environments where performance is critical.
    #[default]
    Aggressive,
}

/// Buffer configuration for network and I/O operations
///
/// This struct provides comprehensive buffer management settings including
/// size limits, pooling options, and memory optimization strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferConfig {
    /// Initial buffer size in bytes
    pub initial_size: usize,
    /// Maximum buffer size in bytes
    pub max_size: usize,
    /// Number of buffers to pre-allocate in the pool
    pub pool_size: usize,
    /// Enable buffer pooling for reuse
    pub enable_pooling: bool,
    /// Enable zero-copy optimizations where possible
    pub enable_zero_copy: bool,
}

impl Default for NetworkPerformanceConfig {
    fn default() -> Self {
        Self {
            optimization_level: NetworkOptimizationLevel::Aggressive,
            buffer_config: BufferConfig::default(),
            max_connections: 1000,
            connection_timeout_ms: 30000,
            keepalive_enabled: true,
            keepalive_interval_secs: 60,
        }
    }
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            initial_size: 1024,    // 1KB
            max_size: 1024 * 1024, // 1MB
            pool_size: 10,
            enable_pooling: true,
            enable_zero_copy: true,
        }
    }
}
