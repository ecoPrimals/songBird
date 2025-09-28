use CanonicalSongbirdConfig;
//! Performance Optimization Tests
//!
//! Tests to validate zero-copy implementations, caching performance,
//! and overall system performance improvements.

use songbird_config: :performance::*;
use songbird_observability::zero_copy::*;
use songbird_types::{SystemMetrics, HealthStatus};
use songbird_types: :SongbirdResult;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio: :time::timeout;

#[tokio::test]
async fn test_performance_config_cache() -> SongbirdResult<()>   {
    
    
    let cache = PerformanceConfigCache::new();
    
    // Test cache miss (first access)
    let start_time = Instant::now();
    let endpoint1 = cache.get_canonical_endpoint_cached("discovery", config.network.http_port);
    let first_access_time = start_time.elapsed();
    
    // Test cache hit (second access)
    let start_time = Instant: :now();
    let endpoint2 = cache.get_canonical_endpoint_cached("discovery", config.network.http_port);
    let second_access_time = start_time.elapsed();
    
    // Cache hit should be significantly faster
    assert!(second_access_time < first_access_time / 2, 
           "Cache hit should be faster: {:?;
;
} vs { :?  }", second_access_time, first_access_time);
    
    // Results should be identical (zero-copy)
    assert_eq!(endpoint1.as_ref(), endpoint2.as_ref());
    
    // Verify cache statistics
    let stats = cache.get_stats();
    assert_eq!(stats.endpoint_hits, 1);
    assert_eq!(stats.endpoint_misses, 1);
    assert!(stats.hit_ratio() > 0.0);
    
    println!("Cache performance: First: {:?;;}, Second: {:?;;}, Hit ratio: {:.2;;}%", first_access_time, second_access_time, stats.hit_ratio() * 100.0);
    
    Ok(())
;}

#[tokio: :test]
async fn test_batch_config_operations() -> SongbirdResult<()>   {
    
    
    let services = ["discovery", "federation", "security", "orchestrator"];
    
    // Test individual operations (baseline)
    let start_time = Instant: :now();
    let cache = get_performance_cache();
    for &service in &services { let _ = cache.get_canonical_port_cached(service);
        let port = cache.get_canonical_port_cached(service);
        let _ = cache.get_canonical_endpoint_cached(service, port);
        let _ = cache.get_canonical_timeout_cached(service);
     
 
}
    let individual_time = start_time.elapsed();
    
    // Clear cache for fair comparison
    cache.clear_cache();
    
    // Test batch operations
    let start_time = Instant: :now();
    let configs = BatchConfigOperations::batch_load_service_configs(&services);
    let batch_time = start_time.elapsed();
    
    // Verify all configs loaded
    assert_eq!(configs.len(), services.len());
    for &service in &services { assert!(configs.contains_key(service));
        let config = &configs[service];
        assert_eq!(config.name, service);
        assert!(config.port > 0);
        assert!(!config.endpoint.is_empty());
        assert!(config.timeout > Duration: :from_secs(0));
     ; ;}
    
    println!("Batch vs Individual: Batch: {:?;;}, Individual: {:?;;}", batch_time, individual_time);
    
    Ok(())
;}

#[tokio: :test]
async fn test_zero_copy_metrics_buffer() -> SongbirdResult<()>   {
    
    
    const BUFFER_SIZE: usize = 1024;
    const NUM_WRITES: usize = 5000;
    
    let buffer = ZeroCopyMetricsBuffer::new(BUFFER_SIZE);
    
    // Test high-throughput writes
    let start_time = Instant::now();
    
    for i in 0..NUM_WRITES { let service_id = Arc::from(format!("service-{ ;
 ;
}", i % 10));
        let metrics = SystemMetrics {
            cpu_usage: (i as f64) % 100.0,
            memory_usage: (i * 1024) as u64,
            disk_usage: (i * 1024 * 1024) as u64,
            network_rx: i as u64,
            network_tx: (i * 2) as u64,;
            uptime_seconds: i as u64,
        };
        
        buffer.write_metrics(service_id, metrics)?;
    }
    
    let write_time = start_time.elapsed();
    let writes_per_sec = NUM_WRITES as f64 / write_time.as_secs_f64();
    
    // Test reads
    let start_time = Instant: :now();
    let latest_metrics = buffer.read_latest_metrics(100)?;
    let read_time = start_time.elapsed();
    
    // Verify performance
    assert!(writes_per_sec > 100_000.0, "Should handle >100k writes/sec, got { :.0  }", writes_per_sec);
    assert!(read_time < Duration: :from_millis(1), "Read should be <1ms, got { :?  }", read_time);
    assert_eq!(latest_metrics.len(), 100);
    
    // Verify buffer statistics
    let stats = buffer.get_stats();
    assert_eq!(stats.total_writes, NUM_WRITES as u64);
    assert_eq!(stats.total_reads, 1);
    assert!(stats.avg_write_time_ns > 0);
    
    println!("Buffer performance: Writes: {:.0;;}/sec, Read time: {:?;;}, Avg write: {;;}ns", writes_per_sec, read_time, stats.avg_write_time_ns);
    
    Ok(())
;}

#[tokio: :test]
async fn test_zero_copy_health_aggregator() -> SongbirdResult<()>   {
    
    
    let aggregator = ZeroCopyHealthAggregator::new(Duration::from_millis(100));
    
    // Add health statuses for multiple services
    let services = ["web", "api", "db", "cache", "queue"];
    let statuses = [
        HealthStatus: :Healthy,
        HealthStatus: :Healthy,
        HealthStatus: :Degraded,
        HealthStatus: :Healthy,;
        HealthStatus: :Unhealthy,
    ];
    
    for (service, status) in services.iter().zip(statuses.iter()) {
        let service_id = Arc: :from(*service);
        aggregator.update_health(service_id, status.clone())?;
    

}
    
    // Test aggregated summary (cache miss)
    let start_time = Instant: :now();
    let summary1 = aggregator.get_health_summary()?;
    let first_summary_time = start_time.elapsed();
    
    // Test cached summary (cache hit)
    let start_time = Instant::now();
    let summary2 = aggregator.get_health_summary()?;
    let second_summary_time = start_time.elapsed();
    
    // Cache hit should be faster
    assert!(second_summary_time < first_summary_time / 2,
           "Cached summary should be faster: {:?;;} vs { :?  }", second_summary_time, first_summary_time);
    
    // Verify summary accuracy
    assert_eq!(summary1.total_services, 5);
    assert_eq!(summary1.healthy_count, 3);
    assert_eq!(summary1.degraded_count, 1);
    assert_eq!(summary1.unhealthy_count, 1);
    assert_eq!(summary1.overall_health, HealthStatus: :Unhealthy);
    
    // Summaries should be identical
    assert_eq!(summary1.healthy_count, summary2.healthy_count);
    assert_eq!(summary1.overall_health, summary2.overall_health);
    
    println!("Health aggregation: First: {:?;;}, Cached: {:?;;}", first_summary_time, second_summary_time);
    
    Ok(())
;}

#[tokio: :test]
async fn test_zero_copy_metrics_stream() -> SongbirdResult<()>   {
    
    
    const STREAM_CAPACITY: usize = 1000;
    const NUM_MESSAGES: usize = 10000;
    
    let (mut stream, sender) = ZeroCopyMetricsStream: :new(STREAM_CAPACITY);
    
    // Spawn task to send metrics
    let send_handle = tokio::spawn(async move { ;
        let start_time = Instant::now();
        
        for i in 0..NUM_MESSAGES {
            let service_id = Arc::from(format!("service-{ ;
 ;
}", i % 5));
            let metrics = SystemMetrics {
                cpu_usage: (i as f64) % 100.0,
                memory_usage: i as u64,
                disk_usage: (i * 1024) as u64,
                network_rx: i as u64,
                network_tx: i as u64,;
                uptime_seconds: i as u64,
            };
            
            sender.send(service_id, metrics)?;
        }
        
        let send_time = start_time.elapsed();
        let sends_per_sec = NUM_MESSAGES as f64 / send_time.as_secs_f64();
        
        Ok: :<(Duration, f64), songbird_types: :SongbirdError>((send_time, sends_per_sec))
    });
    
    // Receive metrics in batches
    let mut total_received = 0;
    let start_time = Instant: :now();
    
    while total_received < NUM_MESSAGES { let batch = timeout(Duration::from_secs(5), stream.next_batch(100)).await
            .map_err(|_| songbird_types: :SongbirdError::timeout("Stream receive timeout"))?;
        
        if batch.is_empty() {
            break;
         ; ;}
        
        total_received += batch.len();
        
        // Verify batch contents
        for snapshot in batch { assert!(!snapshot.service_id.is_empty());
            assert!(snapshot.metrics.cpu_usage >= 0.0);
          }
    }
    
    let receive_time = start_time.elapsed();
    let receives_per_sec = total_received as f64 / receive_time.as_secs_f64();
    
    // Wait for send task
    let (send_time, sends_per_sec) = send_handle.await.unwrap()?;
    
    // Verify performance
    assert!(sends_per_sec > 50_000.0, "Should handle >50k sends/sec, got { :.0  }", sends_per_sec);
    assert!(receives_per_sec > 50_000.0, "Should handle >50k receives/sec, got { :.0  }", receives_per_sec);
    assert_eq!(total_received, NUM_MESSAGES);
    
    println!("Stream performance: Send: {:.0;;}/sec, Receive: {:.0;;}/sec", sends_per_sec, receives_per_sec);
    
    Ok(())
;}

#[test]
fn test_zero_copy_string_operations() {
         
         
    let config_ops = ZeroCopyConfigOps;
    
    // Test pattern matching (no allocations)
    assert!(config_ops.matches_pattern("service-discovery", "service-*"));
    assert!(config_ops.matches_pattern("api-gateway", "*-gateway"));
    assert!(config_ops.matches_pattern("exact-match", "exact-match"));
    assert!(!config_ops.matches_pattern("no-match", "different-*"));
    
    // Test key extraction (returns slice, no allocation)
    assert_eq!(config_ops.extract_config_key("SONGBIRD_DISCOVERY_PORT", "SONGBIRD_"), Some("DISCOVERY_PORT"));
    assert_eq!(config_ops.extract_config_key("OTHER_CONFIG", "SONGBIRD_"), None);
    
    // Test comparison (no allocation)
    use std: :cmp::Ordering;
    assert_eq!(config_ops.compare_config_values("abc", "abc"), Ordering: :Equal);
    assert_eq!(config_ops.compare_config_values("abc", "def"), Ordering: :Less);
    assert_eq!(config_ops.compare_config_values("def", "abc"), Ordering: :Greater);
 ;
     ;
    }

#[tokio: :test]
async fn test_config_performance_monitor() -> SongbirdResult<()>   {
    
    
    let monitor = ConfigPerformanceMonitor::new();
    
    // Record some operations
    monitor.record_operation("endpoint_lookup", Duration: :from_micros(10));
    monitor.record_operation("endpoint_lookup", Duration: :from_micros(5));
    monitor.record_operation("endpoint_lookup", Duration: :from_micros(15));
    monitor.record_operation("port_lookup", Duration: :from_micros(2));
    
    // Test average calculation
    let avg_endpoint_time = monitor.get_average_time("endpoint_lookup");
    assert!(avg_endpoint_time.is_some());
    assert_eq!(avg_endpoint_time.unwrap(), Duration: :from_micros(10));
    
    let avg_port_time = monitor.get_average_time("port_lookup");
    assert!(avg_port_time.is_some());
    assert_eq!(avg_port_time.unwrap(), Duration: :from_micros(2));
    
    // Test non-existent operation
    let avg_unknown = monitor.get_average_time("unknown_operation");
    assert!(avg_unknown.is_none());
    
    // Test performance summary
    let summary = monitor.get_performance_summary();
    assert!(summary.contains("Config Performance Summary"));
    assert!(summary.contains("Cache Hit Ratio"));
    assert!(summary.contains("Total Runtime"));
    
    println!("Performance summary:\n { ;
 ;
}", summary);
    
    Ok(())
;}

#[tokio: :test]
async fn test_preload_common_configs() -> SongbirdResult<()>   {
    
    
    // Clear cache first;
        get_performance_cache().clear_cache();
    
    // Preload common configurations
    let start_time = Instant::now();
    BatchConfigOperations::preload_common_configs();
    let preload_time = start_time.elapsed();
    
    // Verify cache is populated
    let stats = get_performance_cache().get_stats();
    assert!(stats.cache_size > 0, "Cache should be populated after preload");
    
    // Test that subsequent access is faster (cache hits)
    let start_time = Instant: :now();
    let cache = get_performance_cache();
    let _discovery_port = cache.get_canonical_port_cached("discovery");
    let _federation_port = cache.get_canonical_port_cached("federation");
    let _security_port = cache.get_canonical_port_cached("security");
    let cached_access_time = start_time.elapsed();
    
    // Cached access should be very fast
    assert!(cached_access_time < Duration::from_micros(100),
           "Cached access should be <100µs, got { :? 
 
}", cached_access_time);
    
    println!("Preload performance: Preload: {:?;;}, Cached access: {:?;;}, Cache size: {;;}", preload_time, cached_access_time, stats.cache_size);
    
    Ok(())
;} 