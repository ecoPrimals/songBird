// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! P2P / family discovery request and response types.

use serde::{Deserialize, Serialize};

/// Request to discover nodes by genetic family tags
///
/// ## Example
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "discover_by_family",
///   "params": {
///     "family_tags": ["my-family", "lan0"],
///     "timeout_ms": 5000
///   },
///   "id": 1
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverByFamilyRequest {
    /// Family tags to filter by (e.g., ["my-family", "lan0"])
    pub family_tags: Vec<String>,

    /// Timeout in milliseconds (optional, default: 5000)
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

const fn default_timeout() -> u64 {
    5000
}

/// Response containing discovered nodes filtered by family
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverByFamilyResponse {
    /// Discovered nodes matching family tags
    pub nodes: Vec<DiscoveredNode>,
}

/// Discovered node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredNode {
    /// Unique node identifier
    pub node_id: String,

    /// Human-readable node name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,

    /// Genetic families this node belongs to
    pub genetic_families: Vec<String>,

    /// Sub-federations this node is part of (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sub_federations: Vec<String>,

    /// Capabilities offered by this node
    pub capabilities: Vec<String>,

    /// BTSP endpoint for encrypted tunnels (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub btsp_endpoint: Option<String>,

    /// HTTPS endpoint (fallback)
    pub https_endpoint: String,

    /// When this node was last seen (ISO 8601)
    pub last_seen: String,
}
