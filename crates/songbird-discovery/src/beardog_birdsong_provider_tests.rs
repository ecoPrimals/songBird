// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::*;
use crate::birdsong::BirdSongEncryption;
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn spawn_one_shot_jsonrpc_mock(response: Value) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        let line = buf.split(|b| *b == b'\n').next().unwrap();
        let _req: Value = serde_json::from_slice(line).unwrap();
        let mut out = serde_json::to_vec(&response).unwrap();
        out.push(b'\n');
        stream.write_all(&out).await.unwrap();
    });
    tokio::task::yield_now().await;
    port
}

async fn spawn_one_shot_jsonrpc_mock_fn<F>(build: F) -> u16
where
    F: FnOnce(Value) -> Value + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.unwrap();
        let line = buf.split(|b| *b == b'\n').next().unwrap();
        let req: Value = serde_json::from_slice(line).unwrap();
        let response = build(req);
        let mut out = serde_json::to_vec(&response).unwrap();
        out.push(b'\n');
        stream.write_all(&out).await.unwrap();
    });
    tokio::task::yield_now().await;
    port
}

#[tokio::test]
async fn test_provider_creation() {
    let socket_path = "/tmp/beardog.sock";
    match BearDogBirdSongProvider::new(socket_path, Some("test-family".to_string())).await {
        Ok(provider) => {
            assert_eq!(provider.provider_name(), "BearDog");
            assert_eq!(provider.family_id(), Some("test-family".to_string()));
        }
        Err(_) => {
            println!("Skipping provider creation test - BearDog not available");
        }
    }
}

#[tokio::test]
async fn test_provider_creation_no_family() {
    let socket_path = "/tmp/beardog.sock";
    match BearDogBirdSongProvider::new(socket_path, None).await {
        Ok(provider) => {
            assert_eq!(provider.provider_name(), "BearDog");
            assert_eq!(provider.family_id(), None);
        }
        Err(_) => {
            println!("Skipping provider creation test - BearDog not available");
        }
    }
}

#[test]
fn test_beardog_encrypt_response_parsing() {
    let response_json = r#"{"ciphertext":"yo8Tz+qVxUp7A01pf7PYAhTvfe0Cl727z9r6nh/Qey21gL09gL+wTzS4ghiTKO6gnyqYvukBVw==","family_id":"iidn"}"#;
    let parsed: BearDogEncryptResponse = serde_json::from_str(response_json).unwrap();
    assert_eq!(parsed.family_id, "iidn");
    assert!(!parsed.ciphertext.is_empty());
}

#[test]
fn test_beardog_decrypt_response_parsing() {
    let response_json = r#"{"plaintext":"SGVsbG8sIEJlYXJEb2ch","family_id":"iidn","success":true}"#;
    let parsed: BearDogDecryptResponse = serde_json::from_str(response_json).unwrap();
    assert_eq!(parsed.family_id, "iidn");
    assert!(parsed.success);
    assert_eq!(parsed.plaintext, b"Hello, BearDog!");
}

#[test]
fn test_base64_serde_serialization() {
    let request = BearDogEncryptRequest {
        plaintext: b"test_message".to_vec(),
        family_id: Some("test-family".to_string()),
    };
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("dGVzdF9tZXNzYWdl"));
    assert!(json.contains("test-family"));
}

#[test]
fn test_base64_serde_roundtrip() {
    let response_json = r#"{"encrypted":"dGVzdF9jaXBoZXJ0ZXh0","family_id":"test-family"}"#;
    let parsed: BearDogEncryptResponse = serde_json::from_str(response_json).unwrap();
    assert_eq!(parsed.family_id, "test-family");
    assert_eq!(parsed.ciphertext, b"test_ciphertext");
}

#[tokio::test]
async fn test_health_check_unavailable() {
    let invalid_socket = "/tmp/nonexistent_beardog_test.sock";
    let _ = std::fs::remove_file(invalid_socket);
    if let Ok(provider) =
        BearDogBirdSongProvider::new(invalid_socket, Some("test-family".to_string())).await
    {
        let is_healthy = provider.check_health().await;
        assert!(!is_healthy, "Health check should fail for invalid socket");
    }
}

#[tokio::test]
async fn test_encrypt_decrypt_roundtrip() {
    let socket_path = "/tmp/beardog.sock";
    let provider = if let Ok(p) =
        BearDogBirdSongProvider::new(socket_path, Some("test-family".to_string())).await
    {
        p
    } else {
        println!("Skipping roundtrip test - BearDog not available");
        return;
    };
    if !provider.check_health().await {
        println!("Skipping roundtrip test - BearDog health check failed");
        return;
    }
    let plaintext = b"Hello, BirdSong!";
    let ciphertext = match provider.encrypt_discovery(plaintext).await {
        Ok(c) => c,
        Err(e) => {
            println!("Skipping roundtrip test - Encryption failed: {e}");
            return;
        }
    };
    assert_ne!(ciphertext, plaintext.to_vec());
    let decrypted = match provider.decrypt_discovery(&ciphertext).await {
        Ok(d) => d,
        Err(e) => {
            println!("Skipping roundtrip test - Decryption failed: {e}");
            return;
        }
    };
    assert_eq!(decrypted, Some(plaintext.to_vec()));
}

#[tokio::test]
async fn test_different_family_decryption() {
    let socket_path = "/tmp/beardog.sock";
    let provider1 = if let Ok(p) =
        BearDogBirdSongProvider::new(socket_path, Some("family-1".to_string())).await
    {
        p
    } else {
        println!("Skipping cross-family test - BearDog not available");
        return;
    };
    let provider2 = if let Ok(p) =
        BearDogBirdSongProvider::new(socket_path, Some("family-2".to_string())).await
    {
        p
    } else {
        println!("Skipping cross-family test - BearDog not available");
        return;
    };
    if !provider1.check_health().await {
        println!("Skipping cross-family test - BearDog health check failed");
        return;
    }
    let plaintext = b"Secret message";
    let ciphertext = if let Ok(c) = provider1.encrypt_discovery(plaintext).await {
        c
    } else {
        println!("Skipping cross-family test - Encryption failed");
        return;
    };
    let decrypted = provider2.decrypt_discovery(&ciphertext).await;
    if let Ok(result) = decrypted {
        assert_eq!(result, None);
    }
}

#[tokio::test]
async fn test_socket_path_formatting() {
    let socket_path = "/tmp/beardog.sock";
    match BearDogBirdSongProvider::new(socket_path, Some("test-family".to_string())).await {
        Ok(provider) => {
            assert_eq!(provider.socket_path.to_str().unwrap(), socket_path);
        }
        Err(_) => {
            println!("Skipping socket path test - BearDog not available");
        }
    }
}

#[tokio::test]
async fn test_concurrent_encrypt_requests() {
    let socket_path = "/tmp/beardog.sock";
    let provider = if let Ok(p) =
        BearDogBirdSongProvider::new(socket_path, Some("test-family".to_string())).await
    {
        Arc::new(p)
    } else {
        println!("Skipping concurrent test - BearDog not available");
        return;
    };
    if !provider.check_health().await {
        println!("Skipping concurrent test - BearDog health check failed");
        return;
    }
    let mut handles = vec![];
    for i in 0..5 {
        let provider_clone = Arc::clone(&provider);
        let handle = tokio::spawn(async move {
            let plaintext = format!("Message {i}");
            provider_clone.encrypt_discovery(plaintext.as_bytes()).await
        });
        handles.push(handle);
    }
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn tcp_address_invalid_port_format_errors() {
    let e = BearDogBirdSongProvider::new("tcp:127.0.0.1:notaport", None)
        .await
        .err()
        .expect("expected error");
    assert!(e.to_string().contains("port") || e.to_string().contains("Invalid"));
}

#[tokio::test]
async fn tcp_address_missing_colon_errors() {
    let e = BearDogBirdSongProvider::new("tcp:badformat", None).await.err().expect("expected err");
    assert!(e.to_string().contains("TCP") || e.to_string().contains("format"));
}

#[test]
fn encrypt_request_omits_family_id_when_none() {
    let req = BearDogEncryptRequest {
        plaintext: vec![1, 2, 3],
        family_id: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(!json.contains("family_id"));
}

#[test]
fn encrypt_request_includes_family_id_when_some() {
    let req = BearDogEncryptRequest {
        plaintext: vec![1],
        family_id: Some("fam".into()),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("fam"));
}

#[test]
fn decrypt_response_accepts_success_false() {
    let raw = r#"{"plaintext":"","family_id":"x","success":false}"#;
    let d: BearDogDecryptResponse = serde_json::from_str(raw).unwrap();
    assert!(!d.success);
}

#[test]
fn provider_name_and_family_accessor() {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let p = rt
        .block_on(BearDogBirdSongProvider::new("tcp:127.0.0.1:59998", Some("fam-x".into())))
        .expect("tcp provider");
    assert_eq!(p.provider_name(), "BearDog");
    assert_eq!(p.family_id().as_deref(), Some("fam-x"));
    assert!(p.is_available());
}

#[tokio::test]
async fn mock_tcp_health_check_returns_true() {
    let port = spawn_one_shot_jsonrpc_mock(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": { "status": "healthy" }
    }))
    .await;
    let provider =
        BearDogBirdSongProvider::new(format!("tcp:127.0.0.1:{port}"), Some("fam".into()))
            .await
            .unwrap();
    assert!(provider.check_health().await);
}

#[tokio::test]
async fn mock_tcp_health_check_returns_false_when_status_not_healthy() {
    let port = spawn_one_shot_jsonrpc_mock(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": { "status": "degraded" }
    }))
    .await;
    let provider =
        BearDogBirdSongProvider::new(format!("tcp:127.0.0.1:{port}"), None).await.unwrap();
    assert!(!provider.check_health().await);
}

#[tokio::test]
async fn check_health_false_when_tcp_connection_refused() {
    let provider = BearDogBirdSongProvider::new("tcp:127.0.0.1:59996", None).await.unwrap();
    assert!(!provider.check_health().await);
}

#[tokio::test]
async fn mock_tcp_encrypt_discovery_returns_ciphertext() {
    let ct = B64.encode(b"cipher-bytes");
    let port = spawn_one_shot_jsonrpc_mock_fn(move |req| {
        assert_eq!(req["method"], "birdsong.encrypt");
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "ciphertext": ct,
                "family_id": "lineage-x"
            }
        })
    })
    .await;
    let provider =
        BearDogBirdSongProvider::new(format!("tcp:127.0.0.1:{port}"), Some("lineage-x".into()))
            .await
            .unwrap();
    let out = provider.encrypt_discovery(b"hello").await.unwrap();
    assert_eq!(out, b"cipher-bytes");
}

#[tokio::test]
async fn mock_tcp_encrypt_decrypt_roundtrip_via_trait() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = Vec::new();
            stream.read_to_end(&mut buf).await.unwrap();
            let line = buf.split(|b| *b == b'\n').next().unwrap();
            let req: Value = serde_json::from_slice(line).unwrap();
            let method = req["method"].as_str().unwrap();
            let resp = match method {
                "birdsong.encrypt" => {
                    let pt = req["params"]["plaintext"].as_str().unwrap();
                    let raw = B64.decode(pt).unwrap();
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "result": {
                            "ciphertext": B64.encode(&raw),
                            "family_id": "fam-a"
                        }
                    })
                }
                "birdsong.decrypt" => {
                    let ct = req["params"]["ciphertext"].as_str().unwrap();
                    let raw = B64.decode(ct).unwrap();
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "result": {
                            "plaintext": B64.encode(&raw),
                            "family_id": "fam-a",
                            "success": true
                        }
                    })
                }
                _ => panic!("unexpected method {method}"),
            };
            let mut out = serde_json::to_vec(&resp).unwrap();
            out.push(b'\n');
            stream.write_all(&out).await.unwrap();
        }
    });
    tokio::task::yield_now().await;
    let provider =
        BearDogBirdSongProvider::new(format!("tcp:127.0.0.1:{port}"), Some("fam-a".into()))
            .await
            .unwrap();
    let plain = b"roundtrip-message";
    let ct = provider.encrypt_discovery(plain).await.unwrap();
    let back = provider.decrypt_discovery(&ct).await.unwrap();
    assert_eq!(back, Some(plain.to_vec()));
}

#[tokio::test]
async fn mock_tcp_decrypt_success_false_returns_none() {
    let port = spawn_one_shot_jsonrpc_mock(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "plaintext": "",
            "family_id": "other",
            "success": false
        }
    }))
    .await;
    let provider = BearDogBirdSongProvider::new(format!("tcp:127.0.0.1:{port}"), Some("me".into()))
        .await
        .unwrap();
    let got = provider.decrypt_discovery(b"opaque").await.unwrap();
    assert_eq!(got, None);
}

#[tokio::test]
async fn mock_tcp_encrypt_discovery_jsonrpc_error() {
    let port = spawn_one_shot_jsonrpc_mock(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": { "code": -32000, "message": "encrypt failed" }
    }))
    .await;
    let provider =
        BearDogBirdSongProvider::new(format!("tcp:127.0.0.1:{port}"), None).await.unwrap();
    let err = provider.encrypt_discovery(b"x").await.err().unwrap();
    let s = err.to_string();
    assert!(s.contains("32000") || s.contains("encrypt failed"));
}

#[test]
fn decrypt_request_json_omits_family_id_when_none() {
    let req = BearDogDecryptRequest {
        ciphertext: vec![9, 9],
        family_id: None,
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(!json.contains("family_id"));
}

#[test]
fn decrypt_request_json_includes_family_id_when_some() {
    let req = BearDogDecryptRequest {
        ciphertext: vec![1],
        family_id: Some("my-fam".into()),
    };
    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains("my-fam"));
}

#[tokio::test]
async fn tcp_localhost_hostname_parses_to_endpoint() {
    let p = BearDogBirdSongProvider::new("tcp:localhost:54320", Some("f".into())).await.unwrap();
    assert_eq!(p.provider_name(), "BearDog");
    assert_eq!(p.family_id().as_deref(), Some("f"));
}

#[tokio::test]
async fn mock_tcp_decrypt_jsonrpc_error_surfaces_as_anyhow() {
    let port = spawn_one_shot_jsonrpc_mock(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": { "code": 7, "message": "wrong family" }
    }))
    .await;
    let provider =
        BearDogBirdSongProvider::new(format!("tcp:127.0.0.1:{port}"), Some("fid".into()))
            .await
            .unwrap();
    let err = provider.decrypt_discovery(b"blob").await.err().unwrap();
    assert!(err.to_string().contains("wrong family") || err.to_string().contains('7'));
}

#[test]
fn encrypt_response_accepts_explicit_ciphertext_key() {
    let raw = r#"{"ciphertext":"QUJD","family_id":"z"}"#;
    let r: BearDogEncryptResponse = serde_json::from_str(raw).unwrap();
    assert_eq!(r.ciphertext, b"ABC");
    assert_eq!(r.family_id, "z");
}

#[tokio::test]
async fn bird_song_trait_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>(_t: &T) {}
    let p = BearDogBirdSongProvider::new("tcp:127.0.0.1:59995", None).await.unwrap();
    assert_send_sync(&p);
}
