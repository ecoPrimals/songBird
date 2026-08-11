// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC connection handling (pure Rust, native engine).

use crate::config::QuicConfig;
use crate::error::{QuicError, Result};
use crate::stream::QuicStream;
use crate::transport::flow_control::ConnectionFlowControl;
use crate::transport::state::{CloseReason, Connection as TransportConnection};
use crate::transport::streams::StreamManager;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tracing::debug;

/// QUIC connection.
///
/// Represents an established QUIC connection with multiplexed streams,
/// backed by the native songbird-quic engine (no quinn).
pub struct QuicConnection {
    inner: Arc<Mutex<ConnectionInner>>,
    _config: Arc<QuicConfig>,
}

/// Shared mutable state for a QUIC connection (transport, streams, flow control).
pub struct ConnectionInner {
    pub(crate) transport: TransportConnection,
    pub(crate) streams: StreamManager,
    #[expect(dead_code, reason = "used by transport layer for flow control enforcement")]
    pub(crate) flow_control: ConnectionFlowControl,
}

impl std::fmt::Debug for QuicConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuicConnection").finish_non_exhaustive()
    }
}

impl QuicConnection {
    /// Create a new native connection.
    pub(crate) fn new(
        is_server: bool,
        remote_addr: SocketAddr,
        local_cid: Vec<u8>,
        remote_cid: Vec<u8>,
        config: Arc<QuicConfig>,
    ) -> Self {
        let transport = TransportConnection::new(is_server, remote_addr, local_cid, remote_cid);
        let streams = StreamManager::new(
            is_server,
            config.max_concurrent_bidi_streams,
            config.max_concurrent_uni_streams,
            262_144, // 256 KiB default max stream data
        );
        let flow_control = ConnectionFlowControl::new(1_048_576, 1_048_576);

        Self {
            inner: Arc::new(Mutex::new(ConnectionInner {
                transport,
                streams,
                flow_control,
            })),
            _config: config,
        }
    }

    /// Open a bidirectional stream.
    ///
    /// # Errors
    ///
    /// Returns error if the stream limit is exceeded or the connection is closed.
    #[expect(clippy::unused_async, reason = "async for API stability; future packet I/O")]
    pub async fn open_bi(&self) -> Result<QuicStream> {
        let mut inner = self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if !inner.transport.is_established() {
            return Err(QuicError::NotConnected);
        }
        let id = inner.streams.open_bidi()?;
        debug!("Opened bidirectional stream {id}");
        Ok(QuicStream::new(id, Arc::clone(&self.inner)))
    }

    /// Open a unidirectional stream (send only).
    ///
    /// # Errors
    ///
    /// Returns error if the stream limit is exceeded or the connection is closed.
    #[expect(clippy::unused_async, reason = "async for API stability; future packet I/O")]
    pub async fn open_uni(&self) -> Result<QuicStream> {
        let mut inner = self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if !inner.transport.is_established() {
            return Err(QuicError::NotConnected);
        }
        let id = inner.streams.open_uni()?;
        debug!("Opened unidirectional stream {id}");
        Ok(QuicStream::new(id, Arc::clone(&self.inner)))
    }

    /// Accept a bidirectional stream opened by the peer.
    ///
    /// # Errors
    ///
    /// Returns error if the connection is closed.
    pub fn accept_bi(&self) -> Result<QuicStream> {
        Err(QuicError::Stream("No pending bidirectional stream".into()))
    }

    /// Accept a unidirectional stream opened by the peer.
    ///
    /// # Errors
    ///
    /// Returns error if the connection is closed.
    pub fn accept_uni(&self) -> Result<QuicStream> {
        Err(QuicError::Stream("No pending unidirectional stream".into()))
    }

    /// Get remote address.
    #[expect(clippy::unused_async, reason = "async for API stability; future packet I/O")]
    pub async fn remote_address(&self) -> SocketAddr {
        let inner = self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.transport.remote_addr()
    }

    /// Check if connection is closed.
    #[expect(clippy::unused_async, reason = "async for API stability; future packet I/O")]
    pub async fn is_closed(&self) -> bool {
        let inner = self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.transport.is_closed()
    }

    /// Close connection gracefully.
    #[expect(clippy::unused_async, reason = "async for API stability; future packet I/O")]
    pub async fn close(&self, error_code: u64, reason: &[u8]) {
        let mut inner = self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let _ = inner.transport.initiate_close(CloseReason::Application {
            error_code,
            reason: reason.to_vec(),
        });
    }

    /// Wait for connection to be fully closed.
    ///
    /// # Errors
    ///
    /// Currently infallible; `Result` returned for API compatibility.
    #[expect(clippy::unused_async, reason = "async for API stability; future packet I/O")]
    pub async fn closed(&self) -> Result<()> {
        let mut inner = self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.transport.finish_close();
        Ok(())
    }

    /// Mark the connection as established (called after handshake completion).
    #[expect(clippy::unused_async, reason = "async for API stability; future packet I/O")]
    pub(crate) async fn set_established(&self) -> Result<()> {
        let mut inner = self.inner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        inner.transport.start_handshake()?;
        inner.transport.handshake_complete()?;
        Ok(())
    }
}

impl Drop for QuicConnection {
    fn drop(&mut self) {
        debug!("QUIC connection dropped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Arc<QuicConfig> {
        Arc::new(QuicConfig::new())
    }

    #[tokio::test]
    async fn new_connection_not_established() {
        let conn = QuicConnection::new(
            false,
            "127.0.0.1:4433".parse().unwrap(),
            vec![0x01, 0x02],
            vec![0x03, 0x04],
            test_config(),
        );
        assert!(!conn.is_closed().await);
        // Not established yet, so open_bi should fail
        assert!(conn.open_bi().await.is_err());
    }

    #[tokio::test]
    async fn established_connection_can_open_streams() {
        let conn = QuicConnection::new(
            false,
            "127.0.0.1:4433".parse().unwrap(),
            vec![0x01],
            vec![0x02],
            test_config(),
        );
        conn.set_established().await.unwrap();
        let stream = conn.open_bi().await.unwrap();
        assert_eq!(stream.id(), 0);
    }

    #[tokio::test]
    async fn close_and_closed() {
        let conn = QuicConnection::new(
            false,
            "127.0.0.1:4433".parse().unwrap(),
            vec![],
            vec![],
            test_config(),
        );
        conn.close(0, b"bye").await;
        conn.closed().await.unwrap();
        assert!(conn.is_closed().await);
    }

    #[tokio::test]
    async fn remote_address() {
        let addr: SocketAddr = "10.0.0.1:5000".parse().unwrap();
        let conn = QuicConnection::new(false, addr, vec![], vec![], test_config());
        assert_eq!(conn.remote_address().await, addr);
    }

    #[tokio::test]
    #[allow(clippy::unwrap_used, reason = "test assertion")]
    async fn accept_bi_and_accept_uni_return_errors() {
        let conn = QuicConnection::new(
            false,
            "127.0.0.1:1".parse().unwrap(),
            vec![],
            vec![],
            test_config(),
        );
        conn.set_established().await.unwrap();
        assert!(conn.accept_bi().is_err());
        assert!(conn.accept_uni().is_err());
    }

    #[tokio::test]
    #[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertion")]
    async fn open_bi_fails_when_stream_limit_reached() {
        let cfg = Arc::new(QuicConfig {
            max_concurrent_bidi_streams: 1,
            ..QuicConfig::new()
        });
        let conn = QuicConnection::new(
            false,
            "127.0.0.1:4433".parse().unwrap(),
            vec![0x01],
            vec![0x02],
            Arc::clone(&cfg),
        );
        conn.set_established().await.unwrap();
        conn.open_bi().await.unwrap();
        let err = conn.open_bi().await.expect_err("second bidi should exceed limit");
        assert!(
            err.to_string().contains("Max bidi") || err.to_string().contains("stream"),
            "unexpected: {err}"
        );
    }
}
