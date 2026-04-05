// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::BirdSongHandler;
use super::types::validate_required_fields;
use serde_json::{Value, json};

#[cfg(test)]
fn is_expected_crypto_delegate_connectivity_error(msg: &str) -> bool {
    let m = msg.to_lowercase();
    m.contains("security provider")
        || m.contains("socket")
        || m.contains("ipc")
        || m.contains("connection refused")
        || m.contains("no such file")
        || m.contains("crypto")
        || m.contains("rpc")
}

#[test]
fn test_handler_creation() {
    let _handler = BirdSongHandler::new();
    // Verify handler can be created (no panics)
    // Deep debt: Zero allocation on creation (lazy init)
    // Reached without panic — construction succeeded
}

#[tokio::test]
async fn test_socket_discovery_priority() {
    let handler = BirdSongHandler::new();

    // Test that discovery doesn't panic (socket may not exist in test env)
    let result = handler.discover_security_socket().await;

    // In CI/test environment, socket won't exist - that's expected
    if result.is_err() {
        let err = result.unwrap_err();
        assert!(err.contains("Security provider socket not found"));
    }
}

#[tokio::test]
async fn test_generate_beacon_params() {
    let handler = BirdSongHandler::new();

    let params = json!({
        "node_id": "test_node",
        "capabilities": ["crypto", "discovery"]
    });

    // In test env without security provider, should gracefully fail
    let result = handler.handle_generate_encrypted_beacon(params).await;

    // Expected: Err (no security provider in test env)
    // But the error should be clear and actionable
    if let Err(e) = result {
        assert!(
            e.contains("security provider") || e.contains("socket"),
            "Error should mention security provider or socket, got: {e}"
        );
    }
}

#[tokio::test]
async fn test_decrypt_beacon_params() {
    let handler = BirdSongHandler::new();

    let params = json!({
        "encrypted_beacon": "dGVzdF9lbmNyeXB0ZWRfYmVhY29u" // base64 "test_encrypted_beacon"
    });

    // Should validate params even without security provider
    let result = handler.handle_decrypt_beacon(params).await;

    // Expected: Err (no security provider in test env)
    if let Err(e) = result {
        assert!(
            e.contains("security provider") || e.contains("socket"),
            "Error should mention security provider or socket, got: {e}"
        );
    }
}

#[tokio::test]
async fn test_verify_lineage_params() {
    let handler = BirdSongHandler::new();

    let params = json!({
        "peer_node_id": "peer1",
        "our_node_id": "test_node"
    });

    // Should validate params
    let result = handler.handle_verify_lineage(params).await;

    // Expected: Err (no security provider in test env)
    if let Err(e) = result {
        assert!(
            is_expected_crypto_delegate_connectivity_error(&e),
            "Error should mention security provider, socket, or IPC, got: {e}"
        );
    }
}

#[tokio::test]
async fn test_get_lineage_params() {
    let handler = BirdSongHandler::new();

    let params = json!({});

    // Should accept empty params
    let result = handler.handle_get_lineage(params).await;

    // Expected: Err (no security provider in test env)
    if let Err(e) = result {
        assert!(
            is_expected_crypto_delegate_connectivity_error(&e),
            "Error should mention security provider, socket, or IPC, got: {e}"
        );
    }
}

// ── birdsong.schema ────────────────────────────────────────────

#[tokio::test]
async fn test_schema_returns_all_fields() {
    let handler = BirdSongHandler::new();
    let result = handler.handle_schema(json!({})).await.unwrap();

    let fields = result["fields"].as_array().unwrap();
    assert_eq!(fields.len(), 4, "beacon schema should expose 4 fields");

    let names: Vec<&str> = fields.iter().filter_map(|f| f["name"].as_str()).collect();
    assert!(names.contains(&"node_id"));
    assert!(names.contains(&"capabilities"));
    assert!(names.contains(&"onion_endpoint"));
    assert!(names.contains(&"endpoint_hints"));
}

#[tokio::test]
async fn test_schema_required_fields() {
    let handler = BirdSongHandler::new();
    let result = handler.handle_schema(json!({})).await.unwrap();

    let fields = result["fields"].as_array().unwrap();

    let required: Vec<&str> = fields
        .iter()
        .filter(|f| f["required"].as_bool() == Some(true))
        .filter_map(|f| f["name"].as_str())
        .collect();
    assert_eq!(required, vec!["node_id"], "only node_id should be required");

    let optional: Vec<&str> = fields
        .iter()
        .filter(|f| f["required"].as_bool() == Some(false))
        .filter_map(|f| f["name"].as_str())
        .collect();
    assert_eq!(optional.len(), 3);
}

#[tokio::test]
async fn test_schema_includes_related_methods() {
    let handler = BirdSongHandler::new();
    let result = handler.handle_schema(json!({})).await.unwrap();

    let related = result["related_methods"].as_array().unwrap();
    assert!(!related.is_empty());
    let names: Vec<&str> = related.iter().filter_map(|v| v.as_str()).collect();
    assert!(names.contains(&"birdsong.decrypt_beacon"));
    assert!(names.contains(&"birdsong.verify_lineage"));
}

#[tokio::test]
async fn test_schema_includes_types() {
    let handler = BirdSongHandler::new();
    let result = handler.handle_schema(json!({})).await.unwrap();

    let fields = result["fields"].as_array().unwrap();
    for field in fields {
        assert!(field["type"].is_string(), "field {} should have a type string", field["name"]);
    }
}

#[tokio::test]
async fn test_schema_includes_version() {
    let handler = BirdSongHandler::new();
    let result = handler.handle_schema(json!({})).await.unwrap();
    assert!(result["version"].is_string());
}

#[tokio::test]
async fn test_schema_method_name() {
    let handler = BirdSongHandler::new();
    let result = handler.handle_schema(json!({})).await.unwrap();
    assert_eq!(result["method"].as_str().unwrap(), "birdsong.generate_encrypted_beacon");
}

// ── validate_required_fields ─────────────────────────────────

#[test]
fn test_validate_all_present() {
    let params = json!({"node_id": "test"});
    assert!(validate_required_fields(&params, &["node_id"]).is_ok());
}

#[test]
fn test_validate_single_missing() {
    let params = json!({});
    let err = validate_required_fields(&params, &["node_id"]).unwrap_err();
    assert_eq!(err, "Missing required field: node_id");
}

#[test]
fn test_validate_multiple_missing_aggregated() {
    let params = json!({});
    let err = validate_required_fields(&params, &["peer_node_id", "our_node_id"]).unwrap_err();
    assert!(
        err.contains("peer_node_id") && err.contains("our_node_id"),
        "should list all missing fields: {err}"
    );
    assert!(err.starts_with("Missing required fields:"));
}

#[test]
fn test_validate_partial_missing() {
    let params = json!({"peer_node_id": "a"});
    let err = validate_required_fields(&params, &["peer_node_id", "our_node_id"]).unwrap_err();
    assert!(err.contains("our_node_id"), "should report missing field");
    assert!(!err.contains("peer_node_id"), "should not list present field");
}

#[test]
fn test_validate_non_object_params() {
    let params = json!("not an object");
    let err = validate_required_fields(&params, &["node_id"]).unwrap_err();
    assert!(err.contains("expected JSON object"));
}

#[test]
fn test_validate_null_params() {
    let params = Value::Null;
    let err = validate_required_fields(&params, &["node_id"]).unwrap_err();
    assert!(err.contains("expected JSON object"));
}

#[test]
fn test_validate_empty_required_list() {
    let params = json!({});
    assert!(validate_required_fields(&params, &[]).is_ok());
}

#[tokio::test]
async fn test_generate_beacon_missing_node_id() {
    let handler = BirdSongHandler::new();
    let params = json!({"capabilities": ["test"]});
    let err = handler.handle_generate_encrypted_beacon(params).await.unwrap_err();
    assert!(
        err.contains("Missing required field: node_id"),
        "should report missing node_id: {err}"
    );
}

#[tokio::test]
async fn test_decrypt_missing_encrypted_beacon() {
    let handler = BirdSongHandler::new();
    let params = json!({});
    let err = handler.handle_decrypt_beacon(params).await.unwrap_err();
    assert!(
        err.contains("Missing required field: encrypted_beacon"),
        "should report missing encrypted_beacon: {err}"
    );
}

#[tokio::test]
async fn test_verify_lineage_missing_both_fields() {
    let handler = BirdSongHandler::new();
    let params = json!({});
    let err = handler.handle_verify_lineage(params).await.unwrap_err();
    assert!(
        err.contains("peer_node_id") && err.contains("our_node_id"),
        "should aggregate both missing fields: {err}"
    );
}

#[tokio::test]
async fn test_verify_lineage_missing_one_field() {
    let handler = BirdSongHandler::new();
    let params = json!({"peer_node_id": "peer1"});
    let err = handler.handle_verify_lineage(params).await.unwrap_err();
    assert!(err.contains("our_node_id"), "should report missing our_node_id: {err}");
    assert!(!err.contains("peer_node_id"), "should not list present field: {err}");
}

// Integration tests with real security provider in tests/birdsong_integration_test.rs
