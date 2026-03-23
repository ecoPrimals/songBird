// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

#![cfg_attr(
    test,
    expect(clippy::float_cmp, reason = "test: exact float comparison is intentional")
)]
//! Peer Trust Evaluation
//!
//! Evaluates whether to trust discovered peers by consulting the security provider (security provider).
//! This is part of the USB seed integration - security provider makes the trust decision based on
//! genetic lineage derived from the USB family seed.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::security_capability_client::{
    ConnectionInfo, SecurityCapabilityClient, TrustEvaluationRequest, TrustEvaluationResponse,
};

/// Result of peer trust evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerTrustDecision {
    /// Automatically accept this peer (same family, high trust)
    AutoAccept {
        /// Reason for auto-acceptance
        reason: String,
        /// Confidence score (0.0-1.0)
        confidence: f64,
        /// Encryption tag if available
        encryption_tag: Option<String>,
    },

    /// Prompt user for consent (different family or unknown)
    PromptUser {
        /// Reason for prompting
        reason: String,
        /// Peer information for user review
        peer_id: String,
        /// Recommended action
        recommendation: String,
    },

    /// Reject this peer (no lineage, untrusted, security concern)
    Reject {
        /// Reason for rejection
        reason: String,
        /// Trust level
        trust_level: String,
    },
}

/// Discovered peer information
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    /// Peer node ID
    pub node_id: String,
    /// Peer tags (including security provider encryption tags) - legacy
    pub tags: Vec<String>,
    /// Identity attestations (generic, structured) - NEW
    pub identity_attestations: Vec<crate::trust::UniversalIdentityAttestation>,
    /// Peer endpoint
    pub endpoint: String,
    /// Peer capabilities
    pub capabilities: Vec<String>,
    /// Discovery method
    pub discovery_method: String,
    /// When first seen
    pub first_seen_at: u64,
}

/// Evaluate whether to trust a discovered peer
///
/// Consults security provider (security provider) for trust decision.
/// security provider checks genetic lineage and returns `auto_accept/prompt_user/reject`.
///
/// # Architecture
///
/// - **Songbird**: Discovers peers, asks "should I trust?"
/// - **security provider**: Knows lineage, answers yes/no/prompt
/// - **Clean API**: Songbird doesn't need crypto knowledge
///
/// # Arguments
///
/// * `peer` - Discovered peer information
/// * `security provider_client` - Client for querying security provider
///
/// # Returns
///
/// Decision on whether to trust the peer
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn evaluate_peer_trust(
    peer: &DiscoveredPeer,
    security_client: &SecurityCapabilityClient,
) -> Result<PeerTrustDecision> {
    info!("🔍 Evaluating trust for peer: {}", peer.node_id);

    // Extract peer family from tags (v3.14.1 - tag-based identity)
    // Tags format: "beardog:family:nat0" or "beardog:family:acmecorp"
    let peer_family = extract_family_from_tags(&peer.tags);

    if let Some(ref family) = peer_family {
        info!("🏷️  Peer {} family extracted from tags: {}", peer.node_id, family);
    } else {
        warn!("⚠️  Peer {} has no family tag - security provider will reject", peer.node_id);
    }

    // Build trust evaluation request
    let request = TrustEvaluationRequest {
        peer_id: peer.node_id.clone(),
        peer_family, // ✅ NOW PROVIDED! (v3.14.1 fix)
        peer_tags: peer.tags.clone(),
        connection_info: Some(ConnectionInfo {
            endpoint: peer.endpoint.clone(),
            protocol: "tarpc".to_string(), // or "http" based on config
        }),
        context: Some({
            let mut ctx = std::collections::HashMap::new();
            ctx.insert("discovery_method".to_string(), peer.discovery_method.clone());
            ctx.insert("first_seen_at".to_string(), peer.first_seen_at.to_string());
            ctx
        }),
    };

    // Ask security provider: "Should I trust this peer?"
    match security_client.evaluate_trust(&request).await {
        Ok(response) => {
            // Handle security provider's decision
            handle_trust_response(&peer.node_id, response)
        }
        Err(e) => {
            // security provider unavailable - default to prompting user
            warn!("⚠️ security provider unavailable for peer {}: {}", peer.node_id, e);
            warn!("   Defaulting to prompt user (safe default)");

            Ok(PeerTrustDecision::PromptUser {
                reason: "security_provider_unavailable".to_string(),
                peer_id: peer.node_id.clone(),
                recommendation: "neutral".to_string(),
            })
        }
    }
}

/// Handle trust response from security provider
fn handle_trust_response(
    peer_id: &str,
    response: TrustEvaluationResponse,
) -> Result<PeerTrustDecision> {
    match response.decision.as_str() {
        "auto_accept" => {
            info!("✅ security provider says AUTO-ACCEPT peer {} ({})", peer_id, response.reason);
            info!(
                "   Trust level: {} | Confidence: {:.2}",
                response.trust_level, response.confidence
            );

            Ok(PeerTrustDecision::AutoAccept {
                reason: response.reason,
                confidence: response.confidence,
                encryption_tag: response.encryption_tag,
            })
        }

        "prompt_user" => {
            info!(
                "⚠️ security provider says PROMPT USER for peer {} ({})",
                peer_id, response.reason
            );
            info!(
                "   Trust level: {} | Confidence: {:.2}",
                response.trust_level, response.confidence
            );

            Ok(PeerTrustDecision::PromptUser {
                reason: response.reason.clone(),
                peer_id: peer_id.to_string(),
                recommendation: if response.confidence > 0.5 {
                    "accept".to_string()
                } else {
                    "neutral".to_string()
                },
            })
        }

        "reject" => {
            warn!("❌ security provider says REJECT peer {} ({})", peer_id, response.reason);
            warn!(
                "   Trust level: {} | Confidence: {:.2}",
                response.trust_level, response.confidence
            );

            Ok(PeerTrustDecision::Reject {
                reason: response.reason,
                trust_level: response.trust_level,
            })
        }

        unknown => {
            warn!("❓ Unknown decision from security provider: {}", unknown);
            warn!("   Defaulting to prompt user (safe default)");

            Ok(PeerTrustDecision::PromptUser {
                reason: format!("unknown_decision: {unknown}"),
                peer_id: peer_id.to_string(),
                recommendation: "reject".to_string(),
            })
        }
    }
}

/// Extract family ID from peer tags (v3.14.1)
///
/// Tags format: "beardog:family:nat0" or "beardog:family:acmecorp"
/// Songbird doesn't interpret these tags - it just extracts and passes to security provider.
///
/// # Arguments
///
/// * `tags` - Peer tags from discovery
///
/// # Returns
///
/// Family ID if found in tags, `None` otherwise
fn extract_family_from_tags(tags: &[String]) -> Option<String> {
    const FAMILY_TAG_PREFIX: &str = "beardog:family:";

    for tag in tags {
        if let Some(family_id) = tag.strip_prefix(FAMILY_TAG_PREFIX)
            && !family_id.is_empty()
        {
            return Some(family_id.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![expect(clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[test]
    fn test_extract_family_from_tags_found() {
        let tags = vec![
            "some:other:tag".to_string(),
            "beardog:family:nat0".to_string(),
            "another:tag".to_string(),
        ];

        let family = extract_family_from_tags(&tags);
        assert_eq!(family, Some("nat0".to_string()));
    }

    #[test]
    fn test_extract_family_from_tags_not_found() {
        let tags = vec!["some:other:tag".to_string(), "another:tag".to_string()];

        let family = extract_family_from_tags(&tags);
        assert_eq!(family, None);
    }

    #[test]
    fn test_extract_family_from_tags_empty_family() {
        let tags = vec![
            "beardog:family:".to_string(), // Empty family ID
        ];

        let family = extract_family_from_tags(&tags);
        assert_eq!(family, None); // Should ignore empty family
    }

    #[test]
    fn test_extract_family_from_tags_multiple_families() {
        let tags = vec![
            "beardog:family:nat0".to_string(),
            "beardog:family:acmecorp".to_string(), // Second family (should be ignored)
        ];

        let family = extract_family_from_tags(&tags);
        assert_eq!(family, Some("nat0".to_string())); // Returns first match
    }

    #[test]
    fn test_extract_family_from_tags_complex_family_id() {
        let tags = vec!["beardog:family:acmecorp-engineering-prod".to_string()];

        let family = extract_family_from_tags(&tags);
        assert_eq!(family, Some("acmecorp-engineering-prod".to_string()));
    }

    #[test]
    fn test_peer_trust_decision_types() {
        // Test decision types exist and can be created
        let _auto = PeerTrustDecision::AutoAccept {
            reason: "test".to_string(),
            confidence: 1.0,
            encryption_tag: None,
        };

        let _prompt = PeerTrustDecision::PromptUser {
            reason: "test".to_string(),
            peer_id: "peer1".to_string(),
            recommendation: "neutral".to_string(),
        };

        let _reject = PeerTrustDecision::Reject {
            reason: "test".to_string(),
            trust_level: "none".to_string(),
        };
    }

    #[test]
    fn test_handle_auto_accept_response() {
        use std::collections::HashMap;

        let response = TrustEvaluationResponse {
            decision: "auto_accept".to_string(),
            trust_level: "high".to_string(),
            confidence: 1.0,
            reason: "same_genetic_family".to_string(),
            encryption_tag: Some("beardog:family:a3f2".to_string()),
            metadata: HashMap::new(),
        };

        let decision = handle_trust_response("peer1", response).expect("handle");

        match decision {
            PeerTrustDecision::AutoAccept {
                reason,
                confidence,
                ..
            } => {
                assert_eq!(reason, "same_genetic_family");
                assert_eq!(confidence, 1.0);
            }
            _ => panic!("Expected AutoAccept"),
        }
    }

    #[test]
    fn test_handle_prompt_user_response() {
        use std::collections::HashMap;

        let response = TrustEvaluationResponse {
            decision: "prompt_user".to_string(),
            trust_level: "low".to_string(),
            confidence: 0.5,
            reason: "different_genetic_family".to_string(),
            encryption_tag: None,
            metadata: HashMap::new(),
        };

        let decision = handle_trust_response("peer2", response).expect("handle");

        match decision {
            PeerTrustDecision::PromptUser {
                reason,
                ..
            } => {
                assert_eq!(reason, "different_genetic_family");
            }
            _ => panic!("Expected PromptUser"),
        }
    }

    #[test]
    fn test_handle_reject_response() {
        use std::collections::HashMap;

        let response = TrustEvaluationResponse {
            decision: "reject".to_string(),
            trust_level: "none".to_string(),
            confidence: 0.0,
            reason: "no_genetic_lineage".to_string(),
            encryption_tag: None,
            metadata: HashMap::new(),
        };

        let decision = handle_trust_response("peer3", response).expect("handle");

        match decision {
            PeerTrustDecision::Reject {
                reason,
                ..
            } => {
                assert_eq!(reason, "no_genetic_lineage");
            }
            _ => panic!("Expected Reject"),
        }
    }

    #[test]
    fn test_handle_unknown_response() {
        use std::collections::HashMap;

        let response = TrustEvaluationResponse {
            decision: "unknown_decision".to_string(),
            trust_level: "unknown".to_string(),
            confidence: 0.0,
            reason: "test".to_string(),
            encryption_tag: None,
            metadata: HashMap::new(),
        };

        let decision = handle_trust_response("peer4", response).expect("handle");

        // Unknown decisions should default to PromptUser (safe default)
        match decision {
            PeerTrustDecision::PromptUser {
                ..
            } => {}
            _ => panic!("Expected PromptUser for unknown decision"),
        }
    }

    #[test]
    fn handle_prompt_user_high_confidence_recommends_accept() {
        use std::collections::HashMap;
        let response = TrustEvaluationResponse {
            decision: "prompt_user".to_string(),
            trust_level: "medium".to_string(),
            confidence: 0.9,
            reason: "r".to_string(),
            encryption_tag: None,
            metadata: HashMap::new(),
        };
        let decision = handle_trust_response("p", response).expect("handle");
        match decision {
            PeerTrustDecision::PromptUser {
                recommendation,
                ..
            } => assert_eq!(recommendation, "accept"),
            _ => panic!("expected prompt"),
        }
    }

    #[test]
    fn handle_prompt_user_low_confidence_recommends_neutral() {
        use std::collections::HashMap;
        let response = TrustEvaluationResponse {
            decision: "prompt_user".to_string(),
            trust_level: "low".to_string(),
            confidence: 0.4,
            reason: "r".to_string(),
            encryption_tag: None,
            metadata: HashMap::new(),
        };
        let decision = handle_trust_response("p", response).expect("handle");
        match decision {
            PeerTrustDecision::PromptUser {
                recommendation,
                ..
            } => assert_eq!(recommendation, "neutral"),
            _ => panic!("expected prompt"),
        }
    }

    #[test]
    fn extract_family_ignores_wrong_prefix() {
        let tags = vec!["Beardog:family:nat0".to_string()];
        assert_eq!(extract_family_from_tags(&tags), None);
    }

    #[test]
    fn extract_family_from_tags_empty_slice() {
        let tags: Vec<String> = vec![];
        assert_eq!(extract_family_from_tags(&tags), None);
    }

    #[test]
    fn handle_prompt_user_confidence_boundary_half_recommends_neutral() {
        use std::collections::HashMap;
        let response = TrustEvaluationResponse {
            decision: "prompt_user".to_string(),
            trust_level: "low".to_string(),
            confidence: 0.5,
            reason: "boundary".to_string(),
            encryption_tag: None,
            metadata: HashMap::new(),
        };
        let decision = handle_trust_response("peer-b", response).expect("handle");
        match decision {
            PeerTrustDecision::PromptUser {
                recommendation,
                ..
            } => assert_eq!(recommendation, "neutral"),
            _ => panic!("expected prompt"),
        }
    }

    #[test]
    fn handle_prompt_user_confidence_above_half_recommends_accept() {
        use std::collections::HashMap;
        let response = TrustEvaluationResponse {
            decision: "prompt_user".to_string(),
            trust_level: "low".to_string(),
            confidence: 0.5000001,
            reason: "boundary".to_string(),
            encryption_tag: None,
            metadata: HashMap::new(),
        };
        let decision = handle_trust_response("peer-b2", response).expect("handle");
        match decision {
            PeerTrustDecision::PromptUser {
                recommendation,
                ..
            } => assert_eq!(recommendation, "accept"),
            _ => panic!("expected prompt"),
        }
    }

    #[test]
    fn handle_prompt_user_confidence_just_below_half_neutral() {
        use std::collections::HashMap;
        let response = TrustEvaluationResponse {
            decision: "prompt_user".to_string(),
            trust_level: "low".to_string(),
            confidence: 0.49,
            reason: "boundary".to_string(),
            encryption_tag: None,
            metadata: HashMap::new(),
        };
        let decision = handle_trust_response("peer-c", response).expect("handle");
        match decision {
            PeerTrustDecision::PromptUser {
                recommendation,
                ..
            } => assert_eq!(recommendation, "neutral"),
            _ => panic!("expected prompt"),
        }
    }

    #[test]
    fn peer_trust_decision_serde_roundtrip_auto_accept() {
        let d = PeerTrustDecision::AutoAccept {
            reason: "r".to_string(),
            confidence: 0.75,
            encryption_tag: Some("t".to_string()),
        };
        let json = serde_json::to_string(&d).expect("serialize");
        let back: PeerTrustDecision = serde_json::from_str(&json).expect("deserialize");
        match back {
            PeerTrustDecision::AutoAccept {
                reason,
                confidence,
                ..
            } => {
                assert_eq!(reason, "r");
                assert!((confidence - 0.75).abs() < f64::EPSILON);
            }
            _ => panic!("Expected AutoAccept"),
        }
    }

    #[test]
    fn unknown_decision_uses_reject_recommendation() {
        use std::collections::HashMap;
        let response = TrustEvaluationResponse {
            decision: "weird".to_string(),
            trust_level: "low".to_string(),
            confidence: 0.1,
            reason: "x".to_string(),
            encryption_tag: None,
            metadata: HashMap::new(),
        };
        let decision = handle_trust_response("peer-x", response).expect("handle");
        match decision {
            PeerTrustDecision::PromptUser {
                recommendation,
                reason,
                ..
            } => {
                assert_eq!(recommendation, "reject");
                assert!(reason.contains("unknown_decision"));
            }
            _ => panic!("expected PromptUser"),
        }
    }

    #[test]
    fn reject_response_preserves_trust_level_string() {
        use std::collections::HashMap;
        let response = TrustEvaluationResponse {
            decision: "reject".to_string(),
            trust_level: "blocked".to_string(),
            confidence: 0.0,
            reason: "bad".to_string(),
            encryption_tag: None,
            metadata: HashMap::new(),
        };
        let decision = handle_trust_response("peer-r", response).expect("handle");
        match decision {
            PeerTrustDecision::Reject {
                trust_level,
                reason,
            } => {
                assert_eq!(trust_level, "blocked");
                assert_eq!(reason, "bad");
            }
            _ => panic!("expected Reject"),
        }
    }

    #[test]
    fn auto_accept_passes_through_encryption_tag() {
        use std::collections::HashMap;
        let response = TrustEvaluationResponse {
            decision: "auto_accept".to_string(),
            trust_level: "high".to_string(),
            confidence: 1.0,
            reason: "ok".to_string(),
            encryption_tag: Some("tag-123".to_string()),
            metadata: HashMap::new(),
        };
        let decision = handle_trust_response("peer-a", response).expect("handle");
        match decision {
            PeerTrustDecision::AutoAccept {
                encryption_tag,
                ..
            } => assert_eq!(encryption_tag.as_deref(), Some("tag-123")),
            _ => panic!("expected AutoAccept"),
        }
    }

    #[test]
    fn peer_trust_decision_serde_roundtrip_prompt_and_reject() {
        let p = PeerTrustDecision::PromptUser {
            reason: "ask".to_string(),
            peer_id: "p1".to_string(),
            recommendation: "neutral".to_string(),
        };
        let pj = serde_json::to_string(&p).expect("serialize");
        let pr: PeerTrustDecision = serde_json::from_str(&pj).expect("deserialize");
        assert!(matches!(pr, PeerTrustDecision::PromptUser { .. }));

        let r = PeerTrustDecision::Reject {
            reason: "no".to_string(),
            trust_level: "none".to_string(),
        };
        let rj = serde_json::to_string(&r).expect("serialize");
        let rr: PeerTrustDecision = serde_json::from_str(&rj).expect("deserialize");
        assert!(matches!(rr, PeerTrustDecision::Reject { .. }));
    }

    #[test]
    fn prompt_user_peer_id_matches_input() {
        use std::collections::HashMap;
        let response = TrustEvaluationResponse {
            decision: "prompt_user".to_string(),
            trust_level: "low".to_string(),
            confidence: 0.2,
            reason: "review".to_string(),
            encryption_tag: None,
            metadata: HashMap::new(),
        };
        let decision = handle_trust_response("my-peer-id", response).expect("handle");
        match decision {
            PeerTrustDecision::PromptUser {
                peer_id,
                ..
            } => assert_eq!(peer_id, "my-peer-id"),
            _ => panic!("expected prompt"),
        }
    }
}
