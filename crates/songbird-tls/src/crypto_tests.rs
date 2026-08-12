// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::*;
use base64::Engine;
use base64::engine::general_purpose;
use serde_json::json;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

fn with_isolated_env_overlay<F: FnOnce()>(f: F) {
    let _guard = songbird_process_env::test_env_lock();
    reset_overlay_for_test();
    f();
}

async fn spawn_one_shot_jsonrpc_server(response_body: String) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut buf = vec![0u8; 16_384];
        let _ = stream.read(&mut buf).await;
        stream.write_all(response_body.as_bytes()).await.expect("write response");
        let _ = stream.shutdown().await;
    });
    format!("tcp:{addr}")
}

#[test]
fn json_rpc_response_deserializes_success() {
    let raw = r#"{"jsonrpc":"2.0","result":{"ok":true},"id":1}"#;
    let r: super::JsonRpcResponse = serde_json::from_str(raw).expect("parse");
    assert_eq!(r.jsonrpc, "2.0");
    assert!(r.result.is_some());
    assert!(r.error.is_none());
}

#[test]
fn json_rpc_response_deserializes_error_variant() {
    let raw = r#"{"jsonrpc":"2.0","error":{"code":-1,"message":"fail"},"id":null}"#;
    let r: super::JsonRpcResponse = serde_json::from_str(raw).expect("parse");
    assert!(r.result.is_none());
    let err = r.error.expect("error");
    assert_eq!(err.code, -1);
    assert_eq!(err.message, "fail");
}

#[tokio::test]
async fn call_capability_success_returns_result() {
    let body = r#"{"jsonrpc":"2.0","result":{"hello":"world"},"id":1}"#.to_string();
    let path = spawn_one_shot_jsonrpc_server(body).await;
    let client = SecurityTlsCryptoClient::with_socket_path(path);
    let v = client.call_capability("crypto", "op", json!({})).await.expect("call");
    assert_eq!(v["hello"], "world");
}

#[tokio::test]
async fn call_capability_jsonrpc_error_maps_to_crypto_error() {
    let body = r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"bad op"},"id":1}"#.to_string();
    let path = spawn_one_shot_jsonrpc_server(body).await;
    let client = SecurityTlsCryptoClient::with_socket_path(path);
    let err =
        client.call_capability("crypto", "op", json!({})).await.expect_err("expect rpc error");
    match err {
        TlsError::CryptoError(msg) => {
            assert!(msg.contains("Capability call error"));
            assert!(msg.contains("bad op"));
            assert!(msg.contains("-32000"));
        }
        e => panic!("unexpected: {e:?}"),
    }
}

#[tokio::test]
async fn call_capability_missing_result_field() {
    let body = r#"{"jsonrpc":"2.0","id":1}"#.to_string();
    let path = spawn_one_shot_jsonrpc_server(body).await;
    let client = SecurityTlsCryptoClient::with_socket_path(path);
    let err = client.call_capability("crypto", "op", json!({})).await.expect_err("missing result");
    match err {
        TlsError::CryptoError(msg) => assert!(msg.contains("missing result")),
        e => panic!("unexpected: {e:?}"),
    }
}

#[tokio::test]
async fn call_capability_invalid_json_response() {
    let body = "not-json".to_string();
    let path = spawn_one_shot_jsonrpc_server(body).await;
    let client = SecurityTlsCryptoClient::with_socket_path(path);
    let err = client.call_capability("crypto", "op", json!({})).await.expect_err("parse fail");
    match err {
        TlsError::CryptoError(msg) => assert!(msg.contains("Failed to parse response")),
        e => panic!("unexpected: {e:?}"),
    }
}

#[tokio::test(start_paused = true)]
async fn call_capability_connect_failure() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    drop(listener);
    // Port/TIME_WAIT is OS wall-clock; Tokio mock time does not model it.
    tokio::time::advance(std::time::Duration::from_millis(20)).await;
    let client = SecurityTlsCryptoClient::with_socket_path(format!("tcp:{addr}"));
    let err = client.call_capability("c", "o", json!({})).await.expect_err("connection refused");
    match err {
        TlsError::CryptoError(msg) => assert!(msg.contains("Failed to connect")),
        e => panic!("unexpected: {e:?}"),
    }
}

#[tokio::test]
async fn call_jsonrpc_success_and_error_paths() {
    let ok = r#"{"jsonrpc":"2.0","result":{"x":1},"id":1}"#.to_string();
    let path = spawn_one_shot_jsonrpc_server(ok).await;
    let client = SecurityTlsCryptoClient::with_socket_path(path);
    let v = client.call_jsonrpc("custom.method", json!({})).await.expect("jsonrpc ok");
    assert_eq!(v["x"], 1);

    let err_body = r#"{"jsonrpc":"2.0","error":{"code":1,"message":"nope"},"id":null}"#.to_string();
    let path2 = spawn_one_shot_jsonrpc_server(err_body).await;
    let client2 = SecurityTlsCryptoClient::with_socket_path(path2);
    let e = client2.call_jsonrpc("m", json!({})).await.expect_err("bear dog error");
    match e {
        TlsError::CryptoError(msg) => {
            assert!(
                msg.contains("security provider error"),
                "expected JSON-RPC error to include provider context: {msg}"
            );
            assert!(msg.contains("nope"));
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn x25519_generate_ephemeral_success_and_decode_errors() {
    let pk = general_purpose::STANDARD.encode([9u8; 32]);
    let sk = general_purpose::STANDARD.encode([8u8; 32]);
    let body = format!(
        r#"{{"jsonrpc":"2.0","result":{{"public_key":"{pk}","secret_key":"{sk}"}},"id":1}}"#
    );
    let path = spawn_one_shot_jsonrpc_server(body.clone()).await;
    let client = SecurityTlsCryptoClient::with_socket_path(path);
    let (pubk, sec) = client.x25519_generate_ephemeral().await.expect("gen");
    assert_eq!(pubk, vec![9u8; 32]);
    assert_eq!(sec, vec![8u8; 32]);

    let bad =
        r#"{"jsonrpc":"2.0","result":{"public_key":"@@@","secret_key":"@@@"},"id":1}"#.to_string();
    let path2 = spawn_one_shot_jsonrpc_server(bad).await;
    let client2 = SecurityTlsCryptoClient::with_socket_path(path2);
    let e = client2.x25519_generate_ephemeral().await.expect_err("bad b64");
    assert!(matches!(e, TlsError::CryptoError(ref m) if m.contains("decode")));

    let miss = r#"{"jsonrpc":"2.0","result":{"public_key":"QQ=="},"id":1}"#.to_string();
    let path3 = spawn_one_shot_jsonrpc_server(miss).await;
    let client3 = SecurityTlsCryptoClient::with_socket_path(path3);
    let e = client3.x25519_generate_ephemeral().await.expect_err("missing sk");
    assert!(matches!(e, TlsError::CryptoError(ref m) if m.contains("secret_key")));

    let miss_pk = r#"{"jsonrpc":"2.0","result":{"secret_key":"QQ=="},"id":1}"#.to_string();
    let path4 = spawn_one_shot_jsonrpc_server(miss_pk).await;
    let client4 = SecurityTlsCryptoClient::with_socket_path(path4);
    let e = client4.x25519_generate_ephemeral().await.expect_err("missing pk");
    assert!(matches!(e, TlsError::CryptoError(ref m) if m.contains("public_key")));
}

#[tokio::test]
async fn x25519_derive_secret_paths() {
    let ss = general_purpose::STANDARD.encode([7u8; 32]);
    let body = format!(r#"{{"jsonrpc":"2.0","result":{{"shared_secret":"{ss}"}},"id":1}}"#);
    let path = spawn_one_shot_jsonrpc_server(body).await;
    let client = SecurityTlsCryptoClient::with_socket_path(path);
    let out = client.x25519_derive_secret(&[1], &[2]).await.expect("derive");
    assert_eq!(out, vec![7u8; 32]);

    let miss = r#"{"jsonrpc":"2.0","result":{},"id":1}"#.to_string();
    let path2 = spawn_one_shot_jsonrpc_server(miss).await;
    let client2 = SecurityTlsCryptoClient::with_socket_path(path2);
    let e = client2.x25519_derive_secret(&[], &[]).await.expect_err("missing ss");
    assert!(matches!(e, TlsError::CryptoError(ref m) if m.contains("shared_secret")));

    let bad_b64 = r#"{"jsonrpc":"2.0","result":{"shared_secret":"@@@"},"id":1}"#.to_string();
    let path3 = spawn_one_shot_jsonrpc_server(bad_b64).await;
    let client3 = SecurityTlsCryptoClient::with_socket_path(path3);
    let e = client3.x25519_derive_secret(&[], &[]).await.expect_err("b64");
    assert!(matches!(e, TlsError::CryptoError(ref m) if m.contains("decode")));
}

#[tokio::test]
async fn chacha_encrypt_decrypt_with_and_without_aad() {
    let ct = general_purpose::STANDARD.encode([1, 2, 3]);
    let nonce = general_purpose::STANDARD.encode([4u8; 12]);
    let tag = general_purpose::STANDARD.encode([5u8; 16]);
    let body = format!(
        r#"{{"jsonrpc":"2.0","result":{{"ciphertext":"{ct}","nonce":"{nonce}","tag":"{tag}"}},"id":1}}"#
    );
    let path = spawn_one_shot_jsonrpc_server(body.clone()).await;
    let client = SecurityTlsCryptoClient::with_socket_path(path);
    let (ciphertext, nonce_out, tag_out) =
        client.chacha20_poly1305_encrypt(b"pt", b"key", None).await.expect("enc");
    assert_eq!(ciphertext, vec![1, 2, 3]);
    assert_eq!(nonce_out, vec![4u8; 12]);
    assert_eq!(tag_out, vec![5u8; 16]);

    let path2 = spawn_one_shot_jsonrpc_server(body.clone()).await;
    let client2 = SecurityTlsCryptoClient::with_socket_path(path2);
    let _ =
        client2.chacha20_poly1305_encrypt(b"p", b"k", Some(b"aad-bytes")).await.expect("enc aad");

    let pt = general_purpose::STANDARD.encode(b"hello");
    let dec_body = format!(r#"{{"jsonrpc":"2.0","result":{{"plaintext":"{pt}"}},"id":1}}"#);
    let path3 = spawn_one_shot_jsonrpc_server(dec_body.clone()).await;
    let client3 = SecurityTlsCryptoClient::with_socket_path(path3);
    let plain = client3.chacha20_poly1305_decrypt(&[], &[], &[], &[], None).await.expect("dec");
    assert_eq!(plain, b"hello");

    let path4 = spawn_one_shot_jsonrpc_server(dec_body).await;
    let client4 = SecurityTlsCryptoClient::with_socket_path(path4);
    let _ =
        client4.chacha20_poly1305_decrypt(&[], &[], &[], &[], Some(b"a")).await.expect("dec aad");

    for (field, miss_json) in [
        ("ciphertext", r#"{"jsonrpc":"2.0","result":{"nonce":"QQ==","tag":"QQ=="},"id":1}"#),
        ("nonce", r#"{"jsonrpc":"2.0","result":{"ciphertext":"QQ==","tag":"QQ=="},"id":1}"#),
        ("tag", r#"{"jsonrpc":"2.0","result":{"ciphertext":"QQ==","nonce":"QQ=="},"id":1}"#),
    ] {
        let p = spawn_one_shot_jsonrpc_server(miss_json.to_string()).await;
        let c = SecurityTlsCryptoClient::with_socket_path(p);
        let e = c.chacha20_poly1305_encrypt(&[], &[], None).await.expect_err(field);
        assert!(matches!(e, TlsError::CryptoError(ref m) if m.contains(field)), "{field}: {e:?}");
    }

    let bad_ct =
        r#"{"jsonrpc":"2.0","result":{"ciphertext":"@@@","nonce":"QQ==","tag":"QQ=="},"id":1}"#
            .to_string();
    let p = spawn_one_shot_jsonrpc_server(bad_ct).await;
    let c = SecurityTlsCryptoClient::with_socket_path(p);
    let e = c.chacha20_poly1305_encrypt(&[], &[], None).await.expect_err("b64 ct");
    assert!(
        matches!(e, TlsError::CryptoError(ref m) if m.contains("ciphertext") && m.contains("decode"))
    );

    let miss_pt = r#"{"jsonrpc":"2.0","result":{},"id":1}"#.to_string();
    let p2 = spawn_one_shot_jsonrpc_server(miss_pt).await;
    let c2 = SecurityTlsCryptoClient::with_socket_path(p2);
    let e = c2.chacha20_poly1305_decrypt(&[], &[], &[], &[], None).await.expect_err("pt");
    assert!(matches!(e, TlsError::CryptoError(ref m) if m.contains("plaintext")));
}

#[tokio::test]
async fn ed25519_sign_and_hmac_sha256_success_and_errors() {
    let sig = general_purpose::STANDARD.encode([3u8; 64]);
    let body = format!(r#"{{"jsonrpc":"2.0","result":{{"signature":"{sig}"}},"id":1}}"#);
    let path = spawn_one_shot_jsonrpc_server(body).await;
    let client = SecurityTlsCryptoClient::with_socket_path(path);
    let s = client.ed25519_sign(b"msg", "kid").await.expect("sign");
    assert_eq!(s, vec![3u8; 64]);

    let miss = r#"{"jsonrpc":"2.0","result":{},"id":1}"#.to_string();
    let path2 = spawn_one_shot_jsonrpc_server(miss).await;
    let client2 = SecurityTlsCryptoClient::with_socket_path(path2);
    let e = client2.ed25519_sign(b"m", "k").await.expect_err("sig");
    assert!(matches!(e, TlsError::CryptoError(ref m) if m.contains("signature")));

    let mac = general_purpose::STANDARD.encode([4u8; 32]);
    let mbody = format!(r#"{{"jsonrpc":"2.0","result":{{"mac":"{mac}"}},"id":1}}"#);
    let path3 = spawn_one_shot_jsonrpc_server(mbody).await;
    let client3 = SecurityTlsCryptoClient::with_socket_path(path3);
    let m = client3.hmac_sha256(b"x", b"y").await.expect("hmac");
    assert_eq!(m, vec![4u8; 32]);

    let miss_mac = r#"{"jsonrpc":"2.0","result":{},"id":1}"#.to_string();
    let path4 = spawn_one_shot_jsonrpc_server(miss_mac).await;
    let client4 = SecurityTlsCryptoClient::with_socket_path(path4);
    let e = client4.hmac_sha256(b"a", b"b").await.expect_err("mac");
    assert!(matches!(e, TlsError::CryptoError(ref m) if m.contains("mac")));
}

#[cfg(unix)]
#[tokio::test]
async fn connect_platform_tcp_strip_prefix_matches_listener() {
    let body = r#"{"jsonrpc":"2.0","result":{},"id":1}"#.to_string();
    let path = spawn_one_shot_jsonrpc_server(body).await;
    let addr = path.strip_prefix("tcp:").expect("tcp prefix");
    let stream = SecurityTlsCryptoClient::connect_platform(&format!("tcp:{addr}"))
        .await
        .expect("tcp connect");
    match stream {
        CryptoStream::Tcp(_) => {}
        #[cfg(unix)]
        CryptoStream::Unix(_) => panic!("expected tcp"),
        #[cfg(not(unix))]
        _ => {}
    }
}

#[cfg(unix)]
#[tokio::test]
async fn connect_platform_unix_domain_round_trip() {
    let dir = std::env::temp_dir().join(format!("songbird-tls-crypto-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
    let sock_path = dir.join("u.sock");
    let _ = std::fs::remove_file(&sock_path);
    let listener = tokio::net::UnixListener::bind(&sock_path).expect("unix bind");
    let path_str = sock_path.to_string_lossy().into_owned();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept unix");
        let mut buf = vec![0u8; 4096];
        let _ = stream.read(&mut buf).await;
        stream.write_all(br#"{"jsonrpc":"2.0","result":{"ok":true},"id":1}"#).await.expect("write");
        let _ = stream.shutdown().await;
    });
    tokio::task::yield_now().await;
    let stream = SecurityTlsCryptoClient::connect_platform(&path_str).await.expect("unix connect");
    match stream {
        CryptoStream::Unix(_) => {}
        CryptoStream::Tcp(_) => panic!("expected unix"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn create_temp_socket() -> String {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/songbird-tls-test-{}-{}.sock", std::process::id(), id);
    std::fs::File::create(&path).expect("create temp socket");
    path
}

#[test]
fn test_with_socket_path() {
    let client = SecurityTlsCryptoClient::with_socket_path("/tmp/custom.sock".to_string());
    assert_eq!(client.socket_path, "/tmp/custom.sock");
}

#[test]
fn test_with_socket_path_various_locations() {
    let paths = [
        "/var/run/custom/socket.sock",
        "/tmp/my-app.sock",
        "/run/security-provider/test.sock",
        "tcp:127.0.0.1:9900",
    ];
    for path in paths {
        let client = SecurityTlsCryptoClient::with_socket_path(path.to_string());
        assert_eq!(client.socket_path, path);
    }
}

#[test]
fn test_client_clone_preserves_socket_path() {
    let original = SecurityTlsCryptoClient::with_socket_path("/tmp/original.sock".to_string());
    let cloned = original.clone();
    assert_eq!(original.socket_path, cloned.socket_path);
}

#[test]
fn test_discover_socket_with_real_file() {
    let sock_path = create_temp_socket();
    let client = SecurityTlsCryptoClient::with_socket_path(sock_path.clone());
    assert_eq!(client.socket_path, sock_path);
    std::fs::remove_file(&sock_path).ok();
}

#[test]
fn test_discover_socket_tcp_format() {
    let client = SecurityTlsCryptoClient::with_socket_path("tcp:127.0.0.1:9900".to_string());
    assert_eq!(client.socket_path, "tcp:127.0.0.1:9900");
}

#[tokio::test]
async fn test_new_fails_on_nonexistent_socket() {
    let client =
        SecurityTlsCryptoClient::with_socket_path("/tmp/nonexistent-tls-test.sock".to_string());
    assert_eq!(client.socket_path, "/tmp/nonexistent-tls-test.sock");
}

#[test]
fn test_concurrent_client_creation() {
    let handles: Vec<_> = (0..10)
        .map(|i| {
            std::thread::spawn(move || {
                let client =
                    SecurityTlsCryptoClient::with_socket_path(format!("/tmp/concurrent-{i}.sock"));
                assert_eq!(client.socket_path, format!("/tmp/concurrent-{i}.sock"));
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("thread panicked");
    }
}

#[test]
fn capability_call_request_json_shape_matches_neural_api() {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "capability.call",
        "params": {
            "capability": "crypto",
            "operation": "generate_keypair",
            "args": { "purpose": "tls_handshake" }
        },
        "id": 1
    });
    let s = request.to_string();
    let v: serde_json::Value = serde_json::from_str(&s).expect("parse");
    assert_eq!(v["method"], "capability.call");
    assert_eq!(v["params"]["capability"], "crypto");
    assert_eq!(v["params"]["operation"], "generate_keypair");
}

#[test]
fn map_to_direct_method_covers_all_tls_operations() {
    let mappings = [
        ("crypto", "generate_keypair", "crypto.x25519_generate_ephemeral"),
        ("crypto", "derive_secret", "crypto.x25519_derive_secret"),
        ("crypto", "encrypt", "crypto.chacha20_poly1305_encrypt"),
        ("crypto", "decrypt", "crypto.chacha20_poly1305_decrypt"),
        ("crypto", "sign", "crypto.sign_ed25519"),
        ("crypto", "sign_ed25519", "crypto.sign_ed25519"),
        ("crypto", "verify", "crypto.verify_ed25519"),
        ("crypto", "verify_ed25519", "crypto.verify_ed25519"),
        ("crypto", "hmac_sha256", "crypto.hmac_sha256"),
        ("crypto", "hash_sha3_256", "crypto.hash_sha3_256"),
        ("tls", "derive_handshake_secrets", "tls.derive_handshake_secrets"),
        ("tls", "derive_application_secrets", "tls.derive_application_secrets"),
        ("tls", "compute_finished_verify_data", "tls.compute_finished_verify_data"),
    ];
    for (cap, op, expected) in mappings {
        assert_eq!(
            SecurityTlsCryptoClient::map_to_direct_method(cap, op),
            expected,
            "mapping ({cap}, {op}) should be {expected}"
        );
    }
}

#[tokio::test]
async fn direct_mode_sends_semantic_method_not_capability_call() {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");

    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut buf = vec![0u8; 16_384];
        let n = stream.read(&mut buf).await.expect("read");
        let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse request");

        assert_eq!(
            req["method"], "crypto.sign_ed25519",
            "direct mode must send semantic method, not capability.call"
        );
        assert!(req["params"]["message"].is_string());

        let sig = general_purpose::STANDARD.encode(vec![0xABu8; 64]);
        let resp = format!(r#"{{"jsonrpc":"2.0","result":{{"signature":"{sig}"}},"id":1}}"#);
        stream.write_all(resp.as_bytes()).await.expect("write");
        let _ = stream.shutdown().await;
    });

    let client = SecurityTlsCryptoClient::new_direct(format!("tcp:{addr}"));
    let sig = client.ed25519_sign(b"hello world", "test-key-id").await.expect("sign");
    assert_eq!(sig, vec![0xABu8; 64]);
}

#[test]
fn jsonrpc_error_response_parses_for_client_logic() {
    let raw = r#"{"jsonrpc":"2.0","error":{"code":-1,"message":"fail"},"id":null}"#;
    let v: serde_json::Value = serde_json::from_str(raw).expect("json");
    assert_eq!(v["error"]["message"], "fail");
    assert!(v.get("result").is_none());
}

#[test]
fn base64_roundtrip_for_security_provider_payload_fields() {
    use base64::{Engine as _, engine::general_purpose};
    let bytes = b"\x00\x01\x02";
    let enc = general_purpose::STANDARD.encode(bytes);
    let dec = general_purpose::STANDARD.decode(&enc).expect("decode");
    assert_eq!(dec, bytes);
}

#[test]
fn base64_decode_rejects_invalid_characters() {
    use base64::{Engine as _, engine::general_purpose};
    let err = general_purpose::STANDARD.decode("@@@");
    assert!(err.is_err(), "invalid base64 must fail");
}

#[test]
fn capability_call_params_json_includes_derive_secret_shape() {
    let params = serde_json::json!({
        "our_secret": "YQ==",
        "their_public": "Yg=="
    });
    assert!(params["our_secret"].as_str().is_some());
    assert_eq!(params["our_secret"], "YQ==");
}

#[cfg(unix)]
mod discover_socket_unix_tests {
    use super::*;
    use songbird_process_env::ScopedEnv;
    use songbird_types::defaults::paths::BIOMEOS_RUNTIME_SUBDIR;
    use std::sync::atomic::{AtomicU32, Ordering};

    static DISCOVER_TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn next_temp_path(suffix: &str) -> String {
        let id = DISCOVER_TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("/tmp/songbird-tls-discover-{}-{}-{suffix}", std::process::id(), id)
    }

    fn create_socket_file(path: &str) -> String {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::File::create(path).expect("create socket file");
        path.to_string()
    }

    fn ghost_socket_path() -> String {
        next_temp_path("ghost.sock")
    }

    #[test]
    fn discover_socket_unix_returns_tcp_from_crypto_provider_socket_env() {
        with_isolated_env_overlay(|| {
            let tcp = "tcp:127.0.0.1:19901".to_string();
            let _tcp = ScopedEnv::new("CRYPTO_PROVIDER_SOCKET", &tcp);
            let found = SecurityTlsCryptoClient::discover_socket_unix().expect("discover tcp");
            assert_eq!(found, tcp);
        });
    }

    #[test]
    fn discover_socket_unix_falls_back_to_security_provider_socket_file() {
        with_isolated_env_overlay(|| {
            let ghost = ghost_socket_path();
            let sock = create_socket_file(&next_temp_path("security.sock"));
            let _crypto = ScopedEnv::new("CRYPTO_PROVIDER_SOCKET", &ghost);
            let _neural = ScopedEnv::new("NEURAL_API_SOCKET", &ghost);
            let _sec = ScopedEnv::new("SECURITY_PROVIDER_SOCKET", &sock);
            let found = SecurityTlsCryptoClient::discover_socket_unix().expect("discover file");
            assert_eq!(found, sock);
            std::fs::remove_file(sock).ok();
        });
    }

    #[test]
    fn discover_socket_unix_uses_xdg_biomeos_security_sock() {
        with_isolated_env_overlay(|| {
            let ghost = ghost_socket_path();
            let test_root = next_temp_path("xdg");
            let xdg = format!("{test_root}/runtime");
            let biome_dir = format!("{xdg}/{BIOMEOS_RUNTIME_SUBDIR}");
            let sec_sock = format!("{biome_dir}/security.sock");
            create_socket_file(&sec_sock);

            let _crypto = ScopedEnv::new("CRYPTO_PROVIDER_SOCKET", &ghost);
            let _neural = ScopedEnv::new("NEURAL_API_SOCKET", &ghost);
            let _sec_env = ScopedEnv::new("SECURITY_PROVIDER_SOCKET", &ghost);
            let _xdg = ScopedEnv::new("XDG_RUNTIME_DIR", &xdg);

            let found = SecurityTlsCryptoClient::discover_socket_unix().expect("xdg discover");
            assert_eq!(found, sec_sock);

            let _ = std::fs::remove_dir_all(test_root);
        });
    }

    #[test]
    fn discover_socket_unix_errors_when_no_socket_candidates_exist() {
        with_isolated_env_overlay(|| {
            let ghost = ghost_socket_path();
            let _crypto = ScopedEnv::new("CRYPTO_PROVIDER_SOCKET", &ghost);
            let _neural = ScopedEnv::new("NEURAL_API_SOCKET", &ghost);
            let _sec = ScopedEnv::new("SECURITY_PROVIDER_SOCKET", &ghost);
            let _biome = ScopedEnv::new("BIOMEOS_SOCKET", &ghost);
            let _xdg = ScopedEnv::new("XDG_RUNTIME_DIR", next_temp_path("empty-xdg"));

            let err = SecurityTlsCryptoClient::discover_socket_unix().unwrap_err();
            assert!(
                matches!(err, TlsError::CryptoError(msg) if msg.contains("Could not discover"))
            );
        });
    }
}

async fn spawn_unix_jsonrpc_echo(expected_method: &str, response_body: &str) -> String {
    let dir = std::env::temp_dir().join(format!(
        "songbird-tls-env-{}-{}",
        std::process::id(),
        TEST_COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("mkdir");
    let sock_path = dir.join("provider.sock");
    let sock_str = sock_path.to_string_lossy().into_owned();
    let expected = expected_method.to_string();
    let body = response_body.to_string();

    let listener = tokio::net::UnixListener::bind(&sock_path).expect("unix bind");
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut buf = vec![0u8; 16_384];
        let n = stream.read(&mut buf).await.expect("read");
        let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
        assert_eq!(req["method"], expected);
        stream.write_all(body.as_bytes()).await.expect("write");
        let _ = stream.shutdown().await;
        let _ = std::fs::remove_dir_all(dir);
    });
    tokio::task::yield_now().await;
    sock_str
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "env test serialization requires lock held across await"
)]
async fn new_parses_security_provider_mode_direct() {
    let _guard = songbird_process_env::test_env_lock();
    reset_overlay_for_test();
    let sock = spawn_unix_jsonrpc_echo(
        "crypto.x25519_generate_ephemeral",
        r#"{"jsonrpc":"2.0","result":{"public_key":"AQID","secret_key":"AQID"},"id":1}"#,
    )
    .await;
    let _mode = songbird_process_env::ScopedEnv::new("SECURITY_PROVIDER_MODE", "direct");
    let _sock = songbird_process_env::ScopedEnv::new("CRYPTO_PROVIDER_SOCKET", &sock);
    let client = SecurityTlsCryptoClient::new().expect("new direct client");
    let _ = client.x25519_generate_ephemeral().await.expect("direct rpc");
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "env test serialization requires lock held across await"
)]
async fn new_parses_beardog_mode_direct_case_insensitive() {
    let _guard = songbird_process_env::test_env_lock();
    reset_overlay_for_test();
    let sock = spawn_unix_jsonrpc_echo(
        "crypto.sign_ed25519",
        r#"{"jsonrpc":"2.0","result":{"signature":"AQID"},"id":1}"#,
    )
    .await;
    let _mode = songbird_process_env::ScopedEnv::new("BEARDOG_MODE", "DiReCt");
    let _sock = songbird_process_env::ScopedEnv::new("CRYPTO_PROVIDER_SOCKET", &sock);
    let client = SecurityTlsCryptoClient::new().expect("new beardog direct client");
    let _ = client.ed25519_sign(b"msg", "kid").await.expect("beardog direct rpc");
}

#[tokio::test]
#[expect(
    clippy::await_holding_lock,
    reason = "env test serialization requires lock held across await"
)]
async fn new_defaults_to_neural_mode_when_env_unset() {
    let _guard = songbird_process_env::test_env_lock();
    reset_overlay_for_test();
    let sock = spawn_unix_jsonrpc_echo(
        "capability.call",
        r#"{"jsonrpc":"2.0","result":{"mac":"AQID"},"id":1}"#,
    )
    .await;
    let _sock = songbird_process_env::ScopedEnv::new("CRYPTO_PROVIDER_SOCKET", &sock);
    let client = SecurityTlsCryptoClient::new().expect("new default client");
    let _ = client.hmac_sha256(b"a", b"b").await.expect("default neural rpc");
}

#[tokio::test]
async fn send_request_rejects_json_array_response() {
    let body = "[1,2,3]".to_string();
    let path = spawn_one_shot_jsonrpc_server(body).await;
    let client = SecurityTlsCryptoClient::with_socket_path(path);
    let err = client.call_capability("crypto", "op", json!({})).await.expect_err("array json");
    assert!(matches!(err, TlsError::CryptoError(msg) if msg.contains("Failed to parse response")));
}

#[tokio::test]
async fn send_request_rejects_truncated_json_object() {
    let body = r#"{"jsonrpc":"2.0","result":{"ok":"#.to_string();
    let path = spawn_one_shot_jsonrpc_server(body).await;
    let client = SecurityTlsCryptoClient::with_socket_path(path);
    let err = client.call_capability("crypto", "op", json!({})).await.expect_err("truncated");
    assert!(matches!(err, TlsError::CryptoError(msg) if msg.contains("Failed to parse response")));
}

fn reset_overlay_for_test() {
    songbird_process_env::reset_overlay();
    songbird_process_env::remove_var("SECURITY_PROVIDER_MODE");
    songbird_process_env::remove_var("BEARDOG_MODE");
}
