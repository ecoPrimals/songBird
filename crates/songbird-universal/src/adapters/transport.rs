// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Unified [`CapabilityTransport`] for capability adapters (tarpc, JSON-RPC, HTTP).

#[cfg(test)]
use std::sync::Mutex;
#[cfg(test)]
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{Value, json};
use songbird_http_client::SongbirdHttpClient;
use songbird_types::{SongbirdError, SongbirdResult};
use std::sync::Arc;

use crate::JsonRpcClient;
use crate::TarpcClient;

/// How an adapter reached its provider (for stable [`std::fmt::Debug`] output).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterTransportKind {
    /// `tarpc://` endpoint
    Tarpc,
    /// `unix://` JSON-RPC
    JsonRpc,
    /// HTTP(S) or unknown scheme (HTTP fallback)
    Http,
}

/// Detect transport kind from endpoint URL (matches adapter `new()` logic).
pub fn transport_kind_for_endpoint(endpoint: &str) -> AdapterTransportKind {
    if endpoint.starts_with("tarpc://") {
        AdapterTransportKind::Tarpc
    } else if endpoint.starts_with("unix://") {
        AdapterTransportKind::JsonRpc
    } else {
        AdapterTransportKind::Http
    }
}

fn join_base_path(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
}

/// Unified transport for capability-based RPC calls.
/// Abstracts tarpc, JSON-RPC, and HTTP protocols behind a single interface.
#[async_trait]
pub trait CapabilityTransport: Send + Sync + std::fmt::Debug {
    /// Call an RPC method with optional parameters.
    async fn call_method(&self, method: &str, params: Option<Value>) -> SongbirdResult<Value>;

    /// Send a GET request to a path relative to the HTTP base (HTTP); RPC transports map paths
    /// to the correct `call_method` / JSON-RPC wire shape.
    async fn get(&self, path: &str) -> SongbirdResult<Value>;

    /// Send a POST with a body to a path relative to the HTTP base (HTTP); RPC transports map
    /// well-known paths to JSON-RPC / tarpc methods.
    async fn post(&self, path: &str, body: Value) -> SongbirdResult<Value>;
}

// --- tarpc -----------------------------------------------------------------

/// Wraps [`TarpcClient`]; `get` / `post` map resource paths to RPC methods.
#[derive(Debug)]
pub struct TarpcTransport(pub TarpcClient);

#[async_trait]
impl CapabilityTransport for TarpcTransport {
    async fn call_method(&self, method: &str, params: Option<Value>) -> SongbirdResult<Value> {
        self.0.call_method(method, params).await
    }

    async fn get(&self, path: &str) -> SongbirdResult<Value> {
        match path.trim_start_matches('/') {
            "metrics/security" => self.0.call_method("get_security_metrics", None).await,
            "metrics/compute" => self.0.call_method("get_compute_metrics", None).await,
            "metrics/ai" => self.0.call_method("get_ai_metrics", None).await,
            "api/v1/identity" => self.0.call_method("identity", None).await,
            other => Err(SongbirdError::network(format!(
                "Unknown GET path for tarpc transport: {other}"
            ))),
        }
    }

    async fn post(&self, path: &str, body: Value) -> SongbirdResult<Value> {
        let p = path.trim_start_matches('/');
        match p {
            "auth/verify" => self.0.call_method("verify_auth", Some(body)).await,
            "api/v1/trust/evaluate" => self.0.call_method("trust.evaluate_peer", Some(body)).await,
            _ => self.0.call_method(p, Some(body)).await,
        }
    }
}

// --- JSON-RPC -------------------------------------------------------------

/// Wraps [`JsonRpcClient`]; `get` maps `metrics/security` to legacy `get_metrics` + type filter.
#[derive(Debug)]
pub struct JsonRpcTransport(pub JsonRpcClient);

#[async_trait]
impl CapabilityTransport for JsonRpcTransport {
    async fn call_method(&self, method: &str, params: Option<Value>) -> SongbirdResult<Value> {
        self.0.call_method(method, params).await
    }

    async fn get(&self, path: &str) -> SongbirdResult<Value> {
        match path.trim_start_matches('/') {
            "metrics/security" => {
                self.0.call_method("get_metrics", Some(json!({"type": "security"}))).await
            }
            "metrics/compute" => self.0.call_method("get_compute_metrics", None).await,
            "metrics/ai" => self.0.call_method("get_ai_metrics", None).await,
            "api/v1/identity" => self.0.call_method("identity", None).await,
            other => Err(SongbirdError::network(format!(
                "Unknown GET path for JSON-RPC transport: {other}"
            ))),
        }
    }

    async fn post(&self, path: &str, body: Value) -> SongbirdResult<Value> {
        let p = path.trim_start_matches('/');
        match p {
            "auth/verify" => self.0.call_method("verify_auth", Some(body)).await,
            "api/v1/trust/evaluate" => self.0.call_method("trust.evaluate_peer", Some(body)).await,
            _ => self.0.call_method(p, Some(body)).await,
        }
    }
}

// --- HTTP -----------------------------------------------------------------

/// HTTP(S) transport: full URLs from base endpoint + path; status checks; optional auth mapping.
#[derive(Clone)]
pub struct HttpTransport {
    pub base: String,
    client: SongbirdHttpClient,
}

impl std::fmt::Debug for HttpTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpTransport").field("base", &self.base).finish()
    }
}

impl HttpTransport {
    /// Create transport for a base URL (e.g. `http://localhost:8080`).
    pub fn new(base: impl Into<String>, client: SongbirdHttpClient) -> Self {
        Self {
            base: base.into(),
            client,
        }
    }

    async fn get_inner(&self, path: &str) -> SongbirdResult<Value> {
        let url = join_base_path(&self.base, path);
        let response = self
            .client
            .get(&url)
            .await
            .map_err(|e| SongbirdError::network(format!("HTTP GET failed for {url}: {e}")))?;
        if !(200..300).contains(&response.status) {
            let status = response.status;
            let p = path.trim_start_matches('/');
            return Err(match p {
                "metrics/security" => {
                    SongbirdError::security(format!("HTTP {status}: Security metrics unavailable"))
                }
                "metrics/compute" => {
                    SongbirdError::service("compute", format!("HTTP {status}: Metrics unavailable"))
                }
                "metrics/ai" => {
                    SongbirdError::service("ai", format!("HTTP {status}: AI metrics unavailable"))
                }
                "api/v1/identity" => SongbirdError::security(format!(
                    "Identity request failed: {status} - {}",
                    response.body
                )),
                _ => SongbirdError::network(format!("HTTP {status} for GET {url}")),
            });
        }
        Ok(response.body)
    }

    async fn post_inner(&self, path: &str, body: Value) -> SongbirdResult<Value> {
        let url = join_base_path(&self.base, path);
        let response = self
            .client
            .post(&url, body)
            .await
            .map_err(|e| SongbirdError::network(format!("HTTP POST failed for {url}: {e}")))?;

        let is_auth_verify = path.trim_start_matches('/') == "auth/verify";
        if is_auth_verify && !(200..300).contains(&response.status) {
            // Match prior SecurityAdapter behavior: non-2xx → Unauthorized (no hard error).
            return Ok(json!("Unauthorized"));
        }

        if !(200..300).contains(&response.status) {
            let status = response.status;
            let p = path.trim_start_matches('/');
            return Err(if p == "api/v1/trust/evaluate" {
                SongbirdError::security(format!(
                    "Trust evaluation failed: {status} - {}",
                    response.body
                ))
            } else {
                SongbirdError::network(format!("HTTP {status} for POST {url}"))
            });
        }
        Ok(response.body)
    }
}

#[async_trait]
impl CapabilityTransport for HttpTransport {
    async fn call_method(&self, method: &str, params: Option<Value>) -> SongbirdResult<Value> {
        let body = params.unwrap_or_else(|| json!({}));
        self.post_inner(method, body).await
    }

    async fn get(&self, path: &str) -> SongbirdResult<Value> {
        self.get_inner(path).await
    }

    async fn post(&self, path: &str, body: Value) -> SongbirdResult<Value> {
        self.post_inner(path, body).await
    }
}

/// Build default transport for an adapter endpoint (same scheme rules as `Adapter::new`).
pub fn build_default_transport(endpoint: &str) -> SongbirdResult<Arc<dyn CapabilityTransport>> {
    let kind = transport_kind_for_endpoint(endpoint);
    Ok(match kind {
        AdapterTransportKind::Tarpc => {
            let client = TarpcClient::new(endpoint).map_err(|e| {
                SongbirdError::configuration(format!("Failed to create tarpc client: {e}"))
            })?;
            Arc::new(TarpcTransport(client))
        }
        AdapterTransportKind::JsonRpc => {
            let client = JsonRpcClient::new(endpoint).map_err(|e| {
                SongbirdError::configuration(format!("Failed to create JSON-RPC client: {e}"))
            })?;
            Arc::new(JsonRpcTransport(client))
        }
        AdapterTransportKind::Http => {
            Arc::new(HttpTransport::new(endpoint.to_string(), SongbirdHttpClient::from_env()))
        }
    })
}

/// Used by tests to simulate slow transports.
#[cfg(test)]
#[derive(Debug)]
pub struct DelayTransport<T: std::fmt::Debug + CapabilityTransport + Send + Sync + 'static> {
    pub inner: Arc<T>,
    pub delay: Duration,
}

#[cfg(test)]
#[async_trait]
impl<T: std::fmt::Debug + CapabilityTransport + Send + Sync + 'static> CapabilityTransport
    for DelayTransport<T>
{
    async fn call_method(&self, method: &str, params: Option<Value>) -> SongbirdResult<Value> {
        tokio::time::sleep(self.delay).await;
        self.inner.call_method(method, params).await
    }

    async fn get(&self, path: &str) -> SongbirdResult<Value> {
        tokio::time::sleep(self.delay).await;
        self.inner.get(path).await
    }

    async fn post(&self, path: &str, body: Value) -> SongbirdResult<Value> {
        tokio::time::sleep(self.delay).await;
        self.inner.post(path, body).await
    }
}

#[cfg(test)]
pub struct MockTransport {
    pub responses: Mutex<Vec<SongbirdResult<Value>>>,
}

#[cfg(test)]
impl MockTransport {
    pub fn new(responses: Vec<SongbirdResult<Value>>) -> Self {
        Self {
            responses: Mutex::new(responses),
        }
    }

    fn pop(&self) -> SongbirdResult<Value> {
        let mut g = self.responses.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if g.is_empty() {
            return Err(SongbirdError::network("MockTransport: exhausted response queue"));
        }
        g.remove(0)
    }
}

#[cfg(test)]
impl std::fmt::Debug for MockTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MockTransport")
    }
}

#[cfg(test)]
#[async_trait]
impl CapabilityTransport for MockTransport {
    async fn call_method(&self, _method: &str, _params: Option<Value>) -> SongbirdResult<Value> {
        self.pop()
    }

    async fn get(&self, _path: &str) -> SongbirdResult<Value> {
        self.pop()
    }

    async fn post(&self, _path: &str, _body: Value) -> SongbirdResult<Value> {
        self.pop()
    }
}
