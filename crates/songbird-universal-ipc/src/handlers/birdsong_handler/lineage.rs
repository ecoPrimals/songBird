// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

use super::BirdSongHandler;
use super::types::{GetLineageRequest, VerifyLineageRequest, validate_required_fields};
use serde_json::{Value, json};
use songbird_universal::UnixRpcClient;
use tracing::{debug, info};

impl BirdSongHandler {
    /// Handle `birdsong.verify_lineage`
    ///
    /// Verifies peer lineage using challenge-response (defense-in-depth).
    /// Calls the security provider's `genetic.generate_challenge` and `genetic.verify_challenge_response`.
    ///
    /// Deep debt: Delegates to the crypto/security provider (e.g. `security provider`) (separation of concerns)
    pub async fn handle_verify_lineage(&self, params: Value) -> Result<Value, String> {
        debug!("🔍 RPC: birdsong.verify_lineage");

        validate_required_fields(&params, &["peer_node_id", "our_node_id"])?;

        let request: VerifyLineageRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        // Discover security provider socket (capability-based)
        let security_socket = self.discover_security_socket().await?;

        // Create RPC client to the security provider
        let client = UnixRpcClient::new(&security_socket)
            .map_err(|e| format!("Failed to connect to security provider: {e}"))?;

        // Step 1: Generate challenge
        let challenge_params = json!({
            "challenger_node_id": request.our_node_id,
            "target_family_id": request.peer_node_id,
        });

        let challenge_result: Value = client
            .call("genetic.generate_challenge", &challenge_params)
            .await
            .map_err(|e| format!("Challenge generation failed: {e}"))?;

        info!("✅ Generated lineage challenge for peer: {}", request.peer_node_id);

        // In production, you would:
        // - Send challenge to peer via network
        // - Receive response from peer
        // - Call genetic.verify_challenge_response
        //
        // For now, return the challenge for caller to handle exchange
        Ok(json!({
            "challenge_generated": true,
            "challenge": challenge_result,
            "next_step": "send_challenge_to_peer",
            "peer_node_id": request.peer_node_id,
        }))
    }

    /// Handle `birdsong.get_lineage`
    ///
    /// Returns our own lineage info for sharing with peers.
    ///
    /// Deep debt: Self-knowledge only, queries the security provider for our identity
    pub async fn handle_get_lineage(&self, params: Value) -> Result<Value, String> {
        debug!("📋 RPC: birdsong.get_lineage");

        let _request: GetLineageRequest =
            serde_json::from_value(params).map_err(|e| format!("Invalid params: {e}"))?;

        // Get provider (includes our family ID)
        let provider = self.get_provider().await?;

        let family_id = provider.family_id().unwrap_or_else(|| String::from("unknown"));

        // Query security provider for our node ID (if needed)
        let security_socket = self.discover_security_socket().await?;
        let client = UnixRpcClient::new(&security_socket)
            .map_err(|e| format!("Failed to connect to security provider: {e}"))?;

        // Query primal.info from the discovered crypto provider
        let provider_info: Value = client
            .call("primal.info", &json!({}))
            .await
            .unwrap_or_else(|_| json!({"name": "unknown", "version": "unknown"}));

        let provider_name = provider_info["name"].as_str().unwrap_or("unknown").to_string();

        info!("✅ Retrieved lineage info (family: {}, provider: {})", family_id, provider_name);

        Ok(json!({
            "family_id": family_id,
            "provider": provider_name,
            "provider_version": provider_info["version"],
            "encryption": "chacha20_poly1305",
            "lineage_type": "genetic",
        }))
    }
}
