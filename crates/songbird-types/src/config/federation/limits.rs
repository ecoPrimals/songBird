// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Federation-wide limits: node counts, message size, rate limits, and coarse resource caps.

use serde::{Deserialize, Serialize};

/// **CANONICAL**: Federation limits and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalFederationLimits {
    /// Maximum nodes in federation
    pub max_nodes: usize,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Rate limiting configuration
    pub rate_limits: CanonicalRateLimits,
    /// Resource limits
    /// Resource limitation configurations
    pub resource_limits: CanonicalResourceLimits,
}

impl Default for CanonicalFederationLimits {
    fn default() -> Self {
        Self {
            max_nodes: 10000,
            max_message_size: 1024 * 1024, // 1MB
            rate_limits: CanonicalRateLimits::default(),
            resource_limits: CanonicalResourceLimits::default(),
        }
    }
}

/// **CANONICAL**: Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalRateLimits {
    /// Requests per second per node
    /// Requests Per Second field
    pub requests_per_second: u32,
    /// Burst allowance
    pub burst_size: u32,
    /// Bandwidth limit in bytes per second
    /// Bandwidth Limit field
    pub bandwidth_limit: u64,
}

impl Default for CanonicalRateLimits {
    fn default() -> Self {
        Self {
            requests_per_second: 100,
            burst_size: 200,
            bandwidth_limit: 10 * 1024 * 1024, // 10 MB/s
        }
    }
}

/// **CANONICAL**: Resource limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalResourceLimits {
    /// Maximum memory usage in bytes
    pub memory_bytes: u64,
    /// Maximum CPU usage percentage (0.0-1.0)
    pub cpu_percentage: f64,
    /// Maximum disk usage in bytes
    pub disk_usage: u64,
}

impl Default for CanonicalResourceLimits {
    fn default() -> Self {
        Self {
            memory_bytes: 1024 * 1024 * 1024,    // 1GB
            cpu_percentage: 0.8,                 // 80%
            disk_usage: 10 * 1024 * 1024 * 1024, // 10GB
        }
    }
}
