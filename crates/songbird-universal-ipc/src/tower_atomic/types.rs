// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! JSON-RPC 2.0 request and response types for Tower Atomic.
//!
//! This module defines the wire and application-level DTOs used by the Tower Atomic
//! client and server. Borrowing wire types ([`JsonRpcRequestWire`], [`JsonRpcResponseWire`])
//! avoid allocating duplicate strings on hot paths when parsing newline-delimited JSON.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;

/// JSON-RPC protocol version string.
pub const JSONRPC_VERSION: &str = "2.0";

/// JSON-RPC 2.0 Request
///
/// Per <https://www.jsonrpc.org/specification#request_object>, `id` is
/// omitted for notifications (the server MUST NOT reply).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// Protocol version; must be `"2.0"` for compliant peers.
    pub jsonrpc: String,
    /// Method name to invoke on the server.
    pub method: String,
    /// Optional parameters object or array.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    /// Request correlation id; omitted for notifications.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request (with numeric id).
    pub fn new(method: impl Into<String>, params: Option<Value>, id: u64) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.into(),
            method: method.into(),
            params,
            id: Some(Value::Number(id.into())),
        }
    }

    /// Returns `true` when this is a notification (no `id` field).
    #[must_use]
    pub const fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

/// Borrowing view of a request line — avoids allocating `jsonrpc` / `method` strings on the server hot path.
///
/// This module is not re-exported; `pub` here is crate-internal visibility for sibling `tower_atomic` modules.
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequestWire<'a> {
    #[serde(borrow)]
    pub jsonrpc: Cow<'a, str>,
    #[serde(borrow)]
    pub method: Cow<'a, str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

impl JsonRpcRequestWire<'_> {
    #[must_use]
    pub const fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

/// Client response line: borrows error messages from the read buffer when possible.
#[derive(Debug, Deserialize)]
pub struct JsonRpcResponseWire<'a> {
    #[expect(dead_code, reason = "envelope field; validated implicitly")]
    #[serde(borrow)]
    pub jsonrpc: Cow<'a, str>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcErrorWire<'a>>,
    #[allow(
        dead_code,
        reason = "JSON-RPC correlation id; reserved for future client-side validation"
    )]
    pub id: Value,
}

/// Borrowing JSON-RPC error object for client-side parsing.
#[derive(Debug, Deserialize)]
pub struct JsonRpcErrorWire<'a> {
    #[allow(
        dead_code,
        reason = "standard JSON-RPC error code; message carries user-facing detail"
    )]
    pub code: i32,
    #[serde(borrow)]
    pub message: Cow<'a, str>,
    #[expect(dead_code, reason = "optional JSON-RPC error extension object")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version.
    pub jsonrpc: String,
    /// Result value on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error object on failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Correlation id matching the request.
    pub id: Value,
}

impl JsonRpcResponse {
    /// Create a success response
    #[must_use]
    pub fn success(result: Value, id: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.into(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error response
    #[must_use]
    pub fn error(error: JsonRpcError, id: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.into(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// JSON-RPC error code (see spec for standard ranges).
    pub code: i32,
    /// Human-readable message.
    pub message: String,
    /// Optional structured error data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    // Standard JSON-RPC 2.0 error codes
    /// Parse error (-32700).
    pub const PARSE_ERROR: i32 = -32700;
    /// Invalid Request (-32600).
    pub const INVALID_REQUEST: i32 = -32600;
    /// Method not found (-32601).
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid params (-32602).
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal error (-32603).
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Create a method not found error
    pub fn method_not_found(method: impl Into<String>) -> Self {
        let method = method.into();
        Self {
            code: Self::METHOD_NOT_FOUND,
            message: format!("Method not found: {method}"),
            data: None,
        }
    }

    /// Create an invalid params error
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: Self::INVALID_PARAMS,
            message: message.into(),
            data: None,
        }
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            code: Self::INTERNAL_ERROR,
            message: message.into(),
            data: None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn request_new_sets_id_and_version() {
        let req = JsonRpcRequest::new("health.check", None, 42);
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "health.check");
        assert_eq!(req.id, Some(json!(42)));
        assert!(!req.is_notification());
    }

    #[test]
    fn request_is_notification_when_no_id() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: "event.fire".into(),
            params: None,
            id: None,
        };
        assert!(req.is_notification());
    }

    #[test]
    fn request_serialization_roundtrip() {
        let req = JsonRpcRequest::new("test.method", Some(json!({"key": "value"})), 1);
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.method, "test.method");
        assert_eq!(deserialized.params.unwrap()["key"], "value");
    }

    #[test]
    fn request_skips_null_params_in_serialization() {
        let req = JsonRpcRequest::new("simple", None, 1);
        let serialized = serde_json::to_string(&req).unwrap();
        assert!(!serialized.contains("params"));
    }

    #[test]
    fn response_success_has_result_no_error() {
        let resp = JsonRpcResponse::success(json!("ok"), json!(1));
        assert_eq!(resp.jsonrpc, "2.0");
        assert_eq!(resp.result, Some(json!("ok")));
        assert!(resp.error.is_none());
        assert_eq!(resp.id, json!(1));
    }

    #[test]
    fn response_error_has_error_no_result() {
        let err = JsonRpcError::method_not_found("unknown.method");
        let resp = JsonRpcResponse::error(err, json!(99));
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.id, json!(99));
        let e = resp.error.unwrap();
        assert_eq!(e.code, JsonRpcError::METHOD_NOT_FOUND);
        assert!(e.message.contains("unknown.method"));
    }

    #[test]
    fn response_serialization_roundtrip() {
        let resp = JsonRpcResponse::success(json!({"status": "healthy"}), json!(5));
        let serialized = serde_json::to_string(&resp).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.result.unwrap()["status"], "healthy");
    }

    #[test]
    fn error_method_not_found_code() {
        let err = JsonRpcError::method_not_found("foo");
        assert_eq!(err.code, -32601);
        assert!(err.message.contains("foo"));
        assert!(err.data.is_none());
    }

    #[test]
    fn error_invalid_params_code() {
        let err = JsonRpcError::invalid_params("missing field 'name'");
        assert_eq!(err.code, -32602);
        assert!(err.message.contains("missing field"));
    }

    #[test]
    fn error_internal_error_code() {
        let err = JsonRpcError::internal_error("unexpected failure");
        assert_eq!(err.code, -32603);
        assert!(err.message.contains("unexpected"));
    }

    #[test]
    fn error_constants_match_spec() {
        assert_eq!(JsonRpcError::PARSE_ERROR, -32700);
        assert_eq!(JsonRpcError::INVALID_REQUEST, -32600);
        assert_eq!(JsonRpcError::METHOD_NOT_FOUND, -32601);
        assert_eq!(JsonRpcError::INVALID_PARAMS, -32602);
        assert_eq!(JsonRpcError::INTERNAL_ERROR, -32603);
    }

    #[test]
    fn wire_request_deserialization() {
        let raw = r#"{"jsonrpc":"2.0","method":"test","params":null,"id":1}"#;
        let wire: JsonRpcRequestWire = serde_json::from_str(raw).unwrap();
        assert_eq!(wire.method.as_ref(), "test");
        assert!(!wire.is_notification());
    }

    #[test]
    fn wire_request_notification() {
        let raw = r#"{"jsonrpc":"2.0","method":"event.fire"}"#;
        let wire: JsonRpcRequestWire = serde_json::from_str(raw).unwrap();
        assert!(wire.is_notification());
    }

    #[test]
    fn wire_response_success_deserialization() {
        let raw = r#"{"jsonrpc":"2.0","result":"ok","id":1}"#;
        let wire: JsonRpcResponseWire = serde_json::from_str(raw).unwrap();
        assert_eq!(wire.result, Some(json!("ok")));
        assert!(wire.error.is_none());
    }

    #[test]
    fn wire_response_error_deserialization() {
        let raw = r#"{"jsonrpc":"2.0","error":{"code":-32601,"message":"not found"},"id":2}"#;
        let wire: JsonRpcResponseWire = serde_json::from_str(raw).unwrap();
        assert!(wire.result.is_none());
        let e = wire.error.unwrap();
        assert_eq!(e.code, -32601);
        assert_eq!(e.message.as_ref(), "not found");
    }

    #[test]
    fn jsonrpc_version_constant() {
        assert_eq!(JSONRPC_VERSION, "2.0");
    }
}
