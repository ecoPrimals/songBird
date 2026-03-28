// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! System Resource Detection API
//!
//! Headless API for detecting system resources that biomeOS can consume

#![allow(missing_docs, reason = "resource detection JSON DTOs mirror biomeOS schema")]

use super::{NetworkSpeed, SystemResources};
use crate::errors::SongbirdResult;
use serde::{Deserialize, Serialize};

/// Detect system resources via API
pub async fn detect_system_resources_api() -> SongbirdResult<SystemResources> {
    // Use the parameterized version with default settings for full detection
    let request = ResourceDetectionRequest {
        detect_gpu: true,
        detect_storage: true,
        network_test: true,
    };

    detect_resources_with_params(request).await
}

/// Detect system resources with selective detection for performance (used for light resource checks,
#[allow(dead_code, reason = "fast resource detection reserved for CLI quick-check subcommand")]
pub async fn detect_system_resources_fast() -> SongbirdResult<SystemResources> {
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
) -> SongbirdResult<SystemResources> {
    let cpu_cores = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
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
    songbird_types::sys_metrics::memory_info().map_or(16.0, |m| m.available_gb())
}

/// Pure Rust disk space query via `/sys/block/` (ecoBin v3.0).
fn get_available_disk_space_safe() -> Option<f64> {
    songbird_types::sys_metrics::total_disk_gb().map(|gb| gb as f64)
}

/// Get available storage space in GB
#[must_use]
pub fn get_available_storage() -> Option<f64> {
    get_available_disk_space_safe()
}

fn detect_gpu_availability() -> bool {
    // Check for NVIDIA GPU
    if std::process::Command::new("nvidia-smi")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        return true;
    }

    // Check for AMD GPU
    if std::process::Command::new("rocm-smi")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        return true;
    }

    // Check for Intel GPU
    if std::process::Command::new("intel_gpu_top")
        .arg("--help")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
    {
        return true;
    }

    false
}

async fn detect_network_speed() -> NetworkSpeed {
    if let Ok(speed_str) = songbird_process_env::var("SONGBIRD_NETWORK_SPEED") {
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
