// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Peer trust DTOs for discovery and security-provider evaluation.

use serde::{Deserialize, Serialize};

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
