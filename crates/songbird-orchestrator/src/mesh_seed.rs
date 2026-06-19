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
///
/// Supports two formats:
/// - `node_id@host:port` — explicit identity
/// - `host:port` — auto-generates node_id as `peer-{ip}` (backward-compat with Wave 49 docs)
pub(crate) fn parse_peers_str(raw: &str) -> Vec<(String, String)> {
    raw.split(',')
        .filter_map(|entry| {
            let entry = entry.trim();
            if entry.is_empty() {
                return None;
            }
            let (node_id, address) = if let Some((nid, addr)) = entry.split_once('@') {
                let nid = nid.trim();
                let addr = addr.trim();
                if nid.is_empty() {
                    warn!(entry, "SONGBIRD_PEERS: skipping entry with empty node_id");
                    return None;
                }
                (nid.to_string(), addr.to_string())
            } else {
                let addr = entry.to_string();
                let Ok(sa) = addr.parse::<std::net::SocketAddr>() else {
                    warn!(entry, "SONGBIRD_PEERS: skipping entry with invalid address");
                    return None;
                };
                let node_id = format!("peer-{}", sa.ip());
                (node_id, addr)
            };
            if address.is_empty() {
                return None;
            }
            if address.parse::<std::net::SocketAddr>().is_err() {
                warn!(entry, "SONGBIRD_PEERS: skipping entry with invalid address");
                return None;
            }
            Some((node_id, address))
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

/// Spawn automatic mesh initialization from `SONGBIRD_PEERS` env var or persisted state.
///
/// Called after socket bind. Priority:
/// 1. `SONGBIRD_PEERS` env var (explicit operator intent)
/// 2. Persisted peers from `~/.local/share/songbird/peers.toml` (autonomous recovery)
///
/// If neither is available, mesh requires explicit `mesh.init`.
pub fn spawn_mesh_seed(mesh_handler: Arc<MeshHandler>) {
    let peers = parse_peers_env();
    let (peers, source) = if peers.is_empty() {
        if let Some((_, persisted)) =
            songbird_universal_ipc::handlers::mesh_handler::persistence::load_persisted_peers()
        {
            let converted: Vec<(String, String)> =
                persisted.iter().map(|(nid, addr)| (nid.clone(), addr.to_string())).collect();
            info!(
                peer_count = converted.len(),
                "Restoring mesh from persisted peers (autonomous recovery)"
            );
            (converted, "persisted")
        } else {
            debug!("No SONGBIRD_PEERS and no persisted peers — mesh requires explicit mesh.init");
            return;
        }
    } else {
        (peers, "SONGBIRD_PEERS")
    };

    let node_id = resolve_node_id();
    info!(
        node_id = %node_id,
        peer_count = peers.len(),
        source = source,
        "Auto-seeding mesh"
    );

    let peers_for_trust = peers.clone();
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

                // Phase 2: auto-exchange trust keys with peers (BD-TRUST-01)
                let peer_addrs: Vec<(String, std::net::SocketAddr)> = peers_for_trust
                    .iter()
                    .filter_map(|(nid, addr)| {
                        addr.parse::<std::net::SocketAddr>().ok().map(|sa| (nid.clone(), sa))
                    })
                    .collect();
                crate::mesh_trust_exchange::spawn_trust_exchange(peer_addrs);
            }
            Err(e) => {
                warn!(error = %e, "Failed to auto-seed mesh from SONGBIRD_PEERS");
            }
        }
    });
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "test assertions")]
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
        assert_eq!(peers[0], (String::from("iron-gate"), String::from("192.168.1.238:7700")));
        assert_eq!(peers[1], (String::from("south-gate"), String::from("192.168.4.29:7700")));
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

    #[test]
    fn parse_address_only_format() {
        let peers = parse_peers_str("192.168.1.144:7700,192.168.1.238:7700");
        assert_eq!(peers.len(), 2);
        assert_eq!(
            peers[0],
            (String::from("peer-192.168.1.144"), String::from("192.168.1.144:7700"))
        );
        assert_eq!(
            peers[1],
            (String::from("peer-192.168.1.238"), String::from("192.168.1.238:7700"))
        );
    }

    #[test]
    fn parse_mixed_formats() {
        let peers =
            parse_peers_str("iron-gate@192.168.1.238:7700,192.168.4.29:7700,south@10.0.0.1:7700");
        assert_eq!(peers.len(), 3);
        assert_eq!(peers[0].0, "iron-gate");
        assert_eq!(peers[1].0, "peer-192.168.4.29");
        assert_eq!(peers[2].0, "south");
    }

    #[tokio::test]
    async fn spawn_mesh_seed_populates_mesh() {
        let _guard = crate::test_sync_env::env_lock();
        songbird_process_env::set_var("SONGBIRD_NODE_ID", "test-gate-seed");
        songbird_process_env::set_var(
            "SONGBIRD_PEERS",
            "iron-gate@192.168.1.238:7700,south-gate@192.168.4.29:7700",
        );

        let mesh_handler = Arc::new(MeshHandler::new());
        spawn_mesh_seed(Arc::clone(&mesh_handler));

        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            let guard = mesh_handler.mesh().await;
            if let Some(mesh) = guard.as_ref() {
                let reachable = mesh.get_reachable_nodes().await;
                if reachable.len() >= 2 {
                    break;
                }
            }
            drop(guard);
            assert!(tokio::time::Instant::now() < deadline, "mesh not populated within 2s");
            tokio::task::yield_now().await;
        }

        let guard = mesh_handler.mesh().await;
        let mesh = guard.as_ref().expect("mesh should be initialized");
        let reachable = mesh.get_reachable_nodes().await;

        songbird_process_env::remove_var("SONGBIRD_PEERS");
        songbird_process_env::remove_var("SONGBIRD_NODE_ID");

        assert_eq!(reachable.len(), 2);
        assert!(reachable.contains(&String::from("iron-gate")));
        assert!(reachable.contains(&String::from("south-gate")));
    }
}
