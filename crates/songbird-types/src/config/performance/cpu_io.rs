// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! CPU and I/O performance tuning types.

use serde::{Deserialize, Serialize};

/// CPU optimization flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuOptimizationFlags {
    /// Enable SIMD optimizations
    pub simd: bool,
    /// Enable branch prediction optimizations
    pub branch_prediction: bool,
    /// Enable CPU profiling
    pub profiling: bool,
}

/// CPU optimization configuration - consolidated from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuOptimizationConfig {
    /// Enable CPU optimizations
    pub enabled: bool,
    /// Target CPU architecture optimizations
    pub target_cpu: Option<String>,
    /// CPU optimization flags
    pub flags: CpuOptimizationFlags,
    /// CPU cache optimization level (1-3)
    pub cache_optimization_level: u8,
}

impl Default for CpuOptimizationFlags {
    fn default() -> Self {
        Self {
            simd: true,
            branch_prediction: true,
            profiling: false,
        }
    }
}

impl Default for CpuOptimizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            target_cpu: None, // Auto-detect
            flags: CpuOptimizationFlags::default(),
            cache_optimization_level: 2,
        }
    }
}

/// I/O optimization flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoOptimizationFlags {
    /// Enable asynchronous I/O
    pub async_io: bool,
    /// Enable direct I/O (bypass OS cache)
    pub direct_io: bool,
    /// Enable I/O batching
    pub batching: bool,
}

/// I/O performance configuration - consolidated from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoPerformanceConfig {
    /// Enable I/O optimizations
    pub enabled: bool,
    /// I/O buffer size in KB
    pub buffer_size_kb: usize,
    /// I/O optimization flags
    pub flags: IoOptimizationFlags,
    /// I/O queue depth
    pub queue_depth: u32,
    /// Batch size for I/O operations
    pub batch_size: usize,
}

impl Default for IoOptimizationFlags {
    fn default() -> Self {
        Self {
            async_io: true,
            direct_io: false,
            batching: true,
        }
    }
}

impl Default for IoPerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size_kb: 64,
            flags: IoOptimizationFlags::default(),
            queue_depth: 32,
            batch_size: 16,
        }
    }
}
