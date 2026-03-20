// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Serializable service snapshot types for [`super::production_storage::ProductionServicePersistence`].

use serde::{Deserialize, Serialize};
pub use songbird_discovery::traits::service::ServiceInfo;

/// Health classification persisted with a service entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegistryHealthStatus {
    /// Health not yet determined.
    Unknown,
    /// Service is healthy.
    Healthy,
    /// Service is degraded but usable.
    Degraded,
    /// Service is unhealthy.
    Unhealthy,
}

/// Lightweight metrics block for persistence (distinct from discovery's runtime metrics).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryServiceMetrics {
    /// CPU utilization ratio 0.0–1.0
    pub cpu_utilization: f64,
    /// Memory utilization ratio 0.0–1.0
    pub memory_utilization: f64,
    /// Observed request rate
    pub request_rate: f64,
    /// Average response time in milliseconds
    pub response_time_ms: f64,
    /// Error rate ratio 0.0–1.0
    pub error_rate: f64,
    /// Active connection count
    pub active_connections: u32,
    /// Queued work depth
    pub queue_depth: u32,
}

impl Default for RegistryServiceMetrics {
    fn default() -> Self {
        Self {
            cpu_utilization: 0.0,
            memory_utilization: 0.0,
            request_rate: 0.0,
            response_time_ms: 0.0,
            error_rate: 0.0,
            active_connections: 0,
            queue_depth: 0,
        }
    }
}

/// Full registry row: [`ServiceInfo`] plus scaling / health snapshot fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryServiceEntry {
    /// Core discovery-facing metadata
    pub service_info: ServiceInfo,
    /// Active instances last observed for this logical service.
    pub instance_count: u32,
    /// Upper bound instances may scale to under load.
    pub max_instances: u32,
    /// Lower bound instances should not scale below.
    pub min_instances: u32,
    /// Persisted health snapshot for orchestration decisions.
    pub health_status: RegistryHealthStatus,
    /// Persisted utilization metrics for scaling heuristics.
    pub metrics: RegistryServiceMetrics,
}
