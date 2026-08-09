// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `CanonicalTransport` implementation for TURN client.

use songbird_types::{CanonicalTransport, SongbirdResult, TransportEndpoint, TransportHealth};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Transport adapter for TURN relay client sessions.
pub struct TurnClientTransport {
    running: AtomicBool,
    active_allocations: AtomicU64,
}

impl TurnClientTransport {
    /// Create a new TURN client transport adapter.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            active_allocations: AtomicU64::new(0),
        }
    }

    /// Record an active TURN allocation.
    pub fn allocation_opened(&self) {
        self.active_allocations.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a released TURN allocation.
    pub fn allocation_closed(&self) {
        self.active_allocations.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Default for TurnClientTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonicalTransport for TurnClientTransport {
    fn transport_name(&self) -> &'static str {
        "TURN"
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
            active_connections: self.active_allocations.load(Ordering::Relaxed),
            message: String::from("TURN relay allocations"),
        }
    }

    fn endpoints(&self) -> Vec<TransportEndpoint> {
        vec![]
    }
}
