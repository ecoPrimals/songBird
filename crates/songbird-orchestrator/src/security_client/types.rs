//! Security capability types for trust evaluation
//!
//! This module contains all request/response types used by the security capability client.

use serde::{Deserialize, Serialize};
use songbird_types::{LineageId, LineageProof};
use std::collections::HashMap;

/// Wrapper for potentially wrapped API responses (Agnostic Pattern - Jan 3, 2026)
///
/// Some security providers wrap their responses in `{"success": true, "data": {...}}`.
/// This allows graceful handling of both wrapped and unwrapped formats during transition.
#[derive(Debug, Clone, Deserialize)]
pub(super) struct ApiResponseWrapper<T> {
    pub success: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Identity response from security provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityResponse {
    /// Encryption tag for this node
    ///
    /// Format: `{provider}:family:{family_id}:{node_id}` (provider-agnostic!)
    /// Example: `crypto-provider:family:a3f2:tower1`
    pub encryption_tag: String,

    /// Security provider capabilities
    ///
    /// Example: `["identity", "encryption", "trust-evaluation"]`
    pub capabilities: Vec<String>,

    /// Family ID (optional)
    ///
    /// Example: `ecoPrimals-20260101-a3f2`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_id: Option<String>,
}

/// Trust evaluation request to security provider
///
/// Orchestrator sends peer information to security provider,
/// asking "should I trust this peer?"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEvaluationRequest {
    /// Peer node ID
    pub peer_id: String,

    /// Peer family ID (v3.14.1 - tag-based identity)
    ///
    /// Extracted from peer tags (e.g., "beardog:family:nat0" → "nat0")
    /// Songbird doesn't interpret this - just extracts and passes to security provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_family: Option<String>,

    /// Peer tags (includes security provider encryption tag if present)
    ///
    /// Example: `["crypto:family:a3f2", "encryption_enabled"]`
    pub peer_tags: Vec<String>,

    /// Connection information (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_info: Option<ConnectionInfo>,

    /// Discovery context (optional, flattened HashMap for security provider compatibility)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, String>>,
}

/// Connection information for peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    /// Peer endpoint
    pub endpoint: String,

    /// Protocol used
    pub protocol: String,
}

/// Discovery context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryContext {
    /// How peer was discovered
    pub discovery_method: String,

    /// When peer was first seen (Unix timestamp as string for JSON compatibility)
    pub first_seen_at: String,

    /// Additional metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

/// Trust evaluation response from security provider
///
/// Provider's decision on whether to trust the peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEvaluationResponse {
    /// Decision: "auto_accept", "prompt_user", or "reject"
    pub decision: String,

    /// Trust level: "high", "medium", "low", or "none"
    pub trust_level: String,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,

    /// Human-readable reason
    pub reason: String,

    /// Encryption tag for establishing secure connection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_tag: Option<String>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl TrustEvaluationResponse {
    /// Check if decision is to auto-accept
    #[must_use]
    pub fn is_auto_accept(&self) -> bool {
        self.decision == "auto_accept"
    }

    /// Check if decision requires user prompt
    #[must_use]
    pub fn requires_prompt(&self) -> bool {
        self.decision == "prompt_user"
    }

    /// Check if decision is to reject
    #[must_use]
    pub fn is_reject(&self) -> bool {
        self.decision == "reject"
    }
}

/// Current lineage information from security provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentLineageInfo {
    pub lineage_id: LineageId,
    pub proof: LineageProof,
    pub genesis_timestamp: u64,
}

/// Verification result from security provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub valid: bool,
    pub same_genesis: bool,
    pub lineage_id: LineageId,
    pub messages: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_decision_helpers() {
        let auto_accept = TrustEvaluationResponse {
            decision: "auto_accept".to_string(),
            trust_level: "high".to_string(),
            confidence: 1.0,
            reason: "same_family".to_string(),
            encryption_tag: Some("crypto-provider:family:a3f2".to_string()),
            metadata: HashMap::new(),
        };

        assert!(auto_accept.is_auto_accept());
        assert!(!auto_accept.requires_prompt());
        assert!(!auto_accept.is_reject());

        let prompt = TrustEvaluationResponse {
            decision: "prompt_user".to_string(),
            trust_level: "low".to_string(),
            confidence: 0.5,
            reason: "different_family".to_string(),
            encryption_tag: None,
            metadata: HashMap::new(),
        };

        assert!(!prompt.is_auto_accept());
        assert!(prompt.requires_prompt());
        assert!(!prompt.is_reject());

        let reject = TrustEvaluationResponse {
            decision: "reject".to_string(),
            trust_level: "none".to_string(),
            confidence: 0.0,
            reason: "no_lineage".to_string(),
            encryption_tag: None,
            metadata: HashMap::new(),
        };

        assert!(!reject.is_auto_accept());
        assert!(!reject.requires_prompt());
        assert!(reject.is_reject());
    }

    #[test]
    fn test_identity_response_serialization() {
        let identity = IdentityResponse {
            encryption_tag: "crypto-provider:family:a3f2:tower1".to_string(),
            capabilities: vec!["identity".to_string(), "encryption".to_string()],
            family_id: Some("ecoPrimals-20260101-a3f2".to_string()),
        };

        let json = serde_json::to_string(&identity).unwrap();
        let deserialized: IdentityResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(identity.encryption_tag, deserialized.encryption_tag);
        assert_eq!(identity.capabilities, deserialized.capabilities);
        assert_eq!(identity.family_id, deserialized.family_id);
    }

    #[test]
    fn test_trust_request_serialization() {
        let mut context = HashMap::new();
        context.insert("discovery_method".to_string(), "udp_multicast".to_string());
        context.insert("first_seen_at".to_string(), "2024-01-01T12:00:00Z".to_string());

        let request = TrustEvaluationRequest {
            peer_id: "tower2".to_string(),
            peer_family: Some("a3f2".to_string()), // Extracted from tags
            peer_tags: vec!["crypto-provider:family:a3f2".to_string()],
            connection_info: Some(ConnectionInfo {
                endpoint: "https://192.168.1.134:8080".to_string(),
                protocol: "tarpc".to_string(),
            }),
            context: Some(context),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: TrustEvaluationRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.peer_id, deserialized.peer_id);
        assert_eq!(request.peer_tags, deserialized.peer_tags);
    }
}
