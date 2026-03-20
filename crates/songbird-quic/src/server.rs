// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC server implementation

use crate::config::QuicConfig;
use crate::connection::QuicConnection;
use crate::error::Result;
use quinn::{Endpoint, Incoming};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// QUIC server
///
/// Listens for incoming QUIC connections with `BearDog` crypto delegation
pub struct QuicServer {
    /// Quinn endpoint
    endpoint: Endpoint,

    /// Server configuration
    config: Arc<QuicConfig>,

    /// Local address
    local_addr: SocketAddr,
}

impl QuicServer {
    /// Create new QUIC server
    ///
    /// # Arguments
    ///
    /// * `bind_addr` - Address to bind (IPv6 dual-stack recommended: `[::]`)
    /// * `config` - QUIC configuration
    ///
    /// # Errors
    ///
    /// Returns error if binding fails or configuration invalid
    #[allow(clippy::unused_async)] // async retained for API consistency with accept()
    pub async fn new(bind_addr: &str, config: QuicConfig) -> Result<Self> {
        let addr: SocketAddr = bind_addr.parse()?;

        info!("Starting QUIC server on {}", addr);

        // Build server configuration
        let server_config = config.build_server_config()?;

        // Create endpoint
        let endpoint = Endpoint::server(server_config, addr)?;
        let local_addr = endpoint.local_addr()?;

        info!("✅ QUIC server listening on {}", local_addr);

        Ok(Self {
            endpoint,
            config: Arc::new(config),
            local_addr,
        })
    }

    /// Get local address
    #[must_use]
    pub const fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Accept incoming connections
    ///
    /// Returns channel receiver for new connections
    #[must_use]
    pub fn accept(&self) -> mpsc::Receiver<QuicConnection> {
        let (tx, rx) = mpsc::channel(100);
        let endpoint = self.endpoint.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            while let Some(incoming) = endpoint.accept().await {
                let tx = tx.clone();
                let config = config.clone();

                tokio::spawn(async move {
                    match Self::handle_incoming(incoming, config).await {
                        Ok(conn) => {
                            if tx.send(conn).await.is_err() {
                                warn!("Failed to send connection to channel");
                            }
                        }
                        Err(e) => {
                            warn!("Failed to handle incoming connection: {}", e);
                        }
                    }
                });
            }
        });

        rx
    }

    /// Handle incoming connection
    async fn handle_incoming(
        incoming: Incoming,
        config: Arc<QuicConfig>,
    ) -> Result<QuicConnection> {
        let remote_addr = incoming.remote_address();
        debug!("Accepting connection from {}", remote_addr);

        let connection = incoming.await?;

        info!("✅ Connection established with {}", remote_addr);

        Ok(QuicConnection::new(connection, config))
    }

    /// Close server
    pub async fn close(&self) {
        info!("Closing QUIC server");
        self.endpoint.close(0u32.into(), b"server shutdown");
        self.endpoint.wait_idle().await;
    }
}

impl Drop for QuicServer {
    fn drop(&mut self) {
        debug!("QUIC server dropped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn new_rejects_invalid_bind_addr() {
        let r = QuicServer::new("!!!not-an-address!!!", QuicConfig::new()).await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn new_binds_local_udp() {
        let server = QuicServer::new("127.0.0.1:0", QuicConfig::new()).await.unwrap();
        assert_ne!(server.local_addr().port(), 0);
        server.close().await;
    }

    #[tokio::test]
    async fn accept_returns_receiver() {
        let server = QuicServer::new("127.0.0.1:0", QuicConfig::new()).await.unwrap();
        let mut rx = server.accept();
        assert!(rx.try_recv().is_err());
        server.close().await;
    }
}
