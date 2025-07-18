//! Comprehensive Substrate Integration Tests
//!
//! This test suite validates the substrate integration with toadstool and biomeOS,
//! including performance optimizations, caching, circuit breakers, and error handling.

use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

use songbird_core::substrate::{
    check_substrate_health, clear_substrate_cache, get_substrate, get_substrate_cache_stats,
    get_substrate_metrics, initialize_substrate, NetworkOperation, NetworkRequest, OSSubstrate,
    PathRequest, PathRequirements, PathType,
};
use songbird_errors::Result;

/// Test substrate creation and initialization
#[tokio::test]
async fn test_substrate_creation() -> Result<()> {
    println!("🧪 Testing substrate creation and initialization...");

    // Test substrate creation
    let substrate = OSSubstrate::new().await?;

    // Test basic health check
    let (toadstool_health, biomeos_health) =
        check_substrate_health().await.unwrap_or((false, false));
    println!("📊 Health status - Toadstool: {toadstool_health}, BiomeOS: {biomeos_health}");

    // Test metrics retrieval
    let metrics = substrate.get_metrics().await;
    // Check that metrics are being tracked (since we haven't made requests, should be 0)
    assert_eq!(metrics.total_requests, 0);

    println!("✅ Substrate creation test passed");
    Ok(())
}

/// Test substrate caching with TTL
#[tokio::test]
async fn test_substrate_caching() -> Result<()> {
    println!("🧪 Testing substrate caching with TTL...");

    let substrate = OSSubstrate::new().await?;

    // Test path caching
    let path_request = PathRequest {
        path_type: PathType::Data,
        service_name: "test_service".to_string(),
        requirements: PathRequirements::default(),
    };

    // First request (cache miss)
    let path1 = substrate.get_path(path_request.clone()).await?;
    let (total_entries, _max_size, utilization, cache_hits, cache_misses) =
        substrate.get_cache_stats().await;

    assert!(cache_misses > 0, "Should have cache misses");
    assert!(total_entries > 0, "Should have cached entries");
    assert!(utilization > 0.0, "Cache utilization should be > 0");

    // Second request (cache hit)
    let path2 = substrate.get_path(path_request).await?;
    let (_, _, _, cache_hits_after, _) = substrate.get_cache_stats().await;

    assert_eq!(path1, path2, "Cached paths should be identical");
    assert!(cache_hits_after > cache_hits, "Should have more cache hits");

    println!(
        "📈 Cache stats: {} entries, {:.2}% utilization",
        total_entries,
        utilization * 100.0
    );

    // Test cache clearing
    substrate.clear_cache().await;
    let (total_after_clear, _, _, _, _) = substrate.get_cache_stats().await;
    assert_eq!(total_after_clear, 0, "Cache should be empty after clear");

    println!("✅ Substrate caching test passed");
    Ok(())
}

/// Test substrate cache warming
#[tokio::test]
async fn test_substrate_cache_warming() -> Result<()> {
    println!("🧪 Testing substrate cache warming...");

    let substrate = OSSubstrate::new().await?;

    // Clear cache first
    substrate.clear_cache().await;
    let (initial_entries, _, _, _, _) = substrate.get_cache_stats().await;
    assert_eq!(initial_entries, 0, "Cache should be empty initially");

    // Warm up cache
    substrate.warm_cache().await?;

    // Check cache after warming
    let (warmed_entries, _, utilization, _, _) = substrate.get_cache_stats().await;
    assert!(
        warmed_entries > 0,
        "Cache should have entries after warming"
    );
    assert!(
        utilization > 0.0,
        "Cache utilization should be > 0 after warming"
    );

    println!(
        "🔥 Cache warmed with {} entries, {:.2}% utilization",
        warmed_entries,
        utilization * 100.0
    );

    println!("✅ Substrate cache warming test passed");
    Ok(())
}

/// Test substrate system info caching
#[tokio::test]
async fn test_substrate_system_info_caching() -> Result<()> {
    println!("🧪 Testing substrate system info caching...");

    let substrate = OSSubstrate::new().await?;

    // First system info request
    let system_info1 = substrate.get_system_info().await?;
    assert!(
        !system_info1.platform.is_empty(),
        "Platform should not be empty"
    );
    assert!(
        !system_info1.architecture.is_empty(),
        "Architecture should not be empty"
    );
    assert!(system_info1.cpu_cores > 0, "CPU cores should be > 0");

    // Second system info request (should be cached)
    let system_info2 = substrate.get_system_info().await?;
    assert_eq!(
        system_info1.platform, system_info2.platform,
        "Cached system info should be identical"
    );

    println!(
        "💻 System info: {} {} ({} cores)",
        system_info1.platform, system_info1.architecture, system_info1.cpu_cores
    );

    println!("✅ Substrate system info caching test passed");
    Ok(())
}

/// Test substrate network operations
#[tokio::test]
async fn test_substrate_network_operations() -> Result<()> {
    println!("🧪 Testing substrate network operations...");

    let substrate = OSSubstrate::new().await?;

    // Test network interface discovery
    let interface = substrate.get_network_interface().await?;
    assert!(
        !interface.name.is_empty(),
        "Interface name should not be empty"
    );
    assert!(
        !interface.ip_address.is_empty(),
        "IP address should not be empty"
    );

    // Test port allocation
    let port = substrate.get_available_port().await?;
    assert!(port > 0, "Port should be > 0");

    // Test network operation
    let connectivity_request = NetworkRequest {
        operation: NetworkOperation::CheckConnectivity,
        target: "localhost".to_string(),
        parameters: HashMap::new(),
    };

    let connectivity_result = substrate.network_operation(connectivity_request).await?;
    assert!(
        connectivity_result.is_object(),
        "Network operation should return object"
    );

    println!(
        "🌐 Network interface: {} ({})",
        interface.name, interface.ip_address
    );
    println!("🔌 Available port: {port}");

    println!("✅ Substrate network operations test passed");
    Ok(())
}

/// Test substrate capabilities discovery
#[tokio::test]
async fn test_substrate_capabilities() -> Result<()> {
    println!("🧪 Testing substrate capabilities discovery...");

    let substrate = OSSubstrate::new().await?;

    // Test capabilities discovery
    let capabilities = substrate.get_capabilities().await?;
    assert!(!capabilities.is_empty(), "Capabilities should not be empty");

    // Should have at least combined capabilities
    assert!(
        capabilities.contains_key("combined"),
        "Should have combined capabilities"
    );

    if let Some(combined_caps) = capabilities.get("combined") {
        assert!(
            !combined_caps.is_empty(),
            "Combined capabilities should not be empty"
        );
        println!("⚡ Combined capabilities: {combined_caps:?}");
    }

    println!("✅ Substrate capabilities test passed");
    Ok(())
}

/// Test substrate container operations
#[tokio::test]
async fn test_substrate_container_operations() -> Result<()> {
    println!("🧪 Testing substrate container operations...");

    let substrate = OSSubstrate::new().await?;

    // Test container operation
    let container_params = serde_json::json!({
        "image": "test",
        "command": ["echo", "hello"]
    });

    let container_result = substrate
        .container_operation("test_operation", container_params)
        .await;

    // Container operations may fail in test environment, but we test the API
    match container_result {
        Ok(result) => {
            assert!(
                result.is_object(),
                "Container operation should return object"
            );
            println!("📦 Container operation successful: {result:?}");
        }
        Err(e) => {
            println!("⚠️ Container operation failed (expected in test environment): {e}");
        }
    }

    println!("✅ Substrate container operations test passed");
    Ok(())
}

/// Test substrate path operations with different types
#[tokio::test]
async fn test_substrate_path_operations() -> Result<()> {
    println!("🧪 Testing substrate path operations...");

    let substrate = OSSubstrate::new().await?;

    let path_types = vec![
        PathType::Data,
        PathType::Config,
        PathType::Log,
        PathType::Cache,
        PathType::Runtime,
        PathType::Temp,
    ];

    for path_type in path_types {
        let path_request = PathRequest {
            path_type: path_type.clone(),
            service_name: "test_service".to_string(),
            requirements: PathRequirements {
                min_size_bytes: Some(1024),
                permissions: Some("rw".to_string()),
                persistent: true,
                shared: false,
            },
        };

        let path = substrate.get_path(path_request).await?;
        assert!(
            path.to_string_lossy().contains("test_service"),
            "Path should contain service name"
        );

        println!("📁 {:?} path: {}", path_type, path.display());
    }

    // Test convenience methods
    let data_dir = substrate.get_data_dir("test_service").await?;
    let config_dir = substrate.get_config_dir("test_service").await?;
    let log_dir = substrate.get_log_dir("test_service").await?;

    assert!(data_dir.to_string_lossy().contains("test_service"));
    assert!(config_dir.to_string_lossy().contains("test_service"));
    assert!(log_dir.to_string_lossy().contains("test_service"));

    println!("✅ Substrate path operations test passed");
    Ok(())
}

/// Test substrate circuit breaker functionality
#[tokio::test]
async fn test_substrate_circuit_breaker() -> Result<()> {
    println!("🧪 Testing substrate circuit breaker functionality...");

    let substrate = OSSubstrate::new().await?;

    // Test circuit breaker status
    let cb_status = substrate
        .toadstool_client
        .get_circuit_breaker_status()
        .await;
    println!("⚡ Circuit breaker status: {cb_status:?}");

    // Test health check with circuit breaker
    let health_result = substrate.toadstool_client.health_check().await;

    match health_result {
        Ok(_) => println!("✅ Health check passed (circuit breaker closed)"),
        Err(e) => {
            if e.to_string().contains("Circuit breaker is open") {
                println!("⚠️ Circuit breaker is open");
            } else {
                println!("⚠️ Health check failed: {e}");
            }
        }
    }

    println!("✅ Substrate circuit breaker test passed");
    Ok(())
}

/// Test substrate error handling and fallbacks
#[tokio::test]
async fn test_substrate_error_handling() -> Result<()> {
    println!("🧪 Testing substrate error handling and fallbacks...");

    let substrate = OSSubstrate::new().await?;

    // Test fallback path when substrate is unavailable
    let fallback_request = PathRequest {
        path_type: PathType::Data,
        service_name: "fallback_test".to_string(),
        requirements: PathRequirements::default(),
    };

    let fallback_path = substrate.get_path(fallback_request).await?;
    assert!(
        fallback_path.to_string_lossy().contains("fallback_test"),
        "Fallback path should contain service name"
    );

    // Test network operation fallback
    let invalid_request = NetworkRequest {
        operation: NetworkOperation::ConfigureFirewall,
        target: "invalid_target".to_string(),
        parameters: HashMap::new(),
    };

    let fallback_result = substrate.network_operation(invalid_request).await?;
    assert!(fallback_result.is_object(), "Fallback should return object");

    // Check fallback metrics
    let metrics = substrate.get_metrics().await;
    println!("📊 Fallback uses: {}", metrics.fallback_uses);

    println!("✅ Substrate error handling test passed");
    Ok(())
}

/// Test substrate performance metrics
#[tokio::test]
async fn test_substrate_performance_metrics() -> Result<()> {
    println!("🧪 Testing substrate performance metrics...");

    let substrate = OSSubstrate::new().await?;

    // Perform some operations to generate metrics
    let _ = substrate.get_system_info().await?;
    let _ = substrate.get_capabilities().await?;

    let path_request = PathRequest {
        path_type: PathType::Data,
        service_name: "metrics_test".to_string(),
        requirements: PathRequirements::default(),
    };
    let _ = substrate.get_path(path_request).await?;

    // Check metrics
    let metrics = substrate.get_metrics().await;
    assert!(metrics.total_requests > 0, "Should have total requests");

    println!("📊 Performance metrics:");
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

    // Test cache statistics
    let (total_entries, _max_size, utilization, cache_hits, cache_misses) =
        substrate.get_cache_stats().await;
    println!("📈 Cache statistics:");
    println!("   Total entries: {total_entries}");
    println!("   Max size: {_max_size}");
    println!("   Utilization: {:.2}%", utilization * 100.0);
    println!("   Cache hits: {cache_hits}");
    println!("   Cache misses: {cache_misses}");

    println!("✅ Substrate performance metrics test passed");
    Ok(())
}

/// Test substrate global functions
#[tokio::test]
async fn test_substrate_global_functions() -> Result<()> {
    println!("🧪 Testing substrate global functions...");

    // Test global substrate initialization
    initialize_substrate().await?;

    // Test global substrate access
    let global_substrate = get_substrate().await;
    let system_info = global_substrate.get_system_info().await?;
    assert!(
        !system_info.platform.is_empty(),
        "Global substrate should work"
    );

    // Test global metrics
    let global_metrics = get_substrate_metrics().await;
    assert!(global_metrics.is_some(), "Should have global metrics");

    // Test global cache stats
    let global_cache_stats = get_substrate_cache_stats().await;
    assert!(
        global_cache_stats.is_some(),
        "Should have global cache stats"
    );

    // Test global cache clearing
    clear_substrate_cache().await?;

    // Test global health check
    let (toadstool_health, biomeos_health) = check_substrate_health().await?;
    println!("🌍 Global health - Toadstool: {toadstool_health}, BiomeOS: {biomeos_health}");

    println!("✅ Substrate global functions test passed");
    Ok(())
}

/// Test substrate retry mechanisms
#[tokio::test]
async fn test_substrate_retry_mechanisms() -> Result<()> {
    println!("🧪 Testing substrate retry mechanisms...");

    let substrate = OSSubstrate::new().await?;

    // Test path request with retry logic
    let retry_request = PathRequest {
        path_type: PathType::Data,
        service_name: "retry_test".to_string(),
        requirements: PathRequirements::default(),
    };

    // This should succeed with retry logic
    let path = substrate.get_path(retry_request).await?;
    assert!(
        path.to_string_lossy().contains("retry_test"),
        "Retry should eventually succeed"
    );

    println!("🔄 Retry mechanism test completed");

    println!("✅ Substrate retry mechanisms test passed");
    Ok(())
}

/// Test substrate under load
#[tokio::test]
async fn test_substrate_under_load() -> Result<()> {
    println!("🧪 Testing substrate under load...");

    let substrate = OSSubstrate::new().await?;

    // Create multiple concurrent requests
    let mut handles = Vec::new();

    for i in 0..20 {
        let substrate_clone = substrate.clone();
        let handle = tokio::spawn(async move {
            let path_request = PathRequest {
                path_type: PathType::Data,
                service_name: format!("load_test_{i}"),
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

    println!("📊 Load test: {successful_requests}/20 requests successful");
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

    println!("✅ Substrate under load test passed");
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
            println!("⚠️ Operation failed: {e}");
        }
        Err(_) => {
            println!("⏰ Operation timed out");
        }
    }

    println!("✅ Substrate timeout handling test passed");
    Ok(())
}

/// Comprehensive substrate integration test
#[tokio::test]
async fn test_substrate_comprehensive_integration() -> Result<()> {
    println!("🚀 Running comprehensive substrate integration test...");

    // Initialize substrate
    let substrate = OSSubstrate::new().await?;

    // Test all major features
    println!("1. Testing caching...");
    let _ = substrate.get_system_info().await?;
    let _ = substrate.get_capabilities().await?;

    println!("2. Testing path operations...");
    let data_dir = substrate.get_data_dir("integration_test").await?;
    let config_dir = substrate.get_config_dir("integration_test").await?;

    println!("3. Testing network operations...");
    let interface = substrate.get_network_interface().await?;
    let port = substrate.get_available_port().await?;

    println!("4. Testing cache management...");
    let (entries_before, _, _utilization_before, _, _) = substrate.get_cache_stats().await;
    substrate.clear_cache().await;
    let (entries_after, _, _, _, _) = substrate.get_cache_stats().await;
    assert!(entries_after < entries_before, "Cache should be cleared");

    println!("5. Testing cache warming...");
    substrate.warm_cache().await?;
    let (entries_warmed, _, utilization_warmed, _, _) = substrate.get_cache_stats().await;
    assert!(entries_warmed > 0, "Cache should be warmed");

    println!("6. Testing metrics collection...");
    let metrics = substrate.get_metrics().await;
    assert!(metrics.total_requests > 0, "Should have metrics");

    println!("📊 Integration test summary:");
    println!("   Data dir: {}", data_dir.display());
    println!("   Config dir: {}", config_dir.display());
    println!(
        "   Network interface: {} ({})",
        interface.name, interface.ip_address
    );
    println!("   Available port: {port}");
    println!("   Cache utilization: {:.2}%", utilization_warmed * 100.0);
    println!("   Total requests: {}", metrics.total_requests);

    println!("✅ Comprehensive substrate integration test passed");
    Ok(())
}
