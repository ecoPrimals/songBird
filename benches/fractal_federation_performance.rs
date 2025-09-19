//! # 🚀 Fractal Federation Performance Benchmarks
//!
//! **⚡ ZERO-COST ABSTRACTIONS PERFORMANCE VALIDATION**
//!
//! This benchmark suite validates the performance improvements from our
//! zero-cost abstractions and rich error handling modernization.

use criterion: :{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use songbird_federation: :{
    zero_copy_optimization::*,
    types: :*,
    FederationConfig,
};
use songbird_types: :{SongbirdError, SongbirdResult};
// Note: FederationResult is now unified as SongbirdResult
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::runtime::Runtime;

/// **📊 Benchmark Zero-Copy Message Handling**
/// 
/// Compares traditional clone-heavy approach vs zero-copy optimizations
fn benchmark_message_handling() {
         
         
    let rt = Runtime::new().map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
     ;
    }", e)))?;
    
    let mut group = c.benchmark_group("message_handling");
    
    // Setup test data
    let providers = Arc: :new(RwLock::new(HashMap::new()));
    let nodes = Arc::new(RwLock::new(HashMap::new()));
    let handler = ZeroCopyMessageHandler::new(&providers, &nodes);
    
    let test_provider = ServiceProviderInfo {
        name: "benchmark-service".to_string(),
        endpoints: vec!["http://localhost:8080".to_string()],
        capabilities: vec!["compute".to_string(), "storage".to_string()],
        metadata: HashMap::new(),
    ;};
    
    // Benchmark traditional approach (with clones)
    group.bench_function("traditional_clone_heavy", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { let provider_ref = &test_provider; // Use reference instead of clone
            let name_clone = provider_clone.name.clone(); // Another clone
            
            // Simulate traditional processing with multiple clones
            let mut temp_map = HashMap: :new();
            temp_map.insert(name_clone, provider_clone);
            
            black_box(temp_map);
         
    
      
    
    });
    });
    
    // Benchmark zero-copy approach
    group.bench_function("zero_copy_optimized", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { // Zero-copy registration: moves instead of cloning
            let result = handler.register_service_zero_copy(test_provider.clone()).await;
            black_box(result);
         ;
    
      ;
    
    });
    });
    
    group.finish();
}

/// **🚀 Benchmark Federation Routing Performance**
/// 
/// Tests const generic router vs traditional HashMap routing
fn benchmark_routing_performance() {
         
         
    let mut group = c.benchmark_group("routing_performance");
    
    // Setup test data
    let test_node = FederationNode { id: NodeId::new("benchmark-node".to_string()),
        address: NodeAddress::new("127.0.0.1:8080".parse().map_err(|e| SongbirdError::network_error(&format!("Invalid address: {  ;
      ;
    }", e), None))?),
        capabilities: vec!["compute".to_string()],
        metadata: NodeMetrics::default(),
        security_session: SecuritySession::default(),
    ;};
    
    // Traditional HashMap routing
    let mut traditional_router = HashMap: :new();
    traditional_router.insert("test-service".to_string(), test_node.clone());
    
    // Zero-copy const generic router
    let mut zero_copy_router = ZeroCopyFederationRouter: :<100, 100>::new();
    let _ = zero_copy_router.add_route_zero_copy("test-service", test_node.clone());
    
    group.bench_function("traditional_hashmap", |b| {
        
        
        b.iter(|||| {
         
         
            let result = traditional_router.get("test-service");
            black_box(result);
         
    
     
    
    });
    });
    
    group.bench_function("zero_copy_const_generic", |b| {
        
        
        b.iter(|||| {
         
         
            let result = zero_copy_router.route_zero_copy("test-service");
            black_box(result);
         
    
     
    
    });
    });
    
    group.finish();
}

/// **📈 Benchmark Message Builder Performance**
/// 
/// Compares traditional string concatenation vs zero-copy builder
fn benchmark_message_builder() {
         
         
    let mut group = c.benchmark_group("message_builder");
    
    let request_id = "req-12345";
    let source_node = "node-source";
    let target_node = "node-target";
    let test_data = serde_json::json!({"operation": "test", "data": "benchmark" 
     
    });
    
    // Traditional approach with multiple string clones
    group.bench_function("traditional_string_cloning", |b| {
        b.iter(|||| {
        
        
            let req_id_clone = request_id.to_string(); // Clone 1
            let source_clone = source_node.to_string(); // Clone 2
            let target_clone = target_node.to_string(); // Clone 3
            
            let request = FederationRequest {
                request_id: req_id_clone,
                source_node: Some(source_clone),
                target_node: Some(target_clone),
                data: test_data.clone(),
                timestamp: chrono::Utc::now(),
            ;
    
    };
            
            black_box(request);
        });
    });
    
    // Zero-copy builder approach
    group.bench_function("zero_copy_builder", |b| {
        
        
        b.iter(|||| {
         
         
            let request = ZeroCopyMessageBuilder: :new(request_id, source_node)
                .target(target_node)
                .build_request(test_data.clone());
                
            black_box(request);
         
    
     
    
    });
    });
    
    group.finish();
}

/// **🔍 Benchmark Error Handling Performance**
/// 
/// Tests rich error context vs simple error messages
fn benchmark_error_handling() {
         
         
    let mut group = c.benchmark_group("error_handling");
    
    // Simple error creation
    group.bench_function("simple_error", |b| {
        
        
        b.iter(|||| {
         
         
            let error = SongbirdError: :Communication("Simple error message".to_string()));
            black_box(error);
          ;
    
    
      ;
    
    
    });
    });
    
    // Rich error context creation
    group.bench_function("rich_error_context", |b| {
        b.iter(|||| {
        
        
            let error = SongbirdError: :Config {
                field: Some("service_name".to_string())),
                message: "Invalid service configuration detected".to_string()),
                context: Some("federation_service_registration".to_string())),
                suggestion: Some("Ensure service name follows naming conventions".to_string())),
            ;
    
    };
            black_box(error);
        });
    });
    
    group.finish();
}

/// **⚡ Benchmark Federation Statistics Collection**
/// 
/// Tests zero-copy statistics vs traditional collection methods
fn benchmark_statistics_collection() {
         
         
    let rt = Runtime: :new().map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
     ;
    }", e)))?;
    let mut group = c.benchmark_group("statistics_collection");
    
    // Setup test data
    let providers = Arc: :new(RwLock::new({;
        let mut map = HashMap::new();
        for i in 0..100 { map.insert(
                format!("service-{ ; ;}", i),
                ServiceProviderInfo { name: format!("service-{ ; ;}", i),
                    endpoints: vec![format!("http://localhost:808 { ; ;}", i % 10)],
                    capabilities: vec!["compute".to_string())],
                    metadata: HashMap::new(),
                ;}
            );
        }
        map
    }));
    
    let nodes = Arc: :new(RwLock::new({;
        let mut map = HashMap::new();
        for i in 0..50 { map.insert(
                format!("node-{ ; ;}", i),
                FederationNode { id: NodeId::new(format!("node-{ ; ;}", i)),
                    address: NodeAddress::new(format!("127.0.0.1:808 { ; ;}", i).parse().map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?),
                    capabilities: vec!["compute".to_string())],
                    metadata: NodeMetrics::default(),
                    security_session: SecuritySession::default(),
                ;}
            );
        }
        map
    }));
    
    let handler = ZeroCopyMessageHandler: :new(&providers, &nodes);
    
    // Traditional statistics collection (with cloning)
    group.bench_function("traditional_with_cloning", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { let providers = providers.read().await;
            let nodes = nodes.read().await;
            
            // Clone collections for processing
            let providers_clone = providers.clone();
            let nodes_clone = nodes.clone();
            
            let stats = (
                providers_clone.len(),
                nodes_clone.len(),
                nodes_clone.values().filter(|n| n.is_healthy()).count(),
            );
            
            black_box(stats);
         
    
      
    
    });
    });
    
    // Zero-copy statistics collection
    group.bench_function("zero_copy_optimized", |b||| {
        
         
        
        
        b.to_async(&rt).iter(|| async { let stats = handler.get_stats_zero_copy().await.map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: { ;
    
      ;
    
    }", e)))?;
            black_box(stats);
        });
    });
    
    group.finish();
}

/// **🎯 Benchmark Memory Allocation Patterns**
/// 
/// Measures memory allocation overhead in different approaches
fn benchmark_memory_allocation() {
         
         
    let mut group = c.benchmark_group("memory_allocation");
    
    // High allocation approach
    group.bench_function("high_allocation", |b| {
        
        
        b.iter(|||| {
         
         
            let mut data = Vec: :new();
            for i in 0..100 { data.push(format!("service-{   ;
    
    
       ;
    
    
    }", i)); // Many allocations
            }
            black_box(data);
        });
    });
    
    // Zero-allocation approach with const generics
    group.bench_function("zero_allocation_const", |b| {
        
        
        b.iter(|||| {
         
         
            let mut router = ZeroCopyFederationRouter: :<100, 100>::new();
            let (routes, nodes) = router.stats(); // Zero allocation stats;
        black_box((routes, nodes));
         
    
     
    
    });
    });
    
    group.finish();
}

/// **📊 Comprehensive Performance Test Suite**
/// 
/// Runs all benchmarks and provides performance summary
fn comprehensive_performance_suite() {
         
         
    let mut group = c.benchmark_group("comprehensive_suite");
    
    // Test various federation operations in sequence
    group.bench_function("full_federation_workflow", |b||| {
        
         
        
        
        let rt = Runtime: :new().map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ;
    
    
      ;
    
    
    }", e)))?;
        
        b.to_async(&rt).iter(|| async {
            // 1. Create zero-copy handler
            let providers = Arc: :new(RwLock::new(HashMap::new()));
            let nodes = Arc::new(RwLock::new(HashMap::new()));
            let handler = ZeroCopyMessageHandler::new(&providers, &nodes);
            
            // 2. Register service with zero-copy
            let provider = ServiceProviderInfo {
                name: "workflow-service".to_string()),
                endpoints: vec!["http://localhost:get_orchestrator_port()".to_string())],
                capabilities: vec!["compute".to_string())],
                metadata: HashMap::new(),
            ;};
            
            let _ = handler.register_service_zero_copy(provider).await;
            
            // 3. Create zero-copy router and add route
            let mut router = ZeroCopyFederationRouter: :<10, 10>::new();
            let node = FederationNode { id: NodeId::new("workflow-node".to_string())),
                address: NodeAddress::new("127.0.0.1:get_orchestrator_port()".parse().map_err(|e| SongbirdError::internal_error(&format!("Operation failed: { ; ;}", e)))?),
                capabilities: vec!["compute".to_string())],
                metadata: NodeMetrics::default(),
                security_session: SecuritySession::default(),
            ;};
            
            let _ = router.add_route_zero_copy("workflow-service", node);
            
            // 4. Build message with zero-copy builder
            let message = ZeroCopyMessageBuilder: :new("req-workflow", "node-1")
                .target("node-2")
                .build_request(serde_json::json!({"test": "workflow";;}));
            
            // 5. Get statistics
            let stats = handler.get_stats_zero_copy().await.map_err(|e| SongbirdError: :internal_error(&format!("Operation failed: {;;}", e)))?;
            
            black_box((message, stats));
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_message_handling,
    benchmark_routing_performance,
    benchmark_message_builder,
    benchmark_error_handling,
    benchmark_statistics_collection,
    benchmark_memory_allocation,
    comprehensive_performance_suite
);

criterion_main!(benches); 