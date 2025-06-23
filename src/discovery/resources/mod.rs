use std::process::Command;
use std::str;
use crate::discovery::types::*;

/// Resource detection utilities
pub struct ResourceDetector;

impl ResourceDetector {
    /// Detect local compute resources
    pub fn detect_local_resources() -> ComputeResources {
        let mut resources = ComputeResources {
            cpu_cores: num_cpus::get() as u32,
            cpu_architecture: std::env::consts::ARCH.to_string(),
            memory_total_gb: Self::detect_total_memory_gb(),
            memory_available_gb: Self::detect_available_memory_gb(),
            gpu_info: Self::detect_gpu_info(),
            storage_devices: Self::detect_storage_devices(),
            network_bandwidth_mbps: Self::detect_network_bandwidth(),
        };

        // Estimate available memory more conservatively
        resources.memory_available_gb = (resources.memory_total_gb as f64 * 0.7) as u64;
        
        resources
    }

    /// Detect total system memory in GB
    fn detect_total_memory_gb() -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                for line in meminfo.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(kb) = line.split_whitespace().nth(1) {
                            if let Ok(kb_val) = kb.parse::<u64>() {
                                return kb_val / 1024 / 1024; // Convert KB to GB
                            }
                        }
                    }
                }
            }
        }
        
        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("sysctl")
                .args(&["-n", "hw.memsize"])
                .output()
            {
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
        #[cfg(target_os = "linux")]
        {
            if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
                for line in meminfo.lines() {
                    if line.starts_with("MemAvailable:") {
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
    fn detect_gpu_info() -> Vec<GpuInfo> {
        let mut gpus = Vec::new();

        // Try nvidia-smi first
        if let Ok(output) = Command::new("nvidia-smi")
            .args(&["--query-gpu=name,memory.total,utilization.gpu", "--format=csv,noheader,nounits"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = str::from_utf8(&output.stdout) {
                    for line in output_str.lines() {
                        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                        if parts.len() >= 3 {
                            if let (Ok(memory_mb), Ok(utilization)) = (
                                parts[1].parse::<u32>(),
                                parts[2].parse::<f32>()
                            ) {
                                gpus.push(GpuInfo {
                                    model: parts[0].to_string(),
                                    memory_gb: memory_mb / 1024,
                                    compute_capability: Self::detect_cuda_capability(parts[0]),
                                    utilization_percent: utilization,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Try AMD GPU detection via rocm-smi
        if let Ok(output) = Command::new("rocm-smi")
            .args(&["--showproductname", "--showmeminfo", "vram", "--showuse", "--csv"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = str::from_utf8(&output.stdout) {
                    for line in output_str.lines().skip(1) { // Skip header
                        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                        if parts.len() >= 4 {
                            let model = parts[1].to_string();
                            // Parse VRAM (e.g., "8192 MB")
                            let vram_str = parts[2];
                            let memory_mb = if vram_str.contains("MB") {
                                vram_str.split_whitespace().next()
                                    .and_then(|s| s.parse::<u32>().ok())
                                    .unwrap_or(0)
                            } else if vram_str.contains("GB") {
                                vram_str.split_whitespace().next()
                                    .and_then(|s| s.parse::<f32>().ok())
                                    .map(|gb| (gb * 1024.0) as u32)
                                    .unwrap_or(0)
                            } else {
                                0
                            };
                            
                            // Parse utilization (e.g., "15%")
                            let utilization = parts[3].trim_end_matches('%')
                                .parse::<f32>().unwrap_or(0.0);

                            if memory_mb > 0 {
                                gpus.push(GpuInfo {
                                    model,
                                    memory_gb: memory_mb / 1024,
                                    compute_capability: Self::detect_amd_compute_capability(&parts[1]),
                                    utilization_percent: utilization,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Try Intel GPU detection
        Self::detect_intel_gpus(&mut gpus);

        gpus
    }

    /// Detect Intel GPUs using multiple methods
    fn detect_intel_gpus(gpus: &mut Vec<GpuInfo>) {
        // Method 1: Try intel-gpu-top (if available)
        if let Ok(output) = Command::new("intel_gpu_top")
            .args(&["-s", "1", "-n", "1"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = str::from_utf8(&output.stdout) {
                    // Parse intel_gpu_top output
                    for line in output_str.lines() {
                        if line.contains("Intel") && line.contains("GPU") {
                            // Extract basic info from intel_gpu_top
                            gpus.push(GpuInfo {
                                model: "Intel Integrated GPU".to_string(),
                                memory_gb: 0, // intel_gpu_top doesn't show dedicated memory easily
                                compute_capability: Some("Intel Gen12".to_string()),
                                utilization_percent: 0.0, // Would need parsing
                            });
                            break;
                        }
                    }
                }
            }
        }

        // Method 2: Try Intel XPU-SMI (for discrete Intel GPUs)
        if let Ok(output) = Command::new("xpu-smi")
            .args(&["discovery"])
            .output()
        {
            if output.status.success() {
                if let Ok(output_str) = str::from_utf8(&output.stdout) {
                    for line in output_str.lines() {
                        if line.contains("Intel") && (line.contains("Xe") || line.contains("Arc")) {
                            // Parse Intel discrete GPU info
                            let model = if line.contains("Arc") {
                                "Intel Arc GPU".to_string()
                            } else {
                                "Intel Xe GPU".to_string()
                            };
                            
                            gpus.push(GpuInfo {
                                model,
                                memory_gb: 8, // Default for Arc GPUs, would need better parsing
                                compute_capability: Some("Intel Xe".to_string()),
                                utilization_percent: 0.0,
                            });
                        }
                    }
                }
            }
        }

        // Method 3: Check /sys/class/drm for Intel GPUs (fallback)
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
            let mut has_intel_gpu = false;
            for entry in entries.flatten() {
                let name = entry.file_name();
                if let Some(name_str) = name.to_str() {
                    if name_str.starts_with("card") && !name_str.contains("-") {
                        // Check if it's an Intel GPU
                        let vendor_path = format!("/sys/class/drm/{}/device/vendor", name_str);
                        if let Ok(vendor) = std::fs::read_to_string(vendor_path) {
                            if vendor.trim() == "0x8086" { // Intel vendor ID
                                has_intel_gpu = true;
                                break;
                            }
                        }
                    }
                }
            }
            
            if has_intel_gpu && !gpus.iter().any(|gpu| gpu.model.contains("Intel")) {
                gpus.push(GpuInfo {
                    model: "Intel GPU (detected)".to_string(),
                    memory_gb: 0, // Shared memory, can't easily detect
                    compute_capability: Some("Intel".to_string()),
                    utilization_percent: 0.0,
                });
            }
        }
    }

    /// Detect CUDA compute capability for NVIDIA GPUs
    fn detect_cuda_capability(gpu_name: &str) -> Option<String> {
        // Basic mapping of GPU names to compute capabilities
        if gpu_name.contains("RTX 40") || gpu_name.contains("RTX 4090") || gpu_name.contains("RTX 4080") {
            Some("8.9".to_string())
        } else if gpu_name.contains("RTX 30") || gpu_name.contains("RTX 3090") || gpu_name.contains("RTX 3080") {
            Some("8.6".to_string())
        } else if gpu_name.contains("RTX 20") || gpu_name.contains("GTX 16") {
            Some("7.5".to_string())
        } else if gpu_name.contains("GTX 10") {
            Some("6.1".to_string())
        } else if gpu_name.contains("V100") {
            Some("7.0".to_string())
        } else if gpu_name.contains("A100") {
            Some("8.0".to_string())
        } else if gpu_name.contains("H100") {
            Some("9.0".to_string())
        } else {
            None
        }
    }

    /// Detect AMD compute capability
    fn detect_amd_compute_capability(gpu_name: &str) -> Option<String> {
        // AMD GPU architecture mapping
        if gpu_name.contains("RX 7") || gpu_name.contains("7900") || gpu_name.contains("7800") {
            Some("RDNA3".to_string())
        } else if gpu_name.contains("RX 6") || gpu_name.contains("6900") || gpu_name.contains("6800") {
            Some("RDNA2".to_string())
        } else if gpu_name.contains("RX 5") || gpu_name.contains("5700") {
            Some("RDNA1".to_string())
        } else if gpu_name.contains("Vega") {
            Some("GCN5".to_string())
        } else if gpu_name.contains("MI250") || gpu_name.contains("MI210") {
            Some("CDNA2".to_string())
        } else if gpu_name.contains("MI100") {
            Some("CDNA1".to_string())
        } else {
            Some("GCN".to_string()) // Generic fallback
        }
    }

    /// Detect storage devices
    fn detect_storage_devices() -> Vec<StorageDevice> {
        let mut devices = Vec::new();

        #[cfg(target_os = "linux")]
        {
            // Use df command to get mounted filesystems
            if let Ok(output) = Command::new("df")
                .args(&["-h", "-T", "--exclude-type=tmpfs", "--exclude-type=devtmpfs"])
                .output()
            {
                if let Ok(output_str) = str::from_utf8(&output.stdout) {
                    for line in output_str.lines().skip(1) { // Skip header
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 7 {
                            let filesystem = parts[0];
                            let size_str = parts[2];
                            let available_str = parts[4];
                            let mount_point = parts[6];

                            if let (Some(size_gb), Some(available_gb)) = (
                                Self::parse_disk_size(size_str),
                                Self::parse_disk_size(available_str)
                            ) {
                                devices.push(StorageDevice {
                                    device_type: Self::detect_storage_type(filesystem),
                                    capacity_gb: size_gb,
                                    available_gb: available_gb,
                                    mount_point: mount_point.to_string(),
                                    performance_tier: Self::classify_storage_performance(filesystem),
                                });
                            }
                        }
                    }
                }
            }
        }

        if devices.is_empty() {
            // Fallback default
            devices.push(StorageDevice {
                device_type: "Unknown".to_string(),
                capacity_gb: 100,
                available_gb: 50,
                mount_point: "/".to_string(),
                performance_tier: StoragePerformanceTier::Standard,
            });
        }

        devices
    }

    /// Parse disk size from human-readable format (e.g., "100G", "1.5T")
    fn parse_disk_size(size_str: &str) -> Option<u64> {
        if size_str.is_empty() {
            return None;
        }

        let size_str = size_str.trim();
        let (number_part, unit) = if let Some(last_char) = size_str.chars().last() {
            if last_char.is_alphabetic() {
                (&size_str[..size_str.len()-1], last_char.to_ascii_uppercase())
            } else {
                (size_str, 'B')
            }
        } else {
            return None;
        };

        if let Ok(number) = number_part.parse::<f64>() {
            let multiplier = match unit {
                'K' => 1.0 / 1024.0 / 1024.0,  // KB to GB
                'M' => 1.0 / 1024.0,           // MB to GB
                'G' => 1.0,                    // GB to GB
                'T' => 1024.0,                 // TB to GB
                'P' => 1024.0 * 1024.0,        // PB to GB
                _ => 1.0 / 1024.0 / 1024.0 / 1024.0, // Assume bytes
            };
            Some((number * multiplier) as u64)
        } else {
            None
        }
    }

    /// Detect storage device type from filesystem path
    fn detect_storage_type(filesystem: &str) -> String {
        if filesystem.contains("nvme") {
            "NVMe".to_string()
        } else if filesystem.contains("ssd") || filesystem.starts_with("/dev/sd") {
            "SSD".to_string()
        } else if filesystem.starts_with("/dev/hd") {
            "HDD".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    /// Classify storage performance tier
    fn classify_storage_performance(filesystem: &str) -> StoragePerformanceTier {
        if filesystem.contains("nvme") {
            StoragePerformanceTier::HighPerformance
        } else if filesystem.contains("ssd") || filesystem.starts_with("/dev/sd") {
            StoragePerformanceTier::Standard
        } else {
            StoragePerformanceTier::Archive
        }
    }

    /// Detect network bandwidth (rough estimation)
    fn detect_network_bandwidth() -> f64 {
        #[cfg(target_os = "linux")]
        {
            // Try to read network interface speeds
            if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
                let mut max_speed = 0u64;
                
                for entry in entries.flatten() {
                    let interface_name = entry.file_name();
                    let speed_path = format!("/sys/class/net/{}/speed", interface_name.to_string_lossy());
                    
                    if let Ok(speed_str) = std::fs::read_to_string(speed_path) {
                        if let Ok(speed_mbps) = speed_str.trim().parse::<u64>() {
                            if speed_mbps > max_speed && speed_mbps < 1_000_000 { // Sanity check
                                max_speed = speed_mbps;
                            }
                        }
                    }
                }
                
                if max_speed > 0 {
                    return max_speed as f64;
                }
            }
        }

        // Default assumption: 1 Gbps
        1000.0
    }

    /// Detect network location
    pub fn detect_network_location() -> NetworkLocation {
        NetworkLocation {
            region: Self::detect_region(),
            institution: None, // Would need to be configured
            subnet: Self::detect_subnet(),
            external_ip: Self::detect_external_ip(),
            internal_ip: Self::detect_internal_ip(),
        }
    }

    /// Detect approximate region based on timezone or other heuristics
    fn detect_region() -> String {
        #[cfg(target_os = "linux")]
        {
            if let Ok(timezone) = std::fs::read_to_string("/etc/timezone") {
                let tz = timezone.trim();
                if tz.starts_with("America/") {
                    return "us-east-1".to_string(); // Simplified
                } else if tz.starts_with("Europe/") {
                    return "eu-west-1".to_string();
                } else if tz.starts_with("Asia/") {
                    return "ap-east-1".to_string();
                }
            }
        }

        "unknown".to_string()
    }

    /// Detect internal IP address
    fn detect_internal_ip() -> Option<String> {
        use std::net::UdpSocket;
        
        // Trick: connect to a remote address to determine local IP
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect("8.8.8.8:80").is_ok() {
                if let Ok(local_addr) = socket.local_addr() {
                    return Some(local_addr.ip().to_string());
                }
            }
        }
        
        None
    }

    /// Detect external IP address (simplified)
    fn detect_external_ip() -> Option<String> {
        // In a real implementation, you'd query an external service
        // For now, return None to avoid network calls in this demo
        None
    }

    /// Detect subnet from internal IP
    fn detect_subnet() -> Option<String> {
        if let Some(internal_ip) = Self::detect_internal_ip() {
            if let Ok(ip) = internal_ip.parse::<std::net::IpAddr>() {
                match ip {
                    std::net::IpAddr::V4(ipv4) => {
                        let octets = ipv4.octets();
                        // Assume /24 subnet
                        return Some(format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2]));
                    }
                    std::net::IpAddr::V6(_) => {
                        // IPv6 subnet detection would be more complex
                        return None;
                    }
                }
            }
        }
        None
    }
} 