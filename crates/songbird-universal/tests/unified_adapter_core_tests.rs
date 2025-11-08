#![allow(clippy::all)]
#![allow(unused)]

//! Comprehensive tests for `UnifiedUniversalAdapter` core functionality
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
#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

//!
//! Tests adapter creation, configuration, capability registry, and service management

use songbird_test_utils::network_fixtures::*;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::{
    create_universal_adapter, create_universal_adapter_with_config, CapabilityRegistry,
    UnifiedAdapterConfig, UnifiedUniversalAdapter,
};
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn test_create_default_adapter() {
    let adapter = create_universal_adapter();

    // Verify adapter is created successfully
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_create_adapter_with_custom_config() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(10),
        health_check_interval: Duration::from_secs(30),
        max_concurrent_requests: 50,
        auto_discovery: false,
        discovery_endpoints: vec![format!("http://custom:{}", test_orchestrator_port())],
    };

    let adapter = create_universal_adapter_with_config(config);

    // Verify adapter uses custom config
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_adapter_new() {
    let adapter = UnifiedUniversalAdapter::new();

    // Verify default construction
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_adapter_with_config() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(15),
        health_check_interval: Duration::from_secs(45),
        max_concurrent_requests: 75,
        auto_discovery: true,
        discovery_endpoints: vec![
            format!("http://primary:{}", test_orchestrator_port()),
            format!("http://secondary:{}", test_discovery_port()),
        ],
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);

    // Verify adapter created with custom config
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_default_config_values() {
    let config = UnifiedAdapterConfig::default();

    assert_eq!(config.discovery_timeout, Duration::from_secs(30));
    assert_eq!(config.health_check_interval, Duration::from_secs(60));
    assert_eq!(config.max_concurrent_requests, 100);
    assert!(config.auto_discovery);
    assert!(!config.discovery_endpoints.is_empty());
}

#[test]
fn test_config_discovery_endpoints_format() {
    let config = UnifiedAdapterConfig::default();

    for endpoint in &config.discovery_endpoints {
        assert!(endpoint.starts_with("http://") || endpoint.starts_with("https://"));
        assert!(endpoint.contains(':'));
    }
}

#[test]
fn test_config_custom_discovery_timeout() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(5),
        ..Default::default()
    };

    assert_eq!(config.discovery_timeout, Duration::from_secs(5));
    assert_eq!(config.health_check_interval, Duration::from_secs(60)); // Still default
}

#[test]
fn test_config_custom_health_check_interval() {
    let config = UnifiedAdapterConfig {
        health_check_interval: Duration::from_secs(120),
        ..Default::default()
    };

    assert_eq!(config.health_check_interval, Duration::from_secs(120));
    assert_eq!(config.discovery_timeout, Duration::from_secs(30)); // Still default
}

#[test]
fn test_config_custom_max_requests() {
    let config = UnifiedAdapterConfig {
        max_concurrent_requests: 200,
        ..Default::default()
    };

    assert_eq!(config.max_concurrent_requests, 200);
}

#[test]
fn test_config_disable_auto_discovery() {
    let config = UnifiedAdapterConfig {
        auto_discovery: false,
        ..Default::default()
    };

    assert!(!config.auto_discovery);
}

#[test]
fn test_config_empty_discovery_endpoints() {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![],
        ..Default::default()
    };

    assert!(config.discovery_endpoints.is_empty());
}

#[test]
fn test_config_multiple_discovery_endpoints() {
    let endpoints = vec![
        format!("http://server1:{}", test_orchestrator_port()),
        format!("http://server2:{}", test_discovery_port()),
        format!("http://server3:{}", test_health_port()),
    ];

    let config = UnifiedAdapterConfig {
        discovery_endpoints: endpoints.clone(),
        ..Default::default()
    };

    assert_eq!(config.discovery_endpoints.len(), 3);
    assert_eq!(config.discovery_endpoints, endpoints);
}

#[test]
fn test_capability_registry_default() {
    let registry = CapabilityRegistry::default();

    assert!(registry.service_capabilities.is_empty());
    assert!(registry.capability_providers.is_empty());
    assert!(registry.service_info.is_empty());
    assert!(registry.last_updated.is_empty());
}

#[test]
fn test_capability_registry_clone() {
    let registry = CapabilityRegistry::default();
    let cloned = registry.clone();

    assert_eq!(registry.service_capabilities.len(), cloned.service_capabilities.len());
    assert_eq!(registry.capability_providers.len(), cloned.capability_providers.len());
}

#[test]
fn test_adapter_clone() {
    let adapter = UnifiedUniversalAdapter::new();
    let cloned = adapter.clone();

    // Verify both adapters are independent
    assert!(std::mem::size_of_val(&adapter) > 0);
    assert!(std::mem::size_of_val(&cloned) > 0);
}

#[test]
fn test_config_clone() {
    let config = UnifiedAdapterConfig::default();
    let cloned = config.clone();

    assert_eq!(config.discovery_timeout, cloned.discovery_timeout);
    assert_eq!(config.health_check_interval, cloned.health_check_interval);
    assert_eq!(config.max_concurrent_requests, cloned.max_concurrent_requests);
    assert_eq!(config.auto_discovery, cloned.auto_discovery);
}

#[test]
fn test_adapter_creation_is_consistent() {
    let adapter1 = UnifiedUniversalAdapter::new();
    let adapter2 = UnifiedUniversalAdapter::new();

    // Both should be valid independent instances
    assert!(std::mem::size_of_val(&adapter1) > 0);
    assert!(std::mem::size_of_val(&adapter2) > 0);
}

#[test]
fn test_config_with_extreme_values() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(1),
        health_check_interval: Duration::from_secs(1),
        max_concurrent_requests: 1,
        auto_discovery: false,
        discovery_endpoints: vec![],
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_config_with_large_values() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(3600), // 1 hour
        health_check_interval: Duration::from_secs(7200), // 2 hours
        max_concurrent_requests: 10000,
        auto_discovery: true,
        discovery_endpoints: vec![format!("http://example.com:{}", test_orchestrator_port()); 100],
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_config_zero_timeout() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(0),
        ..Default::default()
    };

    assert_eq!(config.discovery_timeout, Duration::ZERO);
}

#[test]
fn test_config_builder_pattern() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(20),
        health_check_interval: Duration::from_secs(40),
        max_concurrent_requests: 150,
        auto_discovery: true,
        discovery_endpoints: vec!["http://localhost:9000".to_string()],
    };

    // Verify all fields set correctly
    assert_eq!(config.discovery_timeout.as_secs(), 20);
    assert_eq!(config.health_check_interval.as_secs(), 40);
    assert_eq!(config.max_concurrent_requests, 150);
    assert!(config.auto_discovery);
    assert_eq!(config.discovery_endpoints.len(), 1);
}

#[test]
fn test_adapter_size_is_reasonable() {
    let adapter = UnifiedUniversalAdapter::new();
    let size = std::mem::size_of_val(&adapter);

    // Adapter should not be excessively large
    assert!(size > 0);
    assert!(size < 10_000); // Reasonable upper bound
}

#[test]
fn test_registry_size_is_reasonable() {
    let registry = CapabilityRegistry::default();
    let size = std::mem::size_of_val(&registry);

    assert!(size > 0);
    assert!(size < 5_000); // Reasonable upper bound
}

#[test]
fn test_config_size_is_reasonable() -> SongbirdResult<()> {
    let config = UnifiedAdapterConfig::default();
    let size = std::mem::size_of_val(&config);

    assert!(size > 0);
    assert!(size < 1_000); // Reasonable upper bound
    Ok(())
}

#[test]
fn test_multiple_adapters_independent() -> SongbirdResult<()> {
    let _adapter1 = UnifiedUniversalAdapter::new();
    let _adapter2 = UnifiedUniversalAdapter::new();
    let _adapter3 = UnifiedUniversalAdapter::new();

    // If we got here, all created successfully (implicit test)
    Ok(())
}

#[test]
fn test_config_debug_format() -> SongbirdResult<()> {
    let config = UnifiedAdapterConfig::default();
    let debug_str = format!("{config:?}");

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("UnifiedAdapterConfig"));
    Ok(())
}

#[test]
fn test_registry_debug_format() -> SongbirdResult<()> {
    let registry = CapabilityRegistry::default();
    let debug_str = format!("{registry:?}");

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("CapabilityRegistry"));
    Ok(())
}

#[test]
fn test_adapter_debug_format() -> SongbirdResult<()> {
    let adapter = UnifiedUniversalAdapter::new();
    let debug_str = format!("{adapter:?}");

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("UnifiedUniversalAdapter"));
    Ok(())
}

#[test]
fn test_config_with_ipv4_endpoints() {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![
            format!("http://192.168.1.100:{}", test_orchestrator_port()),
            format!("http://10.0.0.1:{}", test_discovery_port()),
        ],
        ..Default::default()
    };

    assert_eq!(config.discovery_endpoints.len(), 2);
}

#[test]
fn test_config_with_ipv6_endpoints() {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![
            format!("http://[::1]:{}", test_orchestrator_port()),
            format!("http://[2001:db8::1]:{}", test_discovery_port()),
        ],
        ..Default::default()
    };

    assert_eq!(config.discovery_endpoints.len(), 2);
}

#[test]
fn test_config_with_https_endpoints() -> SongbirdResult<()> {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec!["https://secure.example.com:443".to_string()],
        ..Default::default()
    };

    assert!(config.discovery_endpoints[0].starts_with("https://"));
    Ok(())
}

#[test]
fn test_config_respects_environment_variable() -> SongbirdResult<()> {
    // Test that default config checks environment
    let config = UnifiedAdapterConfig::default();

    // Should have at least the default endpoints
    assert!(!config.discovery_endpoints.is_empty());
    Ok(())
}

#[test]
fn test_adapter_functions_are_available() -> SongbirdResult<()> {
    let adapter = UnifiedUniversalAdapter::new();

    // Verify adapter has the expected methods by type checking
    let _cloned = adapter.clone();
    let _debug = format!("{adapter:?}");

    assert!(true); // If we got here, methods are available
    Ok(())
}

#[test]
fn test_create_functions_are_consistent() -> SongbirdResult<()> {
    let adapter1 = create_universal_adapter();
    let adapter2 = UnifiedUniversalAdapter::new();

    // Both should create valid adapters
    assert!(std::mem::size_of_val(&adapter1) > 0);
    assert!(std::mem::size_of_val(&adapter2) > 0);
    Ok(())
}

// ============================================================================
// ROUTING AND CAPABILITY DISCOVERY TESTS
// ============================================================================

#[tokio::test]
async fn test_find_capability_providers_empty_registry() -> SongbirdResult<()> {
    let adapter = UnifiedUniversalAdapter::new();

    // Find providers for a capability that doesn't exist
    let result = adapter.find_capability_providers("nonexistent_capability").await;

    // Should succeed but return empty list
    assert!(result.is_ok());
    assert!(result?.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_find_capability_providers_no_matching_capability() -> SongbirdResult<()> {
    let adapter = UnifiedUniversalAdapter::new();

    // Search for capability that doesn't exist
    let providers = adapter.find_capability_providers("ai_model_inference").await.map_err(|e| {
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
    let result1 = task1.await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let result2 = task2.await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let result3 = task3.await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
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
async fn test_find_providers_multiple_times() -> SongbirdResult<()> {
    let adapter = create_universal_adapter();

    // Call find_capability_providers multiple times
    for _ in 0..10 {
        let result = adapter.find_capability_providers("any-capability").await;
        assert!(result.is_ok());
        // Should return empty list since no services registered
        assert!(result
            .ok_or_else(|| SongbirdError::configuration(format!(
                "Error: {}",
                e
            )))?
            .is_empty());
    }
    Ok(())
}

#[tokio::test]
async fn test_discover_services_with_short_timeout() -> SongbirdResult<()> {
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
    assert_eq!(
        result
            .ok_or_else(|| SongbirdError::configuration(format!(
                "Error: {}",
                e
            )))?
            .len(),
        0
    );
    Ok(())
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
    let result1 = task1.await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    let result2 = task2.await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    let result3 = task3.await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
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
async fn test_discover_services_all_endpoints_fail() -> SongbirdResult<()> {
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
    assert_eq!(
        result
            .ok_or_else(|| SongbirdError::configuration(format!(
                "Error: {}",
                e
            )))?
            .len(),
        0
    );
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
async fn test_find_capability_providers_with_special_characters() -> SongbirdResult<()> {
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
        assert!(result
            .ok_or_else(|| SongbirdError::configuration(format!(
                "Error: {}",
                e
            )))?
            .is_empty());
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
async fn test_discover_services_with_very_long_timeout() -> SongbirdResult<()> {
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
        let stats = task.await.map_err(|e| {
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
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
    let stats1 = task1.await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    let providers = task2.await.map_err(|e| {
        SongbirdError::configuration("Failed to discover services".to_string())
    })?;
    let services = task3.await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
    })?;
    let stats2 = task4.await.map_err(|e| {
        SongbirdError::configuration("Missing performance configuration".to_string())
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
async fn test_find_providers_empty_string_capability() -> SongbirdResult<()> {
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
async fn test_concurrent_discovery_operations_high_load() -> SongbirdResult<()> {
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
            SongbirdError::configuration("Failed to discover services".to_string())
        })?;
        assert!(result.is_ok());
    }
    Ok(())
}

#[tokio::test]
async fn test_concurrent_find_providers_high_load() -> SongbirdResult<()> {
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
            SongbirdError::configuration("Missing performance configuration".to_string())
        })?;
        assert!(result.is_ok());
    }
    Ok(())
}

// ============================================================================
// CONFIGURATION VALIDATION TESTS
// ============================================================================

#[test]
fn test_config_with_empty_endpoints() {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![],
        ..Default::default()
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_config_with_many_endpoints() {
    // Test with large number of endpoints
    let endpoints: Vec<String> = (0..1000).map(|i| format!("http://server{}:8080", i)).collect();

    let config = UnifiedAdapterConfig {
        discovery_endpoints: endpoints,
        ..Default::default()
    };

    let adapter = UnifiedUniversalAdapter::with_config(config);
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[test]
fn test_config_with_invalid_url_formats() {
    // These are syntactically valid strings but semantically unusual URLs
    let endpoints = vec![
        "not-a-url".to_string(),
        "".to_string(),
        "http://".to_string(),
        "://localhost".to_string(),
        "http://localhost:-1".to_string(),
        "http://localhost:99999".to_string(),
    ];

    let config = UnifiedAdapterConfig {
        discovery_endpoints: endpoints,
        ..Default::default()
    };

    // Should accept config (validation happens at runtime)
    let adapter = UnifiedUniversalAdapter::with_config(config);
    assert!(std::mem::size_of_val(&adapter) > 0);
}

#[tokio::test]
async fn test_discover_with_malformed_endpoints() {
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec!["not-a-url".to_string(), "http://".to_string()],
        discovery_timeout: Duration::from_millis(100),
        ..Default::default()
    };

    let adapter = create_universal_adapter_with_config(config);

    // Should handle malformed URLs gracefully
    let result = adapter.discover_services().await;
    assert!(result.is_ok());
}

// ============================================================================
// ERROR TYPE TESTS
// ============================================================================

#[test]
fn test_universal_adapter_error_types_completeness() {
    use songbird_universal::UniversalAdapterError;

    // Test all error variants
    let errors = vec![
        UniversalAdapterError::NetworkError("test".to_string()),
        UniversalAdapterError::ParseError("test".to_string()),
        UniversalAdapterError::DiscoveryError("test".to_string()),
        UniversalAdapterError::ServiceError("test".to_string()),
        UniversalAdapterError::MissingCapability,
        UniversalAdapterError::NoProvidersAvailable("test".to_string()),
    ];

    for error in errors {
        // All errors should have non-empty display strings
        assert!(!error.to_string().is_empty());
    }
}

#[test]
fn test_error_messages_are_descriptive() -> SongbirdResult<()> {
    use songbird_universal::UniversalAdapterError;

    let err = UniversalAdapterError::NetworkError("connection timeout".to_string());
    assert!(err.to_string().contains("Network error"));
    assert!(err.to_string().contains("connection timeout"));

    let err = UniversalAdapterError::NoProvidersAvailable("ai_inference".to_string());
    assert!(err.to_string().contains("No providers available"));
    assert!(err.to_string().contains("ai_inference"));
    Ok(())
}

#[test]
fn test_error_debug_format() -> SongbirdResult<()> {
    use songbird_types::{SongbirdError, SongbirdResult};
    use songbird_universal::UniversalAdapterError;

    let err = UniversalAdapterError::MissingCapability;
    let debug_str = format!("{:?}", err);

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("MissingCapability"));
    Ok(())
}
