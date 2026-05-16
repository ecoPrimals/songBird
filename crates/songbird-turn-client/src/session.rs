// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! High-level TURN session with control + data plane.

use crate::error::TurnSessionError;
use bytes::{BufMut, BytesMut};
use songbird_stun::message::{MessageType, StunAttribute, StunMessage};
use songbird_stun::{StunCredentials, TurnAllocation, TurnClient, encode_xor_peer_address};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Default channel number for `ChannelBind` (first valid: 0x4000).
const DEFAULT_CHANNEL: u16 = 0x4000;

/// Maximum payload size for a single TURN data relay.
const MAX_PAYLOAD: usize = 65535;

/// Ephemeral bind address for IPv4 UDP sockets.
const EPHEMERAL_BIND: ([u8; 4], u16) = ([0, 0, 0, 0], 0);

/// Configuration for establishing a TURN session.
#[derive(Debug, Clone)]
pub struct TurnSessionConfig {
    /// TURN server address (e.g. `turn.example.com:3478`).
    pub server_addr: SocketAddr,
    /// Long-term credentials for TURN authentication.
    pub credentials: StunCredentials,
    /// Peer address to relay data to/from.
    pub peer_addr: SocketAddr,
    /// Whether to use `ChannelData` framing (preferred) vs `SendIndication`.
    pub use_channel: bool,
    /// Channel number (0x4000–0x7FFF) when `use_channel` is true.
    pub channel: u16,
    /// Local bind address for the UDP socket (`0.0.0.0:0` for ephemeral).
    pub local_bind: SocketAddr,
    /// Request timeout for control-plane operations.
    pub control_timeout: Duration,
    /// Receive timeout for data-plane operations.
    pub recv_timeout: Duration,
}

impl TurnSessionConfig {
    /// Create a new config with sensible defaults.
    ///
    /// Defaults: `ChannelData` framing enabled, channel 0x4000, ephemeral bind,
    /// 5s control timeout, 30s receive timeout.
    #[must_use]
    pub fn new(
        server_addr: SocketAddr,
        credentials: StunCredentials,
        peer_addr: SocketAddr,
    ) -> Self {
        Self {
            server_addr,
            credentials,
            peer_addr,
            use_channel: true,
            channel: DEFAULT_CHANNEL,
            local_bind: SocketAddr::from(EPHEMERAL_BIND),
            control_timeout: Duration::from_secs(5),
            recv_timeout: Duration::from_secs(30),
        }
    }

    /// Override the local bind address.
    #[must_use]
    pub const fn with_local_bind(mut self, addr: SocketAddr) -> Self {
        self.local_bind = addr;
        self
    }

    /// Disable `ChannelData` framing; use Send/Data Indication instead.
    #[must_use]
    pub const fn without_channel(mut self) -> Self {
        self.use_channel = false;
        self
    }

    /// Override the receive timeout for data-plane operations.
    #[must_use]
    pub const fn with_recv_timeout(mut self, timeout: Duration) -> Self {
        self.recv_timeout = timeout;
        self
    }
}

/// A TURN relay session providing data-plane send/receive.
///
/// Wraps the full lifecycle: Allocate → `CreatePermission` → `ChannelBind` →
/// send/recv data through the relay.
///
/// The session is `Send + Sync` and can be shared across tasks via `Arc`.
pub struct TurnSession {
    client: TurnClient,
    socket: Arc<UdpSocket>,
    allocation: TurnAllocation,
    peer_addr: SocketAddr,
    channel: Option<u16>,
    recv_timeout: Duration,
    recv_buf: Mutex<Vec<u8>>,
}

impl std::fmt::Debug for TurnSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TurnSession")
            .field("relay_addr", &self.allocation.relay_addr)
            .field("peer_addr", &self.peer_addr)
            .field("channel", &self.channel)
            .finish_non_exhaustive()
    }
}

impl TurnSession {
    /// Establish a TURN session: allocate relay, create permission, optionally bind channel.
    ///
    /// # Errors
    ///
    /// Returns an error if allocation, permission, or channel binding fails.
    pub async fn connect(config: TurnSessionConfig) -> Result<Self, TurnSessionError> {
        let client = TurnClient::new(config.server_addr, config.credentials)
            .with_timeout(config.control_timeout);

        let socket = UdpSocket::bind(config.local_bind).await?;
        let socket = Arc::new(socket);

        info!("TURN session: allocating relay on {}", config.server_addr);
        let allocation = client.allocate(&socket).await?;
        info!(
            "TURN session: relay={}, lifetime={}s",
            allocation.relay_addr, allocation.lifetime_secs
        );

        client.create_permission(&socket, config.peer_addr).await?;
        debug!("TURN session: permission granted for {}", config.peer_addr);

        let channel = if config.use_channel {
            client.channel_bind(&socket, config.channel, config.peer_addr).await?;
            debug!("TURN session: channel 0x{:04x} bound", config.channel);
            Some(config.channel)
        } else {
            None
        };

        Ok(Self {
            client,
            socket,
            allocation,
            peer_addr: config.peer_addr,
            channel,
            recv_timeout: config.recv_timeout,
            recv_buf: Mutex::new(vec![0u8; 4096]),
        })
    }

    /// The relay address allocated by the TURN server.
    ///
    /// Peers should send data to this address; the TURN server relays it back.
    #[must_use]
    pub const fn relay_addr(&self) -> SocketAddr {
        self.allocation.relay_addr
    }

    /// The allocation details (relay addr, mapped addr, lifetime).
    #[must_use]
    pub const fn allocation(&self) -> &TurnAllocation {
        &self.allocation
    }

    /// The underlying UDP socket (for advanced use cases).
    #[must_use]
    pub const fn socket(&self) -> &Arc<UdpSocket> {
        &self.socket
    }

    /// Send data to the peer through the TURN relay.
    ///
    /// Uses `ChannelData` framing if a channel is bound, otherwise falls back
    /// to STUN `SendIndication`.
    ///
    /// # Errors
    ///
    /// Returns an error if the payload is too large or the send fails.
    pub async fn send(&self, data: &[u8]) -> Result<(), TurnSessionError> {
        if data.len() > MAX_PAYLOAD {
            return Err(TurnSessionError::PayloadTooLarge(data.len()));
        }

        let wire = self.channel.map_or_else(
            || self.build_send_indication(data),
            |ch| Self::build_channel_data(ch, data),
        );

        self.socket.send_to(&wire, self.client.server_addr()).await?;
        Ok(())
    }

    /// Receive data from the peer through the TURN relay.
    ///
    /// Blocks until data arrives or `recv_timeout` expires. Returns the number
    /// of bytes written to `buf`.
    ///
    /// Transparently handles both `ChannelData` frames and STUN `DataIndication`
    /// messages from the server.
    ///
    /// # Errors
    ///
    /// Returns an error on timeout, I/O failure, or unexpected message type.
    pub async fn recv(&self, buf: &mut [u8]) -> Result<usize, TurnSessionError> {
        let mut recv_buf = self.recv_buf.lock().await;
        if recv_buf.len() < buf.len() {
            recv_buf.resize(buf.len().max(4096), 0);
        }

        let (len, _from) =
            tokio::time::timeout(self.recv_timeout, self.socket.recv_from(&mut recv_buf))
                .await
                .map_err(|_| TurnSessionError::Timeout(self.recv_timeout))?
                .map_err(TurnSessionError::Io)?;

        let raw = recv_buf[..len].to_vec();
        drop(recv_buf);

        if raw.len() >= 4 && Self::is_channel_data(&raw) {
            Self::parse_channel_data(&raw, buf)
        } else {
            Self::parse_data_indication(&raw, buf)
        }
    }

    /// Refresh the allocation lifetime. Call periodically before expiry.
    ///
    /// # Errors
    ///
    /// Returns an error if the server rejects the refresh.
    pub async fn refresh(&self, lifetime_secs: u32) -> Result<u32, TurnSessionError> {
        Ok(self.client.refresh(&self.socket, lifetime_secs).await?)
    }

    /// Release the allocation (lifetime=0) and close the session.
    ///
    /// # Errors
    ///
    /// Returns an error if the release fails.
    pub async fn close(self) -> Result<(), TurnSessionError> {
        let _ = self.client.refresh(&self.socket, 0).await;
        Ok(())
    }

    // ── ChannelData framing (RFC 5766 §11.4) ─────────────────────────────

    fn build_channel_data(channel: u16, data: &[u8]) -> bytes::Bytes {
        let len = u16::try_from(data.len()).unwrap_or(u16::MAX);
        let mut buf = BytesMut::with_capacity(4 + data.len());
        buf.put_u16(channel);
        buf.put_u16(len);
        buf.put_slice(data);
        buf.freeze()
    }

    fn is_channel_data(raw: &[u8]) -> bool {
        raw[0] >= 0x40 && raw[0] <= 0x7F
    }

    fn parse_channel_data(raw: &[u8], buf: &mut [u8]) -> Result<usize, TurnSessionError> {
        if raw.len() < 4 {
            return Err(TurnSessionError::UnexpectedMessage(0));
        }
        let payload_len = u16::from_be_bytes([raw[2], raw[3]]) as usize;
        let payload = &raw[4..];
        let copy_len = payload_len.min(payload.len()).min(buf.len());
        buf[..copy_len].copy_from_slice(&payload[..copy_len]);
        Ok(copy_len)
    }

    // ── STUN Send/Data Indication framing (RFC 5766 §10) ─────────────────

    fn build_send_indication(&self, data: &[u8]) -> bytes::Bytes {
        let mut msg = StunMessage::new_binding_request();
        msg.message_type = MessageType::SendIndication;

        msg.attributes.push(StunAttribute::Unknown(
            0x0012,
            encode_xor_peer_address(&self.peer_addr, &msg.transaction_id),
        ));

        msg.attributes.push(StunAttribute::Unknown(0x0013, bytes::Bytes::copy_from_slice(data)));

        msg.encode()
    }

    fn parse_data_indication(raw: &[u8], buf: &mut [u8]) -> Result<usize, TurnSessionError> {
        let msg = StunMessage::decode(raw).map_err(TurnSessionError::Protocol)?;

        if msg.message_type != MessageType::DataIndication {
            return Err(TurnSessionError::UnexpectedMessage(msg.message_type.to_u16()));
        }

        for attr in &msg.attributes {
            if let StunAttribute::Unknown(0x0013, data) = attr {
                let copy_len = data.len().min(buf.len());
                buf[..copy_len].copy_from_slice(&data[..copy_len]);
                return Ok(copy_len);
            }
        }

        Ok(0)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn config_defaults() {
        let config = TurnSessionConfig::new(
            "192.0.2.1:3478".parse().unwrap(),
            StunCredentials {
                username: "u".into(),
                key: vec![1, 2, 3],
            },
            "10.0.0.5:9200".parse().unwrap(),
        );
        assert!(config.use_channel);
        assert_eq!(config.channel, 0x4000);
        assert_eq!(config.control_timeout, Duration::from_secs(5));
        assert_eq!(config.recv_timeout, Duration::from_secs(30));
    }

    #[test]
    fn config_without_channel() {
        let config = TurnSessionConfig::new(
            "192.0.2.1:3478".parse().unwrap(),
            StunCredentials {
                username: "u".into(),
                key: vec![],
            },
            "10.0.0.5:9200".parse().unwrap(),
        )
        .without_channel();
        assert!(!config.use_channel);
    }

    #[test]
    fn channel_data_roundtrip() {
        let payload = b"hello TURN relay";
        let frame = TurnSession::build_channel_data(0x4000, payload);

        assert!(TurnSession::is_channel_data(&frame));

        let mut buf = vec![0u8; 256];
        let n = TurnSession::parse_channel_data(&frame, &mut buf).unwrap();
        assert_eq!(&buf[..n], payload);
    }

    #[test]
    fn channel_data_detection() {
        assert!(!TurnSession::is_channel_data(&[0x00, 0x01, 0x00, 0x00]));
        assert!(!TurnSession::is_channel_data(&[0x01, 0x01, 0x00, 0x00]));
        assert!(TurnSession::is_channel_data(&[0x40, 0x00, 0x00, 0x04, 0, 0, 0, 0]));
        assert!(TurnSession::is_channel_data(&[0x7F, 0xFF, 0x00, 0x01, 0]));
    }

    #[test]
    fn send_indication_framing() {
        let config = TurnSessionConfig::new(
            "192.0.2.1:3478".parse().unwrap(),
            StunCredentials {
                username: "test".into(),
                key: b"key".to_vec(),
            },
            "10.0.0.5:9200".parse().unwrap(),
        )
        .without_channel();

        let frame = TurnSession::build_channel_data(0x4001, b"test data");
        assert_eq!(frame.len(), 4 + 9);
        assert_eq!(frame[0], 0x40);
        assert_eq!(frame[1], 0x01);
        assert_eq!(u16::from_be_bytes([frame[2], frame[3]]), 9);
        assert!(!config.use_channel);
    }

    #[test]
    fn data_indication_parsing() {
        let mut msg = StunMessage::new_binding_request();
        msg.message_type = MessageType::DataIndication;
        msg.attributes
            .push(StunAttribute::Unknown(0x0013, bytes::Bytes::from_static(b"relayed payload")));
        let wire = msg.encode();

        let mut buf = vec![0u8; 256];
        let n = TurnSession::parse_data_indication(&wire, &mut buf).unwrap();
        assert_eq!(&buf[..n], b"relayed payload");
    }

    #[test]
    fn data_indication_rejects_non_indication() {
        let msg = StunMessage::new_binding_request();
        let wire = msg.encode();

        let mut buf = vec![0u8; 256];
        let result = TurnSession::parse_data_indication(&wire, &mut buf);
        assert!(result.is_err());
    }
}
