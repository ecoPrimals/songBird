// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Automatic mesh peer seeding from environment on startup.
//!
//! Reads `SONGBIRD_PEERS` (comma-separated `node_id@address:port` entries) and
//! auto-initializes the beacon mesh so `discovery.peers` is populated without
//! requiring an explicit `mesh.init` RPC call from an external consumer.
//!
//! Optionally reads `SONGBIRD_OVERLAY_PEERS` to register overlay (WireGuard)
//! endpoints for the same peers — these get priority-0 routing alongside Local.
//!
//! Format: `SONGBIRD_PEERS=iron-gate@192.168.1.238:7700,south-gate@192.168.4.29:7700`
//! Overlay: `SONGBIRD_OVERLAY_PEERS=iron-gate@10.13.37.5:7700,south-gate@10.13.37.6:7700`

use songbird_universal_ipc::handlers::mesh_handler::MeshHandler;
use std::net::IpAddr;
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

/// Parse `SONGBIRD_OVERLAY_PEERS` into `(node_id, address)` pairs for overlay endpoints.
///
/// Same format as `SONGBIRD_PEERS`. These are registered as `EndpointType::Overlay`
/// (priority 0, same as Local) so WireGuard paths are preferred over WAN Direct.
fn parse_overlay_peers_env() -> Vec<(String, String)> {
    let raw = match songbird_process_env::var("SONGBIRD_OVERLAY_PEERS") {
        Ok(v) if !v.is_empty() => v,
        _ => return Vec::new(),
    };
    parse_peers_str(&raw)
}

/// Detect WireGuard overlay interfaces on this host.
///
/// Returns the first detected WG interface IP, if any. Scans `/sys/class/net`
/// for `wg*` interfaces and reads their addresses from `/proc/net/if_inet6` or
/// the ip address assignment.
///
/// If `SONGBIRD_OVERLAY_SUBNET` is set (e.g. "10.13.37"), uses that as the
/// canonical overlay subnet prefix for matching.
pub(crate) fn detect_overlay_address() -> Option<IpAddr> {
    if let Ok(addr) = songbird_process_env::var("SONGBIRD_OVERLAY_IP") {
        return addr.parse::<IpAddr>().ok();
    }

    let prefix = songbird_process_env::var("SONGBIRD_OVERLAY_SUBNET")
        .unwrap_or_else(|_| String::from("10.13.37"));

    let Ok(entries) = std::fs::read_dir("/sys/class/net") else {
        return None;
    };

    for entry in entries.flatten() {
        let name = entry.file_name();
        let iface = name.to_string_lossy();
        if !iface.starts_with("wg") {
            continue;
        }

        // Read assigned IPv4 addresses from /proc/net/fib_trie or use ip command output
        if let Ok(content) = std::fs::read_to_string(format!("/sys/class/net/{iface}/address")) {
            debug!(interface = %iface, mac = %content.trim(), "Found WG interface");
        }

        // Try to find the IP from /proc/net/fib_trie matching our subnet
        if let Ok(fib) = std::fs::read_to_string("/proc/net/fib_trie") {
            for line in fib.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("|--") || trimmed.starts_with("+--") {
                    let ip_str = trimmed.trim_start_matches("|--").trim_start_matches("+--").trim();
                    if ip_str.starts_with(&prefix)
                        && let Ok(ip) = ip_str.parse::<IpAddr>()
                    {
                        info!(overlay_ip = %ip, interface = %iface, "Detected WG overlay address");
                        return Some(ip);
                    }
                }
            }
        }
    }

    None
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

/// Register overlay (WireGuard) endpoints for peers already in the mesh.
///
/// After `mesh.init` seeds Direct endpoints, this adds Overlay endpoints for the
/// same node IDs. Since Overlay has priority 0 (same as Local), the mesh will
/// prefer these paths when reachable — giving WG traffic precedence over WAN.
async fn register_overlay_endpoints(
    mesh_handler: &MeshHandler,
    overlay_peers: &[(String, String)],
    overlay_name: &str,
) {
    use songbird_onion_relay::mesh::{EndpointType, RelayEndpoint};
    use std::time::Instant;

    let guard = mesh_handler.mesh().await;
    let Some(mesh) = guard.as_ref() else {
        warn!("Cannot register overlay endpoints — mesh not initialized");
        return;
    };

    let mut registered = 0;
    for (node_id, addr_str) in overlay_peers {
        let Ok(addr) = addr_str.parse::<std::net::SocketAddr>() else {
            warn!(node_id, addr = addr_str, "SONGBIRD_OVERLAY_PEERS: skipping invalid address");
            continue;
        };
        let endpoint = RelayEndpoint {
            node_id: node_id.clone(),
            endpoint_type: EndpointType::Overlay {
                addr,
                overlay_name: String::from(overlay_name),
            },
            latency: None,
            last_seen: Instant::now(),
            reachable: true,
        };
        mesh.add_endpoint(node_id.clone(), endpoint).await;
        registered += 1;
    }

    info!(
        overlay_name,
        registered, "Registered overlay endpoints (priority 0 — preferred over Direct)"
    );
}

/// Spawn automatic mesh initialization from `SONGBIRD_PEERS` env var or persisted state.
///
/// Called after socket bind. Priority:
/// 1. `SONGBIRD_PEERS` env var (explicit operator intent)
/// 2. Persisted peers from `~/.local/share/songbird/peers.toml` (autonomous recovery)
///
/// If neither is available, mesh requires explicit `mesh.init`.
///
/// If `SONGBIRD_OVERLAY_PEERS` is set, overlay endpoints are registered post-init
/// for the same node IDs, giving them priority-0 routing (WireGuard preference).
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

    let overlay_peers = parse_overlay_peers_env();
    let overlay_name = songbird_process_env::var("SONGBIRD_OVERLAY_NAME")
        .unwrap_or_else(|_| String::from("wireguard"));

    if let Some(local_overlay_ip) = detect_overlay_address() {
        info!(
            overlay_ip = %local_overlay_ip,
            overlay = %overlay_name,
            "Detected local overlay interface"
        );
    }

    let node_id = resolve_node_id();
    info!(
        node_id = %node_id,
        peer_count = peers.len(),
        overlay_count = overlay_peers.len(),
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

                // Register overlay endpoints for WG-preferred routing
                if !overlay_peers.is_empty() {
                    register_overlay_endpoints(&mesh_handler, &overlay_peers, &overlay_name).await;
                }

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

    #[tokio::test]
    async fn spawn_mesh_seed_registers_overlay_peers() {
        let _guard = crate::test_sync_env::env_lock();
        songbird_process_env::set_var("SONGBIRD_NODE_ID", "east-gate-overlay-test");
        songbird_process_env::set_var(
            "SONGBIRD_PEERS",
            "flock-gate@203.0.113.50:7700,golgi@203.0.113.51:7700",
        );
        songbird_process_env::set_var(
            "SONGBIRD_OVERLAY_PEERS",
            "flock-gate@10.13.37.6:7700,golgi@10.13.37.1:7700",
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

        // The overlay endpoint should be the best path (priority 0)
        let best = mesh.get_best_path("flock-gate").await;
        assert!(best.is_some(), "flock-gate should have a path");
        let best = best.unwrap();
        assert!(
            matches!(best.endpoint_type, songbird_onion_relay::mesh::EndpointType::Overlay { .. }),
            "Expected Overlay as best path, got {:?}",
            best.endpoint_type
        );

        songbird_process_env::remove_var("SONGBIRD_PEERS");
        songbird_process_env::remove_var("SONGBIRD_OVERLAY_PEERS");
        songbird_process_env::remove_var("SONGBIRD_NODE_ID");
    }

    #[test]
    fn overlay_peers_parsed_same_format_as_regular() {
        let overlay = parse_peers_str("flock@10.13.37.6:7700,golgi@10.13.37.1:7700");
        assert_eq!(overlay.len(), 2);
        assert_eq!(overlay[0], (String::from("flock"), String::from("10.13.37.6:7700")));
        assert_eq!(overlay[1], (String::from("golgi"), String::from("10.13.37.1:7700")));
    }
}
