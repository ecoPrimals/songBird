// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use songbird_types::SongbirdResult;
use songbird_universal::UnifiedUniversalAdapter;

#[tokio::test]
async fn test_find_capability_providers_empty_registry() -> SongbirdResult<()> {
    // Test finding services when registry is empty

    // ARRANGE: Create fresh adapter with empty registry
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Try to find services with a capability
    let result = adapter.find_capability_providers("compute").await;

    // ASSERT: Should return empty list, not error
    assert!(result.is_ok());
    assert_eq!(result?.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_find_capability_with_special_characters() {
    // Test finding capabilities with special characters in names

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Search for capabilities with special characters
    let result1 = adapter.find_capability_providers("compute-ai").await;
    let result2 = adapter.find_capability_providers("storage/s3").await;
    let result3 = adapter.find_capability_providers("api:v1:health").await;

    // ASSERT: All searches complete without errors
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
}

#[tokio::test]
async fn test_very_long_capability_name() -> SongbirdResult<()> {
    // Edge case: Very long capability names

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Search for capability with very long name
    let long_name = "a".repeat(1000);
    let result = adapter.find_capability_providers(&long_name).await;

    // ASSERT: Handles gracefully
    assert!(result.is_ok());
    assert_eq!(result?.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_find_providers_with_empty_string() -> SongbirdResult<()> {
    // Edge case: Search for providers with empty capability name

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Search with empty string
    let result = adapter.find_capability_providers("").await;

    // ASSERT: Should handle gracefully
    assert!(result.is_ok());
    assert_eq!(result?.len(), 0);
    Ok(())
}

#[tokio::test]
async fn test_find_providers_with_whitespace() {
    // Edge case: Search with whitespace in capability name

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Search with various whitespace patterns
    let result1 = adapter.find_capability_providers(" compute").await;
    let result2 = adapter.find_capability_providers("compute ").await;
    let result3 = adapter.find_capability_providers("  compute  ").await;
    let result4 = adapter.find_capability_providers("com pute").await;

    // ASSERT: All searches complete without errors
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
    assert!(result4.is_ok());
}

#[tokio::test]
async fn test_find_providers_case_sensitivity() {
    // Test capability search with different casing

    // ARRANGE: Create adapter
    let adapter = UnifiedUniversalAdapter::new();

    // ACT: Search with different cases
    let result_lower = adapter.find_capability_providers("compute").await;
    let result_upper = adapter.find_capability_providers("COMPUTE").await;
    let result_mixed = adapter.find_capability_providers("CoMpUtE").await;

    // ASSERT: All searches work (may or may not find results based on implementation)
    assert!(result_lower.is_ok());
    assert!(result_upper.is_ok());
    assert!(result_mixed.is_ok());
}
