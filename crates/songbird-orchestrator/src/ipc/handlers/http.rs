//! HTTP/HTTPS Request Handler - Pure Rust Tower Atomic
//!
//! Exposes Songbird's Pure Rust TLS 1.3 HTTP client via Unix socket JSON-RPC.
//!
//! ## Purpose
//!
//! Enable all primals to make HTTPS requests via Tower Atomic pattern without
//! C dependencies (no reqwest, no openssl, no ring).
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  biomeOS / Squirrel / Any Primal                            │
//! │  (wants to call GitHub API, Anthropic, etc.)               │
//! └────────────────────┬────────────────────────────────────────┘
//!                      │ JSON-RPC over Unix socket
//!                      │ {"jsonrpc": "2.0", "method": "http.request", ...}
//! ┌────────────────────▼────────────────────────────────────────┐
//! │  Songbird IPC Handler (THIS FILE)                          │
//! │  - Parse request parameters                                 │
//! │  - Delegate to Pure Rust HTTP client                        │
//! │  - Return response                                          │
//! └────────────────────┬────────────────────────────────────────┘
//!                      │
//! ┌────────────────────▼────────────────────────────────────────┐
//! │  Songbird HTTP Client (songbird-http-client)               │
//! │  - Pure Rust TLS 1.3 handshake                             │
//! │  - HTTP/1.1 and HTTP/2                                     │
//! │  - Zero C dependencies                                      │
//! └────────────────────┬────────────────────────────────────────┘
//!                      │ Crypto RPC calls
//! ┌────────────────────▼────────────────────────────────────────┐
//! │  BearDog Crypto Provider                                    │
//! │  - X25519 ECDH                                             │
//! │  - ChaCha20-Poly1305 AEAD                                  │
//! │  - BLAKE3 HKDF                                             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## JSON-RPC API
//!
//! ### Method: `http.request`
//!
//! **Request**:
//! ```json
//! {
//!     "jsonrpc": "2.0",
//!     "method": "http.request",
//!     "params": {
//!         "method": "GET" | "POST" | "PUT" | "DELETE" | "HEAD" | "PATCH",
//!         "url": "https://api.github.com/...",
//!         "headers": {
//!             "Authorization": "Bearer ...",
//!             "Content-Type": "application/json"
//!         },
//!         "body": null | "base64-encoded-body"
//!     },
//!     "id": 1
//! }
//! ```
//!
//! **Response**:
//! ```json
//! {
//!     "jsonrpc": "2.0",
//!     "result": {
//!         "status": 200,
//!         "headers": {
//!             "content-type": "application/json",
//!             "content-length": "1234"
//!         },
//!         "body": "base64-encoded-response-body"
//!     },
//!     "id": 1
//! }
//! ```
//!
//! ## Standards Compliance
//!
//! - ✅ **JSON-RPC 2.0**: Standard protocol (wateringHole/PRIMAL_IPC_PROTOCOL.md)
//! - ✅ **Semantic Naming**: `http.request` format (wateringHole/SEMANTIC_METHOD_NAMING_STANDARD.md)
//! - ✅ **Tower Atomic**: Crypto delegation pattern
//! - ✅ **TRUE ecoBin**: Zero C dependencies
//! - ✅ **Pure Rust**: 100% safe Rust

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

use songbird_http_client::{BearDogClient, SongbirdHttpClient};

use crate::ipc::pure_rust_server::JsonRpcError;

/// HTTP method enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Patch,
}

impl HttpMethod {
    /// Parse HTTP method from string
    ///
    /// # Errors
    ///
    /// Returns error if method is not recognized
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "HEAD" => Ok(Self::Head),
            "PATCH" => Ok(Self::Patch),
            _ => Err(anyhow::anyhow!("Unsupported HTTP method: {}", s)),
        }
    }

    /// Convert to string representation
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
            Self::Patch => "PATCH",
        }
    }
}

/// HTTP request handler context
///
/// Holds the BearDog client for crypto operations
pub struct HttpHandler {
    beardog_client: Arc<BearDogClient>,
}

impl HttpHandler {
    /// Create new HTTP handler with BearDog crypto provider
    #[must_use]
    pub fn new(beardog_client: Arc<BearDogClient>) -> Self {
        Self {
            beardog_client,
        }
    }

    /// Handle `http.request` JSON-RPC method
    ///
    /// Makes HTTP/HTTPS requests using Pure Rust TLS 1.3.
    ///
    /// # Parameters
    ///
    /// - `method`: HTTP method (GET, POST, etc.)
    /// - `url`: Target URL (http:// or https://)
    /// - `headers`: Optional headers (name -> value)
    /// - `body`: Optional body (base64-encoded)
    ///
    /// # Returns
    ///
    /// Response with:
    /// - `status`: HTTP status code
    /// - `headers`: Response headers
    /// - `body`: Response body (base64-encoded)
    ///
    /// # Errors
    ///
    /// Returns JSON-RPC error if:
    /// - Parameters are invalid
    /// - HTTP request fails
    /// - Network error occurs
    /// - TLS handshake fails
    pub async fn handle_request(&self, params: Value) -> Result<Value, JsonRpcError> {
        // DEBUG: Log incoming params (Issue #1 & #2 investigation - Jan 28, 2026)
        tracing::info!(
            "🔍 handle_request → params: {}",
            serde_json::to_string(&params).unwrap_or_else(|_| "invalid json".to_string())
        );

        // 1. Parse parameters
        let method_str = params
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError::invalid_params("Missing 'method' parameter"))?;

        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| JsonRpcError::invalid_params("Missing 'url' parameter"))?;

        // Parse headers (optional)
        let headers: HashMap<String, String> = params
            .get("headers")
            .and_then(|h| h.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        // DEBUG: Log parsed headers (Issue #1 & #2 investigation - Jan 28, 2026)
        tracing::info!(
            "🔍 handle_request → method: {}, url: {}, headers: {:?}",
            method_str,
            url,
            headers
        );

        // Parse body (optional, base64-encoded)
        let body = params
            .get("body")
            .and_then(|b| b.as_str())
            .map(|b64| BASE64.decode(b64).context("Failed to decode base64 body"))
            .transpose()
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?
            .map(|bytes| {
                // Convert bytes to JSON Value (string)
                String::from_utf8(bytes)
                    .map(|s| serde_json::Value::String(s))
                    .unwrap_or(serde_json::Value::Null)
            });

        // 2. Parse HTTP method
        let method = HttpMethod::from_str(method_str)
            .map_err(|e| JsonRpcError::invalid_params(&e.to_string()))?;

        // 3. Create HTTP client with BearDog crypto provider
        // Note: SongbirdHttpClient::new takes a socket path string
        let beardog_socket_path = std::env::var("BEARDOG_SOCKET")
            .or_else(|_| std::env::var("SONGBIRD_BEARDOG_SOCKET"))
            .unwrap_or_else(|_| {
                let family_id = std::env::var("SONGBIRD_FAMILY_ID")
                    .or_else(|_| std::env::var("FAMILY_ID"))
                    .unwrap_or_else(|_| "default".to_string());
                format!("/tmp/beardog-{}.sock", family_id)
            });

        let client = SongbirdHttpClient::new(&beardog_socket_path);

        // 4. Make request
        let response = client
            .request(method.as_str(), url, headers, body)
            .await
            .map_err(|e| JsonRpcError::internal_error(&format!("HTTP request failed: {}", e)))?;

        // 5. Return response (body as base64)
        // Note: HttpResponse.body is serde_json::Value (already JSON)
        let body_base64 = match &response.body {
            serde_json::Value::String(s) => BASE64.encode(s.as_bytes()),
            serde_json::Value::Null => String::new(),
            other => BASE64.encode(other.to_string().as_bytes()),
        };

        Ok(json!({
            "status": response.status,
            "headers": response.headers,
            "body": body_base64
        }))
    }

    /// Handle `http.get` convenience method
    ///
    /// Shorthand for `http.request` with method=GET
    ///
    /// # Errors
    ///
    /// Returns error if request fails
    pub async fn handle_get(&self, params: Value) -> Result<Value, JsonRpcError> {
        let mut req_params = params;
        req_params["method"] = json!("GET");
        self.handle_request(req_params).await
    }

    /// Handle `http.post` convenience method
    ///
    /// Shorthand for `http.request` with method=POST
    ///
    /// # Errors
    ///
    /// Returns error if request fails
    pub async fn handle_post(&self, params: Value) -> Result<Value, JsonRpcError> {
        // DEBUG: Log http.post invocation (Issue #1 investigation - Jan 28, 2026)
        tracing::info!(
            "🔍 handle_post → incoming params: {}",
            serde_json::to_string(&params).unwrap_or_else(|_| "invalid json".to_string())
        );

        let mut req_params = params;
        req_params["method"] = json!("POST");

        // DEBUG: Log modified params (Issue #1 investigation - Jan 28, 2026)
        tracing::info!(
            "🔍 handle_post → modified params: {}",
            serde_json::to_string(&req_params).unwrap_or_else(|_| "invalid json".to_string())
        );

        self.handle_request(req_params).await
    }

    /// Handle `http.put` convenience method
    ///
    /// # Errors
    ///
    /// Returns error if request fails
    pub async fn handle_put(&self, params: Value) -> Result<Value, JsonRpcError> {
        let mut req_params = params;
        req_params["method"] = json!("PUT");
        self.handle_request(req_params).await
    }

    /// Handle `http.delete` convenience method
    ///
    /// # Errors
    ///
    /// Returns error if request fails
    pub async fn handle_delete(&self, params: Value) -> Result<Value, JsonRpcError> {
        let mut req_params = params;
        req_params["method"] = json!("DELETE");
        self.handle_request(req_params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_parsing() {
        assert_eq!(HttpMethod::from_str("GET").unwrap(), HttpMethod::Get);
        assert_eq!(HttpMethod::from_str("get").unwrap(), HttpMethod::Get);
        assert_eq!(HttpMethod::from_str("POST").unwrap(), HttpMethod::Post);
        assert_eq!(HttpMethod::from_str("PUT").unwrap(), HttpMethod::Put);
        assert_eq!(HttpMethod::from_str("DELETE").unwrap(), HttpMethod::Delete);
        assert_eq!(HttpMethod::from_str("HEAD").unwrap(), HttpMethod::Head);
        assert_eq!(HttpMethod::from_str("PATCH").unwrap(), HttpMethod::Patch);
        assert!(HttpMethod::from_str("INVALID").is_err());
    }

    #[test]
    fn test_http_method_as_str() {
        assert_eq!(HttpMethod::Get.as_str(), "GET");
        assert_eq!(HttpMethod::Post.as_str(), "POST");
        assert_eq!(HttpMethod::Put.as_str(), "PUT");
        assert_eq!(HttpMethod::Delete.as_str(), "DELETE");
        assert_eq!(HttpMethod::Head.as_str(), "HEAD");
        assert_eq!(HttpMethod::Patch.as_str(), "PATCH");
    }

    #[test]
    fn test_base64_encode_decode() {
        let original = b"Hello, World!";
        let encoded = BASE64.encode(original);
        let decoded = BASE64.decode(&encoded).unwrap();
        assert_eq!(original, &decoded[..]);
    }
}
