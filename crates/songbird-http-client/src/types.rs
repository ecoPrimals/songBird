//! Types for HTTP requests and responses

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Full URL
    pub url: String,
    /// HTTP headers
    pub headers: HashMap<String, String>,
    /// Request body (optional)
    pub body: Option<serde_json::Value>,
}

/// HTTP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    pub body: serde_json::Value,
}

impl HttpRequest {
    /// Create a new HTTP GET request
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            method: "GET".to_string(),
            url: url.into(),
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Create a new HTTP POST request
    pub fn post(url: impl Into<String>, body: serde_json::Value) -> Self {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        Self {
            method: "POST".to_string(),
            url: url.into(),
            headers,
            body: Some(body),
        }
    }

    /// Add a header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set request body
    #[must_use]
    pub fn with_body(mut self, body: serde_json::Value) -> Self {
        self.body = Some(body);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_request() {
        let req = HttpRequest::get("https://example.com");
        assert_eq!(req.method, "GET");
        assert_eq!(req.url, "https://example.com");
        assert!(req.body.is_none());
    }

    #[test]
    fn test_post_request() {
        let body = serde_json::json!({"key": "value"});
        let req = HttpRequest::post("https://example.com/api", body.clone());
        assert_eq!(req.method, "POST");
        assert_eq!(req.body, Some(body));
        assert!(req.headers.contains_key("content-type"));
    }

    #[test]
    fn test_request_builder() {
        let req = HttpRequest::get("https://example.com")
            .header("authorization", "Bearer token")
            .header("user-agent", "songbird");

        assert_eq!(req.headers.get("authorization"), Some(&"Bearer token".to_string()));
        assert_eq!(req.headers.get("user-agent"), Some(&"songbird".to_string()));
    }
}
