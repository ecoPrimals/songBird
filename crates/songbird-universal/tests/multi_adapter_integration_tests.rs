//! Multi-Adapter Integration Tests
//!
//! Comprehensive tests for scenarios involving multiple adapters,
//! concurrent operations, and adapter interactions

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

use songbird_types::{SongbirdError, SongbirdResult};
use songbird_universal::{
    create_universal_adapter, create_universal_adapter_with_config, UnifiedAdapterConfig,
    UniversalRequest,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Barrier;

// ============================================================================
// MULTIPLE ADAPTERS - INDEPENDENCE TESTS
// ============================================================================

#[tokio::test]
async fn test_two_adapters_independent_registries() {
    // ARRANGE: Create two independent adapters
    let adapter1 = create_universal_adapter();
    let adapter2 = create_universal_adapter();

    // ACT: Get stats from both
    let stats1 = adapter1.get_registry_stats().await;
    let stats2 = adapter2.get_registry_stats().await;

    // ASSERT: Both should have independent empty registries
    assert_eq!(stats1.total_services, 0);
    assert_eq!(stats2.total_services, 0);
}

#[tokio::test]
async fn test_multiple_adapters_concurrent_discovery() {
    // ARRANGE: Create multiple adapters with unreachable endpoints
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec!["http://localhost:65534/discovery".to_string()],
        discovery_timeout: Duration::from_millis(50),
        ..Default::default()
    };

    let adapter1 = create_universal_adapter_with_config(config.clone());
    let adapter2 = create_universal_adapter_with_config(config.clone());
    let adapter3 = create_universal_adapter_with_config(config);

    // ACT: Discover concurrently
    let (result1, result2, result3) = tokio::join!(
        adapter1.discover_services(),
        adapter2.discover_services(),
        adapter3.discover_services()
    );

    // ASSERT: All should succeed independently
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
}

#[tokio::test]
async fn test_adapter_clones_share_state() {
    // ARRANGE: Create adapter and clone it
    let adapter1 = create_universal_adapter();
    let adapter2 = adapter1.clone();

    // ACT: Get stats from both
    let stats1 = adapter1.get_registry_stats().await;
    let stats2 = adapter2.get_registry_stats().await;

    // ASSERT: Clones share the same registry
    assert_eq!(stats1.total_services, stats2.total_services);
    assert_eq!(stats1.total_capabilities, stats2.total_capabilities);
}

#[test]
fn test_ten_adapters_creation() {
    // ARRANGE & ACT: Create ten adapters
    let adapters: Vec<_> = (0..10).map(|_| create_universal_adapter()).collect();

    // ASSERT: All should be created successfully
    assert_eq!(adapters.len(), 10);
}

#[test]
fn test_hundred_adapters_creation() {
    // ARRANGE & ACT: Create one hundred adapters (stress test)
    let adapters: Vec<_> = (0..100).map(|_| create_universal_adapter()).collect();

    // ASSERT: All should be created successfully
    assert_eq!(adapters.len(), 100);
}

// ============================================================================
// CONCURRENT OPERATIONS - SAME ADAPTER
// ============================================================================

#[tokio::test]
async fn test_adapter_concurrent_stat_queries() {
    // ARRANGE: Create adapter
    let adapter = create_universal_adapter();

    // ACT: Query stats concurrently many times
    let handles: Vec<_> = (0..50)
        .map(|_| {
            let adapter_clone = adapter.clone();
            tokio::spawn(async move { adapter_clone.get_registry_stats().await })
        })
        .collect();

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // ASSERT: All should succeed
    assert_eq!(results.len(), 50);
    for result in results {
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_adapter_concurrent_capability_queries() -> SongbirdResult<()> {
    // ARRANGE: Create adapter
    let adapter = create_universal_adapter();

    // ACT: Query capabilities concurrently
    let handles: Vec<_> = (0..100)
        .map(|i| {
            let adapter_clone = adapter.clone();
            let cap_name = format!("capability-{}", i);
            tokio::spawn(async move { adapter_clone.find_capability_providers(&cap_name).await })
        })
        .collect();

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // ASSERT: All should succeed (with empty results)
    assert_eq!(results.len(), 100);
    for result in results {
        assert!(result.is_ok());
        let providers = result.expect("Task join failed")?.ok_or_else(|| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?;
        assert_eq!(providers.len(), 0);
    }
    Ok(())
}

#[tokio::test]
async fn test_adapter_concurrent_route_requests() {
    // ARRANGE: Create adapter
    let adapter = create_universal_adapter();

    // ACT: Route requests concurrently
    let handles: Vec<_> = (0..50)
        .map(|i| {
            let adapter_clone = adapter.clone();
            let request = UniversalRequest {
                request_id: format!("req-{}", i),
                source: "test-client".to_string(),
                target: "test-service".to_string(),
                action: "test-action".to_string(),
                parameters: HashMap::new(),
                security_context: None,
            };
            tokio::spawn(async move { adapter_clone.route_request(request).await })
        })
        .collect();

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // ASSERT: All should complete (may fail, but no panics)
    assert_eq!(results.len(), 50);
    for result in results {
        assert!(result.is_ok(), "Task should not panic");
    }
}

#[tokio::test]
async fn test_adapter_mixed_concurrent_operations() {
    // ARRANGE: Create adapter
    let adapter = create_universal_adapter();

    // ACT: Perform mixed operations concurrently
    let mut discovery_handles = Vec::new();
    let mut capability_handles = Vec::new();
    let mut stats_handles = Vec::new();
    let mut route_handles = Vec::new();

    // Discovery operations
    for _ in 0..25 {
        let adapter_clone = adapter.clone();
        discovery_handles
            .push(tokio::spawn(async move { adapter_clone.discover_services().await }));
    }

    // Capability queries
    for i in 0..25 {
        let adapter_clone = adapter.clone();
        capability_handles.push(tokio::spawn(async move {
            adapter_clone.find_capability_providers(&format!("cap-{}", i)).await
        }));
    }

    // Stat queries
    for _ in 0..25 {
        let adapter_clone = adapter.clone();
        stats_handles.push(tokio::spawn(async move { adapter_clone.get_registry_stats().await }));
    }

    // Route requests
    for i in 0..25 {
        let adapter_clone = adapter.clone();
        let request = UniversalRequest {
            request_id: format!("req-{}", i),
            source: "test".to_string(),
            target: "test".to_string(),
            action: "test".to_string(),
            parameters: HashMap::new(),
            security_context: None,
        };
        route_handles.push(tokio::spawn(async move { adapter_clone.route_request(request).await }));
    }

    // Wait for all operations
    let (discovery_results, capability_results, stats_results, route_results) = tokio::join!(
        futures::future::join_all(discovery_handles),
        futures::future::join_all(capability_handles),
        futures::future::join_all(stats_handles),
        futures::future::join_all(route_handles)
    );

    // ASSERT: All should complete without panics
    assert_eq!(discovery_results.len(), 25);
    assert_eq!(capability_results.len(), 25);
    assert_eq!(stats_results.len(), 25);
    assert_eq!(route_results.len(), 25);

    for result in discovery_results {
        assert!(result.is_ok(), "Discovery should not panic");
    }
    for result in capability_results {
        assert!(result.is_ok(), "Capability query should not panic");
    }
    for result in stats_results {
        assert!(result.is_ok(), "Stats query should not panic");
    }
    for result in route_results {
        assert!(result.is_ok(), "Route request should not panic");
    }
}

// ============================================================================
// SYNCHRONIZED CONCURRENT OPERATIONS
// ============================================================================

#[tokio::test]
async fn test_adapter_synchronized_discovery() -> SongbirdResult<()> {
    // ARRANGE: Create adapter and barrier for synchronization
    let adapter = create_universal_adapter();
    let num_tasks = 10;
    let barrier = Arc::new(Barrier::new(num_tasks));

    // ACT: Start discovery at the same time
    let handles: Vec<_> = (0..num_tasks)
        .map(|_| {
            let adapter_clone = adapter.clone();
            let barrier_clone = barrier.clone();
            tokio::spawn(async move {
                barrier_clone.wait().await; // Wait for all tasks to be ready
                adapter_clone.discover_services().await
            })
        })
        .collect();

    // Wait for all to complete
    let results = futures::future::join_all(handles).await;

    // ASSERT: All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.expect("Task join failed").is_ok());
    }
    Ok(())
}

#[tokio::test]
async fn test_adapter_synchronized_capability_queries() -> SongbirdResult<()> {
    // ARRANGE: Create adapter and barrier
    let adapter = create_universal_adapter();
    let num_tasks = 20;
    let barrier = Arc::new(Barrier::new(num_tasks));

    // ACT: Query same capability simultaneously
    let handles: Vec<_> = (0..num_tasks)
        .map(|_| {
            let adapter_clone = adapter.clone();
            let barrier_clone = barrier.clone();
            tokio::spawn(async move {
                barrier_clone.wait().await;
                adapter_clone.find_capability_providers("test-capability").await
            })
        })
        .collect();

    // Wait for all
    let results = futures::future::join_all(handles).await;

    // ASSERT: All should succeed with consistent results
    let first_result_len = results[0]
        .as_ref()
        .expect("Task join failed")
        .as_ref()
        .ok_or_else(|| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?
        .len();
    for result in &results {
        assert!(result.is_ok());
        let providers = result.as_ref().expect("Task join failed").as_ref().ok_or_else(|| {
            SongbirdError::configuration("TODO: Replace with proper error handling".to_string())
        })?;
        assert_eq!(providers.len(), first_result_len, "All queries should return same result");
    }
    Ok(())
}

// ============================================================================
// ADAPTER LIFECYCLE TESTS
// ============================================================================

#[tokio::test]
async fn test_adapter_sequential_usage() {
    // ARRANGE: Create adapter
    let adapter = create_universal_adapter();

    // ACT: Use adapter sequentially
    let stats1 = adapter.get_registry_stats().await;
    let _discovery = adapter.discover_services().await;
    let _providers = adapter.find_capability_providers("test").await;
    let stats2 = adapter.get_registry_stats().await;

    // ASSERT: Should work consistently
    assert_eq!(stats1.total_services, 0);
    assert_eq!(stats2.total_services, 0); // No real services discovered
}

#[tokio::test]
async fn test_adapter_repeated_discovery_cycles() {
    // ARRANGE: Create adapter with fast timeout
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec!["http://localhost:65534/discovery".to_string()],
        discovery_timeout: Duration::from_millis(50),
        ..Default::default()
    };
    let adapter = create_universal_adapter_with_config(config);

    // ACT: Run discovery multiple times
    for _ in 0..10 {
        let result = adapter.discover_services().await;
        assert!(result.is_ok());
    }

    // ASSERT: Stats should be consistent
    let stats = adapter.get_registry_stats().await;
    assert_eq!(stats.total_services, 0);
}

// ============================================================================
// DIFFERENT CONFIGURATIONS - INTERACTION TESTS
// ============================================================================

#[tokio::test]
async fn test_adapters_with_different_timeouts() {
    // ARRANGE: Create adapters with different timeout configurations
    let config1 = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(10),
        ..Default::default()
    };
    let config2 = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(100),
        ..Default::default()
    };
    let config3 = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(1000),
        ..Default::default()
    };

    let adapter1 = create_universal_adapter_with_config(config1);
    let adapter2 = create_universal_adapter_with_config(config2);
    let adapter3 = create_universal_adapter_with_config(config3);

    // ACT: Get stats from all
    let stats1 = adapter1.get_registry_stats().await;
    let stats2 = adapter2.get_registry_stats().await;
    let stats3 = adapter3.get_registry_stats().await;

    // ASSERT: All should be independent
    assert_eq!(stats1.total_services, 0);
    assert_eq!(stats2.total_services, 0);
    assert_eq!(stats3.total_services, 0);
}

#[tokio::test]
async fn test_adapters_with_different_endpoints() {
    // ARRANGE: Create adapters pointing to different endpoints
    let config1 = UnifiedAdapterConfig {
        discovery_endpoints: vec!["http://localhost:10001/discovery".to_string()],
        discovery_timeout: Duration::from_millis(50),
        ..Default::default()
    };
    let config2 = UnifiedAdapterConfig {
        discovery_endpoints: vec!["http://localhost:10002/discovery".to_string()],
        discovery_timeout: Duration::from_millis(50),
        ..Default::default()
    };

    let adapter1 = create_universal_adapter_with_config(config1);
    let adapter2 = create_universal_adapter_with_config(config2);

    // ACT: Both discover independently
    let (result1, result2) =
        tokio::join!(adapter1.discover_services(), adapter2.discover_services());

    // ASSERT: Both should complete independently
    assert!(result1.is_ok());
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_adapters_with_auto_discovery_enabled_disabled() {
    // ARRANGE: Create adapters with different auto-discovery settings
    let config_enabled = UnifiedAdapterConfig {
        auto_discovery: true,
        ..Default::default()
    };
    let config_disabled = UnifiedAdapterConfig {
        auto_discovery: false,
        ..Default::default()
    };

    let adapter_enabled = create_universal_adapter_with_config(config_enabled);
    let adapter_disabled = create_universal_adapter_with_config(config_disabled);

    // ACT: Get stats
    let stats_enabled = adapter_enabled.get_registry_stats().await;
    let stats_disabled = adapter_disabled.get_registry_stats().await;

    // ASSERT: Both should start with empty registries
    assert_eq!(stats_enabled.total_services, 0);
    assert_eq!(stats_disabled.total_services, 0);
}

// ============================================================================
// STRESS TESTS
// ============================================================================

#[tokio::test]
async fn test_many_adapters_concurrent_operations() {
    // ARRANGE: Create many adapters
    let num_adapters = 20;
    let adapters: Vec<_> = (0..num_adapters).map(|_| create_universal_adapter()).collect();

    // ACT: Perform operations on all adapters concurrently
    let mut handles = Vec::new();
    for adapter in adapters {
        handles.push(tokio::spawn(async move {
            let _stats = adapter.get_registry_stats().await;
            let _discovery = adapter.discover_services().await;
            let _providers = adapter.find_capability_providers("test").await;
        }));
    }

    // Wait for all
    let results = futures::future::join_all(handles).await;

    // ASSERT: All should complete
    assert_eq!(results.len(), num_adapters);
    for result in results {
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_adapter_many_sequential_operations() {
    // ARRANGE: Create adapter
    let adapter = create_universal_adapter();

    // ACT: Perform many operations sequentially
    for i in 0..100 {
        let _stats = adapter.get_registry_stats().await;
        let _providers = adapter.find_capability_providers(&format!("cap-{}", i)).await;
    }

    // ASSERT: Should complete without issues
    let final_stats = adapter.get_registry_stats().await;
    assert_eq!(final_stats.total_services, 0);
}

// ============================================================================
// EDGE CASE INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_adapter_clone_after_operations() {
    // ARRANGE: Create adapter and perform operations
    let adapter1 = create_universal_adapter();
    let _discovery = adapter1.discover_services().await;
    let _providers = adapter1.find_capability_providers("test").await;

    // ACT: Clone after operations
    let adapter2 = adapter1.clone();

    // Get stats from both
    let stats1 = adapter1.get_registry_stats().await;
    let stats2 = adapter2.get_registry_stats().await;

    // ASSERT: Stats should be identical (shared state)
    assert_eq!(stats1.total_services, stats2.total_services);
    assert_eq!(stats1.total_capabilities, stats2.total_capabilities);
}

#[tokio::test]
async fn test_adapter_operations_after_failed_discovery() {
    // ARRANGE: Create adapter with invalid endpoint
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec!["http://invalid-host-xyz:99999/discovery".to_string()],
        discovery_timeout: Duration::from_millis(50),
        ..Default::default()
    };
    let adapter = create_universal_adapter_with_config(config);

    // ACT: Attempt discovery (will fail)
    let discovery_result = adapter.discover_services().await;

    // Continue with other operations
    let providers = adapter.find_capability_providers("test").await;
    let stats = adapter.get_registry_stats().await;

    // ASSERT: Adapter should continue functioning
    assert!(discovery_result.is_ok()); // Returns Ok with empty list
    assert!(providers.is_ok());
    assert_eq!(stats.total_services, 0);
}
