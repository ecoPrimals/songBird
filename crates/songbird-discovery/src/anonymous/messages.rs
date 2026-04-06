// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Anonymous Discovery Message Types
//!
//! This module contains the data structures and serialization logic for
//! anonymous discovery messages broadcast via UDP multicast.
//!
//! ## Contents
//! - `AnonymousDiscoveryMessage` - Main discovery message structure
//! - `TransportEndpointMessage` - Network endpoint information (v3.0+)
//! - Session ID generation (rotating, privacy-preserving)
//! - Message validation logic

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Anonymous discovery message (v2.1 and v3.0)
///
/// This message is broadcast over UDP to discover other Songbird towers.
/// It contains NO identity information - only capabilities and connection info.
///
/// ## What's Shared (Anonymous):
/// - Capabilities (what can be done)
/// - Protocols (how to connect)
/// - Port (where to connect)
/// - Session ID (temporary, rotates hourly)
///
/// ## What's NOT Shared (Private):
/// - Hostname
/// - Node ID (except in v3.0 for interface coalescence)
/// - Internal topology
/// - User data
///
/// The IP address is inherently revealed by UDP (sender address), but we don't
/// include it in the message to avoid redundancy and maintain protocol purity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymousDiscoveryMessage {
    /// Protocol version (now "2.1" for connection-aware discovery, or "3.0" for multi-endpoint)
    pub version: String,

    /// Stable node ID (v3.0+) - allows interface coalescence
    ///
    /// In v3.0, this is the stable machine-based UUID.
    /// Receivers can group multiple endpoints under same `node_id`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,

    /// Human-readable node name (v3.0+)
    ///
    /// Example: "eastgate", "westgate", "strandgate"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,

    /// Temporary session ID (v2.x) - rotates every hour
    ///
    /// This prevents tracking across sessions while allowing response correlation.
    /// In v3.0, this is deprecated in favor of `node_id`, but still included for compatibility.
    pub session_id: String,

    /// All transport endpoints for this node (v3.0+)
    ///
    /// Each endpoint represents a different network interface (Ethernet, `WiFi`, etc.)
    /// with its own address and capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<Vec<TransportEndpointMessage>>,

    /// Capabilities offered by this tower
    ///
    /// Examples: "orchestration", "gpu-compute", "storage", "ml-inference"
    pub capabilities: Vec<String>,

    /// Supported protocols for communication
    ///
    /// Examples: "https", "tarpc-tls", "websocket-tls"
    pub protocols: Vec<String>,

    /// Port where this tower's HTTPS/TLS server is listening (v2.x)
    ///
    /// Combined with the UDP sender's IP address, this allows peers to connect.
    /// This is NOT considered identity information - it's connection metadata.
    /// In v3.0, this is deprecated in favor of endpoints array.
    pub port: u16,

    /// Timestamp of message creation (Unix epoch seconds)
    pub timestamp: u64,

    /// Generic tags (NEW - for USB seed integration)
    ///
    /// Contains `security provider` encryption tags and other metadata for trust evaluation.
    /// Songbird doesn't parse these - just passes them to the security provider.
    ///
    /// Examples:
    /// - `security provider` lineage: `"beardog:family:a3f2:tower1"`
    /// - Protocol support: `"btsp_enabled"`, `"birdsong_v2"`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,

    /// Identity attestations (CRITICAL FIX - Jan 3, 2026)
    ///
    /// Structured identity information from security providers (e.g., `security provider`, `ToadStool`).
    /// Enables genetic lineage auto-trust and provider-agnostic authentication.
    ///
    /// MUST be included for federation to work with genetic lineage!
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_attestations: Option<Vec<crate::IdentityAttestation>>,

    /// Optional: Cryptographic proof of capabilities
    ///
    /// This can be used to verify that the tower actually has the claimed capabilities.
    /// For now, this is optional and can be added later for enhanced security.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_proof: Option<String>,
}

/// Transport endpoint in discovery message (v3.0+)
///
/// CRITICAL EVOLUTION (Dec 20, 2025): Changed from "port" to "address" (IP:port)
/// to enable proper multi-interface coalescence. Without the full address, receivers
/// couldn't distinguish between interfaces on the same machine vs different machines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportEndpointMessage {
    /// Interface type (e.g., "ethernet", "wifi", "bluetooth")
    pub interface_type: String,

    /// Full network address for this endpoint (IP:port format)
    ///
    /// This allows receivers to properly coalesce multiple interfaces under a
    /// single node identity based on the stable `node_id`.
    pub address: String,

    /// Protocols supported on this endpoint
    pub protocols: Vec<String>,

    /// Relative preference (0-255, higher = more preferred)
    pub preference: u8,
}

impl AnonymousDiscoveryMessage {
    /// Create a new anonymous discovery message (v2.1 - backward compatible)
    #[must_use]
    pub fn new(capabilities: Vec<String>, protocols: Vec<String>, port: u16) -> Self {
        Self {
            version: "2.1".to_string(),
            node_id: None,
            node_name: None,
            session_id: Self::generate_session_id(),
            endpoints: None,
            capabilities,
            protocols,
            port,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            tags: None, // Will be populated by security provider if configured
            identity_attestations: None, // No attestations in v2.1 (legacy)
            capability_proof: None,
        }
    }

    /// Create a new multi-endpoint discovery message (v3.0)
    ///
    /// This includes stable node identity and multiple transport endpoints.
    /// Receivers can coalesce multiple endpoints under the same `node_id`.
    #[must_use]
    pub fn new_v3(
        node_id: impl Into<String>,
        node_name: impl Into<String>,
        endpoints: Vec<TransportEndpointMessage>,
        capabilities: Vec<String>,
    ) -> Self {
        let node_id = node_id.into();
        let node_name = node_name.into();

        // Get primary endpoint for backward compatibility
        let primary_endpoint = endpoints.first();

        // Extract port from address (format: "IP:port")
        let port = primary_endpoint
            .and_then(|e| e.address.split(':').nth(1))
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        let protocols =
            primary_endpoint.map_or_else(|| vec!["https".to_string()], |e| e.protocols.clone());

        let session_id = Self::generate_session_id_from_node(&node_id);
        Self {
            version: "3.0".to_string(),
            node_id: Some(node_id),
            node_name: Some(node_name),
            session_id,
            endpoints: Some(endpoints),
            capabilities,
            protocols,
            port,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
            tags: None, // Will be populated by security provider if configured
            identity_attestations: None, // Will be populated by security provider if configured
            capability_proof: None,
        }
    }

    /// Set identity attestations (CRITICAL FIX - Jan 3, 2026)
    ///
    /// Adds identity attestations from security provider for genetic lineage auto-trust.
    #[must_use]
    /// Set identity tags (v3.14.0 - tag-based identity)
    ///
    /// Tags are opaque strings we broadcast. We don't interpret them!
    /// Format: `{provider}:{type}:{value}`
    /// Example: `crypto:family:my-family`
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = if tags.is_empty() {
            None
        } else {
            Some(tags)
        };
        self
    }

    #[must_use]
    pub fn with_identity_attestations(
        mut self,
        attestations: Vec<crate::IdentityAttestation>,
    ) -> Self {
        self.identity_attestations = Some(attestations);
        self
    }

    /// Generate a rotating session ID
    ///
    /// Session IDs are based on:
    /// - Current hour (rotates every hour)
    /// - Random UUID (prevents collisions)
    ///
    /// This allows correlation of responses within an hour while preventing long-term tracking.
    fn generate_session_id() -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

        let hour = now / 3600;

        let uuid = Uuid::new_v4();

        let mut buf = Vec::with_capacity(8 + 16);
        buf.extend_from_slice(&hour.to_le_bytes());
        buf.extend_from_slice(uuid.as_bytes());
        let digest = crate::crypto_helpers::sha256_hash_sync(None, &buf);
        hex::encode(digest)
    }

    /// Generate a session ID from stable node ID (v3.0+)
    ///
    /// This creates a deterministic but rotating session ID based on:
    /// - Stable node ID (for consistency within an hour)
    /// - Current hour (for rotation)
    fn generate_session_id_from_node(node_id: &str) -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

        let hour = now / 3600;

        let mut buf = Vec::with_capacity(node_id.len() + 8);
        buf.extend_from_slice(node_id.as_bytes());
        buf.extend_from_slice(&hour.to_le_bytes());
        let digest = crate::crypto_helpers::sha256_hash_sync(None, &buf);
        hex::encode(digest)
    }

    /// Serialize to JSON bytes for UDP transmission
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from JSON bytes received via UDP
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    /// Validate the discovery message
    ///
    /// Checks:
    /// - Protocol version is "2.0", "2.1", or "3.0"
    /// - Session ID is not empty
    /// - At least one capability
    /// - At least one protocol
    /// - Port is valid (non-zero)
    /// - Timestamp is recent (within 5 minutes)
    /// - For v3.0: `node_id` and endpoints are present
    pub fn validate(&self) -> Result<(), String> {
        if self.version != "2.0" && self.version != "2.1" && self.version != "3.0" {
            return Err(format!("Unsupported protocol version: {}", self.version));
        }

        // v3.0 specific validation
        if self.version == "3.0" {
            if self.node_id.is_none() {
                return Err("v3.0 requires node_id".to_string());
            }
            if self.node_name.is_none() {
                return Err("v3.0 requires node_name".to_string());
            }
            if self.endpoints.as_ref().is_none_or(Vec::is_empty) {
                return Err("v3.0 requires at least one endpoint".to_string());
            }
        }

        if self.port == 0 {
            return Err("Invalid port: 0".to_string());
        }

        if self.session_id.is_empty() {
            return Err("Session ID is empty".to_string());
        }

        if self.capabilities.is_empty() {
            return Err("No capabilities specified".to_string());
        }

        if self.protocols.is_empty() {
            return Err("No protocols specified".to_string());
        }

        // Check timestamp is recent (within 5 minutes)
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

        let age = now.saturating_sub(self.timestamp);
        if age > 300 {
            // 5 minutes
            return Err(format!("Message too old: {age} seconds"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_new_v2() {
        let msg = AnonymousDiscoveryMessage::new(
            vec!["orchestration".to_string()],
            vec!["https".to_string()],
            8080,
        );
        assert_eq!(msg.version, "2.1");
        assert!(msg.node_id.is_none());
        assert!(!msg.session_id.is_empty());
    }

    #[test]
    fn test_message_new_v3() {
        let endpoints = vec![TransportEndpointMessage {
            interface_type: "ethernet".to_string(),
            address: "192.168.1.100:8080".to_string(),
            protocols: vec!["https".to_string()],
            preference: 255,
        }];

        let msg = AnonymousDiscoveryMessage::new_v3(
            "test-node-id".to_string(),
            "testnode".to_string(),
            endpoints,
            vec!["orchestration".to_string()],
        );

        assert_eq!(msg.version, "3.0");
        assert_eq!(msg.node_id, Some("test-node-id".to_string()));
        assert_eq!(msg.node_name, Some("testnode".to_string()));
        assert!(msg.endpoints.is_some());
    }

    #[test]
    fn test_message_validation_v2() {
        let msg = AnonymousDiscoveryMessage::new(
            vec!["orchestration".to_string()],
            vec!["https".to_string()],
            8080,
        );
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_message_validation_empty_capabilities() {
        let mut msg = AnonymousDiscoveryMessage::new(
            vec!["orchestration".to_string()],
            vec!["https".to_string()],
            8080,
        );
        msg.capabilities.clear();
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_message_serialization() {
        let msg = AnonymousDiscoveryMessage::new(
            vec!["orchestration".to_string()],
            vec!["https".to_string()],
            8080,
        );

        let bytes = msg.to_bytes().expect("Serialization failed");
        let deserialized =
            AnonymousDiscoveryMessage::from_bytes(&bytes).expect("Deserialization failed");

        assert_eq!(msg.version, deserialized.version);
        assert_eq!(msg.capabilities, deserialized.capabilities);
    }

    #[test]
    fn test_session_id_generation() {
        let id1 = AnonymousDiscoveryMessage::generate_session_id();
        let id2 = AnonymousDiscoveryMessage::generate_session_id();

        // Should be different (random UUID component)
        assert_ne!(id1, id2);

        // Should be 64 hex characters (SHA-256)
        assert_eq!(id1.len(), 64);
    }

    #[test]
    fn test_session_id_from_node() {
        let node_id = "test-node";
        let id1 = AnonymousDiscoveryMessage::generate_session_id_from_node(node_id);
        let id2 = AnonymousDiscoveryMessage::generate_session_id_from_node(node_id);

        // Should be identical within same hour (deterministic)
        assert_eq!(id1, id2);

        // Should be 64 hex characters (SHA-256)
        assert_eq!(id1.len(), 64);
    }
}
