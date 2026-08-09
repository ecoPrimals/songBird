// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `CanonicalTransport` implementation for Onion Relay (hole-punch + mesh).

use songbird_types::{CanonicalTransport, SongbirdResult, TransportEndpoint, TransportHealth};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Transport adapter for onion relay hole-punch coordination.
pub struct OnionRelayTransport {
    running: AtomicBool,
    active_relays: AtomicU64,
}

impl OnionRelayTransport {
    /// Create a new onion relay transport adapter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            active_relays: AtomicU64::new(0),
        }
    }

    /// Record an active relay session.
    pub fn relay_opened(&self) {
        self.active_relays.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a closed relay session.
    pub fn relay_closed(&self) {
        self.active_relays.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Default for OnionRelayTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonicalTransport for OnionRelayTransport {
    fn transport_name(&self) -> &'static str {
        "OnionRelay"
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
            active_connections: self.active_relays.load(Ordering::Relaxed),
            message: String::from("onion relay hole-punch mesh"),
        }
    }

    fn endpoints(&self) -> Vec<TransportEndpoint> {
        vec![]
    }
}
