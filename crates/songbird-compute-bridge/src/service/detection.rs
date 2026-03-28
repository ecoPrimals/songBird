// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Host resource and capability detection.

use super::types::ServiceInfo;

/// Auto-detect system resources
pub async fn detect_resources() -> ServiceInfo {
    use std::process::Command;

    tokio::task::yield_now().await;

    // Detect CPU cores
    let cpu_cores = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);

    // Detect memory (Linux-specific, fallback to estimate)
    let memory_gb = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|contents| {
            contents
                .lines()
                .find(|line| line.starts_with("MemTotal:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|kb| kb.parse::<usize>().ok())
                .map(|kb| kb / 1024 / 1024) // Convert KB to GB
        })
        .unwrap_or(16); // Default estimate

    // Detect GPU (NVIDIA)
    let (gpu_count, gpu_model) = if let Ok(output) = Command::new("nvidia-smi")
        .args(["--query-gpu=name,count", "--format=csv,noheader"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let line = stdout.lines().next().unwrap_or("");
            let parts: Vec<&str> = line.split(',').collect();
            let model = parts.first().map(|s| s.trim().to_string());
            let count = parts.get(1).and_then(|s| s.trim().parse().ok()).unwrap_or(1);
            (count, model)
        } else {
            (0, None)
        }
    } else {
        (0, None)
    };

    let storage_gb = songbird_process_env::var("COMPUTE_STORAGE_GB")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .or_else(detect_disk_gb);

    ServiceInfo {
        cpu_cores,
        memory_gb,
        gpu_count,
        gpu_model,
        storage_gb,
        platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
    }
}

/// Detect available disk space in GB from the filesystem (Linux `statvfs`-based, with fallback).
fn detect_disk_gb() -> Option<usize> {
    std::fs::read_to_string("/proc/mounts").ok().and_then(|mounts| {
        let root_mount =
            mounts.lines().find(|l| l.split_whitespace().nth(1).is_some_and(|mp| mp == "/"));
        root_mount.and_then(|_| {
            let stat = std::process::Command::new("df")
                .args(["--output=avail", "-B1G", "/"])
                .output()
                .ok()?;
            if !stat.status.success() {
                return None;
            }
            let out = String::from_utf8_lossy(&stat.stdout);
            out.lines().nth(1).and_then(|line| line.trim().parse::<usize>().ok())
        })
    })
}

/// Auto-detect capabilities based on resources
pub fn detect_capabilities(info: &ServiceInfo) -> String {
    let mut caps = vec!["compute".to_string(), "cpu".to_string()];

    if info.gpu_count > 0 {
        caps.push("gpu".to_string());
        caps.push("ml-inference".to_string());
    }

    if info.cpu_cores >= 8 {
        caps.push("batch-processing".to_string());
    }

    if info.cpu_cores >= 32 {
        caps.push("parallel-computing".to_string());
    }

    caps.join(",")
}
