// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Result types for hole punch and relay-assisted coordination.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;

/// Result of hole punch attempt
#[derive(Debug, Clone)]
pub enum PunchResult {
    /// Direct connection established
    Direct {
        /// Confirmed peer UDP endpoint.
        peer_addr: SocketAddr,
        /// Local socket bound for the session.
        local_socket: Arc<UdpSocket>,
        /// Time from punch start to first inbound datagram.
        latency: Duration,
    },
    /// Must use relay (hole punch failed)
    Relay {
        /// Number of punch rounds attempted before giving up.
        attempts: u32,
    },
}

/// Result of a relay-assisted coordinated punch attempt
#[derive(Debug, Clone)]
pub enum CoordinatedPunchResult {
    /// Direct connection established — relay can be dropped
    Direct {
        /// Peer's confirmed address
        peer_addr: SocketAddr,
        /// Local socket for direct communication
        local_socket: Arc<UdpSocket>,
        /// Measured latency
        latency: Duration,
    },
    /// Coordinated punch failed — relay remains active (zero disruption)
    KeepRelay {
        /// Number of ports sprayed
        ports_tried: u32,
        /// Reason for failure
        reason: String,
    },
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;

    #[tokio::test]
    async fn punch_result_direct_and_relay_clone_debug() {
        let socket = Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap());
        let peer: SocketAddr = "127.0.0.1:9".parse().unwrap();
        let d = PunchResult::Direct {
            peer_addr: peer,
            local_socket: socket,
            latency: Duration::from_micros(1),
        };
        let _ = format!("{d:?}");
        assert!(matches!(d, PunchResult::Direct { .. }));

        let r = PunchResult::Relay {
            attempts: 3,
        };
        assert!(matches!(
            r,
            PunchResult::Relay {
                attempts: 3
            }
        ));
    }

    #[tokio::test]
    async fn coordinated_punch_result_direct_and_keep_relay() {
        let socket = Arc::new(UdpSocket::bind("127.0.0.1:0").await.unwrap());
        let d = CoordinatedPunchResult::Direct {
            peer_addr: "127.0.0.1:1".parse().unwrap(),
            local_socket: socket,
            latency: Duration::from_nanos(1),
        };
        let _ = format!("{d:?}");
        assert!(matches!(d, CoordinatedPunchResult::Direct { .. }));

        let k = CoordinatedPunchResult::KeepRelay {
            ports_tried: 2,
            reason: "timeout".into(),
        };
        assert!(matches!(k, CoordinatedPunchResult::KeepRelay { .. }));
    }
}
