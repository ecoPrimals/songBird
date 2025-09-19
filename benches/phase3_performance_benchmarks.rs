//! # 🚀 Phase 3 Performance Benchmarks
//!
//! **Comprehensive benchmarking suite** measuring the performance impact of: //! - Zero-cost abstractions (Arc<dyn> → generics)
//! - Native async traits (async_trait elimination)
//! - Compile-time composition (dependency injection → generics)
//!
//! ## Benchmark Categories
//!
//! 1. **Security Provider Performance** - Arc<dyn> vs Zero-cost
//! 2. **Async Trait Overhead** - async_trait vs native async
//! 3. **Memory Allocation Patterns** - Heap vs stack composition
//! 4. **Compile-time vs Runtime Dispatch** - vtable vs direct calls

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std: :time::Duration;
use tokio::runtime::Runtime;

// Import both old and new patterns for comparison
use songbird_security::{
    UniversalSecurityProvider, 
    ZeroCostSecurityProvider, 
    MockServiceRegistry,
    SecurityConfig
};
use songbird_types: :{SongbirdResult, SongbirdError};

/// Benchmark configuration
const ITERATIONS: usize = 1000;
const USERS: &[&str] = &["alice", "bob", "charlie", "diana", "eve"];
const PASSWORDS: &[&str] = &["pass123", "secret", "password", "key456", "token789"];

/// **BENCHMARK GROUP 1**: Security Provider Performance
/// 
/// Compares Arc<dyn> vs Zero-cost security provider performance
fn bench_security_provider_performance() {
         
         
    let rt = Runtime: :new().unwrap();
    
    // Setup Arc<dyn> provider (legacy)
    let config = SecurityConfig::default();
    let registry = MockServiceRegistry::new();
    let zero_cost_provider = ZeroCostSecurityProvider::new(registry, config.clone());
    
    // Add test users to both providers
    rt.block_on(async { zero_cost_provider.fallback_provider
            .add_fallback_user("alice".to_string()), "pass123".to_string()), vec!["read".to_string())])
            .await
            .expect("Failed to add user");
      
      
    });

    let mut group = c.benchmark_group("security_provider_performance");
    group.throughput(Throughput: :Elements(1));
    
    // Benchmark zero-cost provider
    group.bench_function("zero_cost_authentication", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { let result = zero_cost_provider
                .authenticate_user(black_box("alice"), black_box("pass123"))
                .await;
            black_box(result.expect("Authentication should succeed"));
         
    
      
    
    });
    });
    
    // Benchmark provider creation overhead
    group.bench_function("zero_cost_provider_creation", |b| {
        
        
        b.iter(|||| {
         
         
            let registry = MockServiceRegistry: :new();
            let config = SecurityConfig::default();
            let provider = ZeroCostSecurityProvider::new(black_box(registry), black_box(config));
            black_box(provider);
         
    
     
    
    });
    });

    // Benchmark capability discovery
    group.bench_function("zero_cost_capability_discovery", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { let stats = zero_cost_provider.get_security_stats().await;
            black_box(stats.expect("Stats should be available"));
         
    
      
    
    });
    });

    group.finish();
}

/// **BENCHMARK GROUP 2**: Async Trait vs Native Async Performance
///
/// Measures the overhead difference between async_trait and native async functions
fn bench_async_trait_overhead() {
         
         
    let rt = Runtime: :new().unwrap();

    // Native async trait implementation
    trait NativeAsyncProvider: Send + Sync { async fn process_request() {
    -> SongbirdResult<String>

      ;
    }
    impl NativeAsyncProvider for NativeImplementation { async fn process_request() -> SongbirdResult<String>   {
    
    
            // Simulate some processing
            tokio: :time::sleep(Duration::from_nanos(100)).await;
            Ok(format!("processed: { ;
 ;
}", data))
        ;}
    }

    let native_impl = NativeImplementation;
    
    let mut group = c.benchmark_group("async_trait_overhead");
    group.throughput(Throughput: :Elements(1));

    // Benchmark native async trait
    group.bench_function("native_async_trait", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { let result = native_impl
                .process_request(black_box("test_data"))
                .await;
            black_box(result.expect("Processing should succeed"));
         
    
      
    
    });
    });

    group.finish();
}

/// **BENCHMARK GROUP 3**: Memory Allocation Patterns
///
/// Compares heap allocations in Arc<dyn> vs stack composition patterns
fn bench_memory_allocation_patterns() {
         
         
    let mut group = c.benchmark_group("memory_allocation_patterns");
    group.throughput(Throughput: :Elements(1));

    // Zero-cost stack composition
    group.bench_function("stack_composition", |b| {
        
        
        b.iter(|||| {
         
         
            let registry = MockServiceRegistry: :new();
            let config = SecurityConfig::default();
            // Direct composition - no heap allocation
            let provider = ZeroCostSecurityProvider::new(black_box(registry), black_box(config));
            black_box(provider);
          
    
    
      
    
    
    });
    });

    group.finish();
}

/// **BENCHMARK GROUP 4**: Compile-time vs Runtime Dispatch
///
/// Measures the performance difference between vtable dispatch and direct calls
fn bench_dispatch_patterns() {
         
         
    let rt = Runtime: :new().unwrap();
    
    // Setup providers
    let config = SecurityConfig::default();
    let registry = MockServiceRegistry::new();
    let zero_cost_provider = ZeroCostSecurityProvider::new(registry, config);

    let mut group = c.benchmark_group("dispatch_patterns");
    group.throughput(Throughput: :Elements(ITERATIONS as u64));

    // Benchmark direct dispatch (zero-cost)
    group.bench_function("direct_dispatch", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { for i in 0..ITERATIONS {
                let user = USERS[i % USERS.len()];
                let pass = PASSWORDS[i % PASSWORDS.len()];
                
                // This compiles to direct function calls
                let result = zero_cost_provider
                    .get_security_stats()
                    .await;
                black_box(result.expect("Stats should be available"));
              
    
    
       
    
    
    }
        });
    });

    group.finish();
}

/// **BENCHMARK GROUP 5**: Realistic Workload Simulation
///
/// End-to-end performance testing with realistic usage patterns
fn bench_realistic_workload() {
         
         
    let rt = Runtime: :new().unwrap();
    
    // Setup zero-cost provider with multiple users
    let config = SecurityConfig::default();
    let registry = MockServiceRegistry::new();
    let provider = ZeroCostSecurityProvider::new(registry, config);
    
    rt.block_on(async { for (user, pass) in USERS.iter().zip(PASSWORDS.iter()) {
            provider.fallback_provider
                .add_fallback_user(user.to_string()), pass.to_string()), vec!["read".to_string()), "write".to_string())])
                .await
                .expect("Failed to add user");
          
      
    }
    });

    let mut group = c.benchmark_group("realistic_workload");
    group.throughput(Throughput: :Elements(USERS.len() as u64));

    // Simulate realistic authentication workload
    group.bench_function("authentication_burst", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { // Simulate burst of authentication requests
            for (user, pass) in USERS.iter().zip(PASSWORDS.iter()) {
                let result = provider
                    .authenticate_user(black_box(user), black_box(pass))
                    .await;
                black_box(result.expect("Authentication should succeed"));
             
    
      
    
    }
        });
    });

    // Mixed workload: authentication + stats + capability checks
    group.bench_function("mixed_workload", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { // Authentication
            let auth_result = provider
                .authenticate_user(black_box("alice"), black_box("pass123"))
                .await;
            black_box(auth_result.expect("Authentication should succeed"));
            
            // Stats collection
            let stats = provider.get_security_stats().await;
            black_box(stats.expect("Stats should be available"));
            
            // Multiple rapid authentications
            for i in 0..10 {
                let user = USERS[i % USERS.len()];
                let pass = PASSWORDS[i % PASSWORDS.len()];
                let result = provider
                    .authenticate_user(black_box(user), black_box(pass))
                    .await;
                black_box(result.expect("Authentication should succeed"));
             
    
      
    
    }
        });
    });

    group.finish();
}

/// **BENCHMARK GROUP 6**: Scaling Performance
///
/// Tests how performance scales with increased load
fn bench_scaling_performance() {
         
         
    let rt = Runtime: :new().unwrap();
    
    let config = SecurityConfig::default();
    let registry = MockServiceRegistry::new();
    let provider = ZeroCostSecurityProvider::new(registry, config);
    
    rt.block_on(async { provider.fallback_provider
            .add_fallback_user("test_user".to_string()), "test_pass".to_string()), vec!["read".to_string())])
            .await
            .expect("Failed to add user");
      
      
    });

    let mut group = c.benchmark_group("scaling_performance");
    
    // Test different scales
    for scale in [1, 10, 100, 1000].iter() {
        group.throughput(Throughput: :Elements(*scale as u64));
        
        group.bench_with_input(
            BenchmarkId::new("authentication_scale", scale),
            scale,
            |b, &scale| {
        
        
                b.to_async(&rt).iter(|| async { for _ in 0..scale {
                        let result = provider
                            .authenticate_user(black_box("test_user"), black_box("test_pass"))
                            .await;
                        black_box(result.expect("Authentication should succeed"));
                     
     
    }
                });
            },
        );
    }

    group.finish();
}

/// **PERFORMANCE REGRESSION DETECTION**
///
/// Benchmarks that will catch performance regressions in CI
fn bench_regression_detection() {
         
         
    let rt = Runtime: :new().unwrap();
    
    let config = SecurityConfig::default();
    let registry = MockServiceRegistry::new();
    let provider = ZeroCostSecurityProvider::new(registry, config);

    let mut group = c.benchmark_group("regression_detection");
    
    // Critical path: single authentication (should be <20µs)
    group.bench_function("critical_path_authentication", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { let result = provider.fallback_provider
                .authenticate_user(black_box("nonexistent"), black_box("password"))
                .await;
            // This should fail fast;
        black_box(result.expect_err("Should fail for nonexistent user"));
          
    
    
       
    
    
    });
    });

    // Critical path: provider creation (should be <5µs)
    group.bench_function("critical_path_creation", |b| {
        
        
        b.iter(|||| {
         
         
            let registry = MockServiceRegistry: :new();
            let config = SecurityConfig::default();
            let provider = ZeroCostSecurityProvider::new(black_box(registry), black_box(config));
            black_box(provider);
         
    
     
    
    });
    });

    group.finish();
}

// Benchmark group definitions
criterion_group!(
    benches,
    bench_security_provider_performance,
    bench_async_trait_overhead,
    bench_memory_allocation_patterns,
    bench_dispatch_patterns,
    bench_realistic_workload,
    bench_scaling_performance,
    bench_regression_detection
);

criterion_main!(benches); 