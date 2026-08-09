// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `CanonicalTransport` implementation for Network Federation (BTSP).

use songbird_types::{CanonicalTransport, SongbirdResult, TransportEndpoint, TransportHealth};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Transport adapter for network federation (BTSP multi-provider).
pub struct FederationTransport {
    running: AtomicBool,
    active_peers: AtomicU64,
}

impl FederationTransport {
    /// Create a new federation transport adapter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            active_peers: AtomicU64::new(0),
        }
    }

    /// Record a connected federation peer.
    pub fn peer_connected(&self) {
        self.active_peers.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a disconnected federation peer.
    pub fn peer_disconnected(&self) {
        self.active_peers.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Default for FederationTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonicalTransport for FederationTransport {
    fn transport_name(&self) -> &'static str {
        "Federation"
    }

    async fn is_ready(&self) -> bool {
        self.running.load(Ordering::Acquire)
    }

    async fn start(&self) -> SongbirdResult<()> {
        self.running.store(true, Ordering::Release);
        Ok(())
    }

    async fn shutdown(&self) -> SongbirdResult<()> {
        self.running.store(false, Ordering::Release);
        Ok(())
    }

    async fn health(&self) -> TransportHealth {
        TransportHealth {
            ready: self.running.load(Ordering::Acquire),
            active_connections: self.active_peers.load(Ordering::Relaxed),
            message: String::from("BTSP network federation"),
        }
    }

    fn endpoints(&self) -> Vec<TransportEndpoint> {
        vec![]
    }
}
