// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC stream handling

use crate::error::{QuicError, Result};
use quinn::{RecvStream, SendStream};
use tracing::debug;

/// QUIC stream
///
/// Multiplexed stream within a QUIC connection
pub struct QuicStream {
    /// Send half (if bidirectional or send-only)
    send: Option<SendStream>,

    /// Receive half (if bidirectional or receive-only)
    recv: Option<RecvStream>,
}

impl QuicStream {
    /// Create bidirectional stream
    pub(crate) const fn new_bi(send: SendStream, recv: RecvStream) -> Self {
        Self {
            send: Some(send),
            recv: Some(recv),
        }
    }

    /// Create send-only stream
    pub(crate) const fn new_uni_send(send: SendStream) -> Self {
        Self {
            send: Some(send),
            recv: None,
        }
    }

    /// Create receive-only stream
    pub(crate) const fn new_uni_recv(recv: RecvStream) -> Self {
        Self {
            send: None,
            recv: Some(recv),
        }
    }

    /// Write data to stream
    ///
    /// # Errors
    ///
    /// Returns error if stream is not writable or write fails
    pub async fn write(&mut self, data: &[u8]) -> Result<()> {
        let send = self
            .send
            .as_mut()
            .ok_or_else(|| QuicError::Stream("Stream not writable".to_string()))?;

        send.write_all(data).await.map_err(|e| QuicError::Stream(format!("Write failed: {e}")))?;

        Ok(())
    }

    /// Write all data and finish stream
    ///
    /// # Errors
    ///
    /// Returns error if stream is not writable or write fails
    pub async fn write_all_and_finish(&mut self, data: &[u8]) -> Result<()> {
        self.write(data).await?;
        self.finish()?;
        Ok(())
    }

    /// Finish sending (close write side)
    ///
    /// # Errors
    ///
    /// Returns error if stream is not writable
    pub fn finish(&mut self) -> Result<()> {
        let send = self
            .send
            .as_mut()
            .ok_or_else(|| QuicError::Stream("Stream not writable".to_string()))?;

        send.finish().map_err(|e| QuicError::Stream(format!("Finish failed: {e}")))?;

        debug!("Stream finished");
        Ok(())
    }

    /// Read data from stream
    ///
    /// # Errors
    ///
    /// Returns error if stream is not readable or read fails
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let recv = self
            .recv
            .as_mut()
            .ok_or_else(|| QuicError::Stream("Stream not readable".to_string()))?;

        let n = recv
            .read(buf)
            .await
            .map_err(|e| QuicError::Stream(format!("Read failed: {e}")))?
            .ok_or_else(|| QuicError::Stream("Stream closed by peer".to_string()))?;

        Ok(n)
    }

    /// Read exact amount of data
    ///
    /// # Errors
    ///
    /// Returns error if stream is not readable, EOF, or read fails
    pub async fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let recv = self
            .recv
            .as_mut()
            .ok_or_else(|| QuicError::Stream("Stream not readable".to_string()))?;

        recv.read_exact(buf)
            .await
            .map_err(|e| QuicError::Stream(format!("Read exact failed: {e}")))?;

        Ok(())
    }

    /// Read all remaining data until stream closed
    ///
    /// # Errors
    ///
    /// Returns error if stream is not readable or read fails
    pub async fn read_to_end(&mut self, max_size: usize) -> Result<Vec<u8>> {
        let recv = self
            .recv
            .as_mut()
            .ok_or_else(|| QuicError::Stream("Stream not readable".to_string()))?;

        let mut buf = Vec::new();

        loop {
            let mut chunk = [0u8; 4096];
            match recv.read(&mut chunk).await {
                Ok(Some(n)) => {
                    buf.extend_from_slice(&chunk[..n]);
                    if buf.len() > max_size {
                        return Err(QuicError::Stream(format!(
                            "Data exceeds max size: {max_size}"
                        )));
                    }
                }
                Ok(None) => break,
                Err(e) => {
                    return Err(QuicError::Stream(format!("Read failed: {e}")));
                }
            }
        }

        Ok(buf)
    }

    /// Check if stream is writable
    #[must_use]
    pub const fn is_writable(&self) -> bool {
        self.send.is_some()
    }

    /// Check if stream is readable
    #[must_use]
    pub const fn is_readable(&self) -> bool {
        self.recv.is_some()
    }
}

#[cfg(all(test, feature = "ring-crypto"))]
mod tests {

    use crate::client::QuicClient;
    use crate::config::QuicConfig;
    use crate::server::QuicServer;

    #[tokio::test]
    async fn bi_stream_write_read_roundtrip() {
        let config = QuicConfig::new();
        let server = QuicServer::new("127.0.0.1:0", config.clone()).await.unwrap();
        let addr = server.local_addr();
        let mut incoming = server.accept();

        let client_task = tokio::spawn(async move {
            let client = QuicClient::new(config).await.unwrap();
            let conn = client.connect(&addr.to_string()).await.unwrap();
            let mut stream = conn.open_bi().await.unwrap();
            stream.write_all_and_finish(b"hello-quic").await.unwrap();
            conn
        });

        let server_conn = incoming.recv().await.expect("server accept");
        let mut server_stream = server_conn.accept_bi().await.unwrap();

        let mut buf = [0u8; 16];
        let n = server_stream.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"hello-quic");

        let client_conn = client_task.await.expect("client join");
        client_conn.close(0, b"done");
        server.close().await;
    }

    #[tokio::test]
    async fn recv_only_stream_not_writable() {
        let config = QuicConfig::new();
        let server = QuicServer::new("127.0.0.1:0", config.clone()).await.unwrap();
        let addr = server.local_addr();
        let mut incoming = server.accept();

        let client_task = tokio::spawn(async move {
            let client = QuicClient::new(config).await.unwrap();
            let conn = client.connect(&addr.to_string()).await.unwrap();
            let _send = conn.open_uni().await.unwrap();
            conn
        });

        let server_conn = incoming.recv().await.expect("server accept");
        let recv_only = server_conn.accept_uni().await.unwrap();

        assert!(recv_only.is_readable());
        assert!(!recv_only.is_writable());
        let mut err_stream = recv_only;
        assert!(err_stream.write(b"x").await.is_err());

        let client_conn = client_task.await.expect("client join");
        client_conn.close(0, b"done");
        server.close().await;
    }

    #[tokio::test]
    async fn read_to_end_respects_max_size() {
        let config = QuicConfig::new();
        let server = QuicServer::new("127.0.0.1:0", config.clone()).await.unwrap();
        let addr = server.local_addr();
        let mut incoming = server.accept();

        let client_task = tokio::spawn(async move {
            let client = QuicClient::new(config).await.unwrap();
            let conn = client.connect(&addr.to_string()).await.unwrap();
            let mut stream = conn.open_bi().await.unwrap();
            stream.write(&[0u8; 128]).await.unwrap();
            stream.finish().unwrap();
            conn
        });

        let server_conn = incoming.recv().await.expect("server accept");
        let mut stream = server_conn.accept_bi().await.unwrap();

        let err = stream.read_to_end(16).await;
        assert!(err.is_err());

        let client_conn = client_task.await.expect("client join");
        client_conn.close(0, b"done");
        server.close().await;
    }
}
