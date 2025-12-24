//! Mock BearDog implementations for testing
//!
//! These mocks allow Songbird to test lineage relay without BearDog running

use crate::birdsong::{BirdSongCrypto, LineageHint};
use crate::error::Result;
use crate::relay::RelayAuthority;
use crate::types::{MaskingLevel, NodeId, RelayAuthorization};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

/// Mock lineage provider for testing
pub struct MockLineageProvider {
    /// Lineage graph: node_id → parent_id
    lineages: Arc<RwLock<HashMap<String, String>>>,
    /// Descendants: ancestor_id → list of descendant_ids
    descendants: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

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
        descendants
            .entry(parent.to_string())
            .or_insert_with(Vec::new)
            .push(child.to_string());
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

impl Default for MockLineageProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock BirdSong crypto (for testing)
pub struct MockBirdSongCrypto {
    lineage_provider: Arc<MockLineageProvider>,
    my_id: String,
}

impl MockBirdSongCrypto {
    /// Create new mock crypto
    #[must_use]
    pub fn new(lineage_provider: Arc<MockLineageProvider>, my_id: String) -> Self {
        Self {
            lineage_provider,
            my_id,
        }
    }
}

#[async_trait]
impl BirdSongCrypto for MockBirdSongCrypto {
    async fn encrypt_for_lineage(&self, message: &[u8], _hint: LineageHint) -> Result<Vec<u8>> {
        // Mock: prepend "LINEAGE:" to indicate lineage-encrypted
        let mut encrypted = b"LINEAGE:".to_vec();
        encrypted.extend_from_slice(message);
        Ok(encrypted)
    }

    async fn decrypt_birdsong(&self, encrypted: &[u8], sender: &NodeId) -> Result<Option<Vec<u8>>> {
        // Check if we're in sender's lineage
        let can_decrypt = self
            .lineage_provider
            .is_ancestor(&self.my_id, &sender.0)
            .await
            || self
                .lineage_provider
                .is_descendant(&self.my_id, &sender.0)
                .await;

        if can_decrypt && encrypted.starts_with(b"LINEAGE:") {
            Ok(Some(encrypted[8..].to_vec()))
        } else {
            Ok(None)
        }
    }
}

/// Mock relay authority (for testing)
pub struct MockRelayAuthority {
    lineage_provider: Arc<MockLineageProvider>,
}

impl MockRelayAuthority {
    /// Create new mock relay authority
    #[must_use]
    pub fn new(lineage_provider: Arc<MockLineageProvider>) -> Self {
        Self { lineage_provider }
    }
}

#[async_trait]
impl RelayAuthority for MockRelayAuthority {
    async fn authorize_relay(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<RelayAuthorization> {
        // Check if relay_node is ancestor of requester
        let authorized = self
            .lineage_provider
            .is_ancestor(&requester.0, &relay_node.0)
            .await;

        Ok(RelayAuthorization {
            relay_node: relay_node.clone(),
            requester: requester.clone(),
            authorized,
            masking_level: if authorized {
                MaskingLevel::Masked
            } else {
                MaskingLevel::FullVisibility
            },
            ttl_seconds: 300,
            issued_at: SystemTime::now(),
            audit_token: format!("mock_token_{}", uuid::Uuid::new_v4()),
        })
    }

    async fn determine_masking(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<MaskingLevel> {
        // Simple masking: masked for descendants
        let is_ancestor = self
            .lineage_provider
            .is_ancestor(&requester.0, &relay_node.0)
            .await;

        Ok(if is_ancestor {
            MaskingLevel::Masked
        } else {
            MaskingLevel::FullVisibility
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_lineage_provider() {
        let provider = MockLineageProvider::new();

        // Create lineage: child → parent → grandparent
        provider.add_lineage("child", "parent").await;
        provider.add_lineage("parent", "grandparent").await;

        assert!(provider.is_ancestor("child", "parent").await);
        assert!(provider.is_ancestor("child", "grandparent").await);
        assert!(!provider.is_ancestor("parent", "child").await);

        assert!(provider.is_descendant("parent", "child").await);
        assert!(provider.is_descendant("grandparent", "child").await);
    }

    #[tokio::test]
    async fn test_mock_birdsong_crypto() {
        let provider = Arc::new(MockLineageProvider::new());
        provider.add_lineage("child", "parent").await;

        let crypto = MockBirdSongCrypto::new(provider.clone(), "parent".to_string());

        let message = b"test message";
        let encrypted = crypto
            .encrypt_for_lineage(message, LineageHint::DirectAncestors)
            .await
            .unwrap();

        // Parent should be able to decrypt child's message
        let decrypted = crypto
            .decrypt_birdsong(&encrypted, &NodeId::from("child"))
            .await
            .unwrap();
        assert_eq!(decrypted, Some(message.to_vec()));

        // Unrelated node cannot decrypt
        let crypto_unrelated = MockBirdSongCrypto::new(provider, "unrelated".to_string());
        let decrypted_unrelated = crypto_unrelated
            .decrypt_birdsong(&encrypted, &NodeId::from("child"))
            .await
            .unwrap();
        assert_eq!(decrypted_unrelated, None);
    }

    #[tokio::test]
    async fn test_mock_relay_authority() {
        let provider = Arc::new(MockLineageProvider::new());
        provider.add_lineage("child", "parent").await;

        let authority = MockRelayAuthority::new(provider);

        // Parent should be authorized to relay for child
        let auth = authority
            .authorize_relay(&NodeId::from("parent"), &NodeId::from("child"))
            .await
            .unwrap();
        assert!(auth.authorized);

        // Child should NOT be authorized to relay for parent
        let auth = authority
            .authorize_relay(&NodeId::from("child"), &NodeId::from("parent"))
            .await
            .unwrap();
        assert!(!auth.authorized);
    }
}

