// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Framing and serialization for anonymous discovery UDP broadcasts (v2.1 / v3.0, `BirdSong`, Dark Forest).

use tracing::{debug, error, info, warn};

use super::messages::{AnonymousDiscoveryMessage, TransportEndpointMessage};

/// Default HTTPS port for v2.1 fallback when the primary v3.0 endpoint omits `:port`.
///
/// Set `SONGBIRD_DEFAULT_DISCOVERY_PORT` to override; otherwise
/// [`songbird_types::constants::DEFAULT_HTTP_PORT`] (8080).
#[must_use]
pub fn default_v3_fallback_port() -> u16 {
    songbird_process_env::var("SONGBIRD_DEFAULT_DISCOVERY_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(songbird_types::constants::DEFAULT_HTTP_PORT)
}

/// Plain discovery payload after JSON serialization (before optional `BirdSong`).
pub struct PreparedDiscoveryPlaintext {
    /// For logging (`debug!` after send).
    pub session_id: String,
    pub bytes: Vec<u8>,
}

/// Build and serialize the anonymous discovery message (v2.1 or v3.0) including tags and attestations.
#[expect(clippy::too_many_arguments, reason = "discovery protocol has many parameters by design")]
pub fn build_discovery_plaintext(
    version: &str,
    node_id: Option<String>,
    node_name: Option<String>,
    endpoints: Option<Vec<TransportEndpointMessage>>,
    capabilities: Vec<String>,
    protocols: Vec<String>,
    port: u16,
    identity_attestations: Option<Vec<crate::IdentityAttestation>>,
    tags: Option<Vec<String>>,
) -> Result<PreparedDiscoveryPlaintext, serde_json::Error> {
    let mut message = if version == "3.0" {
        AnonymousDiscoveryMessage::new_v3(
            node_id.unwrap_or_default(),
            node_name.unwrap_or_default(),
            endpoints.unwrap_or_default(),
            capabilities,
        )
    } else {
        AnonymousDiscoveryMessage::new(capabilities, protocols, port)
    };

    if let Some(ref attestations) = identity_attestations {
        message = message.with_identity_attestations(attestations.clone());
    }

    if let Some(ref tag_list) = tags {
        debug!("📋 Broadcasting {} identity tags: {:?}", tag_list.len(), tag_list);
        message = message.with_tags(tag_list.clone());
    } else {
        debug!("📋 No identity tags to broadcast");
    }

    let session_id = message.session_id.clone();
    let bytes = message.to_bytes()?;
    Ok(PreparedDiscoveryPlaintext {
        session_id,
        bytes,
    })
}

/// Legacy `BirdSong` encryption (may include plaintext family header).
pub async fn encrypt_birdsong_legacy(
    plain: &[u8],
    birdsong: &crate::birdsong::BirdSongProcessor,
) -> Vec<u8> {
    match birdsong.encrypt_packet(plain).await {
        Ok(encrypted) => {
            debug!("🔒 BirdSong encrypted (legacy): {} -> {} bytes", plain.len(), encrypted.len());
            encrypted
        }
        Err(e) => {
            warn!("⚠️  BirdSong encryption failed: {}, using plaintext", e);
            plain.to_vec()
        }
    }
}

/// Legacy encrypt after Dark Forest build failure (distinct log messages).
pub async fn encrypt_birdsong_after_dark_forest_failure(
    plain: &[u8],
    birdsong: &crate::birdsong::BirdSongProcessor,
) -> Vec<u8> {
    match birdsong.encrypt_packet(plain).await {
        Ok(encrypted) => {
            debug!(
                "🔒 BirdSong encrypted (legacy fallback): {} -> {} bytes",
                plain.len(),
                encrypted.len()
            );
            encrypted
        }
        Err(e) => {
            warn!("⚠️  Legacy encryption also failed: {}, using plaintext", e);
            plain.to_vec()
        }
    }
}

/// Dual-broadcast second packet: legacy `BirdSong` over the serialized discovery JSON.
pub async fn encrypt_birdsong_dual_legacy(
    plain: &[u8],
    birdsong: &crate::birdsong::BirdSongProcessor,
) -> Vec<u8> {
    match birdsong.encrypt_packet(plain).await {
        Ok(encrypted) => encrypted,
        Err(e) => {
            warn!("⚠️  Legacy encryption failed: {}", e);
            plain.to_vec()
        }
    }
}

/// 16-byte beacon ID when the encryption provider has not supplied one yet.
///
/// Uses the first 16 bytes of SHA-256(`node_id`) when v3.0 identity is configured;
/// otherwise fills 16 bytes from [`rand::thread_rng`] (v2.1 / anonymous mode).
pub async fn dark_forest_beacon_id_fallback(
    node_id: Option<&str>,
    crypto: Option<&songbird_crypto_provider::CryptoProvider>,
) -> Vec<u8> {
    use rand::Rng;

    if let Some(nid) = node_id {
        let h = crate::crypto_helpers::sha256_hash(crypto, nid.as_bytes()).await;
        h[..16].to_vec()
    } else {
        let mut id = [0u8; 16];
        rand::thread_rng().fill(&mut id[..]);
        id.to_vec()
    }
}

/// Build encrypted Dark Forest beacon bytes (no UDP I/O).
pub async fn build_dark_forest_beacon_bytes(
    node_id: Option<String>,
    endpoints: Option<&Vec<TransportEndpointMessage>>,
    port: u16,
    capabilities: &[String],
    birdsong: &crate::birdsong::BirdSongProcessor,
) -> Result<Vec<u8>, anyhow::Error> {
    use crate::dark_forest_beacon::BeaconPayload;

    let crypto = songbird_crypto_provider::CryptoProvider::from_env();
    let crypto_ref = Some(&crypto);

    let beacon_id = match birdsong.encryption_provider() {
        Some(enc) if enc.is_available() => match enc.get_beacon_id().await? {
            Some(id) => id,
            None => dark_forest_beacon_id_fallback(node_id.as_deref(), crypto_ref).await,
        },
        _ => dark_forest_beacon_id_fallback(node_id.as_deref(), crypto_ref).await,
    };

    let endpoints_list: Vec<String> = endpoints.map_or_else(
        || vec![format!("tcp:0.0.0.0:{port}")],
        |eps| eps.iter().map(|e| format!("{}:{}", e.interface_type, e.address)).collect(),
    );

    let payload = BeaconPayload::new(
        beacon_id,
        node_id.unwrap_or_else(|| "unknown".to_string()),
        endpoints_list,
        capabilities,
        None,
        super::scheduling::rotating_session_id(),
    );

    let beacon = birdsong
        .encrypt_dark_forest_beacon(&payload)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to encrypt Dark Forest beacon: {e}"))?;

    beacon.to_bytes().map_err(|e| anyhow::anyhow!("Failed to serialize Dark Forest beacon: {e}"))
}

/// Log successful Dark Forest beacon size (metadata-free broadcast).
pub fn log_dark_forest_beacon_sent(beacon_len: usize) {
    info!("🌲 Broadcasted Dark Forest beacon (size: {} bytes, NO metadata leakage)", beacon_len);
}

/// Log serialization failure for discovery JSON.
pub fn log_serialize_error(e: &serde_json::Error) {
    error!("Failed to serialize discovery message: {}", e);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anonymous::messages::{AnonymousDiscoveryMessage, TransportEndpointMessage};
    use crate::birdsong::{BirdSongConfig, BirdSongEncryption, BirdSongProcessor};
    use anyhow::Result;
    use async_trait::async_trait;
    use std::sync::Arc;

    struct MockEnc {
        family_id: String,
    }

    #[async_trait]
    impl BirdSongEncryption for MockEnc {
        async fn encrypt_discovery(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
            Ok(plaintext.to_vec())
        }

        async fn decrypt_discovery(&self, _ciphertext: &[u8]) -> Result<Option<Vec<u8>>> {
            Ok(None)
        }

        fn is_available(&self) -> bool {
            true
        }

        fn family_id(&self) -> Option<String> {
            Some(self.family_id.clone())
        }

        fn provider_name(&self) -> String {
            "mock-protocol-test".to_string()
        }
    }

    #[tokio::test]
    async fn default_v3_fallback_port_env_override_and_default() {
        {
            let _e =
                songbird_test_utils::ScopedEnv::remove("SONGBIRD_DEFAULT_DISCOVERY_PORT").await;
            assert_eq!(default_v3_fallback_port(), songbird_types::constants::DEFAULT_HTTP_PORT);
        }
        let _e =
            songbird_test_utils::ScopedEnv::set("SONGBIRD_DEFAULT_DISCOVERY_PORT", "9443").await;
        assert_eq!(default_v3_fallback_port(), 9443);
    }

    #[test]
    fn build_discovery_plaintext_v21_round_trip() {
        let prep = build_discovery_plaintext(
            "2.1",
            None,
            None,
            None,
            vec!["orchestration".into()],
            vec!["https".into()],
            8443,
            None,
            Some(vec!["tag:a".into()]),
        )
        .expect("serialize v2.1");
        assert!(!prep.session_id.is_empty());
        let msg = AnonymousDiscoveryMessage::from_bytes(&prep.bytes).expect("parse");
        assert_eq!(msg.version, "2.1");
        assert_eq!(msg.capabilities, vec!["orchestration".to_string()]);
        assert_eq!(msg.port, 8443);
        assert_eq!(msg.tags, Some(vec!["tag:a".to_string()]));
    }

    #[test]
    fn build_discovery_plaintext_v30_round_trip() {
        let endpoints = vec![TransportEndpointMessage {
            interface_type: "ethernet".into(),
            address: "127.0.0.1:0".into(),
            protocols: vec!["https".into()],
            preference: 10,
        }];
        let prep = build_discovery_plaintext(
            "3.0",
            Some("nid".into()),
            Some("gate".into()),
            Some(endpoints),
            vec!["storage".into()],
            vec![],
            0,
            None,
            None,
        )
        .expect("serialize v3");
        let msg = AnonymousDiscoveryMessage::from_bytes(&prep.bytes).expect("parse");
        assert_eq!(msg.version, "3.0");
        assert_eq!(msg.node_id.as_deref(), Some("nid"));
        assert!(msg.endpoints.is_some());
    }

    #[tokio::test]
    async fn encrypt_birdsong_helpers_plaintext_when_encryption_disabled() {
        let cfg = BirdSongConfig {
            enabled: false,
            ..BirdSongConfig::default()
        };
        let birdsong = BirdSongProcessor::new(None, cfg);
        let plain = b"{\"x\":1}";
        assert_eq!(encrypt_birdsong_legacy(plain, &birdsong).await, plain);
        assert_eq!(encrypt_birdsong_after_dark_forest_failure(plain, &birdsong).await, plain);
        assert_eq!(encrypt_birdsong_dual_legacy(plain, &birdsong).await, plain);
    }

    #[tokio::test]
    async fn dark_forest_beacon_id_fallback_uses_node_id_hash() {
        let id = dark_forest_beacon_id_fallback(Some("stable-node"), None).await;
        assert_eq!(id.len(), 16);
        let id2 = dark_forest_beacon_id_fallback(Some("stable-node"), None).await;
        assert_eq!(id, id2);
    }

    #[tokio::test]
    async fn build_dark_forest_beacon_bytes_produces_output() {
        let enc = Arc::new(MockEnc {
            family_id: "fam".into(),
        });
        let cfg = BirdSongConfig::default();
        let birdsong = BirdSongProcessor::new(Some(enc), cfg);
        let bytes = build_dark_forest_beacon_bytes(
            Some("node-a".into()),
            None,
            0,
            &[String::from("ai")],
            &birdsong,
        )
        .await
        .expect("dark forest beacon");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn log_dark_forest_and_serialize_helpers_run() {
        log_dark_forest_beacon_sent(128);
        let err = serde_json::from_str::<bool>("not-json").expect_err("invalid json");
        log_serialize_error(&err);
    }
}
