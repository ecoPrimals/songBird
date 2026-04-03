// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Discovered Peer Management
//!
//! This module contains the peer discovery data structure and management logic.
//!
//! ## Contents
//! - `DiscoveredPeer` - Represents a discovered peer with all metadata
//! - Peer endpoint generation
//! - Peer comparison and identity

use std::net::SocketAddr;
use std::time::SystemTime;

use super::messages::TransportEndpointMessage;

/// Discovered peer information
///
/// Represents a peer discovered via UDP multicast with all its metadata.
/// Supports both v2.x (session-based) and v3.0 (node-identity-based) protocols.
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    /// Session ID of the peer (v2.x - deprecated in v3.0)
    pub session_id: String,

    /// Stable node ID (v3.0+) - machine-based UUID for identity coalescence
    pub node_id: Option<String>,

    /// Human-readable node name (v3.0+) - e.g., "eastgate", "westgate"
    pub node_name: Option<String>,

    /// All transport endpoints for this node (v3.0+)
    pub endpoints: Option<Vec<TransportEndpointMessage>>,

    /// Capabilities offered by the peer
    pub capabilities: Vec<String>,

    /// Generic tags (NEW - for USB seed integration)
    /// Contains `security provider` encryption tags for genetic lineage verification
    pub tags: Option<Vec<String>>,

    /// Discovery timestamp (NEW - for USB seed integration)
    /// Unix timestamp when this discovery message was sent
    pub timestamp: Option<u64>,

    /// Identity attestations (CRITICAL FIX - Jan 3, 2026)
    /// Structured identity information from security providers for genetic lineage auto-trust
    pub identity_attestations: Option<Vec<crate::IdentityAttestation>>,

    /// Supported protocols
    pub protocols: Vec<String>,

    /// Port where the peer's HTTPS/TLS server is listening (v2.x)
    pub port: u16,

    /// Socket address where the discovery message came from (UDP source)
    pub address: SocketAddr,

    /// When this peer was last seen
    pub last_seen: SystemTime,

    /// Discovery message version
    pub version: String,
}

impl DiscoveredPeer {
    /// Get the HTTPS endpoint for this peer
    ///
    /// Combines the source IP (from UDP) with the advertised HTTPS port.
    /// This is the primary connection endpoint for v2.x peers.
    #[must_use]
    pub fn https_endpoint(&self) -> String {
        format!("https://{}:{}", self.address.ip(), self.port)
    }

    /// Get the primary endpoint for this peer (v3.0+)
    ///
    /// Returns the highest-preference endpoint, or falls back to HTTPS endpoint.
    #[must_use]
    pub fn primary_endpoint(&self) -> String {
        if let Some(ref endpoints) = self.endpoints
            && let Some(primary) = endpoints.iter().max_by_key(|e| e.preference)
        {
            return primary.address.clone();
        }
        self.https_endpoint()
    }

    /// Check if this peer matches another peer (same identity)
    ///
    /// For v3.0: Compares `node_id`
    /// For v2.x: Compares `session_id` (less reliable)
    #[must_use]
    pub fn is_same_peer(&self, other: &Self) -> bool {
        if let (Some(my_id), Some(other_id)) = (&self.node_id, &other.node_id) {
            // v3.0: Use stable node_id
            my_id == other_id
        } else {
            // v2.x fallback: Use session_id (less reliable)
            self.session_id == other.session_id
        }
    }

    /// Check if this peer has a specific capability
    #[must_use]
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    /// Check if this peer supports a specific protocol
    #[must_use]
    pub fn supports_protocol(&self, protocol: &str) -> bool {
        self.protocols.iter().any(|p| p == protocol)
    }

    /// Get age of this peer information (seconds since `last_seen`)
    ///
    /// Returns None if time calculation fails
    #[must_use]
    pub fn age_secs(&self) -> Option<u64> {
        SystemTime::now().duration_since(self.last_seen).ok().map(|d| d.as_secs())
    }

    /// Check if this peer information is stale (older than TTL)
    ///
    /// Default TTL: 300 seconds (5 minutes)
    #[must_use]
    pub fn is_stale(&self, ttl_secs: u64) -> bool {
        self.age_secs().is_none_or(|age| age > ttl_secs)
    }

    /// Update `last_seen` timestamp to now
    pub fn touch(&mut self) {
        self.last_seen = SystemTime::now();
    }

    /// Get display name for this peer
    ///
    /// Prefers `node_name` (v3.0), falls back to `session_id`
    #[must_use]
    pub fn display_name(&self) -> &str {
        self.node_name.as_deref().unwrap_or(self.session_id.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_peer_https_endpoint() {
        let peer = DiscoveredPeer {
            session_id: "test-session".to_string(),
            node_id: None,
            node_name: None,
            endpoints: None,
            capabilities: vec!["orchestration".to_string()],
            tags: None,
            timestamp: None,
            identity_attestations: None,
            protocols: vec!["https".to_string()],
            port: 8080,
            address: "192.168.1.100:5353".parse().unwrap(),
            last_seen: SystemTime::now(),
            version: "2.1".to_string(),
        };

        assert_eq!(peer.https_endpoint(), "https://192.168.1.100:8080");
    }

    #[test]
    fn test_peer_is_same_v3() {
        let peer1 = DiscoveredPeer {
            session_id: "session1".to_string(),
            node_id: Some("node-abc".to_string()),
            node_name: Some("testnode".to_string()),
            endpoints: None,
            capabilities: vec![],
            tags: None,
            timestamp: None,
            identity_attestations: None,
            protocols: vec![],
            port: 8080,
            address: "192.168.1.100:5353".parse().unwrap(),
            last_seen: SystemTime::now(),
            version: "3.0".to_string(),
        };

        let mut peer2 = peer1.clone();
        peer2.session_id = "different-session".to_string(); // Different session

        // Should match because node_id is same
        assert!(peer1.is_same_peer(&peer2));
    }

    #[test]
    fn test_peer_is_same_v2() {
        let peer1 = DiscoveredPeer {
            session_id: "session1".to_string(),
            node_id: None, // v2.x - no node_id
            node_name: None,
            endpoints: None,
            capabilities: vec![],
            tags: None,
            timestamp: None,
            identity_attestations: None,
            protocols: vec![],
            port: 8080,
            address: "192.168.1.100:5353".parse().unwrap(),
            last_seen: SystemTime::now(),
            version: "2.1".to_string(),
        };

        let mut peer2 = peer1.clone();
        peer2.session_id = "different-session".to_string();

        // Should NOT match because session_id is different
        assert!(!peer1.is_same_peer(&peer2));
    }

    #[test]
    fn test_peer_capabilities() {
        let peer = DiscoveredPeer {
            session_id: "test".to_string(),
            node_id: None,
            node_name: None,
            endpoints: None,
            capabilities: vec!["orchestration".to_string(), "storage".to_string()],
            tags: None,
            timestamp: None,
            identity_attestations: None,
            protocols: vec![],
            port: 8080,
            address: "192.168.1.100:5353".parse().unwrap(),
            last_seen: SystemTime::now(),
            version: "2.1".to_string(),
        };

        assert!(peer.has_capability("orchestration"));
        assert!(peer.has_capability("storage"));
        assert!(!peer.has_capability("compute"));
    }

    #[test]
    fn test_peer_staleness() {
        let mut peer = DiscoveredPeer {
            session_id: "test".to_string(),
            node_id: None,
            node_name: None,
            endpoints: None,
            capabilities: vec![],
            tags: None,
            timestamp: None,
            identity_attestations: None,
            protocols: vec![],
            port: 8080,
            address: "192.168.1.100:5353".parse().unwrap(),
            last_seen: SystemTime::now() - Duration::from_secs(400), // 400 seconds ago
            version: "2.1".to_string(),
        };

        // Should be stale (older than 300 seconds)
        assert!(peer.is_stale(300));

        // Touch to update
        peer.touch();

        // Should not be stale now
        assert!(!peer.is_stale(300));
    }

    #[test]
    fn test_peer_display_name() {
        // v3.0 peer with node_name
        let peer_v3 = DiscoveredPeer {
            session_id: "session-abc".to_string(),
            node_id: Some("node-123".to_string()),
            node_name: Some("eastgate".to_string()),
            endpoints: None,
            capabilities: vec![],
            tags: None,
            timestamp: None,
            identity_attestations: None,
            protocols: vec![],
            port: 8080,
            address: "192.168.1.100:5353".parse().unwrap(),
            last_seen: SystemTime::now(),
            version: "3.0".to_string(),
        };

        assert_eq!(peer_v3.display_name(), "eastgate");

        // v2.x peer without node_name
        let peer_v2 = DiscoveredPeer {
            session_id: "session-xyz".to_string(),
            node_id: None,
            node_name: None,
            endpoints: None,
            capabilities: vec![],
            tags: None,
            timestamp: None,
            identity_attestations: None,
            protocols: vec![],
            port: 8080,
            address: "192.168.1.100:5353".parse().unwrap(),
            last_seen: SystemTime::now(),
            version: "2.1".to_string(),
        };

        assert_eq!(peer_v2.display_name(), "session-xyz");
    }
}
