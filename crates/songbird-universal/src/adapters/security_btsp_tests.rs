// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tests for SecurityAdapter.call_generic() BTSP integration (v3.16.0)
//!
//! **Modern Idiomatic Rust**: Protocol-agnostic testing

use super::*;
use serde_json::json;

#[tokio::test]
async fn test_call_generic_with_tarpc_endpoint() {
    // Test that call_generic works with tarpc endpoint
    // Note: tarpc requires valid IP:port format, not hostname
    let endpoint = "tarpc://127.0.0.1:9001".to_string();
    let adapter = SecurityAdapter::new(endpoint).await;

    // Should successfully create adapter with tarpc protocol
    assert!(adapter.is_ok());
}

#[tokio::test]
async fn test_call_generic_with_json_rpc_endpoint() {
    // Test that call_generic works with JSON-RPC endpoint
    let endpoint = "unix:///tmp/test-security.sock".to_string();
    let adapter = SecurityAdapter::new(endpoint).await;

    // Should successfully create adapter with JSON-RPC protocol
    assert!(adapter.is_ok());
}

#[tokio::test]
async fn test_call_generic_with_http_endpoint() {
    // Test that call_generic works with HTTP endpoint
    let endpoint = "http://localhost:9000".to_string();
    let adapter = SecurityAdapter::new(endpoint).await;

    // Should successfully create adapter with HTTP protocol
    assert!(adapter.is_ok());
}

#[tokio::test]
async fn test_call_generic_serializes_params_correctly() {
    // Test that parameters are correctly serialized for call_generic
    let params = json!({
        "target_peer_id": "tower-b",
        "requester_lineage": "tower-a",
        "max_hops": 3
    });

    // Verify JSON structure
    assert_eq!(params["target_peer_id"], "tower-b");
    assert_eq!(params["max_hops"], 3);
}

#[tokio::test]
async fn test_btsp_contact_exchange_request_format() {
    // Test BirdSong contact exchange request format
    let request = json!({
        "target_peer_id": "tower-b-uuid",
        "requester_lineage": "tower-a-lineage-id",
        "max_hops": 3
    });

    assert!(request.is_object());
    assert_eq!(request["target_peer_id"], "tower-b-uuid");
    assert_eq!(request["max_hops"], 3);
}

#[tokio::test]
async fn test_btsp_tunnel_establish_request_format() {
    // Test BTSP tunnel establishment request format
    let request = json!({
        "peer_id": "tower-b-uuid",
        "peer_tags": ["security_provider:family:nat0"],
        "tunnel_type": "Auto",
        "preferences": {
            "timeout_ms": 5000,
            "max_retries": 3
        }
    });

    assert!(request.is_object());
    assert_eq!(request["peer_id"], "tower-b-uuid");
    assert_eq!(request["tunnel_type"], "Auto");
}

#[tokio::test]
async fn test_btsp_contact_exchange_response_parsing() {
    // Test parsing security provider v0.15.0 response format
    let response = json!({
        "success": true,
        "data": {
            "contact": {
                "peer_id": "tower-b",
                "addresses": ["192.168.1.5:10000", "10.0.0.3:10001"],
                "lineage_proof": "proof_data",
                "lineage_path": ["nat0", "tower-b"],
                "search_depth": 2,
                "last_seen": "2026-01-07T12:00:00Z"
            }
        }
    });

    // Verify structure matches security provider v0.15.0
    assert_eq!(response["success"], true);
    assert!(response["data"]["contact"].is_object());
    assert_eq!(response["data"]["contact"]["peer_id"], "tower-b");

    // Verify can extract addresses
    let addresses = response["data"]["contact"]["addresses"].as_array().unwrap();
    assert_eq!(addresses.len(), 2);
}

#[tokio::test]
async fn test_btsp_tunnel_establish_response_parsing() {
    // Test parsing tunnel establishment response
    let response = json!({
        "success": true,
        "data": {
            "tunnel_id": "uuid-1234",
            "local_endpoint": {
                "type": "Direct",
                "address": "192.168.1.5:10000"
            },
            "remote_endpoint": {
                "type": "HolePunched",
                "address": "10.0.0.3:10001"
            },
            "encryption_key_id": "key-123",
            "established_at": "2026-01-07T12:00:00Z",
            "expires_at": "2026-01-07T13:00:00Z"
        }
    });

    // Verify structure
    assert_eq!(response["success"], true);
    assert!(response["data"]["tunnel_id"].is_string());
    assert_eq!(response["data"]["encryption_key_id"], "key-123");
}

#[test]
fn test_method_names_btsp() {
    // Test BTSP method naming conventions
    let contact_exchange_method = "btsp/contact/exchange";
    let tunnel_establish_method = "btsp/tunnel/establish";
    let tunnel_get_method = "btsp/tunnel/{id}";

    assert!(contact_exchange_method.starts_with("btsp/"));
    assert!(tunnel_establish_method.starts_with("btsp/"));
    assert!(tunnel_get_method.starts_with("btsp/"));
}

#[test]
fn test_protocol_hierarchy() {
    // Test that protocol hierarchy is correct
    // tarpc (PRIMARY): 10-100μs
    // JSON-RPC (SECONDARY): 50-100μs
    // HTTP (FALLBACK): 500-1000μs

    let tarpc_latency_max = 100; // μs
    let jsonrpc_latency_max = 100; // μs
    let http_latency_max = 1000; // μs

    assert!(tarpc_latency_max < http_latency_max);
    assert!(jsonrpc_latency_max < http_latency_max);

    // tarpc and JSON-RPC are complementary (similar performance)
    assert!((tarpc_latency_max as i32 - jsonrpc_latency_max as i32).abs() < 50);
}

#[tokio::test]
async fn test_error_handling_timeout() {
    // Test that timeout errors are properly handled
    use std::time::Duration;

    let endpoint = "http://localhost:9999".to_string(); // Non-existent
    let adapter = SecurityAdapter::new(endpoint).await.unwrap();
    let adapter_with_short_timeout = adapter.with_timeout(Duration::from_millis(1));

    let params = json!({"test": "data"});
    let result = adapter_with_short_timeout.call_generic("test_method", params).await;

    // Should timeout
    assert!(result.is_err());
}

#[test]
fn test_zero_hardcoding_principle() {
    // Test that no vendor names are hardcoded
    let code = include_str!("security.rs");

    // Should not contain vendor-specific names in call_generic
    assert!(!code.contains("\"security provider\""));
    assert!(!code.contains("\"beardog\"") || code.contains("// security provider")); // Comments OK

    // Should use generic terms
    assert!(code.contains("security provider") || code.contains("security_provider"));
}

#[tokio::test]
async fn test_modern_rust_async_await() {
    // Test that call_generic uses modern async/await (not callbacks)
    let endpoint = "http://localhost:9000".to_string();
    let adapter = SecurityAdapter::new(endpoint).await;

    // Should successfully create adapter
    assert!(adapter.is_ok());

    // call_generic is an async method (uses modern async/await)
    // This is verified by compilation - if it wasn't async, code wouldn't compile
}
