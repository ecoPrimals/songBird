// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

use super::{FederationPeersResponse, FederationStatusResponse, IpcServiceHandler};
use serde_json::Value;

impl IpcServiceHandler {
    /// Handle `birdsong.advertise` method
    ///
    /// Generates an encrypted beacon with the onion endpoint (if running).
    /// This is the complete Dark Forest beacon - only family can see the .onion address.
    ///
    /// NEW (Feb 6, 2026) - Combines onion service and birdsong beacon
    pub(super) async fn handle_birdsong_advertise(&self, params: Value) -> Result<Value, String> {
        // Get node_id and capabilities from params
        let node_id = params
            .get("node_id")
            .and_then(|v| v.as_str())
            .ok_or("Missing node_id parameter")?
            .to_string();

        let capabilities: Vec<String> = params
            .get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        // Get onion address if service is running
        let onion_status = self.onion_handler.handle_status(serde_json::json!({})).await?;
        let onion_endpoint = if onion_status.get("running") == Some(&serde_json::json!(true)) {
            let addr = onion_status.get("onion_address").and_then(|v| v.as_str());
            let port = onion_status.get("port").and_then(serde_json::Value::as_u64).unwrap_or(3492);
            addr.map(|a| format!("{a}:{port}"))
        } else {
            None
        };

        // Also include any direct endpoint hints
        let endpoint_hints = params.get("endpoint_hints").cloned();

        // Generate the encrypted beacon with all endpoints
        let beacon_params = serde_json::json!({
            "node_id": node_id,
            "capabilities": capabilities,
            "onion_endpoint": onion_endpoint,
            "endpoint_hints": endpoint_hints,
        });

        let beacon_result =
            self.birdsong_handler.handle_generate_encrypted_beacon(beacon_params).await?;

        // Return combined result
        Ok(serde_json::json!({
            "beacon": beacon_result,
            "onion_endpoint": onion_endpoint,
            "onion_running": onion_status.get("running"),
        }))
    }

    /// Handle `health` method — requires uptime + registry info
    pub(super) async fn handle_health(&self) -> Result<Value, String> {
        let uptime_secs = self.start_time.read().await.elapsed().as_secs();
        let service_count = self.registry.read().await.list_services().await.len();
        Ok(crate::introspection::health(uptime_secs, service_count))
    }

    /// Handle `identity` method — canonical family-id lookup
    pub(super) async fn handle_identity(&self) -> Result<Value, String> {
        let family_id = self.family_id_env.as_ref().map_or_else(
            || crate::introspection::canonical_family_id(|k| songbird_process_env::var(k)),
            |f| crate::introspection::canonical_family_id(|k| (f)(k)),
        );
        Ok(crate::introspection::identity(&family_id))
    }

    /// `songbird.federation.peers` / `federation.peers`
    pub(super) async fn handle_federation_peers_rpc(&self) -> Result<Value, String> {
        let Some(ref state) = self.federation_state else {
            return serde_json::to_value(FederationPeersResponse {
                peers: vec![],
                total_count: 0,
                federation_enabled: false,
            })
            .map_err(|e| format!("Serialization error: {e}"));
        };

        let mut peers: Vec<String> =
            state.active_nodes().await.into_iter().map(|n| n.node_id).collect();
        peers.sort();
        let federation_stats = state.get_stats().await;
        let total_count = peers.len();
        let federation_enabled = federation_stats.total_nodes > 0;

        serde_json::to_value(FederationPeersResponse {
            peers,
            total_count,
            federation_enabled,
        })
        .map_err(|e| format!("Serialization error: {e}"))
    }

    /// `songbird.federation.status` / `federation.status`
    pub(super) async fn handle_federation_status_rpc(&self) -> Result<Value, String> {
        let Some(ref state) = self.federation_state else {
            return serde_json::to_value(FederationStatusResponse {
                enabled: false,
                active_connections: 0,
            })
            .map_err(|e| format!("Serialization error: {e}"));
        };

        let fed_stats = state.get_stats().await;
        serde_json::to_value(FederationStatusResponse {
            enabled: fed_stats.total_nodes > 0,
            active_connections: fed_stats.active_nodes,
        })
        .map_err(|e| format!("Serialization error: {e}"))
    }
}
