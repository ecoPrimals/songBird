// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::SongbirdResult;
use songbird_universal::{UnifiedAdapterConfig, UnifiedUniversalAdapter};
use std::time::Duration;

#[tokio::test]
async fn test_discover_services_empty_response() -> SongbirdResult<()> {
    // This test documents the expected behavior when discovery returns no services

    // ARRANGE: Create adapter with unreachable endpoint (will fail gracefully)
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(100),
        discovery_endpoints: vec!["http://127.0.0.1:59999/services".to_string()], // Port unlikely to be used
        ..Default::default()
    };
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ACT: Attempt discovery
    let result = adapter.discover_services().await;

    // ASSERT: Should return OK with empty list (graceful degradation)
    assert!(result.is_ok());
    let services = result?;
    assert_eq!(services.len(), 0, "Should return empty list when no services found");
    Ok(())
}

#[tokio::test]
async fn test_discover_services_network_timeout() -> SongbirdResult<()> {
    // Test that discovery properly handles network timeouts

    // ARRANGE: Create adapter with very short timeout
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(1), // Extremely short timeout
        discovery_endpoints: vec!["http://192.0.2.1:8080/services".to_string()], // TEST-NET-1 (non-routable)
        ..Default::default()
    };
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ACT: Attempt discovery (should timeout)
    let result = adapter.discover_services().await;

    // ASSERT: Should handle timeout gracefully
    assert!(result.is_ok(), "Discovery should not panic on timeout");
    let services = result?;
    assert_eq!(services.len(), 0, "Timeout should result in no discovered services");
    Ok(())
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

#[tokio::test]
async fn test_discover_services_graceful_failure_handling() -> SongbirdResult<()> {
    // Test that discovery handles failures gracefully and continues

    // ARRANGE: Create adapter with mix of valid and invalid endpoints
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(100),
        discovery_endpoints: vec![
            "http://127.0.0.1:59997/services".to_string(), // Will fail
            "http://127.0.0.1:59998/services".to_string(), // Will fail
            "http://127.0.0.1:59999/services".to_string(), // Will fail
        ],
        ..Default::default()
    };
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ACT: Attempt discovery
    let result = adapter.discover_services().await;

    // ASSERT: Should succeed with empty list (all endpoints failed gracefully)
    assert!(result.is_ok(), "Discovery should handle all failures gracefully");
    let services = result?;
    assert_eq!(services.len(), 0, "No services expected when all endpoints fail");
    Ok(())
}

// ============================================================================
// PERFORMANCE AND EDGE CASES
// ============================================================================

#[tokio::test]
async fn test_discovery_with_zero_timeout() {
    // Edge case: What happens with zero timeout?

    // ARRANGE: Create config with zero timeout
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_secs(0),
        discovery_endpoints: vec!["http://localhost:8080/services".to_string()],
        ..Default::default()
    };
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ACT: Attempt discovery
    let result = adapter.discover_services().await;

    // ASSERT: Should handle gracefully (immediate timeout)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discovery_with_empty_endpoints() -> SongbirdResult<()> {
    // Edge case: No endpoints configured

    // ARRANGE: Create config with no endpoints
    let config = UnifiedAdapterConfig {
        discovery_endpoints: vec![],
        ..Default::default()
    };
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ACT: Attempt discovery
    let result = adapter.discover_services().await;

    // ASSERT: Should return empty list, not error
    assert!(result.is_ok());
    assert_eq!(result?.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_concurrent_discovery_calls() {
    // Test that multiple concurrent discovery calls don't interfere

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Make multiple concurrent discovery calls
    let results = tokio::join!(
        adapter.discover_services(),
        adapter.discover_services(),
        adapter.discover_services(),
    );

    // ASSERT: All calls complete successfully
    assert!(results.0.is_ok());
    assert!(results.1.is_ok());
    assert!(results.2.is_ok());
}

// ============================================================================
// P1 ADDITIONAL TESTS - Increase Coverage for Discovery Module
// ============================================================================

#[tokio::test]
async fn test_registry_stats_after_failed_discovery() {
    // Test that registry stats are correct even after failed discovery attempts

    // ARRANGE: Create adapter with failing endpoints
    let config = UnifiedAdapterConfig {
        discovery_timeout: Duration::from_millis(50),
        discovery_endpoints: vec!["http://127.0.0.1:59999/services".to_string()],
        ..Default::default()
    };
    let adapter = UnifiedUniversalAdapter::with_config(config);

    // ACT: Attempt discovery
    let _ = adapter.discover_services().await;
    let stats = adapter.get_registry_stats().await;

    // ASSERT: Stats are valid even after failure
    assert_eq!(stats.total_services, 0);
    assert_eq!(stats.healthy_services, 0);
}

#[tokio::test]
async fn test_discover_services_repeated_calls() {
    // Test that repeated discovery calls maintain consistent behavior

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Call discover multiple times
    for i in 0..10 {
        let result = adapter.discover_services().await;

        // ASSERT: Each call succeeds
        assert!(result.is_ok(), "Discovery call {i} should succeed");
    }
}
