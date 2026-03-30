// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! QUIC stream handling (pure Rust, native engine).

use crate::error::{QuicError, Result};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

// Re-use the ConnectionInner from connection.rs.
// This type alias avoids exposing internal details.
type ConnectionInner = super::connection::ConnectionInner;

/// QUIC stream.
///
/// Multiplexed stream within a QUIC connection.
pub struct QuicStream {
    /// Stream ID.
    stream_id: u64,
    /// Shared connection state.
    conn: Arc<Mutex<ConnectionInner>>,
}

impl std::fmt::Debug for QuicStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QuicStream")
            .field("id", &self.stream_id)
            .finish()
    }
}

impl QuicStream {
    /// Create a new stream handle.
    pub(crate) fn new(stream_id: u64, conn: Arc<Mutex<ConnectionInner>>) -> Self {
        Self { stream_id, conn }
    }

    /// Stream ID.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.stream_id
    }

    /// Write data to the stream.
    ///
    /// # Errors
    ///
    /// Returns error if the stream is not writable or the connection is closed.
    pub async fn write(&mut self, data: &[u8]) -> Result<()> {
        let mut inner = self.conn.lock().await;
        let entry = inner.streams.get_mut(self.stream_id)
            .ok_or_else(|| QuicError::Stream("Stream not found".into()))?;
        entry.write(data)?;
        Ok(())
    }

    /// Write all data and finish the stream.
    ///
    /// # Errors
    ///
    /// Returns error if the stream is not writable.
    pub async fn write_all_and_finish(&mut self, data: &[u8]) -> Result<()> {
        self.write(data).await?;
        self.finish().await?;
        Ok(())
    }

    /// Finish sending (close write side).
    ///
    /// # Errors
    ///
    /// Returns error if the stream is not writable.
    pub async fn finish(&mut self) -> Result<()> {
        let mut inner = self.conn.lock().await;
        let entry = inner.streams.get_mut(self.stream_id)
            .ok_or_else(|| QuicError::Stream("Stream not found".into()))?;
        entry.finish_send();
        debug!("Stream {} finished", self.stream_id);
        Ok(())
    }

    /// Read data from the stream.
    ///
    /// Returns the number of bytes read.
    ///
    /// # Errors
    ///
    /// Returns error if the stream is not readable.
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut inner = self.conn.lock().await;
        let entry = inner.streams.get_mut(self.stream_id)
            .ok_or_else(|| QuicError::Stream("Stream not found".into()))?;
        let n = entry.read(buf);
        if n == 0 {
            return Err(QuicError::Stream("No data available".into()));
        }
        Ok(n)
    }

    /// Read all remaining data until stream is finished.
    ///
    /// # Errors
    ///
    /// Returns error if the stream is not readable or data exceeds max_size.
    pub async fn read_to_end(&mut self, max_size: usize) -> Result<Vec<u8>> {
        let mut inner = self.conn.lock().await;
        let entry = inner.streams.get_mut(self.stream_id)
            .ok_or_else(|| QuicError::Stream("Stream not found".into()))?;

        let mut buf = Vec::new();
        let mut tmp = [0u8; 4096];
        loop {
            let n = entry.read(&mut tmp);
            if n == 0 {
                break;
            }
            buf.extend_from_slice(&tmp[..n]);
            if buf.len() > max_size {
                return Err(QuicError::Stream(format!(
                    "Data exceeds max size: {max_size}"
                )));
            }
        }
        Ok(buf)
    }

    /// Check if stream has data to read.
    pub async fn is_readable(&self) -> bool {
        let inner = self.conn.lock().await;
        inner.streams.get(self.stream_id)
            .is_some_and(super::transport::streams::StreamEntry::has_readable_data)
    }

    /// Check if stream is writable.
    pub async fn is_writable(&self) -> bool {
        use crate::transport::streams::StreamState;
        let inner = self.conn.lock().await;
        inner.streams.get(self.stream_id)
            .is_some_and(|e| matches!(e.state, StreamState::Open | StreamState::RecvFinished))
    }
}

#[cfg(test)]
mod tests {
    use super::super::connection::QuicConnection;
    use super::super::config::QuicConfig;
    use std::sync::Arc;

    #[tokio::test]
    async fn write_and_read_via_stream() {
        let config = Arc::new(QuicConfig::new());
        let conn = QuicConnection::new(
            false,
            "127.0.0.1:4433".parse().unwrap(),
            vec![0x01],
            vec![0x02],
            config,
        );
        conn.set_established().await.unwrap();

        let mut stream = conn.open_bi().await.unwrap();
        assert_eq!(stream.id(), 0);

        // Write data
        stream.write(b"hello").await.unwrap();

        // Simulate data arriving on the stream (in a real impl, this comes from packets)
        {
            let mut inner = stream.conn.lock().await;
            let entry = inner.streams.get_mut(stream.id()).unwrap();
            entry.receive(b"world").unwrap();
        }

        // Read it back
        let mut buf = [0u8; 10];
        let n = stream.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"world");
    }

    #[tokio::test]
    async fn finish_closes_write_side() {
        let config = Arc::new(QuicConfig::new());
        let conn = QuicConnection::new(
            false,
            "127.0.0.1:4433".parse().unwrap(),
            vec![],
            vec![],
            config,
        );
        conn.set_established().await.unwrap();

        let mut stream = conn.open_bi().await.unwrap();
        stream.finish().await.unwrap();
        assert!(stream.write(b"more").await.is_err());
    }
}
