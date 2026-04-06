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
    #[allow(dead_code, reason = "envelope field; validated implicitly")]
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
    #[allow(dead_code, reason = "optional JSON-RPC error extension object")]
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
