//! BTSP (`BirdSong` Transport Protocol) Types
//!
//! **VPN-Free P2P via Genetic Lineage**
//!
//! `BirdSong` enables secure P2P communication through genetic lineage trust,
//! eliminating the need for VPNs or centralized NAT traversal servers.
//!
//! ## Architecture
//!
//! ```text
//! Tower A (behind NAT) → Tower B (behind NAT)
//!           ↓
//!    Ask genetic lineage for contact info
//!           ↓
//!    Grandparent/Sibling provides contact exchange
//!           ↓
//!    Establish encrypted BTSP tunnel
//! ```
//!
//! ## Key Concepts
//!
//! - **Genetic Lineage**: Trust network based on cryptographic family relationships
//! - **Contact Exchange**: Lineage nodes help peers find each other (like asking family)
//! - **NAT Traversal**: Uses lineage for hole punching and relay
//! - **Encrypted Tunnels**: End-to-end encryption via security provider
//!
//! ## Zero Hardcoding
//!
//! - Discovers security provider via capabilities
//! - No hardcoded vendor names (`BearDog`, etc.)
//! - Runtime protocol negotiation
//! - Primal only knows itself

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::SystemTime;

/// BTSP tunnel connection handle
///
/// Represents an established encrypted tunnel to a remote peer.
/// The tunnel is managed by the security provider (discovered via capabilities).
#[derive(Debug, Clone)]
pub struct BtspTunnel {
    /// Unique tunnel identifier
    pub tunnel_id: String,

    /// Remote peer node ID
    pub remote_node_id: String,

    /// Tunnel endpoint (managed by security provider)
    pub endpoint: BtspEndpoint,

    /// Tunnel state
    pub state: TunnelState,

    /// When tunnel was established
    pub established_at: SystemTime,

    /// Last activity timestamp
    pub last_activity: SystemTime,
}

/// BTSP tunnel endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BtspEndpoint {
    /// Direct connection (no NAT)
    Direct {
        /// Remote address
        addr: SocketAddr,
    },

    /// NAT traversal via lineage relay
    Relayed {
        /// Relay node address (grandparent/sibling)
        relay_addr: SocketAddr,
        /// Relay node ID
        relay_node_id: String,
    },

    /// Hole-punched connection (via lineage coordination)
    HolePunched {
        /// Local endpoint
        local_addr: SocketAddr,
        /// Remote endpoint (discovered via lineage)
        remote_addr: SocketAddr,
    },
}

/// Tunnel state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TunnelState {
    /// Establishing tunnel (requesting from security provider)
    Establishing,

    /// Active and ready for communication
    Active,

    /// Idle but still connected
    Idle,

    /// Connection lost, attempting to reconnect
    Reconnecting,

    /// Tunnel closed
    Closed,
}

/// Request to establish BTSP tunnel
///
/// Sent to security provider to establish encrypted tunnel to remote peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtspTunnelRequest {
    /// Remote peer node ID
    pub remote_node_id: String,

    /// Remote peer's contact information (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_contact: Option<PeerContact>,

    /// Preferred tunnel type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_type: Option<TunnelType>,

    /// Request lineage assistance for NAT traversal
    #[serde(default)]
    pub use_lineage_for_nat: bool,

    /// Timeout for tunnel establishment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

/// Peer contact information
///
/// Exchanged via genetic lineage for NAT traversal.
/// Like asking a family member for someone's contact info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerContact {
    /// Peer's node ID
    pub node_id: String,

    /// Known addresses (may be behind NAT)
    pub addresses: Vec<SocketAddr>,

    /// Peer's genetic lineage ID (for trust verification)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage_id: Option<String>,

    /// Contact exchange path (which lineage nodes helped)
    #[serde(default)]
    pub exchange_path: Vec<String>,

    /// When contact info was obtained
    pub obtained_at: SystemTime,
}

/// Tunnel type preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TunnelType {
    /// Prefer direct connection (fastest)
    Direct,

    /// Prefer hole-punched (NAT traversal, no relay)
    HolePunched,

    /// Accept relay if needed (slowest but most reliable)
    Relayed,

    /// Try all methods, use fastest
    Auto,
}

/// Response from tunnel establishment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtspTunnelResponse {
    /// Tunnel ID (if successful)
    pub tunnel_id: Option<String>,

    /// Tunnel endpoint
    pub endpoint: Option<BtspEndpoint>,

    /// Success or failure
    pub success: bool,

    /// Human-readable message
    pub message: String,

    /// Additional metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// `BirdSong` contact exchange request
///
/// Request lineage assistance to find peer contact info.
/// Like asking a grandparent for a nephew's phone number.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactExchangeRequest {
    /// Target peer node ID
    pub target_node_id: String,

    /// Our node ID (for lineage verification)
    pub requester_node_id: String,

    /// Our lineage ID (for trust)
    pub requester_lineage_id: String,

    /// Maximum hops through lineage (default: 3)
    #[serde(default = "default_max_hops")]
    pub max_hops: u32,
}

const fn default_max_hops() -> u32 {
    3
}

/// Contact exchange response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactExchangeResponse {
    /// Peer contact info (if found)
    pub contact: Option<PeerContact>,

    /// Success or failure
    pub success: bool,

    /// Human-readable message
    pub message: String,

    /// Lineage path used to find contact
    #[serde(default)]
    pub lineage_path: Vec<String>,
}

impl BtspTunnel {
    /// Check if tunnel is active
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(self.state, TunnelState::Active)
    }

    /// Check if tunnel needs reconnection
    #[must_use]
    pub const fn needs_reconnect(&self) -> bool {
        matches!(self.state, TunnelState::Reconnecting | TunnelState::Closed)
    }

    /// Update last activity timestamp
    pub fn mark_activity(&mut self) {
        self.last_activity = SystemTime::now();
    }
}

impl BtspTunnelRequest {
    /// Create new tunnel request
    pub fn new(remote_node_id: impl Into<String>) -> Self {
        Self {
            remote_node_id: remote_node_id.into(),
            remote_contact: None,
            preferred_type: Some(TunnelType::Auto),
            use_lineage_for_nat: true,
            timeout_secs: Some(30),
        }
    }

    /// Set remote contact information
    #[must_use]
    pub fn with_contact(mut self, contact: PeerContact) -> Self {
        self.remote_contact = Some(contact);
        self
    }

    /// Set preferred tunnel type
    #[must_use]
    pub const fn with_tunnel_type(mut self, tunnel_type: TunnelType) -> Self {
        self.preferred_type = Some(tunnel_type);
        self
    }

    /// Disable lineage-based NAT traversal
    #[must_use]
    pub const fn without_lineage(mut self) -> Self {
        self.use_lineage_for_nat = false;
        self
    }
}

impl ContactExchangeRequest {
    /// Create new contact exchange request
    pub fn new(
        target_node_id: impl Into<String>,
        requester_node_id: impl Into<String>,
        requester_lineage_id: impl Into<String>,
    ) -> Self {
        Self {
            target_node_id: target_node_id.into(),
            requester_node_id: requester_node_id.into(),
            requester_lineage_id: requester_lineage_id.into(),
            max_hops: default_max_hops(),
        }
    }

    /// Set maximum hops through lineage
    #[must_use]
    pub const fn with_max_hops(mut self, max_hops: u32) -> Self {
        self.max_hops = max_hops;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btsp_tunnel_request_builder() {
        let request = BtspTunnelRequest::new("peer-123")
            .with_tunnel_type(TunnelType::HolePunched)
            .without_lineage();

        assert_eq!(request.remote_node_id, "peer-123");
        assert_eq!(request.preferred_type, Some(TunnelType::HolePunched));
        assert!(!request.use_lineage_for_nat);
    }

    #[test]
    fn test_contact_exchange_request() {
        let request = ContactExchangeRequest::new("target-456", "requester-789", "lineage-abc")
            .with_max_hops(5);

        assert_eq!(request.target_node_id, "target-456");
        assert_eq!(request.requester_node_id, "requester-789");
        assert_eq!(request.requester_lineage_id, "lineage-abc");
        assert_eq!(request.max_hops, 5);
    }

    #[test]
    fn test_tunnel_state_checks() {
        let mut tunnel = BtspTunnel {
            tunnel_id: "tunnel-1".to_string(),
            remote_node_id: "peer-1".to_string(),
            endpoint: BtspEndpoint::Direct {
                addr: "192.168.1.1:8080".parse().unwrap(),
            },
            state: TunnelState::Active,
            established_at: SystemTime::now(),
            last_activity: SystemTime::now(),
        };

        assert!(tunnel.is_active());
        assert!(!tunnel.needs_reconnect());

        tunnel.state = TunnelState::Closed;
        assert!(!tunnel.is_active());
        assert!(tunnel.needs_reconnect());
    }
}
