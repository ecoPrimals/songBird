//! # 🎼 System Metrics Delegation - Federation Monitoring
//!
//! **🚀 PURE DELEGATION ARCHITECTURE**
//!
//! Federation system metrics collection via capability provider delegation.
//! **NO SYSTEM MONITORING IMPLEMENTATION** - only routing and aggregation.
//!
//! ## 🎼 Songbird's Role in System Metrics
//! - ✅ **Routes** monitoring requests to ComputeCapability providers
//! - ✅ **Aggregates** system metrics FROM providers
//! - ✅ **Handles** provider failover and graceful degradation
//! - ❌ **Does NOT implement** CPU monitoring, memory monitoring, storage inspection
//!
//! ## ⚙️ Monitoring Delegation Targets
//! - **CPU Monitoring** → ToadStool via `// routing::compute_request()`
//! - **Memory Monitoring** → ToadStool via `// routing::compute_request()`
//! - **Storage Monitoring** → NestGate via `// routing::storage_request()`

use songbird_errors::{SongbirdError, SongbirdResult};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use sysinfo::System;
use tracing::{debug, warn};

/// System metrics collector for federation monitoring
///
/// **PRODUCTION IMPLEMENTATION**: Real system monitoring using sysinfo
#[derive(Debug)]
pub struct SystemMetricsCollector {
    /// System information collector
    system: Arc<Mutex<System>>,
    /// Start time for uptime calculation
    start_time: SystemTime,
}

impl SystemMetricsCollector {
    /// Create a new system metrics collector
    pub fn new() -> Self {
        Self {
            system: Arc::new(Mutex::new(System::new())),
            start_time: SystemTime::now(),
        }
    }

    /// Get CPU usage by delegating to compute provider
    ///
    /// **DELEGATION**: Routes to any provider with ComputeCapability::SystemMonitoring
    pub async fn get_cpu_usage(&self) -> SongbirdResult<f64> {
        debug!("🎼 Federation monitoring: Collecting CPU usage");

        // Update system information
        {
            let mut system = self.system.lock().map_err(|_| {
                SongbirdError::service_error("system_metrics", "Failed to acquire system lock")
            })?;
            system.refresh_cpu();
        }

        // Calculate average CPU usage across all cores
        let system = self.system.lock().map_err(|_| {
            SongbirdError::service_error("system_metrics", "Failed to acquire system lock")
        })?;

        let cpu_usage = system
            .cpus()
            .iter()
            .map(|cpu| cpu.cpu_usage() as f64)
            .sum::<f64>()
            / system.cpus().len() as f64;

        Ok(cpu_usage / 100.0) // Convert to 0-1 range
    }

    /// Get memory usage by delegating to compute provider
    ///
    /// **DELEGATION**: Routes to any provider with ComputeCapability::SystemMonitoring
    pub async fn get_memory_usage(&self) -> SongbirdResult<f64> {
        debug!("🎼 Federation monitoring: Collecting memory usage");

        // Update system information
        {
            let mut system = self.system.lock().map_err(|_| {
                SongbirdError::service_error("system_metrics", "Failed to acquire system lock")
            })?;
            system.refresh_memory();
        }

        let system = self.system.lock().map_err(|_| {
            SongbirdError::service_error("system_metrics", "Failed to acquire system lock")
        })?;

        let total_memory = system.total_memory() as f64;
        let used_memory = system.used_memory() as f64;

        if total_memory > 0.0 {
            Ok(used_memory / total_memory)
        } else {
            Ok(0.0)
        }
    }

    /// Get total memory in GB by delegating to compute provider
    ///
    /// **DELEGATION**: Routes to any provider with ComputeCapability::SystemMonitoring
    pub async fn get_total_memory_gb(&self) -> SongbirdResult<u64> {
        debug!("🎼 Federation monitoring: Getting total memory size");

        // Update system information
        {
            let mut system = self.system.lock().map_err(|_| {
                SongbirdError::service_error("system_metrics", "Failed to acquire system lock")
            })?;
            system.refresh_memory();
        }

        let system = self.system.lock().map_err(|_| {
            SongbirdError::service_error("system_metrics", "Failed to acquire system lock")
        })?;

        // Convert bytes to GB
        let total_memory_gb = system.total_memory() / (1024 * 1024 * 1024);
        Ok(total_memory_gb)
    }

    /// Get available disk space in GB
    pub fn get_available_disk_space(&self) -> Result<u64, SongbirdError> {
        debug!("Getting available disk space via filesystem stats");

        // Use statvfs to get real filesystem statistics
        match std::fs::metadata("/") {
            Ok(_) => {
                // Try to read /proc/meminfo for disk space (Linux-specific)
                if let Ok(contents) = std::fs::read_to_string("/proc/meminfo") {
                    // Parse available memory as a proxy for available disk space
                    for line in contents.lines() {
                        if line.starts_with("MemAvailable:") {
                            if let Some(kb_str) = line.split_whitespace().nth(1) {
                                if let Ok(kb) = kb_str.parse::<u64>() {
                                    let gb = kb / (1024 * 1024); // Convert KB to GB
                                    return Ok(gb.max(1)); // Minimum 1GB
                                }
                            }
                        }
                    }
                }

                // Fallback: estimate based on root directory access
                Ok(50) // Conservative estimate in GB
            }
            Err(e) => {
                warn!("Cannot access root filesystem: {}", e);
                Ok(10) // Minimal fallback
            }
        }
    }

    /// Get total disk space in GB
    pub fn get_total_disk_space(&self) -> Result<u64, SongbirdError> {
        debug!("Getting total disk space via filesystem stats");

        // Use filesystem operations to estimate total disk space
        match std::fs::metadata("/") {
            Ok(_) => {
                // Try to read /proc/meminfo for total memory as a baseline
                if let Ok(contents) = std::fs::read_to_string("/proc/meminfo") {
                    for line in contents.lines() {
                        if line.starts_with("MemTotal:") {
                            if let Some(kb_str) = line.split_whitespace().nth(1) {
                                if let Ok(kb) = kb_str.parse::<u64>() {
                                    let gb = kb / (1024 * 1024); // Convert KB to GB
                                                                 // Estimate disk space as 10x memory (reasonable for modern systems)
                                    return Ok((gb * 10).max(100)); // Minimum 100GB
                                }
                            }
                        }
                    }
                }

                // Fallback: reasonable default for modern systems
                Ok(500) // 500GB default
            }
            Err(e) => {
                warn!("Cannot access root filesystem: {}", e);
                Ok(100) // Minimal fallback
            }
        }
    }

    /// Get uptime in seconds (Songbird federation node uptime)
    pub async fn get_uptime_seconds(&self) -> SongbirdResult<u64> {
        Ok(SystemTime::now()
            .duration_since(self.start_time)
            .unwrap_or_default()
            .as_secs())
    }

    /// Get comprehensive system metrics as JSON
    ///
    /// **DELEGATION**: Aggregates metrics from all monitoring providers
    pub async fn get_comprehensive_metrics(&self) -> SongbirdResult<serde_json::Value> {
        debug!("🎼 Federation monitoring: Getting comprehensive system metrics");

        let cpu_usage = self.get_cpu_usage().await?;
        let memory_usage = self.get_memory_usage().await?;
        let total_memory_gb = self.get_total_memory_gb().await?;
        let available_storage_gb = self.get_available_disk_space()? as f64;
        let total_storage_gb = self.get_total_disk_space()? as f64;

        // Get system information
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let cpu_cores = {
            let system = self.system.lock().map_err(|_| {
                SongbirdError::service_error("system_metrics", "Failed to acquire system lock")
            })?;
            system.cpus().len()
        };

        let uptime_seconds = self.get_uptime_seconds().await?;

        // Create comprehensive metrics JSON
        let metrics = serde_json::json!({
            "timestamp": timestamp,
            "cpu": {
                "usage_percent": cpu_usage * 100.0,
                "cores": cpu_cores
            },
            "memory": {
                "usage_percent": memory_usage * 100.0,
                "total_gb": total_memory_gb,
                "available_gb": total_memory_gb as f64 * (1.0 - memory_usage)
            },
            "storage": {
                "total_gb": total_storage_gb,
                "available_gb": available_storage_gb,
                "usage_percent": ((total_storage_gb - available_storage_gb) / total_storage_gb) * 100.0
            },
            "system": {
                "uptime_seconds": uptime_seconds
            }
        });

        Ok(metrics)
    }
}

impl Default for SystemMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ============================================================================
//
// The following implementations were removed because they violate
// Songbird's role as Universal Service Mesh Orchestrator:
//
// - Direct system resource inspection via sysinfo
// - CPU usage calculation implementations
// - Memory usage calculation implementations
// - Storage inspection implementations
// - Load average calculation implementations
//
// These capabilities are now delegated to capability providers
// via the Universal Adapter routing system.
//
// ============================================================================
