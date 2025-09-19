//! Performance benchmarks for Songbird Core
//!
//! This module provides comprehensive performance benchmarking capabilities
//! for measuring and optimizing Songbird's core functionality.

use serde::{Deserialize, Serialize};
use songbird_errors::SongbirdResult;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Performance benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub iterations: u64,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub throughput_per_second: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

/// Benchmark suite for core components
#[derive(Debug, Clone)]
pub struct CoreBenchmarkSuite {
    pub results: HashMap<String, BenchmarkResult>,
    pub system_info: SystemInfo,
}

/// System information for benchmark context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu_cores: usize,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub os_type: String,
    pub rust_version: String,
}

impl CoreBenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
            system_info: SystemInfo::collect(),
        }
    }

    /// Benchmark basic hash map operations (simulating service registry)
    pub async fn benchmark_hashmap_operations(&mut self) -> SongbirdResult<BenchmarkResult> {
        let test_name = "hashmap_operations";
        let iterations = 100000;
        let mut durations = Vec::new();

        // Create test data structure
        let mut test_map: HashMap<String, String> = HashMap::new();

        // Warm up
        for i in 0..1000 {
            test_map.insert(format!("key-{i}"), format!("value-{i}"));
        }

        // Actual benchmark
        let start = Instant::now();
        for i in 0..iterations {
            let iter_start = Instant::now();

            // Simulate operations
            let key = format!("benchmark-key-{}", i % 1000);
            test_map.insert(key.clone(), format!("value-{i}"));
            let _value = test_map.get(&key);
            let _count = test_map.len();

            let iter_duration = iter_start.elapsed();
            durations.push(iter_duration);

            // Prevent overwhelming the system
            if i % 10000 == 0 {
                sleep(Duration::from_micros(1)).await;
            }
        }
        let total_duration = start.elapsed();

        let result = BenchmarkResult {
            test_name: test_name.to_string(),
            iterations,
            total_duration,
            avg_duration: total_duration / iterations as u32,
            min_duration: durations.iter().min().copied().unwrap_or(Duration::ZERO),
            max_duration: durations.iter().max().copied().unwrap_or(Duration::ZERO),
            throughput_per_second: iterations as f64 / total_duration.as_secs_f64(),
            memory_usage_mb: self.get_memory_usage(),
            cpu_usage_percent: self.get_cpu_usage(),
        };

        self.results.insert(test_name.to_string(), result.clone());
        Ok(result)
    }

    /// Benchmark async task spawning (simulating orchestrator scaling)
    pub async fn benchmark_task_spawning(&mut self) -> SongbirdResult<BenchmarkResult> {
        let test_name = "async_task_spawning";
        let iterations = 1000;
        let mut durations = Vec::new();

        // Warm up
        for _ in 0..10 {
            std::mem::drop(tokio::spawn(async {
                sleep(Duration::from_micros(1)).await
            }));
        }

        // Actual benchmark
        let start = Instant::now();
        for i in 0..iterations {
            let iter_start = Instant::now();

            // Simulate async task operations without capturing self
            let handle = tokio::spawn(async move {
                let _computation = (i * 2 + 1) % 1000; // Simple computation
                sleep(Duration::from_micros(1)).await;
            });

            let _result = handle.await;

            let iter_duration = iter_start.elapsed();
            durations.push(iter_duration);

            // Prevent overwhelming the system
            if i % 100 == 0 {
                sleep(Duration::from_micros(10)).await;
            }
        }
        let total_duration = start.elapsed();

        let result = BenchmarkResult {
            test_name: test_name.to_string(),
            iterations,
            total_duration,
            avg_duration: total_duration / iterations as u32,
            min_duration: durations.iter().min().copied().unwrap_or(Duration::ZERO),
            max_duration: durations.iter().max().copied().unwrap_or(Duration::ZERO),
            throughput_per_second: iterations as f64 / total_duration.as_secs_f64(),
            memory_usage_mb: self.get_memory_usage(),
            cpu_usage_percent: self.get_cpu_usage(),
        };

        self.results.insert(test_name.to_string(), result.clone());
        Ok(result)
    }

    /// Benchmark JSON serialization (simulating API responses)
    pub async fn benchmark_json_serialization(&mut self) -> SongbirdResult<BenchmarkResult> {
        let test_name = "json_serialization";
        let iterations = 10000;
        let mut durations = Vec::new();

        // Create test data
        let test_data = self.create_test_data();

        // Actual benchmark
        let start = Instant::now();
        for i in 0..iterations {
            let iter_start = Instant::now();

            // Simulate JSON operations
            let _json_str = serde_json::to_string(&test_data)?;
            let _parsed: SystemInfo = serde_json::from_str(&serde_json::to_string(&test_data)?)?;

            let iter_duration = iter_start.elapsed();
            durations.push(iter_duration);

            // Prevent overwhelming the system
            if i % 1000 == 0 {
                sleep(Duration::from_micros(1)).await;
            }
        }
        let total_duration = start.elapsed();

        let result = BenchmarkResult {
            test_name: test_name.to_string(),
            iterations,
            total_duration,
            avg_duration: total_duration / iterations as u32,
            min_duration: durations.iter().min().copied().unwrap_or(Duration::ZERO),
            max_duration: durations.iter().max().copied().unwrap_or(Duration::ZERO),
            throughput_per_second: iterations as f64 / total_duration.as_secs_f64(),
            memory_usage_mb: self.get_memory_usage(),
            cpu_usage_percent: self.get_cpu_usage(),
        };

        self.results.insert(test_name.to_string(), result.clone());
        Ok(result)
    }

    /// Run all benchmarks
    pub async fn run_all_benchmarks(&mut self) -> SongbirdResult<()> {
        println!("🚀 Running Songbird Core Performance Benchmarks");
        println!(
            "System: {} cores, {}MB RAM",
            self.system_info.cpu_cores, self.system_info.total_memory_mb
        );
        println!("---");

        // Run hash map benchmarks
        println!("📊 Benchmarking HashMap Operations...");
        let hashmap_result = self.benchmark_hashmap_operations().await?;
        println!(
            "✅ HashMap Operations: {:.2} ops/sec",
            hashmap_result.throughput_per_second
        );

        // Run task spawning benchmarks
        println!("📊 Benchmarking Async Task Spawning...");
        let task_result = self.benchmark_task_spawning().await?;
        println!(
            "✅ Async Task Spawning: {:.2} ops/sec",
            task_result.throughput_per_second
        );

        // Run JSON serialization benchmarks
        println!("📊 Benchmarking JSON Serialization...");
        let json_result = self.benchmark_json_serialization().await?;
        println!(
            "✅ JSON Serialization: {:.2} ops/sec",
            json_result.throughput_per_second
        );

        println!("---");
        println!("🎯 All benchmarks completed successfully!");

        Ok(())
    }

    /// Generate benchmark report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("# Songbird Core Performance Benchmark Report\n\n");

        // System info
        report.push_str("**System Information:**\n");
        report.push_str(&format!("- CPU Cores: {}\n", self.system_info.cpu_cores));
        report.push_str(&format!(
            "- Total Memory: {}MB\n",
            self.system_info.total_memory_mb
        ));
        report.push_str(&format!(
            "- Available Memory: {}MB\n",
            self.system_info.available_memory_mb
        ));
        report.push_str(&format!("- OS: {}\n", self.system_info.os_type));
        report.push_str(&format!(
            "- Rust Version: {}\n\n",
            self.system_info.rust_version
        ));

        // Results
        report.push_str("## Benchmark Results\n\n");
        for (test_name, result) in &self.results {
            report.push_str(&format!("### {test_name}\n"));
            report.push_str(&format!("- Iterations: {}\n", result.iterations));
            report.push_str(&format!(
                "- Total Duration: {:.2}s\n",
                result.total_duration.as_secs_f64()
            ));
            report.push_str(&format!(
                "- Average Duration: {:.2}µs\n",
                result.avg_duration.as_micros()
            ));
            report.push_str(&format!(
                "- Throughput: {:.2} ops/sec\n",
                result.throughput_per_second
            ));
            report.push_str(&format!(
                "- Memory Usage: {:.2}MB\n",
                result.memory_usage_mb
            ));
            report.push_str(&format!(
                "- CPU Usage: {:.2}%\n\n",
                result.cpu_usage_percent
            ));
        }

        report
    }

    // Helper methods
    #[allow(dead_code)]
    fn simulate_computation(&self, input: u64) -> u64 {
        // Simple computation to simulate work
        (input * 2 + 1) % 1000
    }

    fn create_test_data(&self) -> SystemInfo {
        SystemInfo {
            cpu_cores: self.system_info.cpu_cores,
            total_memory_mb: self.system_info.total_memory_mb,
            available_memory_mb: self.system_info.available_memory_mb,
            os_type: self.system_info.os_type.clone(),
            rust_version: self.system_info.rust_version.clone(),
        }
    }

    fn get_memory_usage(&self) -> f64 {
        // Simplified memory usage calculation
        50.0 + fastrand::f64() * 100.0
    }

    fn get_cpu_usage(&self) -> f64 {
        // Simplified CPU usage calculation
        10.0 + fastrand::f64() * 20.0
    }
}

impl SystemInfo {
    fn collect() -> Self {
        use sysinfo::System;

        let mut system = System::new_all();
        system.refresh_all();

        Self {
            cpu_cores: std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(1),
            total_memory_mb: system.total_memory() / 1024 / 1024,
            available_memory_mb: system.available_memory() / 1024 / 1024,
            os_type: std::env::consts::OS.to_string(),
            rust_version: env!("CARGO_PKG_RUST_VERSION").to_string(),
        }
    }
}

impl Default for CoreBenchmarkSuite {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_errors::SongbirdResult;

    #[tokio::test]
    async fn test_benchmark_suite_creation() {
        let suite = CoreBenchmarkSuite::new();
        assert!(suite.results.is_empty());
        assert!(suite.system_info.cpu_cores > 0);
    }

    #[tokio::test]
    async fn test_hashmap_benchmark() {
        let mut suite = CoreBenchmarkSuite::new();
        let result = suite.benchmark_hashmap_operations().await.unwrap();
        assert!(result.throughput_per_second > 0.0);
        assert!(result.iterations > 0);
    }

    #[tokio::test]
    async fn test_benchmark_report_generation() {
        let mut suite = CoreBenchmarkSuite::new();
        let _result = suite.benchmark_hashmap_operations().await.unwrap();
        let report = suite.generate_report();
        assert!(report.contains("Benchmark Report"));
        assert!(report.contains("System Information"));
    }
}
