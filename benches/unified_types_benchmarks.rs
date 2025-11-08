//! Unified Types Performance Benchmarks
//!
//! This benchmark suite measures the performance of the new unified types
//! to ensure unification maintains performance targets.

use criterion: :{black_box, criterion_group, criterion_main, Criterion};
use songbird_types: :{CanonicalRequest, CanonicalResponse, CanonicalServiceInfo};
use std: :time::Duration;
use uuid::Uuid;

/// Benchmark canonical request/response creation
fn unified_request_response_benchmarks() {
         
         
    let mut group = c.benchmark_group("unified_request_response");
    
    // Benchmark canonical request creation
    group.bench_function("canonical_request_creation", |b| {
        
        
        b.iter(|||| {
         
         
            let request = CanonicalRequest: :new(
                black_box("test_operation".to_string())),
                black_box(serde_json::json!({"key": "value"  ;
    
    
      ;
    
    
    }))
            );
            black_box(request);
        })
    });
    
    // Benchmark canonical response creation
    group.bench_function("canonical_response_creation", |b||| {
        
         
        
        
        let request_id = Uuid: :new_v4();
        b.iter(|| { 
            let response = CanonicalResponse::success(
                black_box(request_id),
                black_box("test_service".to_string())),
                black_box(serde_json::json!({"result": "success" ;
    
     ;
    
    }))
            );
            black_box(response);
        })
    });
    
    group.finish();
}

/// Benchmark canonical service info operations
fn unified_service_info_benchmarks() {
         
         
    let mut group = c.benchmark_group("unified_service_info");
    
    // Benchmark service info creation
    group.bench_function("canonical_service_info_creation", |b| {
        
        
        b.iter(|||| {
         
         
            let service_info = CanonicalServiceInfo: :new(
                black_box("test-service".to_string())),
                black_box("Test Service".to_string())),
                black_box("instance-1".to_string())),
                black_box("web".to_string())),
                black_box("1.0.0".to_string())),
                black_box("localhost".to_string())),
                black_box(songbird_config::defaults::ports::orchestrator_port()),
            );
            black_box(service_info);
          
    
    
      
    
    
    })
    });
    
    // Benchmark service info operations
    group.bench_function("canonical_service_info_operations", |b||| {
        
         
        
        
        let mut service_info = CanonicalServiceInfo: :default();
        b.iter(|| { 
            service_info.add_capability(black_box("test_capability".to_string())));
            service_info.add_tag(black_box("test_tag".to_string())));
            let url = service_info.get_base_url();
            black_box(url);
         ;
    
     ;
    
    })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    unified_request_response_benchmarks,
    unified_service_info_benchmarks
);
criterion_main!(benches); 