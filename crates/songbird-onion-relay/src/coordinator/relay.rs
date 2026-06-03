// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Relay-assisted coordinated punch (signaling over an active relay session).

use crate::error::{OnionRelayError, Result};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::UdpSocket;
use tokio::time::{sleep, timeout};
use tracing::{debug, info, warn};

use super::core::HolePunchCoordinator;
use super::types::CoordinatedPunchResult;
use super::util::unix_epoch_millis_u64;

impl HolePunchCoordinator {
    /// Relay-assisted coordinated punch
    ///
    /// Uses an active relay session as the signaling channel, combined with
    /// port pattern predictions from STUN probing. If the punch succeeds,
    /// the data path swaps from relay to direct P2P. If it fails, the relay
    /// continues — you never go backwards.
    ///
    /// ## Protocol
    ///
    /// 1. Use port predictions from `stun.probe_port_pattern` for both peers
    /// 2. Coordinate timing via relay signaling channel
    /// 3. Spray predicted ports (± window for prediction error)
    /// 4. Listen for response — first valid reply = success
    /// 5. Report result: Direct (drop relay) or `KeepRelay` (continue)
    ///
    /// # Arguments
    ///
    /// * `peer_node_id` - Target peer's node ID
    /// * `relay_session` - Active relay session for signaling
    /// * `our_pattern` - Our NAT port allocation pattern
    /// * `peer_predicted_port` - Peer's predicted next port
    /// * `peer_public_ip` - Peer's public IP address
    ///
    /// # Errors
    ///
    /// Returns [`OnionRelayError`] when wall-clock time cannot be represented as
    /// milliseconds, UDP bind fails, coordination JSON encoding fails, or the relay
    /// channel cannot send the coordination message.
    ///
    /// # Panics
    ///
    /// This function does not panic.
    #[expect(clippy::too_many_lines, reason = "relay-assisted punch protocol state machine")]
    pub async fn coordinate_relay_punch(
        &self,
        peer_node_id: &str,
        relay_session: &songbird_lineage_relay::relay::RelaySession,
        our_pattern: &songbird_stun::PortPattern,
        peer_predicted_port: u16,
        peer_public_ip: std::net::IpAddr,
    ) -> Result<CoordinatedPunchResult> {
        info!(
            "🎯 Starting relay-assisted coordinated punch to {} (predicted port: {})",
            &peer_node_id[..8.min(peer_node_id.len())],
            peer_predicted_port
        );

        // 1. Bind a new socket for the punch attempt
        let socket =
            Arc::new(UdpSocket::bind(songbird_types::constants::EPHEMERAL_BIND_ADDR).await?);
        let local_addr = socket.local_addr()?;

        // 2. Signal coordination timing via relay
        let start_delay_ms: u64 = 200; // Allow network propagation
        let now_ms = unix_epoch_millis_u64()?;
        let start_at = now_ms + start_delay_ms;

        // Build coordination message
        let coord_msg = serde_json::json!({
            "type": "punch_coordinate",
            "from_node_id": self.my_node_id,
            "to_node_id": peer_node_id,
            "our_local_port": local_addr.port(),
            "peer_predicted_port": peer_predicted_port,
            "start_at_ms": start_at,
            "spray_window": 3,
        });

        let coord_bytes = serde_json::to_vec(&coord_msg)
            .map_err(|e| OnionRelayError::Other(format!("Failed to encode coordination: {e}")))?;

        relay_session.send(&coord_bytes).await.map_err(|e| {
            OnionRelayError::Transport(format!("Failed to send coordination via relay: {e}"))
        })?;

        debug!("📡 Sent coordination message via relay (start_at: {}ms)", start_at);

        // 3. Wait for start time
        let current_ms = unix_epoch_millis_u64()?;

        if start_at > current_ms {
            sleep(std::time::Duration::from_millis(start_at - current_ms)).await;
        }

        // 4. Spray predicted ports (± window for prediction error)
        let spray_window: i32 = 3;
        let punch_msg = b"SONGBIRD_COORDINATED_PUNCH_V1";
        let mut ports_tried: u32 = 0;
        let start = Instant::now();

        for offset in -spray_window..=spray_window {
            let clamped = (i32::from(peer_predicted_port) + offset).clamp(1, 65535);
            #[expect(
                clippy::cast_sign_loss,
                reason = "port clamped to 1..=65535 before cast to u16"
            )]
            let target_port = clamped as u16;
            let target_addr = SocketAddr::new(peer_public_ip, target_port);

            debug!("  🎯 Punch spray → {}:{}", peer_public_ip, target_port);

            if let Err(e) = socket.send_to(punch_msg, target_addr).await {
                debug!("  Send to port {} failed: {}", target_port, e);
                continue;
            }
            ports_tried += 1;
        }

        // Also punch the exact predicted port a few more times for reliability
        for _ in 0..3 {
            let target_addr = SocketAddr::new(peer_public_ip, peer_predicted_port);
            let _ = socket.send_to(punch_msg, target_addr).await;
        }

        // 5. Listen for any response with configurable timeout
        let listen_timeout = self.config.attempt_timeout * 3; // 3x normal for coordinated
        let mut recv_buf = vec![0u8; 1024];

        match timeout(listen_timeout, socket.recv_from(&mut recv_buf)).await {
            Ok(Ok((_len, from_addr))) if from_addr.ip() == peer_public_ip => {
                let latency = start.elapsed();
                info!(
                    "✅ Coordinated punch SUCCESS! {} → {} (latency: {:?})",
                    local_addr, from_addr, latency
                );

                Ok(CoordinatedPunchResult::Direct {
                    peer_addr: from_addr,
                    local_socket: socket,
                    latency,
                })
            }
            Ok(Ok((_len, from_addr))) => {
                warn!(
                    "⚠️ Received response from unexpected IP: {} (expected {})",
                    from_addr.ip(),
                    peer_public_ip
                );
                Ok(CoordinatedPunchResult::KeepRelay {
                    ports_tried,
                    reason: format!(
                        "Response from wrong IP ({}, expected {})",
                        from_addr.ip(),
                        peer_public_ip
                    ),
                })
            }
            Ok(Err(e)) => {
                info!("⚠️ Coordinated punch failed (recv error): {} — relay continues", e);
                Ok(CoordinatedPunchResult::KeepRelay {
                    ports_tried,
                    reason: format!("Network error: {e}"),
                })
            }
            Err(_) => {
                info!(
                    "⚠️ Coordinated punch timed out after {:?} — relay continues ({} ports tried)",
                    listen_timeout, ports_tried
                );

                // Check if our pattern is useful for a hint
                let _our_next = our_pattern.predict_next();

                Ok(CoordinatedPunchResult::KeepRelay {
                    ports_tried,
                    reason: format!(
                        "Timeout after {listen_timeout:?} ({ports_tried} ports sprayed)",
                    ),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use crate::coordinator::config::HolePunchConfig;
    use crate::coordinator::core::HolePunchCoordinator;
    use crate::coordinator::types::CoordinatedPunchResult;
    use songbird_lineage_relay::RelaySession;
    use songbird_lineage_relay::types::{MaskingLevel, NodeId};
    use songbird_stun::PortPattern;
    use std::net::SocketAddr;
    use std::time::Duration;
    use tokio::net::UdpSocket;

    #[tokio::test]
    async fn coordinate_relay_punch_keep_relay_when_no_udp_reply() {
        let server = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let relay_addr: SocketAddr = server.local_addr().unwrap();

        let session = RelaySession::new_unverified(
            NodeId("relay".into()),
            relay_addr,
            NodeId("me".into()),
            NodeId("peer".into()),
            MaskingLevel::None,
        )
        .await
        .expect("relay session");

        let config = HolePunchConfig {
            attempt_timeout: Duration::from_millis(0),
            ..Default::default()
        };

        let (coord, _in, _out) = HolePunchCoordinator::new("me".into(), config);

        let pattern = PortPattern::Unknown;
        let r = coord
            .coordinate_relay_punch(
                "peer",
                &session,
                &pattern,
                40_000,
                "127.0.0.1".parse().unwrap(),
            )
            .await
            .expect("coordination completes");

        assert!(
            matches!(r, CoordinatedPunchResult::KeepRelay { .. }),
            "expected KeepRelay when peer never answers UDP, got {r:?}"
        );
    }
}
