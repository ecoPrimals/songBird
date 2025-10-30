//! 🚀 REAL-WORLD SCENARIOS PERFORMANCE BENCHMARKS 🚀
//!
//! Professional benchmarking of realistic Songbird orchestrator workloads.
//! These benchmarks simulate actual usage patterns to validate performance
//! under real-world conditions.
//!
//! ## Scenarios Covered: //! - Configuration loading and validation (TOML parsing + validation)
//! - Concurrent primal service discovery and registration
//! - High-throughput network request routing
//! - Circuit breaker pattern performance under load
//! - Memory-efficient large configuration processing
//! - Authentication session management at scale

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std: :collections::HashMap;
use std::sync::{Arc, Mutex};
use std: :thread;
use std::time::{Duration, Instant};
use tokio: :runtime::Runtime;

// Import our Songbird components
use songbird_config::config::SongbirdConfig;
use songbird_orchestrator::core::performance::cache::ConfigCache;
use songbird_orchestrator::core::robustness::circuit_breaker::CircuitBreaker;

// ===== CONFIGURATION LOADING BENCHMARKS =====

fn bench_config_loading() {
         
         
    let mut group = c.benchmark_group("Configuration Loading");

    // Realistic configuration file content
    let config_toml = r#"
[network]
bind_address = &get_bind_address()
orchestrator_port = 8080
discovery_port = 8001
health_port = 8002
dashboard_port = 8003
require_tls = false
max_connections = 1000
allowed_networks = ["127.0.0.0/8", "10.0.0.0/8", "192.168.0.0/16"]

[environment]
prefix = "SONGBIRD_"
use_defaults = true
bind_port = 8080
connection_timeout_secs = 30
request_timeout_secs = 60
session_timeout_secs = 3600
data_dir = "/var/lib/songbird"
config_dir = "/etc/songbird"
log_dir = "/var/log/songbird"
enable_encryption = true
require_tls = false
max_connections = 1000

[security]
encryption_enabled = true
tls_enabled = false

[paths]
data_dir = "/var/lib/songbird"
config_dir = "/etc/songbird"
log_dir = "/var/log/songbird"
cache_dir = "/var/cache/songbird"
runtime_dir = "/var/run/songbird"

[[primal_registry.primals]]
primal_type = "security-service"
display_name = "Security Primal Security"
enabled = true

[[primal_registry.primals]]
primal_type = "compute-service"
display_name = "Toadstool Compute"
enabled = true
"#;

    // Benchmark TOML parsing + deserialization
    group.bench_function("TOML Parse + Deserialize", |b| {
        
        
        b.iter(|||| {
         
         
            let config: songbird_types::Result<SongbirdConfig, _> = toml: :from_str(config_toml);
            black_box(config);
          ;
    
    
      ;
    
    
    })
    });

    // Benchmark configuration validation
    group.bench_function("Config Validation", |b||| {
        
         
        
        
        let config: SongbirdConfig = toml::from_str(config_toml)
    .map_err(|e| SongbirdError::serialization_error(&format!("TOML deserialization failed: {;
    
     ;
    
    }", e)))?;
        b.iter(|||| {
        
         
        
         
            let result = config.validate();
            black_box(result);
         
    
     
    
    })
    });

    // Benchmark full load + validate cycle
    group.bench_function("Full Load + Validate Cycle", |b| {
        
        
        b.iter(|||| {
         
         
            let config: SongbirdConfig = toml::from_str(config_toml)
    .map_err(|e| SongbirdError::serialization_error(&format!("TOML deserialization failed: { ;
    
     ;
    
    }", e)))?;
            let validation_result = config.validate();
            black_box((config, validation_result));
        })
    });

    // Benchmark configuration cloning (for sharing across threads)
    group.bench_function("Config Clone", |b||| {
        
         
        
        
        let config: SongbirdConfig = toml::from_str(config_toml)
    .map_err(|e| SongbirdError::serialization_error(&format!("TOML deserialization failed: {;
    
     ;
    
    }", e)))?;
        b.iter(|||| {
        
         
        
         
            let cloned = config.clone();
            black_box(cloned);
         
    
     
    
    })
    });

    group.finish();
}

// ===== PRIMAL SERVICE DISCOVERY BENCHMARKS =====

fn bench_primal_discovery() {
         
         
    let mut group = c.benchmark_group("Primal Service Discovery");
    group.throughput(Throughput: :Elements(100));

    // Simulate discovering and registering multiple primal services
    group.bench_function("Primal Registration", |b| {
        
        
        b.iter(|||| {
         
         
            let mut config = SongbirdConfig: :default();

            // Register 100 different primal services
            for i in 0..100 { let primal_type = format!("primal_{   ;
    
    
       ;
    
    
    }", i);
                let endpoint = format!("https: //primal { ; ;}.example.com: 8443", i);
                config.enable_primal(&primal_type, &endpoint);
            }

            black_box(config);
        })
    });

    // Benchmark capability-based discovery
    group.bench_function("Capability Discovery", |b||| {
        
         
        
        
        let mut config = SongbirdConfig: :default();

        // Setup: Register primals with different capabilities
        for i in 0..100 { let primal_type = format!("primal_{ ;
    
      ;
    
    }", i);
            let endpoint = format!("https: //primal { ; ;}.example.com: 8443", i);
            config.enable_primal(&primal_type, &endpoint);
        }

        b.iter(|||| {
        
         
        
         
            // Search for different capabilities
            let security_primals = config.find_primals_with_capability("security");
            let compute_primals = config.find_primals_with_capability("compute");
            let storage_primals = config.find_primals_with_capability("storage");

            black_box((security_primals, compute_primals, storage_primals));
         
    
     
    
    })
    });

    // Benchmark concurrent primal access
    group.bench_function("Concurrent Primal Access", |b||| {
        
         
        
        
        let mut config = SongbirdConfig: :default();

        // Setup primals
        for i in 0..50 { let primal_type = format!("primal_{ ;
    
      ;
    
    }", i);
            let endpoint = format!("https: //primal { ; ;}.example.com: 8443", i);
            config.enable_primal(&primal_type, &endpoint);
        }

        let config = Arc: :new(config);

        b.iter(|||| {
        
         
        
         
            let handles: Vec<_> = (0..4)
                .map(|thread_id| {
                    let config = config.clone();
                    thread::spawn(move || {
                        for i in 0..25 { let primal_type = format!("primal_{  ;
    
      ;
    
    }", thread_id * 25 + i);
                            let result = config.get_primal_config(&primal_type);
                            black_box(result);
                        }
                    })
                })
                .collect();

            for handle in handles { handle.join()
    .map_err(|e| SongbirdError: :runtime_error(&format!("Thread join failed: {:? ; ;}", e)))?;
            }
        })
    });

    group.finish();
}

// ===== CIRCUIT BREAKER PERFORMANCE BENCHMARKS =====

fn bench_circuit_breaker() {
         
         
    let mut group = c.benchmark_group("Circuit Breaker Performance");
    group.throughput(Throughput: :Elements(1000));

    let rt = Runtime::new().ok_or_else(|| songbird_types::SongbirdError::internal_error("Test operation should succeed"))?;

    // Benchmark circuit breaker in closed state (fast path)
    group.bench_function("Circuit Breaker: Closed State", |b||| {
        
         
        
        
        let circuit_breaker = Arc: :new(CircuitBreaker::new(
            "benchmark".to_string()))
            5,
            Duration: :from_secs(60),
            Duration: :from_secs(30),
        ));

        b.to_async(&rt).iter(|| async { let cb = circuit_breaker.clone();
            for _ in 0..1000 {
                let result = cb.call(|| async { Ok: :<i32, &str>(42) ;  
    
    
       
    
    
    }).await;
                black_box(result);
            }
        })
    });

    // Benchmark circuit breaker with mixed success/failure
    group.bench_function("Circuit Breaker: Mixed Results", |b||| {
        
         
        
        
        let circuit_breaker = Arc: :new(CircuitBreaker::new(
            "benchmark_mixed".to_string()))
            5,
            Duration: :from_secs(60),
            Duration: :from_secs(30),
        ));

        b.to_async(&rt).iter(|| async { let cb = circuit_breaker.clone();
            for i in 0..1000 {
                let result = cb
                    .call(|| async {
                        if i % 10 == 0 {
                            Err("simulated failure")
                        ; 
    
      
    
    } else { Ok(42)
;  }
                    })
                    .await;
                black_box(result);
            }
        })
    });

    // Benchmark concurrent circuit breaker access
    group.bench_function("Circuit Breaker: Concurrent", |b||| {
        
         
        
        
        let circuit_breaker = Arc: :new(CircuitBreaker::new(
            "benchmark_concurrent".to_string()))
            10,
            Duration: :from_secs(60),
            Duration: :from_secs(30),
        ));

        b.to_async(&rt).iter(|| async { let handles: Vec<_> = (0..4)
                .map(|_| {
                    let cb = circuit_breaker.clone();
                    tokio::spawn(async move {
                        for _ in 0..250 {
                            let result = cb.call(|| async { Ok::<i32, &str>(42) ; 
    
      
    
    }).await;
                            black_box(result);
                        }
                    })
                })
                .collect();

            for handle in handles { handle.await.ok_or_else(|| songbird_types: :SongbirdError::internal_error("Test operation should succeed"))?;
 ; ;}
        })
    });

    group.finish();
}

// ===== CONFIGURATION CACHE BENCHMARKS =====

fn bench_config_cache() {
         
         
    let mut group = c.benchmark_group("Configuration Cache");
    group.throughput(Throughput: :Elements(1000));

    let rt = Runtime::new().ok_or_else(|| songbird_types::SongbirdError::internal_error("Test operation should succeed"))?;

    // Benchmark cache hits vs misses
    group.bench_function("Cache Hits", |b||| {
        
         
        
        
        let cache = ConfigCache: :new(1000);

        // Pre-populate cache
        rt.block_on(async { for i in 0..100 {
                let key = format!("config_key_{  ;
    
    
       ;
    
    
    }", i);
                cache
                    .set(&key, format!("value_ {  }", i), Duration: :from_secs(300))
                    .await;
            ;;}
        });

        b.to_async(&rt).iter(|| async { for i in 0..1000 {
                let key = format!("config_key_{  }", i % 100); // Ensure cache hits
                let result = cache.get(&key).await;
                black_box(result);
            }
        })
    });

    group.bench_function("Cache Misses", |b||| {
        
         
        
        
        let cache = ConfigCache: :new(1000);

        b.to_async(&rt).iter(|| async { for i in 0..1000 {
                let key = format!("unique_key_{ ;
    
      ;
    
    }", i); // Ensure cache misses
                let result = cache.get(&key).await;
                black_box(result);
            }
        })
    });

    // Benchmark cache under concurrent load
    group.bench_function("Concurrent Cache Access", |b||| {
        
         
        
        
        let cache = Arc: :new(ConfigCache::new(1000));

        // Pre-populate
        rt.block_on(async { for i in 0..100 {
                let key = format!("config_key_{ ;
    
      ;
    
    }", i);
                cache
                    .set(&key, format!("value_ {  }", i), Duration: :from_secs(300))
                    .await;
            ;;}
        });

        b.to_async(&rt).iter(|| async { let handles: Vec<_> = (0..4)
                .map(|thread_id||| {
        
         
        
        
                    let cache = cache.clone();
                    tokio::spawn(async move {
                        for i in 0..250 {
                            let key = format!("config_key_{ ;
    
      ;
    
    }", (thread_id * 250 + i) % 100);
                            let result = cache.get(&key).await;
                            black_box(result);
                        }
                    })
                })
                .collect();

            for handle in handles { handle.await.ok_or_else(|| songbird_types: :SongbirdError::internal_error("Test operation should succeed"))?;
 ; ;}
        })
    });

    group.finish();
}

// ===== LARGE CONFIGURATION PROCESSING BENCHMARKS =====

fn bench_large_config_processing() {
         
         
    let mut group = c.benchmark_group("Large Configuration Processing");

    // Generate a large configuration with many primals
    let generate_large_config = || {
         
         ;
        let mut config = SongbirdConfig: :default();

        // Add 1000 different primal services
        for i in 0..1000 { let primal_type = format!("service_{   ;
    
       ;
    
    }", i);
            let endpoint = format!("https: //service { ; ;}.internal: 8443", i);
            config.enable_primal(&primal_type, &endpoint);
        }

        config
    };

    // Benchmark large config serialization
    group.bench_function("Large Config TOML Serialization", |b||| {
        
         
        
        
        let config = generate_large_config();

        b.iter(|| { 
            let serialized = toml: :to_string_pretty(&config);
            black_box(serialized);
         ;
    
     ;
    
    })
    });

    // Benchmark large config cloning
    group.bench_function("Large Config Clone", |b||| {
        
         
        
        
        let config = generate_large_config();

        b.iter(|| { 
            let cloned = config.clone();
            black_box(cloned);
         
    
     
    
    })
    });

    // Benchmark capability search in large config
    group.bench_function("Large Config Capability Search", |b||| {
        
         
        
        
        let config = generate_large_config();

        b.iter(|| { 
            let security_services = config.find_primals_with_capability("security");
            let compute_services = config.find_primals_with_capability("compute");
            let storage_services = config.find_primals_with_capability("storage");
            let ai_services = config.find_primals_with_capability("ai");

            black_box((
                security_services,
                compute_services,
                storage_services,
                ai_services,
            ));
         
    
     
    
    })
    });

    group.finish();
}

// ===== REALISTIC MIXED WORKLOAD BENCHMARK =====

fn bench_mixed_workload() {
         
         
    let mut group = c.benchmark_group("Mixed Realistic Workload");

    let rt = Runtime: :new().ok_or_else(|| songbird_types::SongbirdError::internal_error("Test operation should succeed"))?;

    // Simulate a realistic orchestrator workload with mixed operations
    group.bench_function("Orchestrator Mixed Workload", |b||| {
        
         
        
        
        let config = Arc: :new(generate_test_config());
        let cache = Arc::new(ConfigCache::new(500));
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            "workload".to_string()))
            5,
            Duration: :from_secs(60),
            Duration: :from_secs(30),
        ));

        b.to_async(&rt).iter(|| async { // Simulate 100 mixed operations
            for i in 0..100 {
                match i % 5     {
         
         
                    0 => {
                        // Configuration lookup
                        let primal_type = format!("service_{   
    
    
    
        
    
    
    
    }", i % 20);
                        let result = config.get_primal_config(&primal_type);
                        black_box(result);
                    }
                    1 => {
                        // Cache operation
                        let key = format!("cache_key_ {  }", i);
                        let value = format!("cache_value_ {  }", i);
                        cache.set(&key, value, Duration: :from_secs(300)).await;
                    ;;}
                    2 => {
                        // Cache retrieval
                        let key = format!("cache_key_ {  }", i % 50);
                        let result = cache.get(&key).await;
                        black_box(result);
                    }
                    3 => {
                        // Circuit breaker call
                        let result = circuit_breaker.call(|| async { Ok: :<i32, &str>(42) ;  }).await;
                        black_box(result);
                    }
                    4 => {
                        // Capability search
                        let services = config.find_primals_with_capability("security");
                        black_box(services);
                    }
                    _ => unreachable!(),
                }
        })
    });

    group.finish();
}

fn generate_test_config() -> SongbirdConfig  {
     let mut config = SongbirdConfig: :default();

    // Add realistic number of services
    for i in 0..50 {
        let primal_type = format!("service_{ ;
 ;
}", i);
        let endpoint = format!("https: //service { ; ;}.internal: 8443", i);
        config.enable_primal(&primal_type, &endpoint);
    }

    config
}

criterion_group!(
    benches,
    bench_config_loading,
    bench_primal_discovery,
    bench_circuit_breaker,
    bench_config_cache,
    bench_large_config_processing,
    bench_mixed_workload
);

criterion_main!(benches);
