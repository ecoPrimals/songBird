// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Unix Socket Listener for HTTP Gateway
//!
//! This module implements Unix socket listeners that receive JSON-RPC requests
//! from other primals and route them through the universal HTTP gateway.
//!
//! # Philosophy
//! - **Zero Hardcoding**: No primal names, socket paths are discovered at runtime
//! - **Capability-Based**: Each socket represents a capability, not a vendor
//! - **JSON-RPC 2.0**: Standard protocol for inter-primal communication
//! - **Async/Non-blocking**: Modern tokio-based async I/O
//!
//! # Architecture
//! ```text
//! Primal (Unix Socket) → Listener → Capability Router → Universal Proxy → External API
//!                           ↓
//!                    JSON-RPC 2.0
//!                  (Standard Protocol)
//! ```

use super::cache::ResponseCache;
use super::capability_router::{CapabilityRouter, Route};
use super::credentials::CredentialManager;
use super::rate_limiter::RateLimiter;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};

const JSONRPC_VERSION: &str = "2.0";

/// JSON-RPC 2.0 Request
///
/// Per spec, `id` is omitted for notifications (server MUST NOT reply).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    /// Create a new JSON-RPC error
    #[must_use]
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Parse error (-32700)
    #[must_use]
    pub fn parse_error() -> Self {
        Self::new(-32700, "Parse error")
    }

    /// Invalid request (-32600)
    #[must_use]
    pub fn invalid_request() -> Self {
        Self::new(-32600, "Invalid request")
    }

    /// Method not found (-32601)
    #[must_use]
    pub fn method_not_found() -> Self {
        Self::new(-32601, "Method not found")
    }

    /// Invalid params (-32602)
    #[must_use]
    pub fn invalid_params() -> Self {
        Self::new(-32602, "Invalid params")
    }

    /// Internal error (-32603)
    #[must_use]
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(-32603, message)
    }

    /// Server error (custom code)
    #[must_use]
    pub fn server_error(code: i32, message: impl Into<String>) -> Self {
        Self::new(code, message)
    }
}

/// Unix Socket Listener Configuration
#[derive(Debug, Clone)]
pub struct UnixListenerConfig {
    /// Socket path for this listener
    pub socket_path: PathBuf,

    /// Capability this socket handles (e.g., "ai:text-generation")
    pub capability_id: String,

    /// Maximum concurrent connections
    pub max_connections: usize,

    /// Request timeout in seconds
    pub timeout_secs: u64,
}

/// Unix Socket Listener - handles JSON-RPC requests from other primals
pub struct UnixSocketListener {
    config: UnixListenerConfig,
    router: Arc<CapabilityRouter>,
    rate_limiter: Arc<RateLimiter>,
    cache: Arc<ResponseCache>,
    credentials: Arc<CredentialManager>,
    http_client: songbird_http_client::SongbirdHttpClient,
    active_connections: Arc<RwLock<usize>>,
}

impl UnixSocketListener {
    /// Create a new Unix socket listener
    #[must_use]
    pub fn new(
        config: UnixListenerConfig,
        router: Arc<CapabilityRouter>,
        rate_limiter: Arc<RateLimiter>,
        cache: Arc<ResponseCache>,
        credentials: Arc<CredentialManager>,
        http_client: songbird_http_client::SongbirdHttpClient,
    ) -> Self {
        info!(
            "Creating Unix socket listener for capability '{}' at {:?}",
            config.capability_id, config.socket_path
        );

        Self {
            config,
            router,
            rate_limiter,
            cache,
            credentials,
            http_client,
            active_connections: Arc::new(RwLock::new(0)),
        }
    }

    /// Start listening for connections
    ///
    /// This is the main entry point for the listener.
    /// It binds to the Unix socket and spawns tasks for each incoming connection.
    /// # Errors
    ///
    /// Returns an error if the operation fails.
    pub async fn start(self: Arc<Self>) -> Result<()> {
        // Unconditional unlink before bind (prevents EADDRINUSE after crash)
        let _ = tokio::fs::remove_file(&self.config.socket_path).await;

        // Create parent directory if it doesn't exist
        if let Some(parent) = self.config.socket_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Bind to Unix socket
        let listener = UnixListener::bind(&self.config.socket_path)?;

        info!(
            "✅ Unix socket listener started: {:?} (capability: {})",
            self.config.socket_path, self.config.capability_id
        );

        // Accept connections in a loop
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    // Check connection limit
                    let active = *self.active_connections.read().await;
                    if active >= self.config.max_connections {
                        warn!(
                            "Connection limit reached ({}/{}), rejecting new connection",
                            active, self.config.max_connections
                        );
                        continue;
                    }

                    // Spawn task to handle this connection
                    let listener = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = listener.handle_connection(stream).await {
                            error!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    /// Handle a single connection
    async fn handle_connection(&self, mut stream: UnixStream) -> Result<()> {
        // Increment active connections
        {
            let mut active = self.active_connections.write().await;
            *active += 1;
            debug!("Active connections: {}", *active);
        }

        // Ensure we decrement on exit
        let _guard = ConnectionGuard {
            counter: self.active_connections.clone(),
        };

        let (reader, mut writer) = stream.split();
        let mut reader = BufReader::new(reader);
        let mut buffer = Vec::new();

        loop {
            buffer.clear();

            // Read one line (JSON-RPC request)
            let bytes_read = reader.read_until(b'\n', &mut buffer).await?;
            if bytes_read == 0 {
                debug!("Connection closed by client");
                break;
            }

            // Parse JSON-RPC request
            let request: JsonRpcRequest = match serde_json::from_slice(&buffer) {
                Ok(req) => req,
                Err(e) => {
                    warn!("Failed to parse JSON-RPC request: {}", e);
                    let error_response = JsonRpcResponse {
                        jsonrpc: JSONRPC_VERSION.into(),
                        result: None,
                        error: Some(JsonRpcError::parse_error()),
                        id: serde_json::Value::Null,
                    };
                    self.send_response_to_writer(&mut writer, error_response).await?;
                    continue;
                }
            };

            let is_notification = request.id.is_none();
            trace!(
                "Received request: method={}, id={:?}, notification={}",
                request.method, request.id, is_notification
            );

            let response = self.handle_request(request).await;

            if !is_notification {
                self.send_response_to_writer(&mut writer, response).await?;
            }
        }

        Ok(())
    }

    /// Handle a JSON-RPC request
    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone().unwrap_or(serde_json::Value::Null);

        if request.jsonrpc != JSONRPC_VERSION {
            return JsonRpcResponse {
                jsonrpc: JSONRPC_VERSION.into(),
                result: None,
                error: Some(JsonRpcError::invalid_request()),
                id,
            };
        }

        match request.method.as_str() {
            "proxy" | "http.proxy" => self.handle_proxy_request(request).await,
            "ping" => self.handle_ping_request(request).await,
            "capabilities" => self.handle_capabilities_request(request).await,
            _ => JsonRpcResponse {
                jsonrpc: JSONRPC_VERSION.into(),
                result: None,
                error: Some(JsonRpcError::method_not_found()),
                id,
            },
        }
    }

    /// Handle a proxy request (the main functionality)
    async fn handle_proxy_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.unwrap_or(serde_json::Value::Null);

        let params = match request.params.as_object() {
            Some(obj) => obj,
            None => {
                return JsonRpcResponse {
                    jsonrpc: JSONRPC_VERSION.into(),
                    result: None,
                    error: Some(JsonRpcError::invalid_params()),
                    id,
                };
            }
        };

        let capability_id =
            params.get("capability").and_then(|v| v.as_str()).unwrap_or(&self.config.capability_id);
        let http_method = params.get("method").and_then(|v| v.as_str()).unwrap_or("POST");
        let payload = params.get("payload").cloned();

        debug!("Proxy request: capability='{}', method='{}'", capability_id, http_method);

        let route = match self.router.route(capability_id).await {
            Ok(r) => r,
            Err(e) => {
                warn!("Routing failed for capability '{}': {}", capability_id, e);
                return JsonRpcResponse {
                    jsonrpc: JSONRPC_VERSION.into(),
                    result: None,
                    error: Some(JsonRpcError::server_error(-32000, format!("Routing failed: {e}"))),
                    id,
                };
            }
        };

        if let Err(e) = self.rate_limiter.check(&route.provider.id).await {
            warn!("Rate limit exceeded for provider '{}': {}", route.provider.id, e);
            return JsonRpcResponse {
                jsonrpc: JSONRPC_VERSION.into(),
                result: None,
                error: Some(JsonRpcError::server_error(-32001, "Rate limit exceeded")),
                id,
            };
        }

        let cache_key =
            format!("{}:{}", capability_id, serde_json::to_string(&payload).unwrap_or_default());
        if let Some(cached) = self.cache.get(&cache_key).await {
            debug!("Cache hit for key: {}", cache_key);
            return JsonRpcResponse {
                jsonrpc: JSONRPC_VERSION.into(),
                result: Some(cached),
                error: None,
                id,
            };
        }

        let result = self.make_external_request(&route, http_method, payload.as_ref()).await;

        match result {
            Ok(response_data) => JsonRpcResponse {
                jsonrpc: JSONRPC_VERSION.into(),
                result: Some(response_data),
                error: None,
                id,
            },
            Err(e) => {
                error!("External request failed: {}", e);
                JsonRpcResponse {
                    jsonrpc: JSONRPC_VERSION.into(),
                    result: None,
                    error: Some(JsonRpcError::internal_error(format!(
                        "External request failed: {e}"
                    ))),
                    id,
                }
            }
        }
    }

    /// Make an external HTTP request
    async fn make_external_request(
        &self,
        route: &Route,
        method: &str,
        payload: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value> {
        // Get backend configuration
        let backend = route
            .provider
            .backend
            .as_ref()
            .ok_or_else(|| anyhow!("Provider has no backend configuration"))?;

        // Get API key if needed
        let api_key = if backend.api_key_env.is_some() {
            self.credentials.get_api_key(&route.provider.id)
        } else {
            None
        };

        // Build headers
        let mut headers = std::collections::HashMap::new();

        // Add API key header if present
        if let Some(key) = api_key {
            headers.insert("Authorization".to_string(), format!("Bearer {key}"));
        }

        // Add custom headers
        for (name, value) in &backend.headers {
            headers.insert(name.clone(), value.clone());
        }

        // Add content-type if not specified and payload present
        if !backend.headers.contains_key("content-type") && payload.is_some() {
            headers.insert("Content-Type".to_string(), "application/json".to_string());
        }

        // Send request using Pure Rust HTTP client
        let response =
            self.http_client.request(method, &backend.base_url, headers, payload.cloned()).await?;

        // Extract status and body from Pure Rust HTTP response
        let status = response.status;
        let body = response.body;

        if !(200..300).contains(&status) {
            return Err(anyhow!("External API returned error: {status} - {body:?}"));
        }

        Ok(body)
    }

    async fn handle_ping_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.into(),
            result: Some(serde_json::json!({
                "status": "ok",
                "primal": "songbird",
                "capability": self.config.capability_id,
                "active_connections": *self.active_connections.read().await,
            })),
            error: None,
            id: request.id.unwrap_or(serde_json::Value::Null),
        }
    }

    async fn handle_capabilities_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let capabilities = self.router.list_capabilities().await;

        JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.into(),
            result: Some(serde_json::json!({
                "capabilities": capabilities,
            })),
            error: None,
            id: request.id.unwrap_or(serde_json::Value::Null),
        }
    }

    /// Send a JSON-RPC response to a writer
    async fn send_response_to_writer<W>(
        &self,
        writer: &mut W,
        response: JsonRpcResponse,
    ) -> Result<()>
    where
        W: tokio::io::AsyncWrite + Unpin,
    {
        let response_bytes = serde_json::to_vec(&response)?;
        writer.write_all(&response_bytes).await?;
        writer.write_all(b"\n").await?;
        Ok(())
    }
}

/// Guard to ensure connection counter is decremented
struct ConnectionGuard {
    counter: Arc<RwLock<usize>>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let counter = self.counter.clone();
        tokio::spawn(async move {
            let mut active = counter.write().await;
            *active = active.saturating_sub(1);
        });
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_error_creation() {
        let error = JsonRpcError::parse_error();
        assert_eq!(error.code, -32700);
        assert_eq!(error.message, "Parse error");
    }

    #[test]
    fn jsonrpc_error_standard_codes_and_messages() {
        assert_eq!(JsonRpcError::invalid_request().code, -32600);
        assert_eq!(JsonRpcError::method_not_found().code, -32601);
        assert_eq!(JsonRpcError::invalid_params().code, -32602);
        let internal = JsonRpcError::internal_error("boom");
        assert_eq!(internal.code, -32603);
        assert_eq!(internal.message, "boom");
        let custom = JsonRpcError::server_error(-32000, "routing");
        assert_eq!(custom.code, -32000);
        assert_eq!(custom.message, "routing");
    }

    #[test]
    fn jsonrpc_error_new_sets_message() {
        let e = JsonRpcError::new(42, "hello");
        assert_eq!(e.code, 42);
        assert_eq!(e.message, "hello");
        assert!(e.data.is_none());
    }

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: JSONRPC_VERSION.into(),
            method: "proxy".to_string(),
            params: serde_json::json!({"capability": "ai:text-generation"}),
            id: Some(serde_json::json!(1)),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"proxy\""));
    }

    #[test]
    fn test_jsonrpc_response_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.into(),
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
            id: serde_json::json!(1),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn jsonrpc_notification_omits_id_in_json() {
        let request = JsonRpcRequest {
            jsonrpc: JSONRPC_VERSION.into(),
            method: "ping".to_string(),
            params: serde_json::Value::Null,
            id: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(!json.contains("\"id\""), "notification should skip id: {json}");
    }
}
