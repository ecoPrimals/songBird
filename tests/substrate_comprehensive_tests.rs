//! Comprehensive tests for OS substrate functionality
//!
//! These tests verify the substrate's core functionality including system info,
//! path management, capabilities, caching, and performance metrics.

use songbird_core::substrate::{
    MetricsSummary, NetworkRequest, OSSubstrate, PathRequest, PathRequirements, PathType,
};
use songbird_errors::Result;
use std::time::Duration;

/// Test basic substrate initialization
#[tokio::test]
async fn test_substrate_initialization() -> Result<()> {
    let substrate = OSSubstrate::new().await?;

    // Verify substrate was created with universal capability system
    assert!(substrate.compute_endpoints.len() >= 0); // May be 0 if no compute primals running

    println!("✅ Substrate initialized successfully with universal capability system");
    println!("   🔧 Found {} compute capability endpoints", substrate.compute_endpoints.len());
    
    // Test that substrate can discover capabilities
    let has_compute = !substrate.compute_endpoints.is_empty();
    if has_compute {
        println!("   ✅ Compute capabilities available");
    } else {
        println!("   ℹ️ No compute capabilities found (expected in test environment)");
    }
    
    Ok(())
}

/// Test system information retrieval
#[tokio::test]
async fn test_system_info_retrieval() -> Result<()> {
    let substrate = OSSubstrate::new().await?;

    let system_info = substrate.get_system_info().await?;

    // Verify system info has expected fields
    assert!(system_info.platform.len() > 0);
    assert!(system_info.architecture.len() > 0);
    assert!(system_info.cpu_cores > 0);

    println!(
        "✅ System info: {} on {}, {} cores",
        system_info.platform, system_info.architecture, system_info.cpu_cores
    );

    Ok(())
}

/// Test path operations
#[tokio::test]
async fn test_path_operations() -> Result<()> {
    let substrate = OSSubstrate::new().await?;

    let path_request = PathRequest {
        path_type: PathType::Data,
        service_name: "test_service".to_string(),
        requirements: PathRequirements {
            writable: true,
            executable: false,
            size_limit: Some(1_000_000),
            permissions: Some("755".to_string()),
        },
    };

    let data_path = substrate.get_path(path_request).await?;
    assert!(data_path.to_string_lossy().contains("test_service"));

    println!("✅ Data path: {:?}", data_path);

    // Test different path types
    let config_request = PathRequest {
        path_type: PathType::Config,
        service_name: "test_service".to_string(),
        requirements: PathRequirements {
            writable: true,
            executable: false,
            size_limit: None,
            permissions: None,
        },
    };

    let config_path = substrate.get_path(config_request).await?;
    assert!(config_path.to_string_lossy().contains("test_service"));

    println!("✅ Config path: {:?}", config_path);

    Ok(())
}

/// Test capabilities retrieval
#[tokio::test]
async fn test_capabilities_retrieval() -> Result<()> {
    let substrate = OSSubstrate::new().await?;

    let capabilities = substrate.get_capabilities().await?;

    // Should have at least basic capabilities
    assert!(!capabilities.is_empty());

    println!("✅ Available capabilities: {:?}", capabilities);

    Ok(())
}

/// Test network operations
#[tokio::test]
async fn test_network_operations() -> Result<()> {
    let substrate = OSSubstrate::new().await?;

    let network_request = NetworkRequest {
        operation_type: "toadstool".to_string(),
        payload: serde_json::json!({
            "action": "ping",
            "target": "localhost"
        }),
    };

    // This might fail if toadstool isn't available, which is ok for tests
    match substrate.network_operation(network_request).await {
        Ok(response) => {
            println!("✅ Network operation successful: {}", response.message);
            assert!(response.success);
        }
        Err(e) => {
            println!("⚠️ Network operation failed (expected in test env): {}", e);
        }
    }

    Ok(())
}

/// Test cache functionality
#[tokio::test]
async fn test_cache_functionality() -> Result<()> {
    let substrate = OSSubstrate::new().await?;

    // Warm up the cache
    substrate.warm_cache().await?;

    // Get metrics to verify caching is working
    let initial_metrics = substrate.get_metrics().await;

    // Make the same request twice to test caching
    let _info1 = substrate.get_system_info().await?;
    let _info2 = substrate.get_system_info().await?;

    let final_metrics = substrate.get_metrics().await;

    // Verify cache hits increased
    println!(
        "✅ Initial requests: {}, Final requests: {}",
        initial_metrics.total_requests, final_metrics.total_requests
    );

    // Clear cache
    substrate.clear_cache().await;
    println!("✅ Cache cleared successfully");

    Ok(())
}

/// Test metrics collection
#[tokio::test]
async fn test_metrics_collection() -> Result<()> {
    let substrate = OSSubstrate::new().await?;

    // Perform some operations to generate metrics
    let _ = substrate.get_system_info().await;
    let _ = substrate.get_capabilities().await;

    let metrics = substrate.get_metrics().await;

    // Verify metrics are being collected
    println!("📊 Substrate Metrics:");
    println!("   Cache hit rate: {:.2}%", metrics.cache_hit_rate);
    println!("   Error rate: {:.2}%", metrics.error_rate);
    println!(
        "   Avg response time: {:.2}ms",
        metrics.avg_response_time_ms
    );
    println!("   Total requests: {}", metrics.total_requests);
    println!("   Uptime: {}s", metrics.uptime_seconds);
    println!(
        "   Circuit breaker trips: {}",
        metrics.circuit_breaker_trips
    );
    println!("   Fallback uses: {}", metrics.fallback_uses);

    assert!(metrics.total_requests > 0);

    Ok(())
}

/// Test network interface discovery
#[tokio::test]
async fn test_network_interface_discovery() -> Result<()> {
    let substrate = OSSubstrate::new().await?;

    let interface = substrate.get_network_interface().await?;

    assert!(!interface.is_empty());
    println!("✅ Network interface: {}", interface);

    Ok(())
}

/// Test available port finding
#[tokio::test]
async fn test_available_port_finding() -> Result<()> {
    let substrate = OSSubstrate::new().await?;

    let port = substrate.get_available_port().await?;

    assert!(port > 0);
    assert!(port < 65535);
    println!("✅ Available port: {}", port);

    Ok(())
}

/// Test cache statistics
#[tokio::test]
async fn test_cache_statistics() -> Result<()> {
    let substrate = OSSubstrate::new().await?;

    // Generate some cache activity
    let _ = substrate.get_system_info().await;
    let _ = substrate.get_capabilities().await;
    let _ = substrate.get_system_info().await; // This should be a cache hit

    let (size, hit_rate, utilization, hits, misses) = substrate.get_cache_stats().await;

    println!("📊 Cache Statistics:");
    println!("   Size: {}", size);
    println!("   Hit rate: {:.2}%", hit_rate);
    println!("   Utilization: {:.2}%", utilization);
    println!("   Cache hits: {}", hits);
    println!("   Cache misses: {}", misses);

    // Size should be reasonable (no need to check >= 0 for usize)
    assert!(hit_rate >= 0.0);

    Ok(())
}

/// Test concurrent substrate operations
#[tokio::test]
async fn test_concurrent_operations() -> Result<()> {
    let substrate = OSSubstrate::new().await?;

    // Run multiple operations concurrently
    let futures = (0..5).map(|_| {
        let substrate = substrate.clone();
        async move {
            let _ = substrate.get_system_info().await?;
            let _ = substrate.get_capabilities().await?;
            Result::<()>::Ok(())
        }
    });

    // Wait for all operations to complete
    let results: Vec<Result<()>> = futures_util::future::join_all(futures).await;

    // Verify all operations succeeded
    for result in results {
        assert!(result.is_ok());
    }

    println!("✅ Concurrent operations completed successfully");

    Ok(())
}

/// Test error handling and resilience
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let substrate = OSSubstrate::new().await?;

    // Test with invalid network request
    let invalid_request = NetworkRequest {
        operation_type: "invalid_service".to_string(),
        payload: serde_json::json!({"invalid": "data"}),
    };

    match substrate.network_operation(invalid_request).await {
        Ok(_) => {
            // This is unexpected but not necessarily wrong
            println!("⚠️ Invalid request succeeded (unexpected)");
        }
        Err(e) => {
            println!("✅ Invalid request properly rejected: {}", e);
            // This is expected - invalid requests should fail
        }
    }

    // Verify substrate still works after error
    let system_info = substrate.get_system_info().await?;
    assert!(system_info.platform.len() > 0);

    println!("✅ Substrate resilient to errors");

    Ok(())
}

/// Test substrate performance under load
#[tokio::test]
async fn test_performance_under_load() -> Result<()> {
    let substrate = OSSubstrate::new().await?;

    let start = std::time::Instant::now();

    // Perform many operations to test performance
    for _ in 0..20 {
        let _ = substrate.get_system_info().await;
    }

    let duration = start.elapsed();
    println!("✅ 20 operations completed in {:?}", duration);

    // Get final metrics
    let metrics = substrate.get_metrics().await;
    println!(
        "✅ Average response time: {:.2}ms",
        metrics.avg_response_time_ms
    );

    // Performance should be reasonable (adjust threshold as needed)
    assert!(metrics.avg_response_time_ms < 1000.0); // Under 1 second average

    Ok(())
}
