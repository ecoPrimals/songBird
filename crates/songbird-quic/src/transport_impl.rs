// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `CanonicalTransport` implementation for QUIC.

use songbird_types::{CanonicalTransport, SongbirdResult, TransportEndpoint, TransportHealth};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Transport adapter for QUIC server/client lifecycle.
pub struct QuicTransport {
    local_addr: SocketAddr,
    running: AtomicBool,
    active_streams: Arc<AtomicU64>,
}

impl QuicTransport {
    /// Create a new QUIC transport adapter.
    #[must_use]
    pub fn new(local_addr: SocketAddr) -> Self {
        Self {
            local_addr,
            running: AtomicBool::new(false),
            active_streams: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Get a handle to the active streams counter for external tracking.
    #[must_use]
    pub fn active_streams_handle(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.active_streams)
    }
}

impl CanonicalTransport for QuicTransport {
    fn transport_name(&self) -> &'static str {
        "QUIC"
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
            active_connections: self.active_streams.load(Ordering::Relaxed),
            message: String::from("QUIC 0-RTT multiplexed transport"),
        }
    }

    fn endpoints(&self) -> Vec<TransportEndpoint> {
        vec![TransportEndpoint::Tcp {
            host: self.local_addr.ip().to_string(),
            port: self.local_addr.port(),
        }]
    }
}
