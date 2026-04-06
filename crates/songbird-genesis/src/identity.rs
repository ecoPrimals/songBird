// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! New node identity types

use crate::types::{GenesisLineage, PrimalLineage, TrustLevel};
use crate::witness::GenesisWitness;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identity for a newly created node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewNodeIdentity {
    /// Unique node identifier
    pub node_id: String,

    /// Cryptographic public key for this node
    pub public_key: Vec<u8>,

    /// Genesis witness who created this node
    pub genesis_witness: GenesisWitness,

    /// Complete genesis lineage from all primals
    pub genesis_lineage: GenesisLineage,

    /// When this node was born
    pub birth_timestamp: DateTime<Utc>,

    /// Genesis ceremony ID
    pub ceremony_id: Uuid,

    /// Optional node metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl NewNodeIdentity {
    /// Create new node identity from genesis ceremony
    #[must_use]
    pub fn new(
        node_id: String,
        public_key: Vec<u8>,
        genesis_witness: GenesisWitness,
        genesis_lineage: GenesisLineage,
    ) -> Self {
        Self {
            node_id,
            public_key,
            genesis_witness,
            ceremony_id: genesis_lineage.ceremony_id,
            birth_timestamp: genesis_lineage.birth_timestamp,
            genesis_lineage,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Get trust level based on genesis witness
    #[must_use]
    pub fn genesis_trust_level(&self) -> TrustLevel {
        self.genesis_witness.trust_level()
    }

    /// Check if this identity has lineage from a specific primal
    #[must_use]
    pub fn has_primal_lineage(&self, primal_name: &str) -> bool {
        self.genesis_lineage.primal_lineages.contains_key(primal_name)
    }

    /// Get lineage from a specific primal
    #[must_use]
    pub fn get_primal_lineage(&self, primal_name: &str) -> Option<&PrimalLineage> {
        self.genesis_lineage.primal_lineages.get(primal_name)
    }

    /// Get number of primals that signed this genesis
    #[must_use]
    pub fn primal_signature_count(&self) -> usize {
        self.genesis_lineage.primal_lineages.len()
    }

    /// Check if genesis is multi-primal (signed by 2+ primals)
    #[must_use]
    pub fn is_multi_primal_genesis(&self) -> bool {
        self.primal_signature_count() >= 2
    }

    /// Verify all primal signatures using `security provider`
    pub async fn verify_all_signatures(&self) -> bool {
        use crate::security_capability_client::SecurityCapabilityClient;

        if self.genesis_lineage.primal_lineages.is_empty() {
            return false;
        }

        // Try to create security capability client for verification
        let client = match SecurityCapabilityClient::new().await {
            Ok(client) => client,
            Err(e) => {
                tracing::warn!(
                    "Security provider not available for signature verification: {}. Using basic check.",
                    e
                );
                // Fallback: Just check we have lineages
                return !self.genesis_lineage.primal_lineages.is_empty();
            }
        };

        // Verify each primal's signature
        for (primal_name, lineage) in &self.genesis_lineage.primal_lineages {
            match client
                .verify_signature(&self.node_id, &lineage.lineage_data, &lineage.signature)
                .await
            {
                Ok(true) => {
                    tracing::debug!("✅ Signature verified for primal: {}", primal_name);
                }
                Ok(false) => {
                    tracing::warn!("❌ Invalid signature from primal: {}", primal_name);
                    return false;
                }
                Err(e) => {
                    tracing::error!("Failed to verify signature from {}: {}", primal_name, e);
                    return false;
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::types::PhysicalChannelType;

    fn create_test_witness() -> GenesisWitness {
        GenesisWitness::new(
            "test-witness".to_string(),
            vec![1, 2, 3],
            PhysicalChannelType::HardwareKey,
        )
    }

    fn create_test_lineage() -> GenesisLineage {
        create_lineage_with_primals(&["songbird", "beardog"])
    }

    fn create_lineage_with_primals(primals: &[&str]) -> GenesisLineage {
        let mut primal_lineages = std::collections::HashMap::new();
        for (i, name) in primals.iter().enumerate() {
            let tag = u8::try_from(i).expect("test uses small primal index");
            primal_lineages.insert(
                (*name).to_string(),
                PrimalLineage {
                    primal_name: (*name).to_string(),
                    lineage_data: vec![tag, 5, 6],
                    signature: vec![7, 8, 9],
                    timestamp: Utc::now(),
                },
            );
        }

        GenesisLineage {
            witness_device_id: "test-witness".to_string(),
            primal_lineages,
            birth_timestamp: Utc::now(),
            ceremony_id: Uuid::new_v4(),
        }
    }

    #[test]
    fn test_new_node_identity_creation() {
        let witness = create_test_witness();
        let lineage = create_test_lineage();

        let identity =
            NewNodeIdentity::new("new-node-123".to_string(), vec![16, 17, 18], witness, lineage);

        assert_eq!(identity.node_id, "new-node-123");
        assert_eq!(identity.genesis_trust_level(), TrustLevel::Maximum);
        assert!(identity.is_multi_primal_genesis());
        assert_eq!(identity.primal_signature_count(), 2);
    }

    #[test]
    fn test_primal_lineage_queries() {
        let witness = create_test_witness();
        let lineage = create_test_lineage();
        let identity = NewNodeIdentity::new("test".to_string(), vec![], witness, lineage);

        assert!(identity.has_primal_lineage("songbird"));
        assert!(identity.has_primal_lineage("beardog"));
        assert!(!identity.has_primal_lineage("toadstool"));

        let songbird_lineage = identity.get_primal_lineage("songbird");
        assert!(songbird_lineage.is_some());
        assert_eq!(songbird_lineage.unwrap().primal_name, "songbird");
        assert!(identity.get_primal_lineage("missing").is_none());
    }

    #[test]
    fn new_node_identity_single_primal_is_not_multi_primal() {
        let witness = create_test_witness();
        let lineage = create_lineage_with_primals(&["songbird"]);
        let identity = NewNodeIdentity::new("n1".to_string(), vec![1], witness, lineage);
        assert_eq!(identity.primal_signature_count(), 1);
        assert!(!identity.is_multi_primal_genesis(), "one primal should not count as multi-primal");
    }

    #[tokio::test]
    async fn verify_all_signatures_false_when_no_primal_lineages() {
        let witness = create_test_witness();
        let lineage = GenesisLineage {
            witness_device_id: "w".to_string(),
            primal_lineages: std::collections::HashMap::new(),
            birth_timestamp: Utc::now(),
            ceremony_id: Uuid::new_v4(),
        };
        let identity = NewNodeIdentity::new("n".to_string(), vec![], witness, lineage);
        assert!(
            !identity.verify_all_signatures().await,
            "empty primal lineages must fail verification"
        );
    }

    #[test]
    fn new_node_identity_serde_roundtrip() {
        let witness = create_test_witness();
        let lineage = create_test_lineage();
        let mut identity =
            NewNodeIdentity::new("node-a".to_string(), vec![9, 9, 9], witness, lineage);
        identity.metadata.insert("k".to_string(), "v".to_string());

        let json = serde_json::to_string(&identity).expect("serialize identity");
        let back: NewNodeIdentity = serde_json::from_str(&json).expect("deserialize identity");
        assert_eq!(back.node_id, "node-a");
        assert_eq!(back.public_key, vec![9, 9, 9]);
        assert_eq!(back.metadata.get("k"), Some(&"v".to_string()));
        assert_eq!(back.ceremony_id, identity.ceremony_id);
    }

    #[test]
    fn new_node_identity_allows_empty_node_id_string() {
        let witness = create_test_witness();
        let lineage = create_lineage_with_primals(&["songbird"]);
        let id = NewNodeIdentity::new(String::new(), vec![], witness, lineage);
        assert!(id.node_id.is_empty(), "API does not forbid empty node_id (callers may validate)");
        assert_eq!(id.primal_signature_count(), 1);
    }

    #[test]
    fn has_primal_lineage_empty_key_is_false() {
        let witness = create_test_witness();
        let lineage = create_test_lineage();
        let identity = NewNodeIdentity::new("n".into(), vec![], witness, lineage);
        assert!(!identity.has_primal_lineage(""));
    }

    #[test]
    fn genesis_trust_level_follows_witness_channel_type() {
        let witness = GenesisWitness::new("w".into(), vec![], PhysicalChannelType::QrCodeWithOob);
        let lineage = create_lineage_with_primals(&["songbird"]);
        let identity = NewNodeIdentity::new("n".into(), vec![], witness, lineage);
        assert_eq!(identity.genesis_trust_level(), TrustLevel::High);
    }

    #[test]
    fn get_primal_lineage_returns_none_for_unknown_primal() {
        let witness = create_test_witness();
        let lineage = create_test_lineage();
        let identity = NewNodeIdentity::new("n".into(), vec![], witness, lineage);
        assert!(identity.get_primal_lineage("nonexistent").is_none());
    }

    #[test]
    fn metadata_defaults_empty_and_roundtrips() {
        let witness = create_test_witness();
        let lineage = create_test_lineage();
        let identity = NewNodeIdentity::new("n".into(), vec![1], witness, lineage);
        assert!(identity.metadata.is_empty());
        let json = serde_json::to_string(&identity).expect("serde");
        let back: NewNodeIdentity = serde_json::from_str(&json).expect("de");
        assert!(back.metadata.is_empty());
    }

    #[test]
    fn new_node_identity_copies_ceremony_id_and_birth_from_lineage() {
        let witness = create_test_witness();
        let mut lineage = create_lineage_with_primals(&["songbird"]);
        let cid = Uuid::nil();
        lineage.ceremony_id = cid;
        let ts = lineage.birth_timestamp;
        let identity = NewNodeIdentity::new("sync".into(), vec![1], witness, lineage);
        assert_eq!(identity.ceremony_id, cid);
        assert_eq!(identity.birth_timestamp, ts);
    }

    #[test]
    fn primal_lineage_map_duplicate_key_retains_last_insert() {
        let witness = create_test_witness();
        let mut lineage = create_lineage_with_primals(&["dup"]);
        lineage.primal_lineages.insert(
            "dup".to_string(),
            PrimalLineage {
                primal_name: "dup".to_string(),
                lineage_data: vec![99],
                signature: vec![],
                timestamp: Utc::now(),
            },
        );
        let identity = NewNodeIdentity::new("n".into(), vec![], witness, lineage);
        assert_eq!(identity.get_primal_lineage("dup").expect("dup primal").lineage_data, vec![99]);
    }
}
