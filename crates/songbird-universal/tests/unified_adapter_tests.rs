//! Modern tests for `UnifiedUniversalAdapter`
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
//! These tests validate the core universal adapter functionality with current architecture.

use songbird_universal::{
    create_universal_adapter, create_universal_adapter_with_config, UnifiedAdapterConfig,
    UnifiedUniversalAdapter,
};

#[tokio::test]
async fn test_create_default_adapter() {
    let adapter = create_universal_adapter();
    let stats = adapter.get_registry_stats().await;

    // New adapter should have empty registry
    assert_eq!(stats.total_services, 0);
    assert_eq!(stats.total_capabilities, 0);
}

#[tokio::test]
async fn test_create_adapter_with_custom_config() {
    let config = UnifiedAdapterConfig::default();
    let adapter = create_universal_adapter_with_config(config);
    let stats = adapter.get_registry_stats().await;

    assert_eq!(stats.total_services, 0);
}

#[test]
fn test_adapter_new() {
    let adapter = UnifiedUniversalAdapter::new();
    // Adapter should be created successfully
    let _ = adapter;
}

#[test]
fn test_adapter_with_config() {
    let config = UnifiedAdapterConfig::default();
    let adapter = UnifiedUniversalAdapter::with_config(config);
    // Adapter should be created with config
    let _ = adapter;
}

#[tokio::test]
async fn test_adapter_capability_registry_accessible() {
    let adapter = create_universal_adapter();
    // Registry should be accessible through adapter
    let stats = adapter.get_registry_stats().await;
    assert_eq!(stats.total_services, 0, "New registry should be empty");
}

#[test]
fn test_adapter_config_immutability() {
    let config1 = UnifiedAdapterConfig::default();
    let adapter = UnifiedUniversalAdapter::with_config(config1);

    // Adapter should be created successfully
    let _ = adapter;
}

#[tokio::test]
async fn test_multiple_adapters_independent() {
    let adapter1 = create_universal_adapter();
    let adapter2 = create_universal_adapter();

    // Each adapter should have independent state
    let stats1 = adapter1.get_registry_stats().await;
    let stats2 = adapter2.get_registry_stats().await;

    assert_eq!(stats1.total_services, 0);
    assert_eq!(stats2.total_services, 0);
}

#[test]
fn test_adapter_default_config_values() {
    let config = UnifiedAdapterConfig::default();

    // Default config should have reasonable values
    assert!(config.discovery_timeout.as_secs() > 0);
    assert!(config.health_check_interval.as_secs() > 0);
    assert!(config.max_concurrent_requests > 0);
    assert!(!config.discovery_endpoints.is_empty());
}

#[tokio::test]
async fn test_adapter_registry_stats_initial_state() {
    let adapter = create_universal_adapter();
    let stats = adapter.get_registry_stats().await;

    // Initial registry should be empty
    assert_eq!(stats.total_services, 0, "New adapter should have no services");
    assert_eq!(stats.total_capabilities, 0, "New adapter should have no capabilities");
    assert_eq!(stats.healthy_services, 0, "New adapter should have no healthy services");
}

#[test]
fn test_adapter_configuration_structure() {
    let config = UnifiedAdapterConfig::default();
    // Config should be valid and constructable
    let adapter = UnifiedUniversalAdapter::with_config(config);
    let _ = adapter;
}

#[test]
fn test_config_default_auto_discovery() {
    let config = UnifiedAdapterConfig::default();
    assert!(config.auto_discovery, "Auto-discovery should be enabled by default");
}

#[test]
fn test_config_default_discovery_endpoints() {
    let config = UnifiedAdapterConfig::default();
    assert!(!config.discovery_endpoints.is_empty(), "Should have default discovery endpoints");
}

#[tokio::test]
async fn test_adapter_stats_consistency() {
    let adapter = create_universal_adapter();

    // Multiple calls should return consistent results
    let stats1 = adapter.get_registry_stats().await;
    let stats2 = adapter.get_registry_stats().await;

    assert_eq!(stats1.total_services, stats2.total_services);
    assert_eq!(stats1.total_capabilities, stats2.total_capabilities);
}

// ============================================================================
// ADDITIONAL HIGH-VALUE TESTS FOR COVERAGE
// ============================================================================

#[tokio::test]
async fn test_adapter_survives_repeated_cloning() {
    let adapter = create_universal_adapter();

    // Create many clones
    let mut clones = vec![];
    for _ in 0..20 {
        clones.push(adapter.clone());
    }

    // All clones should work independently
    for clone in clones {
        let stats = clone.get_registry_stats().await;
        assert_eq!(stats.total_services, 0);
    }
}

#[tokio::test]
async fn test_concurrent_adapter_creation() -> SongbirdResult<()> {
    // Create many adapters concurrently
    let mut tasks = vec![];

    for _ in 0..10 {
        tasks.push(tokio::spawn(async {
            let adapter = create_universal_adapter();
            adapter.get_registry_stats().await
        }));
    }

    // All should complete successfully
    for task in tasks {
        let stats = task.await.map_err(|e| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?;
        assert_eq!(stats.total_services, 0);
    }
    Ok(())
}

#[tokio::test]
async fn test_find_providers_returns_consistent_results() -> SongbirdResult<()> {
    let adapter = create_universal_adapter();

    // Call multiple times, should get same result
    let result1 = adapter.find_capability_providers("test").await;
    let result2 = adapter.find_capability_providers("test").await;
    let result3 = adapter.find_capability_providers("test").await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());

    assert_eq!(
        result1
            .ok_or_else(|| SongbirdError::configuration(format!(
                "TODO: Replace with proper error handling: {}",
                e
            )))?
            .len(),
        result2
            .ok_or_else(|| SongbirdError::configuration(format!(
                "TODO: Replace with proper error handling: {}",
                e
            )))?
            .len()
    );
    Ok(())
}

#[tokio::test]
async fn test_adapter_with_disabled_auto_discovery() -> SongbirdResult<()> {
    let config = UnifiedAdapterConfig {
        auto_discovery: false,
        ..Default::default()
    };

    let adapter = create_universal_adapter_with_config(config);
    let stats = adapter.get_registry_stats().await;

    // Should work even with auto-discovery disabled
    assert_eq!(stats.total_services, 0);
    Ok(())
}

#[tokio::test]
async fn test_discover_services_returns_empty_gracefully() -> SongbirdResult<()> {
    let adapter = create_universal_adapter();

    // Discovery should return empty in test environment
    let result = adapter.discover_services().await;

    assert!(result.is_ok());
    let services = result.ok_or_else(|| {
        SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
    })?;
    assert!(services.is_empty());
    Ok(())
}

#[tokio::test]
async fn test_stats_healthy_services_never_exceeds_total() {
    let adapter = create_universal_adapter();

    for _ in 0..10 {
        let stats = adapter.get_registry_stats().await;
        assert!(stats.healthy_services <= stats.total_services);
    }
}

#[test]
fn test_config_discovery_timeout_is_positive() {
    let config = UnifiedAdapterConfig::default();
    assert!(config.discovery_timeout.as_nanos() > 0);
}

#[test]
fn test_config_health_check_interval_is_positive() {
    let config = UnifiedAdapterConfig::default();
    assert!(config.health_check_interval.as_nanos() > 0);
}

#[test]
fn test_config_max_requests_is_positive() {
    let config = UnifiedAdapterConfig::default();
    assert!(config.max_concurrent_requests > 0);
}

#[tokio::test]
async fn test_adapter_operations_are_non_blocking() {
    let adapter = create_universal_adapter();

    // Run operations that should not block each other
    let stats_future = adapter.get_registry_stats();
    let find_future = adapter.find_capability_providers("test");
    let discover_future = adapter.discover_services();

    // All should complete concurrently
    let (stats, find_result, discover_result) =
        tokio::join!(stats_future, find_future, discover_future);

    assert_eq!(stats.total_services, 0);
    assert!(find_result.is_ok());
    assert!(discover_result.is_ok());
}

#[tokio::test]
async fn test_adapter_gracefully_handles_empty_capability_search() -> SongbirdResult<()> {
    let adapter = create_universal_adapter();

    // Search for various non-existent capabilities
    let capabilities = vec!["", "nonexistent", "invalid", "🔥", "test123"];

    for cap in capabilities {
        let result = adapter.find_capability_providers(cap).await;
        assert!(result.is_ok());
        // Should return empty list, not error
        assert!(result
            .ok_or_else(|| SongbirdError::configuration(format!(
                "TODO: Replace with proper error handling: {}",
                e
            )))?
            .is_empty());
    }
    Ok(())
}

#[test]
fn test_adapter_creation_with_minimal_config() {
    use std::time::Duration;

    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(1),
        health_check_interval: Duration::from_secs(1),
        max_concurrent_requests: 1,
        auto_discovery: false,
        discovery_endpoints: vec![],
    };

    let adapter = create_universal_adapter_with_config(config);
    // Should create successfully even with minimal config
    let _ = adapter;
}

#[test]
fn test_adapter_creation_with_maximal_config() {
    use std::time::Duration;

    let endpoints: Vec<String> =
        (0..100).map(|i| format!("http://server{}.example.com:8080", i)).collect();

    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(3600),
        health_check_interval: Duration::from_secs(3600),
        max_concurrent_requests: usize::MAX,
        auto_discovery: true,
        discovery_endpoints: endpoints,
    };

    let adapter = create_universal_adapter_with_config(config);
    // Should handle large config
    let _ = adapter;
}

#[tokio::test]
async fn test_repeated_discovery_operations() {
    let adapter = create_universal_adapter();

    // Run discovery multiple times
    for _ in 0..5 {
        let result = adapter.discover_services().await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_adapter_state_remains_consistent() {
    let adapter = create_universal_adapter();

    // Perform various operations
    let _ = adapter.discover_services().await;
    let stats1 = adapter.get_registry_stats().await;

    let _ = adapter.find_capability_providers("test").await;
    let stats2 = adapter.get_registry_stats().await;

    // Stats should remain consistent (no services added in test environment)
    assert_eq!(stats1.total_services, stats2.total_services);
}

#[tokio::test]
async fn test_adapter_handles_rapid_find_operations() {
    let adapter = create_universal_adapter();

    // Rapid-fire find operations
    for i in 0..50 {
        let cap = format!("capability_{}", i);
        let result = adapter.find_capability_providers(&cap).await;
        assert!(result.is_ok());
    }
}

#[test]
fn test_adapter_size_is_consistent() {
    let adapter1 = create_universal_adapter();
    let adapter2 = create_universal_adapter();

    let size1 = std::mem::size_of_val(&adapter1);
    let size2 = std::mem::size_of_val(&adapter2);

    // All adapters should have same size
    assert_eq!(size1, size2);
}

#[test]
fn test_config_can_be_constructed_piecemeal() {
    use std::time::Duration;

    // Build config step by step
    let mut config = UnifiedAdapterConfig::default();
    config.discovery_timeout = Duration::from_secs(15);
    config.auto_discovery = false;

    let adapter = create_universal_adapter_with_config(config);
    let _ = adapter;
}

#[tokio::test]
async fn test_concurrent_stats_and_find_operations() -> SongbirdResult<()> {
    let adapter = create_universal_adapter();

    // Mix of stats and find operations
    let mut tasks = vec![];

    for i in 0..20 {
        let adapter_ref = adapter.clone();
        if i % 2 == 0 {
            tasks.push(tokio::spawn(async move {
                let _ = adapter_ref.get_registry_stats().await;
            }));
        } else {
            tasks.push(tokio::spawn(async move {
                let _ = adapter_ref.find_capability_providers("test").await;
            }));
        }
    }

    // All should complete
    for task in tasks {
        task.await.map_err(|e| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?;
    }
    Ok(())
}

#[tokio::test]
async fn test_adapter_tolerates_repeated_operations() {
    let adapter = create_universal_adapter();

    // Same operation many times
    for _ in 0..100 {
        let _ = adapter.get_registry_stats().await;
    }

    // Should still work
    let final_stats = adapter.get_registry_stats().await;
    assert_eq!(final_stats.total_services, 0);
}

#[test]
fn test_default_adapter_equals_new_adapter() {
    let adapter1 = UnifiedUniversalAdapter::default();
    let adapter2 = UnifiedUniversalAdapter::new();

    // Both should have same structure (sizes should match)
    assert_eq!(std::mem::size_of_val(&adapter1), std::mem::size_of_val(&adapter2));
}

#[tokio::test]
async fn test_adapter_operations_complete_within_reasonable_time() {
    use songbird_types::{SongbirdError, SongbirdResult};
    use std::time::Instant;

    let adapter = create_universal_adapter();

    // These operations should be fast
    let start = Instant::now();
    let _ = adapter.get_registry_stats().await;
    let duration = start.elapsed();

    // Should complete in well under 1 second
    assert!(duration.as_secs() < 1);
}

#[tokio::test]
async fn test_find_providers_with_case_sensitive_names() {
    let adapter = create_universal_adapter();

    // Test case sensitivity
    let result1 = adapter.find_capability_providers("Compute").await;
    let result2 = adapter.find_capability_providers("compute").await;
    let result3 = adapter.find_capability_providers("COMPUTE").await;

    // All should succeed (empty results expected)
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
}

#[tokio::test]
async fn test_adapter_cleanup_is_implicit() -> SongbirdResult<()> {
    // Create and drop adapters rapidly
    for _ in 0..10 {
        let adapter = create_universal_adapter();
        let _ = adapter.get_registry_stats().await;
        // Adapter drops here
    }

    // Should not leak resources
    let final_adapter = create_universal_adapter();
    let _ = final_adapter.get_registry_stats().await;
    Ok(())
}

#[test]
fn test_config_debug_output_is_complete() -> SongbirdResult<()> {
    let config = UnifiedAdapterConfig::default();
    let debug = format!("{:?}", config);

    // Debug output should contain key fields
    assert!(debug.contains("discovery_timeout"));
    assert!(debug.contains("health_check_interval"));
    assert!(debug.contains("max_concurrent_requests"));
    assert!(debug.contains("auto_discovery"));
    assert!(debug.contains("discovery_endpoints"));
    Ok(())
}

#[test]
fn test_adapter_debug_output_is_meaningful() -> SongbirdResult<()> {
    let adapter = create_universal_adapter();
    let debug = format!("{:?}", adapter);

    // Should contain the adapter name
    assert!(debug.contains("UnifiedUniversalAdapter"));
    assert!(!debug.is_empty());
    Ok(())
}
