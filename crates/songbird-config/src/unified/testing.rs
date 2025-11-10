//! # 🧪 Unified Testing Configuration
//!
//! **MIGRATION COMPLETE**: All testing configurations unified
//!
//! This module provides unified configuration for testing, chaos engineering)
//! and validation systems across the Songbird Universal Orchestrator.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Unified experiment configuration - **MIGRATED FROM CHAOS ENGINEERING**
// ✅ CONSOLIDATED: Re-export from songbird-test-utils
pub use songbird_test_utils::chaos_engineering::config::ExperimentConfig;

/// Unified network fault injection configuration - **MIGRATED**
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkFaultConfig {
    /// Inject latency (milliseconds)
    pub latency_ms: Option<u64>,
    /// Packet loss percentage (0-100)
    pub packet_loss_percent: Option<f64>,
    /// Bandwidth limit (bytes per second}
    pub bandwidth_limit_bps: Option<u64>,
    /// Network partitioning enabled
    pub partition_enabled: bool,
}

/// Unified service failure configuration - **MIGRATED**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceFailureConfig {
    /// Service failure rate (0.0-1.0)
    pub failure_rate: f64,
    /// Mean time to failure (seconds)
    pub mean_time_to_failure: Duration,
    /// Mean time to recovery (seconds}
    pub mean_time_to_recovery: Duration,
}

/// Unified resource constraint configuration - **MIGRATED**
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceConstraintConfig {
    /// CPU usage limit (0.0-1.0)
    pub cpu_limit: Option<f64>,
    /// Memory usage limit (bytes)
    pub memory_limit_bytes: Option<u64>,
    /// Disk I/O limit (bytes per second}
    pub disk_io_limit_bps: Option<u64>,
}

/// Unified Byzantine failure configuration - **MIGRATED**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByzantineFailureConfig {
    /// Percentage of Byzantine nodes (0.0-1.0}
    pub byzantine_node_percent: f64,
    /// Message corruption enabled
    pub message_corruption: bool,
    /// Timing attack simulation enabled
    pub timing_attacks: bool,
}

/// Unified performance degradation configuration - **MIGRATED**
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDegradationConfig {
    /// Response time multiplier (1.0 = normal)
    pub response_time_multiplier: f64,
    /// Throughput reduction factor (0.0-1.0)
    pub throughput_reduction: f64,
    /// Error rate increase (0.0-1.0}
    pub error_rate_increase: f64,
}

impl Default for ServiceFailureConfig {
    fn default() -> Self {
        Self {
            failure_rate: 0.01,                              // 1% failure rate
            mean_time_to_failure: Duration::from_secs(3600), // 1 hour
            mean_time_to_recovery: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl Default for ByzantineFailureConfig {
    fn default() -> Self {
        Self {
            byzantine_node_percent: 0.33, // Up to 1/3 Byzantine nodes
            message_corruption: false,
            timing_attacks: false,
        }
    }
}

impl Default for PerformanceDegradationConfig {
    fn default() -> Self {
        Self {
            response_time_multiplier: 1.0,
            throughput_reduction: 0.0,
            error_rate_increase: 0.0,
        }
    }
}
