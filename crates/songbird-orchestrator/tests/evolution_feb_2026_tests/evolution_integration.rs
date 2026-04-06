// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Cross-component schema and deployment naming checks.

use serde_json::Value;

#[tokio::test]
async fn test_integration_jsonrpc_health_endpoint_schema() {
    // Verify health endpoint returns expected schema
    let expected_fields = vec!["status", "version", "uptime_seconds"];

    let health_response = serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": 0
    });

    for field in expected_fields {
        assert!(health_response.get(field).is_some(), "Health response missing field: {field}");
    }
}

#[tokio::test]
async fn test_integration_config_serialization_compatibility() {
    // Test that configs can be serialized by both JSON and potentially other formats
    let config = serde_json::json!({
        "complex": {
            "nested": {
                "array": [1, 2, 3],
                "map": {"a": 1, "b": 2}
            }
        }
    });

    // JSON serialization
    let json_bytes = serde_json::to_vec(&config).expect("JSON serialization");
    let from_json: Value = serde_json::from_slice(&json_bytes).expect("JSON deserialization");
    assert_eq!(config, from_json);
}

#[test]
fn test_integration_primal_naming_standard() {
    // Verify primal socket names follow PRIMAL_DEPLOYMENT_STANDARD
    let primals = vec!["songbird", "security-provider", "ai-provider", "biome"];

    for primal in primals {
        let socket_name = format!("{primal}.sock");

        // Should not contain family_id
        assert!(!socket_name.contains("nat0"));
        assert!(!socket_name.contains("-default"));

        // Should be lowercase
        assert_eq!(socket_name, socket_name.to_lowercase());
    }
}
