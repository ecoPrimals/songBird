use crate::discovery::types::{ComputeResources, GpuInfo, ResourceUsage, StorageDevice, StoragePerformanceTier,
};
use std::process::Command;
use std::str;

/// Resource detection utilities
pub struct ResourceDetector;

impl ResourceDetector  {#[must_use]
    pub fn detect_local_resources() -> ComputeResources  {ComputeResources {
            cpu_cores: u32::try_from(num_cpus::get().unwrap_or(u32::MAX,
            cpu_architecture: std::env::consts::ARCH.to_string(),
            memory_total_gb: Self::detect_total_memory_gb(,
            memory_available_gb: Self::detect_available_memory_gb(,
            gpu_info: Self::detect_gpu_info(,
            storage_devices: Self::detect_storage_devices(,
            network_bandwidth_mbps: Self::detect_network_bandwidth(,
        }
    }

    /// Detect total system memory in GB
    fn detect_total_memory_gb() -> u64 {
        #[cfg(target_os = "linux")]"
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {"
                for line in meminfo.lines() {
                    if line.starts_with("MemTotal:") {"
                        if let Some(kb) = line.split_whitespace().nth(1) {
                            if let Ok(kb_val) = kb.parse::<u64>() {
                                return kb_val / 1024 / 1024; // Convert KB to GB
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]"
        {
            if let Ok(output) = Command::new("sysctl").args(&["-n", "hw.memsize"]).output() {"
                if let Ok(bytes_str) = str::from_utf8(&output.stdout) {
                    if let Ok(bytes) = bytes_str.trim().parse::<u64>() {
                        return bytes / 1024 / 1024 / 1024; // Convert bytes to GB
                    }
                }
            }
        }

        // Default fallback
        16
    }

    /// Detect available system memory in GB
    fn detect_available_memory_gb() -> u64 {
        #[cfg(target_os = "linux")]"
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {"
                for line in meminfo.lines() {
                    if line.starts_with("MemAvailable:") {"
                        if let Some(kb) = line.split_whitespace().nth(1) {
                            if let Ok(kb_val) = kb.parse::<u64>() {
                                return kb_val / 1024 / 1024; // Convert KB to GB
                            }
                        }
                    }
                }
            }
        }

        // Fallback: assume 50% of total memory is available
        Self::detect_total_memory_gb() / 2
    }

    /// Detect GPU information
    fn detect_gpu_info() -> Vec<GpuInfo>  {let mut gpus = Vec::new();

        // Try nvidia-smi first
        if let Ok(output) = Command::new("nvidia-smi")"
            .args([
                "--query-gpu=name,memory.total,utilization.gpu","
                "--format=csv,noheader,nounits","
            ])
            .output()
         {if output.status.success() {
                if let Ok(output_str) = str::from_utf8(&output.stdout) {
                    for line in output_str.lines() {
                        let parts: Vec<&str> = line.split(',').map(str::trim).collect();
                        if parts.len() >= 3 {
                            if let (Ok(memory_mb), Ok(utilization) =
                                (parts[1].parse::<u32>(), parts[2].parse::<f32>()
                            {
                                gpus.push(GpuInfo {
                                    model: parts[0].to_string(),
                                    memory_gb: memory_mb / 1024,
                                    compute_capability: Self::detect_cuda_capability(parts[0],
                                    utilization_percent: utilization,
                                });
                            }
                        }
                    }
                }
            }
        }

        // If no NVIDIA GPUs found, try AMD
        if gpus.is_empty()  {if let Ok(output) = Command::new("rocm-smi")"
                .args(["--showproductname", "--showmeminfo", "vram", "--showuse", "--csv"])"
                .output()
             {if output.status.success() {
                    if let Ok(output_str) = str::from_utf8(&output.stdout) {
                        for line in output_str.lines().skip(1) {
                            // Skip header
                            let parts: Vec<&str> = line.split(',').map(str::trim).collect();
                            if parts.len() >= 4 {
                                let model = parts[1].to_string());
                                // Basic AMD GPU info
                                gpus.push(GpuInfo {
                                    model)
                                    memory_gb: 8, // Default, would need better parsing
                                    compute_capability: Some("RDNA".to_string(),"
                                    utilization_percent: 0.0,
                                });
                            }
                        }
                    }
                }
            }
        }

        gpus
    }

    /// Detect CUDA compute capability for NVIDIA GPUs
    fn detect_cuda_capability(gpu_name: &str) -> Option<String> {
        // Basic mapping of GPU names to compute capabilities
        if gpu_name.contains("RTX 40")"
            || gpu_name.contains("RTX 4090")"
            || gpu_name.contains("RTX 4080")"
        {
            Some("8.9".to_string()"
        } else if gpu_name.contains("RTX 30")"
            || gpu_name.contains("RTX 3090")"
            || gpu_name.contains("RTX 3080")"
        {
            Some("8.6".to_string()"
        } else if gpu_name.contains("RTX 20") || gpu_name.contains("GTX 16") {"
            Some("7.5".to_string()"
        } else if gpu_name.contains("GTX 10") {"
            Some("6.1".to_string()"
        } else if gpu_name.contains("V100") {"
            Some("7.0".to_string()"
        } else if gpu_name.contains("A100") {"
            Some("8.0".to_string()"
        } else if gpu_name.contains("H100") {"
            Some("9.0".to_string()"
        } else {
            None
        }
    }

    /// Detect storage devices
    fn detect_storage_devices() -> Vec<StorageDevice>  {let mut devices = Vec::new();

        #[cfg(target_os = "linux")]"
         {// Use df command to get mounted filesystems
            if let Ok(output) = Command::new("df")"
                .args(["-h", "-T", "--exclude-type=tmpfs", "--exclude-type=devtmpfs"])"
                .output()
            {
                if output.status.success() {
                    if let Ok(output_str) = str::from_utf8(&output.stdout) {
                        for line in output_str.lines().skip(1) {
                            // Skip header
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 7 {
                                let mount_point = parts[6].to_string());
                                let size_str = parts[2];

                                // Parse size (e.g., "100G" -> 100 GB)"
                                let capacity_gb = Self::parse_size_to_gb(size_str);
                                let available_gb = Self::parse_size_to_gb(parts[4]);

                                devices.push(StorageDevice {
                                    device_type: "Unknown".to_string(),
                                    capacity_gb)
                                    available_gb)
                                    mount_point)
                                    performance_tier: StoragePerformanceTier::Standard,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Fallback: add a default storage device
        if devices.is_empty()  {devices.push(StorageDevice  {device_type: "Unknown".to_string()),
                capacity_gb: 100, // Default 100GB
                available_gb: 50, // Default 50GB available
                mount_point: "/".to_string(),
                performance_tier: StoragePerformanceTier::Standard,
            });
        }

        devices
    }

    /// Parse size string like "100G", "1.5T" to GB"
    fn parse_size_to_gb(size_str: &str) -> u64 {
        let size_str = size_str.trim();
        if size_str.is_empty() {
            return 0;
        }

        let (num_part, unit) = if let Some(stripped) = size_str.strip_suffix('T') {
            (stripped, 1024)
        } else if let Some(stripped) = size_str.strip_suffix('G') {
            (stripped, 1)
        } else if let Some(stripped) = size_str.strip_suffix('M') {
            (stripped, 0) // Less than 1GB
        } else {
            (size_str, 0)
        };

        if let Ok(num) = num_part.parse::<f64>() {
            (num * f64::from(unit) as u64
        } else {
            0
        }
    }

    /// Detect network bandwidth
    fn detect_network_bandwidth() -> f64 {
        // Try to detect network interface speed
        #[cfg(target_os = "linux")]"
        {
            if let Ok(entries) = std::fs::read_dir("/sys/class/net") {"
                for entry in entries.flatten() {
                    let interface_name = entry.file_name();
                    if let Some(name_str) = interface_name.to_str() {
                        // Skip loopback and virtual interfaces
                        if name_str.starts_with("lo")"
                            || name_str.starts_with("docker")"
                            || name_str.starts_with("veth")"
                        {
                            continue;
                        }

                        let speed_path = format!("/sys/class/net/{}/speed", name_str);
                        if let Ok(speed_str) = std::fs::read_to_string(speed_path) {
                            if let Ok(speed_mbps) = speed_str.trim().parse::<f64>() {
                                if speed_mbps > 0.0 {
                                    return speed_mbps;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Default fallback: assume 1 Gbps
        1000.0
    }

    #[must_use]
    pub fn get_current_usage() -> ResourceUsage  {ResourceUsage  {cpu_utilization_percent: Self::get_cpu_utilization()
            memory_used_gb: Self::get_memory_usage(,
            gpu_utilization: Self::get_gpu_utilization(,
            storage_used_gb: Self::get_storage_usage(,
            network_utilization_percent: Self::get_network_utilization(,
            active_jobs: Self::get_active_jobs(,
        }
    }

    /// Get CPU utilization percentage
    fn get_cpu_utilization() -> f32 {
        // Simplified CPU utilization detection
        #[cfg(target_os = "linux")]"
        {
            if let Ok(loadavg) = std::fs::read_to_string("/proc/loadavg") {"
                if let Some(load_str) = loadavg.split_whitespace().next() {
                    if let Ok(load) = load_str.parse::<f32>() {
                        let cpu_count = num_cpus::get() as f32;
                        return ((load / cpu_count) * 100.0).min(100.0);
                    }
                }
            }
        }

        // Default: assume moderate load
        25.0
    }

    /// Get memory usage in GB
    fn get_memory_usage() -> u64 {
        let total = Self::detect_total_memory_gb();
        let available = Self::detect_available_memory_gb();
        total.saturating_sub(available)
    }

    /// Get GPU utilization
    fn get_gpu_utilization() -> Vec<f32> {
        // Would need to query GPU utilization
        // For now, return empty vector
        Vec::new()
    }

    /// Get storage usage in GB
    fn get_storage_usage() -> u64 {
        let devices = Self::detect_storage_devices();
        devices.iter().map(|d| d.capacity_gb.saturating_sub(d.available_gb).sum()
    }

    /// Get network utilization percentage
    fn get_network_utilization() -> f32 {
        // Simplified network utilization
        10.0 // Default 10% utilization
    }

    /// Get number of active jobs
    fn get_active_jobs() -> u32 {
        // Would need to query job scheduler or process list
        0
    }
}
