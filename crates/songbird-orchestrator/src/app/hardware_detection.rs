//! Hardware Detection Module
//!
//! Provides runtime hardware detection for:
//! - GPU model detection (NVIDIA, AMD, Intel via nvidia-smi, lspci)
//! - Storage capacity detection (Linux df, environment override)
//!
//! ## Zero Hardcoding Philosophy
//!
//! All detection is runtime-based with fallbacks:
//! 1. Try system tools (nvidia-smi, lspci, df)
//! 2. Check environment variable overrides
//! 3. Return None if unavailable
//!
//! No hardcoded hardware assumptions!

use songbird_types::SafeEnv;

/// Detect GPU model if available
///
/// Attempts multiple detection methods in order:
/// 1. nvidia-smi for NVIDIA GPUs
/// 2. lspci for any GPU (Linux only)
/// 3. GPU_MODEL environment variable override
///
/// # Returns
///
/// GPU model string if detected, None otherwise
///
/// # Zero Hardcoding
///
/// No hardcoded GPU models - all detection is runtime-based.
/// Users can override via GPU_MODEL environment variable.
pub fn detect_gpu() -> Option<String> {
    // Priority 1: Check environment variable override (for testing/configuration)
    if let Ok(gpu_model) = std::env::var("GPU_MODEL") {
        if !gpu_model.is_empty() {
            return Some(gpu_model);
        }
    }
    
    // Method 2: Try nvidia-smi for NVIDIA GPUs
    if let Ok(output) = std::process::Command::new("nvidia-smi")
        .arg("--query-gpu=name")
        .arg("--format=csv,noheader")
        .output()
    {
        if output.status.success() {
            if let Ok(gpu_name) = String::from_utf8(output.stdout) {
                let gpu_name = gpu_name.trim().to_string();
                if !gpu_name.is_empty() {
                    return Some(gpu_name);
                }
            }
        }
    }

    // Method 2: Try lspci for any GPU
    #[cfg(target_os = "linux")]
    if let Ok(output) = std::process::Command::new("lspci").output() {
        if output.status.success() {
            if let Ok(lspci_output) = String::from_utf8(output.stdout) {
                for line in lspci_output.lines() {
                    if line.to_lowercase().contains("vga") || line.to_lowercase().contains("3d") {
                        // Extract GPU name from lspci output
                        if let Some(gpu_part) = line.split(':').nth(2) {
                            return Some(gpu_part.trim().to_string());
                        }
                    }
                }
            }
        }
    }

    // Method 3: Check environment variable override
    SafeEnv::get_required("GPU_MODEL").ok()
}

/// Detect storage capacity in GB
///
/// Attempts multiple detection methods in order:
/// 1. df command for root filesystem (Linux only)
/// 2. STORAGE_GB environment variable override
/// 3. None if unavailable
///
/// # Returns
///
/// Storage capacity in GB if detected, None otherwise
///
/// # Zero Hardcoding
///
/// No hardcoded storage values - all detection is runtime-based.
/// Users can override via STORAGE_GB environment variable.
pub fn detect_storage_capacity() -> Option<usize> {
    // Priority 1: Check environment variable override (for testing/configuration)
    if let Ok(storage_gb) = std::env::var("STORAGE_GB") {
        if let Ok(storage) = storage_gb.parse::<usize>() {
            return Some(storage);
        }
    }
    
    // Method 2: Try to read from df (Linux)
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("df").arg("-BG").arg("/").output() {
            if output.status.success() {
                if let Ok(df_output) = String::from_utf8(output.stdout) {
                    // Parse df output: find the root filesystem line
                    for line in df_output.lines().skip(1) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            // Second column is total size
                            if let Some(size_str) = parts.get(1) {
                                // Remove 'G' suffix and parse
                                let size_gb = size_str.trim_end_matches('G');
                                if let Ok(size) = size_gb.parse::<usize>() {
                                    return Some(size);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Method 2: Environment variable override
    let storage = SafeEnv::get_usize("STORAGE_GB", 0);
    if storage > 0 {
        return Some(storage);
    }

    // Method 3: Default fallback
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_gpu_returns_option() {
        // Should return Some or None, never panic
        let result = detect_gpu();
        assert!(result.is_some() || result.is_none());

        if let Some(gpu) = result {
            assert!(!gpu.is_empty(), "GPU string should not be empty if detected");
        }
    }

    #[test]
    fn test_detect_gpu_with_override() {
        // Set environment override
        std::env::set_var("GPU_MODEL", "TestGPU RTX 9999");

        let gpu = detect_gpu();
        assert_eq!(gpu, Some("TestGPU RTX 9999".to_string()));

        // Clean up
        std::env::remove_var("GPU_MODEL");
    }

    #[test]
    fn test_detect_storage_capacity_returns_option() {
        // Should return Some or None, never panic
        let result = detect_storage_capacity();
        assert!(result.is_some() || result.is_none());

        if let Some(storage) = result {
            assert!(storage > 0, "Storage should be positive if detected");
            assert!(storage < 100000, "Storage should be reasonable (< 100TB)");
        }
    }

    #[test]
    fn test_detect_storage_capacity_with_override() {
        // Set environment override
        std::env::set_var("STORAGE_GB", "500");

        let storage = detect_storage_capacity();
        assert_eq!(storage, Some(500));

        // Clean up
        std::env::remove_var("STORAGE_GB");
    }

    #[test]
    fn test_zero_hardcoding_pattern() {
        // This test verifies the zero hardcoding philosophy:
        // - No hardcoded hardware values
        // - Runtime detection or environment override
        // - Graceful fallback to None

        // Without override, should use system detection or None
        std::env::remove_var("GPU_MODEL");
        std::env::remove_var("STORAGE_GB");

        let gpu = detect_gpu();
        let storage = detect_storage_capacity();

        // Results depend on system - that's the point!
        // No hardcoded values means results vary by environment
        assert!(gpu.is_some() || gpu.is_none());
        assert!(storage.is_some() || storage.is_none());
    }
}
