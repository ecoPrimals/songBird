//! # ⚡ Zero-Cost Abstractions Benchmark
//!
//! **PERFORMANCE VALIDATION** 🚀
//!
//! This benchmark validates that our unified type system and const generics
//! provide true zero-cost abstractions with compile-time optimizations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use songbird_types::{
    performance::{ConstBuffer, PerformanceConfig, ProductionConfig, StackString, StackVec, const_hash},
    primal::CanonicalPrimalType,
};
use songbird_primal_sdk::{
    PrimalConnection, OptimizedPrimalPool, StandardPrimalSDK, HighPerformancePrimalSDK, LightweightPrimalSDK,
    PoolStats,
};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// CONST GENERIC BENCHMARKS
// ============================================================================

fn bench_const_buffer_vs_vec(c: &mut Criterion) {
    let mut group = c.benchmark_group("const_buffer_vs_vec");
    
    // Benchmark const buffer (compile-time sized)
    group.bench_function("const_buffer_push", |b| {
        b.iter(|| {
            let mut buffer: ConstBuffer<u64, 1000> = ConstBuffer::new();
            for i in 0..1000 {
                let _ = buffer.try_push(black_box(i));
            }
            buffer
        });
    });
    
    // Benchmark standard Vec (heap allocated)
    group.bench_function("vec_push", |b| {
        b.iter(|| {
            let mut vec = Vec::with_capacity(1000);
            for i in 0..1000 {
                vec.push(black_box(i));
            }
            vec
        });
    });
    
    group.finish();
}

fn bench_stack_vs_heap_collections(c: &mut Criterion) {
    let mut group = c.benchmark_group("stack_vs_heap_collections");
    
    // Stack-allocated string (zero heap allocation)
    group.bench_function("stack_string", |b| {
        b.iter(|| {
            let mut s: StackString<256> = StackString::new();
            let _ = s.try_push_str(black_box("test_string_for_benchmark"));
            s
        });
    });
    
    // Heap-allocated string
    group.bench_function("heap_string", |b| {
        b.iter(|| {
            let mut s = String::new();
            s.push_str(black_box("test_string_for_benchmark"));
            s
        });
    });
    
    // Stack-allocated vector
    group.bench_function("stack_vec", |b| {
        b.iter(|| {
            let mut vec: StackVec<u32, 64> = StackVec::new();
            for i in 0..64 {
                let _ = vec.try_push(black_box(i));
            }
            vec
        });
    });
    
    // Heap-allocated vector
    group.bench_function("heap_vec", |b| {
        b.iter(|| {
            let mut vec = Vec::with_capacity(64);
            for i in 0..64 {
                vec.push(black_box(i));
            }
            vec
        });
    });
    
    group.finish();
}

// ============================================================================
// COMPILE-TIME OPTIMIZATION BENCHMARKS
// ============================================================================

fn bench_compile_time_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("compile_time_hash");
    
    // Compile-time hash (zero runtime cost)
    group.bench_function("const_hash", |b| {
        b.iter(|| {
            // This should be optimized to a compile-time constant
            const HASH: u64 = const_hash("security_primal_type");
            black_box(HASH)
        });
    });
    
    // Runtime hash calculation
    group.bench_function("runtime_hash", |b| {
        b.iter(|| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            
            let mut hasher = DefaultHasher::new();
            black_box("security_primal_type").hash(&mut hasher);
            hasher.finish()
        });
    });
    
    group.finish();
}

fn bench_performance_config_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_config");
    
    // Production config (debug code eliminated at compile time)
    group.bench_function("production_config", |b| {
        let config = ProductionConfig::new();
        b.iter(|| {
            // This debug code should be eliminated in release builds
            config.debug_only(|| {
                println!("This should not appear in benchmarks");
            });
            black_box(&config)
        });
    });
    
    // Manual runtime check (not optimized)
    group.bench_function("runtime_check", |b| {
        let debug_enabled = false; // Simulating runtime flag
        b.iter(|| {
            if debug_enabled {
                println!("This should not appear in benchmarks");
            }
            black_box(debug_enabled)
        });
    });
    
    group.finish();
}

// ============================================================================
// PRIMAL SDK PERFORMANCE BENCHMARKS
// ============================================================================

fn bench_primal_connection_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("primal_connection");
    
    let primal_type = CanonicalPrimalType::Security;
    let endpoint = "https://security.primal.example.com:8443";
    
    // Zero-allocation connection (stack-based metadata)
    group.bench_function("stack_allocated", |b| {
        b.iter(|| {
            let mut conn = PrimalConnection::new(
                black_box(Uuid::new_v4()),
                black_box(primal_type.clone()),
                black_box(endpoint),
            );
            
            // Add metadata without heap allocation
            let _ = conn.add_metadata("type", "security");
            let _ = conn.add_metadata("version", "1.0");
            let _ = conn.add_metadata("priority", "high");
            
            conn
        });
    });
    
    // Heap-allocated connection (traditional approach)
    group.bench_function("heap_allocated", |b| {
        b.iter(|| {
            let mut metadata = HashMap::new();
            metadata.insert("type".to_string(), "security".to_string());
            metadata.insert("version".to_string(), "1.0".to_string());
            metadata.insert("priority".to_string(), "high".to_string());
            
            (
                black_box(Uuid::new_v4()),
                black_box(primal_type.clone()),
                black_box(endpoint.to_string()),
                metadata,
            )
        });
    });
    
    group.finish();
}

fn bench_connection_pool_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_pool");
    
    // Test different pool sizes
    for pool_size in [4, 16, 64].iter() {
        group.bench_with_input(
            BenchmarkId::new("optimized_pool", pool_size),
            pool_size,
            |b, &size| {
                b.iter(|| {
                    match size {
                        4 => {
                            let mut pool: OptimizedPrimalPool<4> = OptimizedPrimalPool::new();
                            for i in 0..4 {
                                let conn = PrimalConnection::new(
                                    Uuid::new_v4(),
                                    CanonicalPrimalType::Gaming,
                                    &format!("endpoint_{}", i),
                                );
                                let _ = pool.add_connection(black_box(conn));
                            }
                            black_box(pool.stats())
                        },
                        16 => {
                            let mut pool: OptimizedPrimalPool<16> = OptimizedPrimalPool::new();
                            for i in 0..16 {
                                let conn = PrimalConnection::new(
                                    Uuid::new_v4(),
                                    CanonicalPrimalType::Gaming,
                                    &format!("endpoint_{}", i),
                                );
                                let _ = pool.add_connection(black_box(conn));
                            }
                            black_box(pool.stats())
                        },
                        64 => {
                            let mut pool: OptimizedPrimalPool<64> = OptimizedPrimalPool::new();
                            for i in 0..64 {
                                let conn = PrimalConnection::new(
                                    Uuid::new_v4(),
                                    CanonicalPrimalType::Gaming,
                                    &format!("endpoint_{}", i),
                                );
                                let _ = pool.add_connection(black_box(conn));
                            }
                            black_box(pool.stats())
                        },
                        _ => unreachable!(),
                    }
                });
            },
        );
        
        // Compare with heap-based pool
        group.bench_with_input(
            BenchmarkId::new("heap_pool", pool_size),
            pool_size,
            |b, &size| {
                b.iter(|| {
                    let mut pool = Vec::with_capacity(size);
                    for i in 0..size {
                        let conn = (
                            Uuid::new_v4(),
                            CanonicalPrimalType::Gaming,
                            format!("endpoint_{}", i),
                        );
                        pool.push(black_box(conn));
                    }
                    black_box(pool.len())
                });
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// SDK TYPE ALIAS PERFORMANCE
// ============================================================================

fn bench_sdk_type_aliases(c: &mut Criterion) {
    let mut group = c.benchmark_group("sdk_type_aliases");
    
    // Different SDK configurations should have identical performance
    // due to const generics being zero-cost
    
    group.bench_function("lightweight_sdk_stats", |b| {
        let pool: OptimizedPrimalPool<4> = OptimizedPrimalPool::new();
        b.iter(|| {
            black_box(pool.stats())
        });
    });
    
    group.bench_function("standard_sdk_stats", |b| {
        let pool: OptimizedPrimalPool<16> = OptimizedPrimalPool::new();
        b.iter(|| {
            black_box(pool.stats())
        });
    });
    
    group.bench_function("high_performance_sdk_stats", |b| {
        let pool: OptimizedPrimalPool<64> = OptimizedPrimalPool::new();
        b.iter(|| {
            black_box(pool.stats())
        });
    });
    
    group.finish();
}

// ============================================================================
// BENCHMARK GROUPS
// ============================================================================

criterion_group!(
    zero_cost_benches,
    bench_const_buffer_vs_vec,
    bench_stack_vs_heap_collections,
    bench_compile_time_hash,
    bench_performance_config_optimization,
    bench_primal_connection_creation,
    bench_connection_pool_performance,
    bench_sdk_type_aliases,
);

criterion_main!(zero_cost_benches); 