// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Automatic mesh peer seeding from environment on startup.
//!
//! Reads `SONGBIRD_PEERS` (comma-separated `node_id@address:port` entries) and
//! auto-initializes the beacon mesh so `discovery.peers` is populated without
//! requiring an explicit `mesh.init` RPC call from an external consumer.
//!
//! Format: `SONGBIRD_PEERS=iron-gate@192.168.1.238:7700,south-gate@192.168.4.29:7700`

use songbird_universal_ipc::handlers::mesh_handler::MeshHandler;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Parse `SONGBIRD_PEERS` into `(node_id, address)` pairs.
///
/// Format: comma-separated `node_id@host:port` entries.
/// Invalid entries are logged and skipped.
fn parse_peers_env() -> Vec<(String, String)> {
    let raw = match songbird_process_env::var("SONGBIRD_PEERS") {
        Ok(v) if !v.is_empty() => v,
        _ => return Vec::new(),
    };
    parse_peers_str(&raw)
}

/// Parse a peer specification string into `(node_id, address)` pairs.
pub(crate) fn parse_peers_str(raw: &str) -> Vec<(String, String)> {
    raw.split(',')
        .filter_map(|entry| {
            let entry = entry.trim();
            if entry.is_empty() {
                return None;
            }
            let (node_id, address) = entry.split_once('@')?;
            let node_id = node_id.trim();
            let address = address.trim();
            if node_id.is_empty() || address.is_empty() {
                warn!(
                    entry,
                    "SONGBIRD_PEERS: skipping malformed entry (missing node_id or address)"
                );
                return None;
            }
            if address.parse::<std::net::SocketAddr>().is_err() {
                warn!(entry, "SONGBIRD_PEERS: skipping entry with invalid address");
                return None;
            }
            Some((node_id.to_string(), address.to_string()))
        })
        .collect()
}

/// Resolve our own node ID from environment.
fn resolve_node_id() -> String {
    songbird_process_env::var("SONGBIRD_NODE_ID")
        .or_else(|_| songbird_process_env::var("NODE_ID"))
        .or_else(|_| songbird_process_env::var("HOSTNAME"))
        .unwrap_or_else(|_| gethostname::gethostname().to_string_lossy().to_string())
}

/// Spawn automatic mesh initialization from `SONGBIRD_PEERS` env var.
///
/// Called after socket bind. If `SONGBIRD_PEERS` is set, initializes the mesh
/// with the specified peers so `discovery.peers` is immediately populated.
pub fn spawn_mesh_seed(mesh_handler: Arc<MeshHandler>) {
    let peers = parse_peers_env();
    if peers.is_empty() {
        debug!("SONGBIRD_PEERS not set or empty — mesh requires explicit mesh.init");
        return;
    }

    let node_id = resolve_node_id();
    info!(
        node_id = %node_id,
        peer_count = peers.len(),
        "Auto-seeding mesh from SONGBIRD_PEERS"
    );

    tokio::spawn(async move {
        let bootstrap_peers: Vec<serde_json::Value> = peers
            .iter()
            .map(|(nid, addr)| {
                serde_json::json!({
                    "node_id": nid,
                    "address": addr
                })
            })
            .collect();

        let params = serde_json::json!({
            "node_id": node_id,
            "bootstrap_peers": bootstrap_peers
        });

        match mesh_handler.handle_init(params).await {
            Ok(result) => {
                let added = result
                    .get("bootstrap_peers_added")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                info!(
                    peers_added = added,
                    "Mesh auto-seeded from SONGBIRD_PEERS — discovery.peers is live"
                );
            }
            Err(e) => {
                warn!(error = %e, "Failed to auto-seed mesh from SONGBIRD_PEERS");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_string() {
        assert!(parse_peers_str("").is_empty());
    }

    #[test]
    fn parse_valid_entries() {
        let peers = parse_peers_str("iron-gate@192.168.1.238:7700,south-gate@192.168.4.29:7700");
        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0], ("iron-gate".to_string(), "192.168.1.238:7700".to_string()));
        assert_eq!(peers[1], ("south-gate".to_string(), "192.168.4.29:7700".to_string()));
    }

    #[test]
    fn parse_skips_invalid() {
        let peers =
            parse_peers_str("good@192.168.1.1:7700,bad-no-at-sign,missing@not-a-port,,@empty:0");
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].0, "good");
    }

    #[test]
    fn parse_handles_whitespace() {
        let peers = parse_peers_str(" east@10.0.0.1:7700 , west@10.0.0.2:7700 ");
        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0].0, "east");
        assert_eq!(peers[1].0, "west");
    }

    #[tokio::test]
    async fn spawn_mesh_seed_populates_mesh() {
        songbird_process_env::set_var("SONGBIRD_NODE_ID", "test-gate-seed");
        songbird_process_env::set_var(
            "SONGBIRD_PEERS",
            "iron-gate@192.168.1.238:7700,south-gate@192.168.4.29:7700",
        );

        let mesh_handler = Arc::new(MeshHandler::new());
        spawn_mesh_seed(Arc::clone(&mesh_handler));

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let guard = mesh_handler.mesh().await;
        let mesh = guard.as_ref().expect("mesh should be initialized");
        let reachable = mesh.get_reachable_nodes().await;

        songbird_process_env::remove_var("SONGBIRD_PEERS");
        songbird_process_env::remove_var("SONGBIRD_NODE_ID");

        assert_eq!(reachable.len(), 2);
        assert!(reachable.contains(&"iron-gate".to_string()));
        assert!(reachable.contains(&"south-gate".to_string()));
    }
}
