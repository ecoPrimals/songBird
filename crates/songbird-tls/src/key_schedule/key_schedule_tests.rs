// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, reason = "test assertions")]

use super::KeySchedule;
use crate::crypto::SecurityTlsCryptoClient;
use crate::error::TlsError;
use hmac::Hmac;
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn ref_hmac_sha256(message: &[u8], key: &[u8]) -> Vec<u8> {
    use hmac::Mac;
    let mut mac = HmacSha256::new_from_slice(key).expect("key length");
    mac.update(message);
    mac.finalize().into_bytes().to_vec()
}

fn schedule_with_hmac() -> KeySchedule {
    let mut ks = KeySchedule::new();
    ks.set_test_hmac(ref_hmac_sha256);
    ks
}

#[test]
fn test_new_key_schedule() {
    let ks = KeySchedule::new();
    assert_eq!(ks.current_secret.len(), 32);
    assert!(ks.transcript_hash.is_empty());
    assert!(ks.crypto_client.is_none());
}

#[test]
fn test_update_transcript() {
    let mut ks = KeySchedule::new();
    ks.update_transcript(b"Hello");
    ks.update_transcript(b" World");
    assert_eq!(ks.transcript_hash(), b"Hello World");
}

#[test]
fn test_derive_secret_label_format() {
    let ks = KeySchedule::new();
    assert!(ks.crypto_client.is_none());
}

#[test]
fn test_key_schedule_initialization() {
    let ks = KeySchedule::new();
    assert_eq!(ks.current_secret.len(), 32);
    assert!(ks.current_secret.iter().all(|&b| b == 0));
    assert!(ks.transcript_hash().is_empty());
    assert!(ks.crypto_client.is_none());
    assert!(ks.server_secret_key().is_none());
}

#[test]
fn test_set_crypto_client() {
    let mut ks = KeySchedule::new();
    let client = SecurityTlsCryptoClient::with_socket_path("/tmp/test-ks.sock".to_string());
    ks.set_crypto_client(client);
    assert!(ks.crypto_client.is_some());
}

#[test]
fn test_set_and_get_server_secret_key() {
    let mut ks = KeySchedule::new();
    let secret_key = vec![42u8; 32];
    ks.set_server_secret_key(secret_key.clone());
    assert_eq!(ks.server_secret_key(), Some(secret_key.as_slice()));
}

#[test]
fn test_update_transcript_multiple_times() {
    let mut ks = KeySchedule::new();
    ks.update_transcript(b"ClientHello");
    ks.update_transcript(b"ServerHello");
    ks.update_transcript(b"Certificate");
    assert_eq!(ks.transcript_hash(), b"ClientHelloServerHelloCertificate");
}

#[test]
fn test_transcript_hash_empty_initially() {
    let ks = KeySchedule::new();
    assert!(ks.transcript_hash().is_empty());
    assert_eq!(ks.transcript_hash().len(), 0);
}

#[test]
fn test_transcript_hash_preserves_data() {
    let mut ks = KeySchedule::new();
    let data = b"Test Data 12345";
    ks.update_transcript(data);
    assert_eq!(ks.transcript_hash(), data);
}

#[test]
fn test_default_trait() {
    let ks = KeySchedule::default();
    assert_eq!(ks.current_secret.len(), 32);
    assert!(ks.transcript_hash().is_empty());
    assert!(ks.crypto_client.is_none());
}

#[test]
fn test_current_secret_initial_state() {
    let ks = KeySchedule::new();
    assert_eq!(ks.current_secret, vec![0u8; 32]);
}

#[tokio::test]
async fn test_hkdf_extract_requires_crypto_client() {
    let ks = KeySchedule::new();
    let result = ks.hkdf_extract(b"salt", b"ikm").await;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(format!("{e:?}").contains("Crypto client not set"));
    }
}

#[tokio::test]
async fn test_hkdf_expand_requires_crypto_client() {
    let ks = KeySchedule::new();
    let result = ks.hkdf_expand(&[1u8; 32], b"info", 32).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_derive_secret_requires_crypto_client() {
    let ks = KeySchedule::new();
    let result = ks.derive_secret(&[1u8; 32], "label", b"context").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_compute_handshake_secret_requires_crypto_client() {
    let mut ks = KeySchedule::new();
    let result = ks.compute_handshake_secret(&[2u8; 32]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_derive_handshake_traffic_keys_requires_crypto_client() {
    let ks = KeySchedule::new();
    let result = ks.derive_handshake_traffic_keys().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_compute_master_secret_requires_crypto_client() {
    let mut ks = KeySchedule::new();
    let result = ks.compute_master_secret().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_derive_application_traffic_keys_requires_crypto_client() {
    let ks = KeySchedule::new();
    let result = ks.derive_application_traffic_keys().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_derive_traffic_keys_requires_crypto_client() {
    let ks = KeySchedule::new();
    let result = ks.derive_traffic_keys(&[3u8; 32]).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_compute_finished_verify_data_requires_crypto_client() {
    let ks = KeySchedule::new();
    let result = ks.compute_finished_verify_data(&[4u8; 32]).await;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(format!("{e:?}").contains("Crypto client not set"));
    }
}

#[test]
fn test_server_secret_key_none_initially() {
    let ks = KeySchedule::new();
    assert!(ks.server_secret_key().is_none());
}

#[test]
fn test_multiple_transcript_updates_order() {
    let mut ks = KeySchedule::new();
    ks.update_transcript(b"First");
    ks.update_transcript(b"Second");
    ks.update_transcript(b"Third");
    assert_eq!(ks.transcript_hash(), b"FirstSecondThird");
}

#[tokio::test]
async fn hkdf_extract_matches_reference_hmac() {
    let ks = schedule_with_hmac();
    let salt = b"salt";
    let ikm = b"ikm";
    let got = ks.hkdf_extract(salt, ikm).await.expect("extract");
    assert_eq!(got, ref_hmac_sha256(ikm, salt));
}

#[tokio::test]
async fn hkdf_expand_zero_length() {
    let ks = schedule_with_hmac();
    let out = ks.hkdf_expand(&[7u8; 32], b"info", 0).await.expect("expand");
    assert!(out.is_empty());
}

#[tokio::test]
async fn hkdf_expand_matches_manual_reference() {
    let prk = [9u8; 32];
    let info = b"tls13 test";
    let ks = schedule_with_hmac();
    let got = ks.hkdf_expand(&prk, info, 48).await.expect("expand");

    let mut t = Vec::new();
    let mut out = Vec::new();
    let mut counter = 1u8;
    while out.len() < 48 {
        let mut input = Vec::new();
        input.extend_from_slice(&t);
        input.extend_from_slice(info);
        input.push(counter);
        t = ref_hmac_sha256(&input, &prk);
        out.extend_from_slice(&t);
        counter += 1;
    }
    out.truncate(48);
    assert_eq!(got, out);
}

#[tokio::test]
async fn hkdf_expand_iteration_limit_returns_error() {
    let ks = schedule_with_hmac();
    let err = ks.hkdf_expand(&[1u8; 32], b"x", 8161).await.expect_err("too many iterations");
    match err {
        TlsError::InternalError(msg) => assert!(msg.contains("too many iterations")),
        e => panic!("unexpected {e:?}"),
    }
}

#[tokio::test]
async fn derive_secret_context_max_length_ok() {
    let ks = schedule_with_hmac();
    let ctx = vec![0xabu8; 255];
    let out = ks.derive_secret(&[5u8; 32], "x", &ctx).await.expect("derive");
    assert_eq!(out.len(), 32);
}

#[tokio::test]
async fn derive_secret_context_too_long() {
    let ks = schedule_with_hmac();
    let ctx = vec![0u8; 256];
    let err = ks.derive_secret(&[5u8; 32], "x", &ctx).await.expect_err("context");
    match err {
        TlsError::InternalError(msg) => assert!(msg.contains("Context too long")),
        e => panic!("unexpected {e:?}"),
    }
}

#[tokio::test]
async fn derive_secret_label_too_long() {
    let ks = schedule_with_hmac();
    let label = "x".repeat(250);
    let err = ks.derive_secret(&[5u8; 32], &label, &[]).await.expect_err("label");
    match err {
        TlsError::InternalError(msg) => assert!(msg.contains("Label too long")),
        e => panic!("unexpected {e:?}"),
    }
}

#[tokio::test]
async fn derive_secret_changes_with_transcript_context() {
    let mut ks = schedule_with_hmac();
    ks.update_transcript(b"hello");
    let a = ks
        .derive_secret(&ks.current_secret, "c hs traffic", ks.transcript_hash())
        .await
        .expect("a");
    let mut ks2 = schedule_with_hmac();
    ks2.update_transcript(b"world");
    let b = ks2
        .derive_secret(&ks2.current_secret, "c hs traffic", ks2.transcript_hash())
        .await
        .expect("b");
    assert_ne!(a, b);
}

#[tokio::test]
async fn compute_handshake_secret_updates_current_secret() {
    let mut ks = schedule_with_hmac();
    let before = ks.current_secret.clone();
    ks.compute_handshake_secret(&[0xee; 32]).await.expect("hs");
    assert_ne!(ks.current_secret, before);
    assert_eq!(ks.current_secret.len(), 32);
}

#[tokio::test]
async fn full_schedule_flow_deterministic() {
    let mut ks = schedule_with_hmac();
    ks.update_transcript(b"synthetic transcript");

    ks.compute_handshake_secret(&[0x11; 32]).await.expect("hs");
    let (c_hs, s_hs) = ks.derive_handshake_traffic_keys().await.expect("hs traffic");
    assert_ne!(c_hs, s_hs);

    ks.compute_master_secret().await.expect("master");
    let (c_ap, s_ap) = ks.derive_application_traffic_keys().await.expect("ap traffic");
    assert_ne!(c_ap, s_ap);
    assert_ne!(c_ap, c_hs);

    let (key, iv) = ks.derive_traffic_keys(&c_ap).await.expect("keys");
    assert_eq!(key.len(), 32);
    assert_eq!(iv.len(), 12);
}

#[tokio::test]
async fn derive_handshake_traffic_secrets_are_distinct_from_master_branch() {
    let mut ks = schedule_with_hmac();
    ks.update_transcript(b"t");
    ks.compute_handshake_secret(&[3u8; 32]).await.expect("hs");
    let (c_hs, s_hs) = ks.derive_handshake_traffic_keys().await.expect("derive");
    ks.compute_master_secret().await.expect("ms");
    let (c_ap, s_ap) = ks.derive_application_traffic_keys().await.expect("ap");
    assert_ne!(c_hs, c_ap);
    assert_ne!(s_hs, s_ap);
}

#[tokio::test]
async fn compute_finished_verify_data_matches_reference() {
    let mut ks = schedule_with_hmac();
    ks.update_transcript(b"handshake bytes");
    let base = [0x55u8; 32];
    let vd = ks.compute_finished_verify_data(&base).await.expect("finished");
    let finished_key = ks.derive_secret(&base, "finished", &[]).await.expect("fk");
    assert_eq!(vd, ref_hmac_sha256(ks.transcript_hash(), &finished_key));
}

#[tokio::test]
async fn test_hmac_prefers_test_hook_over_security_client() {
    let mut ks = KeySchedule::new();
    ks.set_crypto_client(SecurityTlsCryptoClient::with_socket_path(
        "/nonexistent/security-provider.sock".to_string(),
    ));
    ks.set_test_hmac(ref_hmac_sha256);
    let out = ks.hkdf_extract(b"a", b"b").await.expect("uses test hmac");
    assert_eq!(out, ref_hmac_sha256(b"b", b"a"));
}

#[tokio::test]
async fn hkdf_expand_single_byte_output_truncates_hmac_block() {
    let ks = schedule_with_hmac();
    let prk = [0xabu8; 32];
    let out = ks.hkdf_expand(&prk, b"lbl", 1).await.expect("one byte");
    assert_eq!(out.len(), 1);
    let full = ks.hkdf_expand(&prk, b"lbl", 32).await.expect("full block");
    assert_eq!(out[0], full[0]);
}

#[tokio::test]
async fn derive_secret_empty_label_is_valid() {
    let ks = schedule_with_hmac();
    let out = ks.derive_secret(&[1u8; 32], "", &[]).await.expect("empty label");
    assert_eq!(out.len(), 32);
}

#[tokio::test]
async fn compute_handshake_secret_accepts_empty_ecdhe_material() {
    let mut ks = schedule_with_hmac();
    ks.compute_handshake_secret(&[]).await.expect("empty IKM is still a valid HKDF-Extract input");
    assert_eq!(ks.current_secret.len(), 32);
}

#[tokio::test]
async fn derive_traffic_keys_produces_12_byte_iv_for_chacha() {
    let ks = schedule_with_hmac();
    let ts = [0x33u8; 32];
    let (key, iv) = ks.derive_traffic_keys(&ts).await.expect("traffic keys");
    assert_eq!(key.len(), 32);
    assert_eq!(iv.len(), 12);
}

#[tokio::test]
async fn transcript_hash_isolation_between_schedule_instances() {
    let mut a = schedule_with_hmac();
    let mut b = schedule_with_hmac();
    a.update_transcript(b"alpha");
    b.update_transcript(b"beta");
    let sa = a.derive_secret(&[1u8; 32], "x", a.transcript_hash()).await.expect("a");
    let sb = b.derive_secret(&[1u8; 32], "x", b.transcript_hash()).await.expect("b");
    assert_ne!(sa, sb);
}

#[tokio::test]
async fn derive_handshake_traffic_keys_change_when_transcript_updates_mid_flow() {
    let mut ks = schedule_with_hmac();
    ks.update_transcript(b"first");
    ks.compute_handshake_secret(&[9u8; 32]).await.expect("hs");
    let before = ks.derive_handshake_traffic_keys().await.expect("before");
    ks.update_transcript(b"second");
    let after = ks.derive_handshake_traffic_keys().await.expect("after");
    assert_ne!(before.0, after.0);
}

#[tokio::test]
async fn hkdf_expand_supports_output_len_8128_without_hitting_u8_counter_limit() {
    let ks = schedule_with_hmac();
    let out = ks.hkdf_expand(&[2u8; 32], b"i", 8128).await.expect("254 full HMAC blocks");
    assert_eq!(out.len(), 8128);
}
