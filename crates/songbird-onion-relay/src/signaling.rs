// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

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

/// Rendezvous protocol messages (register, punch, relay, errors).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SignalingMessage {
    /// Register presence with rendezvous
    #[serde(rename = "register")]
    Register {
        /// Local peer metadata for the rendezvous directory.
        peer_info: PeerInfo,
        /// Encrypted with beacon seed (only family can read)
        encrypted_beacon: Option<String>,
    },

    /// Query for a specific peer
    #[serde(rename = "query")]
    Query {
        /// Node id whose [`PeerInfo`] should be returned.
        target_node_id: String,
    },

    /// Response to query with peer info
    #[serde(rename = "peer_info")]
    PeerInfoResponse {
        /// Matching peer when known, otherwise `None`.
        peer_info: Option<PeerInfo>,
    },

    /// Request to initiate hole punch
    #[serde(rename = "punch_request")]
    PunchRequest {
        /// Initiator metadata.
        from: PeerInfo,
        /// Intended responder node id.
        to_node_id: String,
        /// Nonce for this punch attempt
        nonce: [u8; 16],
    },

    /// Acknowledge punch request - start simultaneous open
    #[serde(rename = "punch_ack")]
    PunchAck {
        /// Responder metadata.
        from: PeerInfo,
        /// Echo of the punch nonce.
        nonce: [u8; 16],
        /// When to start punching (coordinated time)
        start_at_ms: u64,
    },

    /// Report punch result
    #[serde(rename = "punch_result")]
    PunchResult {
        /// Nonce tied to the punch attempt.
        nonce: [u8; 16],
        /// Whether UDP packets were observed on the expected path.
        success: bool,
        /// If successful, the working address
        connected_addr: Option<SocketAddr>,
    },

    /// Heartbeat to keep registration alive
    #[serde(rename = "heartbeat")]
    Heartbeat {
        /// Sender node id.
        node_id: String,
    },

    /// Relay data through rendezvous (fallback if punch fails)
    #[serde(rename = "relay")]
    RelayData {
        /// Source node id.
        from_node_id: String,
        /// Destination node id.
        to_node_id: String,
        /// Encrypted payload
        data: Vec<u8>,
    },

    /// Error response
    #[serde(rename = "error")]
    Error {
        /// Application-level error code.
        code: i32,
        /// Human-readable detail.
        message: String,
    },
}

impl SignalingMessage {
    /// Serializes this message to JSON.
    ///
    /// # Errors
    ///
    /// Returns [`serde_json::Error`] if serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Parses a [`SignalingMessage`] from JSON text.
    ///
    /// # Errors
    ///
    /// Returns [`serde_json::Error`] when the payload is not valid JSON or mismatches the schema.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl PeerInfo {
    /// Create new peer info from STUN discovery
    #[must_use]
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
    #[must_use]
    pub fn is_fresh(&self) -> bool {
        self.timestamp.elapsed().is_ok_and(|elapsed| elapsed.as_secs() < 60)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use std::time::SystemTime;

    #[test]
    fn peer_info_new_defaults_and_capabilities() {
        let peer = PeerInfo::new("tower-123".to_string(), "1.2.3.4:5678".parse().unwrap());
        assert_eq!(peer.node_id, "tower-123");
        assert_eq!(peer.public_addr.to_string(), "1.2.3.4:5678");
        assert!(peer.local_addr.is_none());
        assert_eq!(peer.nat_type, NatType::Unknown);
        assert!(peer.capabilities.contains(&"relay".to_string()));
        assert!(peer.is_fresh(), "new peer should be fresh");
    }

    #[test]
    fn peer_info_stale_timestamp_is_not_fresh() {
        let peer = PeerInfo {
            node_id: "x".into(),
            public_addr: "9.9.9.9:1".parse().unwrap(),
            local_addr: None,
            nat_type: NatType::Symmetric,
            timestamp: SystemTime::UNIX_EPOCH,
            capabilities: vec![],
        };
        assert!(!peer.is_fresh(), "epoch timestamp should not be fresh relative to now");
    }

    #[test]
    fn nat_type_copy_eq() {
        assert_eq!(NatType::FullCone, NatType::FullCone);
        assert_ne!(NatType::Symmetric, NatType::Unknown);
    }

    #[test]
    fn register_roundtrip_json() {
        let peer = PeerInfo::new("tower-123".to_string(), "1.2.3.4:5678".parse().unwrap());
        let msg = SignalingMessage::Register {
            peer_info: peer,
            encrypted_beacon: Some("encrypted_data".to_string()),
        };
        let json = msg.to_json().expect("serialize");
        let parsed = SignalingMessage::from_json(&json).expect("deserialize");
        match parsed {
            SignalingMessage::Register {
                peer_info,
                encrypted_beacon,
            } => {
                assert_eq!(peer_info.node_id, "tower-123");
                assert_eq!(encrypted_beacon.as_deref(), Some("encrypted_data"));
            }
            other => panic!("expected Register, got {other:?}"),
        }
    }

    #[test]
    fn query_and_peer_info_response_roundtrip() {
        let q = SignalingMessage::Query {
            target_node_id: "peer-a".into(),
        };
        let json = q.to_json().unwrap();
        assert!(json.contains("query"));
        let r = SignalingMessage::from_json(&json).unwrap();
        assert!(matches!(r, SignalingMessage::Query { .. }));

        let resp = SignalingMessage::PeerInfoResponse {
            peer_info: None,
        };
        let back = SignalingMessage::from_json(&resp.to_json().unwrap()).unwrap();
        assert!(matches!(
            back,
            SignalingMessage::PeerInfoResponse {
                peer_info: None
            }
        ));
    }

    #[test]
    fn punch_request_ack_result_heartbeat_relay_error_roundtrip() {
        let from = PeerInfo::new("a".into(), "1.1.1.1:1".parse().unwrap());
        let nonce = [9u8; 16];
        let pr = SignalingMessage::PunchRequest {
            from,
            to_node_id: "b".into(),
            nonce,
        };
        let pr_back = SignalingMessage::from_json(&pr.to_json().unwrap()).unwrap();
        assert!(matches!(pr_back, SignalingMessage::PunchRequest { .. }));

        let ack = SignalingMessage::PunchAck {
            from: PeerInfo::new("b".into(), "2.2.2.2:2".parse().unwrap()),
            nonce,
            start_at_ms: 1_700_000_000_000,
        };
        let ack_back = SignalingMessage::from_json(&ack.to_json().unwrap()).unwrap();
        assert!(matches!(ack_back, SignalingMessage::PunchAck { .. }));

        let res = SignalingMessage::PunchResult {
            nonce,
            success: true,
            connected_addr: Some("3.3.3.3:3".parse().unwrap()),
        };
        let res_back = SignalingMessage::from_json(&res.to_json().unwrap()).unwrap();
        assert!(matches!(res_back, SignalingMessage::PunchResult { .. }));

        let hb = SignalingMessage::Heartbeat {
            node_id: "h".into(),
        };
        assert!(matches!(
            SignalingMessage::from_json(&hb.to_json().unwrap()).unwrap(),
            SignalingMessage::Heartbeat { .. }
        ));

        let relay = SignalingMessage::RelayData {
            from_node_id: "f".into(),
            to_node_id: "t".into(),
            data: vec![1, 2, 3],
        };
        assert!(matches!(
            SignalingMessage::from_json(&relay.to_json().unwrap()).unwrap(),
            SignalingMessage::RelayData { .. }
        ));

        let err = SignalingMessage::Error {
            code: 42,
            message: "oops".into(),
        };
        let err_back = SignalingMessage::from_json(&err.to_json().unwrap()).unwrap();
        assert!(matches!(err_back, SignalingMessage::Error { .. }));
    }

    #[test]
    fn from_json_invalid_returns_error() {
        let r = SignalingMessage::from_json("not json");
        assert!(r.is_err(), "expected serde error for invalid JSON");
    }

    #[test]
    fn from_json_wrong_shape_returns_error() {
        let r = SignalingMessage::from_json(r#"{"type":"unknown_type"}"#);
        assert!(r.is_err(), "expected error for unknown tagged variant");
    }
}
