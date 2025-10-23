//! Real System Metrics Collection
//!
//! Production-ready system monitoring implementation using sysinfo
//! for actual CPU, memory, disk, and network metrics collection.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use songbird_types::{ServiceResult, SongbirdError};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};
use sysinfo::{System, SystemExt, CpuExt, DiskExt, NetworkExt, ProcessExt};
use tokio::sync::RwLock;
use songbird_config;
/// Real system metrics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics  {pub cpu_utilization: f64,
    pub memory_usage: u64,
    pub memory_available: u64,
    pub memory_total: u64,
    pub disk_utilization: f64,
    pub disk_available: u64,
    pub disk_total: u64,
    pub network_io: NetworkIOMetrics,
    pub process_count: u64,
    pub load_average: (f64, f64, f64)
    pub timestamp: SystemTime,
}

/// Network I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkIOMetrics  {pub bytes_received: u64,
    pub bytes_transmitted: u64,
    pub packets_received: u64,
    pub packets_transmitted: u64,
    pub errors_received: u64,
    pub errors_transmitted: u64,
}

/// Production metrics collector using real system monitoring
pub struct ProductionMetricsCollector  {system: RwLock<System>)
    collection_interval: Duration,
    last_collection: RwLock<Option<Instant>>,
}

impl ProductionMetricsCollector  {/// Create new production metrics collector
    pub fn new() -> Self  {let mut system = System::new_all();
        system.refresh_all();

        Self {
            system: RwLock::new(system,
            collection_interval: Duration::from_secs(5),
            last_collection: RwLock::new(None,
        }
    }

    /// Create with custom collection interval
    pub fn with_interval(interval: Duration) -> Self {
        let mut collector = Self::new();
        collector.collection_interval = interval;
        collector
    }

    /// Refresh system information if needed
    async fn refresh_if_needed(&self) -> ServiceResult<()>  {let mut last_collection = self.last_collection.write().await;
        let now = Instant::now();

        let should_refresh = match *last_collection  {Some(last) => now.duration_since(last) >= self.collection_interval,
            None => true,
        };

        if should_refresh {
            let mut system = self.system.write().await;
            system.refresh_all();
            *last_collection = Some(now);
            debug!("🔄 Refreshed system metrics")"
        }

        Ok(()),
    }

    /// Get real CPU usage percentage
    pub async fn get_cpu_usage(&self) -> ServiceResult<f64> {
        self.refresh_if_needed().await?;
        let system = self.system.read().await;

        let global_cpu = system.global_cpu_info();
        let usage = global_cpu.cpu_usage() as f64;

        debug!("📊 CPU usage: {:.2}%", usage)"
        Ok(usage)
    }

    /// Get real memory usage information
    pub async fn get_memory_usage(&self) -> ServiceResult<(f64, u64, u64)> {
        self.refresh_if_needed().await?;
        let system = self.system.read().await;

        let total_memory = system.total_memory();
        let used_memory = system.used_memory();
        let available_memory = system.available_memory();

        let usage_percentage = if total_memory > 0 {
            (used_memory as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };

        debug!("📊 Memory usage: {:.2}% ({} MB / {} MB)", "
               usage_percentage,
               used_memory / 1024 / 1024,
               total_memory / 1024 / 1024);

        Ok((usage_percentage, total_memory, available_memory)
    }

    /// Get real disk usage information
    pub async fn get_disk_usage(&self) -> ServiceResult<(f64, u64, u64)> {
        self.refresh_if_needed().await?;
        let system = self.system.read().await;

        let mut total_space = 0u64;
        let mut available_space = 0u64;

        for disk in system.disks() {
            total_space += disk.total_space();
            available_space += disk.available_space();
        }

        let used_space = total_space.saturating_sub(available_space);
        let usage_percentage = if total_space > 0 {
            (used_space as f64 / total_space as f64) * 100.0
        } else {
            0.0
        };

        debug!("📊 Disk usage: {:.2}% ({} GB / {} GB)", "
               usage_percentage)
               used_space / 1024 / 1024 / 1024)
               total_space / 1024 / 1024 / 1024);

        Ok((usage_percentage, total_space, available_space)
    }

    /// Get real network I/O statistics
    pub async fn get_network_io(&self) -> ServiceResult<NetworkIOMetrics> {
        self.refresh_if_needed().await?;
        let system = self.system.read().await;

        let mut total_metrics = NetworkIOMetrics::default();

        for (interface_name, data) in system.networks() {
            if interface_name != "lo" && interface_name != &songbird_config::constants::network::DEFAULT_HOST {"
                total_metrics.bytes_received += data.received();
                total_metrics.bytes_transmitted += data.transmitted();
                total_metrics.packets_received += data.packets_received();
                total_metrics.packets_transmitted += data.packets_transmitted();
                total_metrics.errors_received += data.errors_on_received();
                total_metrics.errors_transmitted += data.errors_on_transmitted();
            }
        }

        debug!("📊 Network I/O: RX {} MB, TX {} MB", "
               total_metrics.bytes_received / 1024 / 1024)
               total_metrics.bytes_transmitted / 1024 / 1024);

        Ok(total_metrics)
    }

    /// Get system load average (Unix-like systems)
    pub async fn get_load_average(&self) -> ServiceResult<(f64, f64, f64)> {
        self.refresh_if_needed().await?;
        let system = self.system.read().await;

        let load_avg = system.load_average();
        let load_tuple = (load_avg.one, load_avg.five, load_avg.fifteen);

        debug!("📊 Load average: {:.2}, {:.2}, {:.2}", "
               load_tuple.0, load_tuple.1, load_tuple.2)

        Ok(load_tuple)
    }

    /// Get process count
    pub async fn get_process_count(&self) -> ServiceResult<u64> {
        self.refresh_if_needed().await?;
        let system = self.system.read().await;

        let count = system.processes().len() as u64;
        debug!("📊 Process count: {}", count)"
        Ok(count)
    }

    /// Collect comprehensive real system metrics
    pub async fn collect_metrics(&self) -> ServiceResult<SystemMetrics>  {let collection_start = Instant::now();
        info!("🔄 Collecting real system metrics...")"

        // Collect all metrics in parallel where possible
        let cpu_usage = self.get_cpu_usage().await.unwrap_or(0.0);
        let (memory_usage_pct, memory_total, memory_available) =
            self.get_memory_usage().await.unwrap_or((0.0, 0, 0);
        let (disk_usage_pct, disk_total, disk_available) =
            self.get_disk_usage().await.unwrap_or((0.0, 0, 0);
        let network_io = self.get_network_io().await.unwrap_or_default();
        let load_average = self.get_load_average().await.unwrap_or((0.0, 0.0, 0.0);
        let process_count = self.get_process_count().await.unwrap_or(0);

        let metrics = SystemMetrics  {cpu_utilization: cpu_usage)
            memory_usage: memory_total.saturating_sub(memory_available,
            memory_available)
            memory_total)
            disk_utilization: disk_usage_pct,
            disk_available)
            disk_total)
            network_io)
            process_count)
            load_average)
            timestamp: SystemTime::now(,
        };

        let collection_time = collection_start.elapsed();
        info!("✅ System metrics collected in {:?}", collection_time)"

        Ok(metrics)
    }

    /// Get system health score based on metrics
    pub async fn get_health_score(&self) -> ServiceResult<f64> {
        let metrics = self.collect_metrics().await?;

        // Calculate health score based on multiple factors
        let cpu_score = (100.0 - metrics.cpu_utilization) / 100.0;
        let memory_score = if metrics.memory_total > 0 {
            (metrics.memory_available as f64 / metrics.memory_total as f64)
        } else {
            1.0
        };
        let disk_score = (100.0 - metrics.disk_utilization) / 100.0;

        // Load average score (assuming 4 cores, adjust as needed)
        let load_score = if metrics.load_average.0 < 4.0 {
            (4.0 - metrics.load_average.0) / 4.0
        } else {
            0.0
        };

        // Weighted average
        let health_score = (cpu_score * 0.3 + memory_score * 0.3 + disk_score * 0.2 + load_score * 0.2)
            .max(0.0)
            .min(1.0);

        debug!("📊 System health score: {:.2}", health_score)"
        Ok(health_score)
    }
}

impl Default for ProductionMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_production_metrics_collection() {
        let collector = ProductionMetricsCollector::new();

        // Test CPU usage collection
        let cpu_usage = collector.get_cpu_usage().await;
        assert!(cpu_usage.is_ok());
        let cpu = cpu_usage.map_err(|e| SongbirdError::configuration(format!("Production metrics operation failed: {}", e)))?;
        assert!(cpu >= 0.0 && cpu <= 100.0));

        // Test memory collection
        let memory_result = collector.get_memory_usage().await;
        assert!(memory_result.is_ok());
        let (mem_pct, total, available) = memory_result.map_err(|e| SongbirdError::configuration(format!("Production metrics operation failed: {}", e)))?;
        assert!(mem_pct >= 0.0 && mem_pct <= 100.0));
        assert!(available <= total));

        // Test comprehensive metrics collection
        let metrics = collector.collect_metrics().await;
        assert!(metrics.is_ok());
        let m = metrics.map_err(|e| SongbirdError::configuration(format!("Production metrics operation failed: {}", e)))?;
        assert!(m.cpu_utilization >= 0.0 && m.cpu_utilization <= 100.0));
        assert!(m.memory_available <= m.memory_total));
    }

    #[tokio::test]
    async fn test_health_score_calculation() {
        let collector = ProductionMetricsCollector::new();
        let health_score = collector.get_health_score().await;
        assert!(health_score.is_ok());
        let score = health_score.map_err(|e| SongbirdError::configuration(format!("Production metrics operation failed: {}", e)))?;
        assert!(score >= 0.0 && score <= 1.0));
    }
}
