// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Relay endpoint descriptors and transport kinds for the beacon mesh.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

/// A relay endpoint (could be direct, family relay, or Tor)
#[derive(Debug, Clone)]
pub struct RelayEndpoint {
    /// Node ID of the relay
    pub node_id: String,
    /// How to reach this relay
    pub endpoint_type: EndpointType,
    /// Last measured latency
    pub latency: Option<Duration>,
    /// Last successful contact
    pub last_seen: Instant,
    /// Is this relay currently reachable?
    pub reachable: bool,
}

/// Type of relay endpoint
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndpointType {
    /// Direct UDP connection (hole punch succeeded)
    Direct {
        /// Peer's reachable UDP address.
        addr: SocketAddr,
    },
    /// Relay through another family member
    FamilyRelay {
        /// Node id of the relay participant.
        relay_node_id: String,
    },
    /// Tor onion service (bootstrap/fallback)
    TorOnion {
        /// `.onion` hostname or full rendezvous URL.
        onion_addr: String,
    },
    /// Local network (same LAN)
    Local {
        /// LAN peer address.
        addr: SocketAddr,
    },
}

impl EndpointType {
    /// Priority for selection (lower = better)
    #[must_use]
    pub const fn priority(&self) -> u8 {
        match self {
            Self::Local {
                ..
            } => 0,
            Self::Direct {
                ..
            } => 1,
            Self::FamilyRelay {
                ..
            } => 2,
            Self::TorOnion {
                ..
            } => 3,
        }
    }
}
