// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Trust evaluation pipeline: tags → security provider → [`PeerTrustDecision`](super::types::PeerTrustDecision).

use anyhow::Result;
use tracing::{info, warn};

use crate::security_capability_client::{
    ConnectionInfo, SecurityCapabilityClient, TrustEvaluationRequest, TrustEvaluationResponse,
};

use super::types::{DiscoveredPeer, PeerTrustDecision};

/// Evaluate whether to trust a discovered peer
///
/// Consults security provider (security provider) for trust decision.
/// security provider checks genetic lineage and returns `auto_accept/prompt_user/reject`.
///
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn evaluate_peer_trust(
    peer: &DiscoveredPeer,
    security_client: &SecurityCapabilityClient,
) -> Result<PeerTrustDecision> {
    info!("🔍 Evaluating trust for peer: {}", peer.node_id);

    let peer_family = extract_family_from_tags(&peer.tags);

    if let Some(ref family) = peer_family {
        info!("🏷️  Peer {} family extracted from tags: {}", peer.node_id, family);
    } else {
        warn!("⚠️  Peer {} has no family tag - security provider will reject", peer.node_id);
    }

    let request = TrustEvaluationRequest {
        peer_id: peer.node_id.clone(),
        peer_family,
        peer_tags: peer.tags.clone(),
        connection_info: Some(ConnectionInfo {
            endpoint: peer.endpoint.clone(),
            protocol: String::from("tarpc"),
        }),
        context: Some({
            let mut ctx = std::collections::HashMap::new();
            ctx.insert(String::from("discovery_method"), peer.discovery_method.clone());
            ctx.insert(String::from("first_seen_at"), peer.first_seen_at.to_string());
            ctx
        }),
    };

    match security_client.evaluate_trust(&request).await {
        Ok(response) => handle_trust_response(&peer.node_id, response),
        Err(e) => {
            warn!("⚠️ security provider unavailable for peer {}: {}", peer.node_id, e);
            warn!("   Defaulting to prompt user (safe default)");

            Ok(PeerTrustDecision::PromptUser {
                reason: String::from("security_provider_unavailable"),
                peer_id: peer.node_id.clone(),
                recommendation: String::from("neutral"),
            })
        }
    }
}

/// Handle trust response from security provider
pub fn handle_trust_response(
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
                    String::from("accept")
                } else {
                    String::from("neutral")
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
                recommendation: String::from("reject"),
            })
        }
    }
}

/// Extract family ID from peer tags (v3.14.1)
pub fn extract_family_from_tags(tags: &[String]) -> Option<String> {
    const FAMILY_TAG_PREFIXES: &[&str] = &["crypto:family:", "security:family:", "beardog:family:"];

    for tag in tags {
        for prefix in FAMILY_TAG_PREFIXES {
            if let Some(family_id) = tag.strip_prefix(prefix)
                && !family_id.is_empty()
            {
                return Some(family_id.to_string());
            }
        }
    }

    None
}
