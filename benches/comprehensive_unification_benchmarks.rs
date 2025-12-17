//! Comprehensive Unification Benchmarks
//!
//! Performance benchmarks for unified configuration and type systems

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use songbird_types::UnifiedSongbirdConfig;

/// Benchmark configuration creation performance
fn benchmark_config_creation(c: &mut Criterion) {
    let _group = c.bench_function("unified_config_creation", |b| {
        b.iter(|| black_box(UnifiedSongbirdConfig::default()));
    });
}

/// Benchmark configuration serialization performance
#[allow(clippy::expect_used)] // Acceptable in benchmarks
fn benchmark_config_serialization(c: &mut Criterion) {
    let config = UnifiedSongbirdConfig::default();
    let _group = c.bench_function("unified_config_serialization", |b| {
        b.iter(|| {
            black_box(
                serde_json::to_string(&config).expect("Serialization should succeed in benchmarks"),
            )
        });
    });
}

criterion_group!(benches, benchmark_config_creation, benchmark_config_serialization);
criterion_main!(benches);
