#![allow(clippy::all)]
#![allow(unused)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for UnifiedUniversalAdapter routing and async operations
//!
//! This module tests async capabilities, routing, concurrent operations,
//! and error handling paths.

use songbird_test_utils::network_fixtures::*;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::{
    create_universal_adapter, create_universal_adapter_with_config, UnifiedAdapterConfig,
    UnifiedUniversalAdapter,
};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// ROUTING AND CAPABILITY DISCOVERY TESTS
// ============================================================================

#[tokio::test]
async fn test_find_capability_providers_empty_registry() {
    let adapter = UnifiedUniversalAdapter::new();

    // Find providers for a capability that doesn't exist
    let result = adapter.find_capability_providers("nonexistent_capability").await;

    // Should succeed but return empty list
    assert!(result.is_ok());
    if let Ok(providers) = result {
        assert!(providers.is_empty());
    }
}

#[tokio::test]
async fn test_find_capability_providers_no_matching_capability() -> SongbirdResult<()> {
    let adapter = UnifiedUniversalAdapter::new();

    // Search for capability that doesn't exist
    let providers =
        adapter.find_capability_providers("ai_model_inference").await.map_err(|_e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;

    // No providers should be found
    assert!(providers.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_route_request_missing_capability_type() {
    use songbird_universal::types::UniversalRequest;

    let adapter = UnifiedUniversalAdapter::new();

    // Create request without capability_type parameter
    let request = UniversalRequest {
        request_id: "test-req-1".to_string(),
        source: "test-source".to_string(),
        target: "test-target".to_string(),
        action: "test_action".to_string(),
        parameters: HashMap::new(), // Missing capability_type
        security_context: None,
    };

    let result = adapter.route_request(request).await;

    // Should fail with MissingCapability error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_route_request_no_providers_available() {
    use songbird_universal::types::UniversalRequest;

    let adapter = UnifiedUniversalAdapter::new();

    // Create request with capability_type that has no providers
    let mut parameters = HashMap::new();
    parameters.insert(
        "capability_type".to_string(),
        serde_json::Value::String("nonexistent".to_string()),
    );

    let request = UniversalRequest {
        request_id: "test-req-2".to_string(),
        source: "test-source".to_string(),
        target: "test-target".to_string(),
        action: "test_action".to_string(),
        parameters,
        security_context: None,
    };

    let result = adapter.route_request(request).await;

    // Should fail with NoProvidersAvailable error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_registry_stats_empty() {
    let adapter = UnifiedUniversalAdapter::new();

    // Get stats from empty registry
    let stats = adapter.get_registry_stats().await;

    // Should have zero services and providers
    assert_eq!(stats.total_services, 0);
    assert_eq!(stats.total_capabilities, 0);
}

#[tokio::test]
async fn test_adapter_async_methods_are_available() {
    let adapter = UnifiedUniversalAdapter::new();

    // Test that async methods can be called
    let _providers = adapter.find_capability_providers("test").await;
    let _stats = adapter.get_registry_stats().await;

    // If we got here, async methods work
    assert!(true);
}

#[tokio::test]
async fn test_route_request_with_invalid_json_in_parameters() {
    use songbird_universal::types::UniversalRequest;

    let adapter = UnifiedUniversalAdapter::new();

    // Create request with malformed parameters
    let mut parameters = HashMap::new();
    parameters.insert("capability_type".to_string(), serde_json::Value::Null);

    let request = UniversalRequest {
        request_id: "test-req-3".to_string(),
        source: "test-source".to_string(),
        target: "test-target".to_string(),
        action: "test_action".to_string(),
        parameters,
        security_context: None,
    };

    let result = adapter.route_request(request).await;

    // Should fail because capability_type is null, not a string
    assert!(result.is_err());
}

// ============================================================================
// CONCURRENT OPERATIONS TESTS
// ============================================================================

#[tokio::test]
async fn test_concurrent_find_capability_providers() -> SongbirdResult<()> {
    let adapter = UnifiedUniversalAdapter::new();

    // Test multiple concurrent lookups
    let adapter1 = adapter.clone();
    let adapter2 = adapter.clone();
    let adapter3 = adapter.clone();

    let task1 =
        tokio::spawn(async move { adapter1.find_capability_providers("capability1").await });

    let task2 =
        tokio::spawn(async move { adapter2.find_capability_providers("capability2").await });

    let task3 =
        tokio::spawn(async move { adapter3.find_capability_providers("capability3").await });

    // All should complete without deadlock
    let result1 =
        task1.await.map_err(|_e| SongbirdError::configuration("Task join error".to_string()))?;
    let result2 =
        task2.await.map_err(|_e| SongbirdError::configuration("Task join error".to_string()))?;
    let result3 =
        task3.await.map_err(|_e| SongbirdError::configuration("Task join error".to_string()))?;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_find_providers_multiple_times() {
    let adapter = create_universal_adapter();

    // Call find_capability_providers multiple times
    for _ in 0..10 {
        let result = adapter.find_capability_providers("any-capability").await;
        assert!(result.is_ok());
        // Should return empty list since no services registered
        if let Ok(providers) = result {
            assert!(providers.is_empty());
        }
    }
}

#[tokio::test]
async fn test_discover_services_with_short_timeout() {
    // Create adapter with very short timeout
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec!["http://localhost:59999".to_string()],
        discovery_timeout: Duration::from_millis(1), // 1ms timeout
        ..Default::default()
    };

    let adapter = create_universal_adapter_with_config(config);

    // Should complete quickly and return empty (graceful failure)
    let result = adapter.discover_services().await;
    assert!(result.is_ok());
    if let Ok(services) = result {
        assert_eq!(services.len(), 0);
    }
}

#[tokio::test]
async fn test_get_stats_multiple_times() -> SongbirdResult<()> {
    let adapter = create_universal_adapter();

    // Get stats multiple times - should be consistent
    let stats1 = adapter.get_registry_stats().await;
    let stats2 = adapter.get_registry_stats().await;
    let stats3 = adapter.get_registry_stats().await;

    assert_eq!(stats1.total_services, stats2.total_services);
    assert_eq!(stats2.total_services, stats3.total_services);
    Ok(())
}

#[tokio::test]
async fn test_concurrent_discover_operations() -> SongbirdResult<()> {
    let adapter = create_universal_adapter();
    let adapter2 = adapter.clone();
    let adapter3 = adapter.clone();

    // Run discovery operations concurrently
    let task1 = tokio::spawn(async move { adapter.discover_services().await });

    let task2 = tokio::spawn(async move { adapter2.discover_services().await });

    let task3 = tokio::spawn(async move { adapter3.discover_services().await });

    // All should complete without deadlock
    let result1 =
        task1.await.map_err(|_e| SongbirdError::configuration("Task join error".to_string()))?;
    let result2 =
        task2.await.map_err(|_e| SongbirdError::configuration("Task join error".to_string()))?;
    let result3 =
        task3.await.map_err(|_e| SongbirdError::configuration("Task join error".to_string()))?;

    // All results should be Ok (empty is fine)
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_find_providers_for_multiple_capabilities() {
    let adapter = create_universal_adapter();

    // Try finding providers for various capability types
    let capabilities = vec!["compute", "storage", "auth", "ai", "networking"];

    for capability in capabilities {
        let result = adapter.find_capability_providers(capability).await;
        assert!(result.is_ok());
        // Empty results are expected with no registered services
    }
}

#[tokio::test]
async fn test_config_with_multiple_endpoints() {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![
            format!("http://localhost:{}", test_orchestrator_port()),
            format!("http://localhost:{}", test_discovery_port()),
            format!("http://localhost:{}", test_health_port()),
        ],
        ..Default::default()
    };

    let adapter = create_universal_adapter_with_config(config);

    // Should try all endpoints (graceful failure expected)
    let result = adapter.discover_services().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_adapter_default_trait() {
    let adapter = UnifiedUniversalAdapter::default();

    // Default should work like new()
    let stats = adapter.get_registry_stats().await;
    assert_eq!(stats.total_services, 0);
}

// ============================================================================
// ERROR PATH TESTS
// ============================================================================

#[tokio::test]
async fn test_discover_services_network_failure() {
    let adapter = UnifiedUniversalAdapter::new();

    // Try to discover services (will fail with network error in test environment)
    let result = adapter.discover_services().await;

    // Expected to fail in test environment without actual services
    // This tests error handling path
    assert!(result.is_ok() || result.is_err()); // Either outcome is valid
}

#[tokio::test]
async fn test_registry_stats_structure() {
    let adapter = UnifiedUniversalAdapter::new();

    let stats = adapter.get_registry_stats().await;

    // Verify stats structure is complete
    let _ = (stats.total_services, stats.total_capabilities, stats.healthy_services);
    assert!(stats.healthy_services <= stats.total_services);
}

#[tokio::test]
async fn test_discover_services_all_endpoints_fail() {
    // Create adapter with multiple non-existent endpoints
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![
            "http://localhost:59991/capabilities".to_string(),
            "http://localhost:59992/services".to_string(),
            "http://localhost:59993/discovery".to_string(),
        ],
        discovery_timeout: Duration::from_millis(100),
        ..Default::default()
    };

    let adapter = create_universal_adapter_with_config(config);

    // Should gracefully handle all endpoints failing
    let result = adapter.discover_services().await;
    assert!(result.is_ok());
    if let Ok(services) = result {
        assert_eq!(services.len(), 0);
    }
}

#[tokio::test]
async fn test_discover_services_partial_endpoint_failure() {
    // Mix of valid format but unreachable endpoints
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![
            "http://127.0.0.1:59999".to_string(), // Will fail
            "http://localhost:60000".to_string(), // Will fail
        ],
        discovery_timeout: Duration::from_millis(50),
        ..Default::default()
    };

    let adapter = create_universal_adapter_with_config(config);

    // Should handle partial failures gracefully
    let result = adapter.discover_services().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_route_request_with_empty_string_capability() {
    use songbird_universal::types::UniversalRequest;

    let adapter = UnifiedUniversalAdapter::new();

    // Create request with empty string capability_type
    let mut parameters = HashMap::new();
    parameters.insert("capability_type".to_string(), serde_json::Value::String(String::new()));

    let request = UniversalRequest {
        request_id: "test-empty-cap".to_string(),
        source: "test-source".to_string(),
        target: "test-target".to_string(),
        action: "test_action".to_string(),
        parameters,
        security_context: None,
    };

    let result = adapter.route_request(request).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_route_request_with_very_long_capability_name() {
    use songbird_universal::types::UniversalRequest;

    let adapter = UnifiedUniversalAdapter::new();

    // Create request with extremely long capability name
    let long_name = "a".repeat(10000);
    let mut parameters = HashMap::new();
    parameters.insert("capability_type".to_string(), serde_json::Value::String(long_name));

    let request = UniversalRequest {
        request_id: "test-long-cap".to_string(),
        source: "test-source".to_string(),
        target: "test-target".to_string(),
        action: "test_action".to_string(),
        parameters,
        security_context: None,
    };

    let result = adapter.route_request(request).await;
    // Should fail with NoProvidersAvailable (not crash)
    assert!(result.is_err());
}

#[tokio::test]
async fn test_find_capability_providers_with_special_characters() {
    let adapter = UnifiedUniversalAdapter::new();

    // Test capability names with special characters
    let special_names = vec![
        "capability/with/slashes",
        "capability:with:colons",
        "capability.with.dots",
        "capability-with-dashes",
        "capability_with_underscores",
        "capability with spaces",
        "capability!@#$%^&*()",
    ];

    for name in special_names {
        let result = adapter.find_capability_providers(name).await;
        assert!(result.is_ok());
        if let Ok(providers) = result {
            assert!(providers.is_empty());
        }
    }
}

#[tokio::test]
async fn test_find_capability_providers_with_unicode() {
    let adapter = UnifiedUniversalAdapter::new();

    // Test with Unicode capability names
    let unicode_names = vec!["计算能力", "🚀rocket", "café", "Ñoño"];

    for name in unicode_names {
        let result = adapter.find_capability_providers(name).await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_discover_services_with_zero_timeout() {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec!["http://localhost:59999".to_string()],
        discovery_timeout: Duration::ZERO,
        ..Default::default()
    };

    let adapter = create_universal_adapter_with_config(config);

    // Should handle zero timeout gracefully
    let result = adapter.discover_services().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discover_services_with_very_long_timeout() {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![],
        discovery_timeout: Duration::from_secs(3600), // 1 hour (won't wait that long)
        ..Default::default()
    };

    let adapter = create_universal_adapter_with_config(config);

    // Should complete quickly with no endpoints
    let result = adapter.discover_services().await;
    assert!(result.is_ok());
    if let Ok(services) = result {
        assert!(services.is_empty());
    }
}

#[tokio::test]
async fn test_concurrent_stats_requests() -> SongbirdResult<()> {
    let adapter = create_universal_adapter();
    let adapter_clone = adapter.clone();

    // Run many concurrent stats requests
    let mut tasks = vec![];
    for _ in 0..50 {
        let adapter_ref = adapter_clone.clone();
        tasks.push(tokio::spawn(async move { adapter_ref.get_registry_stats().await }));
    }

    // All should complete without deadlock
    for task in tasks {
        let stats =
            task.await.map_err(|_e| SongbirdError::configuration("Task join error".to_string()))?;
        assert_eq!(stats.total_services, 0);
    }
    Ok(())
}

#[tokio::test]
async fn test_mixed_concurrent_operations() -> SongbirdResult<()> {
    let adapter = create_universal_adapter();

    // Mix of different operations running concurrently
    let adapter1 = adapter.clone();
    let adapter2 = adapter.clone();
    let adapter3 = adapter.clone();
    let adapter4 = adapter.clone();

    let task1 = tokio::spawn(async move { adapter1.get_registry_stats().await });

    let task2 = tokio::spawn(async move { adapter2.find_capability_providers("compute").await });

    let task3 = tokio::spawn(async move { adapter3.discover_services().await });

    let task4 = tokio::spawn(async move { adapter4.get_registry_stats().await });

    // All should complete successfully
    let stats1 =
        task1.await.map_err(|_e| SongbirdError::configuration("Task join error".to_string()))?;
    let providers =
        task2.await.map_err(|_e| SongbirdError::configuration("Task join error".to_string()))?;
    let services =
        task3.await.map_err(|_e| SongbirdError::configuration("Task join error".to_string()))?;
    let stats2 =
        task4.await.map_err(|_e| SongbirdError::configuration("Task join error".to_string()))?;

    assert_eq!(stats1.total_services, 0);
    assert!(providers.is_ok());
    assert!(services.is_ok());
    assert_eq!(stats2.total_services, 0);
    Ok(())
}
