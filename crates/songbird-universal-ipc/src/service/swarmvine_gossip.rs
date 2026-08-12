// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Mesh gossip injection helpers.
//!
//! Handles capability advertisement injection into the mesh gossip provider's
//! engine via UDS. The provider is discovered at runtime by capability
//! (`mesh_gossip`), never by primal name or hardcoded socket path.

use tracing::debug;

/// Capability token for epidemic gossip propagation (owned by mesh gossip provider).
#[cfg(unix)]
const MESH_GOSSIP_CAPABILITY: &str = "mesh_gossip";

/// Inject a capability advertisement into the mesh gossip provider.
///
/// Discovers the provider UDS socket via capability-based resolution, then
/// sends a `gossip.inject` JSON-RPC call with the capability advertisement.
/// Fire-and-forget: if the provider is unavailable, the announcement still
/// propagates via the existing songBird `mesh.capabilities_announce` path.
#[cfg(unix)]
pub(super) async fn inject_to_swarmvine(node_id: &str, primal_id: &str, capabilities: &[String]) {
    let socket = discover_mesh_gossip_socket();
    let Some(socket_path) = socket else {
        debug!("mesh gossip provider socket not found — skipping gossip inject");
        return;
    };

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "gossip.inject",
        "params": {
            "topic": "tower",
            "key": format!("capability.advertise:{node_id}:{primal_id}"),
            "payload": {
                "capabilities": capabilities,
                "primal": primal_id,
                "gate": node_id,
            }
        },
        "id": null,
    });

    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    match tokio::net::UnixStream::connect(&socket_path).await {
        Ok(stream) => {
            let mut reader = BufReader::new(stream);
            let writer = reader.get_mut();
            if writer.write_all(&[0xEC, 0x01]).await.is_err() {
                return;
            }
            let msg = format!("{payload}\n");
            if writer.write_all(msg.as_bytes()).await.is_err() {
                return;
            }
            let mut response = String::new();
            let _ = reader.read_line(&mut response).await;
            debug!(response = %response.trim(), "mesh gossip inject response");
        }
        Err(e) => {
            debug!(error = %e, "mesh gossip provider not reachable — gossip inject skipped");
        }
    }
}

#[cfg(not(unix))]
pub(super) async fn inject_to_swarmvine(
    _node_id: &str,
    _primal_id: &str,
    _capabilities: &[String],
) {
    debug!("mesh gossip inject not available on this platform (UDS-only)");
}

/// Discover mesh gossip provider socket via capability-based resolution.
///
/// Priority:
/// 1. `CAPABILITY_MESH_GOSSIP_ENDPOINT` (unix path or `unix://` URL)
/// 2. `MESH_GOSSIP_PROVIDER_SOCKET` / `MESH_GOSSIP_SOCKET`
/// 3. `$XDG_RUNTIME_DIR/biomeos/mesh-gossip.sock` (capability symlink)
/// 4. Temp biomeOS capability-named socket candidates
#[cfg(unix)]
pub(super) fn discover_swarmvine_socket() -> Option<std::path::PathBuf> {
    discover_mesh_gossip_socket()
}

/// Discover mesh gossip provider socket (capability-based; no primal names).
#[cfg(unix)]
fn discover_mesh_gossip_socket() -> Option<std::path::PathBuf> {
    if let Ok(endpoint) = songbird_process_env::var("CAPABILITY_MESH_GOSSIP_ENDPOINT")
        && !endpoint.is_empty()
    {
        return parse_unix_socket_endpoint(&endpoint);
    }

    for env_key in ["MESH_GOSSIP_PROVIDER_SOCKET", "MESH_GOSSIP_SOCKET"] {
        if let Ok(path) = songbird_process_env::var(env_key)
            && !path.is_empty()
        {
            return parse_unix_socket_endpoint(&path);
        }
    }

    if let Ok(runtime_dir) = songbird_process_env::var("XDG_RUNTIME_DIR") {
        for name in songbird_types::defaults::paths::MESH_GOSSIP_CAPABILITY_SOCKET_FILENAMES {
            let p = std::path::PathBuf::from(&runtime_dir)
                .join(songbird_types::defaults::paths::BIOMEOS_RUNTIME_SUBDIR)
                .join(name);
            if p.exists() {
                return Some(p);
            }
        }
    }

    for p in songbird_types::defaults::paths::mesh_gossip_socket_candidates() {
        if p.exists() {
            return Some(p);
        }
    }

    debug!(capability = MESH_GOSSIP_CAPABILITY, "mesh gossip provider socket not found");
    None
}

#[cfg(unix)]
fn parse_unix_socket_endpoint(endpoint: &str) -> Option<std::path::PathBuf> {
    let path = endpoint.strip_prefix("unix://").unwrap_or(endpoint);
    if path.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(path))
    }
}

#[cfg(not(unix))]
#[allow(dead_code, reason = "stub for platform parity; callers are cfg(unix)")]
pub(super) fn discover_swarmvine_socket() -> Option<std::path::PathBuf> {
    None
}
