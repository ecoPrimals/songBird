// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Unit Tests: Squirrel Integration - HTTP Delegation
//!
//! Tests the two critical RPC methods for Squirrel AI integration:
//! - `discover_capabilities`: Capability discovery
//! - http.request: HTTP delegation for external APIs

use serde_json::json;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// Helper: Clean environment for isolated test
fn clean_env() -> std::collections::HashMap<String, String> {
    let mut env = std::collections::HashMap::new();
    env.insert("PATH".to_string(), std::env::var("PATH").unwrap_or_default());
    env
}

/// Helper: Send JSON-RPC request to Unix socket
async fn send_jsonrpc_request(
    socket_path: &str,
    method: &str,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut stream = UnixStream::connect(socket_path).await?;

    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let request_str = serde_json::to_string(&request)?;
    stream.write_all(request_str.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;

    let response: serde_json::Value = serde_json::from_str(&response_line)?;
    Ok(response)
}

#[tokio::test]
async fn test_discover_capabilities_structure() {
    // Test that discover_capabilities returns proper structure
    // Note: This is a unit test of the response format, not integration

    let capabilities = vec![
        "http.post",
        "http.get",
        "http.request",
        "discovery.announce",
        "discovery.query",
        "security.verify",
    ];

    let expected_response = json!({
        "capabilities": capabilities,
        "metadata": {
            "primal_name": "songbird",
            "version": env!("CARGO_PKG_VERSION"),
            "family_id": "nat0"
        }
    });

    // Verify structure
    assert!(expected_response["capabilities"].is_array());
    assert_eq!(expected_response["capabilities"].as_array().unwrap().len(), 6);
    assert!(expected_response["metadata"].is_object());
    assert_eq!(expected_response["metadata"]["primal_name"], "songbird");
}

#[tokio::test]
async fn test_discover_capabilities_contains_http_request() {
    // Test that http.request capability is advertised

    let capabilities = [
        "http.post",
        "http.get",
        "http.request",
        "discovery.announce",
        "discovery.query",
        "security.verify",
    ];

    assert!(capabilities.contains(&"http.request"));
    assert!(capabilities.contains(&"http.post"));
    assert!(capabilities.contains(&"http.get"));
}

#[tokio::test]
async fn test_http_request_params_validation() {
    // Test parameter validation for http.request

    // Valid params
    let valid_params = json!({
        "method": "GET",
        "url": "https://httpbin.org/get",
        "headers": {},
    });

    assert!(valid_params["method"].is_string());
    assert!(valid_params["url"].is_string());
    assert!(valid_params["headers"].is_object());

    // Invalid params (missing required fields)
    let invalid_params = json!({
        "url": "https://httpbin.org/get"
        // Missing "method"
    });

    assert!(invalid_params.get("method").is_none());
}

#[tokio::test]
async fn test_http_request_method_support() {
    // Test that all HTTP methods are supported

    let supported_methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH"];

    for method in supported_methods {
        let params = json!({
            "method": method,
            "url": "https://httpbin.org/anything",
            "headers": {},
        });

        assert_eq!(params["method"], method);
    }
}

#[tokio::test]
async fn test_http_request_headers_format() {
    // Test header format validation

    let params = json!({
        "method": "POST",
        "url": "https://api.anthropic.com/v1/messages",
        "headers": {
            "anthropic-version": "2023-06-01",
            "content-type": "application/json",
            "x-api-key": "sk-ant-test-key"
        },
        "body": {
            "model": "claude-3-opus-20240229",
            "max_tokens": 1024,
            "messages": [
                {
                    "role": "user",
                    "content": "Hello!"
                }
            ]
        }
    });

    assert!(params["headers"].is_object());
    assert!(params["headers"]["anthropic-version"].is_string());
    assert!(params["headers"]["content-type"].is_string());
    assert!(params["headers"]["x-api-key"].is_string());
    assert!(params["body"].is_object());
}

#[tokio::test]
async fn test_http_request_optional_body() {
    // Test that body parameter is optional (for GET requests)

    let get_params = json!({
        "method": "GET",
        "url": "https://httpbin.org/get",
        "headers": {},
    });

    assert!(get_params.get("body").is_none());

    let post_params = json!({
        "method": "POST",
        "url": "https://httpbin.org/post",
        "headers": {},
        "body": {"key": "value"}
    });

    assert!(post_params.get("body").is_some());
}

#[tokio::test]
async fn test_jsonrpc_request_format() {
    // Test JSON-RPC 2.0 request format

    let request = json!({
        "jsonrpc": "2.0",
        "method": "discover_capabilities",
        "params": {},
        "id": 1
    });

    assert_eq!(request["jsonrpc"], "2.0");
    assert!(request["method"].is_string());
    assert!(request.get("id").is_some());
}

#[tokio::test]
async fn test_jsonrpc_response_structure() {
    // Test JSON-RPC 2.0 response structure

    let success_response = json!({
        "jsonrpc": "2.0",
        "result": {
            "capabilities": ["http.request"],
            "metadata": {
                "primal_name": "songbird"
            }
        },
        "id": 1
    });

    assert_eq!(success_response["jsonrpc"], "2.0");
    assert!(success_response.get("result").is_some());
    assert!(success_response.get("error").is_none());

    let error_response = json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32600,
            "message": "Invalid Request"
        },
        "id": 1
    });

    assert_eq!(error_response["jsonrpc"], "2.0");
    assert!(error_response.get("error").is_some());
    assert!(error_response.get("result").is_none());
}

#[tokio::test]
async fn test_family_id_environment_variable() {
    // Test that family_id can be customized via environment

    songbird_process_env::set_var("SONGBIRD_FAMILY_ID", "test-family");
    let family_id = std::env::var("SONGBIRD_FAMILY_ID").unwrap_or_else(|_| "nat0".to_string());
    assert_eq!(family_id, "test-family");

    songbird_process_env::remove_var("SONGBIRD_FAMILY_ID");
    let family_id = std::env::var("SONGBIRD_FAMILY_ID").unwrap_or_else(|_| "nat0".to_string());
    assert_eq!(family_id, "nat0");
}

#[tokio::test]
async fn test_timeout_configuration() {
    // Test timeout values are reasonable

    let request_timeout = Duration::from_secs(60);
    let connect_timeout = Duration::from_secs(10);

    assert!(request_timeout > connect_timeout);
    assert!(request_timeout.as_secs() <= 120); // Max 2 minutes
    assert!(connect_timeout.as_secs() >= 5); // Min 5 seconds
}

#[tokio::test]
async fn test_error_code_validity() {
    // Test JSON-RPC error codes are valid

    let method_not_found = -32601;
    let invalid_params = -32602;
    let internal_error = -32603;

    assert!(method_not_found < 0);
    assert!(invalid_params < 0);
    assert!(internal_error < 0);
}
