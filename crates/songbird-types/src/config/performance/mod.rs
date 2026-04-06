// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Performance configuration — consolidated canonical types.
//!
//! Submodules group types by operational concern: memory/threading, CPU/I/O,
//! network buffers, caching, monitoring, and scalability.

mod caching;
mod cpu_io;
mod memory_threading;
mod monitoring;
mod network_perf;
mod scalability;

pub use caching::{CacheEvictionPolicy, CacheLayerConfig, CachingConfig};
pub use cpu_io::{
    CpuOptimizationConfig, CpuOptimizationFlags, IoOptimizationFlags, IoPerformanceConfig,
};
pub use memory_threading::{CanonicalMemoryConfig, CanonicalThreadingConfig};
pub use monitoring::{
    ApplicationMonitoringConfig, MetricAggregation, MetricConfig, MonitoringFeatures,
    PerformanceMonitoringConfig, SystemMonitoringConfig,
};
pub use network_perf::{BufferConfig, NetworkOptimizationLevel, NetworkPerformanceConfig};
pub use scalability::{
    HealthCheckConfig, LoadBalancingAlgorithm, LoadBalancingConfig, ScalabilityConfig,
};

use serde::{Deserialize, Serialize};

/// Rust-specific optimization features
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RustOptimizationFeatures {
    /// Zero-cost abstractions
    /// Zero Cost Abstractions field
    pub zero_cost_abstractions: bool,
    /// Native async traits
    /// Native Async field
    pub native_async: bool,
    /// Generic specialization
    /// Generic Specialization field
    pub generic_specialization: bool,
}

/// **CANONICAL**: Comprehensive Performance Configuration - Single Source of Truth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalPerformanceConfig {
    /// Enable performance optimizations globally
    /// Enabled field
    pub enabled: bool,

    /// Rust-specific optimization features
    /// Rust Optimizations field
    pub rust_optimizations: RustOptimizationFeatures,
    /// Memory optimization settings
    pub memory: CanonicalMemoryConfig,
    /// Threading configuration
    /// Threading field
    pub threading: CanonicalThreadingConfig,
    /// CPU optimization settings
    pub cpu: CpuOptimizationConfig,
    /// I/O performance settings
    pub io: IoPerformanceConfig,
    /// Network performance settings
    pub network: NetworkPerformanceConfig,
    /// Caching configuration
    /// Caching field
    pub caching: CachingConfig,
    /// Monitoring and metrics
    /// Monitoring field
    pub monitoring: PerformanceMonitoringConfig,
    /// Scalability settings
    pub scalability: ScalabilityConfig,
}

impl Default for CanonicalPerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rust_optimizations: RustOptimizationFeatures {
                zero_cost_abstractions: true,
                native_async: true,
                generic_specialization: true,
            },
            memory: CanonicalMemoryConfig::default(),
            threading: CanonicalThreadingConfig::default(),
            cpu: CpuOptimizationConfig::default(),
            io: IoPerformanceConfig::default(),
            network: NetworkPerformanceConfig::default(),
            caching: CachingConfig::default(),
            monitoring: PerformanceMonitoringConfig::default(),
            scalability: ScalabilityConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::time::Duration;

    fn roundtrip<T>(v: &T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        let a: Value = serde_json::to_value(v).expect("serialize");
        let back: T = serde_json::from_value(a.clone()).expect("deserialize");
        assert_eq!(serde_json::to_value(&back).expect("serialize again"), a);
    }

    #[test]
    fn default_rust_optimization_features() {
        let r = RustOptimizationFeatures::default();
        assert!(!r.zero_cost_abstractions);
        assert!(!r.native_async);
    }

    #[test]
    fn default_canonical_performance_config() {
        let c = CanonicalPerformanceConfig::default();
        assert!(c.enabled);
        assert!(c.rust_optimizations.zero_cost_abstractions);
        assert_eq!(c.memory.pool_size_mb, 512);
        assert_eq!(c.threading.worker_threads, 0);
        assert!(matches!(c.network.optimization_level, NetworkOptimizationLevel::Aggressive));
    }

    #[test]
    fn default_canonical_memory_config() {
        let c = CanonicalMemoryConfig::default();
        assert_eq!(c.gc_threshold_mb, 256);
        assert_eq!(c.monitoring_interval, Duration::from_secs(60));
    }

    #[test]
    fn default_canonical_threading_config() {
        let c = CanonicalThreadingConfig::default();
        assert!(c.work_stealing);
        assert_eq!(c.stack_size_kb, 2048);
    }

    #[test]
    fn default_cpu_optimization_flags_and_config() {
        let f = CpuOptimizationFlags::default();
        assert!(f.simd);
        let c = CpuOptimizationConfig::default();
        assert_eq!(c.cache_optimization_level, 2);
    }

    #[test]
    fn default_io_flags_and_performance() {
        let f = IoOptimizationFlags::default();
        assert!(f.async_io);
        let c = IoPerformanceConfig::default();
        assert_eq!(c.queue_depth, 32);
    }

    #[test]
    fn default_network_performance_and_buffer() {
        let n = NetworkPerformanceConfig::default();
        assert_eq!(n.max_connections, 1000);
        let b = BufferConfig::default();
        assert!(b.enable_pooling);
    }

    #[test]
    fn default_network_optimization_level() {
        assert!(matches!(
            NetworkOptimizationLevel::default(),
            NetworkOptimizationLevel::Aggressive
        ));
    }

    #[test]
    fn default_caching_config() {
        let c = CachingConfig::default();
        assert_eq!(c.layers.len(), 2);
        assert!(matches!(c.eviction_policy, CacheEvictionPolicy::Lru));
    }

    #[test]
    fn default_performance_monitoring_and_features() {
        let m = PerformanceMonitoringConfig::default();
        assert_eq!(m.collection_interval_secs, 60);
        let f = MonitoringFeatures::default();
        assert!(f.system_monitoring.cpu_monitoring);
    }

    #[test]
    fn default_system_and_application_monitoring() {
        let s = SystemMonitoringConfig::default();
        assert!(s.disk_monitoring);
        let a = ApplicationMonitoringConfig::default();
        assert!(a.error_monitoring);
    }

    #[test]
    fn default_scalability_and_load_balancing() {
        let s = ScalabilityConfig::default();
        assert_eq!(s.min_instances, 1);
        let l = LoadBalancingConfig::default();
        assert!(matches!(l.algorithm, LoadBalancingAlgorithm::RoundRobin));
    }

    #[test]
    fn default_health_check_config() {
        let h = HealthCheckConfig::default();
        assert_eq!(h.failure_threshold, 3);
    }

    #[test]
    fn serde_roundtrip_canonical_performance_config() {
        roundtrip(&CanonicalPerformanceConfig::default());
    }

    #[test]
    fn serde_roundtrip_memory_threading_cpu_io_network() {
        roundtrip(&CanonicalMemoryConfig::default());
        roundtrip(&CanonicalThreadingConfig::default());
        roundtrip(&CpuOptimizationConfig::default());
        roundtrip(&IoPerformanceConfig::default());
        roundtrip(&NetworkPerformanceConfig::default());
    }

    #[test]
    fn serde_roundtrip_network_optimization_level() {
        roundtrip(&NetworkOptimizationLevel::Basic);
    }

    #[test]
    fn serde_roundtrip_buffer_and_caching() {
        roundtrip(&BufferConfig::default());
        roundtrip(&CachingConfig::default());
        roundtrip(&CacheEvictionPolicy::Lfu);
    }

    #[test]
    fn serde_roundtrip_cache_layer() {
        let layer = CacheLayerConfig {
            name: "L0".to_string(),
            size_mb: 32,
            ttl: Duration::from_secs(60),
        };
        roundtrip(&layer);
    }

    #[test]
    fn serde_roundtrip_monitoring_stack() {
        roundtrip(&PerformanceMonitoringConfig::default());
        roundtrip(&MonitoringFeatures::default());
        roundtrip(&SystemMonitoringConfig::default());
        roundtrip(&ApplicationMonitoringConfig::default());
    }

    #[test]
    fn serde_roundtrip_metric_config_and_aggregation() {
        let m = MetricConfig {
            enabled: true,
            sample_rate: 0.5,
            aggregation: MetricAggregation::Max,
        };
        roundtrip(&m);
        roundtrip(&MetricAggregation::Sum);
    }

    #[test]
    fn serde_roundtrip_scalability_load_balancing_health() {
        roundtrip(&ScalabilityConfig::default());
        roundtrip(&LoadBalancingConfig::default());
        roundtrip(&LoadBalancingAlgorithm::LeastConnections);
        roundtrip(&HealthCheckConfig::default());
    }

    #[test]
    fn serde_roundtrip_rust_optimization_features() {
        roundtrip(&RustOptimizationFeatures::default());
    }
}
