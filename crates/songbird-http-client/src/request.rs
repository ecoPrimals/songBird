// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP request building
//!
//! Handles construction of HTTP request bytes with adaptive headers based on domain rules.

use crate::error::Result;
use crate::http_config::HttpClientConfig;
use hyper::Uri;
use std::collections::HashMap;
use tracing::trace;

/// HTTP request builder
///
/// Constructs raw HTTP/1.1 request bytes with:
/// - Request line (method, path, HTTP version)
/// - Host header
/// - Adaptive headers based on domain rules
/// - Optional JSON body with Content-Length
pub struct RequestBuilder;

impl RequestBuilder {
    /// Build HTTP/1.1 request bytes
    ///
    /// # Arguments
    ///
    /// * `uri` - Request URI (scheme, host, path)
    /// * `method` - HTTP method (GET, POST, etc.)
    /// * `config` - HTTP client configuration (for adaptive headers)
    /// * `caller_headers` - Headers provided by caller (override defaults)
    /// * `body` - Optional JSON body
    ///
    /// # Returns
    ///
    /// Raw HTTP request bytes ready to send over the wire
    ///
    /// # Errors
    ///
    /// Returns error if JSON serialization fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let uri: Uri = "https://example.com/api".parse()?;
    /// let config = HttpClientConfig::standard();
    /// let headers = HashMap::new();
    /// let body = Some(serde_json::json!({"key": "value"}));
    ///
    /// let request_bytes = RequestBuilder::build(&uri, "POST", &config, &headers, body.as_ref())?;
    /// ```
    pub fn build(
        uri: &Uri,
        method: &str,
        config: &HttpClientConfig,
        caller_headers: &HashMap<String, String>,
        body: Option<&serde_json::Value>,
    ) -> Result<Vec<u8>> {
        let mut request = Vec::new();

        // Request line: "GET /path HTTP/1.1\r\n"
        Self::write_request_line(&mut request, method, uri);

        // Get host for header routing (include port for non-standard ports per HTTP/1.1)
        let host = uri.host().unwrap_or("unknown");
        let scheme = uri.scheme_str().unwrap_or("http");
        let default_port = if scheme == "https" {
            443
        } else {
            80
        };
        let host_header = match uri.port_u16() {
            Some(port) if port != default_port => format!("{host}:{port}"),
            _ => host.to_string(),
        };

        // Host header (always first after request line)
        Self::write_host_header(&mut request, &host_header);

        // Get merged headers from config (defaults + domain rules + caller overrides)
        let headers = config.headers_for_domain(host, caller_headers);

        // DEBUG: Log caller headers vs merged headers (Issue #2 investigation - Jan 28, 2026)
        tracing::info!("🔍 RequestBuilder → caller_headers: {:?}", caller_headers);
        tracing::info!("🔍 RequestBuilder → merged_headers: {:?}", headers);

        // Log applied headers for debugging
        Self::log_header_application(config, host, &headers);

        // Write headers (sorted for deterministic output)
        Self::write_headers(&mut request, &headers);

        // Write body with Content-Length if present
        Self::write_body(&mut request, body)?;

        // DEBUG: Log final HTTP request (Issue #2 investigation - Jan 28, 2026)
        if let Ok(request_str) = std::str::from_utf8(&request) {
            let lines: Vec<&str> = request_str.lines().collect();
            tracing::info!("🔍 Final HTTP request ({} lines):", lines.len());
            for (i, line) in lines.iter().enumerate() {
                if i < 20 {
                    // Log first 20 lines to avoid flooding logs
                    tracing::info!("  {}: {}", i + 1, line);
                }
            }
            if lines.len() > 20 {
                tracing::info!("  ... ({} more lines)", lines.len() - 20);
            }
        }

        Ok(request)
    }

    /// Write HTTP request line
    ///
    /// Format: "METHOD /path HTTP/1.1\r\n"
    fn write_request_line(request: &mut Vec<u8>, method: &str, uri: &Uri) {
        let path = uri.path_and_query().map_or("/", http::uri::PathAndQuery::as_str);
        request.extend_from_slice(format!("{method} {path} HTTP/1.1\r\n").as_bytes());
    }

    /// Write Host header
    ///
    /// Always written first after request line per HTTP/1.1 spec
    fn write_host_header(request: &mut Vec<u8>, host: &str) {
        request.extend_from_slice(format!("Host: {host}\r\n").as_bytes());
    }

    /// Log header application for debugging
    fn log_header_application(
        config: &HttpClientConfig,
        host: &str,
        headers: &HashMap<String, String>,
    ) {
        if config.is_bot_protected(host) {
            trace!("🛡️  {} is bot-protected - applying adaptive headers", host);
        }
        trace!("📋 Applying {} headers for {}", headers.len(), host);
    }

    /// Write HTTP headers
    ///
    /// Headers are sorted alphabetically for deterministic output (important for testing)
    fn write_headers(request: &mut Vec<u8>, headers: &HashMap<String, String>) {
        // Sort headers for deterministic output
        let mut header_pairs: Vec<_> = headers.iter().collect();
        header_pairs.sort_by(|a, b| a.0.cmp(b.0));

        // DEBUG: Log each header being written (Issue #2 investigation - Jan 28, 2026)
        for (key, value) in &header_pairs {
            if !key.eq_ignore_ascii_case("host") {
                tracing::debug!("🔍 Writing header: {}: {}", key, value);
            }
        }

        for (key, value) in header_pairs {
            // Skip Host (already written)
            if key.eq_ignore_ascii_case("host") {
                continue;
            }
            request.extend_from_slice(format!("{key}: {value}\r\n").as_bytes());
        }
    }

    /// Write request body with Content-Length
    ///
    /// If body is present:
    /// - Serialize to JSON
    /// - Write Content-Length header
    /// - Write blank line (end of headers)
    /// - Write body bytes
    ///
    /// If no body:
    /// - Write blank line (end of headers)
    fn write_body(request: &mut Vec<u8>, body: Option<&serde_json::Value>) -> Result<()> {
        if let Some(b) = body {
            let body_bytes = serde_json::to_vec(b)?;
            request
                .extend_from_slice(format!("Content-Length: {}\r\n", body_bytes.len()).as_bytes());
            request.extend_from_slice(b"\r\n");
            request.extend_from_slice(&body_bytes);
        } else {
            request.extend_from_slice(b"\r\n");
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
#[allow(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::http_config::HttpClientConfig;

    #[test]
    fn test_build_get_request() {
        let uri: Uri = "https://example.com/api/test".parse().unwrap();
        let config = HttpClientConfig::minimal(); // No default headers
        let headers = HashMap::new();

        let request = RequestBuilder::build(&uri, "GET", &config, &headers, None).unwrap();
        let request_str = String::from_utf8_lossy(&request);

        assert!(request_str.starts_with("GET /api/test HTTP/1.1\r\n"));
        assert!(request_str.contains("Host: example.com\r\n"));
        assert!(request_str.ends_with("\r\n\r\n")); // Empty body
    }

    #[test]
    fn test_build_post_request_with_body() {
        let uri: Uri = "https://api.example.com/data".parse().unwrap();
        let config = HttpClientConfig::minimal();
        let headers = HashMap::new();
        let body = serde_json::json!({"key": "value"});

        let request = RequestBuilder::build(&uri, "POST", &config, &headers, Some(&body)).unwrap();
        let request_str = String::from_utf8_lossy(&request);

        assert!(request_str.starts_with("POST /data HTTP/1.1\r\n"));
        assert!(request_str.contains("Host: api.example.com\r\n"));
        assert!(request_str.contains("Content-Length: 15\r\n")); // {"key":"value"}
        assert!(request_str.contains(r#"{"key":"value"}"#));
    }

    #[test]
    fn test_custom_headers() {
        let uri: Uri = "https://example.com/".parse().unwrap();
        let config = HttpClientConfig::minimal();
        let mut headers = HashMap::new();
        headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
        headers.insert("Authorization".to_string(), "Bearer token123".to_string());

        let request = RequestBuilder::build(&uri, "GET", &config, &headers, None).unwrap();
        let request_str = String::from_utf8_lossy(&request);

        assert!(request_str.contains("X-Custom-Header: custom-value\r\n"));
        assert!(request_str.contains("Authorization: Bearer token123\r\n"));
    }

    #[test]
    fn test_header_sorting() {
        let uri: Uri = "https://example.com/".parse().unwrap();
        let config = HttpClientConfig::minimal();
        let mut headers = HashMap::new();
        headers.insert("Z-Last".to_string(), "last".to_string());
        headers.insert("A-First".to_string(), "first".to_string());
        headers.insert("M-Middle".to_string(), "middle".to_string());

        let request = RequestBuilder::build(&uri, "GET", &config, &headers, None).unwrap();
        let request_str = String::from_utf8_lossy(&request);

        // Find positions of headers
        let pos_first = request_str.find("A-First").unwrap();
        let pos_middle = request_str.find("M-Middle").unwrap();
        let pos_last = request_str.find("Z-Last").unwrap();

        // Verify alphabetical order
        assert!(pos_first < pos_middle);
        assert!(pos_middle < pos_last);
    }

    #[test]
    fn test_adaptive_headers() {
        let uri: Uri = "https://example.com/".parse().unwrap();
        let config = HttpClientConfig::standard(); // Has User-Agent
        let headers = HashMap::new();

        let request = RequestBuilder::build(&uri, "GET", &config, &headers, None).unwrap();
        let request_str = String::from_utf8_lossy(&request);

        // Should include User-Agent from config
        assert!(request_str.contains("User-Agent:"));
    }

    #[test]
    fn test_path_with_query() {
        let uri: Uri = "https://example.com/api?key=value&foo=bar".parse().unwrap();
        let config = HttpClientConfig::minimal();
        let headers = HashMap::new();

        let request = RequestBuilder::build(&uri, "GET", &config, &headers, None).unwrap();
        let request_str = String::from_utf8_lossy(&request);

        assert!(request_str.starts_with("GET /api?key=value&foo=bar HTTP/1.1\r\n"));
    }

    #[test]
    fn test_host_header_includes_explicit_port() {
        let uri: Uri = "https://example.com:4443/path".parse().unwrap();
        let config = HttpClientConfig::minimal();
        let request = RequestBuilder::build(&uri, "GET", &config, &HashMap::new(), None).unwrap();
        let request_str = String::from_utf8_lossy(&request);
        assert!(request_str.contains("Host: example.com:4443\r\n"));
    }

    #[test]
    fn test_host_header_omits_default_https_port() {
        let uri: Uri = "https://example.com:443/path".parse().unwrap();
        let config = HttpClientConfig::minimal();
        let request = RequestBuilder::build(&uri, "GET", &config, &HashMap::new(), None).unwrap();
        let request_str = String::from_utf8_lossy(&request);
        assert!(request_str.contains("Host: example.com\r\n"));
    }

    #[test]
    fn test_request_line_root_path() {
        let uri: Uri = "https://example.com".parse().unwrap();
        let config = HttpClientConfig::minimal();
        let request = RequestBuilder::build(&uri, "GET", &config, &HashMap::new(), None).unwrap();
        let request_str = String::from_utf8_lossy(&request);
        assert!(request_str.starts_with("GET / HTTP/1.1\r\n"));
    }
}
