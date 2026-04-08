// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::common::*;

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
    let providers =
        adapter.find_capability_providers("ai_model_inference").await.map_err(|_e| {
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

    let providers = adapter.find_capability_providers("test").await.unwrap();
    let stats = adapter.get_registry_stats().await;

    assert!(providers.is_empty());
    assert_eq!(stats.total_services, 0);
    assert_eq!(stats.total_capabilities, 0);
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
    let result1 = task1.await.map_err(|_e| {
        UniversalAdapterError::NetworkError("Missing performance configuration".to_string())
    })?;
    let result2 = task2.await.map_err(|_e| {
        UniversalAdapterError::NetworkError("Missing performance configuration".to_string())
    })?;
    let result3 = task3.await.map_err(|_e| {
        UniversalAdapterError::NetworkError("Missing performance configuration".to_string())
    })?;

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
    Ok(())
}
