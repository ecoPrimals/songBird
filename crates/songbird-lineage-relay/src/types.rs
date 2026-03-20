// SPDX-License-Identifier: AGPL-3.0-only
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
    pub node_id: NodeId,
    pub parent: Option<NodeId>,
    pub ancestors: Vec<NodeId>,
    pub birth_signature: Vec<u8>, // Signed by BearDog
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
    pub relay_node: NodeId,
    pub requester: NodeId,
    pub authorized: bool,
    pub masking_level: MaskingLevel,
    pub ttl_seconds: u64,
    pub issued_at: SystemTime,
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
    pub authorized: bool,
    pub masking_level: MaskingLevel,
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
    pub node_id: NodeId,
    pub addresses: Vec<SocketAddr>,
    pub discovered_at: SystemTime,
}

/// Connection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionType {
    /// Direct connection (no relay)
    Direct,
    /// Relayed through ancestor
    Relayed,
    /// Attempting upgrade from relay to direct
    Upgrading,
}

/// Connection statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub established_at: Option<SystemTime>,
    pub connection_type: Option<ConnectionType>,
}
