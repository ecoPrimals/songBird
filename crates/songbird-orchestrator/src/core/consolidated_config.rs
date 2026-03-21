// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Consolidated orchestrator configuration types and related defaults.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::api::ApiConfig;
use songbird_config::canonical::resilience::LoadBalancerConfig as CanonicalLoadBalancerConfig;

/// Consolidated orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsolidatedOrchestratorConfig {
    /// Load balancing configuration
    /// **CONSOLIDATED**: Now uses `CanonicalLoadBalancerConfig` from songbird-config
    pub load_balancing: CanonicalLoadBalancerConfig,

    /// Performance monitoring configuration
    pub performance: PerformanceConfig,

    /// Service registry configuration
    pub registry: RegistryConfig,

    /// Auto-scaling configuration
    pub scaling: ScalingConfig,

    /// API configuration
    pub api: ApiConfig,

    /// Zero-touch deployment configuration
    pub zero_touch: ZeroTouchConfig,
}

// ============================================================================
// NOTE: LoadBalancingConfig has been CONSOLIDATED
// ============================================================================
//
// LoadBalancingConfig was removed and replaced with CanonicalLoadBalancerConfig
// from songbird_config::canonical::resilience::LoadBalancerConfig
//
// Migration: Use CanonicalLoadBalancerConfig instead
// - strategy (LoadBalancingStrategy) → algorithm (LoadBalancingAlgorithm)
// - health_check_interval (u64) → health_check.interval (HealthCheckConfig field)
// - max_retries → handled at usage site or via RetryConfig
//
// NEW comprehensive fields available:
// - sticky_sessions: bool - Enable session affinity (default: false)
// - session_timeout: Duration - Session timeout (default: 300s)
// - max_connections_per_backend: usize - Connection pooling (default: 100)
// - connection_timeout: Duration - Connection timeout (default: 30s)
// - fail_fast: bool - Enable fail-fast mode (default: false)
//
// Date: November 10, 2025
// ============================================================================

/// Performance monitoring configuration
///
/// **ZERO-COPY OPTIMIZATION** (Dec 8, 2025):
/// Uses `Arc<str>` for threshold keys to avoid cloning in hot paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub metrics_interval: u64,
    #[serde(with = "arc_str_map_serde")]
    pub alert_thresholds: HashMap<Arc<str>, f64>,
    pub enable_benchmarking: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert(Arc::from("cpu_usage"), 80.0);
        thresholds.insert(Arc::from("memory_usage"), 85.0);
        thresholds.insert(Arc::from("response_time"), 1000.0);
        Self {
            metrics_interval: 60,
            alert_thresholds: thresholds,
            enable_benchmarking: true,
        }
    }
}

/// Serde helper for `Arc<str>` `HashMap` keys
mod arc_str_map_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::collections::HashMap;
    use std::sync::Arc;

    pub fn serialize<S>(map: &HashMap<Arc<str>, f64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        let mut ser_map = serializer.serialize_map(Some(map.len()))?;
        for (k, v) in map {
            ser_map.serialize_entry(k.as_ref(), v)?;
        }
        ser_map.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<Arc<str>, f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = HashMap::<String, f64>::deserialize(deserializer)?;
        Ok(map.into_iter().map(|(k, v)| (Arc::from(k.as_str()), v)).collect())
    }
}

/// Service registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub discovery_interval: u64,
    pub service_timeout: u64,
    pub max_services: u32,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            discovery_interval: 30,
            service_timeout: 300,
            max_services: 1000,
        }
    }
}

/// Auto-scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    pub enable_auto_scaling: bool,
    pub scale_up_threshold: f64,
    pub scale_down_threshold: f64,
    pub min_instances: u32,
    pub max_instances: u32,
}

impl Default for ScalingConfig {
    fn default() -> Self {
        Self {
            enable_auto_scaling: true,
            scale_up_threshold: 70.0,
            scale_down_threshold: 30.0,
            min_instances: 1,
            max_instances: 10,
        }
    }
}

/// Zero-touch deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroTouchConfig {
    pub enable_auto_deployment: bool,
    pub deployment_strategy: DeploymentStrategy,
    pub rollback_on_failure: bool,
}

impl Default for ZeroTouchConfig {
    fn default() -> Self {
        Self {
            enable_auto_deployment: false,
            deployment_strategy: DeploymentStrategy::BlueGreen,
            rollback_on_failure: true,
        }
    }
}

/// Deployment strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeploymentStrategy {
    BlueGreen,
    RollingUpdate,
    Canary,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::{
        ConsolidatedOrchestratorConfig, DeploymentStrategy, PerformanceConfig, RegistryConfig,
        ScalingConfig, ZeroTouchConfig,
    };
    use std::sync::Arc;

    #[test]
    fn consolidated_default_is_serializable() {
        let c = ConsolidatedOrchestratorConfig::default();
        let j = serde_json::to_string(&c).unwrap();
        let back: ConsolidatedOrchestratorConfig = serde_json::from_str(&j).unwrap();
        assert_eq!(c.performance.metrics_interval, back.performance.metrics_interval);
    }

    #[test]
    fn performance_config_default_thresholds() {
        let p = PerformanceConfig::default();
        assert_eq!(p.metrics_interval, 60);
        assert!(p.enable_benchmarking);
        assert!(p.alert_thresholds.contains_key(&Arc::from("cpu_usage")));
    }

    #[test]
    fn registry_config_default() {
        let r = RegistryConfig::default();
        assert_eq!(r.discovery_interval, 30);
        assert_eq!(r.max_services, 1000);
    }

    #[test]
    fn scaling_config_default() {
        let s = ScalingConfig::default();
        assert!(s.enable_auto_scaling);
        assert_eq!(s.min_instances, 1);
        assert_eq!(s.max_instances, 10);
    }

    #[test]
    fn zero_touch_default() {
        let z = ZeroTouchConfig::default();
        assert!(!z.enable_auto_deployment);
        assert_eq!(z.deployment_strategy, DeploymentStrategy::BlueGreen);
        assert!(z.rollback_on_failure);
    }

    #[test]
    fn deployment_strategy_serde_roundtrip() {
        for d in [
            DeploymentStrategy::BlueGreen,
            DeploymentStrategy::RollingUpdate,
            DeploymentStrategy::Canary,
        ] {
            let j = serde_json::to_string(&d).unwrap();
            let back: DeploymentStrategy = serde_json::from_str(&j).unwrap();
            assert_eq!(d, back);
        }
    }

    #[test]
    fn performance_config_serde_roundtrip() {
        let p = PerformanceConfig::default();
        let j = serde_json::to_string(&p).unwrap();
        let back: PerformanceConfig = serde_json::from_str(&j).unwrap();
        assert_eq!(p.metrics_interval, back.metrics_interval);
        assert_eq!(p.alert_thresholds.len(), back.alert_thresholds.len());
    }
}
