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
///
/// Per <https://www.jsonrpc.org/specification#request_object>, `id` is
/// omitted for notifications (the server MUST NOT reply).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request (with numeric id).
    pub fn new(method: impl Into<String>, params: Option<Value>, id: u64) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
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

                    let request = match serde_json::from_str::<JsonRpcRequest>(&line) {
                        Ok(req) => req,
                        Err(e) => {
                            let resp = JsonRpcResponse::error(
                                JsonRpcError {
                                    code: JsonRpcError::PARSE_ERROR,
                                    message: format!("Failed to parse request: {e}"),
                                    data: None,
                                },
                                Value::Null,
                            );
                            Self::write_response(&mut writer, &resp).await?;
                            continue;
                        }
                    };

                    let is_notification = request.is_notification();
                    debug!(
                        "JSON-RPC request: {} (notification={})",
                        request.method, is_notification
                    );
                    let response = Self::handle_request(request, &*handler).await;

                    if is_notification {
                        continue;
                    }

                    Self::write_response(&mut writer, &response).await?;
                }
                Err(e) => {
                    error!("Failed to read from socket: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Serialize and write a JSON-RPC response with safe fallback.
    ///
    /// If serialization fails (should never happen for our types), a
    /// hard-coded internal-error JSON is written so the client always
    /// sees a valid frame rather than a dropped connection.
    async fn write_response<W: tokio::io::AsyncWrite + Unpin>(
        writer: &mut W,
        response: &JsonRpcResponse,
    ) -> IpcResult<()> {
        const FALLBACK: &[u8] =
            b"{\"jsonrpc\":\"2.0\",\"error\":{\"code\":-32603,\"message\":\"Internal serialization error\"},\"id\":null}\n";

        match serde_json::to_vec(response) {
            Ok(mut buf) => {
                buf.push(b'\n');
                writer.write_all(&buf).await.map_err(|e| IpcError::Other(e.to_string()))?;
            }
            Err(e) => {
                error!("JSON-RPC response serialization failed: {e}");
                writer.write_all(FALLBACK).await.map_err(|e| IpcError::Other(e.to_string()))?;
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

        let id = id.unwrap_or(Value::Null);

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
#[path = "tower_atomic_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tower_atomic_jsonrpc_parse_fuzz_style_tests.rs"]
mod jsonrpc_parse_fuzz_style_tests;
