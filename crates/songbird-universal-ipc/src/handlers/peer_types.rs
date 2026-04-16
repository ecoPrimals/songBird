// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Shared peer connection types (extracted to avoid circular deps between `peer_handler` and `udp_peer_connector`).

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerConnectParams {
    /// Target peer address (IP:port)
    pub target_address: String,
    /// Our STUN binding (for symmetric NAT, optional)
    pub our_binding: Option<String>,
    /// Rendezvous token (if using rendezvous, optional)
    pub rendezvous_token: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PeerConnectResult {
    /// Connection ID
    pub connection_id: String,
    /// Connection state
    pub state: String, // "connecting", "connected", "failed"
    /// Established channel info (if connected)
    pub channel: Option<PeerChannel>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PeerChannel {
    /// Local endpoint
    pub local_address: String,
    /// Remote endpoint
    pub remote_address: String,
    /// Protocol (udp/tcp)
    pub protocol: String,
    /// Latency (ms, if measured)
    pub latency_ms: Option<u64>,
}
