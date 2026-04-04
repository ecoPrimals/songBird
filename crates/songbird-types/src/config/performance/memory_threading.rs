// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Memory and threading performance settings.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// **CANONICAL**: Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalMemoryConfig {
    /// Enable memory optimization
    /// Optimization Enabled field
    pub optimization_enabled: bool,
    /// Memory pool size in
    pub pool_size_mb: usize,
    /// Enable memory compaction
    /// Compaction Enabled field
    pub compaction_enabled: bool,
    /// Garbage collection threshold
    pub gc_threshold_mb: usize,
    /// Memory monitoring interval
    /// Monitoring Interval field
    pub monitoring_interval: Duration,
}

impl Default for CanonicalMemoryConfig {
    fn default() -> Self {
        Self {
            optimization_enabled: true,
            pool_size_mb: 512,
            compaction_enabled: true,
            gc_threshold_mb: 256,
            monitoring_interval: Duration::from_secs(60),
        }
    }
}

/// **CANONICAL**: Threading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalThreadingConfig {
    /// Enable thread pool optimization
    /// Optimization Enabled field
    pub optimization_enabled: bool,
    /// Number of worker threads (0 = auto-detect)
    /// Worker Threads field
    pub worker_threads: usize,
    /// Enable work-stealing scheduler
    /// Work Stealing field
    pub work_stealing: bool,
    /// Thread stack size in
    pub stack_size_kb: usize,
    /// Thread affinity enabled
    /// Affinity Enabled field
    pub affinity_enabled: bool,
}

impl Default for CanonicalThreadingConfig {
    fn default() -> Self {
        Self {
            optimization_enabled: true,
            worker_threads: 0, // Auto-detect based on CPU cores
            work_stealing: true,
            stack_size_kb: 2048, // 2MB stack
            affinity_enabled: false,
        }
    }
}
