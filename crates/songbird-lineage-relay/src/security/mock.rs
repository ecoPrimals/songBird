// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

// ═══════════════════════════════════════════════════════════════════
// TEST MOCKS - Gated behind cfg(test) or feature = "test-mocks"
//
// Unit tests get these via #[cfg(test)]; integration tests
// enable the `test-utils` feature in dev-dependencies.
// ═══════════════════════════════════════════════════════════════════

use crate::error::Result;
use crate::types::{LineageHint, MaskingLevel, NodeId, RelayAuthorization};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

#[cfg(any(test, feature = "test-mocks"))]
/// In-memory lineage graph for tests and the `test-utils` feature (replaces a live security provider in CI).
#[derive(Debug)]
pub struct MockLineageProvider {
    /// Lineage graph: `node_id` → `parent_id`
    lineages: Arc<RwLock<HashMap<String, String>>>,
    /// Descendants: `ancestor_id` → list of `descendant_ids`
    descendants: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

#[cfg(any(test, feature = "test-mocks"))]
impl MockLineageProvider {
    /// Create new mock lineage provider
    #[must_use]
    pub fn new() -> Self {
        Self {
            lineages: Arc::new(RwLock::new(HashMap::new())),
            descendants: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add lineage relationship (for testing)
    pub async fn add_lineage(&self, child: &str, parent: &str) {
        self.lineages.write().await.insert(child.to_string(), parent.to_string());

        // Add to descendants
        let mut descendants = self.descendants.write().await;
        descendants.entry(parent.to_string()).or_insert_with(Vec::new).push(child.to_string());
    }

    /// Check if node2 is an ancestor of node1
    pub async fn is_ancestor(&self, node: &str, potential_ancestor: &str) -> bool {
        let lineages = self.lineages.read().await;
        let mut current = node.to_string();

        // Walk up the lineage chain
        while let Some(parent) = lineages.get(&current) {
            if parent == potential_ancestor {
                return true;
            }
            current = parent.clone();
        }

        false
    }

    /// Check if node2 is a descendant of node1
    pub async fn is_descendant(&self, node: &str, potential_descendant: &str) -> bool {
        self.is_ancestor(potential_descendant, node).await
    }
}

#[cfg(any(test, feature = "test-mocks"))]
impl Default for MockLineageProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock `BirdSong` crypto (for testing and integration tests)
#[cfg(any(test, feature = "test-mocks"))]
#[derive(Clone, Debug)]
pub struct MockBirdSongCrypto {
    lineage_provider: Arc<MockLineageProvider>,
    my_id: String,
}

#[cfg(any(test, feature = "test-mocks"))]
impl MockBirdSongCrypto {
    /// Create new mock crypto
    #[must_use]
    pub const fn new(lineage_provider: Arc<MockLineageProvider>, my_id: String) -> Self {
        Self {
            lineage_provider,
            my_id,
        }
    }
}

#[cfg(any(test, feature = "test-mocks"))]
impl MockBirdSongCrypto {
    /// Encrypt message for lineage (mock: `LINEAGE:` prefix).
    pub async fn encrypt_for_lineage(&self, message: &[u8], _hint: LineageHint) -> Result<Vec<u8>> {
        let mut encrypted = b"LINEAGE:".to_vec();
        encrypted.extend_from_slice(message);
        Ok(encrypted)
    }

    /// Decrypt if the mock lineage graph allows reception from `sender`.
    pub async fn decrypt_birdsong(
        &self,
        encrypted: &[u8],
        sender: &NodeId,
    ) -> Result<Option<Vec<u8>>> {
        let can_decrypt = self.lineage_provider.is_ancestor(&self.my_id, &sender.0).await
            || self.lineage_provider.is_descendant(&self.my_id, &sender.0).await;

        if can_decrypt && encrypted.starts_with(b"LINEAGE:") {
            Ok(Some(encrypted[8..].to_vec()))
        } else {
            Ok(None)
        }
    }
}

/// Mock relay authority (for testing and integration tests)
#[cfg(any(test, feature = "test-mocks"))]
#[derive(Clone, Debug)]
pub struct MockRelayAuthority {
    lineage_provider: Arc<MockLineageProvider>,
}

#[cfg(any(test, feature = "test-mocks"))]
impl MockRelayAuthority {
    /// Create new mock relay authority
    #[must_use]
    pub const fn new(lineage_provider: Arc<MockLineageProvider>) -> Self {
        Self {
            lineage_provider,
        }
    }
}

#[cfg(any(test, feature = "test-mocks"))]
impl MockRelayAuthority {
    /// Authorize relay when `relay_node` is an ancestor of `requester` in the mock graph.
    pub async fn authorize_relay(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<RelayAuthorization> {
        let authorized = self.lineage_provider.is_ancestor(&requester.0, &relay_node.0).await;

        Ok(RelayAuthorization {
            relay_node: relay_node.clone(),
            requester: requester.clone(),
            authorized,
            masking_level: if authorized {
                MaskingLevel::Masked
            } else {
                MaskingLevel::FullVisibility
            },
            ttl_seconds: 300_u64,
            issued_at: SystemTime::now(),
            audit_token: format!("mock_token_{}", uuid::Uuid::new_v4()),
        })
    }

    /// Masking tier for the mock lineage relationship.
    pub async fn determine_masking(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<MaskingLevel> {
        let is_ancestor = self.lineage_provider.is_ancestor(&requester.0, &relay_node.0).await;

        Ok(if is_ancestor {
            MaskingLevel::Masked
        } else {
            MaskingLevel::FullVisibility
        })
    }
}
