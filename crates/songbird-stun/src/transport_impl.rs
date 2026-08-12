// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! `CanonicalTransport` implementation for STUN.
//!
//! Wraps the STUN server in a transport-agnostic lifecycle adapter.
//! The `StunTransport` struct holds the configuration and runtime state
//! needed for the `CanonicalTransport` interface while `StunServer` itself
//! retains its domain-specific API unchanged.

use songbird_types::{
    CanonicalTransport, SongbirdError, SongbirdResult, TransportEndpoint, TransportHealth,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::StunServer;

/// Transport adapter for the STUN server.
pub struct StunTransport {
    bind_addr: SocketAddr,
    server: Arc<StunServer>,
    running: Arc<RwLock<bool>>,
}

impl StunTransport {
    /// Create a new STUN transport adapter.
    #[must_use]
    pub fn new(bind_addr: SocketAddr) -> Self {
        Self {
            bind_addr,
            server: Arc::new(StunServer::new(bind_addr)),
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Access the underlying server for domain-specific operations.
    #[must_use]
    pub fn server(&self) -> &StunServer {
        &self.server
    }
}

impl CanonicalTransport for StunTransport {
    fn transport_name(&self) -> &'static str {
        "STUN"
    }

    async fn is_ready(&self) -> bool {
        *self.running.read().await
    }

    async fn start(&self) -> SongbirdResult<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(SongbirdError::service("stun", "transport already running"));
        }
        *running = true;
        Ok(())
    }

    async fn shutdown(&self) -> SongbirdResult<()> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }

    async fn health(&self) -> TransportHealth {
        let stats = self.server.stats().await;
        let ready = *self.running.read().await;
        TransportHealth {
            ready,
            active_connections: stats.requests_handled,
            message: format!("handled={} errors={}", stats.requests_handled, stats.errors),
        }
    }

    fn endpoints(&self) -> Vec<TransportEndpoint> {
        vec![TransportEndpoint::Tcp {
            host: self.bind_addr.ip().to_string(),
            port: self.bind_addr.port(),
        }]
    }
}
