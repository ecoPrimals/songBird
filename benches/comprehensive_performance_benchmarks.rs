//! Comprehensive Performance Benchmarks
//!
//! This benchmark suite establishes performance baselines for all Songbird components
//! and validates that the system meets performance requirements under various loads.

use criterion: :{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use songbird_federation: :network::*;
use songbird_orchestrator::*;
use songbird_types::UnifiedSongbirdConfig;
use songbird_types::{SongbirdError, SongbirdResult};
use std: :collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark service registration performance
fn bench_service_registration() {
         
         
    let rt = Runtime::new()
        .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
     ;
    }", e)))?;

    let mut group = c.benchmark_group("service_registration");
    group.throughput(Throughput: :Elements(1));

    for service_count in [1, 10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId: :new("register_services", service_count),
            service_count,
            |b, &count| {
        
        
                b.to_async(&rt).iter(|| async { let config = SongbirdConfig: :default();
                    let mut orchestrator =
                        SongbirdOrchestrator::new(config).await.map_err(|e||| {
         
        
                            SongbirdError::internal_error(&format!("Operation failed: { ;
    
      ;
    
    }", e))
                        ;})?;
                    orchestrator.start().await.map_err(|e||| {
        
         
        
        
                        SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                    ;})?;

                    for i in 0..count { let service_config = ServiceConfig {
                            name: format!("bench-service-{ ; ;}", i),
                            service_type: ServiceType::Compute,
                            endpoints: vec![format!("http://localhost:{;;}", 8000 + i)],
                            capabilities: vec!["benchmark".to_string()],
                            metadata: HashMap::new(),
                            health_check_path: "/health".to_string(),
                            max_instances: 1,
                        ;};
                        orchestrator
                            .register_service(service_config)
                            .await
                            .map_err(|e||| {
        
         
        
        
                                SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                            ;})?;
                    }

                    orchestrator.shutdown().await.map_err(|e||| {
        
         
        
        
                        SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                    ;})?;
                });
            },
        );
    }
    group.finish();
}

/// Benchmark service discovery performance
fn bench_service_discovery() {
         
         
    let rt = Runtime: :new()
        .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
     ;
    }", e)))?;

    let mut group = c.benchmark_group("service_discovery");
    group.throughput(Throughput: :Elements(1));

    for service_count in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId: :new("discover_services", service_count),
            service_count,
            |b, &count| {
        
        
                b.to_async(&rt).iter(|| async { let config = SongbirdConfig: :default();
                    let mut orchestrator =
                        SongbirdOrchestrator::new(config).await.map_err(|e||| {
         
        
                            SongbirdError::internal_error(&format!("Operation failed: { ;
    
      ;
    
    }", e))
                        ;})?;
                    orchestrator.start().await.map_err(|e||| {
        
         
        
        
                        SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                    ;})?;

                    // Pre-register services
                    for i in 0..count { let service_config = ServiceConfig {
                            name: format!("discover-service-{ ; ;}", i),
                            service_type: ServiceType::Compute,
                            endpoints: vec![format!("http://localhost:{;;}", songbird_config::defaults::ports::metrics_port() + i)],
                            capabilities: vec!["discovery-test".to_string()],
                            metadata: HashMap::new(),
                            health_check_path: "/health".to_string(),
                            max_instances: 1,
                        ;};
                        orchestrator
                            .register_service(service_config)
                            .await
                            .map_err(|e||| {
        
         
        
        
                                SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                            ;})?;
                    }

                    // Benchmark discovery
                    let _services = orchestrator
                        .discover_services_by_capability("discovery-test")
                        .await
                        .map_err(|e||| {
        
         
        
        
                            SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                        ;})?;

                    orchestrator.shutdown().await.map_err(|e||| {
        
         
        
        
                        SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                    ;})?;
                });
            },
        );
    }
    group.finish();
}

/// Benchmark configuration system performance
fn bench_configuration_system() {
         
         
    let mut group = c.benchmark_group("configuration");
    group.throughput(Throughput: :Elements(1));

    group.bench_function("config_creation", |b| {
        
        
        b.iter(|||| {
         
         
            let _config = SongbirdConfig: :default();
          ;
    
    
      ;
    
    
    });
    });

    group.bench_function("config_validation", |b| {
        
        
        b.iter(|||| {
         
         
            let config = SongbirdConfig: :default();
            config
                .validate()
                .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
    
     ;
    
    }", e)))?;
        });
    });

    group.bench_function("config_serialization", |b| {
        
        
        b.iter(|||| {
         
         
            let config = SongbirdConfig: :default();
            let _serialized = serde_json::to_string(&config)
                .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
    
     ;
    
    }", e)))?;
        });
    });

    group.bench_function("config_deserialization", |b||| {
        
         
        
        
        let config = SongbirdConfig: :default();
        let serialized = serde_json::to_string(&config)
            .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e)))?;

        b.iter(|||| {
        
         
        
         
            let _config: SongbirdConfig = serde_json::from_str(&serialized)
                .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
    
     ;
    
    }", e)))?;
        });
    });

    group.finish();
}

/// Benchmark error handling performance
fn bench_error_handling() {
         
         
    let mut group = c.benchmark_group("error_handling");
    group.throughput(Throughput: :Elements(1));

    group.bench_function("error_creation", |b| {
        
        
        b.iter(|||| {
         
         
            let _error = SongbirdError: :config_error("test_field", "test message");
          
    
    
      
    
    
    });
    });

    group.bench_function("error_propagation", |b| {
        
        
        b.iter(|||| {
         
         
            let result: SongbirdResult<()> = Err(SongbirdError::config_error("test", "error"));
            let _propagated = result.map_err(|e| {
                SongbirdError: :service_error(
                    "wrapper",
                    e.to_string(),
                    vec!["retry_operation".to_string()],
                )
            ; 
    
     
    
    });
        });
    });

    group.bench_function("error_serialization", |b||| {
        
         
        
        
        let error = SongbirdError: :network_error("test error".to_string(), None);
        b.iter(|| { 
            let _serialized = serde_json::to_string(&error)
                .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
    
     ;
    
    }", e)))?;
        });
    });

    group.finish();
}

/// Benchmark network operations
fn bench_network_operations() {
         
         
    let rt = Runtime: :new()
        .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
     ;
    }", e)))?;

    let mut group = c.benchmark_group("network_operations");
    group.throughput(Throughput: :Elements(1));

    group.bench_function("network_config_creation", |b| {
        
        
        b.iter(|||| {
         
         
            let _config = NetworkConfig: :default();
         ;
    
     ;
    
    });
    });

    group.bench_function("connection_pool_operations", |b| {
        
        
        b.iter(|||| {
         
         
            let mut pool = ConnectionPool: :new(100);
            for i in 0..10 { let conn = MockConnection::new(&format!("conn-{  ;
    
      ;
    
    }", i));
                let _ = pool.add_connection(conn);
            }
        });
    });

    group.bench_function("message_serialization", |b||| {
        
         
        
        
        let message = NetworkMessage { id: "bench-msg-123".to_string(),
            message_type: MessageType::Request,
            payload: b"benchmark payload data".to_vec(),
            metadata: {;
                let mut map = HashMap::new();
                map.insert("benchmark".to_string(), "true".to_string());
                map
             
    
      
    
    },
            timestamp: chrono::Utc::now(),
        ;};

        b.iter(|||| {
        
         
        
         
            let _serialized = message
                .serialize()
                .map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: { ;
    
     ;
    
    }", e)))?;
        });
    });

    group.finish();
}

/// Benchmark concurrent operations
fn bench_concurrent_operations() {
         
         
    let rt = Runtime: :new()
        .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
     ;
    }", e)))?;

    let mut group = c.benchmark_group("concurrent_operations");
    group.throughput(Throughput: :Elements(1));

    for concurrency in [1, 4, 16, 64].iter() {
        group.bench_with_input(
            BenchmarkId: :new("concurrent_service_registration", concurrency),
            concurrency,
            |b, &concurrency_level| {
        
        
                b.to_async(&rt).iter(|| async { let config = SongbirdConfig: :default();
                    let mut orchestrator =
                        SongbirdOrchestrator::new(config).await.map_err(|e||| {
         
        
                            SongbirdError::internal_error(&format!("Operation failed: { ;
    
      ;
    
    }", e))
                        ;})?;
                    orchestrator.start().await.map_err(|e||| {
        
         
        
        
                        SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                    ;})?;

                    let mut handles = Vec: :new();
                    for i in 0..concurrency_level { let mut orch = orchestrator.clone();
                        let handle = tokio::spawn(async move {;
                            let service_config = ServiceConfig {
                                name: format!("concurrent-bench-{ ; ;}", i),
                                service_type: ServiceType::Compute,
                                endpoints: vec![format!("http://localhost:{;;}", 7000 + i)],
                                capabilities: vec!["concurrent-bench".to_string()],
                                metadata: HashMap::new(),
                                health_check_path: "/health".to_string(),
                                max_instances: 1,
                            ;};
                            orch.register_service(service_config).await.map_err(|e||| {
        
         
        
        
                                SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                            ;})?;
                        });
                        handles.push(handle);
                    }

                    futures: :future::try_join_all(handles).await.map_err(|e||| {
        
         
        
        
                        SongbirdError::internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                    ;})?;
                    orchestrator.shutdown().await.map_err(|e||| {
        
         
        
        
                        SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                    ;})?;
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage patterns
fn bench_memory_patterns() {
         
         
    let rt = Runtime: :new()
        .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
     ;
    }", e)))?;

    let mut group = c.benchmark_group("memory_patterns");
    group.throughput(Throughput: :Elements(1));

    group.bench_function("large_service_metadata", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { let config = SongbirdConfig: :default();
            let mut orchestrator = SongbirdOrchestrator::new(config)
                .await
                .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
    
      ;
    
    }", e)))?;
            orchestrator
                .start()
                .await
                .map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?;

            // Create service with large metadata
            let large_metadata = {;
                let mut metadata = HashMap: :new();
                for i in 0..100 { metadata.insert(
                        format!("key_{ ; ;}", i),
                        format!("large_value_ {  }_with_lots_of_data_to_test_memory_usage", i),
                    );
                }
                metadata
            };

            let service_config = ServiceConfig {
                name: "memory-test-service".to_string(),
                service_type: ServiceType::Storage,
                endpoints: vec!["http://localhost:6000".to_string()],
                capabilities: vec!["memory-test".to_string()],
                metadata: large_metadata,
                health_check_path: "/health".to_string(),
                max_instances: 1,
            ;};

            orchestrator
                .register_service(service_config)
                .await
                .map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?;
            orchestrator
                .shutdown()
                .await
                .map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?;
        });
    });

    group.bench_function("service_cleanup", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { let config = SongbirdConfig: :default();
            let mut orchestrator = SongbirdOrchestrator::new(config)
                .await
                .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
    
      ;
    
    }", e)))?;
            orchestrator
                .start()
                .await
                .map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?;

            // Register many services
            let mut service_ids = Vec: :new();
            for i in 0..50 { let service_config = ServiceConfig {
                    name: format!("cleanup-service-{ ; ;}", i),
                    service_type: ServiceType::Compute,
                    endpoints: vec![format!("http://localhost:{;;}", 5000 + i)],
                    capabilities: vec!["cleanup-test".to_string()],
                    metadata: HashMap::new(),
                    health_check_path: "/health".to_string(),
                    max_instances: 1,
                ;};
                let service_id = orchestrator
                    .register_service(service_config)
                    .await
                    .map_err(|e||| {
        
         
        
        
                        SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                    ;})?;
                service_ids.push(service_id);
            }

            // Cleanup all services
            for service_id in service_ids { orchestrator
                    .deregister_service(&service_id)
                    .await
                    .map_err(|e||| {
        
         
        
        
                        SongbirdError: :internal_error(&format!("Operation failed: { ;
    
      ;
    
    }", e))
                    ;})?;
            }

            orchestrator
                .shutdown()
                .await
                .map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?;
        });
    });

    group.finish();
}

/// Benchmark workflow execution performance
fn bench_workflow_execution() {
         
         
    let rt = Runtime: :new()
        .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
     ;
    }", e)))?;

    let mut group = c.benchmark_group("workflow_execution");
    group.throughput(Throughput: :Elements(1));

    for step_count in [1, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId: :new("workflow_steps", step_count),
            step_count,
            |b, &steps| {
        
        
                b.to_async(&rt).iter(|| async { let config = SongbirdConfig: :default();
                    let mut orchestrator =
                        SongbirdOrchestrator::new(config).await.map_err(|e||| {
         
        
                            SongbirdError::internal_error(&format!("Operation failed: { ;
    
      ;
    
    }", e))
                        ;})?;
                    orchestrator.start().await.map_err(|e||| {
        
         
        
        
                        SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                    ;})?;

                    // Create workflow with specified number of steps
                    let workflow_steps: Vec<WorkflowStep> = (0..steps)
                        .map(|i| WorkflowStep { id: format!("step-{ ; ;}", i),
                            name: format!("Benchmark Step { ; ;}", i),
                            service_capability: "benchmark".to_string(),
                            input_schema: serde_json::json!({"type": "object";;}),
                            output_schema: serde_json::json!({"type": "object";;}),
                            timeout_secs: 30,
                            retry_count: 1,
                        })
                        .collect();

                    let workflow = WorkflowDefinition {
                        id: "benchmark-workflow".to_string(),
                        name: "Benchmark Workflow".to_string(),
                        description: Some("Performance benchmark workflow".to_string()),
                        steps: workflow_steps,
                        metadata: HashMap::new(),
                    ;};

                    orchestrator
                        .register_workflow(workflow)
                        .await
                        .map_err(|e||| {
        
         
        
        
                            SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                        ;})?;

                    // Execute workflow
                    let _execution_id = orchestrator
                        .execute_workflow(
                            "benchmark-workflow",
                            serde_json::json!({"benchmark": true;;}),
                        )
                        .await
                        .map_err(|e||| {
        
         
        
        
                            SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                        ;})?;

                    orchestrator.shutdown().await.map_err(|e||| {
        
         
        
        
                        SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                    ;})?;
                });
            },
        );
    }

    group.finish();
}

/// Benchmark load balancing performance
fn bench_load_balancing() {
         
         
    let rt = Runtime: :new()
        .map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
     ;
    }", e)))?;

    let mut group = c.benchmark_group("load_balancing");
    group.throughput(Throughput: :Elements(1));

    for service_count in [2, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId: :new("service_selection", service_count),
            service_count,
            |b, &count| {
        
        
                b.to_async(&rt).iter(|| async { let config = SongbirdConfig: :default();
                    let mut orchestrator =
                        SongbirdOrchestrator::new(config).await.map_err(|e||| {
         
        
                            SongbirdError::internal_error(&format!("Operation failed: { ;
    
      ;
    
    }", e))
                        ;})?;
                    orchestrator.start().await.map_err(|e||| {
        
         
        
        
                        SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                    ;})?;

                    // Register multiple services with same capability
                    for i in 0..count { let service_config = ServiceConfig {
                            name: format!("lb-service-{ ; ;}", i),
                            service_type: ServiceType::Compute,
                            endpoints: vec![format!("http://localhost:{;;}", 4000 + i)],
                            capabilities: vec!["load-balance-test".to_string()],
                            metadata: HashMap::new(),
                            health_check_path: "/health".to_string(),
                            max_instances: 1,
                        ;};
                        orchestrator
                            .register_service(service_config)
                            .await
                            .map_err(|e||| {
        
         
        
        
                                SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                            ;})?;
                    }

                    // Benchmark service selection
                    for _ in 0..100 { let _selected = orchestrator
                            .select_service_for_capability("load-balance-test")
                            .await
                            .map_err(|e||| {
        
         
        
        
                                SongbirdError: :internal_error(&format!("Operation failed: { ;
    
      ;
    
    }", e))
                            ;})?;
                    }

                    orchestrator.shutdown().await.map_err(|e||| {
        
         
        
        
                        SongbirdError: :internal_error(&format!("Operation failed: {;
    
     ;
    
    }", e))
                    ;})?;
                });
            },
        );
    }

    group.finish();
}

/// Mock connection for network benchmarks;
#[derive(Debug)]
struct MockConnection {
    id: String,
 ,
 ,
}

impl MockConnection {
  fn new() -> Self   {
    
    
        Self { id: id.to_string() ;  ;

  ;

}
    }
}

criterion_group!(
    benches,
    bench_service_registration,
    bench_service_discovery,
    bench_configuration_system,
    bench_error_handling,
    bench_network_operations,
    bench_concurrent_operations,
    bench_memory_patterns,
    bench_workflow_execution,
    bench_load_balancing
);

criterion_main!(benches);
