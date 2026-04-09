// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! BTSP / genetic tunnel JSON-RPC types.

use serde::{Deserialize, Serialize};

/// Request to create an encrypted BTSP tunnel using genetic proof
///
/// ## Example
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "create_genetic_tunnel",
///   "params": {
///     "peer_node_id": "node-beta",
///     "peer_endpoint": "udp://192.168.1.101:4433",
///     "genetic_proof": {
///       "family_id": "my-family",
///       "parent_seed_hash": "abc123",
///       "relationship": "sibling"
///     }
///   },
///   "id": 2
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGeneticTunnelRequest {
    /// Target peer node ID
    pub peer_node_id: String,

    /// Peer endpoint (optional, will use discovered endpoint if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_endpoint: Option<String>,

    /// Genetic proof from `security provider` (optional, will verify via `security provider` if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub genetic_proof: Option<GeneticProof>,
}

/// Genetic lineage proof from `security provider`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneticProof {
    /// Family identifier (e.g., "my-family")
    pub family_id: String,

    /// Parent seed hash (from `security provider` verification)
    pub parent_seed_hash: String,

    /// Relationship (e.g., "sibling", "parent", "child")
    pub relationship: String,
}

/// Response after creating a BTSP tunnel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGeneticTunnelResponse {
    /// Unique tunnel identifier
    pub tunnel_id: String,

    /// Tunnel status ("establishing", "established", "failed")
    pub status: String,

    /// Local endpoint for this tunnel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_endpoint: Option<String>,

    /// Remote peer endpoint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_endpoint: Option<String>,

    /// Encryption algorithm (from `security provider`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<String>,

    /// When the tunnel was created (ISO 8601)
    pub created_at: String,
}
