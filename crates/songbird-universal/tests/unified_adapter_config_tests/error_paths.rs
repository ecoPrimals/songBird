// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::common::*;

// ============================================================================
// P0 HIGH-VALUE ERROR PATH TESTS
// ============================================================================

#[tokio::test]
async fn test_discover_services_all_endpoints_fail() -> Result<(), UniversalAdapterError> {
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
    assert_eq!(result?.len(), 0);
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
    parameters.insert("capability_type".to_string(), serde_json::Value::String(String::new()));

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
async fn test_find_capability_providers_with_special_characters()
-> Result<(), UniversalAdapterError> {
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
        assert!(result?.is_empty());
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
async fn test_discover_services_with_very_long_timeout() -> Result<(), UniversalAdapterError> {
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
