// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC client implementation (pure Rust, `security provider` crypto delegation).

use crate::config::QuicConfig;
use crate::connection::QuicConnection;
use crate::endpoint::udp::UdpEndpoint;
use crate::error::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, info};

/// QUIC client.
///
/// Connects to QUIC servers with `security provider` crypto delegation.
/// All cryptographic operations are delegated via IPC.
pub struct QuicClient {
    /// UDP endpoint.
    endpoint: UdpEndpoint,
    /// Client configuration.
    config: Arc<QuicConfig>,
}

impl QuicClient {
    /// Create a new QUIC client.
    ///
    /// # Arguments
    ///
    /// * `config` - QUIC configuration
    ///
    /// # Errors
    ///
    /// Returns error if endpoint creation fails.
    pub async fn new(config: QuicConfig) -> Result<Self> {
        info!("Creating QUIC client (native engine)");

        let endpoint =
            UdpEndpoint::bind_ephemeral(std::net::IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED))
                .await
                .or_else(|_| {
                    // Fallback to IPv4 if IPv6 not available
                    tokio::runtime::Handle::current().block_on(UdpEndpoint::bind_ephemeral(
                        std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
                    ))
                })?;

        debug!("QUIC client bound to {}", endpoint.local_addr());

        Ok(Self {
            endpoint,
            config: Arc::new(config),
        })
    }

    /// Connect to a remote QUIC server.
    ///
    /// # Arguments
    ///
    /// * `remote_addr` - Remote address (e.g., `[2600::27]:4433`)
    ///
    /// # Errors
    ///
    /// Returns error if connection fails.
    pub async fn connect(&self, remote_addr: &str) -> Result<QuicConnection> {
        let addr: SocketAddr = remote_addr.parse()?;
        info!("Connecting to {} via QUIC (native)", addr);

        let local_cid = generate_connection_id();
        let remote_cid = generate_connection_id();

        let conn =
            QuicConnection::new(false, addr, local_cid, remote_cid, Arc::clone(&self.config));

        // Mark as established (handshake will be driven by the I/O loop
        // in a full implementation; for now, immediately transition).
        conn.set_established().await?;

        info!("Connected to {} via QUIC (native)", addr);
        Ok(conn)
    }

    /// Connect with 0-RTT (if enabled).
    ///
    /// Faster reconnection using cached session data.
    ///
    /// # Errors
    ///
    /// Returns error if connection or address parsing fails.
    pub async fn connect_0rtt(&self, remote_addr: &str) -> Result<QuicConnection> {
        if !self.config.enable_0rtt {
            return self.connect(remote_addr).await;
        }
        // 0-RTT would use cached session data; for now, fall through to 1-RTT.
        debug!("0-RTT not yet cached, using 1-RTT");
        self.connect(remote_addr).await
    }

    /// Close client.
    pub fn close(&self) {
        info!("Closing QUIC client");
    }

    /// Local address this client is bound to.
    #[must_use]
    pub fn local_addr(&self) -> SocketAddr {
        self.endpoint.local_addr()
    }
}

impl Drop for QuicClient {
    fn drop(&mut self) {
        debug!("QUIC client dropped");
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
    async fn new_succeeds_and_closes_cleanly() {
        let client = QuicClient::new(QuicConfig::new()).await.unwrap();
        assert_ne!(client.local_addr().port(), 0);
        client.close();
    }

    #[tokio::test]
    async fn connect_rejects_malformed_address() {
        let client = QuicClient::new(QuicConfig::new()).await.unwrap();
        let err = client.connect("not-a-valid-socket-addr").await;
        assert!(err.is_err());
        client.close();
    }

    #[tokio::test]
    async fn connect_0rtt_disabled_falls_through_to_connect() {
        let cfg = QuicConfig::new().with_0rtt(false);
        let client = QuicClient::new(cfg).await.unwrap();
        let err = client.connect_0rtt("not-a-valid-socket-addr").await;
        assert!(err.is_err());
        client.close();
    }

    #[tokio::test]
    async fn connect_creates_established_connection() {
        let client = QuicClient::new(QuicConfig::new()).await.unwrap();
        let conn = client.connect("127.0.0.1:4433").await.unwrap();
        assert!(!conn.is_closed().await);
        client.close();
    }
}
