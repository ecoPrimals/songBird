// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! swarmVine gossip injection helpers.
//!
//! Handles capability advertisement injection into the local swarmVine gossip
//! engine via UDS, and socket discovery for the swarmVine primal.

use tracing::debug;

/// Inject a capability advertisement into the local swarmVine gossip engine.
///
/// Discovers the swarmVine UDS socket via standard biomeOS resolution, then
/// sends a `gossip.inject` JSON-RPC call with the capability advertisement.
/// Fire-and-forget: if swarmVine is unavailable, the announcement still
/// propagates via the existing songBird `mesh.capabilities_announce` path.
#[cfg(unix)]
pub(super) async fn inject_to_swarmvine(node_id: &str, primal_id: &str, capabilities: &[String]) {
    let socket = discover_swarmvine_socket();
    let Some(socket_path) = socket else {
        debug!("swarmVine socket not found — skipping gossip inject (not deployed yet)");
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
            debug!(response = %response.trim(), "swarmVine gossip.inject response");
        }
        Err(e) => {
            debug!(error = %e, "swarmVine not reachable — gossip inject skipped");
        }
    }
}

#[cfg(not(unix))]
pub(super) async fn inject_to_swarmvine(_node_id: &str, _primal_id: &str, _capabilities: &[String]) {
    debug!("swarmVine gossip inject not available on this platform (UDS-only)");
}

/// Discover swarmVine socket via standard biomeOS resolution.
#[cfg(unix)]
pub(super) fn discover_swarmvine_socket() -> Option<std::path::PathBuf> {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let p = std::path::PathBuf::from(format!("{runtime_dir}/biomeos/swarmvine.sock"));
        if p.exists() {
            return Some(p);
        }
    }
    let p = std::path::PathBuf::from("/tmp/biomeos/swarmvine.sock");
    if p.exists() {
        return Some(p);
    }
    None
}

#[cfg(not(unix))]
pub(super) fn discover_swarmvine_socket() -> Option<std::path::PathBuf> {
    None
}
