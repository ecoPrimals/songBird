// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `BirdSong` Payload Structures
//!
//! Data structures for encrypted discovery broadcasts

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Discovery payload for birdSong mode
///
/// This is the decrypted payload that family members can read
/// after verifying lineage and decrypting the broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirdSongPayload {
    /// Protocol version
    pub version: String,

    /// Stable node ID
    pub node_id: String,

    /// Node name (human-readable)
    pub node_name: String,

    /// Available transport endpoints
    pub transports: Vec<TransportEndpoint>,

    /// Advertised capabilities
    pub capabilities: Vec<String>,

    /// Timestamp (for replay protection)
    pub timestamp: u64,

    /// Session ID (ephemeral, rotates frequently)
    pub session_id: String,
}

/// Transport endpoint in birdSong payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportEndpoint {
    /// Interface type (ethernet, wifi, cellular, etc.)
    pub interface_type: String,

    /// Address (IP:port)
    pub address: String,

    /// Supported protocols
    pub protocols: Vec<String>,

    /// Preference (higher = more preferred)
    pub preference: u8,
}

impl BirdSongPayload {
    /// Create a new payload
    #[must_use]
    pub fn new(
        node_id: String,
        node_name: String,
        transports: Vec<TransportEndpoint>,
        capabilities: Vec<String>,
        session_id: String,
    ) -> Self {
        Self {
            version: String::from("3.1"),
            node_id,
            node_name,
            transports,
            capabilities,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            session_id,
        }
    }

    /// Check if timestamp is recent (within 60 seconds)
    #[must_use]
    pub fn is_fresh(&self) -> bool {
        let now =
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs();

        now.saturating_sub(self.timestamp) < 60
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Plaintext discovery payload
///
/// Used when `security provider` is not available
/// WARNING: Everything is visible to network observers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaintextPayload {
    /// Protocol version
    pub version: String,

    /// Stable node ID
    pub node_id: String,

    /// Node name (human-readable)
    pub node_name: String,

    /// Available transport endpoints
    pub transports: Vec<TransportEndpoint>,

    /// Advertised capabilities
    pub capabilities: Vec<String>,

    /// Session ID (ephemeral)
    pub session_id: String,
}

impl PlaintextPayload {
    /// Create a new plaintext payload
    #[must_use]
    pub fn new(
        node_id: String,
        node_name: String,
        transports: Vec<TransportEndpoint>,
        capabilities: Vec<String>,
        session_id: String,
    ) -> Self {
        Self {
            version: String::from("3.1"),
            node_id,
            node_name,
            transports,
            capabilities,
            session_id,
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Convert to `BirdSong` payload (for encryption)
    #[must_use]
    pub fn to_birdsong(&self) -> BirdSongPayload {
        BirdSongPayload::new(
            self.node_id.clone(),
            self.node_name.clone(),
            self.transports.clone(),
            self.capabilities.clone(),
            self.session_id.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_birdsong_payload_fresh() {
        let payload = BirdSongPayload::new(
            String::from("node-123"),
            String::from("test-node"),
            vec![],
            vec![],
            String::from("session-456"),
        );

        assert!(payload.is_fresh());
    }

    #[test]
    fn test_birdsong_payload_json() {
        let payload = BirdSongPayload::new(
            String::from("node-123"),
            String::from("test-node"),
            vec![],
            vec![String::from("compute")],
            String::from("session-456"),
        );

        let json = payload.to_json().unwrap();
        let decoded = BirdSongPayload::from_json(&json).unwrap();

        assert_eq!(decoded.node_id, "node-123");
        assert_eq!(decoded.node_name, "test-node");
        assert_eq!(decoded.capabilities.len(), 1);
    }

    #[test]
    fn test_plaintext_to_birdsong() {
        let plaintext = PlaintextPayload::new(
            String::from("node-789"),
            String::from("plain-node"),
            vec![],
            vec![String::from("storage")],
            String::from("session-012"),
        );

        let birdsong = plaintext.to_birdsong();

        assert_eq!(birdsong.node_id, "node-789");
        assert_eq!(birdsong.node_name, "plain-node");
        assert_eq!(birdsong.capabilities, vec![String::from("storage")]);
    }
}
