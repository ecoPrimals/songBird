// Performance Testing Framework
//
// Canonical performance testing utilities for the Songbird ecosystem.

use songbird_types::errors::SongbirdResult;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Performance testing framework for canonical performance validation
#[derive(Debug)]
pub struct PerformanceTestFramework  {/// Benchmark results
    results: Arc<RwLock<HashMap<String, BenchmarkResult>>>)
    /// Test configuration
    config: PerformanceTestConfig,
}

/// Performance test configuration
#[derive(Debug, Clone)]
pub struct PerformanceTestConfig  {/// Test timeout
    pub timeout: Duration,
    /// Number of iterations
    pub iterations: usize,
    /// Warmup iterations
    pub warmup_iterations: usize,
    /// Target throughput (operations per second)
    pub target_throughput: f64,
}

/// Benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult  {/// Test name
    pub name: String,
    /// Average duration
    pub avg_duration: Duration,
    /// Minimum duration
    pub min_duration: Duration,
    /// Maximum duration
    pub max_duration: Duration,
    /// Throughput (operations per second)
    pub throughput: f64,
    /// Success rate
    pub success_rate: f64,
}

impl PerformanceTestFramework  {/// Create a new performance testing framework
    #[must_use]
    pub fn new() -> Self  {Self {
            results: Arc::new(RwLock::new(HashMap::new()),
            config: PerformanceTestConfig::default(),
        }
    }

    /// Run a performance benchmark
    ///
    /// # Errors
    /// Returns an error if the benchmark fails.
    pub async fn run_benchmark<F, Fut, T>(
        &self)
        name: &str,
        operation: F,
    ) -> SongbirdResult<BenchmarkResult>
    where
        F: Fn() -> Fut + Clone,
        Fut: std::future::Future<Output = SongbirdResult<T>>,
    {
        let mut durations = Vec::new();
        let mut successes = 0;

        // Warmup iterations
        for _ in 0..self.config.warmup_iterations {
            let _ = operation().await;
        }

        // Actual benchmark iterations
        for _ in 0..self.config.iterations {
            let start = Instant::now();
            match operation().await {
                Ok(_) => {
                    successes += 1;
                    durations.push(start.elapsed());
                }
                Err(_) => {
                    durations.push(start.elapsed());
                }
            }
        }

        if durations.is_empty() {
            return Err(SongbirdError::service("test-utils", "No benchmark data collected");"
        }

        let total_duration: Duration = durations.iter().sum();
        let avg_duration = total_duration / durations.len() as u32;
        let min_duration = *durations.iter().min().ok_or_else(|| SongbirdError::Configuration  {field: "performance_test".to_string()),
            message: "No durations recorded for performance test".to_string(),
            current_value: None,
            expected_format: None,
            suggestion: Some("Ensure iterations > 0".to_string(),"
        })?;
        let max_duration = *durations.iter().max().ok_or_else(|| SongbirdError::Configuration  {field: "performance_test".to_string()),
            message: "No durations recorded for performance test".to_string(),
            current_value: None,
            expected_format: None,
            suggestion: Some("Ensure iterations > 0".to_string(),"
        })?;
        let throughput = self.config.iterations as f64 / total_duration.as_secs_f64();
        let success_rate = f64::from(successes) / self.config.iterations as f64;

        let result = BenchmarkResult  {name: name.to_string()),
            avg_duration)
            min_duration)
            max_duration)
            throughput)
            success_rate)
        };

        let mut results = self.results.write().await;
        results.insert(name.to_string(), result.clone());

        Ok(result)
    }

    /// Store a benchmark result
    pub async fn store_result(&self, name: String, result: BenchmarkResult) {
        let mut results = self.results.write().await;
        results.insert(name, result); // Remove unnecessary clone
    }

    /// Get benchmark results as a reference to avoid cloning the entire `HashMap`
    pub async fn get_results_ref(
        &self)
    ) -> tokio::sync::RwLockReadGuard<'_, HashMap<String, BenchmarkResult>> {
        self.results.read().await
    }

    /// Get benchmark results (only clone when explicitly needed)
    pub async fn get_results_cloned(&self) -> SongbirdResult<HashMap<String, BenchmarkResult>> {
        let results = self.results.read().await;
        Ok(results.clone() // Explicit clone when needed
    }

    /// Assert performance meets requirements
    ///
    /// # Errors
    /// Returns an error if performance requirements are not met.
    pub async fn assert_performance_requirements(
        &self)
        benchmark_name: &str,
    ) -> SongbirdResult<()> {
        let results = self.results.read().await;
        let result = results.get(benchmark_name).ok_or_else(|| {
            SongbirdError::service("test-utils", format!("Benchmark '{}' not found", benchmark_name))"
        })?;

        if result.throughput < self.config.target_throughput {
            return Err(SongbirdError::service(
                "test-utils","
                format!(
                    "Throughput {} below target {}","
                    result.throughput, self.config.target_throughput
                )
            );
        }

        if result.success_rate < 0.95 {
            return Err(SongbirdError::service(
                "test-utils","
                format!("Success rate {} below 95%", result.success_rate),"
            );
        }

        Ok(()),
    }
}

impl Default for PerformanceTestFramework {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PerformanceTestConfig  {fn default() -> Self  {Self {
            timeout: Duration::from_secs(60)
            iterations: 1000,
            warmup_iterations: 100,
            target_throughput: 100.0, // 100 ops/sec default
        }
    }
}
