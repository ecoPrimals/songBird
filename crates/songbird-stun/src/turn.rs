// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! RFC 5766 TURN client for relay-assisted NAT traversal (H2-14).
//!
//! TURN (Traversal Using Relays around NAT) provides a relay when direct
//! connectivity and STUN-assisted hole-punching both fail. This module
//! implements the client-side of the TURN protocol:
//!
//! - **Allocate**: Request a relay address from the TURN server
//! - **Refresh**: Keep the allocation alive (or release it)
//! - **CreatePermission**: Allow specific peers to send through the relay
//! - **ChannelBind**: Bind a channel number for efficient peer data relay
//!
//! ## Authentication
//!
//! TURN servers require long-term credentials (RFC 5389 §10.2). In the
//! ecoPrimals ecosystem, these credentials are derived from `BearDog`'s
//! `auth.public_key` (JH-11) — beacon-tier only per
//! `DARK_FOREST_BEACON_GENETICS_STANDARD.md`.
//!
//! ## Wire Protocol
//!
//! TURN reuses STUN message framing with additional method types (Allocate=0x0003,
//! Refresh=0x0004, CreatePermission=0x0008, ChannelBind=0x0009) and attributes
//! (LIFETIME, XOR-RELAYED-ADDRESS, XOR-PEER-ADDRESS, CHANNEL-NUMBER, etc.).

use crate::error::{StunError, StunResult};
use crate::message::{MAGIC_COOKIE, StunAttribute, StunMessage};
use crate::types::StunCredentials;
use bytes::{BufMut, BytesMut};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use tracing::{debug, info};

// TURN method types are in crate::message::MessageType (Allocate, Refresh,
// CreatePermission, ChannelBind and their success/error variants).
use crate::message::MessageType;

/// TURN allocation result.
#[derive(Debug, Clone)]
pub struct TurnAllocation {
    /// Relay address allocated by the TURN server (XOR-RELAYED-ADDRESS).
    pub relay_addr: SocketAddr,
    /// Reflexive address (XOR-MAPPED-ADDRESS from the Allocate response).
    pub mapped_addr: SocketAddr,
    /// Allocation lifetime in seconds.
    pub lifetime_secs: u32,
}

/// TURN client for relay-based NAT traversal.
///
/// Implements Allocate, Refresh, `CreatePermission`, and `ChannelBind`.
#[derive(Debug)]
pub struct TurnClient {
    server_addr: SocketAddr,
    credentials: StunCredentials,
    request_timeout: Duration,
}

impl TurnClient {
    /// Create a new TURN client targeting a specific server.
    #[must_use]
    pub fn new(server_addr: SocketAddr, credentials: StunCredentials) -> Self {
        Self {
            server_addr,
            credentials,
            request_timeout: Duration::from_secs(5),
        }
    }

    /// The TURN server address this client targets.
    #[must_use]
    pub const fn server_addr(&self) -> SocketAddr {
        self.server_addr
    }

    /// Override the request timeout (default 5s).
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Request a relay allocation from the TURN server (RFC 5766 §6).
    ///
    /// Sends an Allocate request with REQUESTED-TRANSPORT (UDP) and
    /// long-term credentials (USERNAME + MESSAGE-INTEGRITY + FINGERPRINT).
    ///
    /// # Errors
    ///
    /// Returns an error if the server is unreachable, rejects credentials,
    /// or returns a malformed response.
    pub async fn allocate(&self, socket: &UdpSocket) -> StunResult<TurnAllocation> {
        info!("TURN: Allocate request to {}", self.server_addr);

        let request = self.build_allocate_request();
        let wire = request.encode_authenticated(&self.credentials.key);

        socket
            .send_to(&wire, self.server_addr)
            .await
            .map_err(|e| StunError::Network(format!("TURN allocate send failed: {e}")))?;

        let mut buf = vec![0u8; 2048];
        let (len, _) = timeout(self.request_timeout, socket.recv_from(&mut buf))
            .await
            .map_err(|_| StunError::Timeout(self.request_timeout))?
            .map_err(|e| StunError::Network(format!("TURN allocate recv failed: {e}")))?;

        let response = StunMessage::decode(&buf[..len])?;

        // Check for error response
        if response.message_type == MessageType::AllocateError {
            return Err(StunError::InvalidResponse(
                "TURN server rejected Allocate request".to_string(),
            ));
        }

        Self::parse_allocate_response(&response)
    }

    /// Refresh the allocation lifetime (RFC 5766 §7).
    ///
    /// # Arguments
    ///
    /// * `lifetime_secs` — Desired lifetime. Use 0 to release the allocation.
    ///
    /// # Errors
    ///
    /// Returns an error if the server rejects the refresh.
    pub async fn refresh(&self, socket: &UdpSocket, lifetime_secs: u32) -> StunResult<u32> {
        info!("TURN: Refresh (lifetime={lifetime_secs}s) to {}", self.server_addr);

        let request = self.build_refresh_request(lifetime_secs);
        let wire = request.encode_authenticated(&self.credentials.key);

        socket
            .send_to(&wire, self.server_addr)
            .await
            .map_err(|e| StunError::Network(format!("TURN refresh send failed: {e}")))?;

        let mut buf = vec![0u8; 2048];
        let (len, _) = timeout(self.request_timeout, socket.recv_from(&mut buf))
            .await
            .map_err(|_| StunError::Timeout(self.request_timeout))?
            .map_err(|e| StunError::Network(format!("TURN refresh recv failed: {e}")))?;

        let response = StunMessage::decode(&buf[..len])?;
        Self::parse_lifetime_from_response(&response)
    }

    /// Create a permission for a peer to send data through the relay (RFC 5766 §9).
    ///
    /// # Errors
    ///
    /// Returns an error if the server rejects the permission request.
    pub async fn create_permission(
        &self,
        socket: &UdpSocket,
        peer_addr: SocketAddr,
    ) -> StunResult<()> {
        info!("TURN: CreatePermission for {peer_addr} to {}", self.server_addr);

        let request = self.build_create_permission_request(peer_addr);
        let wire = request.encode_authenticated(&self.credentials.key);

        socket
            .send_to(&wire, self.server_addr)
            .await
            .map_err(|e| StunError::Network(format!("TURN permission send failed: {e}")))?;

        let mut buf = vec![0u8; 2048];
        let (len, _) = timeout(self.request_timeout, socket.recv_from(&mut buf))
            .await
            .map_err(|_| StunError::Timeout(self.request_timeout))?
            .map_err(|e| StunError::Network(format!("TURN permission recv failed: {e}")))?;

        let response = StunMessage::decode(&buf[..len])?;
        if response.message_type == MessageType::CreatePermissionSuccess {
            debug!("TURN: Permission granted for {peer_addr}");
            Ok(())
        } else {
            let raw = response.message_type.to_u16();
            Err(StunError::InvalidResponse(format!(
                "TURN CreatePermission failed (type=0x{raw:04x})"
            )))
        }
    }

    /// Bind a channel number for efficient data relay (RFC 5766 §11).
    ///
    /// Channel numbers are 0x4000–0x7FFF. Once bound, data can be sent
    /// using the 4-byte ChannelData header instead of full STUN framing.
    ///
    /// # Errors
    ///
    /// Returns an error if the binding is rejected.
    pub async fn channel_bind(
        &self,
        socket: &UdpSocket,
        channel: u16,
        peer_addr: SocketAddr,
    ) -> StunResult<()> {
        if !(0x4000..=0x7FFF).contains(&channel) {
            return Err(StunError::Config(format!(
                "Channel number must be 0x4000–0x7FFF, got 0x{channel:04x}"
            )));
        }

        info!("TURN: ChannelBind 0x{channel:04x} → {peer_addr} to {}", self.server_addr);

        let request = self.build_channel_bind_request(channel, peer_addr);
        let wire = request.encode_authenticated(&self.credentials.key);

        socket
            .send_to(&wire, self.server_addr)
            .await
            .map_err(|e| StunError::Network(format!("TURN channel bind send failed: {e}")))?;

        let mut buf = vec![0u8; 2048];
        let (len, _) = timeout(self.request_timeout, socket.recv_from(&mut buf))
            .await
            .map_err(|_| StunError::Timeout(self.request_timeout))?
            .map_err(|e| StunError::Network(format!("TURN channel bind recv failed: {e}")))?;

        let response = StunMessage::decode(&buf[..len])?;
        if response.message_type == MessageType::ChannelBindSuccess {
            debug!("TURN: Channel 0x{channel:04x} bound to {peer_addr}");
            Ok(())
        } else {
            let raw = response.message_type.to_u16();
            Err(StunError::InvalidResponse(format!("TURN ChannelBind failed (type=0x{raw:04x})")))
        }
    }

    // --- Private helpers ---

    fn build_allocate_request(&self) -> StunMessage {
        let mut msg = StunMessage::new_binding_request();
        // Override message type to Allocate
        msg.message_type = MessageType::Allocate;
        // USERNAME
        msg.attributes.push(StunAttribute::Username(self.credentials.username.clone()));
        // REQUESTED-TRANSPORT (0x0019): UDP = 17
        let mut transport_attr = BytesMut::with_capacity(4);
        transport_attr.put_u8(17); // UDP protocol number
        transport_attr.put_u8(0);
        transport_attr.put_u8(0);
        transport_attr.put_u8(0);
        msg.attributes.push(StunAttribute::Unknown(0x0019, transport_attr.freeze()));
        msg
    }

    fn build_refresh_request(&self, lifetime_secs: u32) -> StunMessage {
        let mut msg = StunMessage::new_binding_request();
        msg.message_type = MessageType::Refresh;
        msg.attributes.push(StunAttribute::Username(self.credentials.username.clone()));
        // LIFETIME (0x000D)
        let mut lifetime_attr = BytesMut::with_capacity(4);
        lifetime_attr.put_u32(lifetime_secs);
        msg.attributes.push(StunAttribute::Unknown(0x000D, lifetime_attr.freeze()));
        msg
    }

    fn build_create_permission_request(&self, peer_addr: SocketAddr) -> StunMessage {
        let mut msg = StunMessage::new_binding_request();
        msg.message_type = MessageType::CreatePermission;
        msg.attributes.push(StunAttribute::Username(self.credentials.username.clone()));
        // XOR-PEER-ADDRESS (0x0012)
        msg.attributes.push(StunAttribute::Unknown(
            0x0012,
            encode_xor_peer_address(&peer_addr, &msg.transaction_id),
        ));
        msg
    }

    fn build_channel_bind_request(&self, channel: u16, peer_addr: SocketAddr) -> StunMessage {
        let mut msg = StunMessage::new_binding_request();
        msg.message_type = MessageType::ChannelBind;
        msg.attributes.push(StunAttribute::Username(self.credentials.username.clone()));
        // CHANNEL-NUMBER (0x000C): 2 bytes channel + 2 bytes RFFU
        let mut chan_attr = BytesMut::with_capacity(4);
        chan_attr.put_u16(channel);
        chan_attr.put_u16(0); // RFFU
        msg.attributes.push(StunAttribute::Unknown(0x000C, chan_attr.freeze()));
        // XOR-PEER-ADDRESS (0x0012)
        msg.attributes.push(StunAttribute::Unknown(
            0x0012,
            encode_xor_peer_address(&peer_addr, &msg.transaction_id),
        ));
        msg
    }

    fn parse_allocate_response(response: &StunMessage) -> StunResult<TurnAllocation> {
        let mapped_addr = response.get_any_mapped_address().ok_or_else(|| {
            StunError::InvalidResponse("Allocate response missing XOR-MAPPED-ADDRESS".to_string())
        })?;

        // Find XOR-RELAYED-ADDRESS (0x0016) in Unknown attributes
        let relay_addr = response
            .attributes
            .iter()
            .find_map(|attr| {
                if let StunAttribute::Unknown(0x0016, data) = attr {
                    StunAttribute::decode_address(
                        data,
                        Some(MAGIC_COOKIE),
                        &response.transaction_id,
                    )
                    .ok()
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                StunError::InvalidResponse(
                    "Allocate response missing XOR-RELAYED-ADDRESS".to_string(),
                )
            })?;

        let lifetime_secs = Self::parse_lifetime_from_response(response).unwrap_or(600);

        info!(
            "TURN: Allocation obtained — relay={relay_addr}, mapped={mapped_addr}, lifetime={lifetime_secs}s"
        );

        Ok(TurnAllocation {
            relay_addr,
            mapped_addr,
            lifetime_secs,
        })
    }

    fn parse_lifetime_from_response(response: &StunMessage) -> StunResult<u32> {
        for attr in &response.attributes {
            if let StunAttribute::Unknown(0x000D, data) = attr
                && data.len() >= 4
            {
                return Ok(u32::from_be_bytes([data[0], data[1], data[2], data[3]]));
            }
        }
        Err(StunError::InvalidResponse("Response missing LIFETIME attribute".to_string()))
    }
}

/// Encode a peer address as XOR-PEER-ADDRESS value bytes (without TLV header).
///
/// Used by both `TurnClient` control-plane methods and `songbird-turn-client`
/// data-plane `SendIndication` framing.
pub fn encode_xor_peer_address(addr: &SocketAddr, transaction_id: &[u8; 12]) -> bytes::Bytes {
    use std::net::IpAddr;

    let mut buf = BytesMut::new();
    buf.put_u8(0); // reserved
    match addr {
        SocketAddr::V4(_) => buf.put_u8(0x01),
        SocketAddr::V6(_) => buf.put_u8(0x02),
    }
    // Port XOR'd with high 16 bits of magic cookie
    let port = addr.port() ^ (MAGIC_COOKIE >> 16) as u16;
    buf.put_u16(port);
    // Address
    match addr.ip() {
        IpAddr::V4(ip) => {
            let xored = u32::from(ip) ^ MAGIC_COOKIE;
            buf.put_u32(xored);
        }
        IpAddr::V6(ip) => {
            let mut xor_pad = [0u8; 16];
            xor_pad[..4].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
            xor_pad[4..].copy_from_slice(transaction_id);
            let raw = ip.octets();
            for i in 0..16 {
                buf.put_u8(raw[i] ^ xor_pad[i]);
            }
        }
    }
    buf.freeze()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn turn_client_creation() {
        let creds = StunCredentials {
            username: "user".to_string(),
            key: b"secret".to_vec(),
        };
        let addr: SocketAddr = "192.0.2.1:3478".parse().unwrap();
        let client = TurnClient::new(addr, creds);
        assert_eq!(client.server_addr, addr);
        assert_eq!(client.request_timeout, Duration::from_secs(5));
    }

    #[test]
    fn turn_client_with_timeout() {
        let creds = StunCredentials {
            username: "u".to_string(),
            key: vec![],
        };
        let addr: SocketAddr = "10.0.0.1:3478".parse().unwrap();
        let client = TurnClient::new(addr, creds).with_timeout(Duration::from_secs(10));
        assert_eq!(client.request_timeout, Duration::from_secs(10));
    }

    #[test]
    fn encode_xor_peer_address_ipv4_roundtrip() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 5)), 9999);
        let tid = [0xAA; 12];
        let encoded = encode_xor_peer_address(&addr, &tid);

        let decoded = StunAttribute::decode_address(&encoded, Some(MAGIC_COOKIE), &tid)
            .expect("decode XOR-PEER-ADDRESS");
        assert_eq!(decoded, addr);
    }

    #[test]
    fn encode_xor_peer_address_ipv6_roundtrip() {
        use std::net::Ipv6Addr;
        let addr =
            SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)), 4433);
        let tid = [0x55; 12];
        let encoded = encode_xor_peer_address(&addr, &tid);

        let decoded = StunAttribute::decode_address(&encoded, Some(MAGIC_COOKIE), &tid)
            .expect("decode IPv6 XOR-PEER-ADDRESS");
        assert_eq!(decoded, addr);
    }

    #[test]
    fn channel_bind_rejects_invalid_channel_number() {
        let creds = StunCredentials {
            username: "u".to_string(),
            key: b"k".to_vec(),
        };
        let addr: SocketAddr = "10.0.0.1:3478".parse().unwrap();
        let client = TurnClient::new(addr, creds);
        // Channel numbers must be 0x4000–0x7FFF
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let err = client
                .channel_bind(&socket, 0x3FFF, "10.0.0.2:5000".parse().unwrap())
                .await
                .expect_err("invalid channel");
            assert!(err.to_string().contains("0x4000"), "got: {err}");
        });
    }

    #[test]
    fn turn_method_wire_values() {
        assert_eq!(MessageType::Allocate.to_u16(), 0x0003);
        assert_eq!(MessageType::AllocateSuccess.to_u16(), 0x0103);
        assert_eq!(MessageType::AllocateError.to_u16(), 0x0113);
        assert_eq!(MessageType::Refresh.to_u16(), 0x0004);
        assert_eq!(MessageType::RefreshSuccess.to_u16(), 0x0104);
        assert_eq!(MessageType::CreatePermission.to_u16(), 0x0008);
        assert_eq!(MessageType::CreatePermissionSuccess.to_u16(), 0x0108);
        assert_eq!(MessageType::ChannelBind.to_u16(), 0x0009);
        assert_eq!(MessageType::ChannelBindSuccess.to_u16(), 0x0109);
    }

    #[test]
    fn turn_allocation_debug() {
        let alloc = TurnAllocation {
            relay_addr: "192.0.2.1:49152".parse().unwrap(),
            mapped_addr: "198.51.100.5:12345".parse().unwrap(),
            lifetime_secs: 600,
        };
        assert!(!format!("{alloc:?}").is_empty());
    }

    async fn start_test_turn_server() -> (tokio::task::JoinHandle<()>, SocketAddr) {
        use crate::turn_server::{StaticCredentialStore, TurnRelayServer};
        use std::sync::Arc;

        let mut store = StaticCredentialStore::new();
        store.insert("turnuser".to_string(), b"turnkey456".to_vec());
        let server = TurnRelayServer::new("127.0.0.1:0".parse().unwrap(), Arc::new(store));
        let (tx, rx) = tokio::sync::oneshot::channel();
        let handle = tokio::spawn(async move {
            let _ = server.run_with_ready(tx).await;
        });
        let addr = rx.await.expect("server ready");
        (handle, addr)
    }

    fn turn_test_creds() -> StunCredentials {
        StunCredentials {
            username: "turnuser".to_string(),
            key: b"turnkey456".to_vec(),
        }
    }

    #[tokio::test]
    async fn turn_client_create_permission_success() {
        let (handle, server_addr) = start_test_turn_server().await;
        let client = TurnClient::new(server_addr, turn_test_creds());
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        client.allocate(&socket).await.expect("allocate");
        let peer: SocketAddr = "127.0.0.1:7777".parse().unwrap();
        client.create_permission(&socket, peer).await.expect("create_permission");

        handle.abort();
    }

    #[tokio::test]
    async fn turn_client_channel_bind_success() {
        let (handle, server_addr) = start_test_turn_server().await;
        let client = TurnClient::new(server_addr, turn_test_creds());
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        client.allocate(&socket).await.expect("allocate");
        let peer: SocketAddr = "127.0.0.1:6666".parse().unwrap();
        client.create_permission(&socket, peer).await.expect("permission");
        client.channel_bind(&socket, 0x4001, peer).await.expect("channel_bind");

        handle.abort();
    }

    #[tokio::test]
    async fn turn_client_allocate_error_on_rejection() {
        let (handle, server_addr) = start_test_turn_server().await;
        let bad_creds = StunCredentials {
            username: "wrong".to_string(),
            key: b"bad".to_vec(),
        };
        let client = TurnClient::new(server_addr, bad_creds);
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        let err = client.allocate(&socket).await.expect_err("allocate rejected");
        assert!(err.to_string().contains("rejected"), "unexpected error: {err}");

        handle.abort();
    }

    #[tokio::test]
    async fn turn_client_refresh_release_with_zero_lifetime() {
        let (handle, server_addr) = start_test_turn_server().await;
        let client = TurnClient::new(server_addr, turn_test_creds());
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        client.allocate(&socket).await.expect("allocate");
        let lifetime = client.refresh(&socket, 0).await.expect("refresh release");
        assert_eq!(lifetime, 0);

        // After release, a new allocation from the same socket should succeed.
        let alloc = client.allocate(&socket).await.expect("re-allocate after release");
        assert!(alloc.lifetime_secs > 0);

        handle.abort();
    }

    #[tokio::test]
    async fn turn_client_refresh_without_prior_allocate() {
        let (handle, server_addr) = start_test_turn_server().await;
        let client = TurnClient::new(server_addr, turn_test_creds());
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        // Server accepts refresh even without an active allocation.
        let lifetime = client.refresh(&socket, 300).await.expect("refresh");
        assert_eq!(lifetime, 300);

        handle.abort();
    }

    #[tokio::test]
    async fn turn_client_server_addr_accessor() {
        let addr: SocketAddr = "203.0.113.1:3478".parse().unwrap();
        let client = TurnClient::new(addr, turn_test_creds());
        assert_eq!(client.server_addr(), addr);
    }

    #[tokio::test]
    async fn turn_client_full_permission_and_channel_flow() {
        let (handle, server_addr) = start_test_turn_server().await;
        let client = TurnClient::new(server_addr, turn_test_creds());
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        let alloc = client.allocate(&socket).await.expect("allocate");
        assert!(alloc.relay_addr.port() > 0);

        let peer: SocketAddr = "127.0.0.1:4444".parse().unwrap();
        client.create_permission(&socket, peer).await.expect("permission");
        client.channel_bind(&socket, 0x4002, peer).await.expect("bind");

        handle.abort();
    }
}
