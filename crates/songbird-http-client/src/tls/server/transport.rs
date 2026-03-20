// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! TLS Record Layer Transport
//!
//! Handles low-level TLS record I/O operations.

use crate::error::{Error, Result};
use crate::tls::{content_type, handshake_type};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info};

use super::core::TlsServer;

impl TlsServer {
    /// Receive `ClientHello` from client
    pub(super) async fn receive_client_hello(&mut self, stream: &mut TcpStream) -> Result<Vec<u8>> {
        // Read TLS record (5-byte header + payload)
        let mut header = [0u8; 5];
        stream.read_exact(&mut header).await.map_err(Error::Io)?;

        let record_type = header[0];
        let tls_version = u16::from_be_bytes([header[1], header[2]]);
        let length = u16::from_be_bytes([header[3], header[4]]) as usize;

        debug!(
            "📥 TLS record: type=0x{:02x}, version=0x{:04x}, length={}",
            record_type, tls_version, length
        );

        if record_type != content_type::HANDSHAKE {
            return Err(Error::TlsHandshake(format!(
                "Expected Handshake record, got 0x{record_type:02x}"
            )));
        }

        // Read payload
        let mut payload = vec![0u8; length];
        stream.read_exact(&mut payload).await.map_err(Error::Io)?;

        // Verify it's a ClientHello
        if payload.is_empty() || payload[0] != handshake_type::CLIENT_HELLO {
            return Err(Error::TlsHandshake(format!(
                "Expected ClientHello (0x01), got 0x{:02x}",
                payload.first().unwrap_or(&0)
            )));
        }

        info!("✅ ClientHello received: {} bytes", payload.len());

        // Add to transcript (SAME as client!)
        self.transcript_mut().update_with_logging(
            &payload,
            "ClientHello (server receiving)",
            false,
        );

        Ok(payload)
    }

    /// Receive TLS record
    pub(super) async fn receive_tls_record(&self, stream: &mut TcpStream) -> Result<Vec<u8>> {
        // Read header
        let mut header = [0u8; 5];
        stream.read_exact(&mut header).await.map_err(Error::Io)?;

        let length = u16::from_be_bytes([header[3], header[4]]) as usize;

        // Read payload
        let mut payload = vec![0u8; length];
        stream.read_exact(&mut payload).await.map_err(Error::Io)?;

        Ok(payload)
    }

    /// Send `ServerHello` (wrap in TLS record and send)
    pub(super) async fn send_server_hello(
        &mut self,
        stream: &mut TcpStream,
        server_hello: &[u8],
    ) -> Result<()> {
        // Add ServerHello to transcript BEFORE sending (SAME as client!)
        self.transcript_mut().update_with_logging(
            server_hello,
            "ServerHello (server sending)",
            false,
        );

        // Send ServerHello (wrap in TLS record)
        let server_hello_record = self.wrap_in_tls_record(content_type::HANDSHAKE, server_hello);
        stream.write_all(&server_hello_record).await.map_err(Error::Io)?;
        info!("✅ ServerHello sent: {} bytes", server_hello_record.len());

        Ok(())
    }
}
