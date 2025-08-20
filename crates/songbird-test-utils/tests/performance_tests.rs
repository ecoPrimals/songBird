//! Performance Tests
//!
//! Tests for performance measurement and benchmarking utilities

use std::time::{Duration, Instant};

#[test]
fn test_benchmark_helpers() {
    let benchmark = Benchmark::new("test_operation");

    let result = benchmark.measure(|| {
        // Simulate work
        let mut sum = 0;
        for i in 0..1000 {
            sum += i;
        }
        sum
    });

    assert!(result.duration < Duration::from_millis(10));
    assert_eq!(result.value, 499500); // Sum of 0..1000
    assert_eq!(result.operation_name, "test_operation");
}

#[test]
fn test_performance_metrics() {
    let mut metrics = PerformanceMetrics::new();

    metrics.record("operation_a", Duration::from_millis(10));
    metrics.record("operation_a", Duration::from_millis(15));
    metrics.record("operation_b", Duration::from_millis(5));

    let stats_a = metrics.get_stats("operation_a");
    assert_eq!(stats_a.count, 2);
    assert_eq!(stats_a.total_duration, Duration::from_millis(25));

    let stats_b = metrics.get_stats("operation_b");
    assert_eq!(stats_b.count, 1);
    assert_eq!(stats_b.total_duration, Duration::from_millis(5));
}

// Performance testing types
#[derive(Debug)]
struct Benchmark {
    operation_name: String,
}

impl Benchmark {
    fn new(name: &str) -> Self {
        Self {
            operation_name: name.to_string(),
        }
    }

    fn measure<F, R>(&self, operation: F) -> BenchmarkResult<R>
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let value = operation();
        let duration = start.elapsed();

        BenchmarkResult {
            operation_name: self.operation_name.clone(),
            duration,
            value,
        }
    }
}

#[derive(Debug)]
struct BenchmarkResult<T> {
    operation_name: String,
    duration: Duration,
    value: T,
}

#[derive(Debug)]
struct PerformanceMetrics {
    operations: std::collections::HashMap<String, Vec<Duration>>,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            operations: std::collections::HashMap::new(),
        }
    }

    fn record(&mut self, operation: &str, duration: Duration) {
        self.operations
            .entry(operation.to_string())
            .or_default()
            .push(duration);
    }

    fn get_stats(&self, operation: &str) -> PerformanceStats {
        let empty_vec = Vec::new();
        let durations = self.operations.get(operation).unwrap_or(&empty_vec);
        let count = durations.len();
        let total_duration = durations.iter().sum();

        PerformanceStats {
            count,
            total_duration,
        }
    }

    #[allow(dead_code)]
    fn get_average_duration(&self, operation: &str) -> f64 {
        if let Some(durations) = self.operations.get(operation) {
            if !durations.is_empty() {
                let total: Duration = durations.iter().sum();
                total.as_secs_f64() / durations.len() as f64
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

#[derive(Debug, Clone)]
struct PerformanceStats {
    count: usize,
    total_duration: Duration,
}
