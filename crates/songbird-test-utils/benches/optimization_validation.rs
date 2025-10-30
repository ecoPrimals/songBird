//! Benchmark for optimization validation
//!
//! Tests the performance of optimized constants and configurations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Test constants
const DEFAULT_BIND_ADDRESS: &str = songbird_config::constants::network::DEFAULT_HOST;
const DEFAULT_LOCALHOST: &str = songbird_config::constants::network::DEFAULT_HOST;
const OPTIMIZATION_PORT: u16 = 8080;
const OPTIMIZATION_TIMEOUT: u64 = 1_000;

/// Benchmark optimization validation
fn benchmark_optimization_validation(c: &mut Criterion) {
    c.bench_function("config_optimization", |b| {
        b.iter(|| {
            black_box(&DEFAULT_BIND_ADDRESS);
            black_box(&DEFAULT_LOCALHOST);
            black_box(OPTIMIZATION_PORT);
            black_box(OPTIMIZATION_TIMEOUT);
        });
    });

    c.bench_function("memory_optimization", |b| {
        b.iter(|| {
            black_box(&DEFAULT_BIND_ADDRESS);
            black_box(&DEFAULT_LOCALHOST);
            black_box(OPTIMIZATION_PORT);
            black_box(OPTIMIZATION_TIMEOUT);
        });
    });
}

criterion_group!(benches, benchmark_optimization_validation);
criterion_main!(benches);
