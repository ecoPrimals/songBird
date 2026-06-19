// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 version string (static to avoid allocations)
pub const JSONRPC_VERSION: &str = "2.0";

/// JSON-RPC 2.0 Request
/// <https://www.jsonrpc.org/specification#request_object>
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version (must be "2.0")
    pub jsonrpc: String,

    /// Method name to call
    pub method: String,

    /// Parameters (can be array or object)
    #[serde(default)]
    pub params: Option<Value>,

    /// Request ID (for responses, null for notifications)
    pub id: Option<Value>,
}

/// JSON-RPC 2.0 Response
/// <https://www.jsonrpc.org/specification#response_object>
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,

    /// Result (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,

    /// Error (on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,

    /// Request ID (same as request, or null)
    pub id: Value,
}

/// JSON-RPC 2.0 Error
/// <https://www.jsonrpc.org/specification#error_object>
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,

    /// Error message
    pub message: String,

    /// Additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Standard JSON-RPC error codes
#[allow(dead_code, reason = "JSON-RPC error constructors and codes reserved for API completeness")]
impl JsonRpcError {
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;

    #[must_use]
    pub fn parse_error() -> Self {
        Self {
            code: Self::PARSE_ERROR,
            message: String::from("Parse error"),
            data: None,
        }
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self {
            code: Self::INVALID_REQUEST,
            message: message.into(),
            data: None,
        }
    }

    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self {
            code: Self::METHOD_NOT_FOUND,
            message: format!("Method not found: {}", method.into()),
            data: None,
        }
    }

    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: Self::INVALID_PARAMS,
            message: message.into(),
            data: None,
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self {
            code: Self::INTERNAL_ERROR,
            message: message.into(),
            data: None,
        }
    }
}
