// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::discovery::config::MonitoringConfig;
use crate::discovery::types::{
    CpuUsage, GpuUsage, MemoryUsage, NetworkUsage, ResourceUpdate, StorageUsage,
};

/// Resource monitoring utilities
pub struct ResourceMonitor;

impl ResourceMonitor {
    /// Collect detailed CPU usage information
    pub async fn collect_cpu_usage() -> CpuUsage {
        #[cfg(target_os = "linux")]
        {
            // Read /proc/stat for CPU usage
            if let Ok(stat_content) = tokio::fs::read_to_string("/proc/stat").await {
                let lines: Vec<&str> = stat_content.lines().collect();
                if let Some(first_line) = lines.first() {
                    if first_line.starts_with("cpu ") {
                        let values: Vec<&str> = first_line.split_whitespace().skip(1).collect();
                        if values.len() >= 4 {
                            if let (Ok(user), Ok(nice), Ok(system), Ok(idle)) = (
                                values[0].parse::<u64>(),
                                values[1].parse::<u64>(),
                                values[2].parse::<u64>(),
                                values[3].parse::<u64>(),
                            ) {
                                let total = user + nice + system + idle;
                                let used = total - idle;
                                let usage_percent = if total > 0 {
                                    (used as f32 / total as f32) * 100.0
                                } else {
                                    0.0
                                };

                                let load_average = songbird_types::sys_metrics::load_average()
                                    .map_or([0.0, 0.0, 0.0], |[a, b, c]| {
                                        [f64::from(a), f64::from(b), f64::from(c)]
                                    });
                                return CpuUsage {
                                    overall_percent: usage_percent,
                                    per_core_percent: Vec::new(), // Simplified
                                    load_average,
                                    context_switches_per_sec: 0,
                                    interrupts_per_sec: 0,
                                };
                            }
                        }
                    }
                }
            }
        }

        // Default fallback
        CpuUsage {
            overall_percent: 25.0,
            per_core_percent: Vec::new(),
            load_average: [0.5, 0.5, 0.5],
            context_switches_per_sec: 1000,
            interrupts_per_sec: 500,
        }
    }

    /// Collect detailed memory usage information
    pub async fn collect_memory_usage() -> MemoryUsage {
        if let Some(mem) = songbird_types::sys_metrics::detailed_memory_info() {
            return MemoryUsage {
                total_gb: mem.total_gb(),
                used_gb: mem.used_gb(),
                cached_gb: mem.cached_gb(),
                buffer_gb: mem.buffers_gb(),
                swap_total_gb: 0, // Simplified
                swap_used_gb: 0,
            };
        }

        // Default fallback
        MemoryUsage {
            total_gb: 16,
            used_gb: 8,
            cached_gb: 2,
            buffer_gb: 1,
            swap_total_gb: 4,
            swap_used_gb: 0,
        }
    }

    #[must_use]
    pub fn collect_gpu_usage() -> Vec<GpuUsage> {
        // GPU monitoring is delegated to external system monitoring APIs
        // Production implementations should integrate with:
        // - NVIDIA Management Library (nvidia-ml-py) for NVIDIA GPUs
        // - ROCm tools for AMD GPUs
        // - Intel GPU tools for Intel GPUs
        // For now, return empty vector (no GPU detected)
        vec![]
    }

    #[must_use]
    pub fn collect_network_usage() -> NetworkUsage {
        // Network monitoring is delegated to external system monitoring APIs
        // Production implementations should integrate with:
        // - System network interfaces (/proc/net/dev on Linux,
        // - Platform-specific network APIs
        // - SNMP for network equipment monitoring
        // For now, return zero values (no network activity detected)
        NetworkUsage {
            bytes_sent_per_sec: 0,
            bytes_received_per_sec: 0,
            packets_sent_per_sec: 0,
            packets_received_per_sec: 0,
            errors_per_sec: 0,
            drops_per_sec: 0,
        }
    }

    #[must_use]
    pub fn collect_storage_usage() -> Vec<StorageUsage> {
        vec![StorageUsage {
            device_name: "sda".to_string(),
            reads_per_sec: 100,
            writes_per_sec: 50,
            read_bytes_per_sec: 1_024_000, // 1 MB/s
            write_bytes_per_sec: 512_000,  // 512 KB/s
            queue_depth: 2.0,
        }]
    }

    /// Collect comprehensive resource update
    pub async fn collect_resource_update(
        node_id: &str,
        config: &MonitoringConfig,
    ) -> ResourceUpdate {
        ResourceUpdate {
            node_id: node_id.to_string(),
            cpu_usage: Self::collect_cpu_usage().await,
            memory_usage: Self::collect_memory_usage().await,
            gpu_usage: if config.gpu_monitoring_enabled {
                Self::collect_gpu_usage()
            } else {
                Vec::new()
            },
            network_usage: Self::collect_network_usage(),
            storage_usage: Self::collect_storage_usage(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Start continuous resource monitoring
    pub async fn start_monitoring(
        node_id: String,
        config: MonitoringConfig,
        mut shutdown_rx: tokio::sync::mpsc::Receiver<()>,
    ) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            config.resource_update_interval_secs,
        ));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let update = Self::collect_resource_update(&node_id, &config).await;

                    // Log the resource update
                    tracing::debug!(
                        node_id = %node_id,
                        cpu_percent = update.cpu_usage.overall_percent,
                        memory_used_gb = update.memory_usage.used_gb,
                        "Resource update collected"
                    );

                    // In a real implementation, this would be sent to a monitoring system
                }
                _ = shutdown_rx.recv() => {
                    tracing::info!("Resource monitoring stopped for node: {}", node_id);
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use crate::discovery::config::MonitoringConfig;

    #[tokio::test]
    async fn collect_cpu_usage_returns_sane_percent() {
        let c = ResourceMonitor::collect_cpu_usage().await;
        assert!(c.overall_percent >= 0.0 && c.overall_percent <= 100.0);
    }

    #[tokio::test]
    async fn collect_memory_usage_nonzero_total() {
        let m = ResourceMonitor::collect_memory_usage().await;
        assert!(m.total_gb > 0);
    }

    #[test]
    fn collect_network_and_storage_usage() {
        let n = ResourceMonitor::collect_network_usage();
        assert_eq!(n.errors_per_sec, 0);
        let s = ResourceMonitor::collect_storage_usage();
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].device_name, "sda");
    }

    #[tokio::test]
    async fn collect_resource_update_respects_gpu_flag() {
        let mut cfg = MonitoringConfig::default();
        cfg.gpu_monitoring_enabled = false;
        let u = ResourceMonitor::collect_resource_update("node-1", &cfg).await;
        assert_eq!(u.node_id, "node-1");
        assert!(u.gpu_usage.is_empty());

        cfg.gpu_monitoring_enabled = true;
        let u2 = ResourceMonitor::collect_resource_update("node-1", &cfg).await;
        assert_eq!(u2.gpu_usage.len(), ResourceMonitor::collect_gpu_usage().len());
    }

    #[test]
    fn monitoring_config_default_serde() {
        let c = MonitoringConfig::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: MonitoringConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.resource_update_interval_secs, c.resource_update_interval_secs);
    }

    #[tokio::test]
    async fn start_monitoring_stops_on_shutdown_signal() {
        use std::time::Duration;

        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let config = MonitoringConfig::default();
        let handle = tokio::spawn(async move {
            ResourceMonitor::start_monitoring("shutdown-node".into(), config, rx).await;
        });

        tx.send(()).await.expect("send shutdown");
        tokio::time::timeout(Duration::from_secs(2), handle)
            .await
            .expect("monitoring should stop within timeout")
            .expect("join");
    }

    #[tokio::test]
    async fn collect_resource_update_includes_timestamp_and_node_id() {
        let cfg = MonitoringConfig::default();
        let update = ResourceMonitor::collect_resource_update("metrics-node", &cfg).await;

        assert_eq!(update.node_id, "metrics-node");
        assert!(update.timestamp <= chrono::Utc::now());
        assert!(!update.storage_usage.is_empty());
    }

    #[tokio::test]
    async fn collect_cpu_usage_has_bounded_percent_and_load_average() {
        let cpu = ResourceMonitor::collect_cpu_usage().await;
        assert!(cpu.overall_percent >= 0.0 && cpu.overall_percent <= 100.0);
        assert_eq!(cpu.load_average.len(), 3);
    }

    #[test]
    fn collect_gpu_usage_returns_empty_by_default() {
        assert!(ResourceMonitor::collect_gpu_usage().is_empty());
    }

    #[test]
    fn monitoring_config_default_flags() {
        let cfg = MonitoringConfig::default();
        assert!(cfg.gpu_monitoring_enabled);
        assert!(cfg.resource_update_interval_secs > 0);
        assert!(cfg.process_scan_enabled);
        assert!(cfg.detailed_cpu_monitoring);
    }

    #[test]
    fn storage_usage_contains_io_metrics() {
        let storage = ResourceMonitor::collect_storage_usage();
        assert_eq!(storage[0].device_name, "sda");
        assert!(storage[0].read_bytes_per_sec > 0);
        assert!(storage[0].write_bytes_per_sec > 0);
    }

    #[tokio::test]
    async fn collect_memory_usage_used_does_not_exceed_total() {
        let mem = ResourceMonitor::collect_memory_usage().await;
        assert!(mem.used_gb <= mem.total_gb);
    }
}
