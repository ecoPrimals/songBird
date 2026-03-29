// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Health status and aggregate system health.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Summarize readiness for load balancers and orchestrators.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// Fully operational within SLO.
    Healthy,
    /// Partially degraded but still serving traffic.
    Degraded,
    /// Failing checks; should not receive new work.
    Unhealthy,
    /// Health could not be determined.
    Unknown,
}

/// Roll up component health for status pages and alerts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    /// Worst-case status across checks.
    pub overall_status: HealthStatus,
    /// Per-component health (database, queue, etc.).
    pub components: HashMap<String, HealthStatus>,
    /// Numeric gauges (latency, error rate) for dashboards.
    pub metrics: HashMap<String, f64>,
    /// When this snapshot was taken.
    pub last_check: SystemTime,
}
