// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::common::*;

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
        let result = task.await.map_err(|_e| {
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
        let result = task.await.map_err(|_e| {
            UniversalAdapterError::NetworkError("Missing performance configuration".to_string())
        })?;
        assert!(result.is_ok());
    }
    Ok(())
}
