//! Metrics collection and aggregation
//!
//! This module provides comprehensive metrics collection from all primals in the ecosystem
//! using capability-based adapters that respect each primal's expertise.

pub mod capability_adapters;

// Re-export main types for convenience
pub use capability_adapters::UniversalMetricsAdapter;

// Temporary stub types for backward compatibility
use serde::{Deserialize, Serialize};
use songbird_errors::SongbirdResult;

/// Stub for ComputeMetrics (delegated to capability adapters)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeMetrics {
    pub cpu_usage_percent: f64, // Changed from cpu_usage
    pub cpu_usage: f64,         // Keep both for compatibility
    pub memory_usage: f64,
    pub load_average: f64,
    // Additional fields expected by performance module
    pub memory_available_bytes: u64,
    pub memory_usage_bytes: u64,
    pub active_containers: u32,
    pub queued_jobs: u32,
    pub performance_score: f64,
    pub zero_copy_operations_per_sec: u64, // Changed from f64 to u64
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for ComputeMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 15.0, // Default CPU usage percentage
            cpu_usage: 0.15,         // Default CPU usage as fraction
            memory_usage: 0.0,
            load_average: 0.0,
            memory_available_bytes: 8_000_000_000, // 8GB default
            memory_usage_bytes: 1_000_000_000,     // 1GB default usage
            active_containers: 0,
            queued_jobs: 0,
            performance_score: 0.8,
            zero_copy_operations_per_sec: 1000, // Changed to u64
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Trait for metrics capability adapters  
#[async_trait::async_trait]
pub trait MetricsCapabilityAdapter: Send + Sync {
    /// Get compute metrics if available
    fn get_compute_metrics(&self) -> ComputeMetrics;

    /// Collect compute metrics asynchronously
    async fn collect_compute_metrics(&self) -> SongbirdResult<ComputeMetrics>;
}

/// Implement the trait for UniversalMetricsAdapter
#[async_trait::async_trait]
impl MetricsCapabilityAdapter for UniversalMetricsAdapter {
    fn get_compute_metrics(&self) -> ComputeMetrics {
        // Default implementation - would delegate to primals
        ComputeMetrics::default()
    }

    async fn collect_compute_metrics(&self) -> SongbirdResult<ComputeMetrics> {
        // Default implementation - would collect from primals
        Ok(ComputeMetrics::default())
    }
}
