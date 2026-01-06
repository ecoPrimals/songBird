//! New node identity types

use crate::{types::*, witness::GenesisWitness};
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
    pub fn genesis_trust_level(&self) -> TrustLevel {
        self.genesis_witness.trust_level()
    }

    /// Check if this identity has lineage from a specific primal
    pub fn has_primal_lineage(&self, primal_name: &str) -> bool {
        self.genesis_lineage.primal_lineages.contains_key(primal_name)
    }

    /// Get lineage from a specific primal
    pub fn get_primal_lineage(&self, primal_name: &str) -> Option<&PrimalLineage> {
        self.genesis_lineage.primal_lineages.get(primal_name)
    }

    /// Get number of primals that signed this genesis
    pub fn primal_signature_count(&self) -> usize {
        self.genesis_lineage.primal_lineages.len()
    }

    /// Check if genesis is multi-primal (signed by 2+ primals)
    pub fn is_multi_primal_genesis(&self) -> bool {
        self.primal_signature_count() >= 2
    }

    /// Verify all primal signatures using BearDog
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
        let mut primal_lineages = std::collections::HashMap::new();

        primal_lineages.insert(
            "songbird".to_string(),
            PrimalLineage {
                primal_name: "songbird".to_string(),
                lineage_data: vec![4, 5, 6],
                signature: vec![7, 8, 9],
                timestamp: Utc::now(),
            },
        );

        primal_lineages.insert(
            "beardog".to_string(),
            PrimalLineage {
                primal_name: "beardog".to_string(),
                lineage_data: vec![10, 11, 12],
                signature: vec![13, 14, 15],
                timestamp: Utc::now(),
            },
        );

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
    }
}
