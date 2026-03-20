// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unnecessary_wraps,
    clippy::await_holding_lock,
    clippy::float_cmp,
    clippy::absurd_extreme_comparisons,
    clippy::needless_collect,
    clippy::nonminimal_bool,
    clippy::used_underscore_binding,
    clippy::field_reassign_with_default,
    clippy::return_self_not_must_use,
    clippy::overly_complex_bool_expr,
    clippy::assertions_on_constants,
    clippy::no_effect_underscore_binding,
    clippy::items_after_statements,
    clippy::empty_line_after_doc_comments,
    clippy::const_is_empty,
    clippy::duplicated_attributes,
    deprecated,
    dead_code,
    clippy::unnecessary_literal_unwrap,
    clippy::needless_pass_by_value,
    clippy::must_use_candidate
)]

//! Dark Forest Protocol Integration Tests
//!
//! Comprehensive tests for all 6 Dark Forest JSON-RPC methods
//! through the IpcServiceHandler wiring (no BearDog required).
//!
//! Tests validate:
//! - Method routing (correct handler called)
//! - Parameter parsing (JSON-RPC params)
//! - Response structure (correct format)
//! - Error handling (missing params, invalid data)
//! - End-to-end wiring (bin_interface.rs fix)
//!
//! Version: v8.19.0
//! Date: January 29, 2026

use serde_json::json;
use songbird_universal_ipc::registry::ServiceRegistry;
use songbird_universal_ipc::service::IpcServiceHandler;
use songbird_universal_ipc::tower_atomic::JsonRpcHandler;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Helper to create a test handler
fn create_test_handler() -> IpcServiceHandler {
    let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
    IpcServiceHandler::new(registry)
}

// ============================================================================
// STUN Methods (Phase 1)
// ============================================================================

#[tokio::test]
async fn test_stun_get_public_address_routing() {
    let handler = create_test_handler();

    // Test that method routes correctly (not "Unknown method")
    let params = json!({});
    let result = handler.handle("stun.get_public_address", params).await;

    // Should return result or specific error, not "Unknown method"
    match result {
        Ok(_) => {
            // Success case - method handled
            assert!(true);
        }
        Err(e) => {
            // Allow specific errors, but not "Unknown method"
            assert!(!e.contains("Unknown method"), "Method not routed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_stun_get_public_address_with_params() {
    let handler = create_test_handler();

    let params = json!({
        "server": "stun.nextcloud.com:3478",
        "local_port": 54321
    });
    let result = handler.handle("stun.get_public_address", params).await;

    // Should handle params without "Unknown method" error
    match result {
        Ok(response) => {
            // Validate response structure
            assert!(response.is_object(), "Response should be JSON object");
            let obj = response.as_object().unwrap();

            // Expected fields (may be present or error)
            let has_address_fields = obj.contains_key("public_address")
                || obj.contains_key("local_address")
                || obj.contains_key("error");

            assert!(has_address_fields, "Response should have address or error fields");
        }
        Err(e) => {
            // Specific error OK, but not "Unknown method"
            assert!(!e.contains("Unknown method"), "Method should be routed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_stun_bind_routing() {
    let handler = create_test_handler();

    let params = json!({
        "stun_server": "stun.nextcloud.com:3478"
    });
    let result = handler.handle("stun.bind", params).await;

    match result {
        Ok(response) => {
            assert!(response.is_object());
            // handle_bind returns: local_address, public_address, public_ip,
            // public_port, nat_type, stun_server
            let obj = response.as_object().unwrap();
            let has_bind_fields = obj.contains_key("public_address")
                || obj.contains_key("local_address")
                || obj.contains_key("nat_type");
            assert!(has_bind_fields, "Bind response should have address/NAT fields");
        }
        Err(e) => {
            // Real STUN request may fail in CI/test environments — accept
            // network errors but not routing errors
            assert!(!e.contains("Unknown method"), "stun.bind should be routed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_stun_bind_missing_params() {
    let handler = create_test_handler();

    // Missing required params
    let params = json!({});
    let result = handler.handle("stun.bind", params).await;

    // Should return specific error about missing params, not "Unknown method"
    match result {
        Ok(_) => {
            // OK if it has default behavior
            assert!(true);
        }
        Err(e) => {
            assert!(!e.contains("Unknown method"), "Should be routed even with bad params");
            // Handler uses defaults for missing params and attempts STUN request,
            // which may timeout or fail with a connection error — both are valid
            assert!(
                e.contains("server")
                    || e.contains("local_port")
                    || e.contains("parameter")
                    || e.contains("timeout")
                    || e.contains("STUN"),
                "Error should mention missing params or STUN failure: {}",
                e
            );
        }
    }
}

// ============================================================================
// Discovery Methods (Phase 1)
// ============================================================================

#[tokio::test]
async fn test_discovery_peers_routing() {
    let handler = create_test_handler();

    let params = json!({});
    let result = handler.handle("discovery.peers", params).await;

    match result {
        Ok(response) => {
            assert!(response.is_object());
            let obj = response.as_object().unwrap();

            // Should have peers array and count
            assert!(obj.contains_key("peers"), "Should have 'peers' field");
            assert!(obj.contains_key("total_count"), "Should have 'total_count' field");

            // Validate structure
            assert!(obj["peers"].is_array(), "peers should be array");
            assert!(obj["total_count"].is_number(), "total_count should be number");
        }
        Err(e) => {
            assert!(!e.contains("Unknown method"), "discovery.peers should be routed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_discovery_peers_returns_empty_initially() {
    let handler = create_test_handler();

    let params = json!({});
    let result = handler.handle("discovery.peers", params).await;

    // Should succeed with empty list initially
    assert!(result.is_ok(), "discovery.peers should work: {:?}", result);

    let response = result.unwrap();
    let peers = response["peers"].as_array().unwrap();
    let count = response["total_count"].as_u64().unwrap();

    assert_eq!(peers.len(), count as usize, "Peers length should match total_count");
}

// ============================================================================
// Rendezvous Methods (Phase 2)
// ============================================================================

#[tokio::test]
async fn test_rendezvous_register_routing() {
    let handler = create_test_handler();

    let params = json!({
        "server": "http://relay.example.com",
        "node_id": "test-node",
        "family_id": "nat0",
        "public_address": "1.2.3.4:5678"
    });
    let result = handler.handle("rendezvous.register", params).await;

    match result {
        Ok(response) => {
            assert!(response.is_object());
            let obj = response.as_object().unwrap();

            // Expected registration fields
            let has_reg_fields = obj.contains_key("registration_id")
                || obj.contains_key("expires_at")
                || obj.contains_key("rendezvous_token")
                || obj.contains_key("error");

            assert!(has_reg_fields, "Registration should have expected fields");
        }
        Err(e) => {
            assert!(!e.contains("Unknown method"), "rendezvous.register should be routed: {}", e);
            // Allow "not yet implemented", "not configured", or parameter errors
            assert!(
                e.contains("not yet")
                    || e.contains("implement")
                    || e.contains("parameter")
                    || e.contains("not configured")
                    || e.contains("Rendezvous"),
                "Error should be specific: {}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_rendezvous_register_missing_params() {
    let handler = create_test_handler();

    let params = json!({
        "server": "http://relay.example.com"
        // Missing node_id, family_id, public_address
    });
    let result = handler.handle("rendezvous.register", params).await;

    match result {
        Err(e) => {
            assert!(!e.contains("Unknown method"), "Should be routed");
            // Should mention missing parameters
            assert!(
                e.contains("node_id")
                    || e.contains("family_id")
                    || e.contains("public_address")
                    || e.contains("parameter"),
                "Should mention missing params: {}",
                e
            );
        }
        Ok(_) => {
            // OK if it has defaults
            assert!(true);
        }
    }
}

#[tokio::test]
async fn test_rendezvous_lookup_routing() {
    let handler = create_test_handler();

    let params = json!({
        "server": "http://relay.example.com",
        "target": "test-node"
    });
    let result = handler.handle("rendezvous.lookup", params).await;

    match result {
        Ok(response) => {
            assert!(response.is_object());
            let obj = response.as_object().unwrap();

            // Should have peers array
            assert!(obj.contains_key("peers"), "Lookup should return peers array");
            assert!(obj["peers"].is_array(), "peers should be array");
        }
        Err(e) => {
            assert!(!e.contains("Unknown method"), "rendezvous.lookup should be routed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_rendezvous_lookup_returns_empty_for_unknown() {
    let handler = create_test_handler();

    let params = json!({
        "server": "http://relay.example.com",
        "target": "nonexistent-node-12345"
    });
    let result = handler.handle("rendezvous.lookup", params).await;

    // Should succeed with empty list
    assert!(result.is_ok(), "Lookup should work even for unknown targets");

    let response = result.unwrap();
    let peers = response["peers"].as_array().unwrap();

    // Should be empty for nonexistent target
    assert_eq!(peers.len(), 0, "Should return empty list for unknown target");
}

// ============================================================================
// Peer Connection Methods (Phase 2)
// ============================================================================

#[tokio::test]
async fn test_peer_connect_routing() {
    let handler = create_test_handler();

    let params = json!({
        "target_address": "1.2.3.4:5678"
    });
    let result = handler.handle("peer.connect", params).await;

    match result {
        Ok(response) => {
            assert!(response.is_object());
            let obj = response.as_object().unwrap();

            // Expected connection fields
            let has_conn_fields = obj.contains_key("connection_id")
                || obj.contains_key("state")
                || obj.contains_key("channel")
                || obj.contains_key("error");

            assert!(has_conn_fields, "Connect should have connection fields");
        }
        Err(e) => {
            assert!(!e.contains("Unknown method"), "peer.connect should be routed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_peer_connect_with_optional_params() {
    let handler = create_test_handler();

    let params = json!({
        "target_address": "1.2.3.4:5678",
        "our_binding": "binding-123",
        "rendezvous_token": "token-456"
    });
    let result = handler.handle("peer.connect", params).await;

    match result {
        Ok(response) => {
            assert!(response.is_object());
            // Validate state field
            if let Some(state) = response["state"].as_str() {
                assert!(
                    ["connecting", "connected", "failed"].contains(&state),
                    "State should be valid: {}",
                    state
                );
            }
        }
        Err(e) => {
            assert!(!e.contains("Unknown method"), "Should be routed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_peer_connect_missing_target() {
    let handler = create_test_handler();

    let params = json!({});
    let result = handler.handle("peer.connect", params).await;

    match result {
        Err(e) => {
            assert!(!e.contains("Unknown method"), "Should be routed");
            assert!(
                e.contains("target_address") || e.contains("parameter"),
                "Should mention missing target: {}",
                e
            );
        }
        Ok(_) => {
            // OK if it has default
            assert!(true);
        }
    }
}

// ============================================================================
// Cross-Method Integration Tests
// ============================================================================

#[tokio::test]
async fn test_all_six_methods_route_correctly() {
    let handler = create_test_handler();

    let methods = vec![
        ("stun.get_public_address", json!({})),
        ("stun.bind", json!({"server": "stun.example.com:3478", "local_port": 12345})),
        ("discovery.peers", json!({})),
        (
            "rendezvous.register",
            json!({"server": "http://relay.example.com", "node_id": "test", "family_id": "nat0", "public_address": "1.2.3.4:5678"}),
        ),
        ("rendezvous.lookup", json!({"server": "http://relay.example.com", "target": "test"})),
        ("peer.connect", json!({"target_address": "1.2.3.4:5678"})),
    ];

    for (method, params) in methods {
        let result = handler.handle(method, params).await;

        // None should return "Unknown method"
        match result {
            Ok(_) => {
                // Success - method routed correctly
                assert!(true, "{} routed successfully", method);
            }
            Err(e) => {
                assert!(
                    !e.contains("Unknown method"),
                    "{} should be routed (got error: {})",
                    method,
                    e
                );
            }
        }
    }
}

#[tokio::test]
async fn test_unknown_method_returns_error() {
    let handler = create_test_handler();

    let result = handler.handle("unknown.method", json!({})).await;

    assert!(result.is_err(), "Unknown methods should error");
    let error = result.unwrap_err();
    assert!(error.contains("Unknown method"), "Should indicate unknown method: {}", error);
}

#[tokio::test]
async fn test_method_case_sensitivity() {
    let handler = create_test_handler();

    // JSON-RPC methods are case-sensitive
    let result = handler.handle("STUN.GET_PUBLIC_ADDRESS", json!({})).await;

    assert!(result.is_err(), "Wrong case should error");
    assert!(result.unwrap_err().contains("Unknown method"), "Should be unknown method");
}

// ============================================================================
// Handler State Tests
// ============================================================================

#[tokio::test]
async fn test_handler_is_stateless_between_calls() {
    let handler = create_test_handler();

    // First call
    let result1 = handler.handle("discovery.peers", json!({})).await;
    assert!(result1.is_ok());

    // Second call should not be affected
    let result2 = handler.handle("discovery.peers", json!({})).await;
    assert!(result2.is_ok());

    // Both should return same structure
    let peers1 = result1.unwrap()["peers"].as_array().unwrap().len();
    let peers2 = result2.unwrap()["peers"].as_array().unwrap().len();

    // Should be consistent (both empty initially)
    assert_eq!(peers1, peers2, "Calls should be independent");
}

#[tokio::test]
async fn test_concurrent_method_calls() {
    let handler = Arc::new(create_test_handler());

    // Spawn multiple concurrent calls
    let mut handles = vec![];

    for i in 0..10 {
        let handler_clone = Arc::clone(&handler);
        let handle = tokio::spawn(async move {
            let method = match i % 6 {
                0 => "stun.get_public_address",
                1 => "stun.bind",
                2 => "discovery.peers",
                3 => "rendezvous.register",
                4 => "rendezvous.lookup",
                _ => "peer.connect",
            };

            let params = match method {
                "stun.bind" => json!({"server": "stun.example.com:3478", "local_port": 12345}),
                "rendezvous.register" => {
                    json!({"server": "http://relay.example.com", "node_id": "test", "family_id": "nat0", "public_address": "1.2.3.4:5678"})
                }
                "rendezvous.lookup" => {
                    json!({"server": "http://relay.example.com", "target": "test"})
                }
                "peer.connect" => json!({"target_address": "1.2.3.4:5678"}),
                _ => json!({}),
            };

            handler_clone.handle(method, params).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        let result = handle.await.unwrap();

        // All should either succeed or have specific error (not "Unknown method")
        match result {
            Ok(_) => assert!(true),
            Err(e) => assert!(
                !e.contains("Unknown method"),
                "Concurrent call failed with unknown method: {}",
                e
            ),
        }
    }
}

// ============================================================================
// JSON-RPC Compliance Tests
// ============================================================================

#[tokio::test]
async fn test_json_rpc_null_params() {
    let handler = create_test_handler();

    // Some methods should handle null params
    let result = handler.handle("discovery.peers", serde_json::Value::Null).await;

    match result {
        Ok(_) => assert!(true, "Should handle null params"),
        Err(e) => {
            // Should not be "Unknown method"
            assert!(!e.contains("Unknown method"), "Should be routed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_json_rpc_array_params() {
    let handler = create_test_handler();

    // Array params (non-standard but should handle gracefully)
    let result = handler.handle("discovery.peers", json!([])).await;

    match result {
        Ok(_) => assert!(true),
        Err(e) => {
            // Should handle or error gracefully (not "Unknown method")
            assert!(!e.contains("Unknown method"), "Should be routed: {}", e);
        }
    }
}
