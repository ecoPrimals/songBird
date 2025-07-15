//! Performance Benchmarking Module
//!
//! Provides performance benchmarking utilities for Songbird components

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Performance benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration: Duration,
    pub iterations: u64,
    pub throughput: Option<f64>,
    pub metadata: HashMap<String, String>,
}

/// Benchmark runner for various Songbird components
#[derive(Default)]
pub struct BenchmarkRunner {
    pub results: Vec<BenchmarkResult>,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Run a benchmark function
    pub fn run_benchmark<F>(
        &mut self,
        name: String,
        iterations: usize,
        mut func: F,
    ) -> BenchmarkResult
    where
        F: FnMut(),
    {
        debug!("Running benchmark: {} ({} iterations)", name, iterations);

        let start = Instant::now();
        for _ in 0..iterations {
            func();
        }
        let duration = start.elapsed();

        let result = BenchmarkResult {
            name: name.to_string(),
            duration,
            iterations: iterations as u64,
            throughput: Some(iterations as f64 / duration.as_secs_f64()),
            metadata: HashMap::new(),
        };

        info!("Benchmark {} completed: {:?}", name, result);
        self.results.push(result.clone());
        result
    }

    /// Benchmark async function
    pub async fn benchmark_async<F, Fut>(
        &mut self,
        name: &str,
        iterations: u64,
        mut func: F,
    ) -> BenchmarkResult
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        debug!(
            "Running async benchmark: {} ({} iterations)",
            name, iterations
        );

        let start = Instant::now();
        for _ in 0..iterations {
            func().await;
        }
        let duration = start.elapsed();

        let result = BenchmarkResult {
            name: name.to_string(),
            duration,
            iterations,
            throughput: Some(iterations as f64 / duration.as_secs_f64()),
            metadata: HashMap::new(),
        };

        info!("Async benchmark {} completed: {:?}", name, result);
        self.results.push(result.clone());
        result
    }

    /// Get benchmark results
    pub fn get_results(&self) -> &[BenchmarkResult] {
        &self.results
    }

    /// Clear all results
    pub fn clear_results(&mut self) {
        self.results.clear();
    }

    /// Get summary statistics
    pub fn get_summary(&self) -> BenchmarkSummary {
        if self.results.is_empty() {
            return BenchmarkSummary::default();
        }

        let total_duration: Duration = self.results.iter().map(|r| r.duration).sum();
        let avg_duration = total_duration / self.results.len() as u32;
        let total_iterations: u64 = self.results.iter().map(|r| r.iterations).sum();

        BenchmarkSummary {
            total_benchmarks: self.results.len(),
            total_duration,
            avg_duration,
            total_iterations,
            fastest: self.results.iter().min_by_key(|r| r.duration).cloned(),
            slowest: self.results.iter().max_by_key(|r| r.duration).cloned(),
        }
    }
}

/// Summary of benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub total_benchmarks: usize,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub total_iterations: u64,
    pub fastest: Option<BenchmarkResult>,
    pub slowest: Option<BenchmarkResult>,
}

impl Default for BenchmarkSummary {
    fn default() -> Self {
        Self {
            total_benchmarks: 0,
            total_duration: Duration::ZERO,
            avg_duration: Duration::ZERO,
            total_iterations: 0,
            fastest: None,
            slowest: None,
        }
    }
}

/// Predefined benchmark suites
pub mod suites {
    use super::*;

    /// Network performance benchmarks
    pub async fn network_benchmarks(runner: &mut BenchmarkRunner) {
        // Connection establishment benchmark
        runner.run_benchmark("connection_establishment".to_string(), 100, || {
            // Simulate connection establishment
            std::thread::sleep(Duration::from_micros(10));
        });

        // Data throughput benchmark
        runner.run_benchmark("data_throughput".to_string(), 1000, || {
            // Simulate data processing
            let _data = vec![0u8; 1024];
        });
    }

    /// Security benchmarks
    pub async fn security_benchmarks(runner: &mut BenchmarkRunner) {
        // Encryption benchmark
        runner.run_benchmark("encryption".to_string(), 100, || {
            // Simulate encryption operation
            std::thread::sleep(Duration::from_micros(50));
        });

        // Authentication benchmark
        runner.run_benchmark("authentication".to_string(), 50, || {
            // Simulate authentication
            std::thread::sleep(Duration::from_micros(100));
        });
    }

    /// Gaming performance benchmarks
    pub async fn gaming_benchmarks(runner: &mut BenchmarkRunner) {
        // Latency benchmark
        runner.run_benchmark("gaming_latency".to_string(), 1000, || {
            // Simulate low-latency gaming operation
            std::thread::sleep(Duration::from_micros(1));
        });

        // Packet processing benchmark
        runner.run_benchmark("packet_processing".to_string(), 5000, || {
            // Simulate packet processing
            let _packet = vec![0u8; 512];
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_runner() {
        let mut runner = BenchmarkRunner::new();

        let result = runner.run_benchmark("test_benchmark".to_string(), 10, || {
            std::thread::sleep(Duration::from_millis(1));
        });

        assert_eq!(result.name, "test_benchmark");
        assert_eq!(result.iterations, 10);
        assert!(result.duration > Duration::from_millis(8));
        assert!(result.throughput.is_some());
    }

    #[tokio::test]
    async fn test_async_benchmark() {
        let mut runner = BenchmarkRunner::new();

        let result = runner
            .benchmark_async("test_async_benchmark", 5, || async {
                tokio::time::sleep(Duration::from_millis(1)).await;
            })
            .await;

        assert_eq!(result.name, "test_async_benchmark");
        assert_eq!(result.iterations, 5);
        assert!(result.duration > Duration::from_millis(4));
    }

    #[test]
    fn test_benchmark_summary() {
        let mut runner = BenchmarkRunner::new();

        runner.run_benchmark("fast".to_string(), 100, || {
            std::thread::sleep(Duration::from_micros(1));
        });

        runner.run_benchmark("slow".to_string(), 10, || {
            std::thread::sleep(Duration::from_millis(1));
        });

        let summary = runner.get_summary();
        assert_eq!(summary.total_benchmarks, 2);
        assert!(summary.fastest.is_some());
        assert!(summary.slowest.is_some());
    }

    #[tokio::test]
    async fn test_benchmark_suites() {
        let mut runner = BenchmarkRunner::new();

        suites::network_benchmarks(&mut runner).await;
        suites::security_benchmarks(&mut runner).await;
        suites::gaming_benchmarks(&mut runner).await;

        let summary = runner.get_summary();
        assert!(summary.total_benchmarks > 0);
    }
}
