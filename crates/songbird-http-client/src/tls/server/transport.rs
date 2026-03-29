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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use crate::tls::TLS_1_2;
    use crate::tls::{content_type, handshake_type};

    /// Mirrors `receive_client_hello` header parsing (type, legacy version, length).
    fn parse_record_header(header: [u8; 5]) -> (u8, u16, usize) {
        let record_type = header[0];
        let tls_version = u16::from_be_bytes([header[1], header[2]]);
        let length = usize::from(u16::from_be_bytes([header[3], header[4]]));
        (record_type, tls_version, length)
    }

    #[test]
    fn client_hello_record_header_decodes_expected_fields() {
        let payload_len = 200usize;
        let mut header = [0u8; 5];
        header[0] = content_type::HANDSHAKE;
        header[1..3].copy_from_slice(&TLS_1_2.to_be_bytes());
        #[expect(
            clippy::cast_possible_truncation,
            reason = "TLS record payload is bounded to u16::MAX by the protocol"
        )]
        let payload_len_u16 = payload_len as u16;
        header[3..5].copy_from_slice(&payload_len_u16.to_be_bytes());

        let (rt, ver, len) = parse_record_header(header);
        assert_eq!(rt, content_type::HANDSHAKE);
        assert_eq!(ver, TLS_1_2);
        assert_eq!(len, payload_len);
    }

    #[test]
    fn application_data_record_type_constant() {
        assert_eq!(content_type::APPLICATION_DATA, 23);
    }

    #[test]
    fn client_hello_handshake_first_byte() {
        let mut payload = [0u8; 4];
        payload[0] = handshake_type::CLIENT_HELLO;
        assert_eq!(payload[0], 1);
    }

    #[test]
    fn tls_legacy_version_is_0303() {
        assert_eq!(TLS_1_2, 0x0303);
    }

    #[test]
    fn record_payload_length_zero() {
        let header = [content_type::HANDSHAKE, 0x03, 0x03, 0x00, 0x00];
        let (_, _, len) = parse_record_header(header);
        assert_eq!(len, 0);
    }

    #[test]
    fn record_payload_length_max_u16() {
        let header = [0x17, 0x03, 0x03, 0xff, 0xff];
        let (_, _, len) = parse_record_header(header);
        assert_eq!(len, 65535);
    }

    #[test]
    fn handshake_and_alert_content_types_differ() {
        assert_ne!(content_type::HANDSHAKE, content_type::ALERT);
        assert_ne!(content_type::ALERT, content_type::APPLICATION_DATA);
    }
}
