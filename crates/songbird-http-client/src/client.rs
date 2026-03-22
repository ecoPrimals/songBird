// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP/HTTPS client implementation
//!
//! ## Features
//!
//! - Adaptive User-Agent headers
//! - Domain-based header routing
//! - Bot protection bypass
//! - Optional redirect following
//! - Configurable timeouts

use crate::connection::{HttpConnection, HttpsConnection};
use crate::crypto::{BearDogProvider, CryptoCapability};
use crate::error::{Error, Result};
use crate::http_config::{HttpClientConfig, RedirectMode};
use crate::redirect::RedirectHandler;
use crate::request::RequestBuilder;
use crate::response::ResponseParser;
use crate::tls::{config::TlsConfig, profiler::ServerProfiler};
use crate::types::HttpResponse;
use hyper::Uri;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tracing::{info, warn};

/// Songbird HTTP client with adaptive behavior
///
/// ## Configuration
///
/// The client supports multiple configuration modes:
/// - [`standard()`](HttpClientConfig::standard) - Sensible defaults with User-Agent and domain rules
/// - [`browser_like()`](HttpClientConfig::browser_like) - Mimics browser behavior for web scraping
/// - [`api()`](HttpClientConfig::api) - Optimized for REST API calls
/// - [`minimal()`](HttpClientConfig::minimal) - No default headers
#[derive(Clone)]
pub struct SongbirdHttpClient {
    crypto: Arc<dyn CryptoCapability>,
    tls_config: TlsConfig,
    http_config: HttpClientConfig,
    /// Profiler for adaptive server learning (future feature)
    #[allow(
        dead_code,
        reason = "reserved for adaptive profiling; field retained on struct for future wiring"
    )]
    profiler: Option<Arc<ServerProfiler>>,
}

impl std::fmt::Debug for SongbirdHttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SongbirdHttpClient")
            .field("crypto", &"<dyn CryptoCapability>")
            .field("tls_config", &self.tls_config)
            .field(
                "http_config",
                &format_args!("HttpClientConfig(ua={})", self.http_config.user_agent.len()),
            )
            .field("profiler", &self.profiler.is_some())
            .finish()
    }
}

impl SongbirdHttpClient {
    /// Create a new Songbird HTTP client with crypto capability discovery
    ///
    /// Uses standard HTTP configuration with adaptive User-Agent and domain rules.
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Path to crypto provider socket (e.g., "/tmp/beardog.sock")
    ///
    /// # Note
    ///
    /// This client uses the `CryptoCapability` trait for agnostic crypto operations.
    /// The underlying provider can be `BearDog` or any other implementation.
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self::with_tls_config(socket_path, TlsConfig::default(), None)
    }

    /// Create from environment variable with standard HTTP config
    ///
    /// Automatically detects Neural API mode or Direct mode based on environment:
    /// - `BEARDOG_MODE=neural` (default): Routes through Neural API for `capability.call`
    /// - `BEARDOG_MODE=direct` (testing): Direct connection to `BearDog`
    ///
    /// Uses `NEURAL_API_SOCKET` or `BEARDOG_SOCKET` accordingly.
    pub fn from_env() -> Self {
        info!("🌐 Creating Songbird HTTP client from environment");

        Self {
            crypto: Arc::new(BearDogProvider::from_env()),
            tls_config: TlsConfig::default(),
            http_config: HttpClientConfig::standard(),
            profiler: None,
        }
    }

    /// Create with custom TLS config and optional profiler (uses standard HTTP config)
    pub fn with_tls_config(
        _socket_path: impl Into<String>,
        tls_config: TlsConfig,
        profiler: Option<Arc<ServerProfiler>>,
    ) -> Self {
        info!(
            "🎛️  Creating Songbird HTTP client with {:?} strategy",
            tls_config.extension_strategy
        );
        if profiler.is_some() {
            info!("🧠 Adaptive learning enabled (profiler attached)");
        }

        // v5.28.0: ALWAYS use environment-based routing (TRUE PRIMAL pattern)
        // This ensures capability.call routing via Neural API
        // The socket_path parameter is ignored - routing determined by BEARDOG_MODE
        Self {
            crypto: Arc::new(BearDogProvider::from_env()),
            tls_config,
            http_config: HttpClientConfig::standard(),
            profiler,
        }
    }

    /// Create with custom HTTP configuration
    ///
    /// Use this for granular control over User-Agent, headers, and routing behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use songbird_http_client::{SongbirdHttpClient, HttpClientConfig};
    ///
    /// // Browser-like behavior for web scraping
    /// let client = SongbirdHttpClient::with_http_config(HttpClientConfig::browser_like());
    ///
    /// // API-focused with JSON defaults
    /// let client = SongbirdHttpClient::with_http_config(HttpClientConfig::api());
    ///
    /// // Custom configuration
    /// let client = SongbirdHttpClient::with_http_config(
    ///     HttpClientConfig::standard()
    ///         .with_user_agent("MyApp/1.0")
    ///         .with_default_header("X-API-Key", "secret")
    /// );
    /// ```
    pub fn with_http_config(http_config: HttpClientConfig) -> Self {
        info!("🎛️  Creating Songbird HTTP client with custom HTTP config");
        info!("   User-Agent: {}", http_config.user_agent);
        info!("   Default headers: {}", http_config.default_headers.len());
        info!("   Domain rules: {}", http_config.header_rules.len());
        info!("   Redirect mode: {:?}", http_config.redirect_mode);

        Self {
            crypto: Arc::new(BearDogProvider::from_env()),
            tls_config: TlsConfig::default(),
            http_config,
            profiler: None,
        }
    }

    /// Create with both TLS and HTTP configuration
    pub fn with_full_config(
        tls_config: TlsConfig,
        http_config: HttpClientConfig,
        profiler: Option<Arc<ServerProfiler>>,
    ) -> Self {
        info!("🎛️  Creating Songbird HTTP client with full custom config");
        if profiler.is_some() {
            info!("🧠 Adaptive learning enabled (profiler attached)");
        }

        Self {
            crypto: Arc::new(BearDogProvider::from_env()),
            tls_config,
            http_config,
            profiler,
        }
    }

    /// Create with explicit crypto capability provider
    ///
    /// Use this when you want to provide your own `CryptoCapability` implementation.
    pub fn with_crypto(
        crypto: Arc<dyn CryptoCapability>,
        tls_config: TlsConfig,
        http_config: HttpClientConfig,
        profiler: Option<Arc<ServerProfiler>>,
    ) -> Self {
        info!("🎛️  Creating Songbird HTTP client with custom crypto provider");
        if profiler.is_some() {
            info!("🧠 Adaptive learning enabled (profiler attached)");
        }

        Self {
            crypto,
            tls_config,
            http_config,
            profiler,
        }
    }

    // Backward compatibility alias
    #[doc(hidden)]
    #[deprecated(since = "0.2.0", note = "Use with_tls_config instead")]
    pub fn with_config(
        socket_path: impl Into<String>,
        config: TlsConfig,
        profiler: Option<Arc<ServerProfiler>>,
    ) -> Self {
        Self::with_tls_config(socket_path, config, profiler)
    }

    /// Get the current HTTP configuration
    #[must_use]
    pub const fn http_config(&self) -> &HttpClientConfig {
        &self.http_config
    }

    /// Get mutable reference to HTTP configuration
    #[must_use]
    pub const fn http_config_mut(&mut self) -> &mut HttpClientConfig {
        &mut self.http_config
    }

    /// Make an HTTP/HTTPS request
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `url` - Full URL
    /// * `headers` - Request headers
    /// * `body` - Optional request body
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - URL parsing fails
    /// - DNS resolution fails
    /// - TLS handshake fails
    /// - Connection cannot be established
    /// - Request or response is malformed
    pub async fn request(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<serde_json::Value>,
    ) -> Result<HttpResponse> {
        self.request_ref(method, url, &headers, body.as_ref()).await
    }

    /// Same as [`Self::request`], but borrows headers and body to avoid cloning on hot paths
    /// (e.g. redirect following).
    async fn request_ref(
        &self,
        method: &str,
        url: &str,
        headers: &HashMap<String, String>,
        body: Option<&serde_json::Value>,
    ) -> Result<HttpResponse> {
        info!("🌐 HTTP {} {}", method, url);

        // Parse URL into URI
        let parsed_uri: Uri = url.parse().map_err(|e| Error::InvalidUrl(format!("{e}")))?;

        let scheme = parsed_uri
            .scheme_str()
            .ok_or_else(|| Error::InvalidUrl("Missing scheme".to_string()))?;
        let host =
            parsed_uri.host().ok_or_else(|| Error::InvalidUrl("Missing host".to_string()))?;
        let port = parsed_uri.port_u16().unwrap_or(if scheme == "https" {
            443
        } else {
            80
        });

        // Enhanced logging for debugging port issues
        info!("🔌 Connection details:");
        info!("   URL: {}", url);
        info!("   Scheme: {}", scheme);
        info!("   Host: {}", host);
        info!(
            "   Port: {} ({})",
            port,
            if parsed_uri.port_u16().is_some() {
                "explicit"
            } else {
                "default"
            }
        );

        // For HTTPS, perform TLS handshake (TCP connection created inside with fallback)
        if scheme == "https" {
            if port != 443 {
                warn!("⚠️  Non-standard HTTPS port: {} (expected 443)", port);
            }
            return self.https_request(host, port, &parsed_uri, method, headers, body).await;
        }

        // For HTTP, establish plain TCP connection
        let addr = format!("{host}:{port}");
        let tcp_stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| Error::Connection(format!("Failed to connect to {addr}: {e}")))?;

        // Use plain connection for HTTP
        self.http_request(tcp_stream, &parsed_uri, method, headers, body).await
    }

    /// Make an HTTP/HTTPS request with automatic redirect following
    ///
    /// This method wraps `request()` and automatically follows HTTP redirects
    /// (301, 302, 303, 307, 308) based on the configured `redirect_mode`.
    ///
    /// # Arguments
    ///
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `url` - Full URL
    /// * `headers` - Request headers
    /// * `body` - Optional request body
    ///
    /// # Redirect Behavior
    ///
    /// - [`RedirectMode::None`]: Returns redirect response as-is
    /// - [`RedirectMode::Follow`]: Follows all redirects (max configured)
    /// - [`RedirectMode::SameOrigin`]: Only follows redirects to same origin
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails (see [`request`](Self::request) for details)
    /// or if maximum redirect limit is exceeded.
    pub async fn request_follow_redirects(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<serde_json::Value>,
    ) -> Result<HttpResponse> {
        let redirect_handler = RedirectHandler::new(self.http_config.max_redirects as usize);
        let mut current_url = url.to_string();
        let mut redirects_followed = 0;

        loop {
            // Make the request (borrow headers/body — no per-hop clone)
            let response = self.request_ref(method, &current_url, &headers, body.as_ref()).await?;

            // Check if we should follow this redirect
            if !redirect_handler.should_follow(
                &response,
                redirects_followed,
                self.http_config.redirect_mode,
            ) {
                // Check if it's actually a redirect that we chose not to follow
                if RedirectHandler::is_redirect_status(response.status) {
                    match self.http_config.redirect_mode {
                        RedirectMode::None => {
                            info!(
                                "↩️  Redirect received ({}), returning as-is (redirect_mode=None)",
                                response.status
                            );
                        }
                        RedirectMode::SameOrigin => {
                            if let Some(location) = response.headers.get("location") {
                                let new_url = RedirectHandler::resolve_url(&current_url, location)?;
                                if !RedirectHandler::is_same_origin(&current_url, &new_url)
                                    .unwrap_or(false)
                                {
                                    info!(
                                        "↩️  Cross-origin redirect, returning as-is (redirect_mode=SameOrigin)"
                                    );
                                }
                            }
                        }
                        RedirectMode::Follow => {
                            if redirects_followed >= self.http_config.max_redirects as usize {
                                warn!(
                                    "⚠️  Maximum redirects ({}) reached",
                                    self.http_config.max_redirects
                                );
                            }
                        }
                    }
                }
                return Ok(response);
            }

            // Extract Location header
            let location = response.headers.get("location").ok_or_else(|| {
                Error::HttpProtocol("Redirect without Location header".to_string())
            })?;

            // Resolve relative URLs
            let new_url = RedirectHandler::resolve_url(&current_url, location)?;

            info!(
                "↪️  Following redirect {}/{}: {} -> {}",
                redirects_followed + 1,
                self.http_config.max_redirects,
                response.status,
                new_url
            );

            current_url = new_url;
            redirects_followed += 1;
        }
    }

    /// Make HTTPS request with TLS
    async fn https_request(
        &self,
        host: &str,
        port: u16,
        uri: &Uri,
        method: &str,
        headers: &HashMap<String, String>,
        body: Option<&serde_json::Value>,
    ) -> Result<HttpResponse> {
        let https_conn = HttpsConnection::new(
            self.crypto.clone(),
            self.tls_config.clone(),
            self.profiler.clone(),
        );

        https_conn
            .execute(
                host,
                port,
                uri,
                method,
                headers,
                body,
                |uri, method, headers, body| self.build_http_request(uri, method, headers, body),
                Self::parse_http_response,
            )
            .await
    }

    /// Attempt TLS handshake with progressive fallback on failure
    ///
    /// CRITICAL FIX (Jan 26, 2026): Each retry attempt creates a FRESH TCP connection!
    /// Bug was: reusing the same TCP stream caused reading stale buffered data on retries.
    /// Make HTTP request without TLS
    async fn http_request(
        &self,
        tcp_stream: TcpStream,
        uri: &Uri,
        method: &str,
        headers: &HashMap<String, String>,
        body: Option<&serde_json::Value>,
    ) -> Result<HttpResponse> {
        HttpConnection::execute(tcp_stream, uri, method, headers, body).await
    }

    /// Build HTTP request bytes with adaptive headers
    ///
    /// Uses `HttpClientConfig` to apply:
    /// - Default User-Agent
    /// - Domain-specific header rules
    /// - Caller-provided headers (override everything)
    fn build_http_request(
        &self,
        uri: &Uri,
        method: &str,
        caller_headers: &HashMap<String, String>,
        body: Option<&serde_json::Value>,
    ) -> Result<Vec<u8>> {
        RequestBuilder::build(uri, method, &self.http_config, caller_headers, body)
    }

    /// Parse HTTP response bytes
    fn parse_http_response(data: &[u8]) -> Result<HttpResponse> {
        ResponseParser::parse(data)
    }

    /// Convenience method for GET requests
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails (see [`request`](Self::request) for details).
    pub async fn get(&self, url: &str) -> Result<HttpResponse> {
        self.request("GET", url, HashMap::new(), None).await
    }

    /// Convenience method for POST requests
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails (see [`request`](Self::request) for details).
    pub async fn post(&self, url: &str, body: serde_json::Value) -> Result<HttpResponse> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.request("POST", url, headers, Some(body)).await
    }

    /// Convenience method for PUT requests
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails (see [`request`](Self::request) for details).
    pub async fn put(&self, url: &str, body: serde_json::Value) -> Result<HttpResponse> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.request("PUT", url, headers, Some(body)).await
    }

    /// Convenience method for DELETE requests
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails (see [`request`](Self::request) for details).
    pub async fn delete(&self, url: &str) -> Result<HttpResponse> {
        self.request("DELETE", url, HashMap::new(), None).await
    }

    /// Convenience method for PATCH requests
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails (see [`request`](Self::request) for details).
    pub async fn patch(&self, url: &str, body: serde_json::Value) -> Result<HttpResponse> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.request("PATCH", url, headers, Some(body)).await
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::http_config::HttpClientConfig;

    #[test]
    fn test_client_creation() {
        let _client = SongbirdHttpClient::new("/tmp/beardog.sock");
        // Client created successfully if we got here
    }

    #[test]
    fn test_build_http_request() {
        let client = SongbirdHttpClient::new("/tmp/beardog.sock");
        let uri: Uri = "http://example.com/test".parse().unwrap();
        let headers = HashMap::new();

        let request = client.build_http_request(&uri, "GET", &headers, None).unwrap();
        let request_str = String::from_utf8_lossy(&request);

        assert!(request_str.contains("GET /test HTTP/1.1"));
        assert!(request_str.contains("Host: example.com"));
    }

    #[test]
    fn test_parse_http_response() {
        let response_data =
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"result\":\"ok\"}";

        let response = SongbirdHttpClient::parse_http_response(response_data).unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.headers.get("content-type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_resolve_redirect_url_absolute() {
        let _handler = RedirectHandler::new(10);

        // Absolute URL should be returned as-is
        let resolved =
            RedirectHandler::resolve_url("https://example.com/path", "https://other.com/new-path")
                .unwrap();
        assert_eq!(resolved, "https://other.com/new-path");
    }

    #[test]
    fn test_resolve_redirect_url_absolute_path() {
        let _handler = RedirectHandler::new(10);

        // Absolute path (starts with /) should use base's scheme and host
        let resolved =
            RedirectHandler::resolve_url("https://example.com/old-path", "/new-path").unwrap();
        assert_eq!(resolved, "https://example.com/new-path");
    }

    #[test]
    fn test_resolve_redirect_url_relative_path() {
        let _handler = RedirectHandler::new(10);

        // Relative path should be resolved relative to base
        let resolved =
            RedirectHandler::resolve_url("https://example.com/path/to/page", "other-page").unwrap();
        assert_eq!(resolved, "https://example.com/path/to/other-page");
    }

    #[test]
    fn test_resolve_redirect_url_with_port() {
        let _handler = RedirectHandler::new(10);

        // Preserve port in redirect
        let resolved =
            RedirectHandler::resolve_url("https://example.com:8443/path", "/new-path").unwrap();
        assert_eq!(resolved, "https://example.com:8443/new-path");
    }

    #[test]
    fn test_extract_host_from_location() {
        let _handler = RedirectHandler::new(10);

        // Absolute URL
        let host = RedirectHandler::extract_host("https://other.com/path", "https://example.com");
        assert_eq!(host, Some("other.com".to_string()));

        // Relative URL should use base host
        let host = RedirectHandler::extract_host("/new-path", "https://example.com/old-path");
        assert_eq!(host, Some("example.com".to_string()));
    }

    #[test]
    fn songbird_http_client_debug_includes_tls_and_config() {
        let client = SongbirdHttpClient::new("/tmp/beardog.sock");
        let dbg = format!("{client:?}");
        assert!(dbg.contains("SongbirdHttpClient"));
        assert!(dbg.contains("tls_config"));
    }

    #[test]
    fn with_http_config_preserves_user_agent_and_headers() {
        let cfg = HttpClientConfig::minimal().with_user_agent("UA-Test/1");
        let client = SongbirdHttpClient::with_http_config(cfg);
        assert_eq!(client.http_config().user_agent, "UA-Test/1");
    }

    #[tokio::test]
    async fn request_rejects_invalid_url() {
        let client = SongbirdHttpClient::new("/tmp/beardog.sock");
        let err = client.request("GET", "not-a-valid-url", HashMap::new(), None).await.unwrap_err();
        assert!(matches!(err, Error::InvalidUrl(_)));
    }

    #[tokio::test]
    async fn request_rejects_missing_scheme() {
        let client = SongbirdHttpClient::new("/tmp/beardog.sock");
        let err = client.request("GET", "example.com", HashMap::new(), None).await.unwrap_err();
        assert!(matches!(err, Error::InvalidUrl(_)));
    }
}
