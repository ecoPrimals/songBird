// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Capability announcement JSON-RPC types.

use serde::{Deserialize, Serialize};

/// Request to announce capabilities and genetic families
///
/// ## Example
///
/// ```json
/// {
///   "jsonrpc": "2.0",
///   "method": "announce_capabilities",
///   "params": {
///     "capabilities": ["storage", "compute"],
///     "sub_federations": ["gaming", "family"],
///     "genetic_families": ["my-family", "lan0"]
///   },
///   "id": 3
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceCapabilitiesRequest {
    /// Capabilities to announce
    pub capabilities: Vec<String>,

    /// Sub-federations this node is part of (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sub_federations: Vec<String>,

    /// Genetic families this node belongs to (optional)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genetic_families: Vec<String>,
}

/// Response after updating capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceCapabilitiesResponse {
    /// Status ("updated", "broadcasting", "failed")
    pub status: String,

    /// Whether broadcasting is active
    pub broadcasting: bool,

    /// When the update was applied (ISO 8601)
    pub updated_at: String,
}
