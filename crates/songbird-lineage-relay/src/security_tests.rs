// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Unit tests for [`crate::security`] (security-provider lineage relay integration and mocks).

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use std::path::PathBuf;
use std::sync::Arc;

use base64::{Engine, engine::general_purpose};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;

use crate::birdsong::{BirdSongCrypto, LineageHint};
use crate::relay::RelayAuthority;
use crate::security::{
    MockBirdSongCrypto, MockLineageProvider, MockRelayAuthority, SecurityBirdSongProvider,
    SecurityRelayAuthority,
};
use crate::types::{MaskingLevel, NodeId};

fn unique_socket_path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "songbird_security_provider_{}.sock",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before epoch")
            .as_nanos()
    ))
}

async fn drain_request(stream: &mut tokio::net::UnixStream) {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 2048];
    loop {
        let n = stream.read(&mut tmp).await.expect("read request");
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
        if buf.contains(&b'\n') || buf.len() > 65536 {
            break;
        }
    }
}

async fn jsonrpc_accept_write_shutdown(listener: UnixListener, result: serde_json::Value) {
    let (mut stream, _) = listener.accept().await.expect("accept");
    drain_request(&mut stream).await;
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": result
    });
    let mut out = serde_json::to_vec(&body).expect("serialize");
    out.push(b'\n');
    stream.write_all(&out).await.expect("write");
    let _ = stream.shutdown().await;
}

#[tokio::test]
async fn test_security_provider_creation() {
    let provider = SecurityBirdSongProvider::new(
        "/tmp/security-provider.sock",
        Some("test-family".to_string()),
    );

    assert_eq!(provider.test_socket_path().to_str().unwrap(), "/tmp/security-provider.sock");
    assert_eq!(provider.test_family_id(), Some(&"test-family".to_string()));
}

#[tokio::test]
async fn test_mock_lineage_provider() {
    let provider = MockLineageProvider::new();

    provider.add_lineage("child", "parent").await;
    provider.add_lineage("parent", "grandparent").await;

    assert!(provider.is_ancestor("child", "parent").await);
    assert!(provider.is_ancestor("child", "grandparent").await);
    assert!(!provider.is_ancestor("parent", "child").await);

    assert!(provider.is_descendant("parent", "child").await);
    assert!(provider.is_descendant("grandparent", "child").await);
}

#[tokio::test]
async fn test_mock_lineage_provider_default_matches_new() {
    let a = MockLineageProvider::default();
    let b = MockLineageProvider::new();
    assert!(!a.is_ancestor("x", "y").await);
    assert!(!b.is_ancestor("x", "y").await);
}

#[tokio::test]
async fn test_mock_lineage_unknown_node_not_ancestor() {
    let p = MockLineageProvider::new();
    p.add_lineage("only", "root").await;
    assert!(!p.is_ancestor("missing", "root").await);
}

#[tokio::test]
async fn test_mock_lineage_multiple_children_under_parent() {
    let p = MockLineageProvider::new();
    p.add_lineage("c1", "p").await;
    p.add_lineage("c2", "p").await;
    assert!(p.is_ancestor("c1", "p").await);
    assert!(p.is_ancestor("c2", "p").await);
    assert!(p.is_descendant("p", "c1").await);
    assert!(p.is_descendant("p", "c2").await);
}

#[tokio::test]
async fn test_mock_birdsong_crypto() {
    let provider = Arc::new(MockLineageProvider::new());
    provider.add_lineage("child", "parent").await;

    let crypto = MockBirdSongCrypto::new(provider.clone(), "parent".to_string());

    let message = b"test message";
    let encrypted =
        crypto.encrypt_for_lineage(message, LineageHint::DirectAncestors).await.unwrap();

    let decrypted = crypto.decrypt_birdsong(&encrypted, &NodeId::from("child")).await.unwrap();
    assert_eq!(decrypted, Some(message.to_vec()));

    let crypto_unrelated = MockBirdSongCrypto::new(provider, "unrelated".to_string());
    let decrypted_unrelated =
        crypto_unrelated.decrypt_birdsong(&encrypted, &NodeId::from("child")).await.unwrap();
    assert_eq!(decrypted_unrelated, None);
}

#[tokio::test]
async fn test_mock_birdsong_encrypt_accepts_all_lineage_hints() {
    let lp = Arc::new(MockLineageProvider::new());
    let crypto = MockBirdSongCrypto::new(lp, "n".into());
    let hints = [
        LineageHint::DirectParent,
        LineageHint::DirectAncestors,
        LineageHint::DirectChildren,
        LineageHint::AllDescendants,
        LineageHint::SpecificAncestor(NodeId::from("other")),
    ];
    for hint in hints {
        let out = crypto.encrypt_for_lineage(b"payload", hint).await.unwrap();
        assert!(out.starts_with(b"LINEAGE:"));
    }
}

#[tokio::test]
async fn test_mock_relay_authority() {
    let provider = Arc::new(MockLineageProvider::new());
    provider.add_lineage("child", "parent").await;

    let authority = MockRelayAuthority::new(provider);

    let auth =
        authority.authorize_relay(&NodeId::from("parent"), &NodeId::from("child")).await.unwrap();
    assert!(auth.authorized);

    let auth =
        authority.authorize_relay(&NodeId::from("child"), &NodeId::from("parent")).await.unwrap();
    assert!(!auth.authorized);
}

#[tokio::test]
async fn test_mock_relay_determine_masking() {
    let provider = Arc::new(MockLineageProvider::new());
    provider.add_lineage("child", "parent").await;
    let authority = MockRelayAuthority::new(provider);

    let m =
        authority.determine_masking(&NodeId::from("parent"), &NodeId::from("child")).await.unwrap();
    assert_eq!(m, MaskingLevel::Masked);

    let m = authority
        .determine_masking(&NodeId::from("stranger"), &NodeId::from("child"))
        .await
        .unwrap();
    assert_eq!(m, MaskingLevel::FullVisibility);
}

#[test]
fn security_birdsong_provider_constructed_without_panicking() {
    let _p = SecurityBirdSongProvider::new("/tmp/security-provider-unit.sock", Some("fam".into()));
}

#[test]
fn security_relay_authority_with_explicit_path_constructed() {
    let _a = SecurityRelayAuthority::with_socket_path("/tmp/relay-auth.sock");
}

#[test]
fn security_relay_authority_default_constructed() {
    let _a = SecurityRelayAuthority::default();
}

#[tokio::test]
async fn mock_lineage_is_ancestor_chain() {
    let p = MockLineageProvider::new();
    p.add_lineage("c", "b").await;
    p.add_lineage("b", "a").await;
    assert!(p.is_ancestor("c", "a").await);
}

#[tokio::test]
async fn mock_birdsong_rejects_unknown_prefix() {
    let lp = Arc::new(MockLineageProvider::new());
    lp.add_lineage("child", "parent").await;
    let crypto = MockBirdSongCrypto::new(lp, "parent".into());
    let dec = crypto.decrypt_birdsong(b"not-lineage-prefix", &NodeId::from("child")).await.unwrap();
    assert!(dec.is_none());
}

#[tokio::test]
async fn security_authorize_relay_jsonrpc_success_parses_fields() {
    let path = unique_socket_path();
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).expect("bind unix listener");
    let result = serde_json::json!({
        "authorized": true,
        "masking_level": "none",
        "ttl_seconds": 123,
        "audit_token": "unit_test_token"
    });
    let serve = tokio::spawn(jsonrpc_accept_write_shutdown(listener, result));
    tokio::task::yield_now().await;

    let auth = SecurityRelayAuthority::with_socket_path(&path)
        .authorize_relay(&NodeId::from("relay-a"), &NodeId::from("req-b"))
        .await
        .unwrap();

    assert!(auth.authorized);
    assert_eq!(auth.masking_level, MaskingLevel::None);
    assert_eq!(auth.ttl_seconds, 123);
    assert_eq!(auth.audit_token, "unit_test_token");
    assert_eq!(auth.relay_node.0, "relay-a");
    assert_eq!(auth.requester.0, "req-b");

    serve.await.unwrap();
}

#[tokio::test]
async fn security_determine_masking_jsonrpc_parses_each_level() {
    let cases = [
        ("none", MaskingLevel::None),
        ("timing_only", MaskingLevel::TimingOnly),
        ("size_obfuscation", MaskingLevel::SizeObfuscation),
        ("full", MaskingLevel::Full),
        ("masked", MaskingLevel::Masked),
        ("sub_masked", MaskingLevel::SubMasked),
        ("full_visibility", MaskingLevel::FullVisibility),
        ("unknown_label", MaskingLevel::FullVisibility),
    ];

    for (level_str, expected) in cases {
        let path = unique_socket_path();
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).expect("bind");
        let result = serde_json::json!({ "masking_level": level_str });
        let serve = tokio::spawn(jsonrpc_accept_write_shutdown(listener, result));
        tokio::task::yield_now().await;

        let level = SecurityRelayAuthority::with_socket_path(&path)
            .determine_masking(&NodeId::from("r"), &NodeId::from("q"))
            .await
            .unwrap();
        assert_eq!(level, expected, "masking_level string: {level_str}");
        serve.await.unwrap();
    }
}

#[tokio::test]
async fn security_authorize_relay_defaults_when_fields_missing() {
    let path = unique_socket_path();
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).expect("bind");
    let result = serde_json::json!({ "authorized": true });
    let serve = tokio::spawn(jsonrpc_accept_write_shutdown(listener, result));
    tokio::task::yield_now().await;

    let auth = SecurityRelayAuthority::with_socket_path(&path)
        .authorize_relay(&NodeId::from("r"), &NodeId::from("q"))
        .await
        .unwrap();

    assert!(auth.authorized);
    assert_eq!(auth.masking_level, MaskingLevel::FullVisibility);
    assert_eq!(auth.ttl_seconds, 300);
    assert_eq!(auth.audit_token, "security_provider_auth");
    serve.await.unwrap();
}

#[tokio::test]
async fn security_authorize_relay_socket_failure_denies_securely() {
    let auth = SecurityRelayAuthority::with_socket_path(
        "/nonexistent/songbird/security-provider/does_not_exist.sock",
    )
    .authorize_relay(&NodeId::from("r"), &NodeId::from("q"))
    .await
    .unwrap();

    assert!(!auth.authorized);
    assert_eq!(auth.masking_level, MaskingLevel::FullVisibility);
    assert_eq!(auth.ttl_seconds, 0);
    assert_eq!(auth.audit_token, "security_provider_unavailable_deny");
}

#[tokio::test]
async fn security_determine_masking_socket_failure_full_visibility() {
    let level = SecurityRelayAuthority::with_socket_path(
        "/nonexistent/songbird/security-provider/does_not_exist.sock",
    )
    .determine_masking(&NodeId::from("r"), &NodeId::from("q"))
    .await
    .unwrap();

    assert_eq!(level, MaskingLevel::FullVisibility);
}

#[tokio::test]
async fn security_birdsong_encrypt_jsonrpc_roundtrip() {
    let path = unique_socket_path();
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).expect("bind");
    let ciphertext = b"cipher-bytes-from-security-provider";
    let b64 = general_purpose::STANDARD.encode(ciphertext);
    let result = serde_json::json!({ "ciphertext": b64 });
    let serve = tokio::spawn(jsonrpc_accept_write_shutdown(listener, result));
    tokio::task::yield_now().await;

    let provider = SecurityBirdSongProvider::new(&path, Some("fam-1".into()));
    let out = provider.encrypt_for_lineage(b"plain", LineageHint::DirectAncestors).await.unwrap();
    assert_eq!(out, ciphertext);
    serve.await.unwrap();
}

#[tokio::test]
async fn security_birdsong_encrypt_accepts_v1_encrypted_field_name() {
    let path = unique_socket_path();
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).expect("bind");
    let ciphertext = b"v1-field";
    let b64 = general_purpose::STANDARD.encode(ciphertext);
    let result = serde_json::json!({ "encrypted": b64 });
    let serve = tokio::spawn(jsonrpc_accept_write_shutdown(listener, result));
    tokio::task::yield_now().await;

    let provider = SecurityBirdSongProvider::new(&path, None);
    let out = provider.encrypt_for_lineage(b"x", LineageHint::DirectParent).await.unwrap();
    assert_eq!(out, ciphertext);
    serve.await.unwrap();
}

#[tokio::test]
async fn security_birdsong_decrypt_jsonrpc_success() {
    let path = unique_socket_path();
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).expect("bind");
    let plaintext = b"recovered";
    let result = serde_json::json!({
        "success": true,
        "plaintext": general_purpose::STANDARD.encode(plaintext)
    });
    let serve = tokio::spawn(jsonrpc_accept_write_shutdown(listener, result));
    tokio::task::yield_now().await;

    let provider = SecurityBirdSongProvider::new(&path, None);
    let out = provider.decrypt_birdsong(b"cipher", &NodeId::from("sender")).await.unwrap();
    assert_eq!(out, Some(plaintext.to_vec()));
    serve.await.unwrap();
}

#[tokio::test]
async fn security_birdsong_decrypt_success_false_returns_none() {
    let path = unique_socket_path();
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).expect("bind");
    let result = serde_json::json!({
        "success": false,
        "plaintext": general_purpose::STANDARD.encode(b"ignored")
    });
    let serve = tokio::spawn(jsonrpc_accept_write_shutdown(listener, result));
    tokio::task::yield_now().await;

    let provider = SecurityBirdSongProvider::new(&path, None);
    let out = provider.decrypt_birdsong(b"cipher", &NodeId::from("sender")).await.unwrap();
    assert!(out.is_none());
    serve.await.unwrap();
}

#[tokio::test]
async fn security_birdsong_decrypt_jsonrpc_error_returns_none() {
    let path = unique_socket_path();
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).expect("bind");
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": { "code": -32000, "message": "no decrypt" }
    });
    let mut out = serde_json::to_vec(&body).unwrap();
    out.push(b'\n');
    let serve = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        drain_request(&mut stream).await;
        stream.write_all(&out).await.unwrap();
        let _ = stream.shutdown().await;
    });
    tokio::task::yield_now().await;

    let provider = SecurityBirdSongProvider::new(&path, None);
    let out = provider.decrypt_birdsong(b"cipher", &NodeId::from("sender")).await.unwrap();
    assert!(out.is_none());
    serve.await.unwrap();
}

#[tokio::test]
async fn security_birdsong_decrypt_missing_plaintext_errors() {
    let path = unique_socket_path();
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).expect("bind");
    let result = serde_json::json!({ "success": true });
    let serve = tokio::spawn(jsonrpc_accept_write_shutdown(listener, result));
    tokio::task::yield_now().await;

    let provider = SecurityBirdSongProvider::new(&path, None);
    let err = provider.decrypt_birdsong(b"cipher", &NodeId::from("sender")).await;
    assert!(err.is_err());
    serve.await.unwrap();
}

#[tokio::test]
async fn security_authorize_relay_jsonrpc_authorized_false() {
    let path = unique_socket_path();
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).expect("bind unix listener");
    let result = serde_json::json!({
        "authorized": false,
        "masking_level": "masked",
        "ttl_seconds": 99,
        "audit_token": "denied_token"
    });
    let serve = tokio::spawn(jsonrpc_accept_write_shutdown(listener, result));
    tokio::task::yield_now().await;

    let auth = SecurityRelayAuthority::with_socket_path(&path)
        .authorize_relay(&NodeId::from("relay-x"), &NodeId::from("req-y"))
        .await
        .unwrap();

    assert!(!auth.authorized);
    assert_eq!(auth.masking_level, MaskingLevel::Masked);
    assert_eq!(auth.ttl_seconds, 99);
    assert_eq!(auth.audit_token, "denied_token");
    serve.await.unwrap();
}

#[tokio::test]
async fn security_birdsong_encrypt_missing_ciphertext_errors() {
    let path = unique_socket_path();
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).expect("bind");
    let result = serde_json::json!({});
    let serve = tokio::spawn(jsonrpc_accept_write_shutdown(listener, result));
    tokio::task::yield_now().await;

    let provider = SecurityBirdSongProvider::new(&path, None);
    let err = provider.encrypt_for_lineage(b"plain", LineageHint::DirectParent).await;
    assert!(err.is_err());
    serve.await.unwrap();
}
