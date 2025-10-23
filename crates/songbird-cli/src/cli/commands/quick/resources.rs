//! System Resource Detection API
//!
//! Headless API for detecting system resources that biomeOS can consume

use super::{NetworkSpeed, SystemResources};
use crate::errors::CliResult;
use serde::{Deserialize, Serialize};

/// Detect system resources via API
pub async fn detect_system_resources_api() -> CliResult<SystemResources> {
    // Use the parameterized version with default settings for full detection
    let request = ResourceDetectionRequest {
        detect_gpu: true,
        detect_storage: true,
        network_test: true,
    };

    detect_resources_with_params(request).await
}

/// Detect system resources with selective detection for performance (used for light resource checks,
#[allow(dead_code)]
pub async fn detect_system_resources_fast() -> CliResult<SystemResources> {
    // Fast detection - skip expensive tests
    let request = ResourceDetectionRequest {
        detect_gpu: false,
        detect_storage: false,
        network_test: false,
    };

    detect_resources_with_params(request).await
}

/// Resource detection request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDetectionRequest {
    pub detect_gpu: bool,
    pub detect_storage: bool,
    pub network_test: bool,
}

/// Enhanced resource detection with parameters
pub async fn detect_resources_with_params(
    request: ResourceDetectionRequest,
) -> CliResult<SystemResources> {
    let cpu_cores = num_cpus::get();
    let memory_gb = detect_available_memory();
    let storage_gb = if request.detect_storage {
        get_available_storage()
    } else {
        None
    };
    let has_gpu = if request.detect_gpu {
        detect_gpu_availability()
    } else {
        false
    };
    let network_speed = if request.network_test {
        detect_network_speed().await
    } else {
        NetworkSpeed::Medium
    };
    let platform = detect_platform();
    let architecture = detect_architecture();

    Ok(SystemResources {
        cpu_cores,
        memory_gb,
        storage_gb,
        has_gpu,
        network_speed,
        platform,
        architecture,
    })
}

fn detect_available_memory() -> f64 {
    // Use sysinfo crate instead of sys_info
    use sysinfo::System;
    let sys = System::new_all();
    sys.available_memory() as f64 / (1024.0 * 1024.0 * 1024.0) // Convert bytes to GB
}

/// Safe disk space query using sysinfo crate
/// 
/// ## Safety Evolution
/// This has been refactored from raw FFI (libc::statvfs/GetDiskFreeSpaceExW) to use
/// the `sysinfo` crate which handles all platform differences and FFI safely.
/// 
/// Benefits:
/// - 100% safe code - no unsafe blocks
/// - Cross-platform - works on Unix, Windows, macOS, FreeBSD, etc.
/// - Well-tested - sysinfo is widely used and maintained
/// - More features - easy to add more disk metrics if needed
fn get_available_disk_space_safe() -> Option<f64> {
    // SAFE: sysinfo uses safe abstractions over platform-specific APIs
    // It handles all the FFI complexity internally with proper safety checks
    use sysinfo::Disks;
    
    let disks = Disks::new_with_refreshed_list();
    
    // Find the disk containing current directory
    let current_dir = std::env::current_dir().ok()?;
    
    // Find the disk that contains our current directory
    // (or use the first disk as fallback)
    let disk = disks
        .iter()
        .find(|d| current_dir.starts_with(d.mount_point()))
        .or_else(|| disks.first())?;
    
    // Convert from bytes to GB
    Some(disk.available_space() as f64 / (1024.0 * 1024.0 * 1024.0))
}

/// Get available storage space in GB
pub fn get_available_storage() -> Option<f64> {
    get_available_disk_space_safe()
}

fn detect_gpu_availability() -> bool {
    // Check for NVIDIA GPU
    if std::process::Command::new("nvidia-smi")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false,
    {
        return true;
    }

    // Check for AMD GPU
    if std::process::Command::new("rocm-smi")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false,
    {
        return true;
    }

    // Check for Intel GPU
    if std::process::Command::new("intel_gpu_top")
        .arg("--help")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false,
    {
        return true;
    }

    false
}

async fn detect_network_speed() -> NetworkSpeed {
    if let Ok(speed_str, = std::env::var("SONGBIRD_NETWORK_SPEED") {
        match speed_str.to_lowercase().as_str() {
            "fast" => return NetworkSpeed::Fast,
            "slow" => return NetworkSpeed::Slow,
            _ => return NetworkSpeed::Medium,
        }
    }

    NetworkSpeed::Medium
}

fn detect_platform() -> String {
    std::env::consts::OS.to_string()
}

fn detect_architecture() -> String {
    std::env::consts::ARCH.to_string()
}
