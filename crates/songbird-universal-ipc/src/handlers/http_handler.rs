//! HTTP/HTTPS IPC Handler - Deep Solution Implementation
//!
//! This module provides JSON-RPC 2.0 handlers for HTTP/HTTPS requests via IPC,
//! exposing Songbird's Pure Rust TLS 1.3 capability to the ecosystem.
//!
//! ## Architecture
//!
//! ```text
//! biomeOS → JSON-RPC → IPC Handler → HTTP Client Factory → BearDog (crypto)
//!                                  ↓
//!                        Pure Rust TLS 1.3 (Tower Atomic)
//! ```
//!
//! ## Design Principles
//!
//! 1. **Capability-Based Discovery** - No hardcoded `BearDog` endpoints
//! 2. **Factory Pattern** - Dependency injection for testability
//! 3. **Trait-Based Abstraction** - Not concrete types
//! 4. **Proper Error Handling** - No unwrap/expect
//! 5. **Modern Async** - tokio, async/await throughout

use crate::error::IpcResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, error, info, instrument};

// ============================================================================
// Request/Response Types
// ============================================================================

/// HTTP request parameters from JSON-RPC
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpRequestParams {
    /// Target URL (http:// or https://)
    pub url: String,

    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    #[serde(default = "default_method")]
    pub method: String,

    /// HTTP headers
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Request body (optional)
    #[serde(default)]
    pub body: Option<String>,

    /// Timeout in milliseconds
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_method() -> String {
    "GET".to_string()
}

fn default_timeout_ms() -> u64 {
    30_000 // 30 seconds
}

/// HTTP response for JSON-RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponseResult {
    /// HTTP status code
    pub status_code: u16,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Response body as string
    pub body: String,

    /// Request elapsed time in milliseconds
    pub elapsed_ms: u128,
}

// ============================================================================
// Abstraction: HTTP Client Capability
// ============================================================================

/// Trait for HTTP client capability
///
/// This abstraction allows for:
/// - Multiple implementations (Pure Rust, mocked, etc.)
/// - Easy testing via dependency injection
/// - Future evolution (HTTP/2, HTTP/3, etc.)
#[async_trait]
pub trait HttpClientCapability: Send + Sync {
    /// Execute HTTP request
    async fn request(
        &self,
        method: &str,
        url: &str,
        headers: &HashMap<String, String>,
        body: Option<&[u8]>,
    ) -> IpcResult<HttpResponse>;
}

/// HTTP response from client
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,
}

// ============================================================================
// Abstraction: Crypto Capability Discovery
// ============================================================================

/// Trait for discovering crypto capabilities at runtime
///
/// This enables:
/// - No hardcoded `BearDog` endpoints
/// - Capability-based discovery
/// - Multiple discovery backends (env, IPC, mDNS)
#[async_trait]
pub trait CryptoCapabilityDiscovery: Send + Sync {
    /// Discover crypto provider by capability
    async fn discover(&self, capability: &str) -> IpcResult<String>;
}

// ============================================================================
// Factory Pattern: HTTP Client Factory
// ============================================================================

/// Factory for creating HTTP clients
///
/// Benefits:
/// - Dependency injection
/// - Easy mocking in tests
/// - Centralized client configuration
#[async_trait]
pub trait HttpClientFactory: Send + Sync {
    /// Create HTTP client with automatic crypto provider discovery
    async fn create_client(&self) -> IpcResult<Arc<dyn HttpClientCapability>>;
}

// ============================================================================
// Production Implementation: Songbird HTTP Client Wrapper
// ============================================================================

/// Production HTTP client using songbird-http-client
pub struct SongbirdHttpClient {
    inner: Arc<songbird_http_client::SongbirdHttpClient>,
}

impl SongbirdHttpClient {
    /// Create new client with `BearDog` crypto provider at socket path
    #[must_use]
    pub fn new(beardog_socket: &str) -> Self {
        let inner = songbird_http_client::SongbirdHttpClient::new(beardog_socket);

        Self {
            inner: Arc::new(inner),
        }
    }
}

#[async_trait]
impl HttpClientCapability for SongbirdHttpClient {
    #[instrument(skip(self, body), fields(method = %method, url = %url))]
    async fn request(
        &self,
        method: &str,
        url: &str,
        headers: &HashMap<String, String>,
        body: Option<&[u8]>,
    ) -> IpcResult<HttpResponse> {
        debug!("Making HTTP request: {} {} with {} headers", method, url, headers.len());

        // FIX: Parse body once, then call request() with headers (Issue #2 - Jan 28, 2026)
        let body_json = body
            .and_then(|b| std::str::from_utf8(b).ok())
            .and_then(|s| serde_json::from_str(s).ok());

        // Use Pure Rust TLS 1.3 via Tower Atomic pattern
        // FIX: Call request() directly (NOT convenience methods like post()) to preserve headers
        let response = self
            .inner
            .request(method, url, headers.clone(), body_json)
            .await
            .map_err(|e| {
                error!("HTTP request failed: {}", e);
                crate::error::IpcError::Internal(format!("HTTP request failed: {e}"))
            })?;

        Ok(HttpResponse {
            status_code: response.status,
            headers: response.headers,
            body: response.body,
        })
    }
}

// ============================================================================
// Production Implementation: Default Factory
// ============================================================================

/// Default factory using runtime capability discovery
pub struct DefaultHttpClientFactory {
    discovery: Arc<dyn CryptoCapabilityDiscovery>,
}

impl DefaultHttpClientFactory {
    pub fn new(discovery: Arc<dyn CryptoCapabilityDiscovery>) -> Self {
        Self {
            discovery,
        }
    }
}

#[async_trait]
impl HttpClientFactory for DefaultHttpClientFactory {
    #[instrument(skip(self))]
    async fn create_client(&self) -> IpcResult<Arc<dyn HttpClientCapability>> {
        // Discover crypto provider at runtime (capability-based!)
        let beardog_socket = self.discovery.discover("crypto.signing").await?;

        info!("Discovered crypto provider at: {}", beardog_socket);

        let client = SongbirdHttpClient::new(&beardog_socket);
        Ok(Arc::new(client))
    }
}

// ============================================================================
// Production Implementation: Environment-Based Discovery
// ============================================================================

/// Discovers crypto capability via environment variables
///
/// Priority:
/// 1. `CRYPTO_ENDPOINT` env var
/// 2. `BEARDOG_SOCKET` env var
/// 3. Default: /primal/beardog
pub struct EnvCryptoDiscovery;

#[async_trait]
impl CryptoCapabilityDiscovery for EnvCryptoDiscovery {
    async fn discover(&self, capability: &str) -> IpcResult<String> {
        debug!("Discovering capability via environment: {}", capability);

        // Try capability-based env var first
        let env_key = format!("{}_ENDPOINT", capability.to_uppercase().replace('.', "_"));
        if let Ok(endpoint) = std::env::var(&env_key) {
            info!("Found {} at {} (via {})", capability, endpoint, env_key);
            return Ok(endpoint);
        }

        // Fall back to BEARDOG_SOCKET
        if let Ok(socket) = std::env::var("BEARDOG_SOCKET") {
            info!("Found crypto provider at {} (via BEARDOG_SOCKET)", socket);
            return Ok(socket);
        }

        // Default to standard primal namespace
        let default = "/primal/beardog".to_string();
        info!("Using default crypto provider: {}", default);
        Ok(default)
    }
}

// ============================================================================
// Handler Implementation
// ============================================================================

/// HTTP/HTTPS IPC Handler
///
/// Handles JSON-RPC 2.0 methods:
/// - `http.request` - Full HTTP/HTTPS request
/// - `http.get` - GET request shorthand
/// - `http.post` - POST request shorthand
pub struct HttpHandler {
    factory: Arc<dyn HttpClientFactory>,
}

impl HttpHandler {
    /// Create new handler with given factory (dependency injection)
    pub fn new(factory: Arc<dyn HttpClientFactory>) -> Self {
        Self {
            factory,
        }
    }

    /// Create handler with default environment-based discovery
    #[must_use]
    pub fn with_default_discovery() -> Self {
        let discovery = Arc::new(EnvCryptoDiscovery);
        let factory = Arc::new(DefaultHttpClientFactory::new(discovery));
        Self::new(factory)
    }

    /// Handle http.request method
    #[instrument(skip(self), fields(url = %params.url, method = %params.method))]
    pub async fn handle_request(&self, params: HttpRequestParams) -> IpcResult<HttpResponseResult> {
        let start = Instant::now();

        info!("IPC http.request: {} {}", params.method, params.url);
        debug!("Headers: {:?}", params.headers);

        // Create client via factory (capability discovery happens here)
        let client = self.factory.create_client().await?;

        // Make request via Pure Rust TLS
        let body = params.body.as_ref().map(std::string::String::as_bytes);
        let response = client.request(&params.method, &params.url, &params.headers, body).await?;

        let elapsed = start.elapsed();

        info!(
            "IPC http.request completed: {} {} in {}ms",
            response.status_code,
            params.url,
            elapsed.as_millis()
        );

        Ok(HttpResponseResult {
            status_code: response.status_code,
            headers: response.headers,
            body: response.body.to_string(),
            elapsed_ms: elapsed.as_millis(),
        })
    }

    /// Handle http.get method (convenience)
    #[instrument(skip(self), fields(url = %url))]
    pub async fn handle_get(&self, url: &str) -> IpcResult<HttpResponseResult> {
        let params = HttpRequestParams {
            url: url.to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
            timeout_ms: 30_000,
        };

        self.handle_request(params).await
    }

    /// Handle http.post method (convenience)
    #[instrument(skip(self, body), fields(url = %url))]
    pub async fn handle_post(
        &self,
        url: &str,
        body: &str,
        content_type: Option<&str>,
        caller_headers: HashMap<String, String>,
    ) -> IpcResult<HttpResponseResult> {
        let mut headers = caller_headers; // FIX: Use caller's headers instead of empty HashMap
        if let Some(ct) = content_type {
            headers.insert("Content-Type".to_string(), ct.to_string());
        }

        let params = HttpRequestParams {
            url: url.to_string(),
            method: "POST".to_string(),
            headers,
            body: Some(body.to_string()),
            timeout_ms: 30_000,
        };

        self.handle_request(params).await
    }
}

// ============================================================================
// JSON-RPC Integration
// ============================================================================

#[async_trait]
impl crate::tower_atomic::JsonRpcHandler for HttpHandler {
    async fn handle(&self, method: &str, params: Value) -> Result<Value, String> {
        match method {
            "http.request" => {
                let params: HttpRequestParams =
                    serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

                let result = self.handle_request(params).await.map_err(|e| e.to_string())?;

                serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
            }
            "http.get" => {
                let url = params
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'url' parameter".to_string())?;

                let result = self.handle_get(url).await.map_err(|e| e.to_string())?;

                serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
            }
            "http.post" => {
                let url = params
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'url' parameter".to_string())?;

                let body = params
                    .get("body")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| "Missing 'body' parameter".to_string())?;

                let content_type = params.get("content_type").and_then(|v| v.as_str());

                // FIX: Extract headers from params (Issue #1 - Jan 28, 2026)
                let headers: HashMap<String, String> = params
                    .get("headers")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();

                let result =
                    self.handle_post(url, body, content_type, headers).await.map_err(|e| e.to_string())?;

                serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
            }
            _ => Err(format!("Unknown method: {method}")),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Mock HTTP client for testing
    struct MockHttpClient {
        responses: Vec<HttpResponse>,
        call_count: AtomicUsize,
    }

    impl MockHttpClient {
        fn new(responses: Vec<HttpResponse>) -> Self {
            Self {
                responses,
                call_count: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl HttpClientCapability for MockHttpClient {
        async fn request(
            &self,
            _method: &str,
            _url: &str,
            _headers: &HashMap<String, String>,
            _body: Option<&[u8]>,
        ) -> IpcResult<HttpResponse> {
            let count = self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(self.responses[count % self.responses.len()].clone())
        }
    }

    // Mock factory for testing
    struct MockClientFactory {
        client: Arc<dyn HttpClientCapability>,
    }

    #[async_trait]
    impl HttpClientFactory for MockClientFactory {
        async fn create_client(&self) -> IpcResult<Arc<dyn HttpClientCapability>> {
            Ok(Arc::clone(&self.client))
        }
    }

    #[tokio::test]
    async fn test_handle_get_request() {
        // Arrange
        let mock_response = HttpResponse {
            status_code: 200,
            headers: HashMap::new(),
            body: serde_json::json!("Hello, World!"),
        };

        let mock_client = Arc::new(MockHttpClient::new(vec![mock_response]));
        let factory = Arc::new(MockClientFactory {
            client: mock_client,
        });
        let handler = HttpHandler::new(factory);

        // Act
        let result = handler.handle_get("https://example.com").await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status_code, 200);
        assert_eq!(response.body, "\"Hello, World!\"");
    }

    #[tokio::test]
    async fn test_handle_post_request() {
        // Arrange
        let mock_response = HttpResponse {
            status_code: 201,
            headers: HashMap::new(),
            body: serde_json::json!("Created"),
        };

        let mock_client = Arc::new(MockHttpClient::new(vec![mock_response]));
        let factory = Arc::new(MockClientFactory {
            client: mock_client,
        });
        let handler = HttpHandler::new(factory);

        // Act
        let result = handler
            .handle_post("https://api.example.com", r#"{"key":"value"}"#, Some("application/json"), HashMap::new())
            .await;

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.status_code, 201);
        assert_eq!(response.body, "\"Created\"");
    }

    #[tokio::test]
    async fn test_environment_discovery() {
        // Arrange - Clear all env vars first
        std::env::remove_var("CRYPTO_SIGNING_ENDPOINT");
        std::env::remove_var("BEARDOG_SOCKET");

        std::env::set_var("CRYPTO_SIGNING_ENDPOINT", "/test/beardog");
        let discovery = EnvCryptoDiscovery;

        // Act
        let result = discovery.discover("crypto.signing").await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/test/beardog");

        // Cleanup
        std::env::remove_var("CRYPTO_SIGNING_ENDPOINT");
    }

    #[tokio::test]
    async fn test_default_discovery_fallback() {
        // Arrange - Clear any existing env vars
        std::env::remove_var("CRYPTO_SIGNING_ENDPOINT");
        std::env::remove_var("BEARDOG_SOCKET");
        let discovery = EnvCryptoDiscovery;

        // Act
        let result = discovery.discover("crypto.signing").await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/primal/beardog");
    }
}
