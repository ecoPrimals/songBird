// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2025 ecoPrimals
//! HTTP Delegation Handler
//!
//! Handler for delegating HTTP/HTTPS requests to external services.
//! Used by coordination / AI capability clients to make outbound HTTP calls via Songbird's pure Rust HTTP client.

use serde::Deserialize;
use serde_json::Value;
use tracing::info;

use crate::ipc::jsonrpc::JsonRpcError;
use songbird_http_client::SongbirdHttpClient;

/// Handle http.request - Delegate HTTP requests to external services
///
/// NEW (Jan 20, 2026): Upstream integration from biomeOS.
/// Enables AI coordination adapters (e.g. Anthropic) to delegate HTTP requests through Songbird.
///
/// **Request Format**:
/// ```json
/// {
///   "method": "POST",
///   "url": "https://api.anthropic.com/v1/messages",
///   "headers": {
///     "anthropic-version": "2023-06-01",
///     "content-type": "application/json",
///     "x-api-key": "sk-ant-..."
///   },
///   "body": { ... }
/// }
/// ```
///
/// **Response Format**:
/// ```json
/// {
///   "status": 200,
///   "headers": { "content-type": "application/json" },
///   "body": { ... }
/// }
/// ```
pub async fn handle_http_request(params: Option<Value>) -> Result<Value, JsonRpcError> {
    #[derive(Deserialize)]
    struct HttpRequestParams {
        method: String,
        url: String,
        #[serde(default)]
        headers: std::collections::HashMap<String, String>,
        #[serde(default)]
        body: Option<Value>,
    }
    
    let params: HttpRequestParams = match params {
        Some(p) => serde_json::from_value(p)
            .map_err(|e| JsonRpcError::invalid_params(e.to_string()))?,
        None => return Err(JsonRpcError::invalid_params("Missing params")),
    };
    
    info!("🌐 HTTP delegation (Pure Rust with Neural API capability translation): {} {}", params.method, params.url);
    
    // ✅ EVOLVED: Use Pure Rust HTTP client with Neural API capability translation (TRUE PRIMAL v2)
    // Instead of discovering crypto provider directly, we route through Neural API which:
    // 1. Translates semantic capabilities (crypto.generate_keypair) to actual methods (x25519_generate_ephemeral)
    // 2. Routes to the appropriate provider (security provider)
    // 3. Returns results transparently
    // This enables zero cross-primal coupling and provider-agnostic capability routing.
    
    // ✅ FIX (Feb 4, 2026): Use XDG-compliant discovery instead of hardcoded path
    let neural_api_socket = songbird_http_client::discover_neural_api_socket();
    
    let client = SongbirdHttpClient::new(neural_api_socket);
    
    // Make request via Pure Rust client
    let response = client
        .request(
            &params.method,
            &params.url,
            params.headers,
            params.body,
        )
        .await
        .map_err(|e| JsonRpcError::internal_error(&format!("HTTP request failed: {}", e)))?;
    
    info!("✅ HTTP delegation complete (Pure Rust): {} (status: {})", params.url, response.status);
    
    Ok(serde_json::json!({
        "status": response.status,
        "headers": response.headers,
        "body": response.body
    }))
}
