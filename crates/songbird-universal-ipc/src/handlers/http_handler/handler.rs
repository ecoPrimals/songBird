// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::IpcResult;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, instrument};

use super::env_discovery::EnvCryptoDiscovery;
use super::factory::DefaultHttpClientFactory;
use super::traits::HttpClientFactory;
use super::types::{HttpRequestParams, HttpResponseResult};

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

                let result = self
                    .handle_post(url, body, content_type, headers)
                    .await
                    .map_err(|e| e.to_string())?;

                serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
            }
            _ => Err(format!("Unknown method: {method}")),
        }
    }
}
