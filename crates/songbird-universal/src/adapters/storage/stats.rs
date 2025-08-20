//! # Storage Performance Statistics
//!
//! Performance tracking and metrics for storage operations.

use std::sync::atomic::{AtomicU64, Ordering};
// std::time::Instant not used in this focused module

/// Storage performance statistics
#[derive(Debug, Default)]
pub struct StorageStats {
    pub operations_total: AtomicU64,
    pub operations_successful: AtomicU64,
    pub operations_failed: AtomicU64,
    pub total_latency_ms: AtomicU64,
    pub bytes_transferred: AtomicU64,
}

impl StorageStats {
    /// Create new storage statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a successful operation
    pub fn record_success(&self, latency: std::time::Duration, bytes: u64) {
        self.operations_total.fetch_add(1, Ordering::Relaxed);
        self.operations_successful.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ms
            .fetch_add(latency.as_millis() as u64, Ordering::Relaxed);
        self.bytes_transferred.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record a failed operation
    pub fn record_failure(&self, latency: std::time::Duration) {
        self.operations_total.fetch_add(1, Ordering::Relaxed);
        self.operations_failed.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ms
            .fetch_add(latency.as_millis() as u64, Ordering::Relaxed);
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        let total = self.operations_total.load(Ordering::Relaxed);
        if total == 0 {
            return 100.0;
        }
        let successful = self.operations_successful.load(Ordering::Relaxed);
        (successful as f64 / total as f64) * 100.0
    }

    /// Get average latency in milliseconds
    pub fn average_latency_ms(&self) -> f64 {
        let total = self.operations_total.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        let total_latency = self.total_latency_ms.load(Ordering::Relaxed);
        total_latency as f64 / total as f64
    }

    /// Get total bytes transferred
    pub fn total_bytes(&self) -> u64 {
        self.bytes_transferred.load(Ordering::Relaxed)
    }

    /// Get throughput in bytes per second (approximation)
    pub fn throughput_bps(&self) -> f64 {
        let total_latency_s = self.total_latency_ms.load(Ordering::Relaxed) as f64 / 1000.0;
        if total_latency_s == 0.0 {
            return 0.0;
        }
        self.total_bytes() as f64 / total_latency_s
    }
}
