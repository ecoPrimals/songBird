// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Aggregated orchestrator and component health snapshot types.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Orchestrator health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorHealth {
    pub status: HealthStatus,
    pub load_balancer_health: ComponentHealth,
    pub performance_health: ComponentHealth,
    pub registry_health: ComponentHealth,
    pub scaling_health: ComponentHealth,
}

/// Overall health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Component health status
///
/// **ZERO-COPY OPTIMIZATION** (Dec 8, 2025):
/// Message uses `Arc<str>` to avoid string clones when health data is shared.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    #[serde(with = "arc_str_option_serde")]
    pub message: Option<Arc<str>>,
    pub last_check: Option<u64>,
}

/// Serde helper for `Option<Arc<str>>`
mod arc_str_option_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::sync::Arc;

    #[allow(clippy::ref_option, reason = "intentional pattern; clippy false positive for this API")]
    pub fn serialize<S>(value: &Option<Arc<str>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(s) => serializer.serialize_some(s.as_ref()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Arc<str>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Option::<String>::deserialize(deserializer)?;
        Ok(s.map(|s| Arc::from(s.as_str())))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::{ComponentHealth, HealthStatus, OrchestratorHealth};
    use std::sync::Arc;

    #[test]
    fn health_status_serde_roundtrip() {
        for s in [
            HealthStatus::Healthy,
            HealthStatus::Degraded,
            HealthStatus::Unhealthy,
            HealthStatus::Unknown,
        ] {
            let j = serde_json::to_string(&s).unwrap();
            let back: HealthStatus = serde_json::from_str(&j).unwrap();
            assert_eq!(s, back);
        }
    }

    #[test]
    fn component_health_serde_with_message() {
        let c = ComponentHealth {
            status: HealthStatus::Degraded,
            message: Some(Arc::from("slow")),
            last_check: Some(99),
        };
        let j = serde_json::to_string(&c).unwrap();
        let back: ComponentHealth = serde_json::from_str(&j).unwrap();
        assert_eq!(c.status, back.status);
        assert_eq!(c.message.as_deref(), back.message.as_deref());
        assert_eq!(c.last_check, back.last_check);
    }

    #[test]
    fn component_health_serde_none_message() {
        let c = ComponentHealth {
            status: HealthStatus::Healthy,
            message: None,
            last_check: None,
        };
        let j = serde_json::to_string(&c).unwrap();
        let back: ComponentHealth = serde_json::from_str(&j).unwrap();
        assert_eq!(c.status, back.status);
        assert!(back.message.is_none());
    }

    #[test]
    fn orchestrator_health_roundtrip() {
        let h = OrchestratorHealth {
            status: HealthStatus::Healthy,
            load_balancer_health: ComponentHealth {
                status: HealthStatus::Healthy,
                message: None,
                last_check: Some(1),
            },
            performance_health: ComponentHealth {
                status: HealthStatus::Unknown,
                message: Some(Arc::from("?")),
                last_check: None,
            },
            registry_health: ComponentHealth {
                status: HealthStatus::Degraded,
                message: None,
                last_check: Some(2),
            },
            scaling_health: ComponentHealth {
                status: HealthStatus::Unhealthy,
                message: Some(Arc::from("stuck")),
                last_check: Some(3),
            },
        };
        let j = serde_json::to_string(&h).unwrap();
        let back: OrchestratorHealth = serde_json::from_str(&j).unwrap();
        assert_eq!(h.status, back.status);
        assert_eq!(h.load_balancer_health.status, back.load_balancer_health.status);
    }
}
