// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

#![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::unused_async)]

use crate::keys::{EphemeralKeypair, OnionIdentity, SessionKeys};

mod stored_identity_tests {
    use crate::keys::OnionIdentity;
    use serde_json::json;

    #[test]
    fn to_from_stored_bytes_v2_roundtrip_without_local_ed25519() {
        let j = json!({
            "secret_key_bytes": vec![9u8; 32],
            "public_key_bytes": vec![8u8; 32],
            "onion_address": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.onion",
            "created_at": 42u64
        });
        let bytes = serde_json::to_vec(&j).expect("serialize fixture");
        let id = OnionIdentity::from_stored_bytes(&bytes).expect("parse v2 stored identity");
        assert_eq!(id.created_at(), 42, "created_at");
        assert_eq!(
            id.onion_address(),
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.onion",
            "onion address"
        );
        assert_eq!(id.public_key_bytes(), &[8u8; 32], "public key bytes");
        let round = id.to_stored_bytes().expect("serialize stored identity");
        let id2 = OnionIdentity::from_stored_bytes(&round).expect("second parse");
        assert_eq!(id2.secret_key_bytes(), id.secret_key_bytes(), "secret roundtrip");
    }

    #[test]
    fn from_stored_bytes_rejects_invalid_json() {
        let r = OnionIdentity::from_stored_bytes(b"{not json");
        assert!(r.is_err(), "expected serde error, got {r:?}");
    }

    /// Production builds without `standalone` cannot reconstruct v1 blobs (secret-only).
    #[cfg(not(feature = "standalone"))]
    #[test]
    fn from_stored_bytes_v1_legacy_returns_crypto_error() {
        let j = json!({
            "secret_key_bytes": vec![3u8; 32],
            "created_at": 0u64
        });
        let bytes = serde_json::to_vec(&j).expect("serialize v1 fixture");
        let r = OnionIdentity::from_stored_bytes(&bytes);
        assert!(
            matches!(r, Err(crate::error::OnionError::CryptoError(ref s)) if s.contains("Legacy")),
            "expected legacy storage error, got {r:?}"
        );
    }

    #[test]
    fn to_stored_bytes_includes_all_v2_fields() {
        let j = json!({
            "secret_key_bytes": vec![1u8; 32],
            "public_key_bytes": vec![2u8; 32],
            "onion_address": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.onion",
            "created_at": 99u64
        });
        let id = OnionIdentity::from_stored_bytes(&serde_json::to_vec(&j).expect("serialize"))
            .expect("load v2");
        let stored = id.to_stored_bytes().expect("serialize stored");
        let parsed: serde_json::Value = serde_json::from_slice(&stored).expect("parse json");
        assert_eq!(parsed["created_at"], 99);
        assert!(parsed["public_key_bytes"].is_array());
        assert_eq!(
            parsed["onion_address"],
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.onion"
        );
    }

    #[test]
    fn from_stored_bytes_rejects_truncated_secret_key_array() {
        let j = json!({
            "secret_key_bytes": vec![1u8; 16],
            "public_key_bytes": vec![2u8; 32],
            "onion_address": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccc.onion",
            "created_at": 0u64
        });
        let bytes = serde_json::to_vec(&j).expect("serialize");
        let r = OnionIdentity::from_stored_bytes(&bytes);
        assert!(r.is_err(), "secret key must be 32 bytes, got {r:?}");
    }
}

mod keys_security_provider_tests {
    use super::*;
    use crate::security_crypto::SecurityCryptoClient;
    use base64::{Engine, engine::general_purpose::STANDARD};
    use serde_json::json;
    use songbird_crypto_provider::{CryptoProvider, RoutingMode};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    fn b64(data: &[u8]) -> String {
        STANDARD.encode(data)
    }

    async fn read_json_rpc_request(stream: &mut tokio::net::UnixStream) -> serde_json::Value {
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await.expect("read request");
        serde_json::from_slice(&buf).expect("parse JSON-RPC request")
    }

    async fn start_identity_mock_server() -> String {
        let path = std::env::temp_dir().join(format!(
            "songbird-onion-keys-{}-{}.sock",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        let path_str = path.to_string_lossy().to_string();
        let listener = UnixListener::bind(&path_str).expect("bind mock socket");

        tokio::spawn(async move {
            while let Ok((mut stream, _)) = listener.accept().await {
                let req = read_json_rpc_request(&mut stream).await;
                let method = req["method"].as_str().unwrap_or("");
                let params = &req["params"];
                let id = req["id"].as_u64().unwrap_or(1);
                let result = match method {
                    "crypto.ed25519_generate_keypair" => json!({
                        "public_key": b64(&[0xAAu8; 32]),
                        "secret_key": b64(&[0xBBu8; 32]),
                    }),
                    "crypto.ed25519_public_from_secret" => json!({
                        "public_key": b64(&[0xCCu8; 32]),
                    }),
                    "crypto.sha3_256" => json!({
                        "hash_base64": b64(&[0xDDu8; 32]),
                    }),
                    "crypto.x25519_generate_ephemeral" => json!({
                        "public_key": b64(&[0xEEu8; 32]),
                        "secret_key": b64(&[0xFFu8; 32]),
                    }),
                    "crypto.x25519_derive_secret" => json!({
                        "shared_secret": b64(&[0x11u8; 32]),
                    }),
                    "crypto.hmac_sha256" => {
                        let data = params
                            .get("data")
                            .and_then(|v| v.as_str())
                            .and_then(|s| STANDARD.decode(s).ok())
                            .unwrap_or_default();
                        let mac = if data.starts_with(b"sovereign-onion client") {
                            b64(&[0x33u8; 32])
                        } else if data.starts_with(b"sovereign-onion server") {
                            b64(&[0x44u8; 32])
                        } else {
                            b64(&[0x22u8; 32])
                        };
                        json!({ "mac": mac })
                    }
                    _ => json!({}),
                };
                let body = json!({"jsonrpc":"2.0","result":result,"id":id}).to_string();
                let _ = stream.write_all(body.as_bytes()).await;
            }
        });

        path_str
    }

    fn mock_client(path: &str) -> SecurityCryptoClient {
        SecurityCryptoClient::from_provider(CryptoProvider::with_mode(path, RoutingMode::Direct))
    }

    #[tokio::test(start_paused = true)]
    async fn generate_via_security_provider_returns_identity_with_onion_suffix() {
        let path = start_identity_mock_server().await;
        let client = mock_client(&path);
        let id = OnionIdentity::generate_via_security_provider(&client)
            .await
            .expect("generate identity");
        assert!(id.onion_address().ends_with(".onion"));
        assert_eq!(id.public_key_bytes(), &[0xAA; 32]);
        assert_eq!(id.secret_key_bytes(), &[0xBB; 32]);
    }

    #[tokio::test(start_paused = true)]
    async fn from_stored_via_security_provider_derives_pubkey_and_address() {
        let path = start_identity_mock_server().await;
        let client = mock_client(&path);
        let fixture = json!({
            "secret_key_bytes": vec![5u8; 32],
            "created_at": 7u64
        });
        let bytes = serde_json::to_vec(&fixture).expect("serialize fixture");
        let id = OnionIdentity::from_stored_via_security_provider(&client, &bytes)
            .await
            .expect("load via provider");
        assert_eq!(id.public_key_bytes(), &[0xCC; 32]);
        assert!(id.onion_address().ends_with(".onion"));
        assert_eq!(id.created_at(), 7);
    }

    #[tokio::test(start_paused = true)]
    async fn ephemeral_keypair_generate_and_derive_shared_secret() {
        let path = start_identity_mock_server().await;
        let client = mock_client(&path);
        let kp = EphemeralKeypair::generate_via_security_provider(&client)
            .await
            .expect("generate ephemeral");
        assert_eq!(kp.public_bytes(), &[0xEE; 32]);
        let peer = [0x99u8; 32];
        let secret = kp
            .derive_shared_secret_via_security_provider(&client, &peer)
            .await
            .expect("derive shared secret");
        assert_eq!(secret, [0x11; 32]);
    }

    #[tokio::test(start_paused = true)]
    async fn session_keys_derive_assigns_client_and_server_roles() {
        let path = start_identity_mock_server().await;
        let client = mock_client(&path);
        let shared = [0x33u8; 32];
        let client_nonce = [0x01u8; 24];
        let server_nonce = [0x02u8; 24];

        let client_keys = SessionKeys::derive_via_security_provider(
            &client,
            &shared,
            &client_nonce,
            &server_nonce,
            true,
        )
        .await
        .expect("client keys");
        let server_keys = SessionKeys::derive_via_security_provider(
            &client,
            &shared,
            &client_nonce,
            &server_nonce,
            false,
        )
        .await
        .expect("server keys");

        assert_eq!(client_keys.send_key, server_keys.recv_key);
        assert_eq!(client_keys.recv_key, server_keys.send_key);
        assert_ne!(client_keys.send_key, client_keys.recv_key);
    }
}
