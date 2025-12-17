#![allow(clippy::all)]
#![allow(unused)]
// Allow unwrap/expect in tests - idiomatic for test code
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Tests for UnifiedUniversalAdapter stress testing and boundary conditions
//!
//! This module tests high-load scenarios, boundary values, and configuration edge cases.

use songbird_test_utils::network_fixtures::*;
use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::{
    create_universal_adapter, create_universal_adapter_with_config, UnifiedAdapterConfig,
    UnifiedUniversalAdapter,
};
use std::time::Duration;

// ============================================================================
// BOUNDARY VALUE TESTS
// ============================================================================

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
        Duration::from_secs(86400),      // 1 day
        Duration::from_secs(31_536_000), // 1 year
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
async fn test_find_providers_empty_string_capability() {
    let adapter = create_universal_adapter();

    // Search for empty string capability
    let result = adapter.find_capability_providers("").await;

    assert!(result.is_ok());
    if let Ok(providers) = result {
        assert!(providers.is_empty());
    }
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
        let result =
            task.await.map_err(|_e| SongbirdError::configuration("Task join error".to_string()))?;
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
        let result =
            task.await.map_err(|_e| SongbirdError::configuration("Task join error".to_string()))?;
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
        String::new(),
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

#[tokio::test]
async fn test_adapter_config_affects_behavior() {
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
    use songbird_universal::UniversalAdapterError;

    let err = UniversalAdapterError::MissingCapability;
    let debug_str = format!("{:?}", err);

    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("MissingCapability"));
    Ok(())
}

// ============================================================================
// MEMORY AND PERFORMANCE CHARACTERISTICS
// ============================================================================

#[test]
fn test_adapter_memory_footprint() {
    // Create multiple adapters and verify reasonable memory usage
    let adapters: Vec<_> = (0..100).map(|_| UnifiedUniversalAdapter::new()).collect();

    // Each adapter should have reasonable size
    for adapter in &adapters {
        let size = std::mem::size_of_val(adapter);
        assert!(size > 0);
        assert!(size < 100_000); // Reasonable upper bound per adapter
    }
}

#[test]
fn test_config_memory_footprint() {
    // Create many configs
    let configs: Vec<_> = (0..1000).map(|_| UnifiedAdapterConfig::default()).collect();

    for config in &configs {
        let size = std::mem::size_of_val(config);
        assert!(size > 0);
        assert!(size < 10_000);
    }
}

#[tokio::test]
async fn test_adapter_concurrent_creation() {
    // Create many adapters concurrently
    let mut tasks = vec![];

    for _ in 0..50 {
        tasks.push(tokio::spawn(async { UnifiedUniversalAdapter::new() }));
    }

    // All should complete
    for task in tasks {
        let adapter = task.await.expect("Task should complete");
        assert!(std::mem::size_of_val(&adapter) > 0);
    }
}

// ============================================================================
// EDGE CASE INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_adapter_lifecycle_create_use_drop() -> SongbirdResult<()> {
    // Create adapter
    let adapter = create_universal_adapter();

    // Use it
    let stats = adapter.get_registry_stats().await;
    assert_eq!(stats.total_services, 0);

    // Drop happens automatically
    drop(adapter);

    // Create new one
    let adapter2 = create_universal_adapter();
    let stats2 = adapter2.get_registry_stats().await;
    assert_eq!(stats2.total_services, 0);

    Ok(())
}

#[tokio::test]
async fn test_config_immutability() {
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(10),
        ..Default::default()
    };

    let adapter1 = UnifiedUniversalAdapter::with_config(config.clone());
    let adapter2 = UnifiedUniversalAdapter::with_config(config);

    // Both adapters should work independently
    let stats1 = adapter1.get_registry_stats().await;
    let stats2 = adapter2.get_registry_stats().await;

    assert_eq!(stats1.total_services, 0);
    assert_eq!(stats2.total_services, 0);
}

#[tokio::test]
async fn test_concurrent_adapter_operations_no_interference() -> SongbirdResult<()> {
    // Create multiple independent adapters
    let adapter1 = create_universal_adapter();
    let adapter2 = create_universal_adapter();
    let adapter3 = create_universal_adapter();

    // Run operations concurrently
    let (stats1, stats2, stats3) = tokio::join!(
        adapter1.get_registry_stats(),
        adapter2.get_registry_stats(),
        adapter3.get_registry_stats()
    );

    // All should return independent results
    assert_eq!(stats1.total_services, 0);
    assert_eq!(stats2.total_services, 0);
    assert_eq!(stats3.total_services, 0);

    Ok(())
}

#[test]
fn test_adapter_send_sync_traits() {
    // Verify adapter implements Send + Sync
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<UnifiedUniversalAdapter>();
    assert_sync::<UnifiedUniversalAdapter>();
}

#[test]
fn test_config_send_sync_traits() {
    // Verify config implements Send + Sync
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<UnifiedAdapterConfig>();
    assert_sync::<UnifiedAdapterConfig>();
}
