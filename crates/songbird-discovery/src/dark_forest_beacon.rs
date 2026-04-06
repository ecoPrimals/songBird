// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Dark Forest Beacon - TRUE encrypted discovery with zero metadata leakage
//!
//! ## The Dark Forest Problem
//!
//! Traditional discovery broadcasts leak metadata through plaintext headers:
//! ```json
//! {
//!   "version": "1.0",
//!   "family_id": "<family-id>",  ← Attackers see this!
//!   "encrypted_payload": "..."
//! }
//! ```
//!
//! Passive observers can:
//! - See which families exist
//! - Track family membership  
//! - Build social graphs
//! - Target specific families
//!
//! ## The Dark Forest Solution
//!
//! **Zero metadata leakage**:
//! - Encrypted blob (looks like random noise)
//! - Public nonce (required for decryption, reveals nothing)
//! - Timestamp (replay protection, reveals nothing about sender)
//!
//! **Discovery mechanism**: Try decryption with all known beacon seeds
//! - Success = same beacon family, extract peer info
//! - Failure = different beacon family, ignore as noise
//!
//! ## Privacy Guarantees
//!
//! 1. **Passive observers** see only random-looking data
//! 2. **Different beacon families** cannot decrypt (noise)
//! 3. **Same beacon family** can decrypt and discover peers
//! 4. **No metadata** in cleartext (family, endpoints, capabilities all encrypted)
//! 5. **Replay protection** via timestamps
//! 6. **Session rotation** prevents long-term tracking
//!
//! ## Beacon Genetics
//!
//! Beacon seeds are exchanged on "meeting" (explicit or implicit):
//! - Meeting establishes mutual beacon visibility
//! - Social graph of discovery, not strict inheritance
//! - Separate from lineage (permissions)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use songbird_discovery::dark_forest_beacon::{DarkForestBeacon, BeaconPayload};
//!
//! // Create payload
//! let payload = BeaconPayload {
//!     beacon_id: vec![1, 2, 3],
//!     node_id: "my-node".to_string(),
//!     endpoints: vec!["/ip4/127.0.0.1/tcp/1234".to_string()],
//!     capabilities_hash: [0u8; 32],
//!     cluster_id: None,
//!     session_id: "session-abc".to_string(),
//!     created_at: 1234567890,
//! };
//!
//! // Encrypt with beacon seed (via security provider)
//! let encrypted = encrypt_with_beacon_seed(&payload.to_bytes()?).await?;
//!
//! // Create Dark Forest beacon
//! let beacon = DarkForestBeacon::new(encrypted, nonce);
//!
//! // Broadcast - observers see only noise!
//! broadcast(&beacon.to_bytes()?).await?;
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Dark Forest beacon - completely encrypted discovery packet
///
/// **Privacy Guarantee**: Network observers see only:
/// - Random-looking encrypted blob (ChaCha20-Poly1305 ciphertext)
/// - Public nonce (required for decryption, reveals nothing)
/// - Timestamp (prevents replay attacks, reveals nothing about sender)
///
/// **NO metadata leakage** - `family_id`, capabilities, endpoints all encrypted.
///
/// Unlike `BirdSongPacket` which has plaintext `family_id`, this provides
/// TRUE Dark Forest security where passive observers learn NOTHING.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DarkForestBeacon {
    /// Encrypted payload (opaque to outsiders, signal to family)
    ///
    /// ChaCha20-Poly1305 AEAD ciphertext containing serialized `BeaconPayload`.
    /// Without the beacon seed, this is indistinguishable from random noise.
    pub encrypted_payload: Vec<u8>,

    /// Nonce for ChaCha20-Poly1305 AEAD (public, 12 bytes)
    ///
    /// Required for decryption but reveals nothing about the sender.
    /// Generated randomly for each beacon.
    #[serde(with = "serde_arrays")]
    pub nonce: [u8; 12],

    /// Timestamp (UNIX epoch seconds) for replay protection
    ///
    /// Beacons older than 5 minutes are rejected to prevent replay attacks.
    /// Does not reveal sender identity or family.
    pub timestamp: u64,

    /// Protocol version (2 = Dark Forest format)
    ///
    /// Version 1.0 was legacy `BirdSongPacket` with plaintext `family_id`.
    /// Version 2 is full Dark Forest with zero metadata leakage.
    pub version: u8,
}

/// Helper for serializing [u8; 12] arrays with serde
mod serde_arrays {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(data: &[u8; 12], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        data.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 12], D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<u8>::deserialize(deserializer)?;
        let array: [u8; 12] =
            vec.try_into().map_err(|_| serde::de::Error::custom("Expected 12 bytes for nonce"))?;
        Ok(array)
    }
}

/// Payload inside Dark Forest beacon (only visible after decryption)
///
/// This is what family members see after successful decryption.
/// Observers without beacon genetics see only `encrypted_payload` (noise).
///
/// ## Privacy Strategy
///
/// Even after decryption, we minimize data exposure:
/// - `capabilities_hash` instead of full list (compare without revealing)
/// - `session_id` rotates periodically (prevents long-term tracking)
/// - `cluster_id` is optional (only if explicitly part of cluster)
///
/// Full peer details exchanged after trust establishment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BeaconPayload {
    /// Beacon ID of sender (derived from their beacon seed)
    ///
    /// This is the sender's public beacon identifier.
    /// Allows tracking which beacon family sent this.
    /// Derived from beacon seed via BLAKE3 hash.
    pub beacon_id: Vec<u8>,

    /// Node ID (unique per node)
    ///
    /// Identifies this specific node within the network.
    /// Format: typically UUID or generated identifier.
    pub node_id: String,

    /// Network endpoints (multiaddr format)
    ///
    /// How to reach this node after discovery.
    /// Examples:
    /// - "/ip4/127.0.0.1/tcp/1234"
    /// - "/`ip6/::1/tcp/5678`"
    /// - "/dns/node.example.com/tcp/9999"
    pub endpoints: Vec<String>,

    /// Capabilities hash (BLAKE3, 32 bytes)
    ///
    /// Privacy-preserving capability comparison.
    /// Peers can check if capabilities match without revealing full list.
    /// Full capability exchange happens after trust establishment.
    pub capabilities_hash: [u8; 32],

    /// Cluster ID if part of cluster (optional)
    ///
    /// Used for cluster-aware discovery.
    /// Only present if node is part of named cluster.
    pub cluster_id: Option<String>,

    /// Session ID (rotates periodically)
    ///
    /// Prevents long-term tracking by rotating session identifiers.
    /// Recommended rotation: every 24 hours.
    pub session_id: String,

    /// External tunnel endpoints (VPN/WireGuard/etc)
    ///
    /// Optional external VPN endpoints for inter-primal communication.
    /// Enables connectivity through existing VPN infrastructure.
    /// Encrypted within beacon payload for privacy.
    pub external_tunnels: Vec<ExternalTunnel>,

    /// Timestamp when payload created (UNIX epoch seconds)
    ///
    /// For staleness checks and debugging.
    /// Separate from beacon timestamp (beacon timestamp is outer envelope).
    pub created_at: u64,
}

/// External tunnel configuration
///
/// Describes an external VPN/tunnel endpoint for connectivity.
/// Used when `WireGuard`, `OpenVPN`, or other external tunnels are available.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalTunnel {
    /// Tunnel type
    pub tunnel_type: TunnelType,

    /// Endpoint address (IP:port or hostname:port)
    pub endpoint: String,

    /// Public key (for `WireGuard`, base64 encoded)
    pub public_key: Option<String>,

    /// Optional metadata (protocol-specific)
    pub metadata: std::collections::HashMap<String, String>,
}

/// Supported external tunnel types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TunnelType {
    /// `WireGuard` VPN
    WireGuard,

    /// `OpenVPN`
    OpenVPN,

    /// `IPsec`
    IPsec,

    /// Future: `ZeroTier`, Tailscale, etc.
    Other(String),
}

impl DarkForestBeacon {
    /// Maximum beacon age in seconds (5 minutes)
    ///
    /// Beacons older than this are rejected to prevent replay attacks.
    /// 5 minutes allows for network delays while preventing stale data.
    pub const MAX_AGE_SECONDS: u64 = 300;

    /// Protocol version for Dark Forest format
    pub const VERSION: u8 = 2;

    /// Create new Dark Forest beacon
    ///
    /// # Arguments
    ///
    /// * `encrypted_payload` - ChaCha20-Poly1305 ciphertext of serialized `BeaconPayload`
    /// * `nonce` - 12-byte nonce for AEAD decryption
    ///
    /// # Returns
    ///
    /// Dark Forest beacon with current timestamp and version 2
    #[must_use]
    pub fn new(encrypted_payload: Vec<u8>, nonce: [u8; 12]) -> Self {
        Self {
            encrypted_payload,
            nonce,
            timestamp: Self::current_timestamp(),
            version: Self::VERSION,
        }
    }

    /// Check if beacon is recent (within `MAX_AGE_SECONDS`)
    ///
    /// Beacons older than 5 minutes are considered stale and should be rejected.
    /// This prevents replay attacks where an attacker captures and resends old beacons.
    ///
    /// # Returns
    ///
    /// `true` if beacon age ≤ 5 minutes, `false` otherwise
    #[must_use]
    pub fn is_recent(&self) -> bool {
        let now = Self::current_timestamp();
        let age = now.saturating_sub(self.timestamp);
        age <= Self::MAX_AGE_SECONDS
    }

    /// Get beacon age in seconds
    ///
    /// Returns how many seconds old this beacon is.
    /// Useful for logging and debugging.
    #[must_use]
    pub fn age_seconds(&self) -> u64 {
        let now = Self::current_timestamp();
        now.saturating_sub(self.timestamp)
    }

    /// Serialize to JSON bytes
    ///
    /// # Returns
    ///
    /// JSON bytes suitable for network transmission
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).context("Failed to serialize DarkForestBeacon")
    }

    /// Deserialize from JSON bytes
    ///
    /// # Arguments
    ///
    /// * `bytes` - JSON bytes from network
    ///
    /// # Returns
    ///
    /// Parsed `DarkForestBeacon` or error if invalid JSON/format
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).context("Failed to deserialize DarkForestBeacon")
    }

    /// Get current UNIX timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
    }
}

impl BeaconPayload {
    /// Hash capabilities for privacy-preserving comparison
    ///
    /// Uses BLAKE3 to hash sorted capability list.
    /// Peers can compare hashes without revealing full capabilities.
    ///
    /// # Arguments
    ///
    /// * `capabilities` - List of capability strings
    ///
    /// # Returns
    ///
    /// 32-byte BLAKE3 hash of sorted capabilities
    ///
    /// # Example
    ///
    /// ```
    /// use songbird_discovery::dark_forest_beacon::BeaconPayload;
    /// let caps = vec!["ai".to_string(), "storage".to_string()];
    /// let hash = BeaconPayload::hash_capabilities(&caps);
    /// ```
    #[must_use]
    pub fn hash_capabilities(capabilities: &[String]) -> [u8; 32] {
        use blake3::Hasher;

        // Sort for deterministic hashing
        let mut sorted = capabilities.to_vec();
        sorted.sort();

        let mut hasher = Hasher::new();
        for cap in sorted {
            hasher.update(cap.as_bytes());
            hasher.update(b"|"); // Separator to prevent ambiguity
        }

        let hash = hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(hash.as_bytes());
        result
    }

    /// Create new beacon payload with current timestamp
    ///
    /// # Arguments
    ///
    /// * `beacon_id` - Sender's beacon ID (from beacon seed)
    /// * `node_id` - Node identifier
    /// * `endpoints` - Network endpoints (multiaddr format)
    /// * `capabilities` - List of capabilities (will be hashed)
    /// * `cluster_id` - Optional cluster ID
    /// * `session_id` - Session identifier (should rotate)
    /// * `external_tunnels` - Optional external tunnel endpoints
    ///
    /// # Returns
    ///
    /// `BeaconPayload` with current timestamp and hashed capabilities
    #[must_use]
    pub fn new(
        beacon_id: Vec<u8>,
        node_id: String,
        endpoints: Vec<String>,
        capabilities: &[String],
        cluster_id: Option<String>,
        session_id: String,
    ) -> Self {
        Self {
            beacon_id,
            node_id,
            endpoints,
            capabilities_hash: Self::hash_capabilities(capabilities),
            cluster_id,
            session_id,
            external_tunnels: Vec::new(), // Empty by default
            created_at: DarkForestBeacon::current_timestamp(),
        }
    }

    /// Add external tunnel endpoint
    #[must_use]
    pub fn with_external_tunnel(mut self, tunnel: ExternalTunnel) -> Self {
        self.external_tunnels.push(tunnel);
        self
    }

    /// Add `WireGuard` tunnel endpoint
    ///
    /// Convenience method for adding `WireGuard` tunnels
    #[must_use]
    pub fn with_wireguard(mut self, endpoint: String, public_key: String) -> Self {
        self.external_tunnels.push(ExternalTunnel {
            tunnel_type: TunnelType::WireGuard,
            endpoint,
            public_key: Some(public_key),
            metadata: std::collections::HashMap::new(),
        });
        self
    }

    /// Serialize to JSON bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).context("Failed to serialize BeaconPayload")
    }

    /// Deserialize from JSON bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(bytes).context("Failed to deserialize BeaconPayload")
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_dark_forest_beacon_creation() {
        let beacon = DarkForestBeacon::new(vec![1, 2, 3, 4], [0u8; 12]);

        assert_eq!(beacon.version, DarkForestBeacon::VERSION);
        assert_eq!(beacon.encrypted_payload, vec![1, 2, 3, 4]);
        assert!(beacon.is_recent());
    }

    #[test]
    fn test_dark_forest_beacon_roundtrip() {
        let beacon = DarkForestBeacon::new(vec![1, 2, 3, 4], [5u8; 12]);

        let bytes = beacon.to_bytes().unwrap();
        let decoded = DarkForestBeacon::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.version, 2);
        assert_eq!(decoded.encrypted_payload, vec![1, 2, 3, 4]);
        assert_eq!(decoded.nonce, [5u8; 12]);
    }

    #[test]
    fn test_beacon_payload_creation() {
        let payload = BeaconPayload::new(
            vec![1, 2, 3],
            "test-node".to_string(),
            vec!["/ip4/127.0.0.1/tcp/1234".to_string()],
            &["ai".to_string(), "storage".to_string()],
            None,
            "session-123".to_string(),
        );

        assert_eq!(payload.node_id, "test-node");
        assert_eq!(payload.endpoints.len(), 1);
        assert_eq!(payload.external_tunnels.len(), 0);
        assert!(payload.created_at > 0);
    }

    #[test]
    fn test_beacon_payload_with_wireguard() {
        let payload = BeaconPayload::new(
            vec![1, 2, 3],
            "test-node".to_string(),
            vec!["/ip4/127.0.0.1/tcp/1234".to_string()],
            &["ai".to_string()],
            None,
            "session-123".to_string(),
        )
        .with_wireguard("1.2.3.4:51820".to_string(), "base64_pubkey_here".to_string());

        assert_eq!(payload.external_tunnels.len(), 1);
        assert_eq!(payload.external_tunnels[0].tunnel_type, TunnelType::WireGuard);
        assert_eq!(payload.external_tunnels[0].endpoint, "1.2.3.4:51820");
        assert_eq!(payload.external_tunnels[0].public_key.as_deref(), Some("base64_pubkey_here"));
    }

    #[test]
    fn test_beacon_payload_roundtrip() {
        let payload = BeaconPayload {
            beacon_id: vec![1, 2, 3],
            node_id: "test-node".to_string(),
            endpoints: vec!["/ip4/127.0.0.1/tcp/1234".to_string()],
            capabilities_hash: [0u8; 32],
            cluster_id: Some("cluster-1".to_string()),
            session_id: "session-123".to_string(),
            external_tunnels: vec![],
            created_at: 1234567890,
        };

        let bytes = payload.to_bytes().unwrap();
        let decoded = BeaconPayload::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.node_id, "test-node");
        assert_eq!(decoded.cluster_id, Some("cluster-1".to_string()));
        assert_eq!(decoded.created_at, 1234567890);
    }

    #[test]
    fn test_beacon_is_recent() {
        let beacon = DarkForestBeacon::new(vec![], [0u8; 12]);
        assert!(beacon.is_recent());

        // Create old beacon
        let old_beacon = DarkForestBeacon {
            encrypted_payload: vec![],
            nonce: [0u8; 12],
            timestamp: 1000000, // Very old (Jan 1970)
            version: 2,
        };
        assert!(!old_beacon.is_recent());

        // Create beacon at boundary (exactly MAX_AGE_SECONDS old)
        let boundary_time =
            DarkForestBeacon::current_timestamp() - DarkForestBeacon::MAX_AGE_SECONDS;
        let boundary_beacon = DarkForestBeacon {
            encrypted_payload: vec![],
            nonce: [0u8; 12],
            timestamp: boundary_time,
            version: 2,
        };
        assert!(boundary_beacon.is_recent());

        // Create beacon just past boundary
        let stale_beacon = DarkForestBeacon {
            encrypted_payload: vec![],
            nonce: [0u8; 12],
            timestamp: boundary_time - 1,
            version: 2,
        };
        assert!(!stale_beacon.is_recent());
    }

    #[test]
    fn test_beacon_age_calculation() {
        let beacon = DarkForestBeacon::new(vec![], [0u8; 12]);
        assert_eq!(beacon.age_seconds(), 0);

        let old_beacon = DarkForestBeacon {
            encrypted_payload: vec![],
            nonce: [0u8; 12],
            timestamp: DarkForestBeacon::current_timestamp() - 100,
            version: 2,
        };
        let age = old_beacon.age_seconds();
        assert!((100..=102).contains(&age)); // Allow 2 seconds of test time
    }

    #[test]
    fn test_capabilities_hashing_deterministic() {
        let caps1 = vec!["ai".to_string(), "storage".to_string()];
        let hash1 = BeaconPayload::hash_capabilities(&caps1);

        let caps2 = vec!["ai".to_string(), "storage".to_string()];
        let hash2 = BeaconPayload::hash_capabilities(&caps2);

        assert_eq!(hash1, hash2, "Same capabilities should hash to same value");
    }

    #[test]
    fn test_capabilities_hashing_order_independent() {
        let caps1 = vec!["ai".to_string(), "storage".to_string()];
        let hash1 = BeaconPayload::hash_capabilities(&caps1);

        let caps2 = vec!["storage".to_string(), "ai".to_string()]; // Different order
        let hash2 = BeaconPayload::hash_capabilities(&caps2);

        assert_eq!(hash1, hash2, "Order should not affect hash");
    }

    #[test]
    fn test_capabilities_hashing_different() {
        let caps1 = vec!["ai".to_string(), "storage".to_string()];
        let hash1 = BeaconPayload::hash_capabilities(&caps1);

        let caps2 = vec!["ai".to_string(), "compute".to_string()]; // Different capability
        let hash2 = BeaconPayload::hash_capabilities(&caps2);

        assert_ne!(hash1, hash2, "Different capabilities should hash differently");
    }

    #[test]
    fn test_empty_capabilities_hash() {
        let hash = BeaconPayload::hash_capabilities(&[]);
        assert_ne!(hash, [0u8; 32], "Empty capabilities should have non-zero hash");
    }

    #[test]
    fn tunnel_type_other_roundtrips_json() {
        let t = TunnelType::Other("custom".into());
        let json = serde_json::to_string(&t).unwrap();
        let back: TunnelType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, t);
    }

    #[test]
    fn external_tunnel_roundtrip() {
        let mut meta = std::collections::HashMap::new();
        meta.insert("k".into(), "v".into());
        let t = ExternalTunnel {
            tunnel_type: TunnelType::OpenVPN,
            endpoint: "e:1".into(),
            public_key: None,
            metadata: meta,
        };
        let bytes = serde_json::to_vec(&t).unwrap();
        let back: ExternalTunnel = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back.endpoint, "e:1");
    }

    #[test]
    fn dark_forest_from_bytes_rejects_invalid_json() {
        assert!(DarkForestBeacon::from_bytes(b"not-json").is_err());
    }

    #[test]
    fn beacon_payload_with_external_tunnel_builder() {
        let p = BeaconPayload::new(vec![1], "n".into(), vec![], &["a".into()], None, "s".into())
            .with_external_tunnel(ExternalTunnel {
                tunnel_type: TunnelType::IPsec,
                endpoint: "x".into(),
                public_key: None,
                metadata: std::collections::HashMap::new(),
            });
        assert_eq!(p.external_tunnels.len(), 1);
    }

    #[test]
    fn version_constant_matches_struct() {
        let b = DarkForestBeacon::new(vec![], [0u8; 12]);
        assert_eq!(b.version, DarkForestBeacon::VERSION);
    }
}
