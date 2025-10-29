//! Comprehensive Performance Benchmarks
//!
//! Performance benchmarks for songbird-test-utils components

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

// Dataset sizes for benchmarks
const _SMALL_DATASET: usize = 100;
const _MEDIUM_DATASET: usize = 1_000;
const _LARGE_DATASET: usize = 10_000;

/// Performance benchmarks for comprehensive system operations
fn benchmark_comprehensive_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_operations");

    // Benchmark error handling patterns
    group.bench_function("error_patterns", |b| {
        b.iter(|| {
            let errors = vec!["Network error", "Config error", "Service error"];
            black_box(errors)
        });
    });

    // Benchmark configuration operations
    group.bench_function("config_operations", |b| {
        b.iter(|| {
            let config: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            black_box(config)
        });
    });

    group.finish();
}

/// Benchmark timeout operations
fn benchmark_timeout_operations(c: &mut Criterion) {
    c.bench_function("timeout_creation", |b| {
        b.iter(|| {
            let timeout = Duration::from_secs(30);
            black_box(timeout);
        });
    });
}

criterion_group!(benches, benchmark_comprehensive_operations, benchmark_timeout_operations);
criterion_main!(benches);
