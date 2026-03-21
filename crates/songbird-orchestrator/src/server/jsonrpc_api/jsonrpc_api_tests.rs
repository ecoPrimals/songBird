// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use serde_json::Value;

#[test]
fn test_jsonrpc_error_codes() {
    assert_eq!(JsonRpcError::PARSE_ERROR, -32700);
    assert_eq!(JsonRpcError::INVALID_REQUEST, -32600);
    assert_eq!(JsonRpcError::METHOD_NOT_FOUND, -32601);
    assert_eq!(JsonRpcError::INVALID_PARAMS, -32602);
    assert_eq!(JsonRpcError::INTERNAL_ERROR, -32603);
}

#[test]
fn test_jsonrpc_error_creation() {
    let error = JsonRpcError::method_not_found("test.method");
    assert_eq!(error.code, JsonRpcError::METHOD_NOT_FOUND);
    assert!(error.message.contains("test.method"));
}

#[test]
fn test_jsonrpc_request_deserialization() {
    let json = r#"{
            "jsonrpc": "2.0",
            "method": "songbird.health",
            "id": 1
        }"#;

    let request: JsonRpcRequest = serde_json::from_str(json).unwrap();
    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "songbird.health");
    assert!(request.id.is_some());
}

#[test]
fn test_jsonrpc_response_serialization() {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(serde_json::json!({"status": "ok"})),
        error: None,
        id: Value::Number(1.into()),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"result\""));
    assert!(!json.contains("\"error\""));
}
