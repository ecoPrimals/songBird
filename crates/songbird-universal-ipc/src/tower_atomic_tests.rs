// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use serde_json::{Value, json};

// Simple test handler
struct MathService;

impl JsonRpcHandler for MathService {
    async fn handle(&self, method: &str, params: Value) -> Result<Value, String> {
        match method {
            "add" => {
                let a = params["a"].as_i64().ok_or("Missing a")?;
                let b = params["b"].as_i64().ok_or("Missing b")?;
                Ok(json!(a + b))
            }
            "multiply" => {
                let a = params["a"].as_i64().ok_or("Missing a")?;
                let b = params["b"].as_i64().ok_or("Missing b")?;
                Ok(json!(a * b))
            }
            _ => Err(format!("Unknown method: {method}")),
        }
    }
}

#[tokio::test]
async fn test_json_rpc_request_creation() {
    let req = JsonRpcRequest::new("test_method", Some(json!({"key": "value"})), 1);
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.method, "test_method");
    assert_eq!(req.id, Some(json!(1)));
}

#[tokio::test]
async fn test_json_rpc_response_success() {
    let resp = JsonRpcResponse::success(json!(42), json!(1));
    assert_eq!(resp.jsonrpc, "2.0");
    assert_eq!(resp.result, Some(json!(42)));
    assert!(resp.error.is_none());
}

#[tokio::test]
async fn test_json_rpc_response_error() {
    let err = JsonRpcError::internal_error("Test error");
    let resp = JsonRpcResponse::error(err, json!(1));
    assert_eq!(resp.jsonrpc, "2.0");
    assert!(resp.result.is_none());
    assert!(resp.error.is_some());
}

#[tokio::test]
async fn test_math_service_handler() {
    let handler = MathService;

    // Test add
    let result = handler.handle("add", json!({"a": 5, "b": 3})).await.expect("Add should succeed");
    assert_eq!(result, json!(8));

    // Test multiply
    let result =
        handler.handle("multiply", json!({"a": 4, "b": 7})).await.expect("Multiply should succeed");
    assert_eq!(result, json!(28));

    // Test unknown method
    let result = handler.handle("unknown", json!({})).await;
    assert!(result.is_err());
}

#[test]
fn json_rpc_request_serde_roundtrip() {
    let req = JsonRpcRequest::new("echo", Some(json!({"k": 1})), 42);
    let s = serde_json::to_string(&req).expect("serialize request");
    let back: JsonRpcRequest = serde_json::from_str(&s).expect("deserialize request");
    assert_eq!(back.jsonrpc, "2.0");
    assert_eq!(back.method, "echo");
    assert_eq!(back.id, Some(json!(42)));
}

#[test]
fn json_rpc_error_helpers_and_serde() {
    let e = JsonRpcError::method_not_found("m");
    assert_eq!(e.code, JsonRpcError::METHOD_NOT_FOUND);
    let e2 = JsonRpcError::invalid_params("bad");
    assert_eq!(e2.code, JsonRpcError::INVALID_PARAMS);
    let e3 = JsonRpcError::internal_error("x");
    assert_eq!(e3.code, JsonRpcError::INTERNAL_ERROR);
    let v = serde_json::to_value(&e3).expect("to value");
    let back: JsonRpcError = serde_json::from_value(v).expect("from value");
    assert_eq!(back.message, "x");
}

#[test]
fn json_rpc_response_roundtrip() {
    let ok = JsonRpcResponse::success(json!(true), json!(7));
    let s = serde_json::to_string(&ok).expect("ser");
    let back: JsonRpcResponse = serde_json::from_str(&s).expect("de");
    assert_eq!(back.result, Some(json!(true)));

    let err = JsonRpcResponse::error(
        JsonRpcError {
            code: JsonRpcError::PARSE_ERROR,
            message: "parse".into(),
            data: None,
        },
        json!(null),
    );
    assert!(err.error.is_some());
}

struct EchoHandler;

impl JsonRpcHandler for EchoHandler {
    async fn handle(&self, _method: &str, params: Value) -> Result<Value, String> {
        Ok(params)
    }
}

#[tokio::test]
async fn handle_request_for_test_rejects_non_2_0_jsonrpc() {
    let handler = EchoHandler;
    let req = JsonRpcRequest {
        jsonrpc: "1.0".into(),
        method: "x".into(),
        params: None,
        id: Some(json!(1)),
    };
    let resp = TowerAtomicServer::handle_request_for_test(req, &handler).await;
    let err = resp.error.expect("error response");
    assert_eq!(err.code, JsonRpcError::INVALID_REQUEST);
}

#[tokio::test]
async fn handle_request_for_test_success_wraps_handler_ok() {
    let handler = EchoHandler;
    let req = JsonRpcRequest::new("echo", Some(json!({"a": 1})), 7);
    let resp = TowerAtomicServer::handle_request_for_test(req, &handler).await;
    assert!(resp.error.is_none());
    assert_eq!(resp.result.expect("result"), json!({"a": 1}));
}

#[tokio::test]
async fn handle_request_for_test_wraps_handler_err_as_internal() {
    struct Fail;
    impl JsonRpcHandler for Fail {
        async fn handle(&self, _method: &str, _params: Value) -> Result<Value, String> {
            Err("boom".into())
        }
    }
    let req = JsonRpcRequest::new("m", None, 1);
    let resp = TowerAtomicServer::handle_request_for_test(req, &Fail).await;
    let err = resp.error.expect("rpc error");
    assert_eq!(err.code, JsonRpcError::INTERNAL_ERROR);
    assert!(err.message.contains("boom"));
}

#[tokio::test]
async fn handle_request_for_test_omitted_params_become_null() {
    struct NullCheck;
    impl JsonRpcHandler for NullCheck {
        async fn handle(&self, _method: &str, params: Value) -> Result<Value, String> {
            assert!(params.is_null());
            Ok(json!(true))
        }
    }
    let req = JsonRpcRequest {
        jsonrpc: "2.0".into(),
        method: "m".into(),
        params: None,
        id: Some(json!(99)),
    };
    let resp = TowerAtomicServer::handle_request_for_test(req, &NullCheck).await;
    assert_eq!(resp.result, Some(json!(true)));
}

#[test]
fn json_rpc_error_standard_codes_match_spec() {
    assert_eq!(JsonRpcError::PARSE_ERROR, -32700);
    assert_eq!(JsonRpcError::INVALID_REQUEST, -32600);
    assert_eq!(JsonRpcError::METHOD_NOT_FOUND, -32601);
}

#[test]
fn json_rpc_request_new_uses_numeric_id() {
    let r = JsonRpcRequest::new("ping", None, 1001);
    assert_eq!(r.id, Some(json!(1001)));
    assert!(r.params.is_none());
}

#[tokio::test]
async fn handle_request_invalid_jsonrpc_version_message() {
    struct X;
    impl JsonRpcHandler for X {
        async fn handle(&self, _method: &str, _params: Value) -> Result<Value, String> {
            Ok(json!(0))
        }
    }
    let req = JsonRpcRequest {
        jsonrpc: "2.1".into(),
        method: "m".into(),
        params: Some(json!({})),
        id: Some(json!("abc")),
    };
    let resp = TowerAtomicServer::handle_request_for_test(req, &X).await;
    let e = resp.error.expect("err");
    assert_eq!(e.code, JsonRpcError::INVALID_REQUEST);
}
