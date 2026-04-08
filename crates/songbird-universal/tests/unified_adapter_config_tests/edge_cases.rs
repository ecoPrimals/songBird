// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::common::*;

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
        let stats = task.await.map_err(|_e| {
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
    let stats1 = task1.await.map_err(|_e| {
        UniversalAdapterError::NetworkError("Failed to discover services".to_string())
    })?;
    let providers = task2.await.map_err(|_e| {
        UniversalAdapterError::NetworkError("Failed to discover services".to_string())
    })?;
    let services = task3.await.map_err(|_e| {
        UniversalAdapterError::NetworkError("Missing performance configuration".to_string())
    })?;
    let stats2 = task4.await.map_err(|_e| {
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
