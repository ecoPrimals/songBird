//! Pure Rust JSON-RPC 2.0 Types
//!
//! Manual implementation of JSON-RPC 2.0 protocol types using only `serde_json`.
//! This approach eliminates the need for heavy RPC libraries and their C dependencies,
//! achieving 100% Pure Rust compliance.
//!
//! Inspired by BearDog's proven production implementation.
//!
//! ## Philosophy
//! - ✅ Simple: ~100 lines vs 50,000+ in jsonrpsee
//! - ✅ Pure Rust: Zero C dependencies
//! - ✅ Full control: Custom error handling and routing
//! - ✅ Production proven: Same approach as BearDog
//! - ✅ Modern idiomatic: async/await, type safety

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 Request
///
/// Fully compliant with JSON-RPC 2.0 specification.
///
/// # Example
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "ping",
///   "params": {"message": "hello"},
///   "id": 1
/// }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC protocol version (must be "2.0")
    pub jsonrpc: String,

    /// Method name to invoke
    pub method: String,

    /// Optional parameters for the method
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,

    /// Request identifier (can be null for notifications)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(
        method: impl Into<String>,
        params: Option<serde_json::Value>,
        id: impl Into<serde_json::Value>,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id: Some(id.into()),
        }
    }

    /// Create a notification (request without id)
    pub fn notification(method: impl Into<String>, params: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id: None,
        }
    }

    /// Check if this is a notification
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

/// JSON-RPC 2.0 Response
///
/// # Success Example
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "result": {"pong": true},
///   "id": 1
/// }
/// ```
///
/// # Error Example
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "error": {
///     "code": -32601,
///     "message": "Method not found"
///   },
///   "id": 1
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC protocol version (always "2.0")
    pub jsonrpc: String,

    /// Result value (present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    /// Error object (present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,

    /// Request identifier (matches request id)
    pub id: serde_json::Value,
}

impl JsonRpcResponse {
    /// Create a success response
    pub fn success(result: serde_json::Value, id: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error response
    pub fn error(error: JsonRpcError, id: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

/// JSON-RPC 2.0 Error Object
///
/// Standard error codes as defined by JSON-RPC 2.0 specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code (standard or application-defined)
    pub code: i32,

    /// Human-readable error message
    pub message: String,

    /// Optional additional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    // Standard JSON-RPC 2.0 error codes

    /// Parse error - Invalid JSON
    pub const PARSE_ERROR: i32 = -32700;

    /// Invalid request - Request object is not valid
    pub const INVALID_REQUEST: i32 = -32600;

    /// Method not found - Method does not exist
    pub const METHOD_NOT_FOUND: i32 = -32601;

    /// Invalid params - Invalid method parameters
    pub const INVALID_PARAMS: i32 = -32602;

    /// Internal error - Internal JSON-RPC error
    pub const INTERNAL_ERROR: i32 = -32603;

    // Application-specific error codes (as per spec, -32000 to -32099)

    /// Unauthorized - Authentication required or failed
    pub const UNAUTHORIZED: i32 = -32000;

    /// Forbidden - Insufficient permissions
    pub const FORBIDDEN: i32 = -32001;

    /// Not found - Resource not found
    pub const NOT_FOUND: i32 = -32002;

    /// Timeout - Operation timed out
    pub const TIMEOUT: i32 = -32003;

    /// Create a new error
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create a new error with additional data
    pub fn with_data(code: i32, message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Create a parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::new(Self::PARSE_ERROR, message)
    }

    /// Create an invalid request error
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(Self::INVALID_REQUEST, message)
    }

    /// Create a method not found error
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self::new(Self::METHOD_NOT_FOUND, format!("Method not found: {}", method.into()))
    }

    /// Create an invalid params error
    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(Self::INVALID_PARAMS, message)
    }

    /// Create an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(Self::INTERNAL_ERROR, message)
    }

    /// Create an unauthorized error
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(Self::UNAUTHORIZED, message)
    }

    /// Create a forbidden error
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(Self::FORBIDDEN, message)
    }

    /// Create a not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(Self::NOT_FOUND, message)
    }

    /// Create a timeout error
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(Self::TIMEOUT, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let req = JsonRpcRequest::new("ping", None, 1);
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "ping");
        assert!(req.params.is_none());
        assert_eq!(req.id, Some(serde_json::json!(1)));
        assert!(!req.is_notification());
    }

    #[test]
    fn test_notification_creation() {
        let req = JsonRpcRequest::notification("notify", Some(serde_json::json!({"msg": "hello"})));
        assert_eq!(req.method, "notify");
        assert!(req.is_notification());
    }

    #[test]
    fn test_success_response() {
        let resp =
            JsonRpcResponse::success(serde_json::json!({"pong": true}), serde_json::json!(1));
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_error_response() {
        let err = JsonRpcError::method_not_found("test");
        let resp = JsonRpcResponse::error(err, serde_json::json!(1));
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, JsonRpcError::METHOD_NOT_FOUND);
    }

    #[test]
    fn test_serialization() {
        let req = JsonRpcRequest::new("ping", None, 1);
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"ping\""));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{"jsonrpc":"2.0","method":"ping","id":1}"#;
        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.method, "ping");
        assert_eq!(req.id, Some(serde_json::json!(1)));
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(JsonRpcError::PARSE_ERROR, -32700);
        assert_eq!(JsonRpcError::INVALID_REQUEST, -32600);
        assert_eq!(JsonRpcError::METHOD_NOT_FOUND, -32601);
        assert_eq!(JsonRpcError::INVALID_PARAMS, -32602);
        assert_eq!(JsonRpcError::INTERNAL_ERROR, -32603);
    }
}
