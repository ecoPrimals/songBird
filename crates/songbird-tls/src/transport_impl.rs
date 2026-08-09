// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `CanonicalTransport` implementation for TLS.

use songbird_types::{CanonicalTransport, SongbirdResult, TransportEndpoint, TransportHealth};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Transport adapter for TLS handshake/record layer services.
pub struct TlsTransport {
    running: AtomicBool,
    active_sessions: AtomicU64,
}

impl TlsTransport {
    /// Create a new TLS transport adapter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            active_sessions: AtomicU64::new(0),
        }
    }

    /// Increment active session count.
    pub fn session_opened(&self) {
        self.active_sessions.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active session count.
    pub fn session_closed(&self) {
        self.active_sessions.fetch_sub(1, Ordering::Relaxed);
    }
}

impl Default for TlsTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl CanonicalTransport for TlsTransport {
    fn transport_name(&self) -> &'static str {
        "TLS"
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
            message: String::from("TLS 1.3 security-provider crypto"),
        }
    }

    fn endpoints(&self) -> Vec<TransportEndpoint> {
        vec![]
    }
}
