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
    /// Peer node ID — if provided, registers the peer in the mesh on success.
    /// If omitted, a federation probe is attempted to discover it.
    pub node_id: Option<String>,
    /// Whether to register this peer in the mesh on successful connect.
    /// Defaults to true when omitted.
    pub register_mesh: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PeerConnectResult {
    /// Connection ID
    pub connection_id: String,
    /// Connection state
    pub state: String, // "connecting", "connected", "failed"
    /// Established channel info (if connected)
    pub channel: Option<PeerChannel>,
    /// Discovered or provided `node_id` (populated on successful mesh registration)
    pub node_id: Option<String>,
    /// Whether the peer was registered in the mesh
    pub mesh_registered: bool,
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
