// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use serde::Deserialize;
use serde_json::Value;
use songbird_discovery::security_birdsong_provider::SecurityBirdSongProvider;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// `BirdSong` handler for encrypted discovery
///
/// Manages family-encrypted discovery beacons using `security provider`'s genetic lineage crypto.
/// All operations discover `security provider` at runtime (no hardcoding).
#[derive(Default)]
pub struct BirdSongHandler {
    /// Cached `security provider` socket path (runtime discovered)
    pub(super) security_socket: Arc<RwLock<Option<PathBuf>>>,

    /// Cached `BirdSong` provider (lazy initialization)
    pub(super) provider: Arc<RwLock<Option<Arc<SecurityBirdSongProvider>>>>,
}

impl BirdSongHandler {
    /// Create new `BirdSong` handler
    ///
    /// Deep debt compliance:
    /// - No hardcoded paths
    /// - Lazy initialization
    /// - Runtime discovery
    #[must_use]
    pub fn new() -> Self {
        Self {
            security_socket: Arc::new(RwLock::new(None)),
            provider: Arc::new(RwLock::new(None)),
        }
    }
}

/// Pre-validate that all required fields are present in the JSON params,
/// reporting **all** missing fields in a single error message.
///
/// Standard serde deserialization reports one missing field at a time,
/// requiring multiple round-trips during integration debugging. This
/// pre-validation collects every missing field into one diagnostic.
pub(super) fn validate_required_fields(params: &Value, required: &[&str]) -> Result<(), String> {
    let Some(obj) = params.as_object() else {
        return Err(String::from("Invalid params: expected JSON object"));
    };

    let missing: Vec<&str> =
        required.iter().filter(|&&field| !obj.contains_key(field)).copied().collect();

    if missing.is_empty() {
        Ok(())
    } else if missing.len() == 1 {
        Err(format!("Missing required field: {}", missing[0]))
    } else {
        Err(format!("Missing required fields: {}", missing.join(", ")))
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct GenerateBeaconRequest {
    pub(super) node_id: String,
    #[serde(default)]
    pub(super) capabilities: Vec<String>,
    /// Sovereign Onion endpoint (e.g., "abc123...xyz.onion:3492")
    /// Dark Forest: Only visible to family members (beacon is encrypted)
    #[serde(default)]
    pub(super) onion_endpoint: Option<String>,
    /// Additional endpoint hints (LAN IP, port, etc.)
    #[serde(default)]
    pub(super) endpoint_hints: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub(super) struct DecryptBeaconRequest {
    pub(super) encrypted_beacon: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct VerifyLineageRequest {
    pub(super) peer_node_id: String,
    pub(super) our_node_id: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct GetLineageRequest {
    // Empty for now, may add filters later
}
