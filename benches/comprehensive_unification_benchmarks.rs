//! Comprehensive Unification Benchmarks
//!
//! Performance benchmarks for unified configuration and type systems

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use songbird_types::UnifiedSongbirdConfig;

fn benchmark_config_creation(c: &mut Criterion) {
    c.bench_function("unified_config_creation", |b| {
        b.iter(|| black_box(UnifiedSongbirdConfig::default()));
    });
}

fn benchmark_config_serialization(c: &mut Criterion) {
    let config = UnifiedSongbirdConfig::default();
    c.bench_function("unified_config_serialization", |b| {
        b.iter(|| black_box(serde_json::to_string(&config).unwrap()));
    });
}

criterion_group!(
    benches,
    benchmark_config_creation,
    benchmark_config_serialization
);
criterion_main!(benches);
