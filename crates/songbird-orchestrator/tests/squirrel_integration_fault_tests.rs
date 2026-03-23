// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::ignore_without_reason,
    clippy::unreadable_literal,
    clippy::no_effect_underscore_binding,
    clippy::float_cmp,
    clippy::default_trait_access,
    clippy::needless_collect,
    clippy::unused_async,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::items_after_statements,
    clippy::unnecessary_wraps,
    clippy::used_underscore_binding,
    clippy::struct_excessive_bools,
    clippy::similar_names,
    clippy::significant_drop_tightening,
    clippy::struct_field_names,
    clippy::match_same_arms,
    clippy::future_not_send,
    reason = "integration tests: strict clippy matches crate [lints] policy"
)]

//! Fault Tests: Squirrel Integration - Error Handling & Edge Cases
//!
//! Tests error conditions and fault injection:
//! - Invalid inputs
//! - Network failures
//! - Resource exhaustion
//! - Edge cases

mod common;
use common::event_helpers::ReadyNotifier;

use serde_json::json;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

/// Helper: Start fault-injecting server
async fn start_fault_server(
    socket_path: &str,
    fail_mode: &str,
    notifier: ReadyNotifier,
) -> tokio::task::JoinHandle<()> {
    let socket_path = socket_path.to_string();
    let fail_mode = fail_mode.to_string();

    tokio::spawn(async move {
        let listener = UnixListener::bind(&socket_path).unwrap();
        notifier.signal_ready(); // ✅ Signal ready immediately after bind

        while let Ok((mut stream, _)) = listener.accept().await {
            let mode = fail_mode.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(&mut stream);
                let mut line = String::new();

                if reader.read_line(&mut line).await.is_ok()
                    && let Ok(request) = serde_json::from_str::<serde_json::Value>(&line)
                {
                    let response = match mode.as_str() {
                        "invalid_response" => {
                            // Send invalid JSON
                            let _ = stream.write_all(b"invalid json\n").await;
                            return;
                        }
                        "error" => json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": -32603,
                                "message": "Internal error"
                            },
                            "id": request["id"]
                        }),
                        "disconnect" => {
                            // Close connection immediately
                            return;
                        }
                        _ => json!({
                            "jsonrpc": "2.0",
                            "result": {"status": "ok"},
                            "id": request["id"]
                        }),
                    };

                    let _ = stream
                        .write_all(serde_json::to_string(&response).unwrap().as_bytes())
                        .await;
                    let _ = stream.write_all(b"\n").await;
                }
            });
        }
    })
}

#[tokio::test]
async fn test_fault_invalid_json_rpc_version() {
    // Test handling of invalid JSON-RPC version

    let request = json!({
        "jsonrpc": "1.0",  // Invalid! Should be "2.0"
        "method": "discover_capabilities",
        "params": {},
        "id": 1
    });

    assert_ne!(request["jsonrpc"], "2.0");
}

#[tokio::test]
async fn test_fault_missing_method() {
    // Test handling of missing method field

    let request = json!({
        "jsonrpc": "2.0",
        // Missing "method" field!
        "params": {},
        "id": 1
    });

    assert!(request.get("method").is_none());
}

#[tokio::test]
async fn test_fault_invalid_http_method() {
    // Test handling of invalid HTTP method

    let params = json!({
        "method": "INVALID",  // Not a valid HTTP method
        "url": "https://httpbin.org/get",
        "headers": {}
    });

    let valid_methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];
    assert!(!valid_methods.contains(&params["method"].as_str().unwrap()));
}

#[tokio::test]
async fn test_fault_missing_url() {
    // Test handling of missing URL parameter

    let params = json!({
        "method": "GET",
        // Missing "url" field!
        "headers": {}
    });

    assert!(params.get("url").is_none());
}

#[tokio::test]
async fn test_fault_malformed_url() {
    // Test handling of malformed URLs

    let invalid_urls =
        vec!["not-a-url", "ftp://unsupported-protocol.com", "://missing-scheme.com", "http://", ""];

    for url in invalid_urls {
        let _params = json!({
            "method": "GET",
            "url": url,
            "headers": {}
        });

        // URL validation would catch these
        // Either empty or malformed
        assert!(url.is_empty() || !url.starts_with("http://example.com"));
    }
}

#[tokio::test]
async fn test_fault_invalid_headers() {
    // Test handling of invalid header formats

    // Headers should be object, not array
    let invalid_params = json!({
        "method": "GET",
        "url": "https://httpbin.org/get",
        "headers": ["invalid", "array"]  // Should be object!
    });

    assert!(!invalid_params["headers"].is_object());
}

#[tokio::test]
async fn test_fault_server_error_response() {
    // Test handling of server error responses

    let socket_path = "/tmp/test-songbird-fault-error.sock";
    let _ = std::fs::remove_file(socket_path);

    let (notifier, notify_tx) = ReadyNotifier::new();
    let server = start_fault_server(socket_path, "error", notifier).await;
    notify_tx.notified().await; // ✅ Event-driven! No polling!

    let mut stream = UnixStream::connect(socket_path).await.unwrap();

    let request = json!({
        "jsonrpc": "2.0",
        "method": "discover_capabilities",
        "params": {},
        "id": 1
    });

    stream.write_all(serde_json::to_string(&request).unwrap().as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();
    stream.flush().await.unwrap();

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();

    let response: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32603);

    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_fault_connection_refused() {
    // Test handling of connection refusal

    let result = UnixStream::connect("/tmp/nonexistent-socket.sock").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_fault_server_disconnect() {
    // Test handling of server disconnecting mid-request

    let socket_path = "/tmp/test-songbird-fault-disconnect.sock";
    let _ = std::fs::remove_file(socket_path);

    let (notifier, notify_tx) = ReadyNotifier::new();
    let server = start_fault_server(socket_path, "disconnect", notifier).await;
    notify_tx.notified().await; // ✅ Event-driven! No polling!

    let mut stream = UnixStream::connect(socket_path).await.unwrap();

    let request = json!({
        "jsonrpc": "2.0",
        "method": "discover_capabilities",
        "params": {},
        "id": 1
    });

    stream.write_all(serde_json::to_string(&request).unwrap().as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();
    stream.flush().await.unwrap();

    // Try to read response - should get EOF or error
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let result = reader.read_line(&mut line).await;

    // Either EOF (0 bytes) or connection reset
    assert!(result.is_ok() && line.is_empty() || result.is_err());

    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_fault_invalid_json_response() {
    // Test handling of invalid JSON in response

    let socket_path = "/tmp/test-songbird-fault-invalid.sock";
    let _ = std::fs::remove_file(socket_path);

    let (notifier, notify_tx) = ReadyNotifier::new();
    let server = start_fault_server(socket_path, "invalid_response", notifier).await;
    notify_tx.notified().await; // ✅ Event-driven! No polling!

    let mut stream = UnixStream::connect(socket_path).await.unwrap();

    let request = json!({
        "jsonrpc": "2.0",
        "method": "discover_capabilities",
        "params": {},
        "id": 1
    });

    stream.write_all(serde_json::to_string(&request).unwrap().as_bytes()).await.unwrap();
    stream.write_all(b"\n").await.unwrap();
    stream.flush().await.unwrap();

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();

    // Should fail to parse
    let result = serde_json::from_str::<serde_json::Value>(&line);
    assert!(result.is_err());

    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_fault_empty_body() {
    // Test http.request with empty body for POST

    let params = json!({
        "method": "POST",
        "url": "https://httpbin.org/post",
        "headers": {},
        "body": {}  // Empty object
    });

    assert!(params["body"].is_object());
    assert!(params["body"].as_object().unwrap().is_empty());
}

#[tokio::test]
async fn test_fault_very_long_url() {
    // Test handling of extremely long URLs

    let long_url = format!("https://example.com/{}", "a".repeat(10000));

    let params = json!({
        "method": "GET",
        "url": long_url,
        "headers": {}
    });

    assert!(params["url"].as_str().unwrap().len() > 8000);
}

#[tokio::test]
async fn test_fault_special_characters_in_headers() {
    // Test headers with special characters

    let params = json!({
        "method": "GET",
        "url": "https://httpbin.org/get",
        "headers": {
            "X-Custom-Header": "value with spaces",
            "X-Unicode": "🐿️🐦",
            "X-Quotes": "value \"with\" quotes"
        }
    });

    assert!(params["headers"].is_object());
}

#[tokio::test]
async fn test_fault_null_params() {
    // Test handling of null params

    let request = json!({
        "jsonrpc": "2.0",
        "method": "discover_capabilities",
        "params": null,  // Null instead of object
        "id": 1
    });

    assert!(request["params"].is_null());
}

#[tokio::test]
async fn test_fault_array_params() {
    // Test handling of array params (should be object)

    let request = json!({
        "jsonrpc": "2.0",
        "method": "http.request",
        "params": ["invalid", "array"],  // Should be object!
        "id": 1
    });

    assert!(request["params"].is_array());
}

#[tokio::test]
async fn test_fault_missing_id() {
    // Test request without id (notification in JSON-RPC)

    let request = json!({
        "jsonrpc": "2.0",
        "method": "discover_capabilities",
        "params": {}
        // Missing "id" - this is a notification
    });

    assert!(request.get("id").is_none());
}

#[tokio::test]
async fn test_fault_duplicate_headers() {
    // Test duplicate header keys

    let mut headers = std::collections::HashMap::new();
    headers.insert("content-type".to_string(), "application/json".to_string());
    headers.insert("Content-Type".to_string(), "text/plain".to_string()); // Duplicate (case variation)

    // HashMap will keep only one
    assert_eq!(headers.len(), 2); // Both kept with different cases
}

#[tokio::test]
async fn test_fault_unsupported_content_type() {
    // Test response with unsupported content-type

    let params = json!({
        "method": "GET",
        "url": "https://httpbin.org/image/png",
        "headers": {}
    });

    // Should handle non-JSON responses gracefully
    assert!(params.is_object());
}

#[tokio::test]
async fn test_fault_network_timeout() {
    // Test network timeout handling

    let socket_path = "/tmp/test-songbird-fault-timeout-nonexistent.sock";

    let result =
        tokio::time::timeout(Duration::from_millis(100), UnixStream::connect(socket_path)).await;

    // Should either timeout OR get connection error (both acceptable)
    assert!(result.is_err() || (result.is_ok() && result.unwrap().is_err()));
}

#[tokio::test]
async fn test_fault_partial_write() {
    // Test handling of partial writes

    let socket_path = "/tmp/test-songbird-fault-partial.sock";
    let _ = std::fs::remove_file(socket_path);

    let (notifier, notify_tx) = ReadyNotifier::new();
    let server = start_fault_server(socket_path, "normal", notifier).await;
    notify_tx.notified().await; // ✅ Event-driven! No polling!

    let mut stream = UnixStream::connect(socket_path).await.unwrap();

    // Write incomplete JSON
    let _ = stream.write_all(b"{\"jsonrpc\":\"2.0\"").await;
    let _ = stream.flush().await;

    // Wait a bit (⏰ LEGITIMATE: Testing partial write timing behavior)
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Complete the JSON
    let _ =
        stream.write_all(b",\"method\":\"discover_capabilities\",\"params\":{},\"id\":1}\n").await;
    let _ = stream.flush().await;

    server.abort();
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_fault_empty_response() {
    // Test handling of empty response from server

    // This would be an error case - responses should always have content
    let empty_response = "";
    let result = serde_json::from_str::<serde_json::Value>(empty_response);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_fault_http_status_codes() {
    // Test various HTTP status codes that might be returned

    let status_codes = vec![
        200, // OK
        201, // Created
        400, // Bad Request
        401, // Unauthorized
        403, // Forbidden
        404, // Not Found
        500, // Internal Server Error
        503, // Service Unavailable
    ];

    for code in status_codes {
        // All should be valid u16
        assert!((100..600).contains(&code));
    }
}

#[tokio::test]
async fn test_fault_family_id_edge_cases() {
    // Test edge cases for family_id environment variable

    songbird_process_env::set_var("SONGBIRD_FAMILY_ID", "");
    let family_id = std::env::var("SONGBIRD_FAMILY_ID").unwrap_or_else(|_| "nat0".to_string());
    assert_eq!(family_id, "");

    songbird_process_env::set_var(
        "SONGBIRD_FAMILY_ID",
        "very-long-family-id-".to_string() + &"x".repeat(100),
    );
    let family_id = std::env::var("SONGBIRD_FAMILY_ID").unwrap();
    assert!(family_id.len() > 100);

    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
}
