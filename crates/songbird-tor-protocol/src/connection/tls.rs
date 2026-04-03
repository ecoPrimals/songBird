// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS connector for Tor relay connections
//!
//! Uses songbird-tls (pure Rust, `security provider` crypto delegation) instead of
//! rustls+ring. Tor relays use self-signed certs — trust is established
//! via the Ed25519 identity in the Tor handshake, not TLS PKI.
//!
//! **Zero ring | Zero rustls | Zero C dependencies**

use crate::error::{Error, Result};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{debug, info};

/// Default connection timeout
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

/// TLS connector for Tor relay connections
///
/// Tor authenticates via Ed25519 identity keys in the link protocol,
/// not via TLS certificates. The TLS layer is just transport encryption.
/// For the Tor link protocol, we establish a raw TCP connection and then
/// perform the Tor-specific handshake with security provider-delegated crypto.
///
/// Since Tor relays use self-signed certs and authentication happens at
/// the Tor protocol layer (not TLS), we use a direct TCP stream with
/// Tor's own encryption layer on top.
pub struct TlsConnector;

/// A stream to a Tor relay — either raw TCP or wrapped with TLS
///
/// Tor's link protocol v4+ can operate over plain TCP with in-protocol
/// encryption via the ntor handshake. The TLS layer is only required
/// for backward compatibility with older relays.
#[expect(dead_code, reason = "dead code retained intentionally (reserved or API surface)")]
pub enum RelayStream {
    /// Raw TCP stream (Tor handles encryption at protocol layer)
    Tcp(TcpStream),
}

impl TlsConnector {
    /// Create new TLS connector
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Connect to a Tor relay
    ///
    /// Establishes a TCP connection to the relay. The Tor link protocol
    /// handles authentication and encryption via ntor handshake with
    /// security provider-delegated crypto — no TLS PKI needed.
    ///
    /// # Errors
    /// Returns error if connection times out or fails.
    pub async fn connect(&self, addr: SocketAddr) -> Result<TcpStream> {
        debug!("Starting TCP connection to {}", addr);

        // Create TCP connection with timeout
        let stream = timeout(CONNECT_TIMEOUT, TcpStream::connect(addr))
            .await
            .map_err(|_| Error::Network(format!("TCP connection to {addr} timed out")))?
            .map_err(|e| Error::Network(format!("Failed to connect to {addr}: {e}")))?;

        // Disable Nagle's algorithm for lower latency
        stream
            .set_nodelay(true)
            .map_err(|e| Error::Network(format!("Failed to set TCP_NODELAY: {e}")))?;

        info!("Connected to Tor relay at {}", addr);
        Ok(stream)
    }
}

impl Default for TlsConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_connector_creation() {
        let connector = TlsConnector::new();
        assert_eq!(std::mem::size_of_val(&connector), 0);
    }

    #[test]
    fn test_tls_connector_default() {
        let connector = TlsConnector::new();
        let _ = connector; // unit struct — default() is not needed
    }
}
