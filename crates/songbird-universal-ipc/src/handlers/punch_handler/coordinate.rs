// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! UDP spray + listen for relay-assisted coordinated punch (`punch.coordinate`).

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};

const PUNCH_MSG: &[u8] = b"SONGBIRD_COORDINATED_PUNCH_V1";

/// Outcome of the local coordinated UDP punch attempt.
pub enum CoordinatePunchOutcome {
    /// Peer responded from the expected public IP.
    Success {
        from_addr: SocketAddr,
        latency: Duration,
        ports_tried: u32,
    },
    /// Listen timed out or packet was not from the expected IP.
    Timeout {
        ports_tried: u32,
    },
}

/// Bind an ephemeral UDP socket, spray predicted ports, then wait for a reply.
pub async fn run_coordinated_udp_punch(
    peer_public_ip: IpAddr,
    peer_predicted_port: u16,
) -> Result<CoordinatePunchOutcome, String> {
    let socket = Arc::new(
        tokio::net::UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| format!("Failed to bind socket: {e}"))?,
    );

    let spray_window: i32 = 3;
    let mut ports_tried: u32 = 0;
    let start = Instant::now();

    for offset in -spray_window..=spray_window {
        let target_port = u16::try_from((i32::from(peer_predicted_port) + offset).clamp(1, 65535))
            .unwrap_or(peer_predicted_port);
        let target_addr = SocketAddr::new(peer_public_ip, target_port);

        if socket.send_to(PUNCH_MSG, target_addr).await.is_ok() {
            ports_tried += 1;
        }
    }

    for _ in 0..3 {
        let target_addr = SocketAddr::new(peer_public_ip, peer_predicted_port);
        let _ = socket.send_to(PUNCH_MSG, target_addr).await;
    }

    let listen_timeout = Duration::from_secs(3);
    let mut recv_buf = vec![0u8; 1024];

    match tokio::time::timeout(listen_timeout, socket.recv_from(&mut recv_buf)).await {
        Ok(Ok((_len, from_addr))) if from_addr.ip() == peer_public_ip => {
            let latency = start.elapsed();
            Ok(CoordinatePunchOutcome::Success {
                from_addr,
                latency,
                ports_tried,
            })
        }
        _ => Ok(CoordinatePunchOutcome::Timeout {
            ports_tried,
        }),
    }
}
