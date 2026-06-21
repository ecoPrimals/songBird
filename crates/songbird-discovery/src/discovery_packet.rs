// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Enhanced Discovery Protocol with Genetic Lineage
//!
//! Extends Songbird's discovery protocol to include cryptographic lineage information,
//! enabling automatic peer trust based on shared genetic ancestry.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use songbird_crypto_provider::CryptoProvider;
use songbird_types::{LineageId, LineageProof};
use std::collections::HashMap;

/// Identity attestation from a security provider
///
/// Generic, provider-agnostic structure for identity information.
/// Each attestation is self-describing and extensible.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityAttestation {
    /// Capability that provided this attestation (e.g., "security/identity")
    pub provider_capability: String,

    /// Format of the attestation data (e.g., "`tag_list`", "`x509_certificate`", "`pgp_key`")
    pub format: String,

    /// The attestation data itself (format-specific, flexible)
    pub data: JsonValue,
}

/// Enhanced discovery packet with genetic lineage support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryPacket {
    /// Unique node identifier
    pub node_id: String,

    /// Human-readable node name
    pub node_name: Option<String>,

    /// Advertised capabilities
    pub capabilities: Vec<String>,

    /// Generic tags (e.g., `security provider` encryption tags, protocol versions)
    ///
    /// Tags are opaque strings that can be used for various purposes:
    /// - `security provider` encryption: `"beardog:family:a3f2:tower1"`
    /// - Protocol support: `"btsp_enabled"`, `"birdsong_v2"`
    /// - Custom metadata: any string format
    ///
    /// Songbird doesn't parse these - just passes them to security provider for evaluation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Identity attestations (generic, structured format)
    ///
    /// Provides structured identity information from various providers.
    /// Each attestation is self-describing and provider-agnostic.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub identity_attestations: Vec<IdentityAttestation>,

    /// Network endpoint (HTTP/HTTPS URL)
    pub endpoint: String,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Genetic lineage identifier (NEW)
    ///
    /// This identifies the cryptographic ancestry of this node,
    /// allowing peers to verify shared genetic lineage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genetic_lineage: Option<LineageId>,

    /// Cryptographic lineage proof (NEW)
    ///
    /// Contains the signature chain proving this node's ancestry,
    /// enabling automatic trust establishment for same-lineage peers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage_proof: Option<LineageProof>,

    /// Discovery timestamp
    #[serde(default = "current_timestamp")]
    pub timestamp: u64,
}

impl DiscoveryPacket {
    /// Create a new discovery packet
    #[must_use]
    pub fn new(
        node_id: impl Into<String>,
        capabilities: Vec<String>,
        endpoint: impl Into<String>,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            node_name: None,
            capabilities,
            tags: Vec::new(),
            identity_attestations: Vec::new(),
            endpoint: endpoint.into(),
            metadata: HashMap::new(),
            genetic_lineage: None,
            lineage_proof: None,
            timestamp: current_timestamp(),
        }
    }

    /// Add tags (e.g., `security provider` encryption tags)
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Add a single tag
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add identity attestations (generic format)
    #[must_use]
    pub fn with_identity_attestations(mut self, attestations: Vec<IdentityAttestation>) -> Self {
        self.identity_attestations = attestations;
        self
    }

    /// Add a single identity attestation
    #[must_use]
    pub fn with_identity_attestation(mut self, attestation: IdentityAttestation) -> Self {
        self.identity_attestations.push(attestation);
        self
    }

    /// Add genetic lineage information
    #[must_use]
    pub fn with_lineage(mut self, lineage_id: LineageId, proof: LineageProof) -> Self {
        self.genetic_lineage = Some(lineage_id);
        self.lineage_proof = Some(proof);
        self
    }

    /// Set node name
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.node_name = Some(name.into());
        self
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Convert to mDNS/DNS-SD TXT records
    ///
    /// Serializes the discovery packet into TXT record format for mDNS broadcasting.
    #[must_use]
    pub fn to_txt_records(&self) -> HashMap<String, String> {
        let mut txt = self.txt_records_base();
        if let Some(proof) = &self.lineage_proof
            && let Ok(encoded) = proof.to_discovery_txt()
        {
            if encoded.len() <= 400 {
                txt.insert(String::from("lineage_proof"), encoded);
            } else {
                let proof_hash =
                    hex::encode(crate::crypto_helpers::sha256_hash_sync(None, encoded.as_bytes()));
                txt.insert(String::from("lineage_proof_hash"), proof_hash);
                txt.insert(
                    String::from("lineage_proof_url"),
                    format!("{}/api/v1/lineage/proof", self.endpoint),
                );
            }
        }
        txt
    }

    pub async fn to_txt_records_with_crypto(
        &self,
        crypto: Option<&CryptoProvider>,
    ) -> HashMap<String, String> {
        let mut txt = self.txt_records_base();
        if let Some(proof) = &self.lineage_proof
            && let Ok(encoded) = proof.to_discovery_txt()
        {
            if encoded.len() <= 400 {
                txt.insert(String::from("lineage_proof"), encoded);
            } else {
                let digest = crate::crypto_helpers::sha256_hash(crypto, encoded.as_bytes()).await;
                txt.insert(String::from("lineage_proof_hash"), hex::encode(digest));
                txt.insert(
                    String::from("lineage_proof_url"),
                    format!("{}/api/v1/lineage/proof", self.endpoint),
                );
            }
        }
        txt
    }

    fn txt_records_base(&self) -> HashMap<String, String> {
        let mut txt = HashMap::new();

        txt.insert(String::from("node_id"), self.node_id.clone());
        txt.insert(String::from("capabilities"), self.capabilities.join(","));
        txt.insert(String::from("tags"), self.tags.join(","));
        txt.insert(String::from("endpoint"), self.endpoint.clone());
        txt.insert(String::from("timestamp"), self.timestamp.to_string());

        if let Some(name) = &self.node_name {
            txt.insert(String::from("node_name"), name.clone());
        }

        for (k, v) in &self.metadata {
            txt.insert(format!("meta_{k}"), v.clone());
        }

        if let Some(lineage) = &self.genetic_lineage {
            txt.insert(String::from("lineage"), lineage.to_string());
        }

        txt
    }

    /// Parse from mDNS/DNS-SD TXT records
    ///
    /// # Errors
    /// Returns error if required fields are missing or invalid
    pub fn from_txt_records(txt: &HashMap<String, String>) -> Result<Self, DiscoveryError> {
        let node_id = txt.get("node_id").ok_or(DiscoveryError::MissingField("node_id"))?.clone();

        let capabilities = txt
            .get("capabilities")
            .map(|s| s.split(',').map(String::from).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default();

        let tags = txt
            .get("tags")
            .map(|s| s.split(',').map(String::from).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default();

        let endpoint = txt.get("endpoint").ok_or(DiscoveryError::MissingField("endpoint"))?.clone();

        let timestamp =
            txt.get("timestamp").and_then(|s| s.parse().ok()).unwrap_or_else(current_timestamp);

        let node_name = txt.get("node_name").cloned();

        // Extract user metadata (keys starting with "meta_")
        let metadata = txt
            .iter()
            .filter(|(k, _)| k.starts_with("meta_"))
            .map(|(k, v)| (k.trim_start_matches("meta_").to_string(), v.clone()))
            .collect();

        // NEW: Parse genetic lineage
        let genetic_lineage = txt.get("lineage").and_then(|s| LineageId::from_str(s).ok());

        // NEW: Parse lineage proof
        // When lineage_proof_hash exists but lineage_proof doesn't, proof is too large for TXT record
        let lineage_proof = txt
            .get("lineage_proof")
            .and_then(|proof_txt| LineageProof::from_discovery_txt(proof_txt).ok());

        Ok(Self {
            node_id,
            node_name,
            capabilities,
            tags,
            identity_attestations: Vec::new(), // Default to empty
            endpoint,
            metadata,
            genetic_lineage,
            lineage_proof,
            timestamp,
        })
    }

    /// Check if this packet has genetic lineage information
    #[must_use]
    pub const fn has_lineage(&self) -> bool {
        self.genetic_lineage.is_some()
    }

    /// Check if this packet has a complete lineage proof
    #[must_use]
    pub const fn has_proof(&self) -> bool {
        self.lineage_proof.is_some()
    }
}

/// Errors that can occur during discovery packet operations
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    /// Required field is missing from TXT records
    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    /// Invalid data format
    #[error("Invalid data format: {0}")]
    InvalidFormat(String),

    /// Lineage error
    #[error("Lineage error: {0}")]
    LineageError(#[from] songbird_types::LineageError),
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use songbird_types::LineageProof;
    use std::collections::HashMap;

    #[test]
    fn test_discovery_packet_creation() {
        let packet = DiscoveryPacket::new(
            "node-123",
            vec![String::from("compute"), String::from("storage")],
            "http://192.168.1.100:8080",
        );

        assert_eq!(packet.node_id, "node-123");
        assert_eq!(packet.capabilities.len(), 2);
        assert_eq!(packet.endpoint, "http://192.168.1.100:8080");
        assert!(!packet.has_lineage());
    }

    #[test]
    fn test_discovery_packet_with_lineage() {
        let lineage_id = LineageId::new("lineage:tower1:2026-01-02:abc123");
        let proof = LineageProof::new(lineage_id.clone(), vec![], 1234567890);

        let packet = DiscoveryPacket::new(
            "node-123",
            vec![String::from("compute")],
            "http://192.168.1.100:8080",
        )
        .with_lineage(lineage_id, proof);

        assert!(packet.has_lineage());
        assert!(packet.has_proof());
    }

    #[test]
    fn test_txt_records_conversion() {
        let packet = DiscoveryPacket::new(
            "node-123",
            vec![String::from("compute"), String::from("storage")],
            "http://192.168.1.100:8080",
        )
        .with_name("test-node")
        .with_metadata("version", "1.0.0");

        let txt = packet.to_txt_records();

        assert_eq!(txt.get("node_id"), Some(&String::from("node-123")));
        assert_eq!(txt.get("node_name"), Some(&String::from("test-node")));
        assert_eq!(txt.get("capabilities"), Some(&String::from("compute,storage")));
        assert_eq!(txt.get("meta_version"), Some(&String::from("1.0.0")));
    }

    #[test]
    fn test_txt_records_round_trip() {
        let original = DiscoveryPacket::new(
            "node-123",
            vec![String::from("compute")],
            "http://192.168.1.100:8080",
        )
        .with_name("test-node");

        let txt = original.to_txt_records();
        let parsed = DiscoveryPacket::from_txt_records(&txt).unwrap();

        assert_eq!(parsed.node_id, original.node_id);
        assert_eq!(parsed.node_name, original.node_name);
        assert_eq!(parsed.capabilities, original.capabilities);
        assert_eq!(parsed.endpoint, original.endpoint);
    }

    #[test]
    fn test_txt_records_with_lineage() {
        let lineage_id = LineageId::new("lineage:tower1:2026-01-02:abc123");
        let proof = LineageProof::new(lineage_id.clone(), vec![], 1234567890);

        let packet = DiscoveryPacket::new(
            "node-123",
            vec![String::from("compute")],
            "http://192.168.1.100:8080",
        )
        .with_lineage(lineage_id, proof);

        let txt = packet.to_txt_records();

        assert!(txt.contains_key("lineage"));
        assert!(txt.contains_key("lineage_proof") || txt.contains_key("lineage_proof_hash"));

        // Parse back
        let parsed = DiscoveryPacket::from_txt_records(&txt).unwrap();
        assert!(parsed.has_lineage());
    }

    #[test]
    fn test_from_txt_records_missing_node_id() {
        let mut txt = HashMap::new();
        txt.insert(String::from("endpoint"), String::from("http://192.168.1.100:8080"));

        let err = DiscoveryPacket::from_txt_records(&txt).unwrap_err();
        assert!(matches!(err, DiscoveryError::MissingField("node_id")));
    }

    #[test]
    fn test_from_txt_records_missing_endpoint() {
        let mut txt = HashMap::new();
        txt.insert(String::from("node_id"), String::from("node-1"));

        let err = DiscoveryPacket::from_txt_records(&txt).unwrap_err();
        assert!(matches!(err, DiscoveryError::MissingField("endpoint")));
    }

    #[test]
    fn test_tags_roundtrip_via_txt_records() {
        let original = DiscoveryPacket::new(
            "node-tags",
            vec![String::from("compute")],
            "http://192.168.1.100:8080",
        )
        .with_tags(vec![String::from("birdsong_v2"), String::from("btsp_enabled")]);

        let parsed = DiscoveryPacket::from_txt_records(&original.to_txt_records()).unwrap();
        assert_eq!(parsed.tags, original.tags);
    }

    #[test]
    fn test_identity_attestation_builder() {
        let attestation = IdentityAttestation {
            provider_capability: String::from("security/identity"),
            format: String::from("tag_list"),
            data: serde_json::json!({"tags": ["family:abc"]}),
        };

        let packet = DiscoveryPacket::new(
            "node-attest",
            vec![String::from("compute")],
            "http://192.168.1.100:8080",
        )
        .with_identity_attestation(attestation.clone());

        assert_eq!(packet.identity_attestations.len(), 1);
        assert_eq!(packet.identity_attestations[0], attestation);
    }

    #[test]
    fn test_empty_capabilities_roundtrip() {
        let original = DiscoveryPacket::new("node-empty-caps", vec![], "http://127.0.0.1:8080");

        let parsed = DiscoveryPacket::from_txt_records(&original.to_txt_records()).unwrap();
        assert!(parsed.capabilities.is_empty());
    }

    #[tokio::test]
    async fn test_to_txt_records_with_crypto_matches_sync() {
        let lineage_id = LineageId::new("lineage:tower1:2026-01-02:abc123");
        let proof = LineageProof::new(lineage_id.clone(), vec![], 1234567890);
        let packet = DiscoveryPacket::new(
            "node-crypto",
            vec![String::from("compute")],
            "http://192.168.1.100:8080",
        )
        .with_lineage(lineage_id, proof);

        let sync_txt = packet.to_txt_records();
        let crypto = songbird_crypto_provider::CryptoProvider::from_env();
        let async_txt = packet.to_txt_records_with_crypto(Some(&crypto)).await;

        assert_eq!(sync_txt.get("node_id"), async_txt.get("node_id"));
        assert_eq!(sync_txt.get("lineage"), async_txt.get("lineage"));
    }

    #[test]
    fn test_malformed_lineage_id_is_ignored_on_parse() {
        let mut txt = HashMap::new();
        txt.insert(String::from("node_id"), String::from("node-bad-lineage"));
        txt.insert(String::from("endpoint"), String::from("http://192.168.1.100:8080"));
        txt.insert(String::from("lineage"), String::from("not-a-valid-lineage"));

        let parsed = DiscoveryPacket::from_txt_records(&txt).unwrap();
        assert!(parsed.genetic_lineage.is_none());
        assert!(!parsed.has_lineage());
    }

    #[test]
    fn test_lineage_roundtrip_preserves_proof() {
        let lineage_id = LineageId::new("lineage:tower1:2026-01-02:abc123");
        let proof = LineageProof::new(lineage_id.clone(), vec![], 1234567890);

        let original = DiscoveryPacket::new(
            "node-proof",
            vec![String::from("compute")],
            "http://192.168.1.100:8080",
        )
        .with_lineage(lineage_id.clone(), proof);

        let parsed = DiscoveryPacket::from_txt_records(&original.to_txt_records()).unwrap();
        assert_eq!(parsed.genetic_lineage.as_ref(), Some(&lineage_id));
        assert!(parsed.has_proof());
    }

    #[test]
    fn test_metadata_prefix_roundtrip() {
        let original = DiscoveryPacket::new(
            "node-meta",
            vec![String::from("compute")],
            "http://192.168.1.100:8080",
        )
        .with_metadata("region", "us-west")
        .with_metadata("tier", "prod");

        let parsed = DiscoveryPacket::from_txt_records(&original.to_txt_records()).unwrap();
        assert_eq!(parsed.metadata.get("region"), Some(&String::from("us-west")));
        assert_eq!(parsed.metadata.get("tier"), Some(&String::from("prod")));
    }
}
