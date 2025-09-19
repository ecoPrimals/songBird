//! # 🎼 Delegated Benchmark Orchestration
//!
//! **🚀 PURE DELEGATION ARCHITECTURE**
//!
//! This module orchestrates production benchmarks by delegating all compute-intensive
//! operations to ToadStool via the Universal Adapter routing system.
//!
//! ## 🎼 Songbird's Role in Benchmarking
//! - ✅ **Orchestrates** benchmark execution across compute providers
//! - ✅ **Aggregates** benchmark results from multiple providers
//! - ✅ **Coordinates** multi-phase benchmark workflows
//! - ✅ **Handles** provider failover and result validation
//! - ❌ **Does NOT implement** actual benchmark algorithms or system inspection
//!
//! ## ⚙️ Benchmark Delegation Targets
//! - **Memory Benchmarks** → ToadStool via `routing::compute_request()`
//! - **Cache Benchmarks** → ToadStool via `routing::compute_request()`
//! - **Load Balancer Benchmarks** → ToadStool via `routing::compute_request()`
//! - **Object Pool Benchmarks** → ToadStool via `routing::compute_request()`

use serde::{Deserialize, Serialize};
use serde_json::json;
use songbird_errors::{SongbirdError, SongbirdResult, success};
// // use songbird_universal_primals  // TEMPORARILY DISABLED  // TEMPORARILY DISABLED::global_adapter::{AdapterContext, routing};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use songbird_errors::SongbirdResult;

/// Production benchmark orchestrator - delegates ALL operations to compute providers
///
/// **CRITICAL**: This is NOT a benchmark runner - it's a routing orchestrator
/// that delegates to actual ComputeCapability providers (like ToadStool).
#[derive(Debug, Clone)]
pub struct ProductionBenchmarkOrchestrator {
    ctx: AdapterContext,
}

impl ProductionBenchmarkOrchestrator {
    /// Create new benchmark orchestrator
    pub fn new() -> Self {
        Self {
            ctx: AdapterContext::new("production_benchmarks"),
        }
    }

    /// Run complete benchmark suite by delegating to compute providers
    pub async fn run_benchmark_suite(&self) -> SongbirdResult<DelegatedBenchmarkSuite> {
        info!("🎼 Starting production benchmark suite orchestration");

        let start_time = Instant::now();

        // Execute all benchmarks in parallel via delegation
        let (memory_result, cache_result, load_balancer_result, object_pool_result) = tokio::join!(
            self.run_memory_benchmarks(),
            self.run_cache_benchmarks(),
            self.run_load_balancer_benchmarks(),
            self.run_object_pool_benchmarks()
        );

        let execution_time = start_time.elapsed();

        let suite = DelegatedBenchmarkSuite {
            memory: memory_result.map(|r| r.data).unwrap_or_else(|e| {
                warn!("Memory benchmarks failed: {}", e);
                MemoryBenchmarkResults::failed(e.to_string())
            }),
            cache: cache_result.map(|r| r.data).unwrap_or_else(|e| {
                warn!("Cache benchmarks failed: {}", e);
                CacheBenchmarkResults::failed(e.to_string())
            }),
            load_balancer: load_balancer_result.map(|r| r.data).unwrap_or_else(|e| {
                warn!("Load balancer benchmarks failed: {}", e);
                LoadBalancerBenchmarkResults::failed(e.to_string())
            }),
            object_pool: object_pool_result.map(|r| r.data).unwrap_or_else(|e| {
                warn!("Object pool benchmarks failed: {}", e);
                ObjectPoolBenchmarkResults::failed(e.to_string())
            }),
            total_execution_time: execution_time,
            orchestration_metadata: self.get_orchestration_metadata(),
        };

        info!(
            "✅ Production benchmark suite completed in {:?}",
            execution_time
        );
        Ok(songbird_errors::evolved_success(success(suite)))
    }

    /// Run memory benchmarks by delegating to compute provider
    ///
    /// **DELEGATION**: Routes to any provider with ComputeCapability::Benchmarking
    pub async fn run_memory_benchmarks(&self) -> SongbirdResult<MemoryBenchmarkResults> {
        debug!("🎼 Benchmark orchestration: Routing memory benchmarks to compute provider");

        let request = json!({
            "operation": "run_memory_benchmarks",
            "benchmark_type": "production",
            "context": "songbird_performance_testing",
            "parameters": {
                "allocation_sizes": [1024, 4096, 16384, 65536],
                "iteration_count": 100000,
                "concurrency_levels": [1, 4, 8, 16]
            }
        });

        match routing::compute_request("run_benchmarks", serde_json::to_value(request)?) {
            Ok(songbird_errors::evolved_success(response)) => {
                debug!("✅ Memory benchmarks delegated successfully");
                Ok(songbird_errors::evolved_success(serde_json::from_value(response.data))?)
            }
            Err(e) => {
                warn!("⚠️ Compute provider memory benchmarks failed: {}", e);
                Err(SongbirdError::internal_error(Service {
                    service: "ToadStool".to_string(),
                    message: format!("Memory benchmark delegation failed: {}", e),
                    suggested_alternatives: vec!["Check ToadStool availability".to_string()],
                    recovery_actions: vec!["Retry with different compute provider".to_string()],
                })
            }
        }
    }

    /// Run cache benchmarks by delegating to compute provider
    pub async fn run_cache_benchmarks(&self) -> SongbirdResult<CacheBenchmarkResults> {
        debug!("🎼 Benchmark orchestration: Routing cache benchmarks to compute provider");

        let request = json!({
            "operation": "run_cache_benchmarks",
            "benchmark_type": "production",
            "context": "songbird_performance_testing",
            "parameters": {
                "cache_sizes": [1000, 10000, 100000],
                "operation_types": ["get", "set", "delete", "evict"],
                "concurrency_levels": [1, 4, 8]
            }
        });

        match routing::compute_request("run_benchmarks", serde_json::to_value(request)?) {
            Ok(songbird_errors::evolved_success(response)) => {
                debug!("✅ Cache benchmarks delegated successfully");
                Ok(songbird_errors::evolved_success(serde_json::from_value(response.data))?)
            }
            Err(e) => {
                warn!("⚠️ Compute provider cache benchmarks failed: {}", e);
                Ok(songbird_errors::evolved_success(songbird_errors::responses::SongbirdResponse::success(
                    CacheBenchmarkResults::failed(e.to_string())),
                ))
            }
        }
    }

    /// Run load balancer benchmarks by delegating to compute provider
    pub async fn run_load_balancer_benchmarks(&self) -> SongbirdResult<LoadBalancerBenchmarkResults> {
        debug!("🎼 Benchmark orchestration: Routing load balancer benchmarks to compute provider");

        let request = json!({
            "operation": "run_load_balancer_benchmarks",
            "benchmark_type": "production",
            "context": "songbird_performance_testing",
            "parameters": {
                "node_counts": [2, 5, 10, 20],
                "request_rates": [100, 500, 1000, 2000],
                "algorithms": ["round_robin", "least_connections", "weighted"]
            }
        });

        match routing::compute_request("run_benchmarks", serde_json::to_value(request)?) {
            Ok(songbird_errors::evolved_success(response)) => {
                debug!("✅ Load balancer benchmarks delegated successfully");
                Ok(songbird_errors::evolved_success(serde_json::from_value(response.data))?)
            }
            Err(e) => {
                warn!("⚠️ Compute provider load balancer benchmarks failed: {}", e);
                Ok(songbird_errors::evolved_success(songbird_errors::responses::SongbirdResponse::success(
                    LoadBalancerBenchmarkResults::failed(e.to_string())),
                ))
            }
        }
    }

    /// Run object pool benchmarks by delegating to compute provider
    pub async fn run_object_pool_benchmarks(&self) -> SongbirdResult<ObjectPoolBenchmarkResults> {
        debug!("🎼 Benchmark orchestration: Routing object pool benchmarks to compute provider");

        let request = json!({
            "operation": "run_object_pool_benchmarks",
            "benchmark_type": "production",
            "context": "songbird_performance_testing",
            "parameters": {
                "pool_sizes": [10, 50, 100, 500],
                "object_types": ["connection", "buffer", "thread"],
                "concurrency_levels": [1, 4, 8, 16]
            }
        });

        match routing::compute_request("run_benchmarks", serde_json::to_value(request)?) {
            Ok(songbird_errors::evolved_success(response)) => {
                debug!("✅ Object pool benchmarks delegated successfully");
                Ok(songbird_errors::evolved_success(serde_json::from_value(response.data))?)
            }
            Err(e) => {
                warn!("⚠️ Compute provider object pool benchmarks failed: {}", e);
                Ok(songbird_errors::evolved_success(songbird_errors::responses::SongbirdResponse::success(
                    ObjectPoolBenchmarkResults::failed(e.to_string())),
                ))
            }
        }
    }

    /// Get orchestration metadata for result tracking
    fn get_orchestration_metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("orchestrator".to_string(), "songbird".to_string());
        metadata.insert("delegation_target".to_string(), "toadstool".to_string());
        metadata.insert("architecture".to_string(), "universal_adapter".to_string());
        metadata.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
        metadata
    }
}

impl Default for ProductionBenchmarkOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// BENCHMARK RESULT TYPES
// ============================================================================

/// Complete benchmark suite results aggregated from compute providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegatedBenchmarkSuite {
    pub memory: MemoryBenchmarkResults,
    pub cache: CacheBenchmarkResults,
    pub load_balancer: LoadBalancerBenchmarkResults,
    pub object_pool: ObjectPoolBenchmarkResults,
    pub total_execution_time: Duration,
    pub orchestration_metadata: HashMap<String, String>,
}

/// Memory benchmark results from compute provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBenchmarkResults {
    pub allocation_throughput: f64, // ops/sec
    pub fragmentation_score: f64,   // 0.0-1.0
    pub gc_pressure: f64,           // MB/sec
    pub peak_memory_usage: u64,     // bytes
    pub success: bool,
    pub error_message: Option<String>,
}

impl MemoryBenchmarkResults {
    pub fn failed(error: String) -> Self {
        Self {
            allocation_throughput: 0.0,
            fragmentation_score: 1.0,
            gc_pressure: 0.0,
            peak_memory_usage: 0,
            success: false,
            error_message: Some(error),
        }
    }
}

/// Cache benchmark results from compute provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheBenchmarkResults {
    pub hit_rate: f64,            // 0.0-1.0
    pub throughput: f64,          // ops/sec
    pub latency_p95: Duration,    // 95th percentile latency
    pub eviction_efficiency: f64, // 0.0-1.0
    pub success: bool,
    pub error_message: Option<String>,
}

impl CacheBenchmarkResults {
    pub fn failed(error: String) -> Self {
        Self {
            hit_rate: 0.0,
            throughput: 0.0,
            latency_p95: Duration::from_millis(0),
            eviction_efficiency: 0.0,
            success: false,
            error_message: Some(error),
        }
    }
}

/// Load balancer benchmark results from compute provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerBenchmarkResults {
    pub requests_per_second: f64,
    pub latency_p95: Duration,
    pub fairness_score: f64, // 0.0-1.0
    pub failover_time: Duration,
    pub success: bool,
    pub error_message: Option<String>,
}

impl LoadBalancerBenchmarkResults {
    pub fn failed(error: String) -> Self {
        Self {
            requests_per_second: 0.0,
            latency_p95: Duration::from_millis(0),
            fairness_score: 0.0,
            failover_time: Duration::from_secs(0),
            success: false,
            error_message: Some(error),
        }
    }
}

/// Object pool benchmark results from compute provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectPoolBenchmarkResults {
    pub acquisition_latency: Duration,
    pub pool_efficiency: f64,  // 0.0-1.0
    pub contention_score: f64, // 0.0-1.0 (lower is better)
    pub leak_detection: bool,
    pub success: bool,
    pub error_message: Option<String>,
}

impl ObjectPoolBenchmarkResults {
    pub fn failed(error: String) -> Self {
        Self {
            acquisition_latency: Duration::from_millis(0),
            pool_efficiency: 0.0,
            contention_score: 1.0,
            leak_detection: false,
            success: false,
            error_message: Some(error),
        }
    }
}
