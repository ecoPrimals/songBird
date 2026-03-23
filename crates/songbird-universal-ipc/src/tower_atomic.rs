// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tower Atomic - JSON-RPC over Universal IPC
//!
//! **Tower Atomic** is the BearDog-inspired pattern for JSON-RPC communication
//! over IPC. This module provides a universal adapter that works across all
//! platforms using the Universal IPC layer.
//!
//! ## Overview
//!
//! Tower Atomic enables:
//! - ✅ **Platform-agnostic JSON-RPC** (works on all platforms)
//! - ✅ **Type-safe RPC** (Rust type system)
//! - ✅ **Async/await** (modern Rust concurrency)
//! - ✅ **Zero hardcoding** (capability-based discovery)
//! - ✅ **Pure Rust** (no C dependencies)
//!
//! ## Architecture
//!
//! ```text
//! Application Layer:
//!   - Call JSON-RPC methods via Tower Atomic client
//!
//! Tower Atomic Layer (this module):
//!   - JSON-RPC 2.0 protocol handling
//!   - Request/response serialization
//!   - Error handling
//!
//! Universal IPC Layer:
//!   - Platform-agnostic transport
//!   - Connection management
//!
//! Platform Layer:
//!   - Unix sockets, Named pipes, TCP
//! ```
//!
//! ## Usage
//!
//! ### Server (Service Provider)
//!
//! ```rust,no_run
//! use songbird_universal_ipc::tower_atomic::{TowerAtomicServer, JsonRpcHandler};
//! use songbird_universal_ipc::ipc;
//! use serde_json::{json, Value};
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!
//! // Define your RPC handler
//! struct MyService;
//!
//! #[async_trait::async_trait]
//! impl JsonRpcHandler for MyService {
//!     async fn handle(&self, method: &str, params: Value) -> Result<Value, String> {
//!         match method {
//!             "add" => {
//!                 let a = params["a"].as_i64().ok_or("Missing a")?;
//!                 let b = params["b"].as_i64().ok_or("Missing b")?;
//!                 Ok(json!(a + b))
//!             }
//!             _ => Err(format!("Unknown method: {}", method))
//!         }
//!     }
//! }
//!
//! // Initialize and start server
//! ipc::init()?;
//! let endpoint = ipc::register("my-service", vec!["math".to_string()]).await?;
//!
//! let server = TowerAtomicServer::new(MyService);
//! server.serve(endpoint).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Client (Service Consumer)
//!
//! ```rust,no_run
//! use songbird_universal_ipc::tower_atomic::TowerAtomicClient;
//! use songbird_universal_ipc::capability::discovery;
//! use serde_json::json;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!
//! // Discover service by capability
//! let provider = discovery::discover("math").await?;
//!
//! // Connect via Tower Atomic
//! let client = TowerAtomicClient::connect(&provider.virtual_endpoint).await?;
//!
//! // Call RPC method
//! let result = client.call("add", json!({"a": 5, "b": 3})).await?;
//! assert_eq!(result, json!(8));
//! # Ok(())
//! # }
//! ```

use crate::endpoint::VirtualEndpoint;
use crate::error::{IpcError, IpcResult};
use crate::ipc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::{debug, error};

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    pub id: Value,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(method: impl Into<String>, params: Option<Value>, id: u64) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id: Value::Number(id.into()),
        }
    }
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Value,
}

impl JsonRpcResponse {
    /// Create a success response
    #[must_use]
    pub fn success(result: Value, id: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// Create an error response
    #[must_use]
    pub fn error(error: JsonRpcError, id: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    // Standard JSON-RPC 2.0 error codes
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
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

/// JSON-RPC handler trait
///
/// Implement this trait to handle JSON-RPC requests in your service.
#[async_trait]
pub trait JsonRpcHandler: Send + Sync {
    /// Handle a JSON-RPC method call
    ///
    /// # Arguments
    /// * `method` - The method name
    /// * `params` - The method parameters (JSON value)
    ///
    /// # Returns
    /// The result value or an error message
    async fn handle(&self, method: &str, params: Value) -> Result<Value, String>;
}

/// Tower Atomic Server - Serves JSON-RPC over Universal IPC
///
/// This server handles JSON-RPC 2.0 requests over the Universal IPC layer,
/// providing a platform-agnostic RPC server.
pub struct TowerAtomicServer<H: JsonRpcHandler> {
    handler: Arc<H>,
}

impl<H: JsonRpcHandler + 'static> TowerAtomicServer<H> {
    /// Create a new Tower Atomic server
    pub fn new(handler: H) -> Self {
        Self {
            handler: Arc::new(handler),
        }
    }

    /// Serve JSON-RPC requests on the given endpoint
    ///
    /// This will listen for connections and handle requests until cancelled.
    pub async fn serve(&self, endpoint: VirtualEndpoint) -> IpcResult<()> {
        debug!("Starting Tower Atomic server on {}", endpoint.path);

        let mut listener = ipc::listen(endpoint).await?;

        loop {
            match listener.accept().await {
                Ok(stream) => {
                    let handler = Arc::clone(&self.handler);
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(stream, handler).await {
                            error!("Connection handler error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Handle a single client connection
    async fn handle_connection(stream: crate::ipc::Stream, handler: Arc<H>) -> IpcResult<()> {
        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    debug!("Client disconnected");
                    break;
                }
                Ok(_) => {
                    if line.trim().is_empty() {
                        continue;
                    }

                    // Parse JSON-RPC request
                    let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
                        Ok(request) => {
                            debug!("JSON-RPC request: {}", request.method);
                            Self::handle_request(request, &*handler).await
                        }
                        Err(e) => JsonRpcResponse::error(
                            JsonRpcError {
                                code: JsonRpcError::PARSE_ERROR,
                                message: format!("Failed to parse request: {e}"),
                                data: None,
                            },
                            Value::Null,
                        ),
                    };

                    // Send response
                    let response_json = serde_json::to_string(&response)
                        .map_err(|e| IpcError::Other(e.to_string()))?;

                    writer
                        .write_all(response_json.as_bytes())
                        .await
                        .map_err(|e| IpcError::Other(e.to_string()))?;
                    writer.write_all(b"\n").await.map_err(|e| IpcError::Other(e.to_string()))?;
                }
                Err(e) => {
                    error!("Failed to read from socket: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle a JSON-RPC request
    async fn handle_request(request: JsonRpcRequest, handler: &H) -> JsonRpcResponse {
        let JsonRpcRequest {
            jsonrpc,
            method,
            params,
            id,
        } = request;

        // Validate JSON-RPC version
        if jsonrpc != "2.0" {
            return JsonRpcResponse::error(
                JsonRpcError {
                    code: JsonRpcError::INVALID_REQUEST,
                    message: "Invalid JSON-RPC version (must be 2.0)".to_string(),
                    data: None,
                },
                id,
            );
        }

        // Call handler
        let params = params.unwrap_or(Value::Null);
        match handler.handle(&method, params).await {
            Ok(result) => JsonRpcResponse::success(result, id),
            Err(message) => JsonRpcResponse::error(JsonRpcError::internal_error(message), id),
        }
    }
}

#[cfg(test)]
impl<H: JsonRpcHandler + 'static> TowerAtomicServer<H> {
    /// Exposes [`TowerAtomicServer::handle_request`] for unit tests (no I/O).
    pub(crate) async fn handle_request_for_test(
        request: JsonRpcRequest,
        handler: &H,
    ) -> JsonRpcResponse {
        Self::handle_request(request, handler).await
    }
}

/// Tower Atomic Client - Call JSON-RPC methods over Universal IPC
///
/// This client makes JSON-RPC 2.0 calls over the Universal IPC layer,
/// providing a platform-agnostic RPC client.
pub struct TowerAtomicClient {
    stream: Arc<Mutex<crate::ipc::Stream>>,
    next_id: Arc<AtomicU64>,
}

impl TowerAtomicClient {
    /// Connect to a service via virtual endpoint path
    ///
    /// # Example
    /// ```rust,no_run
    /// # use songbird_universal_ipc::tower_atomic::TowerAtomicClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = TowerAtomicClient::connect("/primal/beardog").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(virtual_path: &str) -> IpcResult<Self> {
        debug!("Connecting to {} via Tower Atomic", virtual_path);

        let stream = ipc::connect(virtual_path).await?;

        Ok(Self {
            stream: Arc::new(Mutex::new(stream)),
            next_id: Arc::new(AtomicU64::new(1)),
        })
    }

    /// Call a JSON-RPC method
    ///
    /// # Example
    /// ```rust,no_run
    /// # use songbird_universal_ipc::tower_atomic::TowerAtomicClient;
    /// # use serde_json::json;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = TowerAtomicClient::connect("/primal/test").await?;
    /// let result = client.call("add", json!({"a": 5, "b": 3})).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call(&self, method: &str, params: Value) -> IpcResult<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let request = JsonRpcRequest::new(method, Some(params), id);

        // Serialize request
        let request_json = serde_json::to_string(&request)
            .map_err(|e| IpcError::Other(format!("Failed to serialize request: {e}")))?;

        debug!("Sending JSON-RPC request: {}", request_json);

        // Send request and read response (lock scope minimized)
        let response_line = {
            let mut stream = self.stream.lock().await;

            stream
                .write_all(request_json.as_bytes())
                .await
                .map_err(|e| IpcError::Other(e.to_string()))?;
            stream.write_all(b"\n").await.map_err(|e| IpcError::Other(e.to_string()))?;

            let mut line = String::new();
            {
                let mut reader = BufReader::new(&mut *stream);
                reader.read_line(&mut line).await.map_err(|e| IpcError::Other(e.to_string()))?;
            }
            drop(stream);
            line
        };

        debug!("Received JSON-RPC response: {}", response_line);

        // Parse response
        let response: JsonRpcResponse = serde_json::from_str(&response_line)
            .map_err(|e| IpcError::Other(format!("Failed to parse response: {e}")))?;

        // Check for error
        if let Some(error) = response.error {
            return Err(IpcError::RpcError(error.message));
        }

        // Return result
        response.result.ok_or_else(|| IpcError::Other("Missing result in response".to_string()))
    }

    /// Call a JSON-RPC method without parameters
    pub async fn call_no_params(&self, method: &str) -> IpcResult<Value> {
        self.call(method, Value::Null).await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use serde_json::json;

    // Simple test handler
    struct MathService;

    #[async_trait]
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
        assert_eq!(req.id, json!(1));
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
        let result =
            handler.handle("add", json!({"a": 5, "b": 3})).await.expect("Add should succeed");
        assert_eq!(result, json!(8));

        // Test multiply
        let result = handler
            .handle("multiply", json!({"a": 4, "b": 7}))
            .await
            .expect("Multiply should succeed");
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
        assert_eq!(back.id, json!(42));
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

    #[async_trait]
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
            id: json!(1),
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
        #[async_trait]
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
        #[async_trait]
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
            id: json!(99),
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
        assert_eq!(r.id, json!(1001));
        assert!(r.params.is_none());
    }

    #[tokio::test]
    async fn handle_request_invalid_jsonrpc_version_message() {
        struct X;
        #[async_trait]
        impl JsonRpcHandler for X {
            async fn handle(&self, _method: &str, _params: Value) -> Result<Value, String> {
                Ok(json!(0))
            }
        }
        let req = JsonRpcRequest {
            jsonrpc: "2.1".into(),
            method: "m".into(),
            params: Some(json!({})),
            id: json!("abc"),
        };
        let resp = TowerAtomicServer::handle_request_for_test(req, &X).await;
        let e = resp.error.expect("err");
        assert_eq!(e.code, JsonRpcError::INVALID_REQUEST);
    }
}

/// Hand-crafted malformed JSON-RPC inputs for [`JsonRpcRequest`] deserialization (fuzz-style).
#[cfg(test)]
mod jsonrpc_parse_fuzz_style_tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::JsonRpcRequest;
    use serde_json::json;

    #[test]
    fn deserialize_rejects_malformed_json() {
        assert!(serde_json::from_str::<JsonRpcRequest>("{").is_err());
        assert!(serde_json::from_str::<JsonRpcRequest>("not json").is_err());
        assert!(serde_json::from_str::<JsonRpcRequest>("").is_err());
    }

    #[test]
    fn deserialize_rejects_missing_jsonrpc_field() {
        let s = r#"{"method":"m","id":1}"#;
        assert!(serde_json::from_str::<JsonRpcRequest>(s).is_err());
    }

    #[test]
    fn deserialize_rejects_missing_method_field() {
        let s = r#"{"jsonrpc":"2.0","id":1}"#;
        assert!(serde_json::from_str::<JsonRpcRequest>(s).is_err());
    }

    #[test]
    fn deserialize_accepts_various_id_types() {
        let cases = [
            r#"{"jsonrpc":"2.0","method":"a","id":null}"#,
            r#"{"jsonrpc":"2.0","method":"a","id":"s"}"#,
            r#"{"jsonrpc":"2.0","method":"a","id":true}"#,
            r#"{"jsonrpc":"2.0","method":"a","id":[1,2]}"#,
            r#"{"jsonrpc":"2.0","method":"a","id":{"x":1}}"#,
        ];
        for s in cases {
            let r: JsonRpcRequest = serde_json::from_str(s).unwrap();
            assert_eq!(r.jsonrpc, "2.0");
            assert_eq!(r.method, "a");
        }
    }

    #[test]
    fn deserialize_nested_json_deep_structure() {
        let mut inner = json!({});
        for _ in 0..120 {
            inner = json!({ "k": inner });
        }
        let v = json!({
            "jsonrpc": "2.0",
            "method": "deep",
            "id": 1,
            "params": inner
        });
        let s = v.to_string();
        let r: JsonRpcRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(r.method, "deep");
        assert!(r.params.is_some());
    }

    #[test]
    fn deserialize_very_long_method_name() {
        let long = "x".repeat(50_000);
        let v = json!({
            "jsonrpc": "2.0",
            "method": long,
            "id": 0
        });
        let s = v.to_string();
        let r: JsonRpcRequest = serde_json::from_str(&s).unwrap();
        assert_eq!(r.method.len(), 50_000);
    }

    #[test]
    fn deserialize_unicode_method_name() {
        let s = r#"{"jsonrpc":"2.0","method":"ping.тест.😀","id":1}"#;
        let r: JsonRpcRequest = serde_json::from_str(s).unwrap();
        assert_eq!(r.method, "ping.тест.😀");
    }
}
