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
                txt.insert("lineage_proof".to_string(), encoded);
            } else {
                let proof_hash =
                    hex::encode(crate::crypto_helpers::sha256_hash_sync(None, encoded.as_bytes()));
                txt.insert("lineage_proof_hash".to_string(), proof_hash);
                txt.insert(
                    "lineage_proof_url".to_string(),
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
                txt.insert("lineage_proof".to_string(), encoded);
            } else {
                let digest = crate::crypto_helpers::sha256_hash(crypto, encoded.as_bytes()).await;
                txt.insert("lineage_proof_hash".to_string(), hex::encode(digest));
                txt.insert(
                    "lineage_proof_url".to_string(),
                    format!("{}/api/v1/lineage/proof", self.endpoint),
                );
            }
        }
        txt
    }

    fn txt_records_base(&self) -> HashMap<String, String> {
        let mut txt = HashMap::new();

        txt.insert("node_id".to_string(), self.node_id.clone());
        txt.insert("capabilities".to_string(), self.capabilities.join(","));
        txt.insert("tags".to_string(), self.tags.join(","));
        txt.insert("endpoint".to_string(), self.endpoint.clone());
        txt.insert("timestamp".to_string(), self.timestamp.to_string());

        if let Some(name) = &self.node_name {
            txt.insert("node_name".to_string(), name.clone());
        }

        for (k, v) in &self.metadata {
            txt.insert(format!("meta_{k}"), v.clone());
        }

        if let Some(lineage) = &self.genetic_lineage {
            txt.insert("lineage".to_string(), lineage.to_string());
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

    #[test]
    fn test_discovery_packet_creation() {
        let packet = DiscoveryPacket::new(
            "node-123",
            vec!["compute".to_string(), "storage".to_string()],
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
            vec!["compute".to_string()],
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
            vec!["compute".to_string(), "storage".to_string()],
            "http://192.168.1.100:8080",
        )
        .with_name("test-node")
        .with_metadata("version", "1.0.0");

        let txt = packet.to_txt_records();

        assert_eq!(txt.get("node_id"), Some(&"node-123".to_string()));
        assert_eq!(txt.get("node_name"), Some(&"test-node".to_string()));
        assert_eq!(txt.get("capabilities"), Some(&"compute,storage".to_string()));
        assert_eq!(txt.get("meta_version"), Some(&"1.0.0".to_string()));
    }

    #[test]
    fn test_txt_records_round_trip() {
        let original = DiscoveryPacket::new(
            "node-123",
            vec!["compute".to_string()],
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
            vec!["compute".to_string()],
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
}
