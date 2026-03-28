// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! No-Op `BearDog` Provider
//!
//! Production-ready placeholder when `BearDog` is not available.
//! Unlike mocks, this clearly communicates unavailability.
//!
//! **Production Safe**: Explicitly returns errors rather than faking functionality.

use super::{
    AccessLevel, BearDogProvider, BirdSongCrypto, BroadcastKey, EncryptedBirdSong, LineageChain,
    LineageHint, LineageProof, LineageProvider, LineageRelay, RelaySession,
};
use anyhow::{Result, anyhow};

/// No-Op `BearDog` provider for when `BearDog` is not configured
///
/// This is NOT a mock - it explicitly returns errors indicating
/// that `BearDog` functionality is not available.
///
/// Use this in production when:
/// - `BearDog` is not deployed
/// - Security features are optional
/// - Graceful degradation is acceptable
pub struct NoOpBearDogProvider;

impl NoOpBearDogProvider {
    /// Create new no-op provider
    pub fn new() -> Self {
        tracing::warn!(
            "NoOpBearDogProvider created - BearDog features unavailable. \
             Set BEARDOG_URL or SECURITY_URL to enable encryption."
        );
        Self
    }
}

impl Default for NoOpBearDogProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl LineageProvider for NoOpBearDogProvider {
    async fn generate_lineage(&self, _node_id: &str, _parent_id: &str) -> Result<LineageChain> {
        Err(anyhow!(
            "BearDog not available: Cannot generate lineage. \
             Configure BearDog with BEARDOG_URL environment variable."
        ))
    }

    async fn verify_lineage(&self, _proof: &LineageProof) -> Result<bool> {
        Err(anyhow!(
            "BearDog not available: Cannot verify lineage. \
             Configure BearDog with BEARDOG_URL environment variable."
        ))
    }

    async fn get_descendants(&self, _root_id: &str) -> Result<Vec<String>> {
        Err(anyhow!(
            "BearDog not available: Cannot retrieve descendants. \
             Configure BearDog with BEARDOG_URL environment variable."
        ))
    }

    async fn get_lineage_depth(
        &self,
        _ancestor_id: &str,
        _descendant_id: &str,
    ) -> Result<Option<usize>> {
        Err(anyhow!(
            "BearDog not available: Cannot calculate lineage depth. \
             Configure BearDog with BEARDOG_URL environment variable."
        ))
    }
}

#[async_trait::async_trait]
impl BirdSongCrypto for NoOpBearDogProvider {
    async fn encrypt_for_lineage(
        &self,
        _payload: &[u8],
        _lineage_hint: LineageHint,
    ) -> Result<EncryptedBirdSong> {
        Err(anyhow!(
            "BearDog not available: Cannot encrypt for lineage. \
             Configure BearDog with BEARDOG_URL environment variable."
        ))
    }

    async fn decrypt_birdsong(&self, _encrypted: &EncryptedBirdSong) -> Result<Option<Vec<u8>>> {
        Err(anyhow!(
            "BearDog not available: Cannot decrypt birdsong. \
             Configure BearDog with BEARDOG_URL environment variable."
        ))
    }

    async fn request_key(
        &self,
        _lineage_hint: &LineageHint,
        _proof: LineageProof,
    ) -> Result<BroadcastKey> {
        Err(anyhow!(
            "BearDog not available: Cannot request key. \
             Configure BearDog with BEARDOG_URL environment variable."
        ))
    }

    async fn request_keys_batch(
        &self,
        _requests: Vec<(LineageHint, LineageProof)>,
    ) -> Result<Vec<BroadcastKey>> {
        Err(anyhow!(
            "BearDog not available: Cannot request keys batch. \
             Configure BearDog with BEARDOG_URL environment variable."
        ))
    }
}

#[async_trait::async_trait]
impl LineageRelay for NoOpBearDogProvider {
    async fn offer_relay(
        &self,
        _requester: &str,
        _target: &str,
        _lineage_proof: LineageProof,
    ) -> Result<RelaySession> {
        Err(anyhow!(
            "BearDog not available: Cannot offer relay service. \
             Configure BearDog with BEARDOG_URL environment variable."
        ))
    }

    fn get_visibility_level(&self, lineage_depth: usize) -> AccessLevel {
        // Even without BearDog, we can provide the standard mapping
        AccessLevel::from_lineage_depth(lineage_depth)
    }

    async fn relay_packet(&self, _session: &RelaySession, _packet: &[u8]) -> Result<()> {
        Err(anyhow!(
            "BearDog not available: Cannot relay packet. \
             Configure BearDog with BEARDOG_URL environment variable."
        ))
    }

    async fn revoke_relay(&self, _session_id: &str) -> Result<()> {
        Err(anyhow!(
            "BearDog not available: Cannot revoke relay. \
             Configure BearDog with BEARDOG_URL environment variable."
        ))
    }
}

#[async_trait::async_trait]
impl BearDogProvider for NoOpBearDogProvider {
    async fn is_available(&self) -> bool {
        // NoOp provider is explicitly NOT available
        false
    }

    fn version(&self) -> &'static str {
        "0.0.0-noop"
    }

    async fn shutdown(&self) -> Result<()> {
        // Nothing to shut down
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::NoOpBearDogProvider;
    use crate::beardog::{
        AccessLevel, BearDogProvider, BirdSongCrypto, EncryptedBirdSong, LineageChain, LineageHint,
        LineageProof, LineageProvider, LineageRelay, RelaySession,
    };

    #[test]
    fn default_same_as_new() {
        let a = NoOpBearDogProvider::default();
        let b = NoOpBearDogProvider::new();
        let _ = (a, b);
    }

    #[tokio::test]
    async fn lineage_methods_return_bear_dog_unavailable() {
        let p = NoOpBearDogProvider::new();
        assert!(LineageProvider::generate_lineage(&p, "n", "p").await.is_err());
        let proof = LineageProof {
            chain: LineageChain {
                root_id: "r".to_string(),
                node_id: "n".to_string(),
                links: vec![],
                depth: 0,
            },
            claimer_signature: vec![],
        };
        assert!(LineageProvider::verify_lineage(&p, &proof).await.is_err());
        assert!(LineageProvider::get_descendants(&p, "r").await.is_err());
        assert!(LineageProvider::get_lineage_depth(&p, "a", "b").await.is_err());
    }

    #[tokio::test]
    async fn birdsong_crypto_methods_return_bear_dog_unavailable() {
        let p = NoOpBearDogProvider::new();
        assert!(
            BirdSongCrypto::encrypt_for_lineage(&p, b"hi", LineageHint::Universal).await.is_err()
        );
        let enc = EncryptedBirdSong {
            version: 1,
            ciphertext: vec![],
            lineage_hint: LineageHint::Universal,
            timestamp: chrono::Utc::now(),
            signature: vec![],
            genesis_witness: None,
        };
        assert!(BirdSongCrypto::decrypt_birdsong(&p, &enc).await.is_err());
        let proof = LineageProof {
            chain: LineageChain {
                root_id: "r".into(),
                node_id: "n".into(),
                links: vec![],
                depth: 0,
            },
            claimer_signature: vec![],
        };
        assert!(BirdSongCrypto::request_key(&p, &LineageHint::Universal, proof).await.is_err());
        assert!(BirdSongCrypto::request_keys_batch(&p, vec![]).await.is_err());
    }

    #[tokio::test]
    async fn relay_errors_except_visibility_mapping() {
        let p = NoOpBearDogProvider::new();
        let proof = LineageProof {
            chain: LineageChain {
                root_id: "r".into(),
                node_id: "n".into(),
                links: vec![],
                depth: 0,
            },
            claimer_signature: vec![],
        };
        assert!(LineageRelay::offer_relay(&p, "a", "b", proof).await.is_err());
        assert!(
            LineageRelay::relay_packet(
                &p,
                &RelaySession {
                    session_id: "s".into(),
                    requester_id: "a".into(),
                    target_id: "b".into(),
                    relay_id: "c".into(),
                    access_level: AccessLevel::Transport,
                    created_at: chrono::Utc::now(),
                    expires_at: chrono::Utc::now(),
                },
                b"x"
            )
            .await
            .is_err()
        );
        assert!(LineageRelay::revoke_relay(&p, "s").await.is_err());
    }

    #[test]
    fn get_visibility_level_maps_depth() {
        let p = NoOpBearDogProvider::new();
        assert_eq!(LineageRelay::get_visibility_level(&p, 0), AccessLevel::FullLineage);
        assert_eq!(LineageRelay::get_visibility_level(&p, 5), AccessLevel::Masked);
    }

    #[tokio::test]
    async fn beardog_provider_metadata() {
        let p = NoOpBearDogProvider::new();
        assert!(!BearDogProvider::is_available(&p).await);
        assert_eq!(BearDogProvider::version(&p), "0.0.0-noop");
        BearDogProvider::shutdown(&p).await.unwrap();
    }
}
