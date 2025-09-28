//! Critical Path Performance Benchmarks
//!
//! Benchmarks for the most performance-critical operations in Songbird
//! to establish baseline metrics and detect performance regressions.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use songbird_config::SongbirdConfig;
use songbird_errors::{SongbirdError, SongbirdResult};
use songbird_types::*;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

/// Benchmark error creation and conversion performance
fn bench_error_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_operations");
    
    group.bench_function("create_config_error", |b| {
        b.iter(|| {
            let error = SongbirdError::configuration(black_box("Test configuration error"));
            black_box(error);
        });
    });
    
    group.bench_function("create_network_error", |b| {
        b.iter(|| {
            let error = SongbirdError::network(black_box("Test network error"));
            black_box(error);
        });
    });
    
    group.bench_function("parse_int_error_conversion", |b| {
        b.iter(|| {
            let parse_error = black_box("not_a_number").parse::<i32>().unwrap_err();
            let songbird_error: SongbirdError = parse_error.into();
            black_box(songbird_error);
        });
    });
    
    group.finish();
}

/// Benchmark configuration operations
fn bench_configuration_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_operations");
    
    group.bench_function("create_default_config", |b| {
        b.iter(|| {
            let config = SongbirdConfig::default();
            black_box(config);
        });
    });
    
    group.bench_function("config_modification", |b| {
        b.iter(|| {
            let mut config = SongbirdConfig::default();
            if let Some(ref mut network) = config.network {
                network.bind_address = black_box("127.0.0.1".to_string());
                network.http_port = black_box(8080);
            }
            black_box(config);
        });
    });
    
    group.finish();
}

/// Benchmark async operations
fn bench_async_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("async_operations");
    
    group.bench_function("simple_async_task", |b| {
        b.to_async(&rt).iter(|| async {
            let result = async {
                tokio::time::sleep(Duration::from_nanos(1)).await;
                black_box(42)
            }.await;
            black_box(result);
        });
    });
    
    group.bench_function("concurrent_tasks", |b| {
        b.to_async(&rt).iter(|| async {
            let tasks: Vec<_> = (0..black_box(10)).map(|i| {
                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_nanos(1)).await;
                    i * 2
                })
            }).collect();

            let results: Vec<_> = futures::future::join_all(tasks)
                .await
                .into_iter()
                .map(|r| r.unwrap())
                .collect();
            
            black_box(results);
        });
    });
    
    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_operations");
    
    group.bench_function("vec_allocation", |b| {
        b.iter(|| {
            let mut vec = Vec::with_capacity(black_box(1000));
            for i in 0..1000 {
                vec.push(black_box(i));
            }
            black_box(vec);
        });
    });
    
    group.bench_function("string_operations", |b| {
        b.iter(|| {
            let mut string = String::with_capacity(black_box(1000));
            for i in 0..100 {
                string.push_str(&format!("item_{}", black_box(i)));
            }
            black_box(string);
        });
    });
    
    // Benchmark different collection sizes
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("vec_with_size", size), size, |b, &size| {
            b.iter(|| {
                let vec: Vec<usize> = (0..size).collect();
                black_box(vec);
            });
        });
    }
    
    group.finish();
}

/// Benchmark serialization/deserialization operations
fn bench_serialization_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization_operations");
    
    // Create test data
    let test_data = serde_json::json!({
        "service_id": "test-service-001",
        "capabilities": ["compute", "storage", "networking"],
        "metadata": {
            "version": "1.0.0",
            "status": "active",
            "endpoints": ["http://127.0.0.1:8080", "http://127.0.0.1:8081"]
        }
    });
    
    let json_string = serde_json::to_string(&test_data).unwrap();
    
    group.bench_function("json_serialize", |b| {
        b.iter(|| {
            let serialized = serde_json::to_string(&black_box(&test_data)).unwrap();
            black_box(serialized);
        });
    });
    
    group.bench_function("json_deserialize", |b| {
        b.iter(|| {
            let deserialized: serde_json::Value = serde_json::from_str(&black_box(&json_string)).unwrap();
            black_box(deserialized);
        });
    });
    
    group.finish();
}

/// Benchmark time-critical operations
fn bench_time_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("time_operations");
    
    group.bench_function("system_time_now", |b| {
        b.iter(|| {
            let now = std::time::SystemTime::now();
            black_box(now);
        });
    });
    
    group.bench_function("instant_now", |b| {
        b.iter(|| {
            let now = Instant::now();
            black_box(now);
        });
    });
    
    group.bench_function("duration_calculation", |b| {
        let start = Instant::now();
        b.iter(|| {
            let elapsed = start.elapsed();
            black_box(elapsed);
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_error_operations,
    bench_configuration_operations,
    bench_async_operations,
    bench_memory_operations,
    bench_serialization_operations,
    bench_time_operations
);

criterion_main!(benches); 