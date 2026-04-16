// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::IpcServiceHandler;
use crate::handlers::http_handler::HttpRequestParams;
use serde_json::Value;
use tracing::info;

impl IpcServiceHandler {
    /// Handle `http.request` method - Full HTTP/HTTPS request
    pub(super) async fn handle_http_request(&self, params: Value) -> Result<Value, String> {
        let params: HttpRequestParams =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        info!("HTTP request via IPC: {} {}", params.method, params.url);

        let result = self
            .http_handler
            .handle_request(params)
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))?;

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `http.get` method - GET request shorthand
    pub(super) async fn handle_http_get(&self, params: Value) -> Result<Value, String> {
        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'url' parameter".to_string())?;

        info!("HTTP GET via IPC: {}", url);

        let result =
            self.http_handler.handle_get(url).await.map_err(|e| format!("HTTP GET failed: {e}"))?;

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }

    /// Handle `http.post` method - POST request shorthand
    pub(super) async fn handle_http_post(&self, params: Value) -> Result<Value, String> {
        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'url' parameter".to_string())?;

        let body = params
            .get("body")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'body' parameter".to_string())?;

        let content_type = params.get("content_type").and_then(|v| v.as_str());

        let headers: std::collections::HashMap<String, String> = params
            .get("headers")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        info!("HTTP POST via IPC: {}", url);

        let result = self
            .http_handler
            .handle_post(url, body, content_type, headers)
            .await
            .map_err(|e| format!("HTTP POST failed: {e}"))?;

        serde_json::to_value(result).map_err(|e| format!("Serialization error: {e}"))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use super::super::IpcServiceHandler;
    use crate::handlers::http_handler::{
        HttpClient, HttpClientFactory, HttpHandler, HttpResponse, RotatingMockClient,
    };
    use crate::registry::ServiceRegistry;
    use crate::tower_atomic::JsonRpcHandler;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn handler_with_ok_response() -> IpcServiceHandler {
        let mock_response = HttpResponse {
            status_code: 200,
            headers: HashMap::from([("X-Test".to_string(), "ok".to_string())]),
            body: json!({"hello": "world"}),
        };
        let inner = Arc::new(RotatingMockClient::new(vec![mock_response]));
        let factory = Arc::new(HttpClientFactory::InjectTest {
            client: Arc::new(HttpClient::Rotating(Arc::clone(&inner))),
        });
        let http_handler = Arc::new(HttpHandler::new(factory));
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        IpcServiceHandler::with_http_handler(registry, http_handler)
    }

    fn handler_with_failing_factory() -> IpcServiceHandler {
        let http_handler = Arc::new(HttpHandler::new(Arc::new(HttpClientFactory::FailingCreate)));
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        IpcServiceHandler::with_http_handler(registry, http_handler)
    }

    fn handler_with_failing_client() -> IpcServiceHandler {
        let factory = Arc::new(HttpClientFactory::InjectTest {
            client: Arc::new(HttpClient::AlwaysFailRequest),
        });
        let http_handler = Arc::new(HttpHandler::new(factory));
        let registry = Arc::new(RwLock::new(ServiceRegistry::new()));
        IpcServiceHandler::with_http_handler(registry, http_handler)
    }

    #[tokio::test]
    async fn http_request_happy_path_via_json_rpc() {
        let handler = handler_with_ok_response();
        let params = json!({
            "url": "https://example.com/path",
            "method": "GET",
            "headers": {},
            "body": null,
            "timeout_ms": 5000_u64
        });
        let v = handler.handle("http.request", params).await.expect("http.request");
        assert_eq!(v["status_code"], 200);
        assert_eq!(v["body"], json!({"hello": "world"}).to_string());
        assert_eq!(v["headers"]["X-Test"], "ok");
        assert!(v["elapsed_ms"].is_number());
    }

    #[tokio::test]
    async fn http_request_invalid_params_returns_error() {
        let handler = handler_with_ok_response();
        let err = handler
            .handle("http.request", json!("not-an-object"))
            .await
            .expect_err("invalid params");
        assert!(err.contains("Invalid params"), "unexpected: {err}");
    }

    #[tokio::test]
    async fn http_request_missing_required_url_field_fails_deserialize() {
        let handler = handler_with_ok_response();
        let err = handler
            .handle("http.request", json!({"method": "GET"}))
            .await
            .expect_err("missing url");
        assert!(err.contains("Invalid params"), "unexpected: {err}");
    }

    #[tokio::test]
    async fn http_request_propagates_upstream_failure() {
        let handler = handler_with_failing_client();
        let err = handler
            .handle(
                "http.request",
                json!({
                    "url": "https://example.com",
                    "method": "GET"
                }),
            )
            .await
            .expect_err("upstream");
        assert!(err.contains("HTTP request failed"), "unexpected: {err}");
        assert!(err.contains("injected request failure"), "unexpected: {err}");
    }

    #[tokio::test]
    async fn http_request_propagates_factory_failure() {
        let handler = handler_with_failing_factory();
        let err = handler
            .handle(
                "http.request",
                json!({
                    "url": "https://example.com",
                    "method": "GET"
                }),
            )
            .await
            .expect_err("factory");
        assert!(err.contains("HTTP request failed"), "unexpected: {err}");
    }

    #[tokio::test]
    async fn http_get_happy_path_via_json_rpc() {
        let handler = handler_with_ok_response();
        let v = handler
            .handle("http.get", json!({ "url": "https://example.com" }))
            .await
            .expect("http.get");
        assert_eq!(v["status_code"], 200);
        assert_eq!(v["body"], json!({"hello": "world"}).to_string());
    }

    #[tokio::test]
    async fn http_get_missing_url_parameter() {
        let handler = handler_with_ok_response();
        let err = handler.handle("http.get", json!({})).await.expect_err("missing url");
        assert!(err.contains("Missing 'url'"), "unexpected: {err}");
    }

    #[tokio::test]
    async fn http_get_propagates_upstream_failure() {
        let handler = handler_with_failing_client();
        let err = handler
            .handle("http.get", json!({ "url": "https://example.com" }))
            .await
            .expect_err("upstream");
        assert!(err.contains("HTTP GET failed"), "unexpected: {err}");
    }

    #[tokio::test]
    async fn http_post_happy_path_with_content_type_and_headers() {
        let handler = handler_with_ok_response();
        let v = handler
            .handle(
                "http.post",
                json!({
                    "url": "https://api.example.com/v1",
                    "body": r#"{"a":1}"#,
                    "content_type": "application/json",
                    "headers": { "X-Token": "abc" }
                }),
            )
            .await
            .expect("http.post");
        assert_eq!(v["status_code"], 200);
        assert_eq!(v["body"], json!({"hello": "world"}).to_string());
    }

    #[tokio::test]
    async fn http_post_without_content_type_or_headers() {
        let handler = handler_with_ok_response();
        let v = handler
            .handle(
                "http.post",
                json!({
                    "url": "https://api.example.com/v1",
                    "body": "plain"
                }),
            )
            .await
            .expect("http.post");
        assert_eq!(v["status_code"], 200);
    }

    #[tokio::test]
    async fn http_post_missing_url() {
        let handler = handler_with_ok_response();
        let err = handler
            .handle(
                "http.post",
                json!({
                    "body": "x"
                }),
            )
            .await
            .expect_err("missing url");
        assert!(err.contains("Missing 'url'"), "unexpected: {err}");
    }

    #[tokio::test]
    async fn http_post_missing_body() {
        let handler = handler_with_ok_response();
        let err = handler
            .handle(
                "http.post",
                json!({
                    "url": "https://example.com"
                }),
            )
            .await
            .expect_err("missing body");
        assert!(err.contains("Missing 'body'"), "unexpected: {err}");
    }

    #[tokio::test]
    async fn http_post_malformed_headers_json_falls_back_to_empty_map() {
        let handler = handler_with_ok_response();
        let v = handler
            .handle(
                "http.post",
                json!({
                    "url": "https://example.com",
                    "body": "b",
                    "headers": "not-a-map"
                }),
            )
            .await
            .expect("headers ignored");
        assert_eq!(v["status_code"], 200);
    }

    #[tokio::test]
    async fn http_post_propagates_upstream_failure() {
        let handler = handler_with_failing_client();
        let err = handler
            .handle(
                "http.post",
                json!({
                    "url": "https://example.com",
                    "body": "{}"
                }),
            )
            .await
            .expect_err("upstream");
        assert!(err.contains("HTTP POST failed"), "unexpected: {err}");
    }
}
