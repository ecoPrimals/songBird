//! Lineage-Gated Relay Trait
//!
//! BearDog provides relay services based on lineage.

use super::lineage::LineageProof;
use serde::{Deserialize, Serialize};

/// Lineage-gated relay provider
///
/// BearDog implements this to offer relay services to descendants.
#[async_trait::async_trait]
pub trait LineageRelay: Send + Sync {
    /// Offer relay service to descendant
    ///
    /// Verifies lineage proof, then creates a relay session.
    async fn offer_relay(
        &self,
        requester: &str,
        target: &str,
        lineage_proof: LineageProof,
    ) -> anyhow::Result<RelaySession>;

    /// Get visibility level based on lineage depth
    ///
    /// Determines how much metadata this node can see based on lineage.
    fn get_visibility_level(&self, lineage_depth: usize) -> AccessLevel;

    /// Relay packet (with masking enforced)
    ///
    /// Routes a packet between two nodes, enforcing metadata masking.
    async fn relay_packet(&self, session: &RelaySession, packet: &[u8]) -> anyhow::Result<()>;

    /// Revoke relay for a session
    ///
    /// Ancestor can revoke relay privileges at any time.
    async fn revoke_relay(&self, session_id: &str) -> anyhow::Result<()>;
}

/// A relay session between two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelaySession {
    /// Unique session identifier
    pub session_id: String,

    /// Node requesting relay
    pub requester_id: String,

    /// Target node to reach
    pub target_id: String,

    /// Relay node (ancestor)
    pub relay_id: String,

    /// Access level for this session (based on lineage depth)
    pub access_level: AccessLevel,

    /// When this session was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// When this session expires
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Access level for metadata visibility
///
/// Based on lineage depth between relay and nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLevel {
    /// Layer 0: Transport only (opaque)
    /// Can see: Packet size, timing
    /// Cannot see: Payload, keys, identities
    Transport,

    /// Layer 1: Masked identity (default for distant relatives)
    /// Can see: Ephemeral relay IDs
    /// Cannot see: Stable node IDs, topology
    Masked,

    /// Layer 2: Sub-mask access (lineage-gated)
    /// Can see: Stable node ID, network hints
    /// Selective metadata disclosure
    SubMasked,

    /// Layer 3: Full visibility (direct parent only)
    /// Can see: Everything
    /// Can audit, revoke, enforce policy
    FullLineage,
}

impl AccessLevel {
    /// Determine access level from lineage depth
    ///
    /// Visibility flows downward:
    /// - 0 (parent): Full visibility
    /// - 1-3 (close ancestor): Sub-masked
    /// - 4-10 (distant ancestor): Masked
    /// - 11+ (very distant): Transport only
    pub fn from_lineage_depth(depth: usize) -> Self {
        match depth {
            0 => Self::FullLineage,   // Direct parent
            1..=3 => Self::SubMasked, // Close ancestor
            4..=10 => Self::Masked,   // Distant ancestor
            _ => Self::Transport,     // Very distant or unrelated
        }
    }

    /// Check if this level can see stable node IDs
    pub fn can_see_node_id(&self) -> bool {
        matches!(self, Self::SubMasked | Self::FullLineage)
    }

    /// Check if this level can see network topology
    pub fn can_see_topology(&self) -> bool {
        matches!(self, Self::FullLineage)
    }

    /// Check if this level can revoke relay
    pub fn can_revoke(&self) -> bool {
        matches!(self, Self::FullLineage)
    }
}

impl RelaySession {
    /// Check if this session is currently active
    pub fn is_active(&self) -> bool {
        let now = chrono::Utc::now();
        now >= self.created_at && now < self.expires_at
    }

    /// Check if this session is expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() >= self.expires_at
    }

    /// Time remaining in this session
    pub fn time_remaining(&self) -> Option<chrono::Duration> {
        let now = chrono::Utc::now();
        if now >= self.expires_at {
            None
        } else {
            Some(self.expires_at - now)
        }
    }
}
