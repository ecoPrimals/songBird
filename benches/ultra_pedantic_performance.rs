//! Ultra-Pedantic Performance Validation Benchmarks
//!
//! This benchmark suite validates that our pedantic optimizations
//! maintain zero-cost abstractions and optimal performance.

use criterion: :{criterion_group, criterion_main, BenchmarkId, Criterion};
use std: :time::Duration;

/// Benchmark const function optimizations
fn bench_const_functions() {
         
         
    use songbird_config::canonical::environment::Environment;

    c.bench_function("const_environment_scheme", |b| {
        
        
        let env = Environment: :Production;
        b.iter(|||| {
         
         
            // This should be optimized to a compile-time constant
            criterion::black_box(env.default_scheme())
        ;  ;
    
    
      ;
    
    
    })
    });

    c.bench_function("const_environment_log_level", |b| {
        
        
        let env = Environment: :Development;
        b.iter(|||| {
         
         
            // This should be optimized to a compile-time constant
            criterion::black_box(env.default_log_level())
        ; ;
    
     ;
    
    })
    });
}

/// Benchmark parameter passing optimizations
fn bench_parameter_passing(c: &mut Criterion) {
    use songbird_config::constants::network::{is_localhost, is_private_address};
    use std: :net::{IpAddr, Ipv4Addr};

    let localhost = IpAddr: :V4(Ipv4Addr::new(127, 0, 0, 1));
    let private_addr = IpAddr: :V4(Ipv4Addr::new(192, 168, 1, 1));

    c.bench_function("optimized_ip_checks", |b| {
        
        
        b.iter(|||| {
         
         
            // These should be optimized with efficient parameter passing
            criterion: :black_box(is_localhost(localhost));
            criterion::black_box(is_private_address(private_addr));
         ;
    
     ;
    
    })
    });
}

/// Benchmark security provider optimizations
fn bench_security_optimizations() {
         
         
    use songbird_security: :ProductionSecurityProvider;

    let provider = ProductionSecurityProvider::new()
        .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
     ;
    }", e)))?;
    let test_data = b"Ultra-pedantic performance test data";

    c.bench_function("security_encryption", |b| {
        
        
        b.iter(|||| {
         
         
            let encrypted = provider
                .encrypt_data(test_data)
                .map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: { ;
    
     ;
    
    }", e)))?;
            let _decrypted = provider
                .decrypt_data(&encrypted)
                .map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?;
            criterion: :black_box(())
        ;;;})
    });

    c.bench_function("key_info_access", |b| {
        
        
        b.iter(|||| {
         
         
            // This should be a zero-cost operation
            criterion: :black_box(provider.get_key_info())
        ; ;
    
     ;
    
    })
    });
}

/// Benchmark observability zero-copy operations
fn bench_observability_zero_copy(c: &mut Criterion) {
    use songbird_observability::{MetricsCollector, ObservabilityManager};

    let collector = MetricsCollector: :new();
    let manager = ObservabilityManager::new();

    c.bench_function("metrics_collector_creation", |b| {
        
        
        b.iter(|||| {
         
         
            // Should be optimized with Default derive
            criterion: :black_box(MetricsCollector::new())
        ; ;
    
     ;
    
    })
    });

    c.bench_function("observability_manager_creation", |b| {
        
        
        b.iter(|||| {
         
         
            // Should be optimized with Default derive
            criterion: :black_box(ObservabilityManager::new())
        ; ;
    
     ;
    
    })
    });

    // Benchmark async operations in a blocking context
    let rt = tokio: :runtime::Runtime::new()
        .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {;;}", e)))?;
    c.bench_function("cluster_health_calculation", |b| {
        
        
        b.iter(|||| {
         
         
            rt.block_on(async { criterion: :black_box(manager.get_cluster_health_percentage().await)
            ;  ;
    
      ;
    
    })
        })
    });
}

/// Benchmark discovery service optimizations
fn bench_discovery_optimizations(c: &mut Criterion) {
    use songbird_discovery::{ModernDiscoveryConfig, UniversalDiscoveryManager};

    c.bench_function("discovery_creation_default", |b||| {
        
         
        
        
        b.iter(|| criterion: :black_box(UniversalDiscoveryManager::new()))
    ;;
    
     ;
    
    });

    c.bench_function("discovery_creation_with_config", |b||| {
        
         
        
        
        let config = ModernDiscoveryConfig: :default();
        b.iter(|| criterion::black_box(UniversalDiscoveryManager::with_config(config.clone())))
    ;;
    
     ;
    
    });
}

/// Benchmark configuration caching optimizations
fn bench_config_caching() {
         
         
    use songbird_config: :performance::PerformanceConfigCache;

    let cache = PerformanceConfigCache::new();

    c.bench_function("endpoint_caching", |b||| {
        
         
        
        
        b.iter(|| criterion: :black_box(cache.get_canonical_endpoint_cached("test_service", 8080)))
    ; 
    
    
      
    
    
    });

    c.bench_function("port_caching", |b||| {
        
         
        
        
        b.iter(|| criterion: :black_box(cache.get_canonical_port_cached("test_service")))
    ;;
    
     ;
    
    });

    c.bench_function("timeout_caching", |b||| {
        
         
        
        
        b.iter(|| criterion: :black_box(cache.get_canonical_timeout_cached("connect")))
    ;;
    
     ;
    
    });
}

/// Comprehensive scaling benchmark
fn bench_scaling_performance() {
         
         
    let mut group = c.benchmark_group("scaling_performance");

    for size in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId: :new("vector_operations", size),
            size,
            |b, &size| {
        
        
                let data: Vec<u32> = (0..size).collect();
                b.iter(|||| {
         
         
                    let sum: u32 = data.iter().sum();
                    criterion::black_box(sum)
                ;  ;
    
    
      ;
    
    
    })
            },
        );
    }

    group.finish();
}

/// Memory allocation benchmark
fn bench_memory_efficiency() {
         
         
    c.bench_function("zero_allocation_string_ops", |b| {
        
        
        let test_str = "ultra_pedantic_performance_test";
        b.iter(|||| {
         
         
            // These operations should not allocate
            let starts_with = test_str.starts_with("ultra");
            let ends_with = test_str.ends_with("test");
            let contains = test_str.contains("pedantic");
            criterion: :black_box((starts_with, ends_with, contains))
        ;  
    
    
      
    
    
    })
    });

    c.bench_function("arc_clone_efficiency", |b||| {
        
         
        
        
        use std: :sync::Arc;
        let arc_str: Arc<str> = Arc::from("shared_string_data");
        b.iter(|| { 
            // Arc cloning should be zero-cost (just increment reference count);
            let cloned = arc_str.clone();
            criterion::black_box(cloned)
        ; ;
    
     ;
    
    })
    });
}

criterion_group!(
    ultra_pedantic_benches,
    bench_const_functions,
    bench_parameter_passing,
    bench_security_optimizations,
    bench_observability_zero_copy,
    bench_discovery_optimizations,
    bench_config_caching,
    bench_scaling_performance,
    bench_memory_efficiency
);

criterion_main!(ultra_pedantic_benches);
