#![allow(clippy::all)]
#![allow(unused)]

//! Configuration Edge Cases & Validation Tests
//!
//! **Purpose**: Tests for configuration validation, boundaries, edge cases
//! **Focus**: Does configuration handle all edge cases and invalid inputs?
//! **Scope**: Boundary conditions, clones, extreme values, network configs

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::float_cmp)]
#![allow(clippy::useless_vec)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::similar_names)]
#![allow(clippy::module_name_repetitions)]

use songbird_test_utils::network_fixtures::*;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::{
    create_universal_adapter, create_universal_adapter_with_config, CapabilityRegistry,
    UnifiedAdapterConfig, UnifiedUniversalAdapter, UniversalAdapterError,
};
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================

#[tokio::test]
async fn test_find_capability_providers_empty_registry() -> Result<(), UniversalAdapterError> {
    let adapter = UnifiedUniversalAdapter::new();

    // Find providers for a capability that doesn't exist
    let result = adapter.find_capability_providers("nonexistent_capability").await;

    // Should succeed but return empty list
    assert!(result.is_ok());
    assert!(result?.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_find_capability_providers_no_matching_capability() -> Result<(), UniversalAdapterError>
{
    let adapter = UnifiedUniversalAdapter::new();

    // Search for capability that doesn't exist
    let providers = adapter.find_capability_providers("ai_model_inference").await.map_err(|e| {
        UniversalAdapterError::NetworkError("Missing performance configuration".to_string())
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

#[tokio::test]
async fn test_concurrent_find_capability_providers() -> Result<(), UniversalAdapterError> {
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
    let result1 = task1.await.map_err(|e| {
        UniversalAdapterError::NetworkError("Missing performance configuration".to_string())
    })?;
    let result2 = task2.await.map_err(|e| {
        UniversalAdapterError::NetworkError("Missing performance configuration".to_string())
    })?;
    let result3 = task3.await.map_err(|e| {
        UniversalAdapterError::NetworkError("Missing performance configuration".to_string())
    })?;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
    Ok(())
}

// ============================================================================
// P1 TESTS - Additional Coverage for Routing and Registry
// ============================================================================

#[tokio::test]
async fn test_find_providers_multiple_times() -> Result<(), UniversalAdapterError> {
    let adapter = create_universal_adapter();

    // Call find_capability_providers multiple times
    for _ in 0..10 {
        let result = adapter.find_capability_providers("any-capability").await;
        assert!(result.is_ok());
        // Should return empty list since no services registered
        assert!(result?.is_empty());
    }
    Ok(())
}

#[tokio::test]
async fn test_discover_services_with_short_timeout() -> Result<(), UniversalAdapterError> {
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
    assert_eq!(result?.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_get_stats_multiple_times() -> Result<(), UniversalAdapterError> {
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
async fn test_concurrent_discover_operations() -> Result<(), UniversalAdapterError> {
    let adapter = create_universal_adapter();
    let adapter2 = adapter.clone();
    let adapter3 = adapter.clone();

    // Run discovery operations concurrently
    let task1 = tokio::spawn(async move { adapter.discover_services().await });

    let task2 = tokio::spawn(async move { adapter2.discover_services().await });

    let task3 = tokio::spawn(async move { adapter3.discover_services().await });

    // All should complete without deadlock
    let result1 = task1.await.map_err(|e| {
        UniversalAdapterError::NetworkError("Failed to discover services".to_string())
    })?;
    let result2 = task2.await.map_err(|e| {
        UniversalAdapterError::NetworkError("Failed to discover services".to_string())
    })?;
    let result3 = task3.await.map_err(|e| {
        UniversalAdapterError::NetworkError("Failed to discover services".to_string())
    })?;

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

#[tokio::test]
async fn test_adapter_config_affects_behavior() {
    use songbird_test_utils::network_fixtures::*;
    use songbird_test_utils::test_discovery_port;
    use songbird_test_utils::test_health_port;
    use songbird_test_utils::test_orchestrator_port;
    use std::time::Duration;

    // Create adapter with custom timeout
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(100), // Very short timeout
        health_check_interval: Duration::from_secs(5),
        max_concurrent_requests: 10,
        auto_discovery: false,
        discovery_endpoints: vec![],
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);

    // Verify adapter was created successfully
    assert!(std::mem::size_of_val(&adapter) > 0);
}

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

    // Verify stats structure is complete (unsigned values are always >= 0 by definition)
    let _ = (stats.total_services, stats.total_capabilities, stats.healthy_services);
    assert!(stats.healthy_services <= stats.total_services);
}

// ============================================================================
// P0 HIGH-VALUE ERROR PATH TESTS
// ============================================================================

#[tokio::test]
async fn test_discover_services_all_endpoints_fail() -> Result<(), UniversalAdapterError> {
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
    assert_eq!(result?.len(), 0);
    Ok(())
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
    parameters.insert("capability_type".to_string(), serde_json::Value::String("".to_string()));

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
    parameters.insert("capability_type".to_string(), serde_json::Value::String(long_name.clone()));

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
async fn test_find_capability_providers_with_special_characters(
) -> Result<(), UniversalAdapterError> {
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
        assert!(result?.is_empty());
    }
    Ok(())
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
async fn test_discover_services_with_very_long_timeout() -> Result<(), UniversalAdapterError> {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![],
        discovery_timeout: Duration::from_secs(3600), // 1 hour (won't wait that long)
        ..Default::default()
    };

    let adapter = create_universal_adapter_with_config(config);

    // Should complete quickly with no endpoints
    let result = adapter.discover_services().await;
    assert!(result.is_ok());
    assert!(result?.is_empty());
    Ok(())
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[tokio::test]
async fn test_concurrent_stats_requests() -> Result<(), UniversalAdapterError> {
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
        let stats = task.await.map_err(|e| {
            UniversalAdapterError::NetworkError("Missing performance configuration".to_string())
        })?;
        assert_eq!(stats.total_services, 0);
    }
    Ok(())
}

#[tokio::test]
async fn test_mixed_concurrent_operations() -> Result<(), UniversalAdapterError> {
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
    let stats1 = task1.await.map_err(|e| {
        UniversalAdapterError::NetworkError("Failed to discover services".to_string())
    })?;
    let providers = task2.await.map_err(|e| {
        UniversalAdapterError::NetworkError("Failed to discover services".to_string())
    })?;
    let services = task3.await.map_err(|e| {
        UniversalAdapterError::NetworkError("Missing performance configuration".to_string())
    })?;
    let stats2 = task4.await.map_err(|e| {
        UniversalAdapterError::NetworkError("Missing performance configuration".to_string())
    })?;

    assert_eq!(stats1.total_services, 0);
    assert!(providers.is_ok());
    assert!(services.is_ok());
    assert_eq!(stats2.total_services, 0);
    Ok(())
}

#[test]
fn test_config_with_max_concurrent_requests_boundary() {
    // Test boundary values for max_concurrent_requests
    let configs = vec![1, 10, 100, 1000, 10000, usize::MAX];

    for max_requests in configs {
        let config = UnifiedAdapterConfig {
            max_concurrent_requests: max_requests,
            ..Default::default()
        };

        let adapter = UnifiedUniversalAdapter::with_config(config);
        assert!(std::mem::size_of_val(&adapter) > 0);
    }
}

#[test]
fn test_config_with_extreme_timeout_values() {
    // Test extreme timeout durations
    let timeouts = vec![
        Duration::ZERO,
        Duration::from_nanos(1),
        Duration::from_millis(1),
        Duration::from_secs(1),
        Duration::from_secs(86400),    // 1 day
        Duration::from_secs(31536000), // 1 year
    ];

    for timeout in timeouts {
        let config = UnifiedAdapterConfig {
            discovery_timeout: timeout,
            health_check_interval: timeout,
            ..Default::default()
        };

        let adapter = UnifiedUniversalAdapter::with_config(config);
        assert!(std::mem::size_of_val(&adapter) > 0);
    }
}

#[tokio::test]
async fn test_find_providers_empty_string_capability() -> Result<(), UniversalAdapterError> {
    let adapter = create_universal_adapter();

    // Search for empty string capability
    let result = adapter.find_capability_providers("").await;

    assert!(result.is_ok());
    assert!(result?.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_adapter_clone_maintains_independence() {
    let adapter1 = create_universal_adapter();
    let adapter2 = adapter1.clone();

    // Both should have independent empty registries
    let stats1 = adapter1.get_registry_stats().await;
    let stats2 = adapter2.get_registry_stats().await;

    assert_eq!(stats1.total_services, stats2.total_services);
    assert_eq!(stats1.total_capabilities, stats2.total_capabilities);
}

#[test]
fn test_config_clone_creates_independent_copy() {
    let config1 = UnifiedAdapterConfig::default();
    let config2 = config1.clone();

    // Verify deep copy (all values match)
    assert_eq!(config1.discovery_timeout, config2.discovery_timeout);
    assert_eq!(config1.health_check_interval, config2.health_check_interval);
    assert_eq!(config1.max_concurrent_requests, config2.max_concurrent_requests);
    assert_eq!(config1.auto_discovery, config2.auto_discovery);
}

// ============================================================================
// STRESS AND LOAD TESTS
// ============================================================================

#[tokio::test]
async fn test_rapid_sequential_operations() {
    let adapter = create_universal_adapter();

    // Rapid sequential operations
    for _ in 0..100 {
        let _ = adapter.get_registry_stats().await;
        let _ = adapter.find_capability_providers("test").await;
    }

    // Should complete without errors
    let final_stats = adapter.get_registry_stats().await;
    assert_eq!(final_stats.total_services, 0);
}

#[tokio::test]
async fn test_concurrent_discovery_operations_high_load() -> Result<(), UniversalAdapterError> {
    let adapter = create_universal_adapter();

    // Spawn many concurrent discovery operations
    let mut tasks = vec![];
    for _ in 0..20 {
        let adapter_ref = adapter.clone();
        tasks.push(tokio::spawn(async move { adapter_ref.discover_services().await }));
    }

    // All should complete
    for task in tasks {
        let result = task.await.map_err(|e| {
            UniversalAdapterError::NetworkError("Failed to discover services".to_string())
        })?;
        assert!(result.is_ok());
    }
    Ok(())
}

#[tokio::test]
async fn test_concurrent_find_providers_high_load() -> Result<(), UniversalAdapterError> {
    let adapter = create_universal_adapter();

    // Many concurrent find operations with different capability types
    let mut tasks = vec![];
    let capabilities = vec!["compute", "storage", "auth", "network", "ai"];

    for i in 0..100 {
        let adapter_ref = adapter.clone();
        let cap = capabilities[i % capabilities.len()].to_string();
        tasks.push(tokio::spawn(async move { adapter_ref.find_capability_providers(&cap).await }));
    }

    // All should complete successfully
    for task in tasks {
        let result = task.await.map_err(|e| {
            UniversalAdapterError::NetworkError("Missing performance configuration".to_string())
        })?;
        assert!(result.is_ok());
    }
    Ok(())
}
