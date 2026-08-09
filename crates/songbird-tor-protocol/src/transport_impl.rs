// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `CanonicalTransport` implementation for Tor protocol.

use songbird_types::{CanonicalTransport, SongbirdResult, TransportEndpoint, TransportHealth};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Transport adapter for Tor onion routing.
pub struct TorTransport {
    running: AtomicBool,
    active_circuits: AtomicU64,
}

impl TorTransport {
    /// Create a new Tor transport adapter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            active_circuits: AtomicU64::new(0),
        }
    }

    /// Record an active Tor circuit.
    pub fn circuit_opened(&self) {
        self.active_circuits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a closed Tor circuit.
    pub fn circuit_closed(&self) {
        self.active_circuits.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Default for TorTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonicalTransport for TorTransport {
    fn transport_name(&self) -> &'static str {
        "Tor"
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
            active_connections: self.active_circuits.load(Ordering::Relaxed),
            message: String::from("Tor onion routing circuits"),
        }
    }

    fn endpoints(&self) -> Vec<TransportEndpoint> {
        vec![]
    }
}
