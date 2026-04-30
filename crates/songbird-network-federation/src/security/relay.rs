// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Lineage-Gated Relay Trait
//!
//! `security provider` provides relay services based on lineage.

use super::lineage::LineageProof;
use serde::{Deserialize, Serialize};

/// Lineage-gated relay provider
///
/// `security provider` implements this to offer relay services to descendants.
pub trait LineageRelay: Send + Sync {
    /// Offer relay service to descendant
    ///
    /// Verifies lineage proof, then creates a relay session.
    fn offer_relay(
        &self,
        requester: &str,
        target: &str,
        lineage_proof: LineageProof,
    ) -> impl std::future::Future<Output = anyhow::Result<RelaySession>> + Send;

    /// Get visibility level based on lineage depth
    ///
    /// Determines how much metadata this node can see based on lineage.
    fn get_visibility_level(&self, lineage_depth: usize) -> AccessLevel;

    /// Relay packet (with masking enforced)
    ///
    /// Routes a packet between two nodes, enforcing metadata masking.
    fn relay_packet(
        &self,
        session: &RelaySession,
        packet: &[u8],
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;

    /// Revoke relay for a session
    ///
    /// Ancestor can revoke relay privileges at any time.
    fn revoke_relay(
        &self,
        session_id: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
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
    #[must_use]
    pub const fn from_lineage_depth(depth: usize) -> Self {
        match depth {
            0 => Self::FullLineage,   // Direct parent
            1..=3 => Self::SubMasked, // Close ancestor
            4..=10 => Self::Masked,   // Distant ancestor
            _ => Self::Transport,     // Very distant or unrelated
        }
    }

    /// Check if this level can see stable node IDs
    #[must_use]
    pub const fn can_see_node_id(&self) -> bool {
        matches!(self, Self::SubMasked | Self::FullLineage)
    }

    /// Check if this level can see network topology
    #[must_use]
    pub const fn can_see_topology(&self) -> bool {
        matches!(self, Self::FullLineage)
    }

    /// Check if this level can revoke relay
    #[must_use]
    pub const fn can_revoke(&self) -> bool {
        matches!(self, Self::FullLineage)
    }
}

impl RelaySession {
    /// Check if this session is currently active
    #[must_use]
    pub fn is_active(&self) -> bool {
        let now = chrono::Utc::now();
        now >= self.created_at && now < self.expires_at
    }

    /// Check if this session is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() >= self.expires_at
    }

    /// Time remaining in this session
    #[must_use]
    pub fn time_remaining(&self) -> Option<chrono::Duration> {
        let now = chrono::Utc::now();
        if now >= self.expires_at {
            None
        } else {
            Some(self.expires_at - now)
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn access_level_from_lineage_depth_boundaries() {
        assert_eq!(AccessLevel::from_lineage_depth(0), AccessLevel::FullLineage);
        assert_eq!(AccessLevel::from_lineage_depth(1), AccessLevel::SubMasked);
        assert_eq!(AccessLevel::from_lineage_depth(3), AccessLevel::SubMasked);
        assert_eq!(AccessLevel::from_lineage_depth(4), AccessLevel::Masked);
        assert_eq!(AccessLevel::from_lineage_depth(10), AccessLevel::Masked);
        assert_eq!(AccessLevel::from_lineage_depth(11), AccessLevel::Transport);
        assert_eq!(AccessLevel::from_lineage_depth(usize::MAX), AccessLevel::Transport);
    }

    #[test]
    fn access_level_capability_flags_match_policy() {
        assert!(AccessLevel::SubMasked.can_see_node_id());
        assert!(AccessLevel::FullLineage.can_see_node_id());
        assert!(!AccessLevel::Masked.can_see_node_id());

        assert!(AccessLevel::FullLineage.can_see_topology());
        assert!(!AccessLevel::SubMasked.can_see_topology());

        assert!(AccessLevel::FullLineage.can_revoke());
        assert!(!AccessLevel::SubMasked.can_revoke());
    }

    #[test]
    fn relay_session_serde_roundtrip() {
        let created = chrono::Utc::now() - chrono::Duration::minutes(5);
        let expires = chrono::Utc::now() + chrono::Duration::hours(1);
        let session = RelaySession {
            session_id: "sid".into(),
            requester_id: "req".into(),
            target_id: "tgt".into(),
            relay_id: "rel".into(),
            access_level: AccessLevel::Masked,
            created_at: created,
            expires_at: expires,
        };
        let json = serde_json::to_string(&session).unwrap();
        let back: RelaySession = serde_json::from_str(&json).unwrap();
        assert_eq!(back.session_id, session.session_id);
        assert_eq!(back.access_level, AccessLevel::Masked);
        assert_eq!(back.created_at, created);
        assert_eq!(back.expires_at, expires);
    }

    #[test]
    fn relay_session_active_expiry_helpers_follow_wall_clock() {
        let now = chrono::Utc::now();
        let active = RelaySession {
            session_id: "a".into(),
            requester_id: "r".into(),
            target_id: "t".into(),
            relay_id: "l".into(),
            access_level: AccessLevel::Transport,
            created_at: now - chrono::Duration::minutes(1),
            expires_at: now + chrono::Duration::minutes(30),
        };
        assert!(active.is_active());
        assert!(!active.is_expired());
        assert!(active.time_remaining().is_some());

        let expired = RelaySession {
            expires_at: now - chrono::Duration::seconds(2),
            ..active.clone()
        };
        assert!(!expired.is_active());
        assert!(expired.is_expired());
        assert!(expired.time_remaining().is_none());

        let not_yet_started = RelaySession {
            created_at: now + chrono::Duration::minutes(1),
            expires_at: now + chrono::Duration::hours(1),
            ..active
        };
        assert!(!not_yet_started.is_active());
    }
}
