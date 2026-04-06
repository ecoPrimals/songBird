// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::BirdSongHandler;
use super::types::{DecryptBeaconRequest, validate_required_fields};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use songbird_discovery::birdsong::BirdSongEncryption;
use tracing::{debug, error, info};

impl BirdSongHandler {
    /// Handle `birdsong.decrypt_beacon`
    ///
    /// Attempts to decrypt a received beacon.
    /// Returns success=true only if beacon is from family member.
    ///
    /// Deep debt: Family gate, graceful failure, no information leakage
    pub async fn handle_decrypt_beacon(&self, params: Value) -> Result<Value, String> {
        debug!("🔐 RPC: birdsong.decrypt_beacon");

        validate_required_fields(&params, &["encrypted_beacon"])?;

        let request: DecryptBeaconRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        // Get provider
        let provider = self.get_provider().await?;

        // Decode base64
        let encrypted = STANDARD
            .decode(&request.encrypted_beacon)
            .map_err(|e| format!("Invalid base64: {e}"))?;

        // Attempt decryption (family gate)
        match provider.decrypt_discovery(&encrypted).await {
            Ok(Some(plaintext)) => {
                // SUCCESS: Same family, can decrypt
                let discovery_message: serde_json::Value = serde_json::from_slice(&plaintext)
                    .map_err(|e| format!("Decrypted but invalid JSON: {e}"))?;

                let node_id =
                    discovery_message["node_id"].as_str().unwrap_or("unknown").to_string();

                let capabilities: Vec<String> = discovery_message["capabilities"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(String::from).collect())
                    .unwrap_or_default();

                // Feb 6, 2026: Extract onion endpoint for Sovereign NAT traversal
                let onion_endpoint = discovery_message["onion_endpoint"].as_str().map(String::from);

                let endpoint_hints = discovery_message.get("endpoint_hints").cloned();

                if let Some(ref onion) = onion_endpoint {
                    info!(
                        "✅ Decrypted beacon from family member: {} (onion: {}, capabilities: {:?})",
                        node_id, onion, capabilities
                    );
                } else {
                    info!(
                        "✅ Decrypted beacon from family member: {} (capabilities: {:?})",
                        node_id, capabilities
                    );
                }

                Ok(json!({
                    "success": true,
                    "is_family": true,
                    "node_id": node_id,
                    "capabilities": capabilities,
                    "onion_endpoint": onion_endpoint,
                    "endpoint_hints": endpoint_hints,
                    "timestamp": discovery_message["timestamp"],
                }))
            }
            Ok(None) => {
                // GRACEFUL FAILURE: Different family (cannot decrypt)
                // No information leakage - just return false
                debug!("⛔ Beacon is from different family (graceful ignore)");

                Ok(json!({
                    "success": false,
                    "is_family": false,
                    "reason": "different_family",
                }))
            }
            Err(e) => {
                // SYSTEM ERROR (not decryption failure)
                error!("❌ System error during decryption: {}", e);
                Err(format!("Decryption system error: {e}"))
            }
        }
    }
}
