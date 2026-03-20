// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Trust Evaluation Types
//!
//! Shared types for trust evaluation across all adapters.
//! These types are protocol-agnostic and work with any security provider.

use serde::{Deserialize, Serialize};
use songbird_types::TrustLevel;
use std::collections::HashMap; // ← Import TrustLevel enum with Phase 1 deserializer

/// Request for trust evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEvaluationRequest {
    /// Peer node ID
    pub peer_id: String,

    /// Peer family ID (v3.14.1 - tag-based identity)
    ///
    /// Extracted from peer tags (e.g., "beardog:family:nat0" → "nat0")
    /// Songbird doesn't interpret this - just extracts and passes to `BearDog`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_family: Option<String>,

    /// Peer tags (includes security provider encryption tag if present)
    ///
    /// Example: `["crypto:family:a3f2", "encryption_enabled"]`
    pub peer_tags: Vec<String>,

    /// Connection information (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_info: Option<HashMap<String, String>>,

    /// Additional context for trust evaluation (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<HashMap<String, String>>,
}

/// Response from trust evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEvaluationResponse {
    /// Decision: "`auto_accept`", "`prompt_user`", or "reject"
    pub decision: String,

    /// Trust level: `TrustLevel` enum (accepts both integer and string via Phase 1 custom deserializer)
    pub trust_level: TrustLevel,

    /// Reason for decision
    pub reason: String,

    /// Suggested next action (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_action: Option<String>,

    /// Additional metadata (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
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
}

impl TrustEvaluationRequest {
    /// Create a new trust evaluation request
    pub fn new(peer_id: impl Into<String>, peer_tags: Vec<String>) -> Self {
        Self {
            peer_id: peer_id.into(),
            peer_family: None, // ✅ Initialize to None (v3.14.1)
            peer_tags,
            connection_info: None,
            context: None,
        }
    }

    /// Set peer family ID (v3.14.1 - tag-based identity)
    #[must_use]
    pub fn with_peer_family(mut self, family: impl Into<String>) -> Self {
        self.peer_family = Some(family.into());
        self
    }

    /// Add connection information
    #[must_use]
    pub fn with_connection_info(mut self, info: HashMap<String, String>) -> Self {
        self.connection_info = Some(info);
        self
    }

    /// Add context
    #[must_use]
    pub fn with_context(mut self, context: HashMap<String, String>) -> Self {
        self.context = Some(context);
        self
    }
}

impl TrustEvaluationResponse {
    /// Check if the decision is to auto-accept
    #[must_use]
    pub fn is_auto_accept(&self) -> bool {
        self.decision == "auto_accept"
    }

    /// Check if the decision is to reject
    #[must_use]
    pub fn is_reject(&self) -> bool {
        self.decision == "reject"
    }

    /// Check if user prompt is required
    #[must_use]
    pub fn requires_user_prompt(&self) -> bool {
        self.decision == "prompt_user"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_request_creation() {
        let request = TrustEvaluationRequest::new("tower2", vec!["crypto:family:a3f2".to_string()]);

        assert_eq!(request.peer_id, "tower2");
        assert_eq!(request.peer_tags.len(), 1);
        assert!(request.connection_info.is_none());
    }

    #[test]
    fn test_trust_request_with_context() {
        let mut context = HashMap::new();
        context.insert("source".to_string(), "discovery".to_string());

        let request = TrustEvaluationRequest::new("tower2", vec!["crypto:family:a3f2".to_string()])
            .with_context(context);

        assert!(request.context.is_some());
    }

    #[test]
    fn test_trust_response_is_auto_accept() {
        let response = TrustEvaluationResponse {
            decision: "auto_accept".to_string(),
            trust_level: TrustLevel::Highest, // Phase 1: Use enum
            reason: "Same family".to_string(),
            suggested_action: None,
            metadata: None,
        };

        assert!(response.is_auto_accept());
        assert!(!response.is_reject());
        assert!(!response.requires_user_prompt());
    }

    #[test]
    fn test_trust_response_is_reject() {
        let response = TrustEvaluationResponse {
            decision: "reject".to_string(),
            trust_level: TrustLevel::None, // Phase 1: Use enum
            reason: "Unknown peer".to_string(),
            suggested_action: None,
            metadata: None,
        };

        assert!(!response.is_auto_accept());
        assert!(response.is_reject());
        assert!(!response.requires_user_prompt());
    }
}
