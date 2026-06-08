// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! HTTP Remote Connection — plain HTTP JSON-RPC for LAN peers.
//!
//! Used as a fallback when BTSP is unavailable (e.g., security provider denies
//! crypto operations without a capability token). Connects directly to the
//! remote peer's HTTP JSON-RPC endpoint.
//!
//! This is the appropriate transport for LAN mesh peers that serve plain HTTP
//! on their federation port.

use anyhow::{Context, Result};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use serde_json::Value;
use songbird_types::TrustLevel;
use std::time::Duration;
use tracing::{debug, warn};

use super::check_operation_allowed;

const HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// HTTP remote connection for LAN peers.
///
/// Sends JSON-RPC 2.0 requests over plain HTTP to the peer's network endpoint.
/// No TLS, no BTSP — for trusted LAN environments where crypto provider is
/// unavailable or the peer doesn't support encrypted channels.
pub struct HttpRemoteConnection {
    peer_id: String,
    endpoint: String,
    trust_level: TrustLevel,
    allowed_capabilities: Vec<String>,
    denied_capabilities: Vec<String>,
}

impl HttpRemoteConnection {
    /// Create an HTTP remote connection at the specified trust level.
    pub fn new(peer_id: String, endpoint: String, trust_level: TrustLevel) -> Self {
        Self {
            peer_id,
            allowed_capabilities: trust_level.default_allowed_capabilities(),
            denied_capabilities: trust_level.default_denied_capabilities(),
            endpoint,
            trust_level,
        }
    }

    pub fn trust_level(&self) -> TrustLevel {
        self.trust_level
    }

    pub fn allowed_capabilities(&self) -> &[String] {
        &self.allowed_capabilities
    }

    pub fn denied_capabilities(&self) -> &[String] {
        &self.denied_capabilities
    }

    pub fn is_operation_allowed(&self, operation: &str) -> bool {
        check_operation_allowed(operation, &self.allowed_capabilities, &self.denied_capabilities)
    }

    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Call a JSON-RPC method on the remote peer via HTTP POST.
    pub async fn call(&self, operation: &str, request: Value) -> Result<Value> {
        if !self.is_operation_allowed(operation) {
            anyhow::bail!(
                "Operation '{}' denied at trust level {} for peer '{}'",
                operation,
                self.trust_level.name(),
                self.peer_id
            );
        }

        debug!("🌐 HTTP call '{}' to peer '{}' at {}", operation, self.peer_id, self.endpoint);

        let jsonrpc_url = format!("{}/jsonrpc", self.endpoint.trim_end_matches('/'));

        let jsonrpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": operation,
            "params": request,
            "id": 1
        });

        let body_bytes =
            serde_json::to_vec(&jsonrpc_request).context("Failed to serialize JSON-RPC request")?;

        let uri: hyper::Uri = jsonrpc_url
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid peer URI '{}': {}", jsonrpc_url, e))?;

        let http_request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(&uri)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(body_bytes)))
            .context("Failed to build HTTP request")?;

        let client = Client::builder(TokioExecutor::new()).build_http::<Full<Bytes>>();

        let response = tokio::time::timeout(HTTP_REQUEST_TIMEOUT, client.request(http_request))
            .await
            .map_err(|_| anyhow::anyhow!("HTTP request to peer '{}' timed out", self.peer_id))?
            .context(format!("HTTP POST to peer '{}' failed", self.peer_id))?;

        if !response.status().is_success() {
            warn!(
                "Peer '{}' returned HTTP {} for operation '{}'",
                self.peer_id,
                response.status(),
                operation
            );
        }

        let body = response
            .into_body()
            .collect()
            .await
            .context("Failed to read response body")?
            .to_bytes();

        let json_response: Value =
            serde_json::from_slice(&body).context("Failed to parse JSON-RPC response")?;

        if let Some(error) = json_response.get("error") {
            let msg = error.get("message").and_then(Value::as_str).unwrap_or("unknown error");
            anyhow::bail!("Remote peer '{}' error: {}", self.peer_id, msg);
        }

        json_response.get("result").cloned().ok_or_else(|| {
            anyhow::anyhow!("No result in JSON-RPC response from '{}'", self.peer_id)
        })
    }

    pub async fn close(&self) -> Result<()> {
        Ok(())
    }
}
