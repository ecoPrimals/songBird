// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::BirdSongHandler;
use super::types::{GenerateBeaconRequest, validate_required_fields};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use songbird_discovery::birdsong::BirdSongEncryption;
use tracing::{debug, info};

impl BirdSongHandler {
    /// Handle `birdsong.generate_encrypted_beacon`
    ///
    /// Generates a family-encrypted beacon for broadcast.
    /// Only family members can decrypt this beacon.
    ///
    /// Deep debt: Production implementation, complete with no mocks
    pub async fn handle_generate_encrypted_beacon(&self, params: Value) -> Result<Value, String> {
        debug!("🌲 RPC: birdsong.generate_encrypted_beacon");

        validate_required_fields(&params, &["node_id"])?;

        let request: GenerateBeaconRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        // Get provider (lazy init, runtime discovery)
        let provider = self.get_provider().await?;

        // Build discovery message (JSON)
        // Feb 6, 2026: Added onion_endpoint for Sovereign Onion Service
        // Dark Forest: Only family members can see this endpoint (encrypted beacon)
        let discovery_message = json!({
            "node_id": request.node_id,
            "capabilities": request.capabilities,
            "onion_endpoint": request.onion_endpoint,  // Sovereign Onion address (optional)
            "endpoint_hints": request.endpoint_hints,  // Additional connection hints
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": env!("CARGO_PKG_VERSION"),
        });

        let plaintext = serde_json::to_vec(&discovery_message)
            .map_err(|e| format!("Failed to serialize discovery message: {e}"))?;

        // Encrypt via security provider (Pure Rust, Unix socket IPC)
        let encrypted = provider
            .encrypt_discovery(&plaintext)
            .await
            .map_err(|e| format!("Encryption failed: {e}"))?;

        // Encode to base64 for JSON transport
        let encrypted_b64 = STANDARD.encode(&encrypted);

        // Get family ID for plaintext header
        let family_id = provider.family_id().unwrap_or_else(|| "unknown".to_string());

        info!(
            "✅ Generated encrypted beacon for node: {} (family: {}, size: {} bytes)",
            request.node_id,
            family_id,
            encrypted.len()
        );

        Ok(json!({
            "encrypted_beacon": encrypted_b64,
            "family_id": family_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "node_id": request.node_id,
            "beacon_size_bytes": encrypted.len(),
        }))
    }
}
