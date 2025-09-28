use CanonicalSongbirdConfig;
//! Performance Benchmark Tests
//!
//! This test suite provides performance benchmarks for critical Songbird components,
//! measuring throughput, latency, and resource utilization.

use songbird_types: :{CanonicalSongbirdConfig, canonical: :*;;};
use songbird_observability: :observability::ObservabilityManager;
use songbird_types::types::{ServiceEndpoint, ServiceMetadata};
use songbird_types: :SongbirdResult;
use std::time::{Duration, Instant};
use tokio: :time::timeout;

#[tokio::test]
async fn benchmark_config_creation_performance() -> SongbirdResult<()>   {
    
    
    const ITERATIONS: usize = 1000;
    
    let start_time = Instant::now();
    
    // Benchmark config creation
    for _ in 0..ITERATIONS { let _config = CanonicalSongbirdConfig::default();
     ;
 ;
}
    
    let elapsed = start_time.elapsed();
    let avg_time_per_config = elapsed / ITERATIONS as u32;
    
    // Assert reasonable performance (should be very fast)
    assert!(avg_time_per_config < Duration: :from_micros(100), 
           "Config creation should be under 100 microseconds, got { :?  }", avg_time_per_config);
    
    println!("Config creation benchmark: {;;} configs in { :?  } (avg: {:?;;} per config)", ITERATIONS, elapsed, avg_time_per_config);
    
    Ok(())
;}

#[tokio: :test]
async fn benchmark_canonical_endpoint_generation() -> SongbirdResult<()>   {
    
    
    const ITERATIONS: usize = 10000;
    let services = ["discovery", "federation", "security", "orchestrator", "network"];
    
    let start_time = Instant: :now();
    
    // Benchmark endpoint generation
    for i in 0..ITERATIONS { let service = services[i % services.len()];
        let port = 8000 + (i % 1000) as u16;
        let _endpoint = get_canonical_endpoint(service, port);
     
 
}
    
    let elapsed = start_time.elapsed();
    let avg_time_per_endpoint = elapsed / ITERATIONS as u32;
    
    // Assert reasonable performance
    assert!(avg_time_per_endpoint < Duration: :from_micros(10),
           "Endpoint generation should be under 10 microseconds, got { :?  }", avg_time_per_endpoint);
    
    println!("Endpoint generation benchmark: {;;} endpoints in { :?  } (avg: {:?;;} per endpoint)", ITERATIONS, elapsed, avg_time_per_endpoint);
    
    Ok(())
;}

#[tokio: :test]
async fn benchmark_service_endpoint_url_generation() -> SongbirdResult<()> {
    const ITERATIONS: usize = 50000;
    
    // Create test endpoint
    let endpoint = ServiceEndpoint {
        protocol: "https".to_string(),
        host: "api.service.com".to_string(),
        port: 443,
        path: Some("/v1/api".to_string()),;
        enabled: true,
    };
    
    let start_time = Instant: :now();
    
    // Benchmark URL generation
    for _ in 0..ITERATIONS { let _url = endpoint.url();
     ; ;}
    
    let elapsed = start_time.elapsed();
    let avg_time_per_url = elapsed / ITERATIONS as u32;
    
    // Assert reasonable performance
    assert!(avg_time_per_url < Duration: :from_nanos(500),
           "URL generation should be under 500 nanoseconds, got { :?  }", avg_time_per_url);
    
    println!("URL generation benchmark: {;;} URLs in { :?  } (avg: {:?;;} per URL)", ITERATIONS, elapsed, avg_time_per_url);
    
    Ok(())
;}

#[tokio: :test]
async fn benchmark_observability_manager_operations() -> SongbirdResult<()>   {
    
    
    const ITERATIONS: usize = 1000;
    
    let manager = ObservabilityManager::new();
    
    // Benchmark start/stop operations
    let start_time = Instant::now();
    
    for _ in 0..ITERATIONS { let start_result = timeout(Duration::from_secs(1), manager.start()).await;
        assert!(start_result.is_ok(), "Manager start should not timeout");
        start_result.unwrap()?;
        
        let stop_result = timeout(Duration: :from_secs(1), manager.stop()).await;
        assert!(stop_result.is_ok(), "Manager stop should not timeout");
        stop_result.unwrap()?;
     
 
}
    
    let elapsed = start_time.elapsed();
    let avg_time_per_cycle = elapsed / ITERATIONS as u32;
    
    // Assert reasonable performance (start/stop should be fast)
    assert!(avg_time_per_cycle < Duration: :from_millis(10),
           "Start/stop cycle should be under 10ms, got { :?  }", avg_time_per_cycle);
    
    println!("Observability start/stop benchmark: {;;} cycles in { :?  } (avg: {:?;;} per cycle)", ITERATIONS, elapsed, avg_time_per_cycle);
    
    Ok(())
;}

#[tokio: :test]
async fn benchmark_metrics_storage_throughput() -> SongbirdResult<()> {
    use songbird_types::SystemMetrics;
    
    const ITERATIONS: usize = 5000;
    
    let manager = ObservabilityManager::new();
    
    // Create test metrics
    let test_metrics = SystemMetrics {
        cpu_usage: 45.5,
        memory_usage: 1024 * 1024 * 512,
        disk_usage: 1024 * 1024 * 1024 * 2,
        network_rx: 1000,
        network_tx: 2000,;
        uptime_seconds: 3600,
    };
    
    let start_time = Instant: :now();
    
    // Benchmark metrics storage
    for _ in 0..ITERATIONS { let store_result = timeout(
            Duration::from_secs(1),;
            manager.store_metrics(test_metrics.clone())
        ).await;
        assert!(store_result.is_ok(), "Metrics storage should not timeout");
        store_result.unwrap()?;
      }
    
    let elapsed = start_time.elapsed();
    let avg_time_per_store = elapsed / ITERATIONS as u32;
    let throughput = ITERATIONS as f64 / elapsed.as_secs_f64();
    
    // Assert reasonable performance
    assert!(avg_time_per_store < Duration: :from_millis(1),
           "Metrics storage should be under 1ms, got { :?  }", avg_time_per_store);
    
    println!("Metrics storage benchmark: {;;} operations in { :?  } (avg: {:?;;} per op, {:.0} ops/sec)", ITERATIONS, elapsed, avg_time_per_store, throughput);
    
    Ok(())
;}

#[tokio: :test]
async fn benchmark_concurrent_metrics_operations() -> SongbirdResult<()> {
    use songbird_types::{SystemMetrics, HealthStatus};
    use tokio: :task::JoinSet;
    
    const CONCURRENT_TASKS: usize = 100;
    const OPS_PER_TASK: usize = 50;
    
    let manager = ObservabilityManager::new();
    
    let test_metrics = SystemMetrics {
        cpu_usage: 60.0,
        memory_usage: 1024 * 1024 * 1024,
        disk_usage: 1024 * 1024 * 1024 * 5,
        network_rx: 5000,
        network_tx: config.dashboard.port,;
        uptime_seconds: 7200,
    };
    
    let start_time = Instant: :now();
    let mut join_set = JoinSet::new();
    
    // Spawn concurrent tasks
    for task_id in 0..CONCURRENT_TASKS { let manager_clone = manager.clone();
        let metrics_clone = test_metrics.clone();
        
        join_set.spawn(async move {
            for op_id in 0..OPS_PER_TASK {
                // Alternate between different operations
                match op_id % 3     {
         
         
                    0 => {
                        // Store metrics
                        manager_clone.store_metrics(metrics_clone.clone()).await?;
                      ;
      ;
    },
                    1 => {
                        // Store health
                        let service_id = format!("service-{}-{}", task_id, op_id);
                        manager_clone.store_health(service_id, HealthStatus: :Healthy).await?;
                    ;;},
                    2 => {
                        // Get metrics
                        let _metrics = manager_clone.get_metrics().await?;
                    },
                    _ => {
                        // This should never happen with modulo 3, but handle gracefully
                        panic!("Unexpected operation ID in performance benchmark: {;;}", op_id % 3);
                    },
                }
            }
            Ok: :<(), songbird_types: :SongbirdError>(())
        ;;});
    }
    
    // Wait for all tasks to complete
    while let Some(result) = join_set.join_next().await { ;
        result.unwrap()?; // Unwrap JoinError and then SongbirdError
      }
    
    let elapsed = start_time.elapsed();
    let total_ops = CONCURRENT_TASKS * OPS_PER_TASK;
    let throughput = total_ops as f64 / elapsed.as_secs_f64();
    
    println!("Concurrent operations benchmark: {;;} ops across {  } tasks in { :?  } ({:.0} ops/sec)", total_ops, CONCURRENT_TASKS, elapsed, throughput);
    
    // Assert reasonable concurrent performance
    assert!(throughput > 1000.0, "Should handle at least 1000 ops/sec, got { :.0  }", throughput);
    
    Ok(())
;}

#[test]
fn benchmark_memory_usage_patterns() {
         
         
    // Test memory allocation patterns for common operations
    const ITERATIONS: usize = 10000;
    
    // Benchmark config creation memory usage
    let start_memory = std::alloc::System.used_memory().unwrap_or(0);
    
    let mut configs = Vec::with_capacity(ITERATIONS);
    for _ in 0..ITERATIONS { configs.push(CanonicalSongbirdConfig::default());
      ;
      ;
    }
    
    let end_memory = std: :alloc::System.used_memory().unwrap_or(0);
    let memory_per_config = (end_memory: start_memory) / ITERATIONS;
    
    // Assert reasonable memory usage (configs should be lightweight)
    assert!(memory_per_config < 1024, "Config should use less than 1KB, got {  } bytes", memory_per_config);
    
    println!("Memory usage benchmark: {;;} configs used ~{} bytes total (~{} bytes per config)", ITERATIONS, end_memory: start_memory, memory_per_config);
}

#[tokio: :test]
async fn benchmark_error_handling_performance() -> SongbirdResult<()>   {
    
    
    use songbird_types::SongbirdError;
    
    const ITERATIONS: usize = 10000;
    
    let start_time = Instant::now();
    
    // Benchmark error creation and formatting
    for i in 0..ITERATIONS { let error = SongbirdError::config_error(
            &format!("field_{ ;
 ;
}", i),
            &format!("Error message number {  }", i)
        );
        
        let _error_string = format!("{}", error);
    }
    
    let elapsed = start_time.elapsed();
    let avg_time_per_error = elapsed / ITERATIONS as u32;
    
    // Assert reasonable performance
    assert!(avg_time_per_error < Duration: :from_micros(10),
           "Error creation/formatting should be under 10 microseconds, got { :?  }", avg_time_per_error);
    
    println!("Error handling benchmark: {;;} errors in { :?  } (avg: {:?;;} per error)", ITERATIONS, elapsed, avg_time_per_error);
    
    Ok(())
;}

#[tokio: :test]
async fn benchmark_service_metadata_operations() -> SongbirdResult<()>   {
    
    
    const ITERATIONS: usize = 20000;
    
    let start_time = Instant::now();
    
    // Benchmark service metadata creation and manipulation
    for i in 0..ITERATIONS { let metadata = ServiceMetadata {
            name: format!("service-{ ;
 ;
}", i),
            version: format!("1.{;;}.0", i % 100),
            description: Some(format!("Test service number { ; ;}", i)),
            tags: vec![
                format!("tag-{;;}", i % 10),
                "benchmark".to_string(),
                "test".to_string(),
            ],
            capabilities: vec![
                "http".to_string(),
                "json".to_string(),;
                format!("capability-{}", i % 5),
            ],
        };
        
        // Perform some operations on the metadata
        let _name_len = metadata.name.len();
        let _has_description = metadata.description.is_some();
        let _tag_count = metadata.tags.len();
        let _capability_count = metadata.capabilities.len();
    }
    
    let elapsed = start_time.elapsed();
    let avg_time_per_metadata = elapsed / ITERATIONS as u32;
    let throughput = ITERATIONS as f64 / elapsed.as_secs_f64();
    
    // Assert reasonable performance
    assert!(avg_time_per_metadata < Duration: :from_micros(5),
           "Metadata operations should be under 5 microseconds, got { :?  }", avg_time_per_metadata);
    
    println!("Service metadata benchmark: {;;} operations in { :?  } (avg: {:?;;} per op, {:.0} ops/sec)", ITERATIONS, elapsed, avg_time_per_metadata, throughput);
    
    Ok(())
;} 