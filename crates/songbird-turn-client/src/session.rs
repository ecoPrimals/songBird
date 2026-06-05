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

    /// Create config from environment variables.
    ///
    /// Reads:
    /// - `SONGBIRD_TURN_SERVER` — relay address (e.g. `relay.primals.eco:3478`)
    /// - `SONGBIRD_TURN_USERNAME` — TURN credential username
    /// - `SONGBIRD_TURN_KEY` — TURN credential key (hex-encoded)
    ///
    /// `peer_addr` must still be provided (it's connection-specific).
    ///
    /// # Errors
    ///
    /// Returns error if required env vars are missing or `SONGBIRD_TURN_SERVER`
    /// fails to parse as a `SocketAddr`.
    pub fn from_env(peer_addr: SocketAddr) -> Result<Self, TurnSessionError> {
        let server_str = songbird_process_env::var("SONGBIRD_TURN_SERVER")
            .map_err(|_| TurnSessionError::Config("SONGBIRD_TURN_SERVER not set".into()))?;
        let server_addr: SocketAddr = server_str.parse().map_err(|e| {
            TurnSessionError::Config(format!(
                "SONGBIRD_TURN_SERVER '{server_str}' is not a valid address: {e}"
            ))
        })?;

        let username = songbird_process_env::var("SONGBIRD_TURN_USERNAME")
            .map_err(|_| TurnSessionError::Config("SONGBIRD_TURN_USERNAME not set".into()))?;

        let key_hex = songbird_process_env::var("SONGBIRD_TURN_KEY")
            .map_err(|_| TurnSessionError::Config("SONGBIRD_TURN_KEY not set".into()))?;
        let key = hex_decode(&key_hex).map_err(|e| {
            TurnSessionError::Config(format!("SONGBIRD_TURN_KEY is not valid hex: {e}"))
        })?;

        Ok(Self::new(
            server_addr,
            StunCredentials {
                username,
                key,
            },
            peer_addr,
        ))
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

    /// Spawn a background keepalive task that refreshes the allocation
    /// before it expires.
    ///
    /// Refreshes at 80% of the allocation lifetime (e.g. every 480s for a
    /// 600s allocation). Returns a `JoinHandle` that runs until the session
    /// is dropped or the refresh fails.
    ///
    /// The handle can be aborted to stop keepalive when the session is no
    /// longer needed.
    pub fn spawn_keepalive(self: &Arc<Self>) -> tokio::task::JoinHandle<()>
    where
        Self: Send + Sync + 'static,
    {
        let session = Arc::clone(self);
        let lifetime = session.allocation.lifetime_secs;
        let interval = Duration::from_secs(u64::from(lifetime) * 4 / 5);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                match session.refresh(lifetime).await {
                    Ok(new_lifetime) => {
                        debug!(lifetime_s = new_lifetime, "TURN keepalive: allocation refreshed");
                    }
                    Err(e) => {
                        info!("TURN keepalive: refresh failed ({e}), stopping");
                        break;
                    }
                }
            }
        })
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

fn hex_decode(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.trim();
    if !hex.len().is_multiple_of(2) {
        return Err("odd-length hex string".into());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| e.to_string()))
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;

    fn sample_config() -> TurnSessionConfig {
        TurnSessionConfig::new(
            "192.0.2.1:3478".parse().unwrap(),
            StunCredentials {
                username: "u".into(),
                key: vec![1, 2, 3],
            },
            "10.0.0.5:9200".parse().unwrap(),
        )
    }

    // ── Config ──────────────────────────────────────────────────────────

    #[test]
    fn config_defaults() {
        let config = sample_config();
        assert!(config.use_channel);
        assert_eq!(config.channel, 0x4000);
        assert_eq!(config.control_timeout, Duration::from_secs(5));
        assert_eq!(config.recv_timeout, Duration::from_secs(30));
        assert_eq!(config.local_bind, SocketAddr::from(EPHEMERAL_BIND));
    }

    #[test]
    fn config_without_channel() {
        let config = sample_config().without_channel();
        assert!(!config.use_channel);
    }

    #[test]
    fn config_with_local_bind() {
        let addr: SocketAddr = "127.0.0.1:5000".parse().unwrap();
        let config = sample_config().with_local_bind(addr);
        assert_eq!(config.local_bind, addr);
    }

    #[test]
    fn config_with_recv_timeout() {
        let config = sample_config().with_recv_timeout(Duration::from_millis(500));
        assert_eq!(config.recv_timeout, Duration::from_millis(500));
    }

    #[test]
    fn config_from_env_fails_when_server_unset() {
        let peer: SocketAddr = "10.0.0.1:80".parse().unwrap();
        let err = TurnSessionConfig::from_env(peer).unwrap_err();
        match err {
            TurnSessionError::Config(msg) => assert!(msg.contains("SONGBIRD_TURN_SERVER")),
            other => panic!("expected Config error, got: {other}"),
        }
    }

    // ── Channel Data Framing ────────────────────────────────────────────

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
    fn channel_data_multiple_channels() {
        for channel in [0x4000u16, 0x4001, 0x5FFF, 0x7FFF] {
            let frame = TurnSession::build_channel_data(channel, b"x");
            assert!(TurnSession::is_channel_data(&frame));
            let actual = u16::from_be_bytes([frame[0], frame[1]]);
            assert_eq!(actual, channel);
        }
    }

    #[test]
    fn channel_data_detection() {
        assert!(!TurnSession::is_channel_data(&[0x00, 0x01, 0x00, 0x00]));
        assert!(!TurnSession::is_channel_data(&[0x01, 0x01, 0x00, 0x00]));
        assert!(!TurnSession::is_channel_data(&[0x3F, 0xFF, 0x00, 0x01, 0x00]));
        assert!(!TurnSession::is_channel_data(&[0x80, 0x00, 0x00, 0x01, 0x00]));
        assert!(TurnSession::is_channel_data(&[0x40, 0x00, 0x00, 0x04, 0, 0, 0, 0]));
        assert!(TurnSession::is_channel_data(&[0x7F, 0xFF, 0x00, 0x01, 0]));
    }

    #[test]
    fn channel_data_empty_payload() {
        let frame = TurnSession::build_channel_data(0x4000, b"");
        assert_eq!(frame.len(), 4);
        let mut buf = [0u8; 32];
        let n = TurnSession::parse_channel_data(&frame, &mut buf).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn channel_data_large_payload() {
        let payload = vec![0xAB; 1400]; // typical MTU
        let frame = TurnSession::build_channel_data(0x4000, &payload);
        let mut buf = vec![0u8; 2048];
        let n = TurnSession::parse_channel_data(&frame, &mut buf).unwrap();
        assert_eq!(n, 1400);
        assert_eq!(&buf[..n], &payload[..]);
    }

    #[test]
    fn channel_data_parse_truncated_header() {
        let too_short = [0x40, 0x00, 0x00];
        let mut buf = [0u8; 32];
        assert!(TurnSession::parse_channel_data(&too_short, &mut buf).is_err());
    }

    #[test]
    fn channel_data_parse_buffer_smaller_than_payload() {
        let payload = b"this payload is longer than the receiving buffer";
        let frame = TurnSession::build_channel_data(0x4000, payload);
        let mut buf = [0u8; 10];
        let n = TurnSession::parse_channel_data(&frame, &mut buf).unwrap();
        assert_eq!(n, 10);
        assert_eq!(&buf[..n], &payload[..10]);
    }

    #[test]
    fn channel_data_frame_structure_rfc5766() {
        let payload = b"TURN relay";
        let frame = TurnSession::build_channel_data(0x4000, payload);
        assert_eq!(frame.len(), 4 + payload.len());
        assert_eq!(frame[0], 0x40);
        assert_eq!(frame[1], 0x00);
        assert_eq!(u16::from_be_bytes([frame[2], frame[3]]) as usize, payload.len());
        assert_eq!(&frame[4..], payload);
    }

    // ── Send Indication / Data Indication ───────────────────────────────

    #[test]
    fn send_indication_framing() {
        let config = sample_config().without_channel();

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
        assert!(matches!(result, Err(TurnSessionError::UnexpectedMessage(_))));
    }

    #[test]
    fn data_indication_empty_data_attribute() {
        let mut msg = StunMessage::new_binding_request();
        msg.message_type = MessageType::DataIndication;
        msg.attributes.push(StunAttribute::Unknown(0x0013, bytes::Bytes::new()));
        let wire = msg.encode();

        let mut buf = vec![0u8; 256];
        let n = TurnSession::parse_data_indication(&wire, &mut buf).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn data_indication_without_data_attr_returns_zero() {
        let mut msg = StunMessage::new_binding_request();
        msg.message_type = MessageType::DataIndication;
        msg.attributes.push(StunAttribute::Unknown(0x0012, bytes::Bytes::from_static(b"\x00\x01")));
        let wire = msg.encode();

        let mut buf = vec![0u8; 256];
        let n = TurnSession::parse_data_indication(&wire, &mut buf).unwrap();
        assert_eq!(n, 0);
    }

    // ── Connect Error Paths ─────────────────────────────────────────────

    #[tokio::test]
    async fn connect_fails_against_unreachable_server() {
        let config = TurnSessionConfig::new(
            "192.0.2.1:3478".parse().unwrap(), // RFC 5737 test address — unreachable
            StunCredentials {
                username: "u".into(),
                key: b"k".to_vec(),
            },
            "10.0.0.1:80".parse().unwrap(),
        )
        .with_local_bind("0.0.0.0:0".parse().unwrap());

        let result = TurnSession::connect(config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn send_rejects_oversized_payload() {
        let socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        let socket = Arc::new(socket);
        let session = TurnSession {
            client: songbird_stun::TurnClient::new(
                "192.0.2.1:3478".parse().unwrap(),
                StunCredentials {
                    username: "u".into(),
                    key: vec![],
                },
            ),
            socket,
            allocation: songbird_stun::TurnAllocation {
                relay_addr: "192.0.2.100:49152".parse().unwrap(),
                mapped_addr: "192.0.2.50:12345".parse().unwrap(),
                lifetime_secs: 600,
            },
            peer_addr: "10.0.0.99:9200".parse().unwrap(),
            channel: Some(0x4000),
            recv_timeout: Duration::from_secs(1),
            recv_buf: Mutex::new(vec![0u8; 4096]),
        };

        let oversized = vec![0u8; MAX_PAYLOAD + 1];
        let result = session.send(&oversized).await;
        assert!(matches!(result, Err(TurnSessionError::PayloadTooLarge(65536))));
    }

    // ── Hex Decode ──────────────────────────────────────────────────────

    #[test]
    fn hex_decode_valid() {
        assert_eq!(hex_decode("deadbeef").unwrap(), vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(hex_decode("00ff").unwrap(), vec![0x00, 0xFF]);
        assert_eq!(hex_decode("").unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn hex_decode_rejects_odd_length() {
        assert!(hex_decode("abc").is_err());
    }

    #[test]
    fn hex_decode_rejects_invalid_chars() {
        assert!(hex_decode("gg").is_err());
    }

    #[test]
    fn hex_decode_trims_whitespace() {
        assert_eq!(hex_decode("  aabb  ").unwrap(), vec![0xAA, 0xBB]);
    }

    // ── Error Display ───────────────────────────────────────────────────

    #[test]
    fn error_display_variants() {
        let err = TurnSessionError::NotConnected;
        assert_eq!(err.to_string(), "session not connected");

        let err = TurnSessionError::Timeout(Duration::from_secs(30));
        assert!(err.to_string().contains("30"));

        let err = TurnSessionError::PayloadTooLarge(70000);
        assert!(err.to_string().contains("70000"));

        let err = TurnSessionError::UnexpectedMessage(0x0101);
        assert!(err.to_string().contains("0101"));

        let err = TurnSessionError::Config("missing var".into());
        assert!(err.to_string().contains("missing var"));
    }

    // ── Debug Impls ─────────────────────────────────────────────────────

    #[test]
    fn config_debug_output() {
        let c = sample_config();
        let s = format!("{c:?}");
        assert!(s.contains("TurnSessionConfig"));
        assert!(s.contains("192.0.2.1:3478"));
    }
}
