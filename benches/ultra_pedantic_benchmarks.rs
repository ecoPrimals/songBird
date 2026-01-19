//! Ultra-Pedantic Performance Benchmarks
//!
//! Comprehensive benchmarks validating all performance optimizations
//! including const functions, inline optimizations, and zero-copy patterns.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

// Import the optimized types
use songbird_config::canonical_network::CanonicalNetworkConfig;
use songbird_types::{
    errors::SongbirdError,
    primal::{CanonicalPrimalConfig, CanonicalPrimalType},
    response::CanonicalResponse,
};

/// Benchmark const function performance
fn bench_const_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("const_functions");

    let config = CanonicalNetworkConfig {
        bind_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        orchestrator_port: 8080,
        discovery_port: 8001,
        metrics_bind_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        metrics_port: 9090,
        federation_bind_address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        federation_port: 8005,
        ..Default::default()
    };

    // Benchmark const endpoint functions
    group.bench_function("orchestrator_endpoint", |b| {
        b.iter(|| black_box(config.orchestrator_endpoint()))
    });

    group.bench_function("discovery_endpoint", |b| {
        b.iter(|| black_box(config.discovery_endpoint()))
    });

    group.bench_function("metrics_endpoint", |b| b.iter(|| black_box(config.metrics_endpoint())));

    group.bench_function("federation_endpoint", |b| {
        b.iter(|| black_box(config.federation_endpoint()))
    });

    group.finish();
}

/// Benchmark error handling performance
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    let error = SongbirdError::Configuration {
        message: "Test configuration error".to_string(),
        field: Some("test_field".to_string()),
        suggestion: Some("Use correct configuration".to_string()),
    };

    // Benchmark const error methods
    group.bench_function("error_category", |b| b.iter(|| black_box(error.category())));

    group.bench_function("error_is_recoverable", |b| b.iter(|| black_box(error.is_recoverable())));

    group.finish();
}

/// Benchmark response handling performance
fn bench_response_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_handling");

    let success_response = CanonicalResponse::success("test_data".to_string());
    let error_response = CanonicalResponse::error("test_error".to_string());

    // Benchmark const response methods
    group.bench_function("success_check", |b| b.iter(|| black_box(success_response.is_success())));

    group.bench_function("error_check", |b| b.iter(|| black_box(error_response.is_error())));

    // Benchmark response creation
    group.bench_function("success_creation", |b| {
        b.iter(|| black_box(CanonicalResponse::success("benchmark_data".to_string())))
    });

    group.finish();
}

/// Benchmark primal type operations
fn bench_primal_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("primal_operations");

    let primal_config = CanonicalPrimalConfig {
        primal_type: CanonicalPrimalType::Security,
        instance_id: "test_instance".to_string(),
        capabilities: vec!["auth".to_string(), "encryption".to_string()],
        metadata: std::collections::HashMap::new(),
        health_check_interval: Duration::from_secs(30),
        timeout: Duration::from_secs(10),
    };

    // Benchmark const primal methods
    group.bench_function("get_type", |b| b.iter(|| black_box(primal_config.get_type())));

    group.bench_function("is_security", |b| b.iter(|| black_box(primal_config.is_security())));

    group.finish();
}

/// Benchmark throughput with different data sizes
fn bench_throughput_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_scaling");

    // Test with different payload sizes
    for size in [1, 10, 100, 1000, 10000].iter() {
        let data = "x".repeat(*size);

        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::new("response_creation", size), size, |b, &_size| {
            b.iter(|| black_box(CanonicalResponse::success(data.clone())))
        });
    }

    group.finish();
}

/// Benchmark zero-copy patterns
fn bench_zero_copy_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("zero_copy");

    let large_string = "x".repeat(10000);

    // Compare different approaches
    group.bench_function("string_clone", |b| {
        b.iter(|| {
            let _cloned = black_box(large_string.clone());
        })
    });

    group.bench_function("string_reference", |b| {
        b.iter(|| {
            let _reference = black_box(&large_string);
        })
    });

    group.finish();
}

/// Benchmark inline function performance
fn bench_inline_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("inline_performance");

    let config = CanonicalNetworkConfig::default();

    // Benchmark calls to inlined functions in a tight loop
    group.bench_function("inline_endpoint_calls", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(config.orchestrator_endpoint());
                black_box(config.discovery_endpoint());
                black_box(config.metrics_endpoint());
                black_box(config.federation_endpoint());
            }
        })
    });

    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    // Benchmark stack vs heap allocation patterns
    group.bench_function("stack_allocation", |b| {
        b.iter(|| {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
            black_box(addr)
        })
    });

    group.bench_function("heap_allocation", |b| {
        b.iter(|| {
            let data = vec![1, 2, 3, 4, 5];
            black_box(data)
        })
    });

    group.finish();
}

/// Comprehensive benchmark suite
fn bench_comprehensive_suite(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive");

    // Real-world scenario: Creating a complete network configuration
    group.bench_function("full_config_creation", |b| {
        b.iter(|| {
            let config = black_box(CanonicalNetworkConfig {
                bind_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                orchestrator_port: 8080,
                discovery_port: 8001,
                metrics_bind_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                metrics_port: 9090,
                federation_bind_address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                federation_port: 8005,
                ..Default::default()
            });

            // Use all the const functions
            let _orch = config.orchestrator_endpoint();
            let _disc = config.discovery_endpoint();
            let _metrics = config.metrics_endpoint();
            let _fed = config.federation_endpoint();
        })
    });

    group.finish();
}

criterion_group!(
    ultra_pedantic_benches,
    bench_const_functions,
    bench_error_handling,
    bench_response_handling,
    bench_primal_operations,
    bench_throughput_scaling,
    bench_zero_copy_patterns,
    bench_inline_performance,
    bench_memory_patterns,
    bench_comprehensive_suite
);

criterion_main!(ultra_pedantic_benches);
