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
/// (priority 1) so WireGuard paths are preferred over WAN Direct but not over LAN.
fn parse_overlay_peers_env() -> Vec<(String, String)> {
    let raw = match songbird_process_env::var("SONGBIRD_OVERLAY_PEERS") {
        Ok(v) if !v.is_empty() => v,
        _ => return Vec::new(),
    };
    parse_peers_str(&raw)
}

/// Parse `SONGBIRD_LOCAL_PEERS` into `(node_id, address)` pairs for LAN endpoints.
///
/// Same format as `SONGBIRD_PEERS`. These are registered as `EndpointType::Local`
/// (priority 0) — always preferred over overlay/direct. Use for same-subnet peers
/// where sub-millisecond latency is available (e.g., same MikroTik switch).
///
/// Format: `SONGBIRD_LOCAL_PEERS=eastGate@192.168.4.244:7700,sporeGate@192.168.4.2:7700`
fn parse_local_peers_env() -> Vec<(String, String)> {
    let raw = match songbird_process_env::var("SONGBIRD_LOCAL_PEERS") {
        Ok(v) if !v.is_empty() => v,
        _ => return Vec::new(),
    };
    parse_peers_str(&raw)
}

/// Detect `WireGuard` overlay interfaces on this host.
///
/// Returns the first detected WG interface IP, if any. Scans `/sys/class/net`
/// for `wg*` interfaces and reads their addresses from `/proc/net/if_inet6` or
/// the ip address assignment.
///
/// If `SONGBIRD_OVERLAY_SUBNET` is set (e.g. "10.13.37"), uses that as the
/// canonical overlay subnet prefix for matching. The default "10.13.37"
/// matches the `WireGuard` overlay defined in `ecosystem_manifest.toml`
/// `[gates.*] wg_ip` fields.
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

/// Discover mesh peers from `WireGuard` configuration.
///
/// Multi-tier approach (first success wins):
/// 1. `wg show all dump` (requires root or `CAP_NET_ADMIN`)
/// 2. WG config file (`SONGBIRD_WG_CONF` env or `/etc/wireguard/wg0.conf`)
/// 3. User-accessible mesh peers file (`~/.config/songbird/mesh-peers.toml`)
///
/// Each peer's overlay IP is used with the standard songbird port to form a mesh
/// peer entry. Returns `None` if no tier produces peers.
fn discover_wireguard_peers() -> Option<Vec<(String, String)>> {
    let prefix = songbird_process_env::var("SONGBIRD_OVERLAY_SUBNET")
        .unwrap_or_else(|_| String::from("10.13.37"));

    // Tier 1: `wg show all dump` (root/CAP_NET_ADMIN)
    if let Some(peers) = discover_wg_from_command(&prefix) {
        return Some(peers);
    }

    // Tier 2: WG config file
    if let Some(peers) = discover_wg_from_config(&prefix) {
        return Some(peers);
    }

    // Tier 3: User-accessible mesh-peers.toml
    discover_wg_from_mesh_peers_file()
}

/// Tier 1: Run `wg show all dump` and parse output.
fn discover_wg_from_command(subnet_prefix: &str) -> Option<Vec<(String, String)>> {
    let output = std::process::Command::new("wg").args(["show", "all", "dump"]).output().ok()?;

    if !output.status.success() {
        debug!("wg show failed (not root or no WG interfaces) — trying config file fallback");
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let result = parse_wg_dump(&stdout, subnet_prefix);
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

/// Tier 2: Parse a `WireGuard` config file (INI-style).
///
/// Looks for `SONGBIRD_WG_CONF` env, then tries standard paths.
fn discover_wg_from_config(subnet_prefix: &str) -> Option<Vec<(String, String)>> {
    let paths_to_try: Vec<std::path::PathBuf> =
        if let Ok(custom) = songbird_process_env::var("SONGBIRD_WG_CONF") {
            vec![std::path::PathBuf::from(custom)]
        } else {
            vec![
                std::path::PathBuf::from("/etc/wireguard/wg0.conf"),
                songbird_types::defaults::paths::config_dir().join("wg0.conf"),
            ]
        };

    for path in &paths_to_try {
        if let Ok(content) = std::fs::read_to_string(path) {
            let result = parse_wg_conf(&content, subnet_prefix);
            if !result.is_empty() {
                info!(path = %path.display(), peers = result.len(), "Discovered peers from WG config file");
                return Some(result);
            }
        }
    }

    debug!("No readable WG config file found");
    None
}

/// Tier 3: Read a user-accessible `mesh-peers.toml` file.
///
/// Format:
/// ```toml
/// [[peers]]
/// node_id = "east-gate"
/// address = "10.13.37.5:7700"
///
/// [[peers]]
/// node_id = "golgi"
/// address = "10.13.37.1:7700"
/// ```
///
/// Located at `$XDG_CONFIG_HOME/songbird/mesh-peers.toml` or
/// `~/.config/songbird/mesh-peers.toml`.
fn discover_wg_from_mesh_peers_file() -> Option<Vec<(String, String)>> {
    let path = songbird_types::defaults::paths::config_dir().join("mesh-peers.toml");
    let content = std::fs::read_to_string(&path).ok()?;

    let parsed: MeshPeersFile = toml::from_str(&content).ok()?;
    let peers: Vec<(String, String)> = parsed
        .peers
        .into_iter()
        .filter(|p| !p.node_id.is_empty() && !p.address.is_empty())
        .map(|p| (p.node_id, p.address))
        .collect();

    if peers.is_empty() {
        return None;
    }

    info!(path = %path.display(), peers = peers.len(), "Discovered peers from mesh-peers.toml");
    Some(peers)
}

/// Deserialization target for `mesh-peers.toml`.
#[derive(serde::Deserialize)]
struct MeshPeersFile {
    #[serde(default)]
    peers: Vec<MeshPeerEntry>,
}

/// A single peer entry in `mesh-peers.toml`.
#[derive(serde::Deserialize)]
struct MeshPeerEntry {
    node_id: String,
    address: String,
}

/// Parse a `WireGuard` INI-style config file to extract peer allowed-IPs.
///
/// Extracts `[Peer]` sections and their `AllowedIPs` lines, filtering to
/// IPs matching the overlay subnet prefix.
fn parse_wg_conf(content: &str, subnet_prefix: &str) -> Vec<(String, String)> {
    use songbird_types::defaults::ports::DEFAULT_MESH_PEER_PORT;

    let mut peers: Vec<(String, String)> = Vec::new();
    let mut in_peer_section = false;
    let mut current_pubkey: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case("[peer]") {
            in_peer_section = true;
            current_pubkey = None;
            continue;
        }
        if trimmed.starts_with('[') {
            in_peer_section = false;
            current_pubkey = None;
            continue;
        }
        if !in_peer_section {
            continue;
        }

        if let Some(val) = trimmed.strip_prefix("PublicKey") {
            let val = val.trim_start_matches([' ', '=']).trim();
            current_pubkey = Some(val.to_string());
        } else if let Some(val) = trimmed.strip_prefix("AllowedIPs") {
            let val = val.trim_start_matches([' ', '=']).trim();
            for cidr in val.split(',') {
                let ip_str = cidr.trim().split('/').next().unwrap_or("");
                if ip_str.starts_with(subnet_prefix) {
                    let node_id = if let Some(ref pk) = current_pubkey {
                        format!("wg-{}", &pk[..8.min(pk.len())])
                    } else {
                        format!("wg-peer-{ip_str}")
                    };
                    let addr = format!("{ip_str}:{DEFAULT_MESH_PEER_PORT}");
                    peers.push((node_id, addr));
                    break;
                }
            }
        }
    }

    peers
}

/// Parse `wg show all dump` output to extract overlay peers.
///
/// Separated from `discover_wireguard_peers` for testability.
fn parse_wg_dump(dump: &str, subnet_prefix: &str) -> Vec<(String, String)> {
    use songbird_types::defaults::ports::DEFAULT_MESH_PEER_PORT;

    let mut peers: Vec<(String, String)> = Vec::new();

    for line in dump.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        // wg dump format — interface line: iface privkey pubkey listen-port fwmark
        //                   peer line:     iface pubkey preshared endpoint allowed-ips ...
        if fields.len() < 5 {
            continue;
        }
        let allowed_ips = fields.get(4).unwrap_or(&"");
        if allowed_ips.is_empty() || *allowed_ips == "(none)" {
            continue;
        }

        let pubkey = fields[1];
        // Interface lines have a private key in field 2; peer lines have "(none)" or preshared
        if fields.get(2).is_some_and(|f| !f.is_empty() && *f != "(none)") {
            continue;
        }

        for cidr in allowed_ips.split(',') {
            let ip_str = cidr.trim().split('/').next().unwrap_or("");
            if ip_str.starts_with(subnet_prefix) {
                let addr = format!("{ip_str}:{DEFAULT_MESH_PEER_PORT}");
                let node_id = format!("wg-{}", &pubkey[..8.min(pubkey.len())]);
                peers.push((node_id, addr));
                debug!(peer_ip = %ip_str, node_id_prefix = &pubkey[..8.min(pubkey.len())], "Discovered WG peer");
                break;
            }
        }
    }

    peers
}

/// Parse a peer specification string into `(node_id, address)` pairs.
///
/// Supports two formats:
/// - `node_id@host:port` — explicit identity
/// - `host:port` — auto-generates `node_id` as `peer-{ip}` (backward-compat with Wave 49 docs)
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

/// Register overlay (`WireGuard`) endpoints for peers already in the mesh.
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

/// Register LAN endpoints for peers with known LAN addresses.
///
/// LAN endpoints have priority 0 (`EndpointType::Local`), making them the
/// preferred path when reachable — sub-millisecond latency on same-subnet peers.
async fn register_lan_endpoints(
    mesh_handler: &MeshHandler,
    lan_peers: &[(String, std::net::SocketAddr)],
) {
    use songbird_onion_relay::mesh::{EndpointType, RelayEndpoint};
    use std::time::Instant;

    let guard = mesh_handler.mesh().await;
    let Some(mesh) = guard.as_ref() else {
        warn!("Cannot register LAN endpoints — mesh not initialized");
        return;
    };

    let mut registered = 0;
    for (node_id, addr) in lan_peers {
        let endpoint = RelayEndpoint {
            node_id: node_id.clone(),
            endpoint_type: EndpointType::Local {
                addr: *addr,
            },
            latency: None,
            last_seen: Instant::now(),
            reachable: true,
        };
        mesh.add_endpoint(node_id.clone(), endpoint).await;
        registered += 1;
    }

    info!(registered, "Registered LAN endpoints (priority 0 — preferred over all)");
}

/// Spawn automatic mesh initialization from `SONGBIRD_PEERS` env var or persisted state.
///
/// Called after socket bind. Priority:
/// 1. `SONGBIRD_PEERS` env var (explicit operator intent)
/// 2. Persisted peers from `~/.local/share/songbird/peers.toml` (autonomous recovery)
/// 3. `WireGuard` peer auto-detection from `wg show` (zero-config mesh on WG hosts)
///
/// If none succeed, mesh requires explicit `mesh.init`.
///
/// If `SONGBIRD_OVERLAY_PEERS` is set, overlay endpoints are registered post-init
/// for the same node IDs, giving them priority-0 routing (`WireGuard` preference).
pub fn spawn_mesh_seed(mesh_handler: Arc<MeshHandler>) {
    let peers = parse_peers_env();
    let mut lan_peers: Vec<(String, std::net::SocketAddr)> = Vec::new();
    let (peers, source) = if peers.is_empty() {
        if let Some((_, persisted)) =
            songbird_universal_ipc::handlers::mesh_handler::persistence::load_persisted_peers_full()
        {
            for p in &persisted {
                if let Some(lan) = p.lan_addr {
                    lan_peers.push((p.node_id.clone(), lan));
                }
            }
            let converted: Vec<(String, String)> =
                persisted.iter().map(|p| (p.node_id.clone(), p.address.to_string())).collect();
            info!(
                peer_count = converted.len(),
                lan_count = lan_peers.len(),
                "Restoring mesh from persisted peers (autonomous recovery)"
            );
            (converted, "persisted")
        } else if let Some(wg_peers) = discover_wireguard_peers() {
            info!(peer_count = wg_peers.len(), "Auto-discovered peers from WireGuard interface");
            (wg_peers, "wireguard")
        } else {
            debug!(
                "No SONGBIRD_PEERS, no persisted peers, no WG peers — mesh requires explicit mesh.init"
            );
            return;
        }
    } else {
        (peers, "SONGBIRD_PEERS")
    };

    let overlay_peers = parse_overlay_peers_env();
    let local_peers_env = parse_local_peers_env();
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
        local_count = lan_peers.len() + local_peers_env.len(),
        source = source,
        "Auto-seeding mesh"
    );

    // When source is "wireguard", the peers are already overlay IPs — register them as overlay
    let overlay_peers = if overlay_peers.is_empty() && source == "wireguard" {
        peers.clone()
    } else {
        overlay_peers
    };
    let peers_for_trust = peers.clone();
    let source_owned = String::from(source);
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
                    source = %source_owned,
                    "Mesh auto-seeded — discovery.peers is live"
                );

                if !overlay_peers.is_empty() {
                    register_overlay_endpoints(&mesh_handler, &overlay_peers, &overlay_name).await;
                }

                // Merge persisted LAN peers with SONGBIRD_LOCAL_PEERS env
                let mut all_lan_peers = lan_peers;
                for (nid, addr_str) in &local_peers_env {
                    if let Ok(addr) = addr_str.parse::<std::net::SocketAddr>()
                        && !all_lan_peers.iter().any(|(n, _)| n == nid)
                    {
                        all_lan_peers.push((nid.clone(), addr));
                    }
                }

                if !all_lan_peers.is_empty() {
                    register_lan_endpoints(&mesh_handler, &all_lan_peers).await;
                }

                let peer_addrs: Vec<(String, std::net::SocketAddr)> = peers_for_trust
                    .iter()
                    .filter_map(|(nid, addr)| {
                        addr.parse::<std::net::SocketAddr>().ok().map(|sa| (nid.clone(), sa))
                    })
                    .collect();
                crate::mesh_trust_exchange::spawn_trust_exchange(peer_addrs);
            }
            Err(e) => {
                warn!(error = %e, source = %source_owned, "Failed to auto-seed mesh");
            }
        }
    });
}

#[cfg(test)]
#[path = "mesh_seed_tests.rs"]
mod tests;
