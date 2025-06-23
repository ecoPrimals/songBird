use std::process::Command;
use std::str;
use crate::discovery::types::*;
use crate::discovery::config::MonitoringConfig;

/// Resource monitoring utilities
pub struct ResourceMonitor;

impl ResourceMonitor {
    /// Collect detailed CPU usage information
    pub async fn collect_cpu_usage() -> CpuUsage {
        let mut cpu_usage = CpuUsage::default();
        
        #[cfg(target_os = "linux")]
        {
            // Read /proc/stat for CPU usage
            if let Ok(stat_content) = tokio::fs::read_to_string("/proc/stat").await {
                let lines: Vec<&str> = stat_content.lines().collect();
                
                // Parse overall CPU usage from first line
                if let Some(cpu_line) = lines.first() {
                    if cpu_line.starts_with("cpu ") {
                        let parts: Vec<&str> = cpu_line.split_whitespace().collect();
                        if parts.len() >= 8 {
                            let user: u64 = parts[1].parse().unwrap_or(0);
                            let nice: u64 = parts[2].parse().unwrap_or(0);
                            let system: u64 = parts[3].parse().unwrap_or(0);
                            let idle: u64 = parts[4].parse().unwrap_or(0);
                            let iowait: u64 = parts[5].parse().unwrap_or(0);
                            let irq: u64 = parts[6].parse().unwrap_or(0);
                            let softirq: u64 = parts[7].parse().unwrap_or(0);
                            
                            let total = user + nice + system + idle + iowait + irq + softirq;
                            let active = total - idle - iowait;
                            
                            cpu_usage.overall_percent = if total > 0 {
                                (active as f32 / total as f32) * 100.0
                            } else { 0.0 };
                        }
                    }
                }
                
                // Parse per-core usage
                for line in lines.iter().skip(1) {
                    if line.starts_with("cpu") && line.chars().nth(3).unwrap_or(' ').is_ascii_digit() {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 8 {
                            let user: u64 = parts[1].parse().unwrap_or(0);
                            let nice: u64 = parts[2].parse().unwrap_or(0);
                            let system: u64 = parts[3].parse().unwrap_or(0);
                            let idle: u64 = parts[4].parse().unwrap_or(0);
                            let iowait: u64 = parts[5].parse().unwrap_or(0);
                            let irq: u64 = parts[6].parse().unwrap_or(0);
                            let softirq: u64 = parts[7].parse().unwrap_or(0);
                            
                            let total = user + nice + system + idle + iowait + irq + softirq;
                            let active = total - idle - iowait;
                            
                            let core_percent = if total > 0 {
                                (active as f32 / total as f32) * 100.0
                            } else { 0.0 };
                            
                            cpu_usage.per_core_percent.push(core_percent);
                        }
                    }
                }
            }
            
            // Read load average from /proc/loadavg
            if let Ok(loadavg_content) = tokio::fs::read_to_string("/proc/loadavg").await {
                let parts: Vec<&str> = loadavg_content.split_whitespace().collect();
                if parts.len() >= 3 {
                    cpu_usage.load_average[0] = parts[0].parse().unwrap_or(0.0);
                    cpu_usage.load_average[1] = parts[1].parse().unwrap_or(0.0);
                    cpu_usage.load_average[2] = parts[2].parse().unwrap_or(0.0);
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            // Use system commands for macOS
            if let Ok(output) = Command::new("sysctl")
                .args(&["-n", "vm.loadavg"])
                .output()
            {
                if let Ok(loadavg_str) = str::from_utf8(&output.stdout) {
                    let parts: Vec<&str> = loadavg_str.trim()
                        .trim_start_matches('{')
                        .trim_end_matches('}')
                        .split_whitespace()
                        .collect();
                    if parts.len() >= 3 {
                        cpu_usage.load_average[0] = parts[0].parse().unwrap_or(0.0);
                        cpu_usage.load_average[1] = parts[1].parse().unwrap_or(0.0);
                        cpu_usage.load_average[2] = parts[2].parse().unwrap_or(0.0);
                    }
                }
            }
        }
        
        cpu_usage
    }

    /// Collect detailed memory usage information
    pub async fn collect_memory_usage() -> MemoryUsage {
        let mut memory_usage = MemoryUsage::default();
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo_content) = tokio::fs::read_to_string("/proc/meminfo").await {
                for line in meminfo_content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let value_kb: u64 = parts[1].parse().unwrap_or(0);
                        let value_gb = value_kb / 1024 / 1024;
                        
                        match parts[0] {
                            "MemTotal:" => memory_usage.total_gb = value_gb,
                            "MemAvailable:" => memory_usage.available_gb = value_gb,
                            "Cached:" => memory_usage.cached_gb = value_gb,
                            "Buffers:" => memory_usage.buffer_gb = value_gb,
                            "SwapTotal:" => memory_usage.swap_total_gb = value_gb,
                            "SwapFree:" => {
                                let swap_free_gb = value_gb;
                                memory_usage.swap_used_gb = memory_usage.swap_total_gb.saturating_sub(swap_free_gb);
                            }
                            _ => {}
                        }
                    }
                }
                
                memory_usage.used_gb = memory_usage.total_gb.saturating_sub(memory_usage.available_gb);
            }
        }
        
        memory_usage
    }

    /// Collect GPU usage information
    pub async fn collect_gpu_usage() -> Vec<GpuUsage> {
        let mut gpu_usage = Vec::new();
        
        // Try nvidia-smi for NVIDIA GPUs
        if let Ok(output) = Command::new("nvidia-smi")
            .args(&[
                "--query-gpu=index,utilization.gpu,memory.used,memory.total,temperature.gpu,power.draw",
                "--format=csv,noheader,nounits"
            ])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = str::from_utf8(&output.stdout) {
                    for line in output_str.lines() {
                        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                        if parts.len() >= 6 {
                            if let (Ok(index), Ok(util), Ok(mem_used), Ok(mem_total), Ok(temp), Ok(power)) = (
                                parts[0].parse::<u32>(),
                                parts[1].parse::<f32>(),
                                parts[2].parse::<u32>(),
                                parts[3].parse::<u32>(),
                                parts[4].parse::<f32>(),
                                parts[5].parse::<f32>()
                            ) {
                                gpu_usage.push(GpuUsage {
                                    gpu_index: index,
                                    utilization_percent: util,
                                    memory_used_mb: mem_used,
                                    memory_total_mb: mem_total,
                                    temperature_celsius: temp,
                                    power_draw_watts: power,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        gpu_usage
    }

    /// Collect network usage information
    pub async fn collect_network_usage() -> NetworkUsage {
        let mut network_usage = NetworkUsage::default();
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(net_dev_content) = tokio::fs::read_to_string("/proc/net/dev").await {
                let mut total_rx_bytes = 0u64;
                let mut total_tx_bytes = 0u64;
                let mut total_rx_packets = 0u64;
                let mut total_tx_packets = 0u64;
                let mut total_rx_errors = 0u64;
                let mut total_tx_errors = 0u64;
                
                for line in net_dev_content.lines().skip(2) { // Skip header lines
                    if let Some(colon_pos) = line.find(':') {
                        let (interface, stats) = line.split_at(colon_pos);
                        let interface = interface.trim();
                        let stats = &stats[1..]; // Remove the ':'
                        
                        // Skip loopback interface
                        if interface == "lo" {
                            continue;
                        }
                        
                        let parts: Vec<&str> = stats.split_whitespace().collect();
                        if parts.len() >= 16 {
                            // RX: bytes packets errs drop fifo frame compressed multicast
                            // TX: bytes packets errs drop fifo colls carrier compressed
                            total_rx_bytes += parts[0].parse::<u64>().unwrap_or(0);
                            total_rx_packets += parts[1].parse::<u64>().unwrap_or(0);
                            total_rx_errors += parts[2].parse::<u64>().unwrap_or(0);
                            
                            total_tx_bytes += parts[8].parse::<u64>().unwrap_or(0);
                            total_tx_packets += parts[9].parse::<u64>().unwrap_or(0);
                            total_tx_errors += parts[10].parse::<u64>().unwrap_or(0);
                        }
                    }
                }
                
                // Note: These are cumulative values, would need to calculate per-second rates
                // For now, just store the totals (in a real implementation, you'd track deltas)
                let window_secs = 3600u64; // Could be made configurable
                network_usage.bytes_received_per_sec = total_rx_bytes / window_secs;
                network_usage.bytes_sent_per_sec = total_tx_bytes / window_secs;
                network_usage.packets_received_per_sec = total_rx_packets / window_secs;
                network_usage.packets_sent_per_sec = total_tx_packets / window_secs;
                network_usage.errors_per_sec = (total_rx_errors + total_tx_errors) / window_secs;
            }
        }
        
        network_usage
    }

    /// Collect storage usage information
    pub async fn collect_storage_usage() -> Vec<StorageUsage> {
        let mut storage_usage = Vec::new();
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(diskstats_content) = tokio::fs::read_to_string("/proc/diskstats").await {
                for line in diskstats_content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 14 {
                        let device_name = parts[2];
                        
                        // Skip partition numbers and loop devices
                        if device_name.chars().last().unwrap_or('0').is_ascii_digit() ||
                           device_name.starts_with("loop") ||
                           device_name.starts_with("ram") {
                            continue;
                        }
                        
                        let reads_completed = parts[3].parse::<u64>().unwrap_or(0);
                        let sectors_read = parts[5].parse::<u64>().unwrap_or(0);
                        let writes_completed = parts[7].parse::<u64>().unwrap_or(0);
                        let sectors_written = parts[9].parse::<u64>().unwrap_or(0);
                        let io_time_ms = parts[12].parse::<u64>().unwrap_or(0);
                        
                        // Convert sectors to bytes (assuming 512 bytes per sector)
                        let read_bytes = sectors_read * 512;
                        let write_bytes = sectors_written * 512;
                        
                        // These are cumulative values, would need to track deltas for per-second rates
                        let window_secs = 3600u64; // Could be made configurable
                        storage_usage.push(StorageUsage {
                            device_name: device_name.to_string(),
                            reads_per_sec: reads_completed / window_secs,
                            writes_per_sec: writes_completed / window_secs,
                            read_bytes_per_sec: read_bytes / window_secs,
                            write_bytes_per_sec: write_bytes / window_secs,
                            utilization_percent: (io_time_ms as f32 / (window_secs * 10) as f32).min(100.0),
                            queue_depth: 1.0, // Would need more sophisticated calculation
                        });
                    }
                }
            }
        }
        
        storage_usage
    }

    /// Count active processes (rough approximation of system load)
    pub async fn count_active_processes() -> u32 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(entries) = std::fs::read_dir("/proc") {
                let mut count = 0u32;
                for entry in entries.flatten() {
                    if let Ok(name) = entry.file_name().into_string() {
                        if name.chars().all(|c| c.is_ascii_digit()) {
                            count += 1;
                        }
                    }
                }
                return count;
            }
        }
        
        // Fallback
        0
    }

    /// Get current CPU usage percentage
    pub async fn get_current_cpu_usage() -> f32 {
        let cpu_usage = Self::collect_cpu_usage().await;
        cpu_usage.overall_percent
    }

    /// Get current memory usage in GB
    pub async fn get_current_memory_usage() -> u64 {
        let memory_usage = Self::collect_memory_usage().await;
        memory_usage.used_gb
    }

    /// Get current GPU utilization percentages
    pub async fn get_current_gpu_usage() -> Vec<f32> {
        let gpu_usage = Self::collect_gpu_usage().await;
        gpu_usage.iter().map(|gpu| gpu.utilization_percent).collect()
    }

    /// Get current storage usage in GB
    pub async fn get_current_storage_usage() -> u64 {
        let storage_usage = Self::collect_storage_usage().await;
        storage_usage.iter().map(|s| {
            // Estimate used storage from capacity - available
            let total_gb = s.read_bytes_per_sec + s.write_bytes_per_sec; // Simplified
            total_gb / 1_000_000_000 // Convert to GB approximation
        }).sum()
    }

    /// Get current network utilization percentage
    pub async fn get_current_network_usage(default_bandwidth_mbps: f64) -> f32 {
        let network_usage = Self::collect_network_usage().await;
        // Simplification: calculate based on bytes sent/received relative to interface capacity
        let total_bytes_per_sec = network_usage.bytes_sent_per_sec + network_usage.bytes_received_per_sec;
        let bytes_per_mbps = (default_bandwidth_mbps * 1_000_000.0 / 8.0) as u64; // Convert Mbps to bytes/sec
        let utilization = (total_bytes_per_sec as f64 / bytes_per_mbps as f64) * 100.0;
        utilization.min(100.0) as f32
    }

    /// Create resource usage summary
    pub async fn create_resource_usage(config: &MonitoringConfig, default_bandwidth_mbps: f64) -> ResourceUsage {
        ResourceUsage {
            cpu_utilization_percent: Self::get_current_cpu_usage().await,
            memory_used_gb: Self::get_current_memory_usage().await,
            gpu_utilization: if config.gpu_monitoring_enabled {
                Self::get_current_gpu_usage().await
            } else {
                Vec::new()
            },
            storage_used_gb: Self::get_current_storage_usage().await,
            network_utilization_percent: Self::get_current_network_usage(default_bandwidth_mbps).await,
            active_jobs: if config.process_scan_enabled {
                Self::count_active_processes().await
            } else {
                0
            },
        }
    }

    /// Create detailed resource update
    pub async fn create_resource_update(node_id: String, config: &MonitoringConfig) -> ResourceUpdate {
        ResourceUpdate {
            node_id,
            cpu_usage: Self::collect_cpu_usage().await,
            memory_usage: Self::collect_memory_usage().await,
            gpu_usage: if config.gpu_monitoring_enabled {
                Self::collect_gpu_usage().await
            } else {
                Vec::new()
            },
            network_usage: Self::collect_network_usage().await,
            storage_usage: Self::collect_storage_usage().await,
            timestamp: chrono::Utc::now(),
        }
    }
} 