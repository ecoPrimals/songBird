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
    pub(crate) fn new_bi(send: SendStream, recv: RecvStream) -> Self {
        Self {
            send: Some(send),
            recv: Some(recv),
        }
    }

    /// Create send-only stream
    pub(crate) fn new_uni_send(send: SendStream) -> Self {
        Self {
            send: Some(send),
            recv: None,
        }
    }

    /// Create receive-only stream
    pub(crate) fn new_uni_recv(recv: RecvStream) -> Self {
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
    pub fn is_writable(&self) -> bool {
        self.send.is_some()
    }

    /// Check if stream is readable
    #[must_use]
    pub fn is_readable(&self) -> bool {
        self.recv.is_some()
    }
}
