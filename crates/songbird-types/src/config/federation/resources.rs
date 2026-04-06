// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Resource management: monitoring thresholds and allocation strategy for federated workloads.

use serde::{Deserialize, Serialize};

use super::limits::CanonicalResourceLimits;

/// Resource management configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceManagementConfig {
    /// Resource limits
    pub limits: CanonicalResourceLimits,
    /// Resource monitoring
    /// Monitoring field
    pub monitoring: ResourceMonitoringConfig,
    /// Resource allocation
    /// Allocation field
    pub allocation: ResourceAllocationConfig,
}

/// Resource monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMonitoringConfig {
    /// Enable monitoring
    pub enabled: bool,
    /// Monitoring interval in seconds
    pub interval: u64,
    /// Alert thresholds
    pub thresholds: ResourceThresholds,
}

impl Default for ResourceMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: 30,
            thresholds: ResourceThresholds::default(),
        }
    }
}

/// Resource thresholds configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceThresholds {
    /// Memory usage threshold (0.0-1.0)
    /// Memory field
    pub memory: f64,
    /// CPU usage threshold (0.0-1.0)
    /// Cpu field
    pub cpu: f64,
    /// Disk usage threshold (0.0-1.0)
    /// Disk field
    pub disk: f64,
}

impl Default for ResourceThresholds {
    fn default() -> Self {
        Self {
            memory: 0.8,
            cpu: 0.8,
            disk: 0.9,
        }
    }
}

/// Resource allocation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocationConfig {
    /// Allocation strategy
    pub strategy: AllocationStrategy,
    /// Reserved resources
    pub reserved: ReservedResources,
}

impl Default for ResourceAllocationConfig {
    fn default() -> Self {
        Self {
            strategy: AllocationStrategy::Balanced,
            reserved: ReservedResources::default(),
        }
    }
}

/// Allocation strategy enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AllocationStrategy {
    /// Balanced allocation
    Balanced,
    /// CPU-optimized allocation
    CpuOptimized,
    /// Memory-optimized allocation
    MemoryOptimized,
    /// Custom allocation strategy
    Custom(String),
}

/// Reserved resources configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReservedResources {
    /// Reserved memory in bytes
    pub memory: u64,
    /// Reserved CPU percentage (0.0-1.0)
    pub cpu: f64,
    /// Reserved disk space in bytes
    pub disk: u64,
}

impl Default for ReservedResources {
    fn default() -> Self {
        Self {
            memory: 512 * 1024 * 1024, // 512MB
            cpu: 0.1,                  // 10%
            disk: 1024 * 1024 * 1024,  // 1GB
        }
    }
}
