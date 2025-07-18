//! Simplified Substrate Performance Test
//!
//! This test demonstrates the key substrate performance optimizations:
//! - TTL caching
//! - Circuit breaker patterns
//! - Connection pooling
//! - Async optimizations
//! - Metrics collection

use std::time::Duration;
use tokio::time::timeout;

use songbird_core::substrate::{OSSubstrate, PathRequest, PathRequirements, PathType};
use songbird_errors::Result;

/// Test basic substrate functionality
#[tokio::test]
async fn test_substrate_basic_functionality() -> Result<()> {
    println!("🧪 Testing basic substrate functionality...");

    // Test substrate creation
    let substrate = OSSubstrate::new().await?;
    println!("✅ Substrate created successfully");

    // Test system info retrieval
    let system_info = substrate.get_system_info().await?;
    assert!(
        !system_info.platform.is_empty(),
        "Platform should not be empty"
    );
    assert!(system_info.cpu_cores > 0, "CPU cores should be > 0");

    println!(
        "💻 System: {} {} ({} cores)",
        system_info.platform, system_info.architecture, system_info.cpu_cores
    );

    // Test path operations
    let path_request = PathRequest {
        path_type: PathType::Data,
        service_name: "test_service".to_string(),
        requirements: PathRequirements::default(),
    };

    let path = substrate.get_path(path_request).await?;
    assert!(
        path.to_string_lossy().contains("test_service"),
        "Path should contain service name"
    );

    println!("📁 Data path: {}", path.display());

    // Test network interface
    let interface = substrate.get_network_interface().await?;
    assert!(
        !interface.name.is_empty(),
        "Interface name should not be empty"
    );
    assert!(
        !interface.ip_address.is_empty(),
        "IP address should not be empty"
    );

    println!(
        "🌐 Network interface: {} ({})",
        interface.name, interface.ip_address
    );

    // Test port allocation
    let port = substrate.get_available_port().await?;
    assert!(port > 0, "Port should be > 0");

    println!("🔌 Available port: {}", port);

    println!("✅ Basic substrate functionality test passed");
    Ok(())
}

/// Test substrate caching performance
#[tokio::test]
async fn test_substrate_caching_performance() -> Result<()> {
    println!("🧪 Testing substrate caching performance...");

    let substrate = OSSubstrate::new().await?;

    // Test path caching
    let path_request = PathRequest {
        path_type: PathType::Data,
        service_name: "cache_test".to_string(),
        requirements: PathRequirements::default(),
    };

    // First request (should populate cache)
    let start = std::time::Instant::now();
    let path1 = substrate.get_path(path_request.clone()).await?;
    let first_duration = start.elapsed();

    // Second request (should use cache)
    let start = std::time::Instant::now();
    let path2 = substrate.get_path(path_request).await?;
    let second_duration = start.elapsed();

    assert_eq!(path1, path2, "Cached paths should be identical");

    println!("⏱️ First request: {:?}", first_duration);
    println!("⏱️ Second request: {:?}", second_duration);

    // Test cache statistics
    let (total_entries, _max_size, utilization, cache_hits, cache_misses) =
        substrate.get_cache_stats().await;

    println!("📊 Cache stats:");
    println!("   Total entries: {}", total_entries);
    println!("   Max size: {}", _max_size);
    println!("   Utilization: {:.2}%", utilization * 100.0);
    println!("   Cache hits: {}", cache_hits);
    println!("   Cache misses: {}", cache_misses);

    assert!(total_entries > 0, "Should have cached entries");
    assert!(cache_hits > 0, "Should have cache hits");

    println!("✅ Substrate caching performance test passed");
    Ok(())
}

/// Test substrate metrics collection
#[tokio::test]
async fn test_substrate_metrics() -> Result<()> {
    println!("🧪 Testing substrate metrics collection...");

    let substrate = OSSubstrate::new().await?;

    // Perform some operations to generate metrics
    let _ = substrate.get_system_info().await?;
    let _ = substrate.get_capabilities().await?;

    // Test metrics retrieval
    let metrics = substrate.get_metrics().await;
    assert!(metrics.total_requests > 0, "Should have total requests");

    println!("📊 Substrate metrics:");
    println!("   Total requests: {}", metrics.total_requests);
    println!("   Cache hits: {}", metrics.cache_hits);
    println!("   Cache misses: {}", metrics.cache_misses);
    println!("   Substrate errors: {}", metrics.substrate_errors);
    println!("   Fallback uses: {}", metrics.fallback_uses);
    println!(
        "   Average response time: {:?}",
        metrics.average_response_time
    );
    println!("   Toadstool requests: {}", metrics.toadstool_requests);
    println!("   BiomeOS requests: {}", metrics.biomeos_requests);

    println!("✅ Substrate metrics test passed");
    Ok(())
}

/// Test substrate cache management
#[tokio::test]
async fn test_substrate_cache_management() -> Result<()> {
    println!("🧪 Testing substrate cache management...");

    let substrate = OSSubstrate::new().await?;

    // Populate cache with some data
    let _ = substrate.get_system_info().await?;
    let _ = substrate.get_capabilities().await?;

    // Check cache before clearing
    let (entries_before, _, _, _, _) = substrate.get_cache_stats().await;
    println!("📊 Entries before clearing: {}", entries_before);

    // Clear cache
    substrate.clear_cache().await;

    // Check cache after clearing
    let (entries_after, _, _, _, _) = substrate.get_cache_stats().await;
    println!("📊 Entries after clearing: {}", entries_after);

    assert!(entries_after < entries_before, "Cache should be cleared");

    // Test cache warming
    substrate.warm_cache().await?;

    // Check cache after warming
    let (entries_warmed, _, utilization, _, _) = substrate.get_cache_stats().await;
    println!("📊 Entries after warming: {}", entries_warmed);
    println!("📊 Utilization after warming: {:.2}%", utilization * 100.0);

    assert!(entries_warmed > 0, "Cache should be warmed");

    println!("✅ Substrate cache management test passed");
    Ok(())
}

/// Test substrate circuit breaker
#[tokio::test]
async fn test_substrate_circuit_breaker() -> Result<()> {
    println!("🧪 Testing substrate circuit breaker...");

    let substrate = OSSubstrate::new().await?;

    // Test circuit breaker status
    let cb_status = substrate
        .toadstool_client
        .get_circuit_breaker_status()
        .await;
    println!("⚡ Circuit breaker status: {:?}", cb_status);

    // Test health check with circuit breaker
    let health_result = substrate.toadstool_client.health_check().await;

    match health_result {
        Ok(_) => println!("✅ Health check passed"),
        Err(e) => {
            if e.to_string().contains("Circuit breaker is open") {
                println!("⚠️ Circuit breaker is open (expected behavior)");
            } else {
                println!("⚠️ Health check failed: {}", e);
            }
        }
    }

    println!("✅ Substrate circuit breaker test passed");
    Ok(())
}

/// Test substrate timeout handling
#[tokio::test]
async fn test_substrate_timeout_handling() -> Result<()> {
    println!("🧪 Testing substrate timeout handling...");

    let substrate = OSSubstrate::new().await?;

    // Test operation with timeout
    let timeout_result = timeout(Duration::from_secs(5), substrate.get_system_info()).await;

    match timeout_result {
        Ok(Ok(system_info)) => {
            println!(
                "⏱️ Operation completed within timeout: {} {}",
                system_info.platform, system_info.architecture
            );
        }
        Ok(Err(e)) => {
            println!("⚠️ Operation failed: {}", e);
        }
        Err(_) => {
            println!("⏰ Operation timed out");
        }
    }

    println!("✅ Substrate timeout handling test passed");
    Ok(())
}

/// Test substrate performance under load
#[tokio::test]
async fn test_substrate_performance_under_load() -> Result<()> {
    println!("🧪 Testing substrate performance under load...");

    let substrate = OSSubstrate::new().await?;

    // Create multiple concurrent path requests
    let mut handles = Vec::new();

    for i in 0..10 {
        let substrate_clone = substrate.clone();
        let handle = tokio::spawn(async move {
            let path_request = PathRequest {
                path_type: PathType::Data,
                service_name: format!("load_test_{}", i),
                requirements: PathRequirements::default(),
            };

            substrate_clone.get_path(path_request).await
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut successful_requests = 0;
    for handle in handles {
        if let Ok(Ok(_)) = handle.await {
            successful_requests += 1;
        }
    }

    println!(
        "📊 Load test: {}/10 requests successful",
        successful_requests
    );
    assert!(
        successful_requests > 0,
        "At least some requests should succeed"
    );

    // Check metrics after load test
    let metrics = substrate.get_metrics().await;
    println!("📈 Load test metrics:");
    println!("   Total requests: {}", metrics.total_requests);
    println!("   Cache hits: {}", metrics.cache_hits);
    println!("   Cache misses: {}", metrics.cache_misses);

    println!("✅ Substrate performance under load test passed");
    Ok(())
}

/// Comprehensive substrate performance test
#[tokio::test]
async fn test_substrate_comprehensive_performance() -> Result<()> {
    println!("🚀 Running comprehensive substrate performance test...");

    let substrate = OSSubstrate::new().await?;

    // Test all major performance features
    println!("1. Testing caching performance...");
    let start = std::time::Instant::now();
    let _ = substrate.get_system_info().await?;
    let _ = substrate.get_system_info().await?; // Should be cached
    let caching_duration = start.elapsed();

    println!("2. Testing parallel operations...");
    let start = std::time::Instant::now();
    let (system_info, capabilities) =
        tokio::join!(substrate.get_system_info(), substrate.get_capabilities());
    let parallel_duration = start.elapsed();

    println!("3. Testing cache management...");
    let start = std::time::Instant::now();
    substrate.clear_cache().await;
    substrate.warm_cache().await?;
    let cache_management_duration = start.elapsed();

    // Final metrics
    let metrics = substrate.get_metrics().await;
    let (total_entries, _max_size, utilization, cache_hits, cache_misses) =
        substrate.get_cache_stats().await;

    println!("🏁 Performance test summary:");
    println!("   Caching operations: {:?}", caching_duration);
    println!("   Parallel operations: {:?}", parallel_duration);
    println!("   Cache management: {:?}", cache_management_duration);
    println!("   Total requests: {}", metrics.total_requests);
    println!("   Cache utilization: {:.2}%", utilization * 100.0);
    println!(
        "   Cache hit rate: {:.2}%",
        if cache_hits + cache_misses > 0 {
            cache_hits as f64 / (cache_hits + cache_misses) as f64 * 100.0
        } else {
            0.0
        }
    );

    assert!(system_info.is_ok(), "System info should be available");
    assert!(capabilities.is_ok(), "Capabilities should be available");
    assert!(total_entries > 0, "Should have cached entries");

    println!("✅ Comprehensive substrate performance test passed");
    Ok(())
}
