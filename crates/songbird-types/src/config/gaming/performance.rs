// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Gaming performance tuning, benchmarks, and `QoS`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Gaming performance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingPerformanceSettings {
    /// Target FPS for gaming
    pub target_fps: u32,
    /// Buffer size for gaming operations
    pub buffer_size: usize,
    /// Enable low latency mode
    /// Low Latency field
    pub low_latency: bool,
}

impl Default for GamingPerformanceSettings {
    fn default() -> Self {
        Self {
            target_fps: 60,
            buffer_size: 8192,
            low_latency: true,
        }
    }
}

/// Gaming optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingOptimizationConfig {
    /// Enable optimizations
    /// Enabled field
    pub enabled: bool,
    /// CPU optimization
    /// Cpu Optimization field
    pub cpu_optimization: bool,
    /// Memory optimization
    /// Memory Optimization field
    pub memory_optimization: bool,
}

impl Default for GamingOptimizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cpu_optimization: true,
            memory_optimization: true,
        }
    }
}

/// Gaming performance configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GamingPerformanceConfig {
    /// Performance settings
    /// Settings field
    pub settings: GamingPerformanceSettings,
    /// Optimization configuration
    /// Optimization field
    pub optimization: GamingOptimizationConfig,
}

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Enable benchmarking
    /// Enabled field
    pub enabled: bool,
    /// Benchmark interval
    /// Interval field
    pub interval: Duration,
    /// Number of benchmark iterations
    /// Iterations field
    pub iterations: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval: Duration::from_secs(60),
            iterations: 10,
        }
    }
}

/// Quality of Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QoSConfig {
    /// Enable `QoS`
    /// Enabled field
    pub enabled: bool,
    /// Priority levels
    pub priority_levels: u8,
    /// Bandwidth allocation
    pub bandwidth_allocation: HashMap<String, u64>,
}

impl Default for QoSConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            priority_levels: 3,
            bandwidth_allocation: HashMap::new(),
        }
    }
}
