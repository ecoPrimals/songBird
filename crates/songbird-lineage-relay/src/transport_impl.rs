// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `CanonicalTransport` implementation for Lineage Relay.

use songbird_types::{CanonicalTransport, SongbirdResult, TransportEndpoint, TransportHealth};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Transport adapter for genetic lineage relay (sovereignty-first P2P).
pub struct LineageRelayTransport {
    running: AtomicBool,
    active_sessions: AtomicU64,
}

impl LineageRelayTransport {
    /// Create a new lineage relay transport adapter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            active_sessions: AtomicU64::new(0),
        }
    }

    /// Record an active relay session.
    pub fn session_opened(&self) {
        self.active_sessions.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a closed relay session.
    pub fn session_closed(&self) {
        self.active_sessions.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Default for LineageRelayTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonicalTransport for LineageRelayTransport {
    fn transport_name(&self) -> &'static str {
        "LineageRelay"
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
            active_connections: self.active_sessions.load(Ordering::Relaxed),
            message: String::from("genetic lineage relay sessions"),
        }
    }

    fn endpoints(&self) -> Vec<TransportEndpoint> {
        vec![]
    }
}
