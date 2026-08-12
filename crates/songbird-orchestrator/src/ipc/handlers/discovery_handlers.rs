// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Discovery JSON-RPC handlers merging mesh peers and service registry.

use super::IpcHandlers;

impl IpcHandlers {
    /// Handle `discovery.peers` — merges service registry peers with mesh peers.
    pub async fn discovery_peers_json(
        &self,
        _params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        let mut peers: Vec<serde_json::Value> = Vec::new();
        let mut seen_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Source 1: Mesh peers (bootstrap + discovered) via mesh.peers handler
        if let Ok(mesh_result) = self.mesh_handler.handle_peers(serde_json::json!({})).await
            && let Some(mesh_peers) = mesh_result.get("peers").and_then(|p| p.as_array())
        {
            for mp in mesh_peers {
                let node_id = mp.get("node_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if !node_id.is_empty() {
                    seen_ids.insert(node_id.clone());
                    peers.push(serde_json::json!({
                        "node_id": node_id,
                        "address": mp.get("address").unwrap_or(&serde_json::Value::Null),
                        "tcp_port": mp.get("address")
                            .and_then(|a| a.as_str())
                            .and_then(|a| a.parse::<std::net::SocketAddr>().ok())
                            .map(|a| a.port()),
                        "latency_ms": mp.get("latency_ms").unwrap_or(&serde_json::Value::Null),
                        "source": "mesh",
                        "reachable": mp.get("reachable").and_then(serde_json::Value::as_bool).unwrap_or(true),
                        "protocols": ["tcp"]
                    }));
                }
            }
        }

        // Source 2: Service registry (registered primals)
        if let Ok(registry_result) =
            self.discover_by_capability_json(Some(serde_json::json!({ "capability": "*" }))).await
            && let Some(services) = registry_result.get("services").and_then(|s| s.as_array())
        {
            for svc in services {
                let id = svc
                    .get("primal_id")
                    .or_else(|| svc.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if !id.is_empty() && !seen_ids.contains(&id) {
                    seen_ids.insert(id.clone());
                    peers.push(serde_json::json!({
                        "node_id": id,
                        "address": svc.get("endpoint").unwrap_or(&serde_json::Value::Null),
                        "source": "registry",
                        "reachable": true,
                        "protocols": ["uds"]
                    }));
                }
            }
        }

        Ok(serde_json::json!({
            "peers": peers,
            "total_count": peers.len()
        }))
    }

    /// Handle `discovery.topology` — mesh gate topology.
    pub async fn discovery_topology_json(
        &self,
        _params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        let mesh_result = self
            .mesh_handler
            .handle_peers(serde_json::json!({}))
            .await
            .unwrap_or_else(|_| serde_json::json!({"peers": []}));

        let peers =
            mesh_result.get("peers").and_then(|p| p.as_array()).cloned().unwrap_or_default();

        let node_id = songbird_process_env::var("SONGBIRD_NODE_ID").unwrap_or_default();

        Ok(serde_json::json!({
            "gates": peers,
            "gate_count": peers.len(),
            "self_node_id": node_id,
        }))
    }

    /// Handle `discovery.health` — node health status for composition consumers.
    pub async fn discovery_health_json(
        &self,
        _params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        let mesh_status = self.mesh_handler.handle_status(serde_json::json!({})).await;
        let mesh_active = mesh_status
            .as_ref()
            .ok()
            .and_then(|v| v.get("initialized"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        let registry_count = self.service_registry.list_all_services().await.len();

        Ok(serde_json::json!({
            "alive": true,
            "mesh_active": mesh_active,
            "registered_services": registry_count,
        }))
    }

    /// Handle `discovery.query` — generic capability/service discovery.
    pub async fn discovery_query_json(
        &self,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        let cap = params
            .as_ref()
            .and_then(|p| p.get("capability"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("*");

        self.discover_by_capability_json(Some(serde_json::json!({ "capability": cap }))).await
    }

    /// Handle `discovery.bonds` — external API bonds from drawbridge allowlist.
    #[expect(clippy::unused_async, reason = "called from async dispatch; may need await in future")]
    pub async fn discovery_bonds_json(
        &self,
        _params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, crate::ipc::pure_rust_server::JsonRpcError> {
        let allowlist_raw =
            songbird_process_env::var("SONGBIRD_DRAWBRIDGE_EXTERNAL_ALLOWLIST").unwrap_or_default();

        let bonds: Vec<serde_json::Value> = allowlist_raw
            .split(',')
            .filter_map(|entry| {
                let entry = entry.trim();
                let (name, url) = entry.split_once('=')?;
                Some(serde_json::json!({
                    "service": name.trim(),
                    "base_url": url.trim(),
                }))
            })
            .collect();

        Ok(serde_json::json!({
            "bonds": bonds,
            "bond_count": bonds.len(),
        }))
    }
}
