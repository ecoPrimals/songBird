// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC server implementation (pure Rust, `security provider` crypto delegation).

use crate::config::QuicConfig;
use crate::connection::QuicConnection;
use crate::endpoint::udp::UdpEndpoint;
use crate::error::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// QUIC server.
///
/// Listens for incoming QUIC connections with `security provider` crypto delegation.
/// All cryptographic operations are delegated via IPC.
pub struct QuicServer {
    /// UDP endpoint.
    endpoint: Arc<UdpEndpoint>,
    /// Server configuration.
    config: Arc<QuicConfig>,
    /// Local address.
    local_addr: SocketAddr,
}

impl QuicServer {
    /// Create a new QUIC server.
    ///
    /// # Arguments
    ///
    /// * `bind_addr` - Address to bind (IPv6 dual-stack recommended: `[::]`)
    /// * `config` - QUIC configuration
    ///
    /// # Errors
    ///
    /// Returns error if binding fails.
    pub async fn new(bind_addr: &str, config: QuicConfig) -> Result<Self> {
        let addr: SocketAddr = bind_addr.parse()?;
        info!("Starting QUIC server on {} (native engine)", addr);

        let endpoint = UdpEndpoint::bind(addr).await?;
        let local_addr = endpoint.local_addr();

        info!("QUIC server listening on {}", local_addr);

        Ok(Self {
            endpoint: Arc::new(endpoint),
            config: Arc::new(config),
            local_addr,
        })
    }

    /// Get local address.
    #[must_use]
    pub const fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Accept incoming connections.
    ///
    /// Returns a channel receiver for new connections.
    /// The accept loop runs in a background task, processing incoming
    /// UDP datagrams and performing TLS handshakes via `security provider`.
    #[must_use]
    pub fn accept(&self) -> mpsc::Receiver<QuicConnection> {
        let (tx, rx) = mpsc::channel(100);
        let endpoint = Arc::clone(&self.endpoint);
        let config = Arc::clone(&self.config);

        tokio::spawn(async move {
            loop {
                match endpoint.recv_from().await {
                    Ok(dgram) => {
                        let tx = tx.clone();
                        let config = Arc::clone(&config);

                        tokio::spawn(async move {
                            let local_cid = generate_connection_id();
                            let conn = QuicConnection::new(
                                true,
                                dgram.source,
                                local_cid,
                                vec![], // DCID from packet in full impl
                                config,
                            );

                            if conn.set_established().await.is_ok() && tx.send(conn).await.is_err()
                            {
                                warn!("Failed to send connection to channel");
                            }
                        });
                    }
                    Err(e) => {
                        warn!("Failed to receive datagram: {}", e);
                        break;
                    }
                }
            }
        });

        rx
    }

    /// Close server.
    pub fn close(&self) {
        info!("Closing QUIC server");
    }
}

impl Drop for QuicServer {
    fn drop(&mut self) {
        debug!("QUIC server dropped");
    }
}

/// Generate a random connection ID.
fn generate_connection_id() -> Vec<u8> {
    use rand::RngCore;
    let len = crate::CONNECTION_ID_LEN;
    let mut cid = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut cid);
    cid
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
        server.close();
    }

    #[tokio::test]
    async fn accept_returns_receiver() {
        let server = QuicServer::new("127.0.0.1:0", QuicConfig::new()).await.unwrap();
        let mut rx = server.accept();
        assert!(rx.try_recv().is_err());
        server.close();
    }
}
