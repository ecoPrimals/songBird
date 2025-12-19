//! Benchmark for optimization validation
//!
//! Tests the performance of optimized constants and configurations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Test constants - use functions for environment-aware values
const OPTIMIZATION_PORT: u16 = 8080;
const OPTIMIZATION_TIMEOUT: u64 = 1_000;

/// Benchmark optimization validation
fn benchmark_optimization_validation(c: &mut Criterion) {
    c.bench_function("config_optimization", |b| {
        b.iter(|| {
            // Test configuration access patterns using environment-aware functions
            let _bind_addr = black_box(songbird_config::canonical::constants::get_bind_address());
            let _host = black_box(songbird_config::canonical::constants::network::default_host());
            let _port = black_box(OPTIMIZATION_PORT);
            let _timeout = black_box(OPTIMIZATION_TIMEOUT);
        });
    });

    c.bench_function("memory_optimization", |b| {
        b.iter(|| {
            // Test memory access patterns
            let _bind_addr = black_box(songbird_config::canonical::constants::get_bind_address());
            let _host = black_box(songbird_config::canonical::constants::network::default_host());
            let _port = black_box(OPTIMIZATION_PORT);
            let _timeout = black_box(OPTIMIZATION_TIMEOUT);
        });
    });
}

criterion_group!(benches, benchmark_optimization_validation);
criterion_main!(benches);
