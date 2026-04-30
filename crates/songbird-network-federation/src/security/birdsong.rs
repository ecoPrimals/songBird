// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `BirdSong` Encryption Trait
//!
//! `security provider` provides encryption/decryption for birdSong broadcasts.

use super::genesis::GenesisWitnessProof;
use super::lineage::LineageProof;
use serde::{Deserialize, Serialize};

/// `BirdSong` encryption provider
///
/// `security provider` implements this to encrypt/decrypt birdSong messages.
pub trait BirdSongCrypto: Send + Sync {
    /// Encrypt payload for a specific lineage
    ///
    /// Creates an encrypted birdSong that only the specified lineage can decrypt.
    fn encrypt_for_lineage(
        &self,
        payload: &[u8],
        lineage_hint: LineageHint,
    ) -> impl std::future::Future<Output = anyhow::Result<EncryptedBirdSong>> + Send;

    /// Decrypt birdSong (if we're in the lineage)
    ///
    /// Returns the decrypted payload if we have the key, None otherwise.
    fn decrypt_birdsong(
        &self,
        encrypted: &EncryptedBirdSong,
    ) -> impl std::future::Future<Output = anyhow::Result<Option<Vec<u8>>>> + Send;

    /// Request decryption key for a lineage
    ///
    /// `security provider` verifies the lineage proof, then provides the key if authorized.
    fn request_key(
        &self,
        lineage_hint: &LineageHint,
        proof: LineageProof,
    ) -> impl std::future::Future<Output = anyhow::Result<BroadcastKey>> + Send;

    /// Batch key request (for efficiency)
    ///
    /// Request multiple keys at once to amortize overhead.
    fn request_keys_batch(
        &self,
        requests: Vec<(LineageHint, LineageProof)>,
    ) -> impl std::future::Future<Output = anyhow::Result<Vec<BroadcastKey>>> + Send;
}

/// Encrypted birdSong message
///
/// This is what gets broadcast over UDP.
/// Only family (with lineage proof) can decrypt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedBirdSong {
    /// Protocol version (for future evolution)
    pub version: u8,

    /// Encrypted payload
    pub ciphertext: Vec<u8>,

    /// Hint about which lineage can decrypt
    /// NOT a full lineage proof, just a hint for key selection
    pub lineage_hint: LineageHint,

    /// Timestamp (prevents replay attacks)
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Signature (proves authenticity, NOT for decryption)
    pub signature: Vec<u8>,

    /// Optional genesis witness for new nodes
    /// Present when a new node is broadcasting its genesis certification
    /// during initial discovery ("bird in a dark forest")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genesis_witness: Option<GenesisWitnessProof>,
}

/// Hint about which lineage can decrypt
///
/// This is NOT a proof, just a hint to help select the right key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LineageHint {
    /// My direct descendants only (children)
    DirectDescendants,

    /// All descendants (any generation)
    AllDescendants,

    /// Specific lineage root
    LineageRoot(String),

    /// Broadcast to all (but only family can decrypt)
    Universal,
}

/// Broadcast key for encrypting/decrypting birdSong
///
/// Derived from lineage, distributed by `security provider`.
/// **Phase 3**: Used for `BirdSong` encrypted broadcasts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastKey {
    /// Key identifier (for caching)
    pub key_id: String,

    /// The actual key material (kept private)
    /// **Phase 3**: Used by `encrypt_broadcast()` and `decrypt_broadcast()` methods
    #[allow(dead_code, reason = "Phase 3: key material wired when BirdSong crypto is finalized")]
    #[serde(skip)]
    pub(crate) key_data: Vec<u8>,

    /// When this key becomes valid
    pub valid_from: chrono::DateTime<chrono::Utc>,

    /// When this key expires (for rotation)
    pub valid_until: chrono::DateTime<chrono::Utc>,
}

impl BroadcastKey {
    /// Encrypt broadcast data using this key via `security provider` `ChaCha20-Poly1305`.
    ///
    /// Production builds delegate to `CryptoProvider` for real AEAD encryption.
    /// Test/mock builds use a non-cryptographic XOR stand-in for framing tests.
    #[allow(dead_code, reason = "Phase 3: encrypt path until BirdSong integration ships")]
    pub async fn encrypt_broadcast(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        #[cfg(any(test, feature = "test-mocks"))]
        {
            Ok(self.key_data.iter().cycle().zip(data.iter()).map(|(k, d)| k ^ d).collect())
        }
        #[cfg(not(any(test, feature = "test-mocks")))]
        {
            use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

            let crypto = songbird_crypto_provider::CryptoProvider::from_env();

            // 12-byte nonce derived from key_id for deterministic replay detection
            let mut nonce = [0u8; 12];
            let id_bytes = self.key_id.as_bytes();
            for (i, b) in id_bytes.iter().take(12).enumerate() {
                nonce[i] = *b;
            }

            let result = crypto
                .call(
                    "crypto.encrypt_chacha20_poly1305",
                    serde_json::json!({
                        "key": BASE64.encode(&self.key_data),
                        "nonce": BASE64.encode(nonce),
                        "plaintext": BASE64.encode(data),
                        "aad": BASE64.encode(b"birdsong"),
                    }),
                )
                .await
                .map_err(|e| {
                    anyhow::anyhow!(
                        "CryptoUnavailable: BirdSong encrypt via security provider: {e}"
                    )
                })?;

            let ct_b64 = result
                .as_str()
                .or_else(|| result.get("ciphertext").and_then(serde_json::Value::as_str))
                .unwrap_or("");

            BASE64.decode(ct_b64).map_err(|e| {
                anyhow::anyhow!(
                    "BirdSong encrypt: failed to decode security provider response: {e}"
                )
            })
        }
    }

    /// Check if this key is currently valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        let now = chrono::Utc::now();
        now >= self.valid_from && now <= self.valid_until
    }

    /// Check if this key is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.valid_until
    }

    /// Time until expiration
    #[must_use]
    pub fn time_until_expiry(&self) -> Option<chrono::Duration> {
        let now = chrono::Utc::now();
        if now > self.valid_until {
            None
        } else {
            Some(self.valid_until - now)
        }
    }
}

impl EncryptedBirdSong {
    /// Check if this birdSong is recent (not a replay)
    ///
    /// Rejects messages older than 60 seconds.
    #[must_use]
    pub fn is_recent(&self) -> bool {
        let age = chrono::Utc::now() - self.timestamp;
        age.num_seconds() < 60
    }

    /// Check if timestamp is valid (not in the future)
    #[must_use]
    pub fn has_valid_timestamp(&self) -> bool {
        let now = chrono::Utc::now();
        self.timestamp <= now
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::security::genesis::{
        GenesisTrustLevel, GenesisWitnessProof, PhysicalChannelType, PrimalWitnessSignature,
    };

    #[test]
    fn lineage_hint_json_roundtrip_all_variants() {
        let variants = vec![
            LineageHint::DirectDescendants,
            LineageHint::AllDescendants,
            LineageHint::LineageRoot("root-a".into()),
            LineageHint::Universal,
        ];
        for hint in variants {
            let json = serde_json::to_string(&hint).unwrap();
            let back: LineageHint = serde_json::from_str(&json).unwrap();
            assert_eq!(back, hint);
        }
    }

    #[test]
    fn encrypted_birdsong_json_roundtrip_without_genesis() {
        let ts = chrono::Utc::now();
        let msg = EncryptedBirdSong {
            version: 7,
            ciphertext: vec![1, 2, 3],
            lineage_hint: LineageHint::DirectDescendants,
            timestamp: ts,
            signature: vec![9],
            genesis_witness: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: EncryptedBirdSong = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version, msg.version);
        assert_eq!(back.ciphertext, msg.ciphertext);
        assert_eq!(back.lineage_hint, msg.lineage_hint);
        assert_eq!(back.timestamp, msg.timestamp);
        assert_eq!(back.signature, msg.signature);
        assert!(back.genesis_witness.is_none());
    }

    #[test]
    fn encrypted_birdsong_json_roundtrip_with_genesis() {
        let witness = GenesisWitnessProof {
            ceremony_id: "c".into(),
            node_id: "n".into(),
            witness_device_id: "w".into(),
            witness_signature: vec![1],
            physical_channel: PhysicalChannelType::Bluetooth,
            primal_witnesses: vec![PrimalWitnessSignature {
                primal_name: "Songbird".into(),
                lineage_data: vec![2],
                signature: vec![3],
                witness_timestamp: chrono::Utc::now(),
            }],
            birth_timestamp: chrono::Utc::now(),
            trust_level: GenesisTrustLevel::Basic,
        };
        let msg = EncryptedBirdSong {
            version: 1,
            ciphertext: vec![],
            lineage_hint: LineageHint::Universal,
            timestamp: chrono::Utc::now(),
            signature: vec![],
            genesis_witness: Some(witness.clone()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let back: EncryptedBirdSong = serde_json::from_str(&json).unwrap();
        assert_eq!(back.genesis_witness.as_ref().unwrap().node_id, witness.node_id);
    }

    #[test]
    fn encrypted_birdsong_is_recent_and_timestamp_checks() {
        let fresh = EncryptedBirdSong {
            version: 1,
            ciphertext: vec![],
            lineage_hint: LineageHint::Universal,
            timestamp: chrono::Utc::now(),
            signature: vec![],
            genesis_witness: None,
        };
        assert!(fresh.is_recent());
        assert!(fresh.has_valid_timestamp());

        let stale = EncryptedBirdSong {
            timestamp: chrono::Utc::now() - chrono::Duration::seconds(120),
            ..fresh.clone()
        };
        assert!(!stale.is_recent());

        let future_ts = EncryptedBirdSong {
            timestamp: chrono::Utc::now() + chrono::Duration::hours(24),
            ..fresh
        };
        assert!(!future_ts.has_valid_timestamp());
    }

    #[test]
    fn broadcast_key_json_roundtrip_preserves_public_fields() {
        let from = chrono::Utc::now() - chrono::Duration::hours(1);
        let until = chrono::Utc::now() + chrono::Duration::hours(1);
        let key = BroadcastKey {
            key_id: "kid-1".into(),
            key_data: vec![10, 20],
            valid_from: from,
            valid_until: until,
        };
        let json = serde_json::to_string(&key).unwrap();
        assert!(!json.contains("key_data"), "key_data must not appear in JSON");
        let back: BroadcastKey = serde_json::from_str(&json).unwrap();
        assert_eq!(back.key_id, key.key_id);
        assert!(back.key_data.is_empty(), "skipped field deserializes empty");
        assert_eq!(back.valid_from, key.valid_from);
        assert_eq!(back.valid_until, key.valid_until);
    }

    #[test]
    fn broadcast_key_validity_helpers_track_expiry_window() {
        let from_past = chrono::Utc::now() - chrono::Duration::hours(2);
        let until_past = chrono::Utc::now() - chrono::Duration::seconds(1);
        let expired = BroadcastKey {
            key_id: "exp".into(),
            key_data: vec![1],
            valid_from: from_past,
            valid_until: until_past,
        };
        assert!(!expired.is_valid());
        assert!(expired.is_expired());
        assert!(expired.time_until_expiry().is_none());

        let active = BroadcastKey {
            key_id: "act".into(),
            key_data: vec![2],
            valid_from: from_past,
            valid_until: chrono::Utc::now() + chrono::Duration::hours(5),
        };
        assert!(active.is_valid());
        assert!(!active.is_expired());
        let remaining = active.time_until_expiry().expect("still valid");
        assert!(remaining > chrono::Duration::zero());
    }

    #[tokio::test]
    async fn broadcast_key_encrypt_broadcast_xor_under_test_cfg() {
        let key = BroadcastKey {
            key_id: "ab".into(),
            key_data: vec![0x0F, 0xF0],
            valid_from: chrono::Utc::now(),
            valid_until: chrono::Utc::now() + chrono::Duration::hours(1),
        };
        let plain = [1u8, 2, 3, 4];
        let ct = key.encrypt_broadcast(&plain).await.unwrap();
        assert_eq!(ct.len(), plain.len());
        let expected: Vec<u8> =
            key.key_data.iter().cycle().zip(plain.iter()).map(|(k, d)| k ^ d).collect();
        assert_eq!(ct, expected);
    }
}
