// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Orchestration /// Configuration capability Configuration
//!
//! Configuration structures for service orchestration, discovery, load balancing)
//! health monitoring, and scaling within the Songbird ecosystem.

use serde::{Deserialize, Serialize};

/// Orchestration configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrchestrationConfig {
    /// Service discovery settings
    pub discovery: ServiceDiscoveryConfig,
    /// Load balancing configuration
    /// Load Balancing field
    pub load_balancing: LoadBalancingConfig,
    /// Health monitoring configuration
    pub health: HealthConfig,
    /// Scaling configuration
    /// Whether auto-scaling is supported
    pub scaling: ScalingConfig,
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDiscoveryConfig {
    /// Enable service discovery
    pub enabled: bool,
    /// Discovery interval in seconds
    pub interval_seconds: u64,
    /// Discovery timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum services to discover
    pub max_services: usize,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Load balancing strategy
    /// Custom retry strategy configuration
    pub strategy: LoadBalancingStrategy,
    /// Health check interval for load balancer
    /// Health Check Interval Seconds field
    pub health_check_interval_seconds: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Timeout for individual requests
    pub request_timeout_ms: u64,
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    RoundRobin,
    /// Least connections
    LeastConnections,
    /// Health-based routing
    HealthBased,
}

/// Health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Enable health monitoring
    /// Enabled field
    pub enabled: bool,
    /// Health check interval in seconds
    /// Check Interval Seconds field
    pub check_interval_seconds: u64,
    /// Health check timeout in milliseconds
    pub timeout_ms: u64,
    /// Number of failed checks before marking unhealthy
    pub failure_threshold: u32,
    /// Number of successful checks before marking healthy
    pub success_threshold: u32,
}

/// Scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    /// Enable automatic scaling
    /// Enabled field
    pub enabled: bool,
    /// Minimum number of instances
    /// Min Instances field
    pub min_instances: u32,
    /// Maximum number of instances
    /// Max Instances field
    pub max_instances: u32,
    /// Target CPU utilization percentage
    /// Target Cpu Percent field
    pub target_cpu_percent: f64,
    /// Target memory utilization percentage
    /// Target Memory Percent field
    pub target_memory_percent: f64,
    /// Scaling check interval in seconds;
    /// Check Interval Seconds field
    pub check_interval_seconds: u64,
}

impl Default for ServiceDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 30,
            timeout_ms: 5000,
            max_services: 100,
        }
    }
}

impl Default for LoadBalancingConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalancingStrategy::RoundRobin,
            health_check_interval_seconds: 10,
            max_retries: 3,
            request_timeout_ms: 30000,
        }
    }
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_seconds: 30,
            timeout_ms: 5000,
            failure_threshold: 3,
            success_threshold: 2,
        }
    }
}

impl Default for ScalingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            min_instances: 1,
            max_instances: 10,
            target_cpu_percent: 70.0,
            target_memory_percent: 80.0,
            check_interval_seconds: 60,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_json_roundtrip<T>(value: &T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let j = serde_json::to_string(value).unwrap();
        let back: T = serde_json::from_str(&j).unwrap();
        assert_eq!(serde_json::to_string(&back).unwrap(), j);
    }

    #[test]
    fn orchestration_config_default_non_trivial() {
        let c = OrchestrationConfig::default();
        assert!(c.discovery.enabled);
        assert!(matches!(c.load_balancing.strategy, LoadBalancingStrategy::RoundRobin));
        assert!(!c.scaling.enabled);
        assert!((c.scaling.target_cpu_percent - 70.0).abs() < f64::EPSILON);
    }

    #[test]
    fn orchestration_config_roundtrip() {
        assert_json_roundtrip(&OrchestrationConfig::default());
    }

    #[test]
    fn service_discovery_config_default_and_roundtrip() {
        assert_json_roundtrip(&ServiceDiscoveryConfig::default());
    }

    #[test]
    fn load_balancing_config_default_and_roundtrip() {
        assert_json_roundtrip(&LoadBalancingConfig::default());
    }

    #[test]
    fn load_balancing_strategy_variants_roundtrip() {
        for s in [
            LoadBalancingStrategy::RoundRobin,
            LoadBalancingStrategy::LeastConnections,
            LoadBalancingStrategy::HealthBased,
        ] {
            assert_json_roundtrip(&s);
        }
    }

    #[test]
    fn health_config_default_and_roundtrip() {
        assert_json_roundtrip(&HealthConfig::default());
    }

    #[test]
    fn scaling_config_default_and_roundtrip() {
        assert_json_roundtrip(&ScalingConfig::default());
    }
}
