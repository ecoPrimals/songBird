//! HTTP/HTTPS client implementation
//!
//! ## Features
//!
//! - Adaptive User-Agent headers
//! - Domain-based header routing
//! - Bot protection bypass
//! - Optional redirect following
//! - Configurable timeouts

use crate::crypto::{BearDogProvider, CryptoCapability};
use crate::error::{Error, Result};
use crate::http_config::HttpClientConfig;
use crate::tls::{
    config::TlsConfig, handshake::TlsHandshake, profiler::ServerProfiler, record::TlsRecordLayer,
};
use crate::types::HttpResponse;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, Uri};
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tracing::{debug, error, info, trace, warn};

/// Songbird HTTP client with adaptive behavior
///
/// ## Configuration
///
/// The client supports multiple configuration modes:
/// - `standard()` - Sensible defaults with User-Agent and domain rules
/// - `browser_like()` - Mimics browser behavior for web scraping
/// - `api()` - Optimized for REST API calls
/// - `minimal()` - No default headers
#[derive(Clone)]
pub struct SongbirdHttpClient {
    crypto: Arc<dyn CryptoCapability>,
    tls_config: TlsConfig,
    http_config: HttpClientConfig,
    /// Profiler for adaptive server learning (future feature)
    #[allow(dead_code)]
    profiler: Option<Arc<ServerProfiler>>,
}

impl std::fmt::Debug for SongbirdHttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SongbirdHttpClient")
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
    /// This client uses the CryptoCapability trait for agnostic crypto operations.
    /// The underlying provider can be BearDog or any other implementation.
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self::with_tls_config(socket_path, TlsConfig::default(), None)
    }

    /// Create from environment variable with standard HTTP config
    ///
    /// Automatically detects Neural API mode or Direct mode based on environment:
    /// - BEARDOG_MODE=neural (default): Routes through Neural API for capability.call
    /// - BEARDOG_MODE=direct (testing): Direct connection to BearDog
    ///
    /// Uses NEURAL_API_SOCKET or BEARDOG_SOCKET accordingly.
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
    /// Use this when you want to provide your own CryptoCapability implementation.
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
    pub fn http_config(&self) -> &HttpClientConfig {
        &self.http_config
    }

    /// Get mutable reference to HTTP configuration
    pub fn http_config_mut(&mut self) -> &mut HttpClientConfig {
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
        info!("🌐 HTTP {} {}", method, url);

        // Parse URL into URI
        let parsed_uri: Uri = url.parse().map_err(|e| Error::InvalidUrl(format!("{}", e)))?;

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
        let addr = format!("{}:{}", host, port);
        let tcp_stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| Error::Connection(format!("Failed to connect to {}: {}", addr, e)))?;

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
    /// - `RedirectMode::None`: Returns redirect response as-is
    /// - `RedirectMode::Follow`: Follows all redirects (max configured)
    /// - `RedirectMode::SameOrigin`: Only follows redirects to same origin
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
        use crate::http_config::RedirectMode;

        let mut current_url = url.to_string();
        let mut redirects_followed = 0;
        let max_redirects = self.http_config.max_redirects;
        let original_host = Uri::try_from(url).ok().and_then(|u| u.host().map(|h| h.to_string()));

        loop {
            // Make the request
            let response =
                self.request(method, &current_url, headers.clone(), body.clone()).await?;

            // Check if this is a redirect status
            let is_redirect = matches!(response.status, 301 | 302 | 303 | 307 | 308);

            if !is_redirect {
                return Ok(response);
            }

            // Check redirect mode
            match self.http_config.redirect_mode {
                RedirectMode::None => {
                    info!(
                        "↩️  Redirect received ({}), returning as-is (redirect_mode=None)",
                        response.status
                    );
                    return Ok(response);
                }
                RedirectMode::Follow => {
                    // Continue to follow redirect
                }
                RedirectMode::SameOrigin => {
                    // Check if redirect is to same origin
                    if let Some(location) = response.headers.get("location") {
                        let redirect_host = self.extract_host_from_location(location, &current_url);
                        if redirect_host != original_host {
                            info!(
                                "↩️  Cross-origin redirect to {:?}, returning as-is (redirect_mode=SameOrigin)",
                                redirect_host
                            );
                            return Ok(response);
                        }
                    }
                }
            }

            // Check redirect limit
            if redirects_followed >= max_redirects {
                warn!("⚠️  Maximum redirects ({}) reached", max_redirects);
                return Ok(response);
            }

            // Extract Location header
            let location = response.headers.get("location").ok_or_else(|| {
                Error::HttpProtocol("Redirect without Location header".to_string())
            })?;

            // Resolve relative URLs
            let new_url = self.resolve_redirect_url(&current_url, location)?;

            info!(
                "↪️  Following redirect {}/{}: {} -> {}",
                redirects_followed + 1,
                max_redirects,
                response.status,
                new_url
            );

            current_url = new_url;
            redirects_followed += 1;
        }
    }

    /// Extract host from a Location header value
    fn extract_host_from_location(&self, location: &str, base_url: &str) -> Option<String> {
        // Try to parse location as absolute URL
        if let Ok(uri) = Uri::try_from(location) {
            if let Some(host) = uri.host() {
                return Some(host.to_string());
            }
        }

        // If relative URL, use base URL's host
        Uri::try_from(base_url).ok().and_then(|u| u.host().map(|h| h.to_string()))
    }

    /// Resolve a redirect URL (handles relative and absolute URLs)
    fn resolve_redirect_url(&self, base_url: &str, location: &str) -> Result<String> {
        // If location is absolute, use it directly
        if location.starts_with("http://") || location.starts_with("https://") {
            return Ok(location.to_string());
        }

        // Parse base URL to get scheme and host
        let base: Uri =
            base_url.parse().map_err(|e| Error::InvalidUrl(format!("Invalid base URL: {}", e)))?;

        let scheme = base.scheme_str().unwrap_or("https");
        let host =
            base.host().ok_or_else(|| Error::InvalidUrl("Missing host in base URL".to_string()))?;
        let port = base.port_u16();

        // Build new URL
        let new_url = if location.starts_with('/') {
            // Absolute path relative to host
            match port {
                Some(p) => format!("{}://{}:{}{}", scheme, host, p, location),
                None => format!("{}://{}{}", scheme, host, location),
            }
        } else {
            // Relative path - append to base path
            let base_path = base.path();
            let parent = base_path.rsplit_once('/').map(|(p, _)| p).unwrap_or("");
            match port {
                Some(p) => format!("{}://{}:{}{}/{}", scheme, host, p, parent, location),
                None => format!("{}://{}{}/{}", scheme, host, parent, location),
            }
        };

        Ok(new_url)
    }

    /// Make HTTPS request with TLS
    async fn https_request(
        &self,
        host: &str,
        port: u16,
        uri: &Uri,
        method: &str,
        headers: HashMap<String, String>,
        body: Option<serde_json::Value>,
    ) -> Result<HttpResponse> {
        debug!("🔒 Performing TLS handshake with {}", host);

        // Attempt TLS handshake with progressive fallback
        // CRITICAL FIX: Each retry creates a FRESH TCP connection to avoid reading stale data!
        let addr = format!("{}:{}", host, port);
        let (mut tcp_stream, session_keys) =
            self.attempt_handshake_with_fallback(&addr, host).await?;

        info!("✅ TLS handshake complete with {}", host);
        info!("════════════════════════════════════════════════════════════");
        info!("  APPLICATION DATA PHASE - HTTP Request/Response Exchange");
        info!("════════════════════════════════════════════════════════════");

        // Create TLS record layer
        let mut record_layer = TlsRecordLayer::new(self.crypto.clone(), session_keys);
        debug!("✅ TLS record layer initialized (sequence numbers at 0)");

        // Build HTTP request
        let http_request = self.build_http_request(uri, method, &headers, body.as_ref())?;
        info!("🔼 SENDING HTTP REQUEST to server:");
        info!("   Method: {}", method);
        info!("   URI: {}", uri);
        info!("   Size: {} bytes", http_request.len());
        debug!("HTTP request content:\n{}", String::from_utf8_lossy(&http_request));

        // Validate TCP stream before sending
        if let Ok(peer) = tcp_stream.peer_addr() {
            debug!("TCP stream peer address: {}", peer);
        }

        // Send HTTP request over TLS
        info!("════════════════════════════════════════════════════════════");
        info!("📤 SENDING HTTP REQUEST (DIAGNOSTIC INFO)");
        info!("════════════════════════════════════════════════════════════");
        info!("Cipher suite: 0x{:04x}", record_layer.keys().cipher_suite);
        info!("HTTP request size: {} bytes", http_request.len());
        info!("Write sequence number: {}", record_layer.write_sequence_number());
        info!("Using: APPLICATION traffic keys (NOT handshake keys)");
        debug!("Client write key length: {} bytes", record_layer.keys().client_write_key.len());
        debug!("Client write IV length: {} bytes", record_layer.keys().client_write_iv.len());
        debug!("Client write key (hex): {}", hex::encode(&record_layer.keys().client_write_key));
        debug!("Client write IV (hex): {}", hex::encode(&record_layer.keys().client_write_iv));
        info!("════════════════════════════════════════════════════════════");

        record_layer.write_application_data(&mut tcp_stream, &http_request).await.map_err(|e| {
            error!("❌ Failed to send HTTP request: {}", e);
            e
        })?;
        info!("✅ HTTP request SENT to server (encrypted with application traffic keys)");
        info!("   Now waiting for server's HTTP response...");
        info!("────────────────────────────────────────────────────────────");

        // Read HTTP response over TLS (may span multiple APPLICATION_DATA records!)
        // RFC 8446 Section 5.1: Records can be max 2^14 bytes (16384) of plaintext
        // Large HTTP responses will be fragmented across multiple TLS records
        info!("🔽 READING HTTP RESPONSE from server:");
        info!("   Response may span multiple TLS APPLICATION_DATA records...");

        let mut response_data = Vec::new();
        let mut records_read = 0;
        let mut headers_complete = false;
        let max_response_size = 10_000_000; // 10 MB safety limit

        // Read TLS records until we have a complete HTTP response
        loop {
            records_read += 1;
            debug!("   Reading TLS APPLICATION_DATA record #{}...", records_read);

            let chunk = record_layer.read_application_data(&mut tcp_stream).await.map_err(|e| {
                error!("❌ Failed to read HTTP response (record #{}): {}", records_read, e);
                if records_read == 1 {
                    error!("   This error occurred AFTER successfully sending request");
                    error!("   Request size was: {} bytes", http_request.len());
                }
                e
            })?;

            // Empty record = connection closed (close_notify or EOF)
            if chunk.is_empty() {
                if records_read == 1 {
                    warn!("⚠️  Connection closed before receiving any data (close_notify or EOF)");
                    warn!("   Server may have rejected request or encountered error");
                } else {
                    info!(
                        "✅ Server closed connection after sending {} record(s)",
                        records_read - 1
                    );
                    info!("   Response complete ({} bytes total)", response_data.len());
                }
                break;
            }

            debug!("   ✅ Record #{}: {} bytes", records_read, chunk.len());
            response_data.extend_from_slice(&chunk);

            // Check if we have complete HTTP headers (\r\n\r\n)
            if !headers_complete {
                if let Some(headers_end) = response_data.windows(4).position(|w| w == b"\r\n\r\n") {
                    headers_complete = true;
                    debug!("   📋 HTTP headers complete ({} bytes)", headers_end);

                    // Parse headers to determine response type
                    let headers_str = String::from_utf8_lossy(&response_data[..headers_end]);
                    let headers_lower = headers_str.to_lowercase();

                    // Check for Transfer-Encoding: chunked
                    let is_chunked = headers_lower.contains("transfer-encoding: chunked")
                        || headers_lower.contains("transfer-encoding:chunked");

                    // Check for Connection: close
                    let connection_close = headers_lower.contains("connection: close")
                        || headers_lower.contains("connection:close");

                    if is_chunked {
                        info!("   📦 Transfer-Encoding: chunked detected");
                        // For chunked responses, look for terminator: 0\r\n\r\n
                        // This indicates end of chunked body
                    } else if let Some(content_length_line) = headers_str
                        .lines()
                        .find(|line| line.to_lowercase().starts_with("content-length:"))
                    {
                        if let Some(content_length) = content_length_line
                            .split(':')
                            .nth(1)
                            .and_then(|val| val.trim().parse::<usize>().ok())
                        {
                            let body_start = headers_end + 4;
                            let total_expected = body_start + content_length;
                            debug!(
                                "   📊 Content-Length: {} bytes, expecting {} total",
                                content_length, total_expected
                            );

                            // If we already have the complete response, we're done
                            if response_data.len() >= total_expected {
                                debug!(
                                    "   ✅ Complete response received in {} record(s)",
                                    records_read
                                );
                                break;
                            }

                            // Continue reading until we have the full body
                            continue;
                        }
                    } else if connection_close {
                        debug!("   🔌 Connection: close - will read until server closes");
                    } else {
                        // No Content-Length, no chunked, no connection close
                        debug!("   ⚠️  No Content-Length or chunked encoding, reading until close");
                    }
                }
            }

            // Check for chunked encoding termination: 0\r\n\r\n
            // This is the final chunk marker indicating end of chunked body
            if headers_complete {
                // Look for the chunked encoding terminator anywhere in the body
                // The terminator is: "0\r\n\r\n" (zero-length chunk followed by empty trailer)
                // Some servers also send "0\r\n\r\n" variations with trailers
                if let Some(headers_end) = response_data.windows(4).position(|w| w == b"\r\n\r\n") {
                    let body = &response_data[headers_end + 4..];

                    // Check for chunked terminator patterns
                    let has_terminator = body.windows(5).any(|w| w == b"0\r\n\r\n")
                        || body.ends_with(b"0\r\n\r\n")
                        || body.ends_with(b"\r\n0\r\n\r\n");

                    if has_terminator {
                        info!("   ✅ Chunked encoding terminator (0\\r\\n\\r\\n) found");
                        break;
                    }

                    // Also check Content-Length completion
                    let headers_str = String::from_utf8_lossy(&response_data[..headers_end]);
                    if let Some(content_length) = headers_str
                        .lines()
                        .find(|line| line.to_lowercase().starts_with("content-length:"))
                        .and_then(|line| line.split(':').nth(1))
                        .and_then(|val| val.trim().parse::<usize>().ok())
                    {
                        let body_start = headers_end + 4;
                        let total_expected = body_start + content_length;

                        if response_data.len() >= total_expected {
                            debug!(
                                "   ✅ Complete response received ({} bytes) in {} record(s)",
                                response_data.len(),
                                records_read
                            );
                            break;
                        }
                        debug!(
                            "   📥 Still reading body: {}/{} bytes",
                            response_data.len() - body_start,
                            content_length
                        );
                    }
                }
            }

            // Safety: Prevent infinite loops or memory exhaustion
            if response_data.len() > max_response_size {
                warn!(
                    "⚠️  HTTP response exceeds {} MB limit, stopping read",
                    max_response_size / 1_000_000
                );
                break;
            }

            // Safety: Prevent reading too many records
            if records_read > 100 {
                warn!("⚠️  Read {} TLS records, stopping (possible issue)", records_read);
                break;
            }
        }

        // Validate we received data
        if response_data.is_empty() {
            error!("❌ No HTTP response data received (server closed connection without sending response)");
            return Err(Error::HttpProtocol("No response data received from server".to_string()));
        }

        info!("✅ HTTP response RECEIVED from server:");
        info!("   Total size: {} bytes across {} TLS record(s)", response_data.len(), records_read);
        debug!(
            "HTTP response content:\n{}",
            String::from_utf8_lossy(&response_data[..std::cmp::min(500, response_data.len())])
        );
        info!("════════════════════════════════════════════════════════════");

        // Parse HTTP response
        debug!("Parsing HTTP response...");
        self.parse_http_response(&response_data)
    }

    /// Attempt TLS handshake with progressive fallback on failure
    ///
    /// CRITICAL FIX (Jan 26, 2026): Each retry attempt creates a FRESH TCP connection!
    /// Bug was: reusing the same TCP stream caused reading stale buffered data on retries.
    async fn attempt_handshake_with_fallback(
        &self,
        addr: &str,
        host: &str,
    ) -> Result<(TcpStream, crate::tls::session::SessionKeys)> {
        use crate::tls::config::{ExtensionStrategy, FallbackStrategy};

        let max_attempts = self.tls_config.max_retries as usize;
        let mut last_error = None;

        // Build list of strategies to try based on fallback strategy
        let strategies_to_try = match self.tls_config.fallback_strategy {
            FallbackStrategy::None => {
                // Single attempt with configured strategy
                vec![self.tls_config.extension_strategy.clone()]
            }
            FallbackStrategy::Progressive => {
                // Try Modern → Standard → Minimal
                info!("🔄 Progressive fallback enabled: Modern → Standard → Minimal");
                vec![
                    ExtensionStrategy::Modern,
                    ExtensionStrategy::Standard,
                    ExtensionStrategy::Minimal,
                ]
            }
            FallbackStrategy::Reverse => {
                // Try Minimal → Standard → Modern
                info!("🔄 Reverse fallback enabled: Minimal → Standard → Modern");
                vec![
                    ExtensionStrategy::Minimal,
                    ExtensionStrategy::Standard,
                    ExtensionStrategy::Modern,
                ]
            }
            FallbackStrategy::Exhaustive => {
                // Try all strategies
                info!("🔄 Exhaustive fallback enabled: Trying all strategies");
                vec![
                    ExtensionStrategy::Modern,
                    ExtensionStrategy::Standard,
                    ExtensionStrategy::Minimal,
                    ExtensionStrategy::MaxCompatibility,
                ]
            }
        };

        // Try each strategy with FRESH TCP connection
        for (attempt, strategy) in strategies_to_try.iter().enumerate().take(max_attempts) {
            let attempt_num = attempt + 1;

            if attempt > 0 {
                info!(
                    "🔄 Retry attempt {}/{} with {:?} strategy (FRESH TCP connection)",
                    attempt_num,
                    strategies_to_try.len(),
                    strategy
                );
            }

            // CRITICAL: Create FRESH TCP connection for each attempt!
            // This prevents reading stale buffered data from previous attempts.
            let mut tcp_stream = match TcpStream::connect(addr).await {
                Ok(stream) => {
                    // Log connection details for debugging
                    let local = stream
                        .local_addr()
                        .map(|a| a.to_string())
                        .unwrap_or_else(|_| "unknown".into());
                    let peer = stream
                        .peer_addr()
                        .map(|a| a.to_string())
                        .unwrap_or_else(|_| "unknown".into());
                    info!("✅ TCP connection established:");
                    info!("   Local: {}", local);
                    info!("   Remote: {} (expected: {})", peer, addr);

                    // Verify we connected to the right port
                    if let Ok(peer_addr) = stream.peer_addr() {
                        if peer_addr.port() != 443 && addr.contains(":443") {
                            warn!("⚠️  Connected to port {} but expected 443!", peer_addr.port());
                        }
                    }

                    stream
                }
                Err(e) => {
                    warn!("⚠️  Failed to connect to {}: {}", addr, e);
                    last_error =
                        Some(Error::Connection(format!("Failed to connect to {}: {}", addr, e)));
                    continue; // Try next strategy with fresh connection
                }
            };

            // Create config with current strategy
            let mut attempt_config = self.tls_config.clone();
            attempt_config.extension_strategy = strategy.clone();

            // Attempt handshake on FRESH connection
            let handshake_start = std::time::Instant::now();
            let mut handshake = TlsHandshake::with_config(
                self.crypto.clone(),
                attempt_config,
                self.profiler.clone(),
            );

            match handshake.handshake(&mut tcp_stream, host).await {
                Ok(keys) => {
                    let handshake_duration = handshake_start.elapsed();
                    info!(
                        "✅ TLS handshake succeeded with {:?} strategy in {:?}",
                        strategy, handshake_duration
                    );

                    if attempt > 0 {
                        info!("🎯 Fallback successful after {} attempt(s)", attempt_num);
                    }

                    // Record success with profiler
                    if let Some(profiler) = &self.profiler {
                        profiler.record_success(
                            host,
                            vec![],
                            keys.cipher_suite,
                            handshake_duration,
                        );
                        debug!("🧠 Profiler updated: success for {} with {:?}", host, strategy);
                    }

                    // Return BOTH the successful stream AND the keys
                    return Ok((tcp_stream, keys));
                }
                Err(e) => {
                    let handshake_duration = handshake_start.elapsed();
                    warn!(
                        "⚠️  TLS handshake failed with {:?} strategy after {:?}: {}",
                        strategy, handshake_duration, e
                    );

                    // Record failure with profiler
                    if let Some(profiler) = &self.profiler {
                        profiler.record_failure(host, vec![], None, &e.to_string());
                        debug!("🧠 Profiler updated: failure for {} with {:?}", host, strategy);
                    }

                    last_error = Some(e);
                    // tcp_stream dropped here, connection closed cleanly

                    // If this was the last attempt, break
                    if attempt_num >= strategies_to_try.len() || attempt_num >= max_attempts {
                        break;
                    }

                    // Otherwise, continue to next strategy with fresh connection
                    debug!("Closing failed connection, will try next strategy with fresh TCP...");
                }
            }
        }

        // All attempts failed
        let final_error = last_error.unwrap_or_else(|| {
            Error::Connection("TLS handshake failed with all strategies".to_string())
        });

        error!(
            "❌ TLS handshake failed after {} attempt(s) with all strategies",
            strategies_to_try.len()
        );
        Err(final_error)
    }

    /// Make HTTP request without TLS
    async fn http_request(
        &self,
        tcp_stream: TcpStream,
        uri: &Uri,
        method: &str,
        headers: HashMap<String, String>,
        body: Option<serde_json::Value>,
    ) -> Result<HttpResponse> {
        debug!("📡 Making HTTP request (no TLS)");

        let io = TokioIo::new(tcp_stream);

        // Create HTTP client
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

        // Spawn connection task
        tokio::spawn(async move {
            if let Err(err) = conn.await {
                tracing::error!("Connection error: {:?}", err);
            }
        });

        // Build request
        let mut req_builder = Request::builder().method(method).uri(uri);

        // Add headers
        for (key, value) in headers {
            req_builder = req_builder.header(&key, &value);
        }

        // Build body
        let body_bytes = if let Some(b) = body {
            Bytes::from(serde_json::to_vec(&b)?)
        } else {
            Bytes::new()
        };

        let request = req_builder.body(Full::new(body_bytes))?;

        // Send request
        let response = sender.send_request(request).await?;

        // Read response
        let status = response.status().as_u16();
        let response_headers: HashMap<String, String> = response
            .headers()
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|s| (k.as_str().to_string(), s.to_string())))
            .collect();

        let body_bytes = response.into_body().collect().await?.to_bytes();
        let body: serde_json::Value = if body_bytes.is_empty() {
            serde_json::json!("")
        } else {
            serde_json::from_slice(&body_bytes).unwrap_or_else(|_| {
                serde_json::json!(String::from_utf8_lossy(&body_bytes).to_string())
            })
        };

        Ok(HttpResponse {
            status,
            headers: response_headers,
            body,
        })
    }

    /// Build HTTP request bytes with adaptive headers
    ///
    /// Uses HttpClientConfig to apply:
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
        let mut request = Vec::new();

        // Request line
        let path = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
        request.extend_from_slice(format!("{} {} HTTP/1.1\r\n", method, path).as_bytes());

        // Get host for header routing
        let host = uri.host().unwrap_or("unknown");

        // Host header (always first)
        request.extend_from_slice(format!("Host: {}\r\n", host).as_bytes());

        // Get merged headers from config (defaults + rules + caller)
        let headers = self.http_config.headers_for_domain(host, caller_headers);

        // Log applied headers for debugging
        if self.http_config.is_bot_protected(host) {
            trace!("🛡️  {} is bot-protected - applying adaptive headers", host);
        }
        trace!("📋 Applying {} headers for {}", headers.len(), host);

        // Headers (sorted for deterministic output in tests)
        let mut header_pairs: Vec<_> = headers.iter().collect();
        header_pairs.sort_by(|a, b| a.0.cmp(b.0));

        for (key, value) in header_pairs {
            // Skip Host (already added)
            if key.eq_ignore_ascii_case("host") {
                continue;
            }
            request.extend_from_slice(format!("{}: {}\r\n", key, value).as_bytes());
        }

        // Body
        if let Some(b) = body {
            let body_bytes = serde_json::to_vec(b)?;
            request
                .extend_from_slice(format!("Content-Length: {}\r\n", body_bytes.len()).as_bytes());
            request.extend_from_slice(b"\r\n");
            request.extend_from_slice(&body_bytes);
        } else {
            request.extend_from_slice(b"\r\n");
        }

        Ok(request)
    }

    /// Parse HTTP response bytes
    fn parse_http_response(&self, data: &[u8]) -> Result<HttpResponse> {
        // Debug: Log first bytes to understand any corruption
        debug!("📝 parse_http_response: {} bytes total", data.len());
        debug!("📝 First 50 bytes (hex): {:?}", &data[..std::cmp::min(50, data.len())]);
        debug!(
            "📝 First 50 bytes (str): {:?}",
            String::from_utf8_lossy(&data[..std::cmp::min(50, data.len())])
        );

        let response = String::from_utf8_lossy(data);
        let mut lines = response.lines();

        // Status line
        let status_line =
            lines.next().ok_or_else(|| Error::InvalidResponse("Empty response".to_string()))?;

        // Debug: Log the status line to understand parsing issues
        debug!("📝 Parsing status line: {:?}", status_line);
        debug!("📝 Status line bytes: {:?}", status_line.as_bytes());

        let status = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| {
                debug!("📝 Extracted status code string: {:?}", s);
                s.parse::<u16>().ok()
            })
            .ok_or_else(|| {
                Error::InvalidResponse(format!("Invalid status line: {:?}", status_line))
            })?;

        // Headers
        let mut headers = HashMap::new();
        let mut body_start = 0;

        for (idx, line) in lines.enumerate() {
            if line.is_empty() {
                body_start = idx + 2; // +2 for status line and empty line
                break;
            }

            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }

        // Body
        let body_lines: Vec<&str> = response.lines().skip(body_start).collect();
        let body_str = body_lines.join("\n");

        let body: serde_json::Value = if body_str.is_empty() {
            serde_json::json!("")
        } else {
            serde_json::from_str(&body_str).unwrap_or_else(|_| serde_json::json!(body_str))
        };

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
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
mod tests {
    use super::*;

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
        let client = SongbirdHttpClient::new("/tmp/beardog.sock");
        let response_data =
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"result\":\"ok\"}";

        let response = client.parse_http_response(response_data).unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.headers.get("content-type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_resolve_redirect_url_absolute() {
        let client = SongbirdHttpClient::new("/tmp/beardog.sock");

        // Absolute URL should be returned as-is
        let resolved = client
            .resolve_redirect_url("https://example.com/path", "https://other.com/new-path")
            .unwrap();
        assert_eq!(resolved, "https://other.com/new-path");
    }

    #[test]
    fn test_resolve_redirect_url_absolute_path() {
        let client = SongbirdHttpClient::new("/tmp/beardog.sock");

        // Absolute path (starts with /) should use base's scheme and host
        let resolved =
            client.resolve_redirect_url("https://example.com/old-path", "/new-path").unwrap();
        assert_eq!(resolved, "https://example.com/new-path");
    }

    #[test]
    fn test_resolve_redirect_url_relative_path() {
        let client = SongbirdHttpClient::new("/tmp/beardog.sock");

        // Relative path should be resolved relative to base
        let resolved =
            client.resolve_redirect_url("https://example.com/path/to/page", "other-page").unwrap();
        assert_eq!(resolved, "https://example.com/path/to/other-page");
    }

    #[test]
    fn test_resolve_redirect_url_with_port() {
        let client = SongbirdHttpClient::new("/tmp/beardog.sock");

        // Preserve port in redirect
        let resolved =
            client.resolve_redirect_url("https://example.com:8443/path", "/new-path").unwrap();
        assert_eq!(resolved, "https://example.com:8443/new-path");
    }

    #[test]
    fn test_extract_host_from_location() {
        let client = SongbirdHttpClient::new("/tmp/beardog.sock");

        // Absolute URL
        let host =
            client.extract_host_from_location("https://other.com/path", "https://example.com");
        assert_eq!(host, Some("other.com".to_string()));

        // Relative URL should use base host
        let host = client.extract_host_from_location("/new-path", "https://example.com/old-path");
        assert_eq!(host, Some("example.com".to_string()));
    }
}
