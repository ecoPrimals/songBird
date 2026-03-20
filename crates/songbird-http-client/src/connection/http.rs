// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Plain HTTP connection handling (no TLS)
//!
//! Handles HTTP requests over plain TCP connections using Hyper.

use crate::error::Result;
use crate::types::HttpResponse;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, Uri};
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use tokio::net::TcpStream;
use tracing::debug;

/// Plain HTTP connection handler
///
/// Manages HTTP/1.1 requests over plain TCP (no TLS).
pub struct HttpConnection;

impl HttpConnection {
    /// Execute HTTP request over plain TCP
    ///
    /// # Arguments
    ///
    /// * `tcp_stream` - Established TCP connection
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
    /// - JSON serialization fails
    /// - HTTP handshake fails
    /// - Request fails
    /// - Response cannot be read
    pub async fn execute(
        tcp_stream: TcpStream,
        uri: &Uri,
        method: &str,
        headers: HashMap<String, String>,
        body: Option<serde_json::Value>,
    ) -> Result<HttpResponse> {
        debug!("📡 Making HTTP request (no TLS)");

        let io = TokioIo::new(tcp_stream);

        // Create HTTP client with handshake
        let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

        // Spawn connection task to drive the connection
        tokio::spawn(async move {
            if let Err(err) = conn.await {
                tracing::error!("Connection error: {:?}", err);
            }
        });

        // Build request
        let mut req_builder = Request::builder().method(method).uri(uri);

        // Add Host header if not already provided (required by HTTP/1.1)
        // Include port for non-standard ports per RFC 7230
        if !headers.contains_key("host")
            && !headers.contains_key("Host")
            && let Some(host) = uri.host()
        {
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
            req_builder = req_builder.header("Host", host_header);
            debug!("📋 Added Host header: {}", uri.host().unwrap_or("unknown"));
        }

        // Add caller-provided headers
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

        // Parse response
        Self::parse_response(response).await
    }

    /// Parse Hyper response into our `HttpResponse` type
    async fn parse_response(
        response: hyper::Response<hyper::body::Incoming>,
    ) -> Result<HttpResponse> {
        // Extract status
        let status = response.status().as_u16();

        // Extract headers
        let response_headers: HashMap<String, String> = response
            .headers()
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|s| (k.as_str().to_string(), s.to_string())))
            .collect();

        // Read body
        let body_bytes = response.into_body().collect().await?.to_bytes();
        let body: serde_json::Value = if body_bytes.is_empty() {
            serde_json::json!("")
        } else {
            // Try to parse as JSON, fall back to string
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::no_effect_underscore_binding)] // Compilation test only
    fn test_http_connection_exists() {
        // Module compilation test - ensure HttpConnection type exists
        let _conn = HttpConnection;
    }

    // Note: Integration tests for HttpConnection are in the main client tests
    // since they require actual TCP connections and servers
}
