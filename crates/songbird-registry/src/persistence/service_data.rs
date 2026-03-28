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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use songbird_discovery::traits::service::{ServiceInfo, ServiceStatus};
    use std::collections::HashMap;

    fn minimal_service_info() -> ServiceInfo {
        let now = Utc::now();
        ServiceInfo {
            service_id: "id-1".to_string(),
            name: "svc".to_string(),
            version: "1.0.0".to_string(),
            service_type: "test".to_string(),
            description: None,
            endpoints: vec![],
            health_check_endpoint: None,
            metadata: HashMap::new(),
            tags: vec![],
            dependencies: vec![],
            status: ServiceStatus::Running,
            created_at: now,
            updated_at: now,
            instance_id: "inst-1".to_string(),
            host: "127.0.0.1".to_string(),
            port: 8080,
        }
    }

    fn assert_json_roundtrip<T>(value: &T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let j = serde_json::to_string(value).unwrap();
        let back: T = serde_json::from_str(&j).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), j);
    }

    #[test]
    fn registry_health_status_roundtrip() {
        for s in [
            RegistryHealthStatus::Unknown,
            RegistryHealthStatus::Healthy,
            RegistryHealthStatus::Degraded,
            RegistryHealthStatus::Unhealthy,
        ] {
            assert_json_roundtrip(&s);
        }
    }

    #[test]
    fn registry_service_metrics_default_and_roundtrip() {
        let m = RegistryServiceMetrics::default();
        assert_eq!(m.cpu_utilization, 0.0);
        assert_json_roundtrip(&m);
    }

    #[test]
    fn registry_service_entry_roundtrip() {
        let entry = RegistryServiceEntry {
            service_info: minimal_service_info(),
            instance_count: 2,
            max_instances: 10,
            min_instances: 1,
            health_status: RegistryHealthStatus::Healthy,
            metrics: RegistryServiceMetrics {
                cpu_utilization: 0.42,
                memory_utilization: 0.5,
                request_rate: 100.0,
                response_time_ms: 12.0,
                error_rate: 0.01,
                active_connections: 3,
                queue_depth: 0,
            },
        };
        assert_json_roundtrip(&entry);
    }
}
