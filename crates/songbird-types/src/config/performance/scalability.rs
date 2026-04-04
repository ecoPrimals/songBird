// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Auto-scaling, load balancing, and health-check types for performance tuning.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Scalability configuration - consolidated from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalabilityConfig {
    /// Enable auto-scaling
    /// Auto Scaling Enabled field
    pub auto_scaling_enabled: bool,
    /// Minimum instances
    /// Min Instances field
    pub min_instances: u32,
    /// Maximum instances
    /// Max Instances field
    pub max_instances: u32,
    /// CPU threshold for scaling up (percentage)
    /// Scale Up Cpu Threshold field
    pub scale_up_cpu_threshold: f64,
    /// CPU threshold for scaling down (percentage)
    /// Scale Down Cpu Threshold field
    pub scale_down_cpu_threshold: f64,
    /// Memory threshold for scaling up (percentage)
    /// Scale Up Memory Threshold field
    pub scale_up_memory_threshold: f64,
    /// Memory threshold for scaling down (percentage)
    /// Scale Down Memory Threshold field
    pub scale_down_memory_threshold: f64,
    /// Scaling cooldown period
    /// Scaling Cooldown field
    pub scaling_cooldown: Duration,
    /// Load balancing configuration
    /// Load Balancing field
    pub load_balancing: LoadBalancingConfig,
}

impl Default for ScalabilityConfig {
    fn default() -> Self {
        Self {
            auto_scaling_enabled: true,
            min_instances: 1,
            max_instances: 10,
            scale_up_cpu_threshold: 70.0,
            scale_down_cpu_threshold: 30.0,
            scale_up_memory_threshold: 80.0,
            scale_down_memory_threshold: 40.0,
            scaling_cooldown: Duration::from_secs(300), // 5 minutes
            load_balancing: LoadBalancingConfig::default(),
        }
    }
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Enable load balancing
    /// Enabled field
    pub enabled: bool,
    /// Load balancing algorithm
    pub algorithm: LoadBalancingAlgorithm,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Session affinity enabled
    /// Session Affinity field
    pub session_affinity: bool,
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: LoadBalancingAlgorithm::RoundRobin,
            health_check: HealthCheckConfig::default(),
            session_affinity: false,
        }
    }
}

/// Load balancing algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    /// Round-robin distribution
    RoundRobin,
    /// Least connections
    LeastConnections,
    /// Weighted round-robin
    WeightedRoundRobin,
    /// IP hash
    IpHash,
}

/// Health check configuration
///
/// **NOTE** (Week 2, Nov 10 2025): Kept in types crate (doesn't depend on config).
/// Fields aligned with canonical naming where possible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Enable health checks
    pub enabled: bool,
    /// Health check interval
    pub interval: Duration,
    /// Health check timeout
    pub timeout: Duration,
    /// Failure threshold (consecutive failures)
    pub failure_threshold: u32,
    /// Recovery threshold (consecutive successes)
    pub recovery_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
            failure_threshold: 3,
            recovery_threshold: 2,
        }
    }
}
