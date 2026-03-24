// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(
    clippy::clone_on_ref_ptr,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions"
)]
// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Integration tests for Songbird HTTP Client

use songbird_http_client::SongbirdHttpClient;
use std::collections::HashMap;

/// Test client creation
#[test]
fn test_client_creation() {
    let client = SongbirdHttpClient::new("/tmp/beardog-nat0.sock");
    // If this doesn't panic, client was created successfully
    drop(client);
}

/// Test HTTP request structure
#[test]
fn test_http_request_params() {
    use serde_json::json;

    let params = json!({
        "method": "GET",
        "url": "https://example.com",
        "headers": {},
        "body": null
    });

    assert_eq!(params["method"], "GET");
    assert_eq!(params["url"], "https://example.com");
}

/// Test HTTP response structure
#[test]
fn test_http_response_structure() {
    use serde_json::json;
    use songbird_http_client::HttpResponse;

    let response = HttpResponse {
        status: 200,
        headers: HashMap::new(),
        body: json!({"result": "ok"}),
    };

    assert_eq!(response.status, 200);
    assert_eq!(response.body["result"], "ok");
}

/// Test Pure Rust verification
#[test]
fn test_pure_rust() {
    assert!(songbird_http_client::is_pure_rust());
}

/// Test version info
#[test]
fn test_version_info() {
    let version = songbird_http_client::VERSION;
    assert!(!version.is_empty());
    assert!(version.starts_with("0."));
}
