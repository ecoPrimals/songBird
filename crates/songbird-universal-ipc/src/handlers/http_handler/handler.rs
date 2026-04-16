// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use crate::error::IpcResult;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, instrument};

use super::http_dispatch::HttpClientFactory;
use super::types::{HttpRequestParams, HttpResponseResult};

/// HTTP/HTTPS IPC Handler
///
/// Handles JSON-RPC 2.0 methods:
/// - `http.request` - Full HTTP/HTTPS request
/// - `http.get` - GET request shorthand
/// - `http.post` - POST request shorthand
pub struct HttpHandler {
    factory: Arc<HttpClientFactory>,
}

impl HttpHandler {
    /// Create new handler with given factory (dependency injection)
    #[must_use]
    pub fn new(factory: Arc<HttpClientFactory>) -> Self {
        Self {
            factory,
        }
    }

    /// Create handler with default environment-based discovery
    #[must_use]
    pub fn with_default_discovery() -> Self {
        Self::new(Arc::new(HttpClientFactory::with_default_crypto_discovery()))
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
        let mut headers = caller_headers;
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::HttpHandler;
    use crate::error::{IpcError, IpcResult};
    use crate::tower_atomic::JsonRpcHandler;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;

    use super::super::http_dispatch::{HttpClient, HttpClientFactory};
    use super::super::test_support::QueuedMockClient;
    use super::super::types::HttpResponse;

    fn sample_ok_response() -> HttpResponse {
        HttpResponse {
            status_code: 200,
            headers: HashMap::from([("X-Mock".into(), "yes".into())]),
            body: json!({"ok": true}),
        }
    }

    fn handler_with_client(mock: Arc<QueuedMockClient>) -> HttpHandler {
        let client = Arc::new(HttpClient::Queued(mock));
        HttpHandler::new(Arc::new(HttpClientFactory::InjectTest {
            client,
        }))
    }

    #[tokio::test]
    async fn jsonrpc_http_request_get_put_delete_via_dispatch() {
        for (verb, expect_body) in [
            ("GET", None::<Vec<u8>>),
            ("PUT", Some(b"put-payload".to_vec())),
            ("DELETE", None::<Vec<u8>>),
        ] {
            let mock = Arc::new(QueuedMockClient::new(vec![Ok(sample_ok_response())]));
            let handler = handler_with_client(Arc::clone(&mock));

            let mut params = json!({
                "url": format!("https://example.test/{verb}"),
                "method": verb,
                "headers": { "X-Verb": verb },
            });
            if let Some(b) = &expect_body {
                params["body"] = json!(String::from_utf8_lossy(b));
            }

            let out = handler.handle("http.request", params).await;
            assert!(out.is_ok(), "{verb}: {out:?}");
            let v = out.expect("ok");
            assert_eq!(v["status_code"], 200);
            assert_eq!(v["headers"]["X-Mock"], "yes");

            let caps = mock.take_captures();
            assert_eq!(caps.len(), 1);
            assert_eq!(caps[0].0, verb);
            assert_eq!(caps[0].1, format!("https://example.test/{verb}"));
            assert_eq!(caps[0].2.get("X-Verb"), Some(&verb.to_string()));
            assert_eq!(caps[0].3.as_ref(), expect_body.as_ref());
        }
    }

    #[tokio::test]
    async fn jsonrpc_http_request_post_body_and_headers_from_params() {
        let mock = Arc::new(QueuedMockClient::new(vec![Ok(sample_ok_response())]));
        let handler = handler_with_client(mock.clone());

        let params = json!({
            "url": "https://example.test/post",
            "method": "POST",
            "headers": { "Authorization": "Bearer t", "X-Custom": "c" },
            "body": "{\"a\":1}",
        });

        let out = handler.handle("http.request", params).await.expect("ok");
        assert_eq!(out["status_code"], 200);

        let caps = mock.take_captures();
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0].0, "POST");
        assert_eq!(caps[0].1, "https://example.test/post");
        assert_eq!(caps[0].2.get("Authorization"), Some(&"Bearer t".to_string()));
        assert_eq!(caps[0].2.get("X-Custom"), Some(&"c".to_string()));
        assert_eq!(caps[0].3.as_deref(), Some(&b"{\"a\":1}"[..]));
    }

    #[tokio::test]
    async fn jsonrpc_http_request_nonstandard_method_passes_through() {
        let mock = Arc::new(QueuedMockClient::new(vec![Ok(sample_ok_response())]));
        let handler = handler_with_client(mock.clone());

        let params = json!({
            "url": "https://example.test/patch",
            "method": "PATCH",
        });

        let out = handler.handle("http.request", params).await.expect("ok");
        assert_eq!(out["status_code"], 200);
        let caps = mock.take_captures();
        assert_eq!(caps[0].0, "PATCH");
    }

    #[tokio::test]
    async fn jsonrpc_http_request_invalid_params_shape() {
        let mock = Arc::new(QueuedMockClient::new(vec![]));
        let handler = handler_with_client(mock);

        let err = handler
            .handle("http.request", json!("not-an-object"))
            .await
            .expect_err("expected invalid params");
        assert!(err.starts_with("Invalid params:"), "got {err:?}");

        let err = handler
            .handle("http.request", json!({"url": 42}))
            .await
            .expect_err("url must be string");
        assert!(err.starts_with("Invalid params:"), "got {err:?}");
    }

    #[tokio::test]
    async fn jsonrpc_http_get_success_and_missing_url() {
        let mock = Arc::new(QueuedMockClient::new(vec![Ok(sample_ok_response())]));
        let handler = handler_with_client(mock.clone());

        let out =
            handler.handle("http.get", json!({ "url": "https://get.test/" })).await.expect("ok");
        assert_eq!(out["status_code"], 200);

        let caps = mock.take_captures();
        assert_eq!(caps[0].0, "GET");
        assert_eq!(caps[0].1, "https://get.test/");
        assert!(caps[0].2.is_empty());
        assert!(caps[0].3.is_none());

        let err = handler.handle("http.get", json!({})).await.expect_err("missing url");
        assert_eq!(err, "Missing 'url' parameter");

        let err =
            handler.handle("http.get", json!({ "url": 99 })).await.expect_err("url not string");
        assert_eq!(err, "Missing 'url' parameter");
    }

    #[tokio::test]
    async fn jsonrpc_http_post_success_missing_url_body_and_malformed_headers() {
        let mock = Arc::new(QueuedMockClient::new(vec![
            Ok(sample_ok_response()),
            Ok(sample_ok_response()),
        ]));
        let handler = handler_with_client(mock.clone());

        let out = handler
            .handle(
                "http.post",
                json!({
                    "url": "https://post.test/",
                    "body": "{}",
                    "content_type": "application/json",
                    "headers": { "X-Api": "k" },
                }),
            )
            .await
            .expect("ok");
        assert_eq!(out["status_code"], 200);

        let caps = mock.take_captures();
        assert_eq!(caps[0].2.get("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(caps[0].2.get("X-Api"), Some(&"k".to_string()));

        assert_eq!(
            handler.handle("http.post", json!({ "body": "{}" })).await.expect_err("url"),
            "Missing 'url' parameter"
        );
        assert_eq!(
            handler.handle("http.post", json!({ "url": "https://x" })).await.expect_err("body"),
            "Missing 'body' parameter"
        );

        let out_bad_headers = handler
            .handle(
                "http.post",
                json!({
                    "url": "https://post.test/h2",
                    "body": "x",
                    "headers": "not-a-map",
                }),
            )
            .await
            .expect("headers default on parse failure");
        assert_eq!(out_bad_headers["status_code"], 200);
        let caps = mock.take_captures();
        let last = caps.last().expect("captures");
        assert!(!last.2.contains_key("X-Api"), "prior headers should not leak");
        assert_eq!(last.3.as_deref(), Some(b"x".as_slice()));
    }

    #[tokio::test]
    async fn jsonrpc_http_post_headers_object_with_invalid_entry_yields_default() {
        let mock = Arc::new(QueuedMockClient::new(vec![Ok(sample_ok_response())]));
        let handler = handler_with_client(mock.clone());

        let _ = handler
            .handle(
                "http.post",
                json!({
                    "url": "https://post.test/",
                    "body": "y",
                    "headers": { "bad": 1 },
                }),
            )
            .await
            .expect("ok");

        let caps = mock.take_captures();
        assert!(caps[0].2.is_empty() || !caps[0].2.contains_key("bad"));
    }

    #[tokio::test]
    async fn jsonrpc_unknown_method_error_format() {
        let mock = Arc::new(QueuedMockClient::new(vec![]));
        let handler = handler_with_client(mock);

        let err = handler.handle("http.options", json!({})).await.expect_err("unknown rpc method");
        assert_eq!(err, "Unknown method: http.options");
    }

    #[tokio::test]
    async fn jsonrpc_map_err_create_client_and_request_and_ipc_display() {
        let handler = HttpHandler::new(Arc::new(HttpClientFactory::FailingCreate));
        let err = handler
            .handle("http.request", json!({ "url": "https://x", "method": "GET" }))
            .await
            .expect_err("factory fails");
        assert!(err.contains("Connection failed") && err.contains("mock factory"), "{err:?}");

        let failing_client =
            Arc::new(QueuedMockClient::new(vec![Err(IpcError::Internal("req failed".into()))]));
        let handler = handler_with_client(failing_client);
        let err = handler
            .handle("http.get", json!({ "url": "https://y" }))
            .await
            .expect_err("request fails");
        assert!(err.contains("Internal error") && err.contains("req failed"), "{err:?}");

        let err = handler
            .handle("http.post", json!({ "url": "https://z", "body": "b" }))
            .await
            .expect_err("second call no queued outcome");
        assert!(
            err.contains("Internal error") && err.contains("no queued mock response"),
            "{err:?}"
        );
    }
}
