// Performance Tests
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]

//
// Tests for performance measurement and benchmarking utilities

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

    fn measure<F, T>(&self, operation: F) -> BenchmarkResult<T>
    where
        F: FnOnce() -> T,
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
    operations: std::collections::HashMap<String, OperationStats>,
}

impl PerformanceMetrics {
    fn new() -> Self {
        Self {
            operations: std::collections::HashMap::new(),
        }
    }

    fn record(&mut self, operation: &str, duration: Duration) {
        let stats =
            self.operations.entry(operation.to_string()).or_insert_with(|| OperationStats {
                count: 0,
                total_duration: Duration::from_secs(0),
            });

        stats.count += 1;
        stats.total_duration += duration;
    }

    fn get_stats(&self, operation: &str) -> OperationStats {
        self.operations.get(operation).cloned().unwrap_or_else(|| OperationStats {
            count: 0,
            total_duration: Duration::from_secs(0),
        })
    }
}

#[derive(Debug, Clone)]
struct OperationStats {
    count: usize,
    total_duration: Duration,
}
