// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_universal::{UnifiedAdapterConfig, UnifiedUniversalAdapter};
use std::time::Duration;

#[tokio::test]
async fn test_multiple_adapters_independent() {
    // Test that multiple adapter instances operate independently

    // ARRANGE: Create two adapters with different configs
    let config1 = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(100),
        max_concurrent_requests: 10,
        ..Default::default()
    };
    let config2 = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(200),
        max_concurrent_requests: 20,
        ..Default::default()
    };

    let adapter1 = UnifiedUniversalAdapter::with_config(config1);
    let adapter2 = UnifiedUniversalAdapter::with_config(config2);

    // ACT: Use both adapters concurrently
    let result1 = adapter1.discover_services().await;
    let result2 = adapter2.discover_services().await;

    // ASSERT: Both work independently
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_adapter_clone_behavior() {
    // Test that cloned adapters work correctly

    // ARRANGE: Create adapter and clone it
    let adapter1 = UnifiedUniversalAdapter::new();
    let adapter2 = adapter1.clone();

    // ACT: Use both adapters
    let result1 = adapter1.discover_services().await;
    let result2 = adapter2.discover_services().await;

    // ASSERT: Both work correctly
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_stats_consistency_after_multiple_operations() {
    // Test that stats remain consistent after various operations

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Perform multiple operations
    let _ = adapter.discover_services().await;
    let stats1 = adapter.get_registry_stats().await;

    let _ = adapter.find_capability_providers("compute").await;
    let stats2 = adapter.get_registry_stats().await;

    let _ = adapter.discover_services().await;
    let stats3 = adapter.get_registry_stats().await;

    // ASSERT: Stats are consistent (no services since no real endpoints)
    assert_eq!(stats1.total_services, stats2.total_services);
    assert_eq!(stats2.total_services, stats3.total_services);
}
