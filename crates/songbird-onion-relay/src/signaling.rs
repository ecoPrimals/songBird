//! Signaling Protocol for NAT Traversal
//!
//! Messages exchanged via rendezvous to coordinate hole punching.
//! Transport-agnostic: works over Tor, WebSocket, TCP, etc.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::SystemTime;

/// Peer information for hole punch coordination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer's node ID (from lineage)
    pub node_id: String,

    /// STUN-discovered public address
    pub public_addr: SocketAddr,

    /// Local address (for LAN fallback)
    pub local_addr: Option<SocketAddr>,

    /// NAT type if known
    pub nat_type: NatType,

    /// Timestamp for freshness
    pub timestamp: SystemTime,

    /// Capabilities this peer supports
    pub capabilities: Vec<String>,
}

/// Detected NAT type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NatType {
    /// Full cone - best case, hole punch almost always works
    FullCone,
    /// Address restricted cone - good success rate
    AddressRestricted,
    /// Port restricted cone - moderate success rate
    PortRestricted,
    /// Symmetric - lowest success rate (~30%)
    Symmetric,
    /// Unknown - treat as symmetric (worst case)
    Unknown,
}

/// Signaling messages exchanged via rendezvous
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SignalingMessage {
    /// Register presence with rendezvous
    #[serde(rename = "register")]
    Register {
        peer_info: PeerInfo,
        /// Encrypted with beacon seed (only family can read)
        encrypted_beacon: Option<String>,
    },

    /// Query for a specific peer
    #[serde(rename = "query")]
    Query {
        target_node_id: String,
    },

    /// Response to query with peer info
    #[serde(rename = "peer_info")]
    PeerInfoResponse {
        peer_info: Option<PeerInfo>,
    },

    /// Request to initiate hole punch
    #[serde(rename = "punch_request")]
    PunchRequest {
        from: PeerInfo,
        to_node_id: String,
        /// Nonce for this punch attempt
        nonce: [u8; 16],
    },

    /// Acknowledge punch request - start simultaneous open
    #[serde(rename = "punch_ack")]
    PunchAck {
        from: PeerInfo,
        nonce: [u8; 16],
        /// When to start punching (coordinated time)
        start_at_ms: u64,
    },

    /// Report punch result
    #[serde(rename = "punch_result")]
    PunchResult {
        nonce: [u8; 16],
        success: bool,
        /// If successful, the working address
        connected_addr: Option<SocketAddr>,
    },

    /// Heartbeat to keep registration alive
    #[serde(rename = "heartbeat")]
    Heartbeat {
        node_id: String,
    },

    /// Relay data through rendezvous (fallback if punch fails)
    #[serde(rename = "relay")]
    RelayData {
        from_node_id: String,
        to_node_id: String,
        /// Encrypted payload
        data: Vec<u8>,
    },

    /// Error response
    #[serde(rename = "error")]
    Error {
        code: i32,
        message: String,
    },
}

impl SignalingMessage {
    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl PeerInfo {
    /// Create new peer info from STUN discovery
    pub fn new(node_id: String, public_addr: SocketAddr) -> Self {
        Self {
            node_id,
            public_addr,
            local_addr: None,
            nat_type: NatType::Unknown,
            timestamp: SystemTime::now(),
            capabilities: vec!["relay".to_string(), "stun".to_string()],
        }
    }

    /// Check if peer info is fresh (within last 60 seconds)
    pub fn is_fresh(&self) -> bool {
        if let Ok(elapsed) = self.timestamp.elapsed() {
            elapsed.as_secs() < 60
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signaling_serialization() {
        let peer = PeerInfo::new("tower-123".to_string(), "1.2.3.4:5678".parse().unwrap());

        let msg = SignalingMessage::Register {
            peer_info: peer,
            encrypted_beacon: Some("encrypted_data".to_string()),
        };

        let json = msg.to_json().unwrap();
        let parsed = SignalingMessage::from_json(&json).unwrap();

        if let SignalingMessage::Register {
            peer_info,
            ..
        } = parsed
        {
            assert_eq!(peer_info.node_id, "tower-123");
        } else {
            panic!("Wrong message type");
        }
    }

    #[test]
    fn test_peer_freshness() {
        let peer = PeerInfo::new("test".to_string(), "1.1.1.1:1234".parse().unwrap());
        assert!(peer.is_fresh());
    }
}
