//! Performance Profiling Tests
//!
//! Tests to identify performance bottlenecks and optimize critical paths

use songbird_lib::errors::Result;
use songbird_lib::production_benchmarks::*;
use std::time::Duration;
use tokio;

#[tokio::test]
async fn test_run_full_benchmark_suite() -> Result<()> {
    println!("🚀 Starting Full Benchmark Suite...");

    let config = BenchmarkConfig {
        service_instance_count: 100,
        requests_per_test: 1000,
        concurrent_workers: 10,
        cache_test_data_size: 1024,
        object_pool_iterations: 10000,
        batch_test_size: 100,
        warmup_duration: Duration::from_secs(5),
        test_duration: Duration::from_secs(30),
    };

    let mut benchmark_runner = BenchmarkRunner::new(config);
    let results = benchmark_runner.run_full_benchmark_suite().await?;

    println!("📊 Benchmark Results:");
    println!(
        "  Load Balancer Performance: {:.2}ms avg",
        results.load_balancer_avg_latency_ms
    );
    println!(
        "  Cache Performance: {:.2}ms avg",
        results.cache_avg_latency_ms
    );
    println!(
        "  Memory Optimizer: {:.2}ms avg",
        results.memory_optimizer_avg_latency_ms
    );
    println!(
        "  Batch Processing: {:.2}ms avg",
        results.batch_processing_avg_latency_ms
    );

    // Export results to JSON
    let json_results = benchmark_runner.export_results_json().await?;
    println!("📄 JSON Results exported successfully");

    Ok(())
}

#[tokio::test]
async fn test_quick_production_check() -> Result<()> {
    println!("⚡ Running Quick Production Check...");

    let is_production_ready = quick_production_check().await?;

    if is_production_ready {
        println!("✅ System is production ready");
    } else {
        println!("⚠️  System needs optimization for production");
    }

    Ok(())
}

#[tokio::test]
async fn test_performance_bottleneck_identification() -> Result<()> {
    println!("🔍 Identifying Performance Bottlenecks...");

    let config = BenchmarkConfig {
        service_instance_count: 50,
        requests_per_test: 500,
        concurrent_workers: 5,
        cache_test_data_size: 512,
        object_pool_iterations: 5000,
        batch_test_size: 50,
        warmup_duration: Duration::from_secs(2),
        test_duration: Duration::from_secs(10),
    };

    let mut benchmark_runner = BenchmarkRunner::new(config);
    let results = benchmark_runner.run_full_benchmark_suite().await?;

    // Identify bottlenecks
    let mut bottlenecks = Vec::new();

    if results.load_balancer_avg_latency_ms > 100.0 {
        bottlenecks.push("Load Balancer: High latency detected");
    }

    if results.cache_avg_latency_ms > 50.0 {
        bottlenecks.push("Cache: High latency detected");
    }

    if results.memory_optimizer_avg_latency_ms > 200.0 {
        bottlenecks.push("Memory Optimizer: High latency detected");
    }

    if results.batch_processing_avg_latency_ms > 150.0 {
        bottlenecks.push("Batch Processing: High latency detected");
    }

    if bottlenecks.is_empty() {
        println!("✅ No performance bottlenecks identified");
    } else {
        println!("⚠️  Performance bottlenecks identified:");
        for bottleneck in bottlenecks {
            println!("  - {}", bottleneck);
        }
    }

    Ok(())
}
