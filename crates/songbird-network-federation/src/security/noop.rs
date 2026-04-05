// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! No-op security provider — **fallback when no security provider process is available**.
//!
//! ## Role
//!
//! `SecurityProviderFactory::discover` walks UPA /
//! environment / well-known sockets (see wateringHole v1.2 capability layout). When nothing
//! responds, callers may use `SecurityProviderFactory::create_noop` to obtain a
//! [`NoOpSecurityProvider`] instead of holding `Option<dyn SecurityProvider>` everywhere.
//!
//! This type is **not** a mock: crypto and lineage RPCs return [`Result::Err`] with
//! [`NoOpSecurityError`], not fake ciphertext. [`LineageRelay::get_visibility_level`] is a
//! pure local mapping ([`AccessLevel::from_lineage_depth`]) and does not contact a remote.
//! [`SecurityProvider::is_available`] is `false`; [`SecurityProvider::shutdown`] is a no-op
//! success.
//!
//! ## Runtime selection (not Cargo feature-gated)
//!
//! The real provider is chosen at runtime via capability discovery and env vars
//! (`SECURITY_PROVIDER_SOCKET`, `SECURITY_SOCKET`, legacy `BEARDOG_SOCKET`; `SECURITY_URL` or
//! legacy `BEARDOG_URL` for `unix://` paths). This module is always compiled so binaries can degrade gracefully
//! without a separate feature flag.

use super::{
    AccessLevel, BirdSongCrypto, BroadcastKey, EncryptedBirdSong, LineageChain, LineageHint,
    LineageProof, LineageProvider, LineageRelay, RelaySession, SecurityProvider,
};
use thiserror::Error;

/// Errors from [`NoOpSecurityProvider`] — all operations that require a remote security provider fail.
#[derive(Debug, Error)]
pub enum NoOpSecurityError {
    /// No security provider; lineage operations are unavailable.
    #[error(
        "security provider unavailable: cannot {operation}; set SECURITY_PROVIDER_SOCKET or SECURITY_SOCKET (legacy BEARDOG_SOCKET), or discover the security capability (see SecurityProviderFactory)"
    )]
    LineageUnavailable {
        /// Operation name for logs and diagnostics.
        operation: &'static str,
    },

    /// No security provider; `BirdSong` crypto operations are unavailable.
    #[error(
        "security provider unavailable: cannot {operation}; set SECURITY_PROVIDER_SOCKET or SECURITY_SOCKET (legacy BEARDOG_SOCKET), or discover the security capability"
    )]
    BirdSongUnavailable {
        /// Operation name for logs and diagnostics.
        operation: &'static str,
    },

    /// No security provider; relay operations are unavailable.
    #[error(
        "security provider unavailable: cannot {operation}; set SECURITY_PROVIDER_SOCKET or SECURITY_SOCKET (legacy BEARDOG_SOCKET), or discover the security capability"
    )]
    RelayUnavailable {
        /// Operation name for logs and diagnostics.
        operation: &'static str,
    },
}

/// No-op provider when the security provider is not configured or discoverable.
///
/// Prefer `SecurityProviderFactory::discover` first; use [`NoOpSecurityProvider::new`] only when
/// you explicitly need a [`SecurityProvider`] trait object that reports unavailable for crypto.
pub struct NoOpSecurityProvider;

impl NoOpSecurityProvider {
    /// Create a new no-op provider (logs once at `warn` level).
    pub fn new() -> Self {
        tracing::warn!(
            "NoOpSecurityProvider: no security provider — encryption and lineage RPCs will fail. \
             Set SECURITY_PROVIDER_SOCKET or SECURITY_SOCKET (legacy BEARDOG_SOCKET), or ensure UPA \
             discovers the security capability."
        );
        Self
    }
}

impl Default for NoOpSecurityProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl LineageProvider for NoOpSecurityProvider {
    async fn generate_lineage(
        &self,
        _node_id: &str,
        _parent_id: &str,
    ) -> anyhow::Result<LineageChain> {
        Err(NoOpSecurityError::LineageUnavailable {
            operation: "generate_lineage",
        }
        .into())
    }

    async fn verify_lineage(&self, _proof: &LineageProof) -> anyhow::Result<bool> {
        Err(NoOpSecurityError::LineageUnavailable {
            operation: "verify_lineage",
        }
        .into())
    }

    async fn get_descendants(&self, _root_id: &str) -> anyhow::Result<Vec<String>> {
        Err(NoOpSecurityError::LineageUnavailable {
            operation: "get_descendants",
        }
        .into())
    }

    async fn get_lineage_depth(
        &self,
        _ancestor_id: &str,
        _descendant_id: &str,
    ) -> anyhow::Result<Option<usize>> {
        Err(NoOpSecurityError::LineageUnavailable {
            operation: "get_lineage_depth",
        }
        .into())
    }
}

#[async_trait::async_trait]
impl BirdSongCrypto for NoOpSecurityProvider {
    async fn encrypt_for_lineage(
        &self,
        _payload: &[u8],
        _lineage_hint: LineageHint,
    ) -> anyhow::Result<EncryptedBirdSong> {
        Err(NoOpSecurityError::BirdSongUnavailable {
            operation: "encrypt_for_lineage",
        }
        .into())
    }

    async fn decrypt_birdsong(
        &self,
        _encrypted: &EncryptedBirdSong,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        Err(NoOpSecurityError::BirdSongUnavailable {
            operation: "decrypt_birdsong",
        }
        .into())
    }

    async fn request_key(
        &self,
        _lineage_hint: &LineageHint,
        _proof: LineageProof,
    ) -> anyhow::Result<BroadcastKey> {
        Err(NoOpSecurityError::BirdSongUnavailable {
            operation: "request_key",
        }
        .into())
    }

    async fn request_keys_batch(
        &self,
        _requests: Vec<(LineageHint, LineageProof)>,
    ) -> anyhow::Result<Vec<BroadcastKey>> {
        Err(NoOpSecurityError::BirdSongUnavailable {
            operation: "request_keys_batch",
        }
        .into())
    }
}

#[async_trait::async_trait]
impl LineageRelay for NoOpSecurityProvider {
    async fn offer_relay(
        &self,
        _requester: &str,
        _target: &str,
        _lineage_proof: LineageProof,
    ) -> anyhow::Result<RelaySession> {
        Err(NoOpSecurityError::RelayUnavailable {
            operation: "offer_relay",
        }
        .into())
    }

    fn get_visibility_level(&self, lineage_depth: usize) -> AccessLevel {
        AccessLevel::from_lineage_depth(lineage_depth)
    }

    async fn relay_packet(&self, _session: &RelaySession, _packet: &[u8]) -> anyhow::Result<()> {
        Err(NoOpSecurityError::RelayUnavailable {
            operation: "relay_packet",
        }
        .into())
    }

    async fn revoke_relay(&self, _session_id: &str) -> anyhow::Result<()> {
        Err(NoOpSecurityError::RelayUnavailable {
            operation: "revoke_relay",
        }
        .into())
    }
}

#[async_trait::async_trait]
impl SecurityProvider for NoOpSecurityProvider {
    async fn is_available(&self) -> bool {
        false
    }

    fn version(&self) -> &'static str {
        "0.0.0-noop"
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::NoOpSecurityProvider;
    use crate::security::{
        AccessLevel, BirdSongCrypto, EncryptedBirdSong, LineageChain, LineageHint, LineageProof,
        LineageProvider, LineageRelay, RelaySession, SecurityProvider,
    };

    #[test]
    fn default_same_as_new() {
        let a = NoOpSecurityProvider;
        let b = NoOpSecurityProvider::new();
        let _ = (a, b);
    }

    #[tokio::test]
    async fn lineage_methods_return_unavailable() {
        let p = NoOpSecurityProvider::new();
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
    async fn birdsong_crypto_methods_return_unavailable() {
        let p = NoOpSecurityProvider::new();
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
        let p = NoOpSecurityProvider::new();
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
        let p = NoOpSecurityProvider::new();
        assert_eq!(LineageRelay::get_visibility_level(&p, 0), AccessLevel::FullLineage);
        assert_eq!(LineageRelay::get_visibility_level(&p, 5), AccessLevel::Masked);
    }

    #[tokio::test]
    async fn security_provider_metadata() {
        let p = NoOpSecurityProvider::new();
        assert!(!SecurityProvider::is_available(&p).await);
        assert_eq!(SecurityProvider::version(&p), "0.0.0-noop");
        SecurityProvider::shutdown(&p).await.unwrap();
    }
}
