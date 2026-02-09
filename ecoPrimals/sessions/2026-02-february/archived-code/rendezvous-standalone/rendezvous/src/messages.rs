//! Message types for rendezvous protocol
//!
//! Following specification in `specs/RENDEZVOUS_PROTOCOL_SPEC.md`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Message type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    RegisterPresence,
    Heartbeat,
    QueryPeers,
    RequestConnection,
    ResponseConnection,
    ConnectionEstablished,
}

/// Node identity (no IP address!)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIdentity {
    /// Stable node UUID
    pub node_id: String,

    /// Ephemeral session ID (rotates every 10-15min)
    pub ephemeral_session_id: String,

    /// Public key fingerprint (SHA-256 of public key)
    pub public_key_fingerprint: String,

    /// Node capabilities
    pub capabilities: Vec<String>,

    /// Supported protocols
    pub protocols: Vec<String>,
}

/// Network context (no IP!)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkContext {
    /// NAT type (cone, symmetric, open, unknown)
    pub nat_type: String,

    /// Reachability (direct, relayed, unknown)
    pub reachability: String,

    /// Connection quality (excellent, good, poor, unknown)
    pub connection_quality: String,
}

/// Security information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityInfo {
    /// Signature of entire message (BearDog)
    pub signature: Option<String>,
}

/// Message 1: Register Presence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPresenceMessage {
    pub message_type: MessageType,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub node_identity: NodeIdentity,
    pub network_context: NetworkContext,
    pub security: SecurityInfo,
}

/// Response to registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPresenceResponse {
    pub status: String,
    pub session_id: String,
    pub expires_at: DateTime<Utc>,
    pub rendezvous_endpoint: Option<String>,
}

/// Message 2: Heartbeat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<String>,
}

/// Message 3: Query Peers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPeersMessage {
    pub message_type: MessageType,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub requester: RequesterInfo,
    pub query: PeerQuery,
    pub filters: Option<QueryFilters>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequesterInfo {
    pub session_id: String,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerQuery {
    pub capabilities_required: Vec<String>,
    pub capabilities_optional: Vec<String>,
    pub exclude_node_ids: Vec<String>,
    pub max_results: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilters {
    pub connection_quality_min: Option<String>,
    pub prefer_direct_connections: Option<bool>,
}

/// Response to query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPeersResponse {
    pub peers: Vec<PeerInfo>,
    pub total_matches: usize,
    pub returned: usize,
}

/// Peer information (no IP!)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub ephemeral_session_id: String,
    pub public_key_fingerprint: String,
    pub capabilities: Vec<String>,
    pub protocols: Vec<String>,
    pub network_context: NetworkContext,
    pub last_heartbeat: DateTime<Utc>,
}

/// Message 4: Request Connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestConnectionMessage {
    pub message_type: MessageType,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub requester: RequesterInfo,
    pub target: TargetInfo,
    pub connection_intent: ConnectionIntent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionIntent {
    pub purpose: String,
    pub required_protocols: Vec<String>,
    /// ICE candidates encrypted with target's public key
    pub ice_candidates_encrypted: Option<Vec<String>>,
}

/// Response to connection request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestConnectionResponse {
    pub status: String,
    pub coordination_token: String,
    pub relay_endpoint: Option<String>,
}

/// Message 5: Connection Response (Accept/Reject)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseConnectionMessage {
    pub message_type: MessageType,
    pub version: String,
    pub timestamp: DateTime<Utc>,
    pub responder: RequesterInfo,
    pub coordination_token: String,
    pub decision: String, // "accept", "reject", "later"
    pub response_data: Option<ResponseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    pub ice_candidates_encrypted: Option<Vec<String>>,
    pub btsp_ready: bool,
    pub preferred_protocol: String,
}

/// Message 6: Connection Established
///
/// **Status**: Phase 4-5 - Confirms successful peer connection
#[allow(dead_code)] // Phase 4-5 implementation pending
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEstablishedMessage {
    pub message_type: MessageType,
    pub coordination_token: String,
    pub timestamp: DateTime<Utc>,
    pub connection_type: String, // "direct" or "relayed"
    pub signature: Option<String>,
}
