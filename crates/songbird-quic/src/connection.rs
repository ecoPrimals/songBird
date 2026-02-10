//! QUIC connection handling

use crate::config::QuicConfig;
use crate::error::{QuicError, Result};
use crate::stream::QuicStream;
use quinn::Connection;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::debug;

/// QUIC connection
///
/// Represents an established QUIC connection with multiplexed streams
pub struct QuicConnection {
    /// Quinn connection
    connection: Connection,

    /// Configuration (used during stream negotiation)
    _config: Arc<QuicConfig>,
}

impl QuicConnection {
    /// Create new connection wrapper
    pub(crate) fn new(connection: Connection, config: Arc<QuicConfig>) -> Self {
        Self {
            connection,
            _config: config,
        }
    }

    /// Open bidirectional stream
    ///
    /// # Errors
    ///
    /// Returns error if stream creation fails
    pub async fn open_bi(&self) -> Result<QuicStream> {
        let (send, recv) = self
            .connection
            .open_bi()
            .await
            .map_err(|e| QuicError::Stream(format!("Failed to open stream: {e}")))?;

        debug!("Opened bidirectional stream");

        Ok(QuicStream::new_bi(send, recv))
    }

    /// Open unidirectional stream (send only)
    ///
    /// # Errors
    ///
    /// Returns error if stream creation fails
    pub async fn open_uni(&self) -> Result<QuicStream> {
        let send = self
            .connection
            .open_uni()
            .await
            .map_err(|e| QuicError::Stream(format!("Failed to open stream: {e}")))?;

        debug!("Opened unidirectional stream");

        Ok(QuicStream::new_uni_send(send))
    }

    /// Accept bidirectional stream
    ///
    /// # Errors
    ///
    /// Returns error if connection closed
    pub async fn accept_bi(&self) -> Result<QuicStream> {
        let (send, recv) = self
            .connection
            .accept_bi()
            .await
            .map_err(|e| QuicError::Stream(format!("Failed to accept stream: {e}")))?;

        debug!("Accepted bidirectional stream");

        Ok(QuicStream::new_bi(send, recv))
    }

    /// Accept unidirectional stream (receive only)
    ///
    /// # Errors
    ///
    /// Returns error if connection closed
    pub async fn accept_uni(&self) -> Result<QuicStream> {
        let recv = self
            .connection
            .accept_uni()
            .await
            .map_err(|e| QuicError::Stream(format!("Failed to accept stream: {e}")))?;

        debug!("Accepted unidirectional stream");

        Ok(QuicStream::new_uni_recv(recv))
    }

    /// Get remote address
    #[must_use]
    pub fn remote_address(&self) -> SocketAddr {
        self.connection.remote_address()
    }

    /// Get local IP (may change due to migration)
    #[must_use]
    pub fn local_ip(&self) -> Option<std::net::IpAddr> {
        self.connection.local_ip()
    }

    /// Check if connection is closed
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.connection.close_reason().is_some()
    }

    /// Get connection statistics
    #[must_use]
    pub fn stats(&self) -> quinn::ConnectionStats {
        self.connection.stats()
    }

    /// Close connection gracefully
    pub fn close(&self, error_code: u32, reason: &[u8]) {
        self.connection.close(error_code.into(), reason);
    }

    /// Wait for connection to be fully closed
    ///
    /// # Errors
    ///
    /// Currently infallible; `Result` is returned for future extensibility.
    pub async fn closed(&self) -> Result<()> {
        self.connection.closed().await;
        Ok(())
    }
}

impl Drop for QuicConnection {
    fn drop(&mut self) {
        if !self.is_closed() {
            debug!("Connection dropped, closing gracefully");
            self.close(0, b"connection dropped");
        }
    }
}
