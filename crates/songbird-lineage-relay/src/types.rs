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
    pub birth_signature: Vec<u8>,  // Signed by BearDog
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaskingLevel {
    /// Minimal metadata visible (default)
    Masked,
    /// Some metadata revealed (proven lineage)
    SubMasked,
    /// Full visibility (ancestor privilege)
    FullVisibility,
}

impl Default for MaskingLevel {
    fn default() -> Self {
        Self::Masked
    }
}

/// Relay authorization token
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

