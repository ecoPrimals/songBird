//! Universal Trust Evaluation API
//!
//! Generic, provider-agnostic API for trust evaluation across any security provider.
//! Works with security provider, `ToadStool`, hardware HSMs, and future cryptographic systems.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Universal trust evaluation request (v1)
///
/// Generic format that works with any security provider.
/// Providers extract the attestation formats they understand.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalTrustRequest {
    /// Request format version
    pub request_format: String,

    /// Information about the peer being evaluated
    pub evaluator: EvaluatorInfo,

    /// Context about how/when the peer was discovered
    pub context: DiscoveryContext,
}

/// Information about the peer being evaluated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatorInfo {
    /// Unique peer identifier
    pub peer_id: String,

    /// Identity attestations from various providers
    pub attestations: Vec<IdentityAttestation>,
}

/// Identity attestation from a security provider
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityAttestation {
    /// Optional hint about which provider issued this
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Format of the attestation data (e.g., "`tag_list`", "`x509_certificate`", "`pgp_key`")
    pub format: String,

    /// The attestation data itself (format-specific, flexible)
    pub data: JsonValue,
}

/// Context about how/when the peer was discovered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryContext {
    /// How peer was discovered (e.g., "`udp_multicast`", "mdns", "manual", "registry")
    pub discovery_method: String,

    /// When peer was first seen (ISO8601 timestamp)
    pub first_seen_at: String,

    /// Peer's network endpoint
    pub endpoint: String,

    /// Peer's advertised capabilities
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Provider-specific context (extensible)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, JsonValue>,
}

/// Universal trust evaluation response (v1)
///
/// Generic format that works with any security provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalTrustResponse {
    /// Response format version
    pub response_format: String,

    /// Trust decision
    pub decision: TrustDecision,

    /// Confidence level (0.0 = no trust, 1.0 = full trust)
    pub confidence: f64,

    /// Human-readable explanation
    pub reason: String,

    /// Machine-readable reason code
    pub reason_code: String,

    /// Additional context (provider-specific)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, JsonValue>,

    /// When this trust decision expires (for caching)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,

    /// Provider-specific response data (extensible)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom: HashMap<String, JsonValue>,
}

/// Trust decision
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrustDecision {
    /// Automatically accept this peer (high trust, same family/group)
    AutoAccept,

    /// Prompt user for consent (unknown or different family)
    PromptUser,

    /// Reject this peer (known malicious or policy violation)
    Reject,
}

impl UniversalTrustRequest {
    /// Create a new universal trust request
    pub fn new(peer_id: impl Into<String>, attestations: Vec<IdentityAttestation>) -> Self {
        Self {
            request_format: "universal_trust_v1".to_string(),
            evaluator: EvaluatorInfo {
                peer_id: peer_id.into(),
                attestations,
            },
            context: DiscoveryContext {
                discovery_method: "unknown".to_string(),
                first_seen_at: chrono::Utc::now().to_rfc3339(),
                endpoint: String::new(),
                capabilities: Vec::new(),
                custom: HashMap::new(),
            },
        }
    }

    /// Set discovery context
    #[must_use]
    pub fn with_context(mut self, context: DiscoveryContext) -> Self {
        self.context = context;
        self
    }

    /// Set discovery method
    #[must_use]
    pub fn with_discovery_method(mut self, method: impl Into<String>) -> Self {
        self.context.discovery_method = method.into();
        self
    }

    /// Set endpoint
    #[must_use]
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.context.endpoint = endpoint.into();
        self
    }

    /// Set capabilities
    #[must_use]
    pub fn with_capabilities(mut self, capabilities: Vec<String>) -> Self {
        self.context.capabilities = capabilities;
        self
    }
}

impl IdentityAttestation {
    /// Create a new identity attestation
    pub fn new(format: impl Into<String>, data: JsonValue) -> Self {
        Self {
            provider: None,
            format: format.into(),
            data,
        }
    }

    /// Set provider hint
    #[must_use]
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// Create a tag list attestation (for security provider-style tags)
    #[must_use]
    pub fn tag_list(tags: Vec<String>) -> Self {
        Self {
            provider: None,
            format: "tag_list".to_string(),
            data: serde_json::json!({
                "tags": tags
            }),
        }
    }

    /// Create a tag list attestation with family ID
    pub fn tag_list_with_family(tags: Vec<String>, family_id: impl Into<String>) -> Self {
        Self {
            provider: None,
            format: "tag_list".to_string(),
            data: serde_json::json!({
                "tags": tags,
                "family_id": family_id.into()
            }),
        }
    }
}

impl UniversalTrustResponse {
    /// Check if the decision is auto-accept
    #[must_use]
    pub fn is_auto_accept(&self) -> bool {
        self.decision == TrustDecision::AutoAccept
    }

    /// Check if the decision is prompt user
    #[must_use]
    pub fn is_prompt_user(&self) -> bool {
        self.decision == TrustDecision::PromptUser
    }

    /// Check if the decision is reject
    #[must_use]
    pub fn is_reject(&self) -> bool {
        self.decision == TrustDecision::Reject
    }

    /// Get metadata value
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&JsonValue> {
        self.metadata.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universal_trust_request_creation() {
        let attestations =
            vec![IdentityAttestation::tag_list(vec!["beardog:family:iidn:tower1".to_string()])];

        let request = UniversalTrustRequest::new("tower1", attestations)
            .with_discovery_method("udp_multicast")
            .with_endpoint("https://192.168.1.100:8080")
            .with_capabilities(vec!["orchestration".to_string()]);

        assert_eq!(request.request_format, "universal_trust_v1");
        assert_eq!(request.evaluator.peer_id, "tower1");
        assert_eq!(request.context.discovery_method, "udp_multicast");
    }

    #[test]
    fn test_identity_attestation_tag_list() {
        let attestation = IdentityAttestation::tag_list_with_family(
            vec!["beardog:family:iidn:tower1".to_string()],
            "iidn",
        );

        assert_eq!(attestation.format, "tag_list");
        assert!(attestation.data.get("tags").is_some());
        assert!(attestation.data.get("family_id").is_some());
    }

    #[test]
    fn test_trust_decision_serialization() {
        let decision = TrustDecision::AutoAccept;
        let json = serde_json::to_string(&decision).unwrap();
        assert_eq!(json, r#""auto_accept""#);

        let decision = TrustDecision::PromptUser;
        let json = serde_json::to_string(&decision).unwrap();
        assert_eq!(json, r#""prompt_user""#);

        let decision = TrustDecision::Reject;
        let json = serde_json::to_string(&decision).unwrap();
        assert_eq!(json, r#""reject""#);
    }

    #[test]
    fn test_universal_trust_response_helpers() {
        let response = UniversalTrustResponse {
            response_format: "universal_trust_v1".to_string(),
            decision: TrustDecision::AutoAccept,
            confidence: 1.0,
            reason: "Same family".to_string(),
            reason_code: "same_genetic_family".to_string(),
            metadata: HashMap::new(),
            expires_at: None,
            custom: HashMap::new(),
        };

        assert!(response.is_auto_accept());
        assert!(!response.is_prompt_user());
        assert!(!response.is_reject());
    }
}
