// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! UDP multicast / broadcast discovery for local mesh peers.

use serde_json::Value;
use std::net::SocketAddr;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Sends a beacon on multicast and broadcast, collects `songbird_discovery` responses.
///
/// Returns (`node_id`, `SocketAddr` using the peer IP and announced `jsonrpc_port`).
pub(super) async fn udp_multicast_discover(
    our_node_id: &str,
    port: u16,
    timeout: Duration,
) -> Vec<(String, SocketAddr)> {
    let mut discovered = Vec::new();

    let socket = match tokio::net::UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to bind UDP socket for discovery: {}", e);
            return discovered;
        }
    };

    if let Err(e) = socket.set_broadcast(true) {
        warn!("Failed to enable broadcast: {}", e);
        return discovered;
    }

    let jsonrpc_port: u16 = songbird_process_env::var("SONGBIRD_HTTP_PORT")
        .or_else(|_| songbird_process_env::var("SONGBIRD_PORT"))
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let beacon = serde_json::json!({
        "type": "songbird_discovery",
        "node_id": our_node_id,
        "version": env!("CARGO_PKG_VERSION"),
        "jsonrpc_port": jsonrpc_port,
        "capabilities": ["mesh", "relay", "stun", "punch"],
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });

    let beacon_bytes = serde_json::to_vec(&beacon).unwrap_or_default();

    let multicast_addr = SocketAddr::from(([239, 255, 77, 77], port));
    let broadcast_addr = SocketAddr::from(([255, 255, 255, 255], port));

    let _ = socket.send_to(&beacon_bytes, multicast_addr).await;
    let _ = socket.send_to(&beacon_bytes, broadcast_addr).await;

    let mut buf = vec![0u8; 4096];
    let deadline = tokio::time::Instant::now() + timeout;

    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }

        match tokio::time::timeout(remaining, socket.recv_from(&mut buf)).await {
            Ok(Ok((len, addr))) => {
                if let Ok(response) = serde_json::from_slice::<Value>(&buf[..len])
                    && (response.get("type").and_then(|t| t.as_str())
                        == Some("songbird_discovery_response")
                        || response.get("type").and_then(|t| t.as_str())
                            == Some("songbird_discovery"))
                    && let Some(peer_id) = response.get("node_id").and_then(|n| n.as_str())
                    && peer_id != our_node_id
                {
                    let jsonrpc_port = u16::try_from(
                        response
                            .get("jsonrpc_port")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or(8080),
                    )
                    .unwrap_or(8080);
                    let peer_addr = SocketAddr::new(addr.ip(), jsonrpc_port);
                    info!(
                        "🔍 Discovered peer {} at {} (jsonrpc_port: {})",
                        peer_id, peer_addr, jsonrpc_port
                    );
                    discovered.push((peer_id.to_string(), peer_addr));
                }
            }
            Ok(Err(e)) => {
                debug!("UDP recv error during discovery: {}", e);
                break;
            }
            Err(_) => {
                break;
            }
        }
    }

    discovered
}
