// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_socket_discovery() {
    // ✅ Concurrent-safe: Uses discover_socket_path_with (no env vars)
    // Test with explicit socket path
    let env1: HashMap<String, String> =
        HashMap::from([("SONGBIRD_SOCKET".to_string(), "/tmp/test.sock".to_string())]);
    let path = IpcHttpClient::discover_socket_path_with(|name| env1.get(name).cloned());
    assert_eq!(path, PathBuf::from("/tmp/test.sock"));

    // Test with family ID (no explicit socket — falls back to /tmp)
    let env2: HashMap<String, String> =
        HashMap::from([("SONGBIRD_FAMILY_ID".to_string(), "test".to_string())]);
    let path = IpcHttpClient::discover_socket_path_with(|name| env2.get(name).cloned());
    assert!(path.to_string_lossy().contains("songbird-test.sock"));
}

#[test]
fn test_socket_discovery_songbird_socket_wins_over_ipc_socket() {
    let env: HashMap<String, String> = HashMap::from([
        ("SONGBIRD_SOCKET".to_string(), "/explicit/primary.sock".to_string()),
        ("SONGBIRD_IPC_SOCKET".to_string(), "/explicit/secondary.sock".to_string()),
    ]);
    let path = IpcHttpClient::discover_socket_path_with(|name| env.get(name).cloned());
    assert_eq!(path, PathBuf::from("/explicit/primary.sock"));
}

#[test]
fn test_socket_discovery_ipc_socket_when_no_primary() {
    let env: HashMap<String, String> =
        HashMap::from([("SONGBIRD_IPC_SOCKET".to_string(), "/only/ipc.sock".to_string())]);
    let path = IpcHttpClient::discover_socket_path_with(|name| env.get(name).cloned());
    assert_eq!(path, PathBuf::from("/only/ipc.sock"));
}

#[test]
fn test_socket_discovery_family_id_alias() {
    let env: HashMap<String, String> =
        HashMap::from([("FAMILY_ID".to_string(), "prod".to_string())]);
    let path = IpcHttpClient::discover_socket_path_with(|name| env.get(name).cloned());
    assert!(path.to_string_lossy().contains("songbird-prod.sock"));
}

#[test]
fn test_response_is_success_and_headers() {
    let ok = Response {
        status: 201,
        headers: HashMap::from([("X-Test".to_string(), "1".to_string())]),
        body: vec![],
    };
    assert!(ok.is_success());
    assert_eq!(ok.status(), 201);
    assert_eq!(ok.headers().get("X-Test"), Some(&"1".to_string()));

    let fail = Response {
        status: 404,
        headers: HashMap::new(),
        body: vec![],
    };
    assert!(!fail.is_success());
}

#[tokio::test]
async fn test_response_text_and_bytes() {
    let r = Response {
        status: 200,
        headers: HashMap::new(),
        body: b"hello utf8".to_vec(),
    };
    assert_eq!(r.text().await.expect("utf8 body"), "hello utf8");

    let raw = Response {
        status: 200,
        headers: HashMap::new(),
        body: vec![0, 159, 146, 150],
    };
    assert!(raw.text().await.is_err());
    let bytes = Response {
        status: 200,
        headers: HashMap::new(),
        body: vec![1, 2, 3],
    };
    assert_eq!(bytes.bytes().await, vec![1, 2, 3]);
}

#[tokio::test]
#[ignore = "requires running Songbird instance"]
async fn test_http_get() {
    let client = IpcHttpClient::new().await.unwrap();
    let response = client.get("https://httpbin.org/get").await.unwrap();
    assert_eq!(response.status(), 200);
}

#[tokio::test]
#[ignore = "requires running Songbird instance"]
async fn test_http_post_json() {
    let client = IpcHttpClient::new().await.unwrap();
    let body = json!({"test": "data"});

    let response =
        client.post("https://httpbin.org/post").await.json(&body).unwrap().send().await.unwrap();

    assert_eq!(response.status(), 200);
}

#[test]
fn test_ipc_http_client_builder_default() {
    let _ = IpcHttpClientBuilder::default();
}

#[tokio::test]
async fn test_ipc_http_client_debug_format() {
    let c = IpcHttpClient::builder()
        .socket_path(PathBuf::from("/tmp/dbg.sock"))
        .build()
        .await
        .expect("build");
    let s = format!("{c:?}");
    assert!(s.contains("IpcHttpClient"));
    assert!(s.contains("dbg.sock"));
}

#[tokio::test]
async fn test_request_builder_headers_and_json() {
    let client = IpcHttpClient::builder()
        .socket_path(PathBuf::from("/tmp/rb.sock"))
        .build()
        .await
        .expect("build");
    let rb = client.post("https://example.com").await;
    let rb = rb.header("X-Test", "1");
    let rb = rb.json(&json!({"a": 1})).expect("json body");
    assert!(rb.send().await.is_err());
}

#[tokio::test]
async fn test_request_builder_body_bytes() {
    let client = IpcHttpClient::builder()
        .socket_path(PathBuf::from("/tmp/rb2.sock"))
        .build()
        .await
        .expect("build");
    let rb = client.put("https://example.com/x").await;
    let _rb = rb.body(vec![1, 2, 3]);
}

#[tokio::test]
async fn test_response_json_roundtrip() {
    let r = Response {
        status: 200,
        headers: HashMap::new(),
        body: br#"{"ok":true}"#.to_vec(),
    };
    let v: serde_json::Value = r.json().await.expect("json");
    assert_eq!(v["ok"], true);
}

#[tokio::test]
async fn test_response_json_invalid() {
    let r = Response {
        status: 200,
        headers: HashMap::new(),
        body: b"{".to_vec(),
    };
    assert!(r.json::<serde_json::Value>().await.is_err());
}

#[tokio::test]
async fn test_delete_returns_err_without_ipc() {
    let client = IpcHttpClient::builder()
        .socket_path(PathBuf::from("/tmp/del.sock"))
        .build()
        .await
        .expect("build");
    assert!(client.delete("https://example.com").await.is_err());
}
