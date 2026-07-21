// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Mesh enrollment — BTSP-validated gate joining.
//!
//! Handles `mesh.enroll` requests by verifying HMAC enrollment proofs via
//! bearDog's `enrollment.verify` endpoint, then persisting and meshing the node.

use serde_json::{Value, json};

use super::MeshHandler;

impl MeshHandler {
    /// Handle `mesh.enroll` — BTSP-validated gate enrollment into the mesh.
    ///
    /// Accepts enrollment requests from new gates seeking to join. The gate
    /// proves its identity via a BTSP session proof (Ed25519 signature from
    /// its security provider). On success, returns the `WireGuard` peer config
    /// and mesh topology needed for the gate to connect.
    ///
    /// Requires: hub-side peer addition automation (cellMembrane `gate.enroll`
    /// client calls this endpoint after `wg.keygen`).
    pub async fn handle_enroll(&self, params: Value) -> Result<Value, String> {
        let node_id = params
            .get("node_id")
            .and_then(Value::as_str)
            .ok_or("mesh.enroll requires 'node_id' (gate name)")?;
        let public_key = params
            .get("public_key")
            .and_then(Value::as_str)
            .ok_or("mesh.enroll requires 'public_key' (WireGuard public key)")?;
        let timestamp = params.get("timestamp").and_then(Value::as_u64).unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs())
        });
        let proof = params
            .get("proof")
            .and_then(Value::as_str)
            .ok_or("mesh.enroll requires 'proof' (HMAC enrollment proof from family seed)")?;
        let address = params.get("address").and_then(Value::as_str).unwrap_or("");

        tracing::info!(node_id = %node_id, "mesh.enroll: verifying enrollment proof");

        let security_socket =
            songbird_crypto_provider::socket_discovery::discover_security_socket();
        let security_client = songbird_http_client::SecurityRpcClient::new_direct(security_socket);

        match security_client.verify_enrollment_proof(node_id, public_key, timestamp, proof).await {
            Ok(v) if v.verified => self.complete_enrollment(node_id, public_key, address).await,
            Ok(v) => {
                let reason = v.reason.unwrap_or_else(|| String::from("proof_invalid"));
                tracing::warn!(node_id = %node_id, reason = %reason, "mesh.enroll: rejected");
                Ok(json!({ "enrolled": false, "reason": reason, "node_id": node_id }))
            }
            Err(e) => {
                tracing::warn!(node_id = %node_id, error = %e, "mesh.enroll: provider unavailable");
                Ok(json!({
                    "enrolled": false,
                    "reason": "security_provider_unavailable",
                    "message": format!("Cannot verify proof: {e}"),
                    "node_id": node_id
                }))
            }
        }
    }

    async fn complete_enrollment(
        &self,
        node_id: &str,
        public_key: &str,
        address: &str,
    ) -> Result<Value, String> {
        tracing::info!(node_id = %node_id, "mesh.enroll: proof verified, enrolling node");

        let mesh_guard = self.mesh.read().await;
        let mesh_active = if let Some(mesh) = mesh_guard.as_ref() {
            if let Ok(addr) = address.parse::<std::net::SocketAddr>() {
                let endpoint = songbird_onion_relay::mesh::RelayEndpoint {
                    node_id: node_id.to_string(),
                    endpoint_type: songbird_onion_relay::mesh::EndpointType::Direct {
                        addr,
                    },
                    latency: None,
                    last_seen: std::time::Instant::now(),
                    reachable: true,
                };
                mesh.add_endpoint(node_id.to_string(), endpoint).await;
            }
            true
        } else {
            false
        };
        drop(mesh_guard);

        super::persistence::save_enrolled_peer(node_id, public_key, address);

        Ok(json!({
            "enrolled": true,
            "node_id": node_id,
            "mesh_active": mesh_active,
            "message": if mesh_active {
                "Node enrolled and added to mesh"
            } else {
                "Node enrolled (mesh not yet initialized — will join on next seed)"
            }
        }))
    }
}
