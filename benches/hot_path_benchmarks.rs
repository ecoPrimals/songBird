//! Hot Path Performance Benchmarks
//!
//! Benchmarks for identifying and measuring performance of hot paths
//! in Songbird to guide optimization efforts.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use songbird_test_utils::fixtures::*;
use songbird_types::{CapabilityRequest, ServiceId};
use songbird_universal::UniversalCapabilityAdapter;
use std::time::Duration;

/// Benchmark service discovery operations
fn bench_service_discovery(c: &mut Criterion) {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    // Register test services
    let service = compute_service_fixture();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        adapter.register_service(service).await.ok();
    });

    c.bench_function("service_discovery", |b| {
        b.to_async(&rt).iter(|| async {
            adapter.discover_capability_providers(black_box("compute")).await.ok()
        })
    });
}

/// Benchmark capability routing
fn bench_capability_routing(c: &mut Criterion) {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    let service = compute_service_fixture();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        adapter.register_service(service).await.ok();
    });

    let request = CapabilityRequest {
        capability: "compute".to_string(),
        operation: "process".to_string(),
        parameters: Default::default(),
        timeout: Duration::from_secs(10),
    };

    c.bench_function("capability_routing", |b| {
        b.to_async(&rt).iter(|| async {
            adapter.execute_capability_request(black_box(request.clone())).await.ok()
        })
    });
}

/// Benchmark service registration
fn bench_service_registration(c: &mut Criterion) {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("service_registration", |b| {
        b.to_async(&rt).iter(|| async {
            let service = compute_service_fixture();
            adapter.register_service(black_box(service)).await.ok()
        })
    });
}

/// Benchmark health check operations
fn bench_health_checks(c: &mut Criterion) {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    let service = compute_service_fixture();
    let service_id = service.id.clone();

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        adapter.register_service(service).await.ok();
    });

    c.bench_function("health_check", |b| {
        b.to_async(&rt)
            .iter(|| async { adapter.get_service_health(black_box(&service_id)).await.ok() })
    });
}

/// Benchmark configuration loading
fn bench_config_operations(c: &mut Criterion) {
    use songbird_config::EnvironmentConfig;

    c.bench_function("config_load", |b| b.iter(|| EnvironmentConfig::from_env().ok()));
}

/// Benchmark clone operations vs references
fn bench_clone_vs_reference(c: &mut Criterion) {
    let service = compute_service_fixture();

    let mut group = c.benchmark_group("clone_comparison");

    group.bench_function("service_clone", |b| {
        b.iter(|| {
            let _cloned = black_box(&service).clone();
        })
    });

    group.bench_function("service_reference", |b| {
        b.iter(|| {
            let _ref = black_box(&service);
        })
    });

    group.finish();
}

/// Benchmark string operations (clone vs reference)
fn bench_string_operations(c: &mut Criterion) {
    let test_string = "test_service_name_with_reasonable_length".to_string();

    let mut group = c.benchmark_group("string_comparison");

    group.bench_function("string_clone", |b| {
        b.iter(|| {
            let _cloned = black_box(&test_string).clone();
        })
    });

    group.bench_function("string_reference", |b| {
        b.iter(|| {
            let _ref: &str = black_box(&test_string);
        })
    });

    group.finish();
}

/// Benchmark various data structure sizes
fn bench_structure_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("structure_cloning");

    for size in [10, 100, 1000].iter() {
        let vec: Vec<String> = (0..*size).map(|i| format!("item_{}", i)).collect();

        group.bench_with_input(BenchmarkId::new("vec_clone", size), size, |b, _| {
            b.iter(|| {
                let _cloned = black_box(&vec).clone();
            })
        });

        group.bench_with_input(BenchmarkId::new("vec_reference", size), size, |b, _| {
            b.iter(|| {
                let _ref = black_box(&vec);
            })
        });
    }

    group.finish();
}

/// Benchmark Arc operations
fn bench_arc_operations(c: &mut Criterion) {
    use std::sync::Arc;

    let service = Arc::new(compute_service_fixture());

    let mut group = c.benchmark_group("arc_comparison");

    group.bench_function("arc_clone", |b| {
        b.iter(|| {
            let _cloned = Arc::clone(black_box(&service));
        })
    });

    group.bench_function("direct_clone", |b| {
        let service = compute_service_fixture();
        b.iter(|| {
            let _cloned = black_box(&service).clone();
        })
    });

    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent_access(c: &mut Criterion) {
    let env = create_healthy_environment();
    let adapter = UniversalCapabilityAdapter::new(env.discovery_config.clone())
        .expect("Failed to create adapter");

    let service = compute_service_fixture();
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        adapter.register_service(service).await.ok();
    });

    c.bench_function("concurrent_discovery", |b| {
        b.to_async(&rt).iter(|| async {
            let mut handles = vec![];

            for _ in 0..10 {
                let adapter = adapter.clone();
                let handle = tokio::spawn(async move {
                    adapter.discover_capability_providers("compute").await.ok()
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.await.ok();
            }
        })
    });
}

/// Benchmark memory allocation patterns
fn bench_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_patterns");

    group.bench_function("frequent_small_allocs", |b| {
        b.iter(|| {
            for i in 0..100 {
                let _s = format!("string_{}", i);
            }
        })
    });

    group.bench_function("single_large_alloc", |b| {
        b.iter(|| {
            let _v: Vec<String> = (0..100).map(|i| format!("string_{}", i)).collect();
        })
    });

    group.bench_function("reuse_allocation", |b| {
        let mut buffer = String::with_capacity(1000);
        b.iter(|| {
            for i in 0..100 {
                buffer.clear();
                buffer.push_str(&format!("string_{}", i));
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_service_discovery,
    bench_capability_routing,
    bench_service_registration,
    bench_health_checks,
    bench_config_operations,
    bench_clone_vs_reference,
    bench_string_operations,
    bench_structure_sizes,
    bench_arc_operations,
    bench_concurrent_access,
    bench_allocation_patterns,
);

criterion_main!(benches);
