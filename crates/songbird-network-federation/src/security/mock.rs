// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Mock `security provider` Provider
//!
//! This file is only compiled when the parent module enables it:
//! `#[cfg(any(test, feature = "test-mocks"))]` in [`crate::security`].
//!
//! For testing Songbird without `security provider` deployed.
//!
//! **SECURITY WARNING**: This is for testing only!
//! - No real encryption
//! - No real lineage verification
//! - Simulates the interface only
//!
//! **Note**: This mock is only available in test builds.
//! For production code, use the actual `security provider` provider.

use super::{
    AccessLevel, BroadcastKey, EncryptedBirdSong, LineageChain, LineageHint, LineageLink,
    LineageProof, RelaySession,
};
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock `security provider` provider for testing
///
/// Implements all `security provider` traits but with fake crypto.
pub struct MockSecurityProvider {
    /// Fake lineage graph
    lineages: Arc<RwLock<HashMap<String, Vec<String>>>>,

    /// Fake keys
    keys: Arc<RwLock<HashMap<String, BroadcastKey>>>,

    /// Active relay sessions
    sessions: Arc<RwLock<HashMap<String, RelaySession>>>,
}

impl MockSecurityProvider {
    /// Create new mock provider
    #[must_use]
    pub fn new() -> Self {
        Self {
            lineages: Arc::new(RwLock::new(HashMap::new())),
            keys: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a fake lineage for testing
    pub async fn add_test_lineage(&self, root: String, descendants: Vec<String>) {
        self.lineages.write().await.insert(root, descendants);
    }

    /// Generate lineage for a new node
    pub async fn generate_lineage(&self, node_id: &str, parent_id: &str) -> Result<LineageChain> {
        tracing::warn!("🐻 MOCK: Generating fake lineage");

        Ok(LineageChain {
            root_id: parent_id.to_string(),
            node_id: node_id.to_string(),
            links: vec![LineageLink {
                parent_id: parent_id.to_string(),
                child_id: node_id.to_string(),
                signature: vec![0xDE, 0xAD, 0xBE, 0xEF], // Fake signature
                created_at: chrono::Utc::now(),
            }],
            depth: 1,
        })
    }

    /// Verify a lineage proof
    pub async fn verify_lineage(&self, proof: &LineageProof) -> Result<bool> {
        tracing::warn!("🐻 MOCK: Fake lineage verification (always true)");
        Ok(proof.chain.verify_integrity().await?)
    }

    /// Get all descendants of a root
    pub async fn get_descendants(&self, root_id: &str) -> Result<Vec<String>> {
        let lineages = self.lineages.read().await;
        Ok(lineages.get(root_id).cloned().unwrap_or_default())
    }

    /// Get lineage depth between two nodes
    pub async fn get_lineage_depth(
        &self,
        ancestor_id: &str,
        descendant_id: &str,
    ) -> Result<Option<usize>> {
        let lineages = self.lineages.read().await;

        if let Some(descendants) = lineages.get(ancestor_id)
            && descendants.contains(&descendant_id.to_string())
        {
            return Ok(Some(1)); // Fake depth
        }

        Ok(None)
    }

    /// Encrypt payload for a specific lineage
    pub async fn encrypt_for_lineage(
        &self,
        payload: &[u8],
        lineage_hint: LineageHint,
    ) -> Result<EncryptedBirdSong> {
        tracing::warn!("🐻 MOCK: Fake encryption (plaintext with marker)");

        // "Encrypt" by just adding a marker
        let mut ciphertext = b"MOCK_ENCRYPTED:".to_vec();
        ciphertext.extend_from_slice(payload);

        Ok(EncryptedBirdSong {
            version: 1,
            ciphertext,
            lineage_hint,
            timestamp: chrono::Utc::now(),
            signature: vec![0xCA, 0xFE, 0xBA, 0xBE], // Fake signature
            genesis_witness: None,                   // No genesis witness for mock broadcasts
        })
    }

    /// Decrypt birdSong (if we're in the lineage)
    pub async fn decrypt_birdsong(&self, encrypted: &EncryptedBirdSong) -> Result<Option<Vec<u8>>> {
        tracing::warn!("🐻 MOCK: Fake decryption");

        // "Decrypt" by removing marker
        if encrypted.ciphertext.starts_with(b"MOCK_ENCRYPTED:") {
            let payload = encrypted.ciphertext[15..].to_vec();
            Ok(Some(payload))
        } else {
            Ok(None)
        }
    }

    /// Request decryption key for a lineage
    pub async fn request_key(
        &self,
        lineage_hint: &LineageHint,
        _proof: LineageProof,
    ) -> Result<BroadcastKey> {
        tracing::warn!("🐻 MOCK: Generating fake key");

        let key_id = format!("mock_key_{lineage_hint:?}");
        let key = BroadcastKey {
            key_id: key_id.clone(),
            key_data: vec![0x12, 0x34, 0x56, 0x78], // Fake key
            valid_from: chrono::Utc::now(),
            valid_until: chrono::Utc::now() + chrono::Duration::days(30),
        };

        self.keys.write().await.insert(key_id, key.clone());
        Ok(key)
    }

    /// Batch key request (for efficiency)
    pub async fn request_keys_batch(
        &self,
        requests: Vec<(LineageHint, LineageProof)>,
    ) -> Result<Vec<BroadcastKey>> {
        let mut keys = Vec::new();
        for (hint, proof) in requests {
            keys.push(self.request_key(&hint, proof).await?);
        }
        Ok(keys)
    }

    /// Offer relay service to descendant
    pub async fn offer_relay(
        &self,
        requester: &str,
        target: &str,
        _lineage_proof: LineageProof,
    ) -> Result<RelaySession> {
        tracing::warn!("🐻 MOCK: Creating fake relay session");

        let session = RelaySession {
            session_id: format!("mock_relay_{}", uuid::Uuid::new_v4()),
            requester_id: requester.to_string(),
            target_id: target.to_string(),
            relay_id: "mock_relay_node".to_string(),
            access_level: AccessLevel::SubMasked,
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        };

        self.sessions.write().await.insert(session.session_id.clone(), session.clone());
        Ok(session)
    }

    /// Get visibility level based on lineage depth
    pub fn get_visibility_level(&self, lineage_depth: usize) -> AccessLevel {
        AccessLevel::from_lineage_depth(lineage_depth)
    }

    /// Relay packet (with masking enforced)
    pub async fn relay_packet(&self, session: &RelaySession, _packet: &[u8]) -> Result<()> {
        if !session.is_active() {
            return Err(anyhow!("Session expired"));
        }

        tracing::debug!("🐻 MOCK: Relaying packet (no-op)");
        Ok(())
    }

    /// Revoke relay for a session
    pub async fn revoke_relay(&self, session_id: &str) -> Result<()> {
        self.sessions.write().await.remove(session_id);
        tracing::warn!("🐻 MOCK: Revoked relay session");
        Ok(())
    }

    /// Check if the provider is available and operational
    pub async fn is_available(&self) -> bool {
        tracing::warn!("🐻 MOCK: Always available (but not real!)");
        true
    }

    /// Provider version for compatibility checking
    pub fn version(&self) -> &'static str {
        "0.0.0-mock"
    }

    /// Graceful shutdown
    pub async fn shutdown(&self) -> Result<()> {
        tracing::warn!("🐻 MOCK: Shutting down (no-op)");
        Ok(())
    }
}

impl Default for MockSecurityProvider {
    fn default() -> Self {
        Self::new()
    }
}
