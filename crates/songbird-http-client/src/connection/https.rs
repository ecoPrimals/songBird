// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! HTTPS connection handling with TLS 1.3
//!
//! Handles HTTPS requests over TLS 1.3 connections with progressive fallback.

use crate::crypto::SecurityCryptoProvider;
use crate::error::{Error, Result};
use crate::tls::config::{ExtensionStrategy, FallbackStrategy, TlsConfig};
use crate::tls::handshake::TlsHandshake;
use crate::tls::profiler::ServerProfiler;
use crate::tls::record::TlsRecordLayer;
use crate::tls::session::SessionKeys;
use crate::types::HttpResponse;
use hyper::Uri;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tracing::{debug, error, info, warn};

/// HTTPS connection handler with TLS 1.3
///
/// Manages HTTPS requests over TLS 1.3 connections, including:
/// - TLS handshake with progressive fallback
/// - Application data encryption/decryption
/// - HTTP response parsing from TLS records
pub struct HttpsConnection {
    crypto: Arc<SecurityCryptoProvider>,
    tls_config: TlsConfig,
    profiler: Option<Arc<ServerProfiler>>,
}

impl HttpsConnection {
    /// Create new HTTPS connection handler
    ///
    /// # Arguments
    ///
    /// * `crypto` - Crypto capability provider (`security provider` or mock)
    /// * `tls_config` - TLS configuration (versions, fallback strategy, etc.)
    /// * `profiler` - Optional server profiler for performance tracking
    pub fn new(
        crypto: Arc<SecurityCryptoProvider>,
        tls_config: TlsConfig,
        profiler: Option<Arc<ServerProfiler>>,
    ) -> Self {
        Self {
            crypto,
            tls_config,
            profiler,
        }
    }

    /// Execute HTTPS request over TLS 1.3
    ///
    /// # Arguments
    ///
    /// * `host` - Server hostname (for SNI)
    /// * `port` - Server port (typically 443)
    /// * `uri` - Request URI
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `headers` - Request headers
    /// * `body` - Optional JSON body
    ///
    /// # Returns
    ///
    /// Parsed HTTP response with status, headers, and body
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - TLS handshake fails
    /// - HTTP request building fails
    /// - Sending request fails
    /// - Reading response fails
    /// - Response parsing fails
    #[allow(
        clippy::too_many_arguments,
        reason = "TLS execute bundles host, port, URI, method, headers, body, and parse/build callbacks"
    )]
    pub async fn execute(
        &self,
        host: &str,
        port: u16,
        uri: &Uri,
        method: &str,
        headers: &HashMap<String, String>,
        body: Option<&serde_json::Value>,
        build_http_request: impl Fn(
            &Uri,
            &str,
            &HashMap<String, String>,
            Option<&serde_json::Value>,
        ) -> Result<Vec<u8>>,
        parse_http_response: impl Fn(&[u8]) -> Result<HttpResponse>,
    ) -> Result<HttpResponse> {
        debug!("🔒 Performing TLS handshake with {}", host);

        // Attempt TLS handshake with progressive fallback
        // CRITICAL FIX: Each retry creates a FRESH TCP connection to avoid reading stale data!
        let addr = format!("{host}:{port}");
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
        let http_request = build_http_request(uri, method, headers, body)?;
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

        let response_data =
            self.read_tls_response(&mut record_layer, &mut tcp_stream, &http_request).await?;

        // Validate we received data
        if response_data.is_empty() {
            error!(
                "❌ No HTTP response data received (server closed connection without sending response)"
            );
            return Err(Error::HttpProtocol(String::from("No response data received from server")));
        }

        info!("✅ HTTP response RECEIVED from server:");
        info!("   Total size: {} bytes", response_data.len());
        debug!(
            "HTTP response content:\n{}",
            String::from_utf8_lossy(&response_data[..std::cmp::min(500, response_data.len())])
        );
        info!("════════════════════════════════════════════════════════════");

        // Parse HTTP response
        debug!("Parsing HTTP response...");
        parse_http_response(&response_data)
    }

    /// Check if TLS response is complete (chunked terminator or Content-Length met)
    fn is_tls_response_complete(response_data: &[u8], records_read: usize) -> bool {
        let Some(headers_end) = response_data.windows(4).position(|w| w == b"\r\n\r\n") else {
            return false;
        };
        let body = &response_data[headers_end + 4..];

        // Check for chunked terminator patterns
        let has_terminator = body.windows(5).any(|w| w == b"0\r\n\r\n")
            || body.ends_with(b"0\r\n\r\n")
            || body.ends_with(b"\r\n0\r\n\r\n");

        if has_terminator {
            info!("   ✅ Chunked encoding terminator (0\\r\\n\\r\\n) found");
            return true;
        }

        // Check Content-Length completion
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
                return true;
            }
            debug!(
                "   📥 Still reading body: {}/{} bytes",
                response_data.len() - body_start,
                content_length
            );
        }
        false
    }

    /// Read complete HTTP response from TLS record layer
    ///
    /// Handles:
    /// - Multi-record responses (TLS max record size is 16KB)
    /// - Content-Length detection
    /// - Transfer-Encoding: chunked
    /// - Connection: close
    async fn read_tls_response(
        &self,
        record_layer: &mut TlsRecordLayer,
        tcp_stream: &mut TcpStream,
        http_request: &[u8],
    ) -> Result<Vec<u8>> {
        let mut response_data = Vec::new();
        let mut records_read = 0;
        let mut headers_complete = false;
        let max_response_size = 10_000_000; // 10 MB safety limit

        // Read TLS records until we have a complete HTTP response
        loop {
            records_read += 1;
            debug!("   Reading TLS APPLICATION_DATA record #{}...", records_read);

            let chunk = record_layer.read_application_data(tcp_stream).await.map_err(|e| {
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
            if !headers_complete
                && let Some(headers_end) = response_data.windows(4).position(|w| w == b"\r\n\r\n")
            {
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

            // Check for chunked encoding termination or Content-Length completion
            if headers_complete && Self::is_tls_response_complete(&response_data, records_read) {
                break;
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

        Ok(response_data)
    }

    /// Build strategy list based on fallback configuration
    fn handshake_strategies_for_fallback(config: &TlsConfig) -> Vec<ExtensionStrategy> {
        match config.fallback_strategy {
            FallbackStrategy::None => vec![config.extension_strategy.clone()],
            FallbackStrategy::Progressive => {
                info!("🔄 Progressive fallback enabled: Modern → Standard → Minimal");
                vec![
                    ExtensionStrategy::Modern,
                    ExtensionStrategy::Standard,
                    ExtensionStrategy::Minimal,
                ]
            }
            FallbackStrategy::Reverse => {
                info!("🔄 Reverse fallback enabled: Minimal → Standard → Modern");
                vec![
                    ExtensionStrategy::Minimal,
                    ExtensionStrategy::Standard,
                    ExtensionStrategy::Modern,
                ]
            }
            FallbackStrategy::Exhaustive => {
                info!("🔄 Exhaustive fallback enabled: Trying all strategies");
                vec![
                    ExtensionStrategy::Modern,
                    ExtensionStrategy::Standard,
                    ExtensionStrategy::Minimal,
                    ExtensionStrategy::MaxCompatibility,
                ]
            }
        }
    }

    /// Attempt TLS handshake with progressive fallback on failure
    ///
    /// CRITICAL FIX (Jan 26, 2026): Each retry attempt creates a FRESH TCP connection!
    /// Bug was: reusing the same TCP stream caused reading stale buffered data on retries.
    ///
    /// # Arguments
    ///
    /// * `addr` - Server address (host:port)
    /// * `host` - Server hostname (for SNI)
    ///
    /// # Returns
    ///
    /// Tuple of (`TcpStream`, `SessionKeys`) on success
    ///
    /// # Errors
    ///
    /// Returns error if all handshake attempts fail
    async fn attempt_handshake_with_fallback(
        &self,
        addr: &str,
        host: &str,
    ) -> Result<(TcpStream, SessionKeys)> {
        let max_attempts = self.tls_config.max_retries as usize;
        let mut last_error = None;

        let strategies_to_try = Self::handshake_strategies_for_fallback(&self.tls_config);

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
                    let local =
                        stream.local_addr().map_or_else(|_| "unknown".into(), |a| a.to_string());
                    let peer =
                        stream.peer_addr().map_or_else(|_| "unknown".into(), |a| a.to_string());
                    info!("✅ TCP connection established:");
                    info!("   Local: {}", local);
                    info!("   Remote: {} (expected: {})", peer, addr);

                    // Verify we connected to the right port
                    let https_port = songbird_types::defaults::ports::HTTPS_STANDARD_PORT;
                    if let Ok(peer_addr) = stream.peer_addr()
                        && peer_addr.port() != https_port
                        && addr.contains(&format!(":{https_port}"))
                    {
                        warn!(
                            "⚠️  Connected to port {} but expected {https_port}!",
                            peer_addr.port()
                        );
                    }

                    stream
                }
                Err(e) => {
                    warn!("⚠️  Failed to connect to {}: {}", addr, e);
                    last_error =
                        Some(Error::Connection(format!("Failed to connect to {addr}: {e}")));
                    continue;
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

                    // If the peer responded with HTTP, no TLS strategy will work — abort immediately
                    if e.is_http_not_tls() {
                        warn!("🌐 Peer {} is serving plain HTTP — aborting TLS retries", host);
                        return Err(e);
                    }

                    last_error = Some(e);
                    // tcp_stream dropped here, connection closed cleanly
                }
            }
        }

        // All attempts failed
        let error = last_error.unwrap_or_else(|| {
            Error::TlsHandshake(String::from("All handshake strategies exhausted"))
        });
        error!("❌ TLS handshake failed: All {} attempt(s) exhausted", strategies_to_try.len());
        error!("   Last error: {}", error);
        Err(error)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::HttpsConnection;
    use crate::crypto::SecurityCryptoProvider;
    use crate::tls::config::{ExtensionStrategy, FallbackStrategy, TlsConfig};
    use std::sync::Arc;

    #[test]
    fn https_connection_new_stores_config() {
        let path = tempfile::env::temp_dir()
            .join("songbird-test-security.sock")
            .to_string_lossy()
            .into_owned();
        let crypto: Arc<SecurityCryptoProvider> = Arc::new(SecurityCryptoProvider::new(path));
        let cfg = TlsConfig {
            max_retries: 4,
            ..Default::default()
        };
        let conn = HttpsConnection::new(crypto, cfg, None);
        assert_eq!(conn.tls_config.max_retries, 4);
    }

    #[test]
    fn handshake_strategies_none_uses_configured_extension_strategy() {
        let cfg = TlsConfig {
            fallback_strategy: FallbackStrategy::None,
            extension_strategy: ExtensionStrategy::Minimal,
            ..Default::default()
        };
        let strategies = HttpsConnection::handshake_strategies_for_fallback(&cfg);
        assert_eq!(strategies, vec![ExtensionStrategy::Minimal]);
    }

    #[test]
    fn handshake_strategies_progressive_order() {
        let cfg = TlsConfig {
            fallback_strategy: FallbackStrategy::Progressive,
            ..Default::default()
        };
        let strategies = HttpsConnection::handshake_strategies_for_fallback(&cfg);
        assert_eq!(
            strategies,
            vec![
                ExtensionStrategy::Modern,
                ExtensionStrategy::Standard,
                ExtensionStrategy::Minimal,
            ]
        );
    }

    #[test]
    fn handshake_strategies_reverse_order() {
        let cfg = TlsConfig {
            fallback_strategy: FallbackStrategy::Reverse,
            ..Default::default()
        };
        let strategies = HttpsConnection::handshake_strategies_for_fallback(&cfg);
        assert_eq!(
            strategies,
            vec![
                ExtensionStrategy::Minimal,
                ExtensionStrategy::Standard,
                ExtensionStrategy::Modern,
            ]
        );
    }

    #[test]
    fn handshake_strategies_exhaustive_includes_max_compat() {
        let cfg = TlsConfig {
            fallback_strategy: FallbackStrategy::Exhaustive,
            ..Default::default()
        };
        let strategies = HttpsConnection::handshake_strategies_for_fallback(&cfg);
        assert!(strategies.contains(&ExtensionStrategy::MaxCompatibility));
    }

    #[test]
    fn is_tls_response_complete_chunked_terminator() {
        let response = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n";
        assert!(HttpsConnection::is_tls_response_complete(response, 1));
    }

    #[test]
    fn is_tls_response_complete_content_length() {
        let body = b"hello";
        let mut response = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\n".to_vec();
        response.extend_from_slice(body);
        assert!(HttpsConnection::is_tls_response_complete(&response, 1));
    }

    #[test]
    fn is_tls_response_complete_incomplete_body() {
        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\nshort";
        assert!(!HttpsConnection::is_tls_response_complete(response, 1));
    }

    #[test]
    fn is_tls_response_complete_headers_not_finished() {
        let response = b"HTTP/1.1 200 OK\r\n";
        assert!(!HttpsConnection::is_tls_response_complete(response, 1));
    }

    #[test]
    fn is_tls_response_complete_chunked_zero_terminator_without_transfer_encoding_header() {
        let response = b"HTTP/1.1 200 OK\r\n\r\n0\r\n\r\n";
        assert!(HttpsConnection::is_tls_response_complete(response, 1));
    }

    #[test]
    fn is_tls_response_complete_chunked_cr_prefix_variant() {
        let response = b"HTTP/1.1 200 OK\r\n\r\n\r\n0\r\n\r\n";
        assert!(HttpsConnection::is_tls_response_complete(response, 1));
    }

    #[test]
    fn is_tls_response_complete_content_length_header_whitespace_tolerance() {
        let mut response = b"HTTP/1.1 200 OK\r\nContent-Length:    3\r\n\r\n".to_vec();
        response.extend_from_slice(b"abc");
        assert!(HttpsConnection::is_tls_response_complete(&response, 2));
    }

    #[test]
    fn handshake_strategies_exhaustive_full_order() {
        let cfg = TlsConfig {
            fallback_strategy: FallbackStrategy::Exhaustive,
            ..Default::default()
        };
        let strategies = HttpsConnection::handshake_strategies_for_fallback(&cfg);
        assert_eq!(
            strategies,
            vec![
                ExtensionStrategy::Modern,
                ExtensionStrategy::Standard,
                ExtensionStrategy::Minimal,
                ExtensionStrategy::MaxCompatibility,
            ]
        );
    }
}

// Note: Full integration tests for HttpsConnection are in the main client tests
// since they require actual TLS handshakes and HTTPS servers
