//! Genetic Lineage Types for Songbird Discovery
//!
//! Provides types for cryptographic lineage authentication and auto-accept logic.
//! Integrates with BearDog Phase 1.5 APIs for sovereign peer discovery.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Genetic lineage identifier
///
/// Uniquely identifies a lineage chain from genesis to current node.
/// Format: `lineage:{tower_id}:{timestamp}:{hash}`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LineageId(String);

impl LineageId {
    /// Create a new lineage ID
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Parse from string
    ///
    /// # Errors
    /// Returns error if the string is not a valid lineage ID format
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, LineageError> {
        if s.starts_with("lineage:") {
            Ok(Self(s.to_string()))
        } else {
            Err(LineageError::InvalidFormat(s.to_string()))
        }
    }

    /// Get the raw lineage ID string
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Extract tower ID from lineage (if available)
    #[must_use]
    pub fn tower_id(&self) -> Option<&str> {
        let parts: Vec<&str> = self.0.split(':').collect();
        parts.get(1).copied()
    }
}

impl fmt::Display for LineageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for LineageId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<LineageId> for String {
    fn from(id: LineageId) -> Self {
        id.0
    }
}

/// Cryptographic proof of genetic lineage
///
/// Contains the signature chain from genesis to current node,
/// allowing peers to verify shared ancestry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageProof {
    /// The lineage identifier this proof validates
    pub lineage_id: LineageId,

    /// Cryptographic signatures in the chain
    pub signatures: Vec<LineageSignature>,

    /// Genesis timestamp (Unix epoch)
    pub genesis_timestamp: u64,

    /// Proof generation timestamp
    pub generated_at: u64,

    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Individual signature in the lineage chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageSignature {
    /// Signer's node ID
    pub signer_node_id: String,

    /// Signature data (hex-encoded)
    pub signature: String,

    /// What was signed (hash of previous link + new node)
    pub signed_data_hash: String,

    /// Timestamp of this signature
    pub timestamp: u64,
}

impl LineageProof {
    /// Create a new lineage proof
    #[must_use]
    pub fn new(
        lineage_id: LineageId,
        signatures: Vec<LineageSignature>,
        genesis_timestamp: u64,
    ) -> Self {
        Self {
            lineage_id,
            signatures,
            genesis_timestamp,
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }

    /// Convert lineage proof to discovery TXT record format
    ///
    /// Encodes the proof as a compact base64 string for mDNS/DNS-SD.
    ///
    /// # Errors
    /// Returns error if serialization fails
    pub fn to_discovery_txt(&self) -> Result<String, LineageError> {
        // Serialize to JSON
        let json = serde_json::to_string(self)
            .map_err(|e| LineageError::SerializationError(e.to_string()))?;

        // Base64 encode for TXT record
        Ok(base64::encode(json.as_bytes()))
    }

    /// Parse lineage proof from discovery TXT record
    ///
    /// Decodes a base64-encoded proof from mDNS/DNS-SD TXT records.
    ///
    /// # Errors
    /// Returns error if decoding or deserialization fails
    pub fn from_discovery_txt(txt: &str) -> Result<Self, LineageError> {
        // Base64 decode
        let json_bytes =
            base64::decode(txt).map_err(|e| LineageError::DecodingError(e.to_string()))?;

        // Deserialize from JSON
        let json_str = String::from_utf8(json_bytes)
            .map_err(|e| LineageError::DecodingError(e.to_string()))?;

        serde_json::from_str(&json_str)
            .map_err(|e| LineageError::DeserializationError(e.to_string()))
    }

    /// Check if this proof is expired
    #[must_use]
    pub fn is_expired(&self, ttl_seconds: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now - self.generated_at > ttl_seconds
    }

    /// Get the number of hops in this lineage chain
    #[must_use]
    pub fn chain_length(&self) -> usize {
        self.signatures.len()
    }
}

/// Lineage verification result from BearDog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageVerification {
    /// Whether the lineage proof is cryptographically valid
    pub valid: bool,

    /// Whether this lineage shares the same genesis as ours
    pub same_genesis: bool,

    /// The verified lineage ID
    pub lineage_id: LineageId,

    /// Any verification errors or warnings
    pub messages: Vec<String>,
}

/// Errors related to genetic lineage operations
#[derive(Debug, thiserror::Error)]
pub enum LineageError {
    /// Invalid lineage ID format
    #[error("Invalid lineage ID format: {0}")]
    InvalidFormat(String),

    /// Serialization error
    #[error("Failed to serialize lineage: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Failed to deserialize lineage: {0}")]
    DeserializationError(String),

    /// Decoding error
    #[error("Failed to decode lineage data: {0}")]
    DecodingError(String),

    /// Verification error
    #[error("Lineage verification failed: {0}")]
    VerificationFailed(String),

    /// BearDog API error
    #[error("BearDog API error: {0}")]
    BearDogError(String),
}

/// Current lineage information for this node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentLineage {
    /// This node's lineage ID
    pub lineage_id: LineageId,

    /// Proof of this lineage
    pub proof: LineageProof,

    /// Genesis tower ID
    pub genesis_tower: Option<String>,

    /// Parent node ID (if spawned from another node)
    pub parent_node_id: Option<String>,
}

impl CurrentLineage {
    /// Create a new current lineage
    #[must_use]
    pub fn new(lineage_id: LineageId, proof: LineageProof) -> Self {
        Self {
            lineage_id,
            proof,
            genesis_tower: None,
            parent_node_id: None,
        }
    }

    /// Set genesis tower
    #[must_use]
    pub fn with_genesis_tower(mut self, tower: impl Into<String>) -> Self {
        self.genesis_tower = Some(tower.into());
        self
    }

    /// Set parent node ID
    #[must_use]
    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent_node_id = Some(parent.into());
        self
    }
}

// Base64 encoding/decoding (using standard base64 crate or implement manually)
mod base64 {
    use base64::{engine::general_purpose, Engine as _};

    pub fn encode(bytes: &[u8]) -> String {
        general_purpose::STANDARD.encode(bytes)
    }

    pub fn decode(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
        general_purpose::STANDARD.decode(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lineage_id_creation() {
        let id = LineageId::new("lineage:tower1:2026-01-02:abc123");
        assert_eq!(id.as_str(), "lineage:tower1:2026-01-02:abc123");
    }

    #[test]
    fn test_lineage_id_tower_extraction() {
        let id = LineageId::new("lineage:tower1:2026-01-02:abc123");
        assert_eq!(id.tower_id(), Some("tower1"));
    }

    #[test]
    fn test_lineage_proof_serialization() {
        let proof = LineageProof {
            lineage_id: LineageId::new("lineage:test:123:abc"),
            signatures: vec![],
            genesis_timestamp: 1234567890,
            generated_at: 1234567900,
            metadata: HashMap::new(),
        };

        // Test TXT record conversion
        let txt = proof.to_discovery_txt().unwrap();
        assert!(!txt.is_empty());

        // Test round-trip
        let decoded = LineageProof::from_discovery_txt(&txt).unwrap();
        assert_eq!(decoded.lineage_id, proof.lineage_id);
        assert_eq!(decoded.genesis_timestamp, proof.genesis_timestamp);
    }

    #[test]
    fn test_lineage_proof_expiration() {
        let proof = LineageProof {
            lineage_id: LineageId::new("lineage:test:123:abc"),
            signatures: vec![],
            genesis_timestamp: 1234567890,
            generated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - 3600, // 1 hour ago
            metadata: HashMap::new(),
        };

        // Should be expired with 30 minute TTL
        assert!(proof.is_expired(1800));

        // Should not be expired with 2 hour TTL
        assert!(!proof.is_expired(7200));
    }

    #[test]
    fn test_current_lineage_builder() {
        let proof = LineageProof::new(LineageId::new("lineage:test:123:abc"), vec![], 1234567890);

        let lineage = CurrentLineage::new(LineageId::new("lineage:test:123:abc"), proof)
            .with_genesis_tower("tower1")
            .with_parent("parent-node-id");

        assert_eq!(lineage.genesis_tower, Some("tower1".to_string()));
        assert_eq!(lineage.parent_node_id, Some("parent-node-id".to_string()));
    }
}
