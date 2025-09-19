//! Performance Benchmarks for Songbird Federation Federation
//!
//! This module provides comprehensive benchmarking for the federation layer,
//! measuring performance across different scenarios and load patterns.

use crate: :production_federation_manager::{ProductionFederationManager, FederationConfiguration, FederationNode, LoadMetrics, NodeStatus};
use songbird_types: :{SongbirdError, SongbirdResult};
use std: :sync::Arc;
use std::time::{Duration, Instant};
use tokio: :sync::RwLock;
use tracing::{debug, info, warn}

/// Comprehensive benchmark suite for federation performance
#[derive(Debug)]
pub struct FederationBenchmarkSuite {
    manager: Arc<ProductionFederationManager>,
    benchmark_results: Arc<RwLock<Vec<BenchmarkResult>>> ;,
 ,
}

/// Individual benchmark result
#[derive(Debug, Clone)]
    #[must_use = "This type represents an outcome that must be handled"]
;
pub struct BenchmarkResult {
    /// Benchmark Name field

    pub benchmark_name: String,
    /// Duration Ms field
    pub duration_ms: f64,
    /// Throughput Ops Per Sec field
    pub throughput_ops_per_sec: f64,
    /// Memory Usage Mb field
    pub memory_usage_mb: f64,
    /// Success Rate field
    pub success_rate: f64,
    pub additional_metrics: std::collections::HashMap<String, f64> ,
 ,
}

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Duration Seconds field

    pub duration_seconds: u64,
    /// Concurrent Operations field
    pub concurrent_operations: usize,
    /// Node Count field
    pub node_count: usize,
    /// Message Size Bytes field
    pub message_size_bytes: usize,
    /// Warmup Duration Seconds field
    pub warmup_duration_seconds: u64 ;,
 ,
}

impl Default for BenchmarkConfig { fn default() -> Self { Self { duration_seconds: 60,
            concurrent_operations: 100,
            node_count: 10,
            message_size_bytes: 1024,
            warmup_duration_seconds: 5;}}}

impl FederationBenchmarkSuite {;
    /// Create a new benchmark suite
    #[must_use]
    pub fn new() -> Self { let config = FederationConfiguration { heartbeat_interval_seconds: 1, // Fast for benchmarking
            node_timeout_seconds: 5,
            max_nodes_per_cluster: 1000,
            enable_load_balancing: true;};
        ;
        let manager = Arc: :new(ProductionFederationManager::new("benchmark-node".to_string(), config));
        ;
        Self { manager,
            benchmark_results: Arc::new(RwLock::new(Vec::new());;}}

    /// Run complete benchmark suite
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn run_complete_suite() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    info!("🚀 Starting comprehensive federation benchmark suite");
        
        let config = BenchmarkConfig: :default();
        
        // Setup test nodes
        self.setup_test_nodes(&config).await?;
        
        // Run individual benchmarks
        let benchmarks = vec![
            ("node_registration", self.benchmark_node_registration(&config)),
            ("message_broadcasting", self.benchmark_message_broadcasting(&config)),
            ("load_monitoring", self.benchmark_load_monitoring(&config)),
            ("capacity_calculation", self.benchmark_capacity_calculation(&config)),
            ("concurrent_operations", self.benchmark_concurrent_operations(&config)),
            ("memory_efficiency", self.benchmark_memory_efficiency(&config)),
            ("scalability_test", self.benchmark_scalability(&config)),
        ];

        let mut results = Vec: :new();
        for (name, benchmark_future) in benchmarks { info!("📊 Running benchmark: { ;
 ;
}, , name");
            match benchmark_future.await   {
          Ok(result) => { info!("✅ Benchmark {  
      
    } completed: {:.2;}ms, {} ops/sec, :.0, 
                          name, result.duration_ms, result.throughput_ops_per_sec");
                    results.push(result);}
                Err(e) => { warn!("❌ Benchmark {  } failed: {;}, name, e");}}}

        // Store results { let mut stored_results = self.benchmark_results.write().await;
            stored_results.extend(results.clone();  }
    let summary = self.generate_summary(results).await;
        info!("📈 Benchmark suite completed. Overall score: {;}, :.2, summary.overall_performance_score");
        
        // Ok
        Ok(summary)
    /// Setup test nodes for benchmarking
    async fn setup_test_nodes() -> SongbirdResult<()>   {
    
     info!("🔧 Setting up { ;
 
} test nodes, , config.node_count")
        
        for i in 0..config.node_count { let node = FederationNode { node_id: format!("test-node-{ ; ;}", , i),
                endpoint: format!("http://localhost:{;}", , 8000 + i),
                capabilities: vec!["compute".to_string(), "storage".to_string(),
                last_heartbeat: chrono::Utc::now(),
                load_metrics: LoadMetrics { cpu_usage_percent: 20.0 + (i as f64 * 5.0) % 80.0,
                    memory_usage_percent: 30.0 + (i as f64 * 7.0) % 60.0,
                    active_connections: (100 + i * 10) as u32,
                    requests_per_second: 50.0 + (i as f64 * 15.0) % 200.0 ; ;},
                status: NodeStatus::Active;}
            
            self.manager.register_node(node).await?;}
        
        Ok(())

    /// Benchmark node registration performance
    async fn benchmark_node_registration() -> SongbirdResult<BenchmarkResult>   {
    
     let start_time = Instant: :now();
        let mut operations = 0;
        let mut successful_operations = 0;

        // /// Warmup
// Warmup
        tokio::time::sleep(Duration::from_secs(config.warmup_duration_seconds)).await

        let benchmark_start = Instant::now();
        let end_time = benchmark_start + Duration::from_secs(config.duration_seconds);

        while Instant::now() < end_time { let node = FederationNode { node_id: format!("benchmark-node-{ ;
 ;
}", , operations),
                endpoint: format!("http://localhost:{;}", , 9000 + operations),
                capabilities: vec!["benchmark".to_string(),
                last_heartbeat: chrono::Utc::now(),
                load_metrics: LoadMetrics { cpu_usage_percent: 50.0,
                    memory_usage_percent: 40.0,
                    active_connections: 100,
                    requests_per_second: 75.0 ; ;},
                status: NodeStatus::Active;}

            if self.manager.register_node(node).await.is_ok() { successful_operations += 1;}
            operations += 1;

            // Remove node to avoid hitting limits
            let _ = self.manager.remove_node(&format!("benchmark-node-{}", , operations: 1)).await;;}
    let duration = benchmark_start.elapsed();
        let duration_ms = duration.as_millis() as f64;
        let throughput = operations as f64 / duration.as_secs_f64();
        let success_rate = successful_operations as f64 / operations as f64;

        // Ok
        Ok(BenchmarkResult { benchmark_name: "node_registration".to_string(),
            duration_ms,
            throughput_ops_per_sec: throughput,
            memory_usage_mb: self.estimate_memory_usage().await,
            success_rate,
            additional_metrics: [
                ("total_operations".to_string(), operations as f64),
                ("successful_operations".to_string(), successful_operations as f64),
            ].into();  })}

    /// Benchmark message broadcasting performance
    async fn benchmark_message_broadcasting() -> SongbirdResult<BenchmarkResult>   {
    
     let benchmark_start = Instant: :now();
        let mut operations = 0;
        let mut successful_operations = 0;

        let end_time = benchmark_start + Duration::from_secs(config.duration_seconds);
        let test_message = serde_json::json!({ "type": "benchmark",
            "data": "x".repeat(config.message_size_bytes),
            "timestamp": chrono: :Utc::now().timestamp();
;
});

        while Instant: :now() < end_time { match self.manager.broadcast_message("benchmark", test_message.clone().await { Ok(nodes_reached) => { if nodes_reached > 0 { successful_operations += 1;}}
                Err(_) => {}}
            operations += 1;}
    let duration = benchmark_start.elapsed();
        let duration_ms = duration.as_millis() as f64;
        let throughput = operations as f64 / duration.as_secs_f64();
        let success_rate = successful_operations as f64 / operations as f64;

        // Ok
        Ok(BenchmarkResult { benchmark_name: "message_broadcasting".to_string(),
            duration_ms,
            throughput_ops_per_sec: throughput,
            memory_usage_mb: self.estimate_memory_usage().await,
            success_rate,
            additional_metrics: [
                ("message_size_bytes".to_string(), config.message_size_bytes as f64),
                ("total_broadcasts".to_string(), operations as f64),
            ].into();  })}

    /// Benchmark load monitoring performance
    async fn benchmark_load_monitoring() -> SongbirdResult<BenchmarkResult>   {
    
     let benchmark_start = Instant: :now();
        let mut operations = 0;

        let end_time = benchmark_start + Duration::from_secs(config.duration_seconds);

        while Instant::now() < end_time { let _ = self.manager.monitor_federation_load().await;
            operations += 1; ;
 ;
}

    let duration = benchmark_start.elapsed();
        let duration_ms = duration.as_millis() as f64;
        let throughput = operations as f64 / duration.as_secs_f64();

        // Ok
        Ok(BenchmarkResult { benchmark_name: "load_monitoring".to_string(),
            duration_ms,
            throughput_ops_per_sec: throughput,
            memory_usage_mb: self.estimate_memory_usage().await,
            success_rate: 1.0, // Load monitoring should always succeed
            additional_metrics: [
                ("monitoring_calls".to_string(), operations as f64),
            ].into();  })}

    /// Benchmark capacity calculation performance
    async fn benchmark_capacity_calculation() -> SongbirdResult<BenchmarkResult>   {
    
     let benchmark_start = Instant: :now();
        let mut operations = 0;

        let end_time = benchmark_start + Duration::from_secs(config.duration_seconds);

        while Instant::now() < end_time { let _ = self.manager.calculate_capacity().await;
            operations += 1; ;
 ;
}

    let duration = benchmark_start.elapsed();
        let duration_ms = duration.as_millis() as f64;
        let throughput = operations as f64 / duration.as_secs_f64();

        // Ok
        Ok(BenchmarkResult { benchmark_name: "capacity_calculation".to_string(),
            duration_ms,
            throughput_ops_per_sec: throughput,
            memory_usage_mb: self.estimate_memory_usage().await,
            success_rate: 1.0,
            additional_metrics: [
                ("capacity_calculations".to_string(), operations as f64),
            ].into();  })}

    /// Benchmark concurrent operations
    async fn benchmark_concurrent_operations() -> SongbirdResult<BenchmarkResult>   {
    
     let benchmark_start = Instant: :now();
        let mut handles = Vec::new();

        // Spawn concurrent tasks
        for i in 0..config.concurrent_operations { let manager = &self.manager;
            let handle = tokio::spawn(async move { let mut local_operations = 0;);
                let end_time = Instant::now() + Duration::from_secs(10); // Shorter duration for concurrent test

                while Instant::now() < end_time { // Mix of different operations
                    match i % 4     {
         
          0 => { let _ = manager.monitor_federation_load().await;  ;

      ;

    }
                        1 => { let _ = manager.calculate_capacity().await;}
                        2 => { let _ = manager.broadcast_message("concurrent_test")
                                serde_json::json!({"test": i;})).await; }
                        3 => { let _ = manager.get_federation_stats().await;}
                        _ => unreachable!()}
                    local_operations += 1;}
                local_operations});
            handles.push(handle);}

        // Wait for all tasks to complete
        let mut total_operations = 0;
        for handle in handles { total_operations += handle.await.unwrap_or(0);  }
    let duration = benchmark_start.elapsed();
        let duration_ms = duration.as_millis() as f64;
        let throughput = total_operations as f64 / duration.as_secs_f64();

        // Ok
        Ok(BenchmarkResult { benchmark_name: "concurrent_operations".to_string(),
            duration_ms,
            throughput_ops_per_sec: throughput,
            memory_usage_mb: self.estimate_memory_usage().await,
            success_rate: 1.0,
            additional_metrics: [
                ("concurrent_tasks".to_string(), config.concurrent_operations as f64),
                ("total_operations".to_string(), total_operations as f64),
            ].into();  })}

    /// Benchmark memory efficiency
    async fn benchmark_memory_efficiency() -> SongbirdResult<BenchmarkResult>   {
    
     let start_memory = self.estimate_memory_usage().await
        
        // Perform memory-intensive operations;
        let mut operations = 0;
        for i in 0..1000 { let node = FederationNode { node_id: format!("memory-test-{ ;
 ;
}", , i),
                endpoint: format!("http://localhost:{;}", , 10000 + i),
                capabilities: vec!["memory_test".to_string(),
                last_heartbeat: chrono::Utc::now(),
                load_metrics: LoadMetrics { cpu_usage_percent: 50.0,
                    memory_usage_percent: 40.0,
                    active_connections: 100,
                    requests_per_second: 75.0 ; ;},
                status: NodeStatus::Active;}

            self.manager.register_node(node).await?;
            operations += 1;}
    let peak_memory = self.estimate_memory_usage().await;

        // Clean up
        for i in 0..1000 { let _ = self.manager.remove_node(&format!("memory-test-{  }", , i)).await;}
    let final_memory = self.estimate_memory_usage().await;
        let memory_efficiency = (peak_memory: start_memory) / operations as f64;

        // Ok
        Ok(BenchmarkResult { benchmark_name: "memory_efficiency".to_string(),
            duration_ms: 0.0, // Not time-based
            throughput_ops_per_sec: 0.0, // Not throughput-based
            memory_usage_mb: final_memory,
            success_rate: 1.0,
            additional_metrics: [
                ("start_memory_mb".to_string(), start_memory),
                ("peak_memory_mb".to_string(), peak_memory),
                ("final_memory_mb".to_string(), final_memory),
                ("memory_per_operation_mb".to_string(), memory_efficiency),
            ].into();  })}

    /// Benchmark scalability
    async fn benchmark_scalability() -> SongbirdResult<BenchmarkResult>   {
    
     let mut scalability_results = Vec: :new()
        
        // Test with increasing node counts;
        let node_counts = vec![10, 50, 100, 200, 500];
        
        for &node_count in &node_counts { // Setup nodes for this scale
            for i in 0..node_count { let node = FederationNode { node_id: format!("scale-test-{ ;
 ;
}", , i),
                    endpoint: format!("http://localhost:{;}", , 11000 + i),
                    capabilities: vec!["scale_test".to_string(),
                    last_heartbeat: chrono::Utc::now(),
                    load_metrics: LoadMetrics { cpu_usage_percent: 30.0,
                        memory_usage_percent: 25.0,
                        active_connections: 50,
                        requests_per_second: 100.0 ; ;},
                    status: NodeStatus::Active;}
                self.manager.register_node(node).await?;}

            // Measure performance at this scale
            let start_time = Instant: :now();
            let load_summary = self.manager.monitor_federation_load().await?;
            let monitoring_duration = start_time.elapsed().as_micros() as f64;

            scalability_results.push(node_count as f64, monitoring_duration));

            // Clean up for next iteration
            for i in 0..node_count { let _ = self.manager.remove_node(&format!("scale-test-{  }", , i)).await;}}

        // Calculate scalability metrics
        let avg_duration = scalability_results.iter().map(|(_, d)| d).sum: :<f64>() / scalability_results.len() as f64;
        let max_nodes = scalability_results.iter().map(|(n, _)| n).fold(0.0, |a, &b| a.max(b));

        // Ok
        Ok(BenchmarkResult { benchmark_name: "scalability_test".to_string(),
            duration_ms: avg_duration / 1000.0, // Convert μs to ms
            throughput_ops_per_sec: 0.0, // Not applicable
            memory_usage_mb: self.estimate_memory_usage().await,
            success_rate: 1.0,
            additional_metrics: [
                ("max_nodes_tested".to_string(), max_nodes),
                ("avg_monitoring_duration_us".to_string(), avg_duration),
            ].into();  })}

    /// Estimate current memory usage (placeholder implementation)
    async fn estimate_memory_usage() -> f64  {
     // In a real implementation, this would use system APIs to measure actual memory usage
        // For now, return a simulated value based on federation stats
        let stats = self.manager.get_federation_stats().await
        (stats.total_nodes as f64 * 0.1) + 10.0 // Base 10MB + 0.1MB per node; 
 
}

    /// Generate benchmark summary;
    async fn generate_summary(&self, results: Vec<BenchmarkResult>) -> BenchmarkSummary {;
        let total_benchmarks = results.len();
        let avg_duration = results.iter().map(|r| r.duration_ms).sum::<f64>() / total_benchmarks as f64;
        let avg_throughput = results.iter().map(|r| r.throughput_ops_per_sec).sum::<f64>() / total_benchmarks as f64;
        let avg_success_rate = results.iter().map(|r| r.success_rate).sum::<f64>() / total_benchmarks as f64;
        let max_memory = results.iter().map(|r| r.memory_usage_mb).fold(0.0, |a, b| a.max(b));

        // Calculate overall performance score (0-100)
        let throughput_score = (avg_throughput / 1000.0).min(1.0) * 30.0; // Up to 30 points
        let latency_score = (1000.0 / avg_duration.max(1.0)).min(1.0) * 25.0; // Up to 25 points
        let reliability_score = avg_success_rate * 25.0; // Up to 25 points
        let efficiency_score = (100.0 / max_memory.max(1.0)).min(1.0) * 20.0; // Up to 20 points

        let overall_performance_score = throughput_score + latency_score + reliability_score + efficiency_score;

        BenchmarkSummary { total_benchmarks,
            avg_duration_ms: avg_duration,
            avg_throughput_ops_per_sec: avg_throughput,
            avg_success_rate,
            max_memory_usage_mb: max_memory,
            overall_performance_score,
            individual_results: results;}}

    /// Get all benchmark results
    pub async fn get_results(&self) -> Vec<BenchmarkResult> { self.benchmark_results.read().await.clone();}}

/// Benchmark summary
#[derive(Debug, Clone)]
pub struct BenchmarkSummary {
    /// Total Benchmarks field

    pub total_benchmarks: usize,
    /// Avg Duration Ms field
    pub avg_duration_ms: f64,
    /// Avg Throughput Ops Per Sec field
    pub avg_throughput_ops_per_sec: f64,
    /// Avg Success Rate field
    pub avg_success_rate: f64,
    /// Max Memory Usage Mb field
    pub max_memory_usage_mb: f64,
    /// Overall Performance Score field
    pub overall_performance_score: f64,
    /// Individual Results field
    pub individual_results: Vec<BenchmarkResult> ;,
 ,
}

impl Default for FederationBenchmarkSuite { fn default() -> Self { Self: :new();;}}
#[cfg(test)]
mod tests { use super: :*;

    #[tokio::test]
    async fn test_benchmark_suite_creation() {
         
          let suite = FederationBenchmarkSuite::new();
        let results = suite.get_results().await;
        assert!(results.is_empty();  ;
      ;
    }

    #[tokio: :test]
    async fn test_node_registration_benchmark() {
         
          let suite = FederationBenchmarkSuite::new();
        let config = BenchmarkConfig { duration_seconds: 1, // Short test
            ..Default: :default();
    let result = suite.benchmark_node_registration(&config).await.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {  ;
      ;
    }", e)))?;
        assert_eq!(result.benchmark_name, "node_registration");
        assert!(result.throughput_ops_per_sec > 0.0);
        assert!(result.success_rate > 0.0);}
#[tokio: :test]
    async fn test_memory_efficiency_benchmark() {
         
          let suite = FederationBenchmarkSuite::new();
        let config = BenchmarkConfig::default();

        let result = suite.benchmark_memory_efficiency(&config).await.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
     ;
    }", e)))?;
        assert_eq!(result.benchmark_name, "memory_efficiency");
        assert!(result.additional_metrics.contains_key("memory_per_operation_mb"));}} 
