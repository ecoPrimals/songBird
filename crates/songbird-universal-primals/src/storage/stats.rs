//! Storage statistics and performance monitoring

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Storage operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// Total number of read operations
    pub total_reads: u64,
    /// Total number of write operations
    pub total_writes: u64,
    /// Total number of delete operations
    pub total_deletes: u64,
    /// Total number of failed operations
    pub total_failures: u64,
    /// Average read latency in milliseconds
    pub avg_read_latency_ms: f64,
    /// Average write latency in milliseconds
    pub avg_write_latency_ms: f64,
    /// Average delete latency in milliseconds
    pub avg_delete_latency_ms: f64,
    /// Total bytes read
    pub total_bytes_read: u64,
    /// Total bytes written
    pub total_bytes_written: u64,
    /// Current error rate (0.0 to 1.0)
    pub current_error_rate: f64,
    /// Provider-specific performance statistics
    pub provider_stats: HashMap<String, ProviderPerformanceStats>,
    /// Last updated timestamp
    pub last_updated: SystemTime,
}

impl Default for StorageStats {
    fn default() -> Self {
        Self {
            total_reads: 0,
            total_writes: 0,
            total_deletes: 0,
            total_failures: 0,
            avg_read_latency_ms: 0.0,
            avg_write_latency_ms: 0.0,
            avg_delete_latency_ms: 0.0,
            total_bytes_read: 0,
            total_bytes_written: 0,
            current_error_rate: 0.0,
            provider_stats: HashMap::new(),
            last_updated: SystemTime::now(),
        }
    }
}

/// Provider performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPerformanceStats {
    pub provider_name: String,
    pub operations_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub avg_latency_ms: f64,
    pub last_operation_time: SystemTime,
}

impl Default for ProviderPerformanceStats {
    fn default() -> Self {
        Self {
            provider_name: String::new(),
            operations_count: 0,
            success_count: 0,
            failure_count: 0,
            avg_latency_ms: 0.0,
            last_operation_time: SystemTime::now(),
        }
    }
}

/// Ecosystem-wide health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemHealthMetrics {
    /// Overall system health score (0.0 to 1.0)
    pub overall_health_score: f64,
    /// Number of healthy providers
    pub healthy_providers: u32,
    /// Total number of providers
    pub total_providers: u32,
    /// Average response time across all providers
    pub avg_response_time_ms: f64,
    /// Current throughput (operations per second)
    pub current_throughput_ops_per_sec: f64,
    /// Peak throughput in the last hour
    pub peak_throughput_ops_per_sec: f64,
    /// Error rate across all providers
    pub overall_error_rate: f64,
    /// Storage capacity utilization
    pub capacity_utilization_percent: f64,
    /// Network bandwidth utilization
    pub bandwidth_utilization_percent: f64,
    /// Active connections count
    pub active_connections: u32,
    /// Queue depth (pending operations)
    pub queue_depth: u32,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// CPU utilization percentage
    pub cpu_utilization_percent: f64,
    /// Disk I/O operations per second
    pub disk_iops: f64,
    /// Network I/O bytes per second
    pub network_io_bytes_per_sec: f64,
    /// Last metrics update time
    pub last_updated: SystemTime,
}

impl Default for EcosystemHealthMetrics {
    fn default() -> Self {
        Self {
            overall_health_score: 1.0,
            healthy_providers: 0,
            total_providers: 0,
            avg_response_time_ms: 0.0,
            current_throughput_ops_per_sec: 0.0,
            peak_throughput_ops_per_sec: 0.0,
            overall_error_rate: 0.0,
            capacity_utilization_percent: 0.0,
            bandwidth_utilization_percent: 0.0,
            active_connections: 0,
            queue_depth: 0,
            memory_usage_bytes: 0,
            cpu_utilization_percent: 0.0,
            disk_iops: 0.0,
            network_io_bytes_per_sec: 0.0,
            last_updated: SystemTime::now(),
        }
    }
}

impl StorageStats {
    /// Record a successful read operation
    pub fn record_read(&mut self, latency: Duration, bytes_read: u64) {
        self.total_reads += 1;
        self.total_bytes_read += bytes_read;
        let total_reads = self.total_reads;
        Self::update_avg_latency(&mut self.avg_read_latency_ms, latency, total_reads);
        self.last_updated = SystemTime::now();
        self.update_error_rate();
    }

    /// Record a successful write operation
    pub fn record_write(&mut self, latency: Duration, bytes_written: u64) {
        self.total_writes += 1;
        self.total_bytes_written += bytes_written;
        let total_writes = self.total_writes;
        Self::update_avg_latency(&mut self.avg_write_latency_ms, latency, total_writes);
        self.last_updated = SystemTime::now();
        self.update_error_rate();
    }

    /// Record a successful delete operation
    pub fn record_delete(&mut self, latency: Duration) {
        self.total_deletes += 1;
        let total_deletes = self.total_deletes;
        Self::update_avg_latency(&mut self.avg_delete_latency_ms, latency, total_deletes);
        self.last_updated = SystemTime::now();
        self.update_error_rate();
    }

    /// Record a failed operation
    pub fn record_failure(&mut self) {
        self.total_failures += 1;
        self.last_updated = SystemTime::now();
        self.update_error_rate();
    }

    /// Update average latency using running average
    fn update_avg_latency(current_avg: &mut f64, new_latency: Duration, operation_count: u64) {
        let new_latency_ms = new_latency.as_millis() as f64;
        if operation_count == 1 {
            *current_avg = new_latency_ms;
        } else {
            *current_avg = (*current_avg * (operation_count - 1) as f64 + new_latency_ms)
                / operation_count as f64;
        }
    }

    /// Update current error rate
    fn update_error_rate(&mut self) {
        let total_operations =
            self.total_reads + self.total_writes + self.total_deletes + self.total_failures;
        if total_operations > 0 {
            self.current_error_rate = self.total_failures as f64 / total_operations as f64;
        }
    }

    /// Get total operations count
    pub fn total_operations(&self) -> u64 {
        self.total_reads + self.total_writes + self.total_deletes
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.total_operations() + self.total_failures;
        if total > 0 {
            self.total_operations() as f64 / total as f64
        } else {
            1.0
        }
    }

    /// Get throughput (operations per second) based on a time window
    pub fn throughput_ops_per_sec(&self, time_window: Duration) -> f64 {
        let elapsed = self
            .last_updated
            .duration_since(SystemTime::now() - time_window)
            .unwrap_or(Duration::from_secs(1));

        self.total_operations() as f64 / elapsed.as_secs_f64()
    }

    /// Update provider statistics
    pub fn update_provider_stats(
        &mut self,
        provider_name: String,
        success: bool,
        latency: Duration,
    ) {
        let stats = self
            .provider_stats
            .entry(provider_name.clone())
            .or_insert_with(|| ProviderPerformanceStats {
                provider_name: provider_name.clone(),
                ..Default::default()
            });

        stats.operations_count += 1;
        if success {
            stats.success_count += 1;
        } else {
            stats.failure_count += 1;
        }

        let latency_ms = latency.as_millis() as f64;
        stats.avg_latency_ms = (stats.avg_latency_ms * (stats.operations_count - 1) as f64
            + latency_ms)
            / stats.operations_count as f64;
        stats.last_operation_time = SystemTime::now();
    }

    /// Get provider statistics
    pub fn get_provider_stats(&self, provider_name: &str) -> Option<&ProviderPerformanceStats> {
        self.provider_stats.get(provider_name)
    }

    /// Get all provider statistics
    pub fn get_all_provider_stats(&self) -> &HashMap<String, ProviderPerformanceStats> {
        &self.provider_stats
    }

    /// Reset all statistics
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl ProviderPerformanceStats {
    /// Get provider success rate
    pub fn success_rate(&self) -> f64 {
        if self.operations_count > 0 {
            self.success_count as f64 / self.operations_count as f64
        } else {
            1.0
        }
    }

    /// Get provider error rate
    pub fn error_rate(&self) -> f64 {
        if self.operations_count > 0 {
            self.failure_count as f64 / self.operations_count as f64
        } else {
            0.0
        }
    }

    /// Check if provider is healthy based on recent performance
    pub fn is_healthy(&self, max_error_rate: f64, max_latency_ms: f64) -> bool {
        self.error_rate() <= max_error_rate && self.avg_latency_ms <= max_latency_ms
    }
}

impl EcosystemHealthMetrics {
    /// Update health score based on current metrics
    pub fn update_health_score(&mut self) {
        let mut score = 1.0;

        // Factor in error rate (higher error rate = lower score)
        score *= (1.0 - self.overall_error_rate).max(0.0);

        // Factor in response time (higher latency = lower score)
        if self.avg_response_time_ms > 1000.0 {
            score *= 0.5; // Significant penalty for high latency
        } else if self.avg_response_time_ms > 500.0 {
            score *= 0.8;
        }

        // Factor in provider availability
        if self.total_providers > 0 {
            let provider_availability = self.healthy_providers as f64 / self.total_providers as f64;
            score *= provider_availability;
        }

        // Factor in capacity utilization
        if self.capacity_utilization_percent > 90.0 {
            score *= 0.7; // High utilization is concerning
        } else if self.capacity_utilization_percent > 80.0 {
            score *= 0.9;
        }

        self.overall_health_score = score.clamp(0.0, 1.0);
        self.last_updated = SystemTime::now();
    }

    /// Check if the ecosystem is healthy
    pub fn is_healthy(&self) -> bool {
        self.overall_health_score >= 0.8
    }

    /// Get health status as a string
    pub fn health_status(&self) -> &'static str {
        if self.overall_health_score >= 0.9 {
            "Excellent"
        } else if self.overall_health_score >= 0.8 {
            "Good"
        } else if self.overall_health_score >= 0.6 {
            "Fair"
        } else if self.overall_health_score >= 0.4 {
            "Poor"
        } else {
            "Critical"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_stats_operations() {
        let mut stats = StorageStats::default();

        stats.record_read(Duration::from_millis(50), 1024);
        stats.record_write(Duration::from_millis(100), 2048);
        stats.record_delete(Duration::from_millis(25));
        stats.record_failure();

        assert_eq!(stats.await.total_reads, 1);
        assert_eq!(stats.await.total_writes, 1);
        assert_eq!(stats.await.total_deletes, 1);
        assert_eq!(stats.await.total_failures, 1);
        assert_eq!(stats.await.total_bytes_read, 1024);
        assert_eq!(stats.await.total_bytes_written, 2048);
        assert_eq!(stats.success_rate(), 0.75); // 3 success out of 4 total
    }

    #[test]
    fn test_provider_stats() {
        let mut stats = StorageStats::default();

        stats.update_provider_stats("provider1".to_string(), true, Duration::from_millis(100));
        stats.update_provider_stats("provider1".to_string(), false, Duration::from_millis(200));

        let provider_stats = stats.get_provider_stats("provider1").map_err(|e| {
            songbird_errors::SongbirdError::operation_error(
                "operation_failed",
                format!("Operation failed: {}", e),
            )
        })?;
        assert_eq!(provider_stats.operations_count, 2);
        assert_eq!(provider_stats.success_count, 1);
        assert_eq!(provider_stats.failure_count, 1);
        assert_eq!(provider_stats.success_rate(), 0.5);
    }

    #[test]
    fn test_ecosystem_health_metrics() {
        let mut metrics = EcosystemHealthMetrics::default();

        metrics.overall_error_rate = 0.1; // 10% error rate
        metrics.avg_response_time_ms = 200.0;
        metrics.healthy_providers = 8;
        metrics.total_providers = 10;
        metrics.capacity_utilization_percent = 70.0;

        metrics.update_health_score();

        assert!(metrics.is_healthy());
        assert_eq!(metrics.health_status(), "Good");
    }
}
