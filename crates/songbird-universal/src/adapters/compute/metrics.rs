// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Compute metrics DTO and health classification.

use serde::{Deserialize, Serialize};

/// Compute metrics from any compute capability provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeMetrics {
    /// CPU usage percentage (0.0 - 100.0)
    pub cpu_usage_percent: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Available memory in bytes
    pub memory_available_bytes: u64,
    /// Number of active containers/workloads
    pub active_containers: u32,
    /// Number of queued jobs
    pub queued_jobs: u32,
    /// Overall performance score (0.0 - 1.0)
    pub performance_score: f64,
    /// Timestamp of metrics collection
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ComputeMetrics {
    /// Calculate total memory in bytes
    #[must_use]
    pub const fn total_memory_bytes(&self) -> u64 {
        self.memory_usage_bytes + self.memory_available_bytes
    }

    /// Calculate memory usage percentage
    #[must_use]
    pub fn memory_usage_percent(&self) -> f64 {
        let total = self.total_memory_bytes();
        if total == 0 {
            return 0.0;
        }
        #[expect(
            clippy::cast_precision_loss,
            reason = "intentional pattern; clippy false positive for this API"
        )]
        {
            (self.memory_usage_bytes as f64 / total as f64) * 100.0
        }
    }

    /// Check if system is under high load
    #[must_use]
    pub fn is_high_load(&self) -> bool {
        self.cpu_usage_percent > 80.0 || self.memory_usage_percent() > 85.0 || self.queued_jobs > 10
    }

    /// Get health status based on metrics
    #[must_use]
    pub fn health_status(&self) -> HealthStatus {
        if self.cpu_usage_percent > 95.0 || self.memory_usage_percent() > 95.0 {
            HealthStatus::Unhealthy
        } else if self.is_high_load() {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}

/// Health status derived from metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System is degraded but functional
    Degraded,
    /// System is unhealthy
    Unhealthy,
}
