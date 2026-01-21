//! Unit tests for songbird-http-client

use songbird_http_client::types::HttpRequest;

#[test]
fn test_http_request_builder() {
    let req = HttpRequest::get("https://example.com/api")
        .header("authorization", "Bearer token123")
        .header("user-agent", "songbird/1.0");
    
    assert_eq!(req.method, "GET");
    assert_eq!(req.url, "https://example.com/api");
    assert_eq!(req.headers.get("authorization"), Some(&"Bearer token123".to_string()));
    assert_eq!(req.headers.get("user-agent"), Some(&"songbird/1.0".to_string()));
    assert!(req.body.is_none());
}

#[test]
fn test_http_post_request() {
    let body = serde_json::json!({"key": "value", "number": 42});
    let req = HttpRequest::post("https://api.example.com/data", body.clone());
    
    assert_eq!(req.method, "POST");
    assert_eq!(req.url, "https://api.example.com/data");
    assert!(req.headers.contains_key("content-type"));
    assert_eq!(req.body, Some(body));
}

#[test]
fn test_http_request_with_body() {
    let body = serde_json::json!({"test": true});
    let req = HttpRequest::get("https://example.com")
        .with_body(body.clone());
    
    assert_eq!(req.body, Some(body));
}

#[test]
fn test_pure_rust_check() {
    // This crate should ALWAYS be Pure Rust
    assert!(songbird_http_client::is_pure_rust());
}

#[test]
fn test_version() {
    let version = songbird_http_client::VERSION;
    // Version should be set from Cargo.toml
    assert!(!version.is_empty());
    // Should be semantic versioning
    assert!(version.contains('.'));
}

