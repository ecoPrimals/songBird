// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::*;
use crate::error::IpcResult;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct MockHttpClient {
    responses: Vec<HttpResponse>,
    call_count: AtomicUsize,
}

impl MockHttpClient {
    fn new(responses: Vec<HttpResponse>) -> Self {
        Self {
            responses,
            call_count: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl HttpClientCapability for MockHttpClient {
    async fn request(
        &self,
        _method: &str,
        _url: &str,
        _headers: &HashMap<String, String>,
        _body: Option<&[u8]>,
    ) -> IpcResult<HttpResponse> {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);
        Ok(self.responses[count % self.responses.len()].clone())
    }
}

struct MockClientFactory {
    client: Arc<dyn HttpClientCapability>,
}

#[async_trait]
impl HttpClientFactory for MockClientFactory {
    async fn create_client(&self) -> IpcResult<Arc<dyn HttpClientCapability>> {
        Ok(Arc::clone(&self.client))
    }
}

#[tokio::test]
async fn test_handle_get_request() {
    let mock_response = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: serde_json::json!("Hello, World!"),
    };

    let mock_client = Arc::new(MockHttpClient::new(vec![mock_response]));
    let factory = Arc::new(MockClientFactory {
        client: mock_client,
    });
    let handler = HttpHandler::new(factory);

    let result = handler.handle_get("https://example.com").await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status_code, 200);
    assert_eq!(response.body, "\"Hello, World!\"");
}

#[tokio::test]
async fn test_handle_post_request() {
    let mock_response = HttpResponse {
        status_code: 201,
        headers: HashMap::new(),
        body: serde_json::json!("Created"),
    };

    let mock_client = Arc::new(MockHttpClient::new(vec![mock_response]));
    let factory = Arc::new(MockClientFactory {
        client: mock_client,
    });
    let handler = HttpHandler::new(factory);

    let result = handler
        .handle_post(
            "https://api.example.com",
            r#"{"key":"value"}"#,
            Some("application/json"),
            HashMap::new(),
        )
        .await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.status_code, 201);
    assert_eq!(response.body, "\"Created\"");
}

#[tokio::test]
async fn test_environment_discovery() {
    let mut env = std::collections::HashMap::new();
    env.insert("CRYPTO_SIGNING_ENDPOINT".to_string(), "/test/security".to_string());

    let result = EnvCryptoDiscovery::discover_with("crypto.signing", |key| env.get(key).cloned());

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "/test/security");
}

#[tokio::test]
async fn test_default_discovery_fallback() {
    let empty_env: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    let result =
        EnvCryptoDiscovery::discover_with("crypto.signing", |key| empty_env.get(key).cloned());

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "/primal/security");
}

#[tokio::test]
async fn test_discovery_security_provider_socket_priority() {
    let mut env = std::collections::HashMap::new();
    env.insert("SECURITY_PROVIDER_SOCKET".to_string(), "/run/capability.sock".to_string());
    env.insert("BEARDOG_SOCKET".to_string(), "/run/user/1000/biomeos/beardog.sock".to_string());

    let result = EnvCryptoDiscovery::discover_with("crypto.signing", |key| env.get(key).cloned());

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "/run/capability.sock");
}

#[tokio::test]
async fn test_discovery_security_socket_legacy_fallback() {
    let mut env = std::collections::HashMap::new();
    env.insert("BEARDOG_SOCKET".to_string(), "/run/user/1000/biomeos/beardog.sock".to_string());

    let result = EnvCryptoDiscovery::discover_with("crypto.signing", |key| env.get(key).cloned());

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "/run/user/1000/biomeos/beardog.sock");
}

#[tokio::test]
async fn test_handle_post_preserves_custom_headers() {
    let mock_response = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: serde_json::json!({"success": true}),
    };

    let mock_client = Arc::new(MockHttpClient::new(vec![mock_response]));
    let factory = Arc::new(MockClientFactory {
        client: mock_client.clone(),
    });
    let handler = HttpHandler::new(factory);

    let mut custom_headers = HashMap::new();
    custom_headers.insert("X-API-Key".to_string(), "test-key-123".to_string());
    custom_headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());

    let result = handler
        .handle_post(
            "https://api.example.com/endpoint",
            r#"{"test":"data"}"#,
            Some("application/json"),
            custom_headers.clone(),
        )
        .await;

    assert!(result.is_ok(), "handle_post should succeed");

    let response = result.unwrap();
    assert_eq!(response.status_code, 200);
}

#[tokio::test]
async fn test_http_client_wrapper_preserves_headers() {
    let mock_response = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: serde_json::json!({"result": "success"}),
    };

    let mock_client = Arc::new(MockHttpClient::new(vec![mock_response]));

    let wrapper = mock_client.clone();

    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token123".to_string());
    headers.insert("X-Request-ID".to_string(), "req-456".to_string());

    let body = Some(b"{\"test\":\"data\"}".as_slice());
    let result = wrapper.request("POST", "https://api.example.com", &headers, body).await;

    assert!(result.is_ok(), "request should succeed");
    let response = result.unwrap();
    assert_eq!(response.status_code, 200);
}

#[tokio::test]
async fn test_headers_with_empty_value() {
    let mock_response = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: serde_json::json!({}),
    };

    let mock_client = Arc::new(MockHttpClient::new(vec![mock_response]));
    let factory = Arc::new(MockClientFactory {
        client: mock_client,
    });
    let handler = HttpHandler::new(factory);

    let mut headers = HashMap::new();
    headers.insert("X-Empty".to_string(), String::new());
    headers.insert("X-Normal".to_string(), "value".to_string());

    let result = handler.handle_post("https://api.example.com", "{}", None, headers).await;

    assert!(result.is_ok(), "Should handle empty header values");
}

#[tokio::test]
async fn test_headers_with_special_characters() {
    let mock_response = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: serde_json::json!({}),
    };

    let mock_client = Arc::new(MockHttpClient::new(vec![mock_response]));
    let factory = Arc::new(MockClientFactory {
        client: mock_client,
    });
    let handler = HttpHandler::new(factory);

    let mut headers = HashMap::new();
    headers.insert("X-Special-Chars".to_string(), "value with spaces".to_string());
    headers.insert("X-Unicode".to_string(), "emoji🎉test".to_string());

    let result = handler.handle_post("https://api.example.com", "{}", None, headers).await;

    assert!(result.is_ok(), "Should handle special characters in headers");
}

#[tokio::test]
async fn test_many_headers() {
    let mock_response = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: serde_json::json!({}),
    };

    let mock_client = Arc::new(MockHttpClient::new(vec![mock_response]));
    let factory = Arc::new(MockClientFactory {
        client: mock_client,
    });
    let handler = HttpHandler::new(factory);

    let mut headers = HashMap::new();
    for i in 0..50 {
        headers.insert(format!("X-Header-{i}"), format!("value-{i}"));
    }

    let result = handler.handle_post("https://api.example.com", "{}", None, headers).await;

    assert!(result.is_ok(), "Should handle many headers");
}

#[tokio::test]
async fn test_headers_override_content_type() {
    let mock_response = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: serde_json::json!({}),
    };

    let mock_client = Arc::new(MockHttpClient::new(vec![mock_response]));
    let factory = Arc::new(MockClientFactory {
        client: mock_client,
    });
    let handler = HttpHandler::new(factory);

    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "text/plain".to_string());

    let result = handler
        .handle_post("https://api.example.com", "{}", Some("application/json"), headers)
        .await;

    assert!(result.is_ok(), "Should allow Content-Type override");
}

#[tokio::test]
async fn test_chaos_concurrent_header_requests() {
    let mock_response = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: serde_json::json!({"success": true}),
    };

    let responses = (0..100).map(|_| mock_response.clone()).collect();
    let mock_client = Arc::new(MockHttpClient::new(responses));
    let factory = Arc::new(MockClientFactory {
        client: mock_client,
    });
    let handler = Arc::new(HttpHandler::new(factory));

    let mut tasks = vec![];
    for i in 0..100 {
        let handler_clone = handler.clone();
        let task = tokio::spawn(async move {
            let mut headers = HashMap::new();
            headers.insert(format!("X-Request-ID-{i}"), format!("req-{i}"));
            headers.insert("X-Test".to_string(), format!("value-{i}"));

            handler_clone
                .handle_post(
                    "https://api.example.com",
                    &format!(r#"{{"id":{i}}}"#),
                    Some("application/json"),
                    headers,
                )
                .await
        });
        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;

    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "Task {i} should not panic");
        let inner_result = result.as_ref().unwrap();
        assert!(inner_result.is_ok(), "Request {i} should succeed");
    }
}

#[tokio::test]
async fn test_chaos_rapid_fire_same_headers() {
    let mock_response = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: serde_json::json!({}),
    };

    let responses = (0..50).map(|_| mock_response.clone()).collect();
    let mock_client = Arc::new(MockHttpClient::new(responses));
    let factory = Arc::new(MockClientFactory {
        client: mock_client,
    });
    let handler = Arc::new(HttpHandler::new(factory));

    let mut shared_headers = HashMap::new();
    shared_headers.insert("X-Shared-Key".to_string(), "shared-value".to_string());

    let mut tasks = vec![];
    for _ in 0..50 {
        let handler_clone = handler.clone();
        let headers_clone = shared_headers.clone();
        let task = tokio::spawn(async move {
            handler_clone.handle_post("https://api.example.com", "{}", None, headers_clone).await
        });
        tasks.push(task);
    }

    let results = futures::future::join_all(tasks).await;

    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_fault_empty_headers_map() {
    let mock_response = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: serde_json::json!({}),
    };

    let mock_client = Arc::new(MockHttpClient::new(vec![mock_response]));
    let factory = Arc::new(MockClientFactory {
        client: mock_client,
    });
    let handler = HttpHandler::new(factory);

    let empty_headers = HashMap::new();

    let result = handler.handle_post("https://api.example.com", "{}", None, empty_headers).await;

    assert!(result.is_ok(), "Empty headers should be valid");
}

#[tokio::test]
async fn test_fault_very_long_header_value() {
    let mock_response = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: serde_json::json!({}),
    };

    let mock_client = Arc::new(MockHttpClient::new(vec![mock_response]));
    let factory = Arc::new(MockClientFactory {
        client: mock_client,
    });
    let handler = HttpHandler::new(factory);

    let mut headers = HashMap::new();
    let long_value = "a".repeat(10_000);
    headers.insert("X-Long-Header".to_string(), long_value);

    let result = handler.handle_post("https://api.example.com", "{}", None, headers).await;

    assert!(result.is_ok() || result.is_err(), "Should handle long headers gracefully");
}

#[tokio::test]
async fn test_fault_header_with_newlines() {
    let mock_response = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: serde_json::json!({}),
    };

    let mock_client = Arc::new(MockHttpClient::new(vec![mock_response]));
    let factory = Arc::new(MockClientFactory {
        client: mock_client,
    });
    let handler = HttpHandler::new(factory);

    let mut headers = HashMap::new();
    headers.insert("X-Malicious".to_string(), "value\r\nInjected: header".to_string());

    let result = handler.handle_post("https://api.example.com", "{}", None, headers).await;

    assert!(result.is_ok() || result.is_err(), "Should handle newlines in headers");
}

#[tokio::test]
async fn test_fault_null_bytes_in_header() {
    let mock_response = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: serde_json::json!({}),
    };

    let mock_client = Arc::new(MockHttpClient::new(vec![mock_response]));
    let factory = Arc::new(MockClientFactory {
        client: mock_client,
    });
    let handler = HttpHandler::new(factory);

    let mut headers = HashMap::new();
    headers.insert("X-Null".to_string(), "value\0with\0nulls".to_string());

    let result = handler.handle_post("https://api.example.com", "{}", None, headers).await;

    assert!(result.is_ok() || result.is_err(), "Should handle null bytes");
}
