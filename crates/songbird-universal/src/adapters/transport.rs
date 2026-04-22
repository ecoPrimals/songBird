// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Unified [`CapabilityTransport`] for capability adapters (tarpc, JSON-RPC, HTTP).

#[cfg(test)]
use std::sync::Mutex;
#[cfg(test)]
use std::time::Duration;

use serde_json::{Value, json};
use songbird_http_client::SongbirdHttpClient;
use songbird_types::{SongbirdError, SongbirdResult};
#[cfg(test)]
use std::future::Future;
#[cfg(test)]
use std::pin::Pin;
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

/// Test-only: sleep then delegate (separate type avoids recursive `async fn` on [`CapabilityTransport`]).
#[cfg(test)]
#[derive(Debug)]
pub struct DelayTransport {
    pub inner: Arc<CapabilityTransport>,
    pub delay: Duration,
}

/// Unified transport for capability-based RPC calls (enum dispatch).
#[derive(Debug)]
pub enum CapabilityTransport {
    /// `tarpc://` transport
    Tarpc(TarpcTransport),
    /// Unix-socket JSON-RPC transport
    JsonRpc(JsonRpcTransport),
    /// HTTP(S) transport
    Http(HttpTransport),
    /// Test helper: delay then delegate.
    #[cfg(test)]
    Delay(DelayTransport),
    /// Test mock FIFO responses.
    #[cfg(test)]
    Mock(MockTransport),
}

#[cfg(test)]
fn dispatch_call_method<'a>(
    ct: &'a CapabilityTransport,
    method: String,
    params: Option<Value>,
) -> Pin<Box<dyn Future<Output = SongbirdResult<Value>> + Send + 'a>> {
    Box::pin(async move {
        match ct {
            CapabilityTransport::Tarpc(t) => t.call_method(&method, params).await,
            CapabilityTransport::JsonRpc(t) => t.call_method(&method, params).await,
            CapabilityTransport::Http(t) => t.call_method(&method, params).await,
            CapabilityTransport::Delay(d) => {
                tokio::time::sleep(d.delay).await;
                dispatch_call_method(d.inner.as_ref(), method, params).await
            }
            CapabilityTransport::Mock(m) => m.call_method(&method, params).await,
        }
    })
}

#[cfg(test)]
fn dispatch_get<'a>(
    ct: &'a CapabilityTransport,
    path: String,
) -> Pin<Box<dyn Future<Output = SongbirdResult<Value>> + Send + 'a>> {
    Box::pin(async move {
        match ct {
            CapabilityTransport::Tarpc(t) => t.get(&path).await,
            CapabilityTransport::JsonRpc(t) => t.get(&path).await,
            CapabilityTransport::Http(t) => t.get(&path).await,
            CapabilityTransport::Delay(d) => {
                tokio::time::sleep(d.delay).await;
                dispatch_get(d.inner.as_ref(), path).await
            }
            CapabilityTransport::Mock(m) => m.get(&path).await,
        }
    })
}

#[cfg(test)]
fn dispatch_post<'a>(
    ct: &'a CapabilityTransport,
    path: String,
    body: Value,
) -> Pin<Box<dyn Future<Output = SongbirdResult<Value>> + Send + 'a>> {
    Box::pin(async move {
        match ct {
            CapabilityTransport::Tarpc(t) => t.post(&path, body).await,
            CapabilityTransport::JsonRpc(t) => t.post(&path, body).await,
            CapabilityTransport::Http(t) => t.post(&path, body).await,
            CapabilityTransport::Delay(d) => {
                tokio::time::sleep(d.delay).await;
                dispatch_post(d.inner.as_ref(), path, body).await
            }
            CapabilityTransport::Mock(m) => m.post(&path, body).await,
        }
    })
}

impl CapabilityTransport {
    /// Call an RPC method with optional parameters.
    pub async fn call_method(&self, method: &str, params: Option<Value>) -> SongbirdResult<Value> {
        #[cfg(not(test))]
        {
            match self {
                Self::Tarpc(t) => t.call_method(method, params).await,
                Self::JsonRpc(t) => t.call_method(method, params).await,
                Self::Http(t) => t.call_method(method, params).await,
            }
        }
        #[cfg(test)]
        dispatch_call_method(self, method.to_string(), params).await
    }

    /// Send a GET request to a path relative to the HTTP base (HTTP); RPC transports map paths
    /// to the correct `call_method` / JSON-RPC wire shape.
    pub async fn get(&self, path: &str) -> SongbirdResult<Value> {
        #[cfg(not(test))]
        {
            match self {
                Self::Tarpc(t) => t.get(path).await,
                Self::JsonRpc(t) => t.get(path).await,
                Self::Http(t) => t.get(path).await,
            }
        }
        #[cfg(test)]
        dispatch_get(self, path.to_string()).await
    }

    /// Send a POST with a body to a path relative to the HTTP base (HTTP); RPC transports map
    /// well-known paths to JSON-RPC / tarpc methods.
    pub async fn post(&self, path: &str, body: Value) -> SongbirdResult<Value> {
        #[cfg(not(test))]
        {
            match self {
                Self::Tarpc(t) => t.post(path, body).await,
                Self::JsonRpc(t) => t.post(path, body).await,
                Self::Http(t) => t.post(path, body).await,
            }
        }
        #[cfg(test)]
        dispatch_post(self, path.to_string(), body).await
    }
}

// --- tarpc -----------------------------------------------------------------

/// Wraps [`TarpcClient`]; `get` / `post` map resource paths to RPC methods.
#[derive(Debug)]
pub struct TarpcTransport(pub TarpcClient);

impl TarpcTransport {
    async fn call_method(&self, method: &str, params: Option<Value>) -> SongbirdResult<Value> {
        self.0.call_method(method, params).await
    }

    async fn get(&self, path: &str) -> SongbirdResult<Value> {
        match path.trim_start_matches('/') {
            "metrics/security" => self.0.call_method("get_security_metrics", None).await,
            "metrics/compute" => self.0.call_method("get_compute_metrics", None).await,
            "metrics/storage" => self.0.call_method("get_storage_metrics", None).await,
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

impl JsonRpcTransport {
    async fn call_method(&self, method: &str, params: Option<Value>) -> SongbirdResult<Value> {
        self.0.call_method(method, params).await
    }

    async fn get(&self, path: &str) -> SongbirdResult<Value> {
        match path.trim_start_matches('/') {
            "metrics/security" => {
                self.0.call_method("get_metrics", Some(json!({"type": "security"}))).await
            }
            "metrics/compute" => self.0.call_method("get_compute_metrics", None).await,
            "metrics/storage" => self.0.call_method("get_storage_metrics", None).await,
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
                "metrics/storage" => SongbirdError::service(
                    "storage",
                    format!("HTTP {status}: Storage metrics unavailable"),
                ),
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
pub fn build_default_transport(endpoint: &str) -> SongbirdResult<Arc<CapabilityTransport>> {
    let kind = transport_kind_for_endpoint(endpoint);
    Ok(match kind {
        AdapterTransportKind::Tarpc => {
            let client = TarpcClient::new(endpoint).map_err(|e| {
                SongbirdError::configuration(format!("Failed to create tarpc client: {e}"))
            })?;
            Arc::new(CapabilityTransport::Tarpc(TarpcTransport(client)))
        }
        AdapterTransportKind::JsonRpc => {
            let client = JsonRpcClient::new(endpoint).map_err(|e| {
                SongbirdError::configuration(format!("Failed to create JSON-RPC client: {e}"))
            })?;
            Arc::new(CapabilityTransport::JsonRpc(JsonRpcTransport(client)))
        }
        AdapterTransportKind::Http => Arc::new(CapabilityTransport::Http(HttpTransport::new(
            endpoint.to_string(),
            SongbirdHttpClient::from_env(),
        ))),
    })
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
#[allow(
    clippy::unused_async,
    reason = "async signature matches transport stack; responses are sync"
)]
impl MockTransport {
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

#[cfg(test)]
mod tests {
    use super::{
        AdapterTransportKind, CapabilityTransport, DelayTransport, HttpTransport, JsonRpcTransport,
        MockTransport, TarpcTransport, build_default_transport, transport_kind_for_endpoint,
    };
    use crate::{JsonRpcClient, TarpcClient};
    use serde_json::json;
    use songbird_http_client::SongbirdHttpClient;
    use songbird_types::SongbirdError;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn transport_kind_for_endpoint_variants() {
        assert_eq!(
            transport_kind_for_endpoint("tarpc://127.0.0.1:9000"),
            AdapterTransportKind::Tarpc
        );
        assert_eq!(
            transport_kind_for_endpoint("unix:///tmp/s.sock"),
            AdapterTransportKind::JsonRpc
        );
        assert_eq!(
            transport_kind_for_endpoint("http://localhost:8080"),
            AdapterTransportKind::Http
        );
        assert_eq!(
            transport_kind_for_endpoint("https://a.example/path"),
            AdapterTransportKind::Http
        );
        assert_eq!(transport_kind_for_endpoint("ftp://x"), AdapterTransportKind::Http);
        assert_eq!(transport_kind_for_endpoint(""), AdapterTransportKind::Http);
    }

    #[tokio::test]
    async fn build_default_transport_tarpc_jsonrpc_http() -> songbird_types::SongbirdResult<()> {
        let _t = build_default_transport("tarpc://127.0.0.1:9101")?;
        let _j = build_default_transport("unix:///tmp/songbird-transport-unit.sock")?;

        let server = mockito::Server::new_async().await;
        let _h = build_default_transport(server.url().as_str())?;
        Ok(())
    }

    #[tokio::test]
    async fn build_default_transport_invalid_tarpc() {
        let err = build_default_transport("tarpc://not-a-host:99999").expect_err("invalid port");
        assert!(err.to_string().contains("tarpc") || err.to_string().contains("configuration"));
    }

    #[tokio::test]
    async fn tarpc_transport_unknown_get_returns_error() -> songbird_types::SongbirdResult<()> {
        let client = TarpcClient::new("tarpc://127.0.0.1:9102")?;
        let t = CapabilityTransport::Tarpc(TarpcTransport(client));
        let err = t.get("unknown/path").await.expect_err("unknown path");
        assert!(err.to_string().contains("Unknown GET path for tarpc"), "{}", err);
        Ok(())
    }

    #[tokio::test]
    async fn jsonrpc_transport_unknown_get_returns_error() -> songbird_types::SongbirdResult<()> {
        let client = JsonRpcClient::new("unix:///tmp/songbird-jsonrpc-transport.sock")?;
        let t = CapabilityTransport::JsonRpc(JsonRpcTransport(client));
        let err = t.get("foo/bar").await.expect_err("unknown path");
        assert!(err.to_string().contains("Unknown GET path for JSON-RPC"), "{}", err);
        Ok(())
    }

    #[tokio::test]
    async fn http_transport_get_inner_success_and_errors() -> songbird_types::SongbirdResult<()> {
        let mut server = mockito::Server::new_async().await;
        let base = server.url();

        let _m_sec = server
            .mock("GET", "/metrics/security")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;
        let transport = CapabilityTransport::Http(HttpTransport::new(
            base.clone(),
            SongbirdHttpClient::from_env(),
        ));
        let v = transport.get("metrics/security").await?;
        assert_eq!(v, json!({"ok": true}));

        let mut server2 = mockito::Server::new_async().await;
        let base2 = server2.url();
        let _m503 = server2
            .mock("GET", "/metrics/security")
            .with_status(503)
            .with_body("no")
            .create_async()
            .await;
        let t2 =
            CapabilityTransport::Http(HttpTransport::new(base2, SongbirdHttpClient::from_env()));
        let e = t2.get("metrics/security").await.expect_err("503");
        assert!(e.to_string().contains("503") || e.to_string().contains("Security"));

        let mut server3 = mockito::Server::new_async().await;
        let base3 = server3.url();
        let _m_comp = server3.mock("GET", "/metrics/compute").with_status(502).create_async().await;
        let t3 =
            CapabilityTransport::Http(HttpTransport::new(base3, SongbirdHttpClient::from_env()));
        let e3 = t3.get("metrics/compute").await.expect_err("502");
        assert!(e3.to_string().contains("compute") || e3.to_string().contains("502"));

        let mut server4 = mockito::Server::new_async().await;
        let base4 = server4.url();
        let _m_ai = server4.mock("GET", "/metrics/ai").with_status(500).create_async().await;
        let t4 =
            CapabilityTransport::Http(HttpTransport::new(base4, SongbirdHttpClient::from_env()));
        let e4 = t4.get("metrics/ai").await.expect_err("500");
        assert!(e4.to_string().contains("ai") || e4.to_string().contains("500"));

        let mut server5 = mockito::Server::new_async().await;
        let base5 = server5.url();
        let _m_id = server5
            .mock("GET", "/api/v1/identity")
            .with_status(401)
            .with_body(r#"{"err":"no"}"#)
            .create_async()
            .await;
        let t5 =
            CapabilityTransport::Http(HttpTransport::new(base5, SongbirdHttpClient::from_env()));
        let e5 = t5.get("api/v1/identity").await.expect_err("401");
        assert!(e5.to_string().contains("401") || e5.to_string().contains("Identity"));

        let mut server6 = mockito::Server::new_async().await;
        let base6 = server6.url();
        let _m_misc = server6.mock("GET", "/other").with_status(404).create_async().await;
        let t6 =
            CapabilityTransport::Http(HttpTransport::new(base6, SongbirdHttpClient::from_env()));
        let e6 = t6.get("other").await.expect_err("404");
        assert!(e6.to_string().contains("404") || e6.to_string().contains("HTTP"));

        Ok(())
    }

    #[tokio::test]
    async fn http_transport_post_auth_verify_non_2xx_returns_unauthorized_json()
    -> songbird_types::SongbirdResult<()> {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/auth/verify")
            .with_status(401)
            .with_body(r#"{"error":"bad"}"#)
            .create_async()
            .await;

        let t = CapabilityTransport::Http(HttpTransport::new(
            server.url(),
            SongbirdHttpClient::from_env(),
        ));
        let v = t.post("auth/verify", json!({"token":"x"})).await?;
        assert_eq!(v, json!("Unauthorized"));
        Ok(())
    }

    #[tokio::test]
    async fn http_transport_post_trust_evaluate_error_maps_to_security() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/api/v1/trust/evaluate")
            .with_status(403)
            .with_body(r#"{"denied":true}"#)
            .create_async()
            .await;

        let t = CapabilityTransport::Http(HttpTransport::new(
            server.url(),
            SongbirdHttpClient::from_env(),
        ));
        let err = t.post("api/v1/trust/evaluate", json!({})).await.expect_err("403");
        assert!(err.to_string().contains("Trust") || err.to_string().contains("403"), "{}", err);
    }

    #[tokio::test]
    async fn http_transport_call_method_posts_to_method_path() -> songbird_types::SongbirdResult<()>
    {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/some_method")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"r":1}"#)
            .create_async()
            .await;

        let t = CapabilityTransport::Http(HttpTransport::new(
            server.url(),
            SongbirdHttpClient::from_env(),
        ));
        let v = t.call_method("some_method", None).await?;
        assert_eq!(v, json!({"r": 1}));
        Ok(())
    }

    #[tokio::test]
    async fn http_transport_debug_smoke() {
        let t = CapabilityTransport::Http(HttpTransport::new(
            "http://localhost:1",
            SongbirdHttpClient::from_env(),
        ));
        let s = format!("{t:?}");
        assert!(s.contains("Http"));
        assert!(s.contains("http://localhost:1"));
    }

    #[tokio::test]
    async fn mock_transport_fifo_and_exhausted() {
        let m = Arc::new(CapabilityTransport::Mock(MockTransport::new(vec![
            Ok(json!({"a":1})),
            Err(SongbirdError::network("boom")),
        ])));
        assert_eq!(m.call_method("x", None).await.unwrap(), json!({"a":1}));
        assert!(m.call_method("x", None).await.is_err());
        assert!(m.get("p").await.expect_err("empty").to_string().contains("exhausted"));
    }

    #[tokio::test(start_paused = true)]
    async fn delay_transport_waits_before_delegating() {
        let inner = Arc::new(CapabilityTransport::Mock(MockTransport::new(vec![Ok(
            json!({"delayed":true}),
        )])));
        let delayed = Arc::new(CapabilityTransport::Delay(DelayTransport {
            inner: inner.clone(),
            delay: Duration::from_secs(2),
        }));

        let handle = tokio::spawn(async move { delayed.call_method("m", None).await });

        tokio::time::advance(Duration::from_secs(1)).await;
        tokio::task::yield_now().await;
        assert!(!handle.is_finished());

        tokio::time::advance(Duration::from_secs(1)).await;
        let out = handle.await.expect("join").expect("ok");
        assert_eq!(out, json!({"delayed": true}));
    }

    #[tokio::test]
    async fn mock_transport_arc_roundtrip() {
        let m: Arc<CapabilityTransport> =
            Arc::new(CapabilityTransport::Mock(MockTransport::new(vec![Ok(json!([]))])));
        let v = m.call_method("any", None).await.unwrap();
        assert_eq!(v, json!([]));
    }
}
