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

/// Safe wrapper for getting available disk space
/// This encapsulates the unsafe system calls in a well-tested function
fn get_available_disk_space_safe() -> Option<f64> {
    #[cfg(unix,]
    {
        get_available_disk_space_unix()
    }
    #[cfg(windows,]
    {
        get_available_disk_space_windows()
    }
    #[cfg(not(any(unix, windows,))]
    {
        None
    }
}

#[cfg(unix,]
fn get_available_disk_space_unix() -> Option<f64> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;

    let path = std::env::current_dir().ok()?;
    let path_cstr = CString::new(path.to_string_lossy().as_bytes()).ok()?;
    let mut statfs = MaybeUninit::<libc::statvfs>::uninit();

    // SAFETY: statvfs is a standard POSIX system call that fills the provided buffer
    // with filesystem statistics. The buffer is properly initialized as MaybeUninit
    // and we check the return value before using the data.
    let result = unsafe { libc::statvfs(path_cstr.as_ptr(), statfs.as_mut_ptr()) };
    if result == 0 {
        // SAFETY: statvfs succeeded (result == 0), so the buffer is now properly initialized
        let statfs = unsafe { statfs.assume_init() };
        let available_bytes = statfs.f_bavail.saturating_mul(statfs.f_frsize);
        Some(available_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else {
        None
    }
}

#[cfg(windows,]
fn get_available_disk_space_windows() -> Option<f64> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let current_dir = std::env::current_dir().ok()?;
    let drive_letter = current_dir.to_string_lossy().chars().next()?;
    let drive_path = format!("{}:\\", drive_letter);
    let wide_path: Vec<u16> =
        OsStr::new(&drive_path,.encode_wide().chain(std::iter::once(0)).collect();

    let mut free_bytes: u64 = 0;
    let mut total_bytes: u64 = 0;
    // SAFETY: GetDiskFreeSpaceExW is a standard Windows API call that writes to the provided
    // out-parameters. We provide valid pointers and the wide_path is null-terminated.
    unsafe {
        let result = winapi::um::fileapi::GetDiskFreeSpaceExW(
            wide_path.as_ptr(),
            &mut free_bytes,
            &mut total_bytes,
            std::ptr::null_mut(),
        );
        if result != 0 {
            Some(free_bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else {
            None
        }
    }
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
