// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Tor link protocol handler
//!
//! Manages the connection lifecycle:
//! 1. TCP + TLS connect
//! 2. VERSIONS cell exchange
//! 3. NETINFO cell exchange
//! 4. Ready for circuit operations

use super::TlsConnector;
use crate::directory::RelayInfo;
use crate::error::{Error, Result};
use crate::protocol::{CELL_LEN, Cell, CellCommand};
use std::net::SocketAddr;
use std::time::SystemTime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

/// Returns true if the link-layer command byte uses variable-length cell framing.
///
/// Variable-length cell commands (per Tor spec): 7 (`VERSIONS`), 128 (`VPADDING`),
/// 129 (`CERTS`), 130 (`AUTH_CHALLENGE`), 131 (`AUTHENTICATE`).
#[must_use]
pub(super) const fn variable_length_cell_command(command: u8) -> bool {
    matches!(command, 7 | 128 | 129 | 130 | 131)
}

/// Link protocol versions we support
/// NOTE: v5 adds padding negotiation which we don't implement yet,
/// so we stick with v4 for better compatibility
const SUPPORTED_VERSIONS: &[u16] = &[4]; // v4 link protocol only

/// Tor connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// TCP connected
    TcpConnected,
    /// TLS handshake complete
    TlsConnected,
    /// VERSIONS exchanged
    VersionsNegotiated,
    /// NETINFO exchanged, ready for circuits
    Ready,
}

/// A connection to a Tor relay
pub struct TorConnection {
    /// Connection state
    state: ConnectionState,
    /// TCP stream (Tor protocol handles its own encryption via ntor)
    stream: Option<TcpStream>,
    /// Relay info
    relay: RelayInfo,
    /// Negotiated link version
    link_version: u16,
}

impl TorConnection {
    /// Create new connection (not yet connected)
    #[must_use]
    pub const fn new(relay: RelayInfo) -> Self {
        Self {
            state: ConnectionState::Disconnected,
            stream: None,
            relay,
            link_version: 0,
        }
    }

    /// Connect to the relay and complete the link protocol
    ///
    /// # Errors
    /// Returns error if connection, TLS, or protocol negotiation fails.
    pub async fn connect(&mut self) -> Result<()> {
        let addr: SocketAddr = match self.relay.address {
            std::net::IpAddr::V4(ip) => SocketAddr::new(ip.into(), self.relay.or_port),
            std::net::IpAddr::V6(ip) => SocketAddr::new(ip.into(), self.relay.or_port),
        };

        info!("Connecting to relay {} at {}", self.relay.nickname, addr);

        // 1. TLS connect
        let connector = TlsConnector::new();
        let stream = connector.connect(addr).await?;
        self.stream = Some(stream);
        self.state = ConnectionState::TlsConnected;
        debug!("TLS connected to {}", addr);

        // 2. VERSIONS exchange
        self.send_versions().await?;
        self.recv_versions().await?;
        self.state = ConnectionState::VersionsNegotiated;
        debug!("Version negotiated: v{}", self.link_version);

        // 3. NETINFO exchange
        self.recv_netinfo().await?;
        self.send_netinfo().await?;
        self.state = ConnectionState::Ready;
        info!("Connection ready to {}", self.relay.nickname);

        // Verify connection readiness by probing for relay-initiated data
        // (replaces hardcoded sleeps with event-driven readiness check)
        if let Some(stream) = self.stream.as_mut() {
            let mut peek_buf = [0u8; 1];
            match tokio::time::timeout(
                std::time::Duration::from_millis(500),
                stream.read(&mut peek_buf),
            )
            .await
            {
                Ok(Ok(0)) => {
                    warn!("Connection closed by relay after NETINFO!");
                    return Err(Error::Network(
                        "Relay closed connection after NETINFO".to_string(),
                    ));
                }
                Ok(Ok(_n)) => {
                    // Got some data - relay may be sending padding or certs
                    debug!("Received post-NETINFO data: {:02x}", peek_buf[0]);
                }
                Ok(Err(e)) => {
                    // Connection error
                    warn!("Connection error after NETINFO: {}", e);
                    return Err(Error::Network(format!("Connection error: {e}")));
                }
                Err(_) => {
                    // Timeout is expected - means connection is idle and ready
                    debug!("Connection ready for circuits (relay idle after NETINFO)");
                }
            }
        }

        Ok(())
    }

    /// Send VERSIONS cell
    async fn send_versions(&mut self) -> Result<()> {
        let stream =
            self.stream.as_mut().ok_or_else(|| Error::Network("Not connected".to_string()))?;

        // VERSIONS is a variable-length cell (not 512 bytes)
        // Format: CircID (2 bytes, 0 for VERSIONS) | Command (1 byte, 7) | Length (2 bytes) | Payload
        let mut buf = Vec::new();
        buf.extend_from_slice(&0u16.to_be_bytes()); // CircID = 0 (link-level)
        buf.push(CellCommand::Versions as u8); // Command = 7

        // Payload: List of 2-byte version numbers
        let versions_len = SUPPORTED_VERSIONS.len() * 2;
        buf.extend_from_slice(
            &u16::try_from(versions_len)
                .map_err(|_| Error::Network("VERSIONS payload too long".to_string()))?
                .to_be_bytes(),
        );

        for &version in SUPPORTED_VERSIONS {
            buf.extend_from_slice(&version.to_be_bytes());
        }

        stream
            .write_all(&buf)
            .await
            .map_err(|e| Error::Network(format!("Failed to send VERSIONS: {e}")))?;
        stream
            .flush()
            .await
            .map_err(|e| Error::Network(format!("Failed to flush VERSIONS: {e}")))?;

        Ok(())
    }

    /// Receive and parse VERSIONS cell
    async fn recv_versions(&mut self) -> Result<()> {
        let stream =
            self.stream.as_mut().ok_or_else(|| Error::Network("Not connected".to_string()))?;

        // Read variable-length VERSIONS cell header
        let mut header = [0u8; 5];
        stream
            .read_exact(&mut header)
            .await
            .map_err(|e| Error::Network(format!("Failed to read VERSIONS header: {e}")))?;

        let circ_id = u16::from_be_bytes([header[0], header[1]]);
        let command = header[2];
        let length = u16::from_be_bytes([header[3], header[4]]) as usize;

        if circ_id != 0 {
            return Err(Error::Protocol(format!("VERSIONS has non-zero CircID: {circ_id}")));
        }
        if command != CellCommand::Versions as u8 {
            return Err(Error::Protocol(format!("Expected VERSIONS (7), got command {command}")));
        }

        // Read payload
        let mut payload = vec![0u8; length];
        stream
            .read_exact(&mut payload)
            .await
            .map_err(|e| Error::Network(format!("Failed to read VERSIONS payload: {e}")))?;

        // Parse versions
        let server_versions: Vec<u16> = payload
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some(u16::from_be_bytes([chunk[0], chunk[1]]))
                } else {
                    None
                }
            })
            .collect();

        debug!("Server supports versions: {:?}", server_versions);

        // Find highest common version
        self.link_version =
            *SUPPORTED_VERSIONS.iter().filter(|v| server_versions.contains(v)).max().ok_or_else(
                || {
                    Error::Protocol(format!(
                        "No common link protocol version. Server: {server_versions:?}, Client: {SUPPORTED_VERSIONS:?}"
                    ))
                },
            )?;

        Ok(())
    }

    /// Receive NETINFO cell (and possibly other cells)
    async fn recv_netinfo(&mut self) -> Result<()> {
        let stream =
            self.stream.as_mut().ok_or_else(|| Error::Network("Not connected".to_string()))?;

        // After VERSIONS, server sends: CERTS, AUTH_CHALLENGE, then NETINFO
        // CERTS and AUTH_CHALLENGE are variable-length cells
        // NETINFO is a fixed-length cell
        loop {
            // First, read the cell header to determine if it's fixed or variable length
            // Header: CircID (4 bytes for v4+) | Command (1 byte)
            let mut header = [0u8; 5];
            stream
                .read_exact(&mut header)
                .await
                .map_err(|e| Error::Network(format!("Failed to read cell header: {e}")))?;

            let circ_id = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
            let command = header[4];

            if variable_length_cell_command(command) {
                // Variable-length cell: read 2-byte length, then payload
                let mut len_buf = [0u8; 2];
                stream
                    .read_exact(&mut len_buf)
                    .await
                    .map_err(|e| Error::Network(format!("Failed to read var cell length: {e}")))?;
                let payload_len = u16::from_be_bytes(len_buf) as usize;

                // Read and discard the payload (we're not processing these for now)
                let mut payload = vec![0u8; payload_len];
                stream
                    .read_exact(&mut payload)
                    .await
                    .map_err(|e| Error::Network(format!("Failed to read var cell payload: {e}")))?;

                match command {
                    7 => debug!("Received VERSIONS cell ({} bytes)", payload_len),
                    129 => debug!("Received CERTS cell ({} bytes)", payload_len),
                    130 => debug!("Received AUTH_CHALLENGE cell ({} bytes)", payload_len),
                    _ => debug!("Received var-length cell cmd={} ({} bytes)", command, payload_len),
                }
            } else {
                // Fixed-length cell: read remaining 507 bytes of payload
                let mut payload = [0u8; CELL_LEN - 5]; // 512 - 5 = 507
                stream
                    .read_exact(&mut payload)
                    .await
                    .map_err(|e| Error::Network(format!("Failed to read cell payload: {e}")))?;

                match command {
                    8 => {
                        // NETINFO - done receiving
                        debug!("Received NETINFO (circ_id={})", circ_id);
                        return Ok(());
                    }
                    _ => {
                        warn!("Received unknown fixed cell command {} during handshake", command);
                    }
                }
            }
        }
    }

    /// Send NETINFO cell
    async fn send_netinfo(&mut self) -> Result<()> {
        let stream =
            self.stream.as_mut().ok_or_else(|| Error::Network("Not connected".to_string()))?;

        // NETINFO cell format (link v4+):
        // CircID (4 bytes) | Command (1 byte, 8) | Payload (507 bytes, padded)
        //
        // Payload:
        // - Timestamp (4 bytes, Unix time)
        // - Other address (1 byte type + 1 byte len + N bytes addr)
        // - Num my addresses (1 byte)
        // - My addresses (same format as other address)

        let mut buf = [0u8; CELL_LEN];

        // CircID = 0 (link-level)
        buf[0..4].copy_from_slice(&0u32.to_be_bytes());
        // Command = NETINFO (8)
        buf[4] = CellCommand::NetInfo as u8;

        // Timestamp
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()
            .and_then(|d| u32::try_from(d.as_secs()).ok())
            .unwrap_or(0);
        buf[5..9].copy_from_slice(&timestamp.to_be_bytes());

        // Other address (relay's address as we see it)
        match self.relay.address {
            std::net::IpAddr::V4(ip) => {
                buf[9] = 4; // Type: IPv4
                buf[10] = 4; // Length: 4 bytes
                buf[11..15].copy_from_slice(&ip.octets());
                // Num my addresses
                buf[15] = 0; // We don't announce our addresses
            }
            std::net::IpAddr::V6(ip) => {
                buf[9] = 6; // Type: IPv6
                buf[10] = 16; // Length: 16 bytes
                buf[11..27].copy_from_slice(&ip.octets());
                // Num my addresses
                buf[27] = 0;
            }
        }

        // Log NETINFO details
        info!("NETINFO cell details:");
        info!("  Timestamp: {}", timestamp);
        info!("  Other addr type: {} (4=IPv4, 6=IPv6)", buf[9]);
        info!("  Other addr len: {}", buf[10]);
        info!("  Other addr: {}.{}.{}.{}", buf[11], buf[12], buf[13], buf[14]);
        info!("  Num my addrs: {}", buf[15]);
        info!("  Full cell hex [0..20]: {:02x?}", &buf[0..20]);

        stream
            .write_all(&buf)
            .await
            .map_err(|e| Error::Network(format!("Failed to send NETINFO: {e}")))?;
        stream
            .flush()
            .await
            .map_err(|e| Error::Network(format!("Failed to flush NETINFO: {e}")))?;
        debug!("NETINFO cell sent and flushed");

        Ok(())
    }

    /// Send a cell
    ///
    /// # Errors
    /// Returns error if not connected or I/O fails.
    pub async fn send_cell(&mut self, cell: &Cell) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        let stream =
            self.stream.as_mut().ok_or_else(|| Error::Network("Not connected".to_string()))?;

        let buf = cell.encode();
        debug!(
            "Sending cell: circ_id={} (0x{:08x}), command={:?}",
            cell.circ_id, cell.circ_id, cell.command
        );
        debug!("Cell bytes [0..20]: {:02x?}", &buf[0..20]);

        // For CREATE2, log more details
        if cell.command == CellCommand::Create2 {
            info!("CREATE2 cell details:");
            info!("  Full cell hex (first 100 bytes): {:02x?}", &buf[0..100]);
            info!("  Payload length in struct: {}", cell.payload.len());
            if cell.payload.len() >= 88 {
                let htype = u16::from_be_bytes([cell.payload[0], cell.payload[1]]);
                let hlen = u16::from_be_bytes([cell.payload[2], cell.payload[3]]);
                info!("  HTYPE: {} (should be 2)", htype);
                info!("  HLEN: {} (should be 84)", hlen);
                info!("  node_id (ID, 20 bytes): {:02x?}", &cell.payload[4..24]);
                info!("  ntor_key (B, 32 bytes): {:02x?}", &cell.payload[24..56]);
                info!("  client_pk (X, 32 bytes): {:02x?}", &cell.payload[56..88]);
            }
        }

        stream
            .write_all(&buf)
            .await
            .map_err(|e| Error::Network(format!("Failed to send cell: {e}")))?;
        stream.flush().await.map_err(|e| Error::Network(format!("Failed to flush stream: {e}")))?;
        debug!("Cell sent and flushed ({} bytes)", buf.len());

        Ok(())
    }

    /// Receive a cell with timeout, skipping PADDING cells
    ///
    /// # Errors
    /// Returns error if connection closed, timeout, or parse fails.
    pub async fn recv_cell(&mut self) -> Result<Cell> {
        use std::time::Duration;
        use tokio::io::AsyncReadExt;
        use tokio::time::timeout;

        let stream =
            self.stream.as_mut().ok_or_else(|| Error::Network("Not connected".to_string()))?;

        loop {
            let mut buf = [0u8; CELL_LEN];
            debug!("Waiting to read {} byte cell (30s timeout)...", CELL_LEN);

            // First try to read just 5 bytes to see if there's any data at all
            let mut header = [0u8; 5];
            match timeout(Duration::from_secs(30), stream.read(&mut header)).await {
                Ok(Ok(0)) => {
                    warn!("Connection closed by remote (read 0 bytes)");
                    return Err(Error::Network("Connection closed by relay".to_string()));
                }
                Ok(Ok(n)) if n < 5 => {
                    warn!("Partial header read: {} bytes: {:02x?}", n, &header[..n]);
                    // Try to read the rest
                    match timeout(Duration::from_secs(5), stream.read_exact(&mut header[n..5]))
                        .await
                    {
                        Ok(Ok(_)) => {}
                        Ok(Err(e)) => {
                            return Err(Error::Network(format!(
                                "Failed to read rest of header: {e}"
                            )));
                        }
                        Err(_) => return Err(Error::Network("Header read timed out".to_string())),
                    }
                }
                Ok(Ok(n)) => {
                    debug!("Read {} bytes for header: {:02x?}", n, &header[..5]);
                }
                Ok(Err(e)) => {
                    return Err(Error::Network(format!("Failed to read cell header: {e}")));
                }
                Err(_) => {
                    warn!("Cell read timed out after 30s - no response from relay");
                    return Err(Error::Network(
                        "Cell read timed out - relay did not respond".to_string(),
                    ));
                }
            }

            // Copy header to buf and read rest
            buf[..5].copy_from_slice(&header);
            match timeout(Duration::from_secs(10), stream.read_exact(&mut buf[5..])).await {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => {
                    warn!("Got header {:02x?} but failed to read payload: {}", &buf[..5], e);
                    return Err(Error::Network(format!("Failed to read cell payload: {e}")));
                }
                Err(_) => {
                    warn!("Got header {:02x?} but payload read timed out", &buf[..5]);
                    return Err(Error::Network("Cell payload read timed out".to_string()));
                }
            }

            let cell = Cell::decode(&buf)?;
            debug!(
                "Received cell: circ_id={} (0x{:08x}), command={:?}",
                cell.circ_id, cell.circ_id, cell.command
            );

            // Skip PADDING cells
            if cell.command == CellCommand::Padding {
                debug!("Skipping PADDING cell");
                continue;
            }

            return Ok(cell);
        }
    }

    /// Get connection state
    #[must_use]
    pub const fn state(&self) -> ConnectionState {
        self.state
    }

    /// Check if connection is ready for circuits
    pub fn is_ready(&self) -> bool {
        self.state == ConnectionState::Ready
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::directory::RelayFlags;
    use std::net::IpAddr;

    fn test_relay() -> RelayInfo {
        RelayInfo {
            nickname: "TestRelay".to_string(),
            fingerprint: [0u8; 20],
            address: IpAddr::from([127, 0, 0, 1]),
            or_port: 9001,
            dir_port: None,
            flags: RelayFlags::empty(),
            bandwidth: 1_000_000,
            ntor_key: None,
            version: None,
        }
    }

    #[test]
    fn test_connection_creation() {
        let relay = test_relay();
        let conn = TorConnection::new(relay);
        assert_eq!(conn.state(), ConnectionState::Disconnected);
        assert!(!conn.is_ready());
    }

    #[test]
    fn connection_ipv6_relay_starts_disconnected() {
        let mut relay = test_relay();
        relay.address = IpAddr::from([0x20, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
        let conn = TorConnection::new(relay);
        assert_eq!(conn.state(), ConnectionState::Disconnected);
        assert!(!conn.is_ready());
    }

    #[test]
    fn variable_length_commands_match_fixed_netinfo() {
        assert!(variable_length_cell_command(CellCommand::Versions as u8));
        assert!(!variable_length_cell_command(CellCommand::NetInfo as u8));
        assert!(!variable_length_cell_command(CellCommand::Relay as u8));
    }

    #[test]
    fn variable_length_includes_auth_handshake_cells() {
        for cmd in [128u8, 129, 130, 131] {
            assert!(variable_length_cell_command(cmd), "cmd {cmd} should use var-length framing");
        }
    }

    #[test]
    fn connection_state_equality_for_teardown_modeling() {
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);
        assert_ne!(ConnectionState::Disconnected, ConnectionState::Ready);
        assert_ne!(ConnectionState::TlsConnected, ConnectionState::VersionsNegotiated);
    }
}
