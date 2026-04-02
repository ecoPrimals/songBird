// SPDX-License-Identifier: AGPL-3.0-only
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
