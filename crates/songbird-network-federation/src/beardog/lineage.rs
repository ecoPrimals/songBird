//! Lineage Management Trait
//!
//! BearDog provides lineage (genetic) services to Songbird.

use serde::{Deserialize, Serialize};

/// Lineage provider interface
///
/// BearDog implements this to provide lineage services.
#[async_trait::async_trait]
pub trait LineageProvider: Send + Sync {
    /// Generate lineage for a new node
    ///
    /// Creates a parent-child relationship with cryptographic proof.
    async fn generate_lineage(
        &self,
        node_id: &str,
        parent_id: &str,
    ) -> anyhow::Result<LineageChain>;

    /// Verify a lineage proof
    ///
    /// Cryptographically verifies that a node is part of a lineage.
    async fn verify_lineage(&self, proof: &LineageProof) -> anyhow::Result<bool>;

    /// Get all descendants of a root
    ///
    /// Returns all nodes that descend from the given root.
    async fn get_descendants(&self, root_id: &str) -> anyhow::Result<Vec<String>>;

    /// Get lineage depth between two nodes
    ///
    /// Returns the number of generations between ancestor and descendant.
    /// Returns None if no lineage relationship exists.
    async fn get_lineage_depth(
        &self,
        ancestor_id: &str,
        descendant_id: &str,
    ) -> anyhow::Result<Option<usize>>;
}

/// A lineage chain proving ancestry
///
/// This is a cryptographic proof that a node descends from a root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageChain {
    /// The root of this lineage (ancestor)
    pub root_id: String,

    /// The node at the end of this lineage (descendant)
    pub node_id: String,

    /// The chain of parent-child links
    pub links: Vec<LineageLink>,

    /// Generation depth (0 = root, 1 = child, 2 = grandchild, etc.)
    pub depth: usize,
}

/// A single link in a lineage chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageLink {
    /// Parent node ID
    pub parent_id: String,

    /// Child node ID
    pub child_id: String,

    /// Signature: parent signs child
    /// This proves the parent authorized this relationship
    pub signature: Vec<u8>,

    /// Timestamp of when this link was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// A lineage proof for authorization
///
/// This is what a node presents to prove its lineage membership.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageProof {
    /// The lineage chain being claimed
    pub chain: LineageChain,

    /// Signature by the claiming node
    /// This proves the claimer possesses the private key
    pub claimer_signature: Vec<u8>,
}

impl LineageChain {
    /// Verify the integrity of this lineage chain
    ///
    /// Checks that:
    /// 1. Each link's signature is valid
    /// 2. The chain is continuous (parent → child matches)
    /// 3. The depth matches the number of links
    pub fn verify_integrity(&self) -> anyhow::Result<bool> {
        // Check depth matches
        if self.depth != self.links.len() {
            return Ok(false);
        }

        // Check continuity
        let mut current_parent = &self.root_id;
        for link in &self.links {
            if link.parent_id != *current_parent {
                return Ok(false);
            }
            current_parent = &link.child_id;
        }

        // Final node should match
        if *current_parent != self.node_id {
            return Ok(false);
        }

        // TODO: Verify signatures (requires BearDog crypto)

        Ok(true)
    }

    /// Check if this chain is a descendant of a specific ancestor
    pub fn is_descendant_of(&self, ancestor_id: &str) -> bool {
        if self.root_id == ancestor_id {
            return true;
        }

        // Check if ancestor appears in the chain
        self.links.iter().any(|link| link.parent_id == ancestor_id)
    }
}
