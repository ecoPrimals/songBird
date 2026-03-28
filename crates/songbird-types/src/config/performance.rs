// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Performance /// Configuration capability Configuration
//!
//! **CANONICAL**: Consolidated performance configuration - Single Source of Truth Truth
//!
//! This module consolidates all performance configurations from across the codebase:
//! - `songbird-config` `PerformanceConfig`
//! - `songbird-orchestrator` (runtime/orchestration performance settings)
//! - `songbird-network-federation` and other networking crates
//! - And other scattered performance configs

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
use std::time::Duration;

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
/// **CANONICAL**: Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalMemoryConfig {
    /// Enable memory optimization
    /// Optimization Enabled field
    pub optimization_enabled: bool,
    /// Memory pool size in
    pub pool_size_mb: usize,
    /// Enable memory compaction
    /// Compaction Enabled field
    pub compaction_enabled: bool,
    /// Garbage collection threshold
    pub gc_threshold_mb: usize,
    /// Memory monitoring interval
    /// Monitoring Interval field
    pub monitoring_interval: Duration,
}

impl Default for CanonicalMemoryConfig {
    fn default() -> Self {
        Self {
            optimization_enabled: true,
            pool_size_mb: 512,
            compaction_enabled: true,
            gc_threshold_mb: 256,
            monitoring_interval: Duration::from_secs(60),
        }
    }
}
/// **CANONICAL**: Threading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalThreadingConfig {
    /// Enable thread pool optimization
    /// Optimization Enabled field
    pub optimization_enabled: bool,
    /// Number of worker threads (0 = auto-detect)
    /// Worker Threads field
    pub worker_threads: usize,
    /// Enable work-stealing scheduler
    /// Work Stealing field
    pub work_stealing: bool,
    /// Thread stack size in
    pub stack_size_kb: usize,
    /// Thread affinity enabled
    /// Affinity Enabled field
    pub affinity_enabled: bool,
}

impl Default for CanonicalThreadingConfig {
    fn default() -> Self {
        Self {
            optimization_enabled: true,
            worker_threads: 0, // Auto-detect based on CPU cores
            work_stealing: true,
            stack_size_kb: 2048, // 2MB stack
            affinity_enabled: false,
        }
    }
}
/// CPU optimization flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuOptimizationFlags {
    /// Enable SIMD optimizations
    pub simd: bool,
    /// Enable branch prediction optimizations
    pub branch_prediction: bool,
    /// Enable CPU profiling
    pub profiling: bool,
}
/// CPU optimization configuration - consolidated from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuOptimizationConfig {
    /// Enable CPU optimizations
    pub enabled: bool,
    /// Target CPU architecture optimizations
    pub target_cpu: Option<String>,
    /// CPU optimization flags
    pub flags: CpuOptimizationFlags,
    /// CPU cache optimization level (1-3)
    pub cache_optimization_level: u8,
}

impl Default for CpuOptimizationFlags {
    fn default() -> Self {
        Self {
            simd: true,
            branch_prediction: true,
            profiling: false,
        }
    }
}

impl Default for CpuOptimizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            target_cpu: None, // Auto-detect
            flags: CpuOptimizationFlags::default(),
            cache_optimization_level: 2,
        }
    }
}
/// I/O optimization flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoOptimizationFlags {
    /// Enable asynchronous I/O
    pub async_io: bool,
    /// Enable direct I/O (bypass OS cache)
    pub direct_io: bool,
    /// Enable I/O batching
    pub batching: bool,
}
/// I/O performance configuration - consolidated from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoPerformanceConfig {
    /// Enable I/O optimizations
    pub enabled: bool,
    /// I/O buffer size in KB
    pub buffer_size_kb: usize,
    /// I/O optimization flags
    pub flags: IoOptimizationFlags,
    /// I/O queue depth
    pub queue_depth: u32,
    /// Batch size for I/O operations
    pub batch_size: usize,
}

impl Default for IoOptimizationFlags {
    fn default() -> Self {
        Self {
            async_io: true,
            direct_io: false,
            batching: true,
        }
    }
}

impl Default for IoPerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_size_kb: 64,
            flags: IoOptimizationFlags::default(),
            queue_depth: 32,
            batch_size: 16,
        }
    }
}
/// Network performance configuration with optimization levels
///
/// This struct provides comprehensive network performance tuning options
/// including optimization levels, buffer management, and connection pooling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPerformanceConfig {
    /// Network optimization level determining performance characteristics
    pub optimization_level: NetworkOptimizationLevel,
    /// Buffer configuration for network operations
    pub buffer_config: BufferConfig,
    /// Maximum number of concurrent connections
    pub max_connections: u32,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Enable connection keepalive
    pub keepalive_enabled: bool,
    /// Keepalive interval in seconds
    pub keepalive_interval_secs: u64,
}
/// Network optimization levels for different performance characteristics
///
/// Each level provides different trade-offs between performance, memory usage,
/// and CPU utilization to match various deployment scenarios.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum NetworkOptimizationLevel {
    /// Disabled optimization for minimal resource usage
    ///
    /// Use this level when resource conservation is more important than performance.
    /// Suitable for resource-constrained environments or testing scenarios.
    Disabled,

    /// Basic optimization with moderate performance improvements
    ///
    /// Provides a balanced approach with reasonable performance gains
    /// while maintaining low resource overhead. Good for most deployments.
    Basic,

    /// Aggressive optimization for maximum performance
    ///
    /// Enables all performance optimizations including advanced buffer pooling,
    /// connection multiplexing, and zero-copy operations. Use in high-throughput
    /// production environments where performance is critical.
    #[default]
    Aggressive,
}
/// Buffer configuration for network and I/O operations
///
/// This struct provides comprehensive buffer management settings including
/// size limits, pooling options, and memory optimization strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferConfig {
    /// Initial buffer size in bytes
    pub initial_size: usize,
    /// Maximum buffer size in bytes
    pub max_size: usize,
    /// Number of buffers to pre-allocate in the pool
    pub pool_size: usize,
    /// Enable buffer pooling for reuse
    pub enable_pooling: bool,
    /// Enable zero-copy optimizations where possible
    pub enable_zero_copy: bool,
}

impl Default for NetworkPerformanceConfig {
    fn default() -> Self {
        Self {
            optimization_level: NetworkOptimizationLevel::Aggressive,
            buffer_config: BufferConfig::default(),
            max_connections: 1000,
            connection_timeout_ms: 30000,
            keepalive_enabled: true,
            keepalive_interval_secs: 60,
        }
    }
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            initial_size: 1024,    // 1KB
            max_size: 1024 * 1024, // 1MB
            pool_size: 10,
            enable_pooling: true,
            enable_zero_copy: true,
        }
    }
}
/// Caching configuration - consolidated from network-layer crates (e.g. federation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachingConfig {
    /// Enable caching
    pub enabled: bool,
    /// Cache TTL
    pub ttl: Duration,
    /// Enable cache compression
    pub compression_enabled: bool,
    /// Cache eviction policy
    pub eviction_policy: CacheEvictionPolicy,
    /// Enable cache statistics
    pub statistics_enabled: bool,
    /// Cache layers configuration
    pub layers: Vec<CacheLayerConfig>,
    /// Cache size in MB
    pub cache_size_mb: usize,
}

impl Default for CachingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_size_mb: 256,
            ttl: Duration::from_secs(3600), // 1 hour
            compression_enabled: false,
            eviction_policy: CacheEvictionPolicy::Lru,
            statistics_enabled: true,
            layers: vec![
                CacheLayerConfig {
                    name: "L1".to_string(),
                    size_mb: 64,
                    ttl: Duration::from_secs(300), // 5 minutes
                },
                CacheLayerConfig {
                    name: "L2".to_string(),
                    size_mb: 192,
                    ttl: Duration::from_secs(3600), // 1 hour
                },
            ],
        }
    }
}
/// Cache eviction policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheEvictionPolicy {
    /// Least Recently Used
    Lru,
    /// Least Frequently Used
    Lfu,
    /// First In, First Out
    Fifo,
    /// Random replacement
    Random,
}
/// Cache layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheLayerConfig {
    /// Layer name
    pub name: String,
    /// Layer size in MB
    pub size_mb: usize,
    /// Time to live
    pub ttl: Duration,
}
/// Performance monitoring configuration for system observability
///
/// This struct defines comprehensive monitoring settings including metrics
/// collection intervals, feature toggles, and performance thresholds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMonitoringConfig {
    /// Monitoring features to enable
    pub features: MonitoringFeatures,
    /// Metrics collection interval in seconds
    pub collection_interval_secs: u64,
    /// Enable performance alerting
    pub enable_alerting: bool,
    /// Performance threshold for alerts
    pub alert_threshold_ms: u64,
}

impl Default for PerformanceMonitoringConfig {
    fn default() -> Self {
        Self {
            features: MonitoringFeatures::default(),
            collection_interval_secs: 60,
            enable_alerting: true,
            alert_threshold_ms: 100,
        }
    }
}
/// Monitoring features configuration
///
/// This struct enables fine-grained control over which monitoring
/// features are active to optimize resource usage and data collection.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitoringFeatures {
    /// System resource monitoring configuration
    pub system_monitoring: SystemMonitoringConfig,
    /// Application monitoring configuration
    pub application_monitoring: ApplicationMonitoringConfig,
}
/// System resource monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMonitoringConfig {
    /// Enable CPU usage monitoring
    pub cpu_monitoring: bool,
    /// Enable memory usage monitoring
    pub memory_monitoring: bool,
    /// Enable disk I/O monitoring
    pub disk_monitoring: bool,
}
/// Application monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationMonitoringConfig {
    /// Enable network I/O monitoring
    pub network_monitoring: bool,
    /// Enable request latency monitoring
    pub latency_monitoring: bool,
    /// Enable error rate monitoring
    pub error_monitoring: bool,
}

impl Default for SystemMonitoringConfig {
    fn default() -> Self {
        Self {
            cpu_monitoring: true,
            memory_monitoring: true,
            disk_monitoring: true,
        }
    }
}

impl Default for ApplicationMonitoringConfig {
    fn default() -> Self {
        Self {
            network_monitoring: true,
            latency_monitoring: true,
            error_monitoring: true,
        }
    }
}
/// Metric configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricConfig {
    /// Enable metric
    /// Enabled field
    pub enabled: bool,
    /// Sample rate (0.0 to 1.0)
    /// Sample Rate field
    pub sample_rate: f64,
    /// Aggregation method
    /// Aggregation field
    pub aggregation: MetricAggregation,
}
/// Metric aggregation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricAggregation {
    /// Average value
    Average,
    /// Sum of values
    Sum,
    /// Minimum value
    Min,
    /// Maximum value
    Max,
    /// Count of values
    Count,
}
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

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
