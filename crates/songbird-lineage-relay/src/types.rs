// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Core types for lineage relay system

use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::SocketAddr;
use std::time::SystemTime;

/// Node identifier (matches Genesis identity)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Cryptographic lineage proof (from Genesis)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageProof {
    /// Identity this proof describes.
    pub node_id: NodeId,
    /// Immediate parent in the lineage tree, if known.
    pub parent: Option<NodeId>,
    /// Ordered ancestor chain toward genesis (oldest last or per convention).
    pub ancestors: Vec<NodeId>,
    /// security provider-signed attestation binding this node to its birth event.
    pub birth_signature: Vec<u8>, // Signed by security provider
    /// When the birth certificate was issued.
    pub birth_timestamp: SystemTime,
}

/// Relationship between two nodes in lineage
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineageRelationship {
    /// Direct parent
    Parent,
    /// Ancestor (N generations up)
    Ancestor(u32),
    /// Direct child
    Child,
    /// Descendant (N generations down)
    Descendant(u32),
    /// Sibling (same parent)
    Sibling,
    /// Unrelated
    Unrelated,
}

/// Hint for which lineage members should receive message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineageHint {
    /// Only my direct parent
    DirectParent,
    /// All ancestors (parent, grandparent, etc.)
    DirectAncestors,
    /// My direct children
    DirectChildren,
    /// All descendants (children, grandchildren, etc.)
    AllDescendants,
    /// Specific ancestor by ID
    SpecificAncestor(NodeId),
}

/// `BirdSong` message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BirdSongType {
    /// Presence announcement
    Presence,
    /// Capability announcement
    CapabilityAnnouncement,
    /// Transport endpoint announcement
    TransportAnnouncement,
    /// Relay request (need help connecting)
    RelayRequest,
    /// Relay offer (can help you connect)
    RelayOffer,
    /// Federation event
    FederationEvent,
    /// Custom application message
    Custom(String),
}

/// `BirdSong` message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdSongMessage {
    /// Protocol version
    pub version: u8,
    /// Message type
    pub message_type: BirdSongType,
    /// Sender (encrypted for family)
    pub sender: NodeId,
    /// Lineage hint (who should receive)
    pub lineage_hint: LineageHint,
    /// Payload (encrypted by `security provider`)
    pub payload: Vec<u8>,
    /// Timestamp
    pub timestamp: u64,
}

/// Relay masking level (privacy control)
///
/// Determines privacy applied to relayed packets based on lineage relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MaskingLevel {
    /// No masking (direct family: parent ↔ child)
    None,
    /// Timing jitter only (close family: siblings)
    TimingOnly,
    /// Size obfuscation via padding (extended family)
    SizeObfuscation,
    /// Full privacy (distant family)
    Full,
    /// Minimal metadata visible (legacy default)
    #[default]
    Masked,
    /// Some metadata revealed (proven lineage - legacy)
    SubMasked,
    /// Full visibility (ancestor privilege - legacy)
    FullVisibility,
}

/// Relay authorization token (legacy format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayAuthorization {
    /// Relay that issued the token.
    pub relay_node: NodeId,
    /// Party allowed to send through the relay.
    pub requester: NodeId,
    /// Whether the policy granted relay use.
    pub authorized: bool,
    /// Privacy tier enforced for this allocation.
    pub masking_level: MaskingLevel,
    /// Validity window in seconds from [`issued_at`](Self::issued_at).
    pub ttl_seconds: u64,
    /// Token issuance time.
    pub issued_at: SystemTime,
    /// Correlates logs across relay, requester, and audits.
    pub audit_token: String,
}

impl RelayAuthorization {
    /// Create authorized relay token
    #[must_use]
    pub fn authorized(
        relay_node: NodeId,
        requester: NodeId,
        masking_level: MaskingLevel,
        ttl_seconds: u64,
    ) -> Self {
        Self {
            relay_node,
            requester,
            authorized: true,
            masking_level,
            ttl_seconds,
            issued_at: SystemTime::now(),
            audit_token: uuid::Uuid::new_v4().to_string(),
        }
    }

    /// Create unauthorized relay token
    #[must_use]
    pub fn unauthorized(relay_node: NodeId, requester: NodeId) -> Self {
        Self {
            relay_node,
            requester,
            authorized: false,
            masking_level: MaskingLevel::default(),
            ttl_seconds: 0,
            issued_at: SystemTime::now(),
            audit_token: String::new(),
        }
    }
}

/// Simple relay authorization result (for server use)
#[derive(Debug, Clone)]
pub struct SimpleRelayAuth {
    /// Whether relay forwarding is permitted.
    pub authorized: bool,
    /// Masking tier to apply on the wire.
    pub masking_level: MaskingLevel,
    /// Remaining lifetime of the grant.
    pub ttl: std::time::Duration,
}

impl From<RelayAuthorization> for SimpleRelayAuth {
    fn from(auth: RelayAuthorization) -> Self {
        Self {
            authorized: auth.authorized,
            masking_level: auth.masking_level,
            ttl: std::time::Duration::from_secs(auth.ttl_seconds),
        }
    }
}

/// Connection endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEndpoint {
    /// Remote [`NodeId`] this endpoint refers to.
    pub node_id: NodeId,
    /// Candidate addresses from discovery or ICE-like gathering.
    pub addresses: Vec<SocketAddr>,
    /// When these addresses were last observed.
    pub discovered_at: SystemTime,
}

/// Connection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    /// Direct connection (no relay)
    Direct,
    /// Relayed through ancestor
    Relayed,
    /// Relayed through sovereign TURN server (RFC 5766)
    TurnRelayed,
    /// Attempting upgrade from relay to direct
    Upgrading,
}

/// Connection statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectionStats {
    /// Application bytes written on this path.
    pub bytes_sent: u64,
    /// Application bytes read on this path.
    pub bytes_received: u64,
    /// Datagram or frame count sent (transport-dependent).
    pub packets_sent: u64,
    /// Datagram or frame count received.
    pub packets_received: u64,
    /// When the session entered the established state, if known.
    pub established_at: Option<SystemTime>,
    /// Whether traffic is direct, relayed, or upgrading.
    pub connection_type: Option<ConnectionType>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::{
        ConnectionEndpoint, ConnectionStats, ConnectionType, LineageProof, LineageRelationship,
        MaskingLevel, NodeId, RelayAuthorization, SimpleRelayAuth,
    };
    use std::time::{Duration, SystemTime};

    #[test]
    fn node_id_display_and_from_str() {
        let n: NodeId = "abc".into();
        assert_eq!(n.to_string(), "abc");
        let n2 = NodeId::from("x".to_string());
        assert_eq!(n2.0, "x");
    }

    #[test]
    fn lineage_relationship_serde_roundtrip() {
        let rel = LineageRelationship::Ancestor(3);
        let json = serde_json::to_string(&rel).expect("ser");
        let back: LineageRelationship = serde_json::from_str(&json).expect("de");
        assert_eq!(back, LineageRelationship::Ancestor(3));
    }

    #[test]
    fn masking_level_default_and_serde() {
        let m: MaskingLevel = MaskingLevel::default();
        let json = serde_json::to_string(&m).expect("ser");
        let _: MaskingLevel = serde_json::from_str(&json).expect("de");
    }

    #[test]
    fn relay_authorization_authorized_and_unauthorized() {
        let a = RelayAuthorization::authorized("r".into(), "q".into(), MaskingLevel::None, 60);
        assert!(a.authorized);
        let simple: SimpleRelayAuth = a.into();
        assert!(simple.authorized);
        assert_eq!(simple.ttl, Duration::from_secs(60));

        let u = RelayAuthorization::unauthorized("r".into(), "q".into());
        assert!(!u.authorized);
    }

    #[test]
    fn lineage_proof_serde_roundtrip() {
        let proof = LineageProof {
            node_id: "n".into(),
            parent: Some("p".into()),
            ancestors: vec!["a".into()],
            birth_signature: vec![1, 2],
            birth_timestamp: SystemTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&proof).expect("ser");
        let back: LineageProof = serde_json::from_str(&json).expect("de");
        assert_eq!(back.node_id.0, "n");
        assert_eq!(back.ancestors.len(), 1);
    }

    #[test]
    fn connection_stats_default() {
        let s = ConnectionStats::default();
        assert_eq!(s.bytes_sent, 0);
        assert!(s.connection_type.is_none());
    }

    #[test]
    fn connection_type_serde_roundtrip() {
        let t = ConnectionType::Relayed;
        let json = serde_json::to_string(&t).expect("ser");
        let back: ConnectionType = serde_json::from_str(&json).expect("de");
        assert_eq!(back, ConnectionType::Relayed);
    }

    #[test]
    fn lineage_relationship_all_variants_roundtrip_json() {
        for rel in [
            LineageRelationship::Parent,
            LineageRelationship::Ancestor(2),
            LineageRelationship::Child,
            LineageRelationship::Descendant(4),
            LineageRelationship::Sibling,
            LineageRelationship::Unrelated,
        ] {
            let json = serde_json::to_string(&rel).expect("ser");
            let back: LineageRelationship = serde_json::from_str(&json).expect("de");
            assert_eq!(back, rel);
        }
    }

    #[test]
    fn connection_endpoint_serde_roundtrip() {
        use std::net::SocketAddr;
        let ep = ConnectionEndpoint {
            node_id: NodeId::from("n"),
            addresses: vec!["192.0.2.1:9000".parse::<SocketAddr>().unwrap()],
            discovered_at: std::time::SystemTime::UNIX_EPOCH,
        };
        let json = serde_json::to_string(&ep).expect("ser");
        let back: ConnectionEndpoint = serde_json::from_str(&json).expect("de");
        assert_eq!(back.node_id, ep.node_id);
        assert_eq!(back.addresses, ep.addresses);
    }

    #[test]
    fn simple_relay_auth_from_unauthorized_token() {
        let u = RelayAuthorization::unauthorized("r".into(), "q".into());
        let s: SimpleRelayAuth = u.into();
        assert!(!s.authorized);
        assert_eq!(s.ttl, Duration::ZERO);
    }

    #[test]
    fn connection_stats_merge_manual_fields() {
        let mut s = ConnectionStats::default();
        s.bytes_sent = 10;
        s.packets_received = 3;
        s.connection_type = Some(ConnectionType::Upgrading);
        assert_eq!(s.bytes_sent, 10);
        assert!(matches!(s.connection_type, Some(ConnectionType::Upgrading)));
    }

    #[test]
    fn connection_type_upgrading_roundtrips_json() {
        let t = ConnectionType::Upgrading;
        let json = serde_json::to_string(&t).expect("ser");
        let back: ConnectionType = serde_json::from_str(&json).expect("de");
        assert_eq!(back, ConnectionType::Upgrading);
    }
}
