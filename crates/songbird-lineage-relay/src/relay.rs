// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Relay discovery and session management
//!
//! Evolution beyond TURN: Ancestors relay for descendants

use crate::birdsong::BirdSongBroadcaster;
use crate::error::{LineageRelayError, Result};
use crate::relay_protocol::RelayProtocol;
use crate::types::{BirdSongType, LineageHint, MaskingLevel, NodeId, RelayAuthorization};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

use tracing::{debug, info, warn};

/// Relay authorization dispatch (security provider + mocks + test harnesses).
#[derive(Clone, Debug)]
pub enum RelayAuthority {
    /// Production: security-provider JSON-RPC.
    Security(crate::security::SecurityRelayAuthority),
    /// Mock lineage graph (`test-utils` / unit tests).
    #[cfg(any(test, feature = "test-utils"))]
    Mock(crate::security::MockRelayAuthority),
    /// Harness: always authorizes with `MaskingLevel::None` for masking stubs.
    StubAllow,
    /// Harness: always denies authorization.
    StubDeny,
}

impl RelayAuthority {
    /// Authorize relay service for requester.
    pub async fn authorize_relay(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<RelayAuthorization> {
        match self {
            Self::Security(a) => a.authorize_relay(relay_node, requester).await,
            #[cfg(any(test, feature = "test-utils"))]
            Self::Mock(m) => m.authorize_relay(relay_node, requester).await,
            Self::StubAllow => Ok(RelayAuthorization::authorized(
                relay_node.clone(),
                requester.clone(),
                MaskingLevel::None,
                300,
            )),
            Self::StubDeny => {
                Ok(RelayAuthorization::unauthorized(relay_node.clone(), requester.clone()))
            }
        }
    }

    /// Resolve masking tier for this relay relationship.
    pub async fn determine_masking(
        &self,
        relay_node: &NodeId,
        requester: &NodeId,
    ) -> Result<MaskingLevel> {
        match self {
            Self::Security(a) => a.determine_masking(relay_node, requester).await,
            #[cfg(any(test, feature = "test-utils"))]
            Self::Mock(m) => m.determine_masking(relay_node, requester).await,
            Self::StubAllow => Ok(MaskingLevel::None),
            Self::StubDeny => Ok(MaskingLevel::Full),
        }
    }
}

impl From<crate::security::SecurityRelayAuthority> for RelayAuthority {
    fn from(value: crate::security::SecurityRelayAuthority) -> Self {
        Self::Security(value)
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl From<crate::security::MockRelayAuthority> for RelayAuthority {
    fn from(value: crate::security::MockRelayAuthority) -> Self {
        Self::Mock(value)
    }
}

/// Broadcast or unicast ask for an ancestor to relay toward [`target`](Self::target).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayRequest {
    /// Node initiating the relay (usually the descendant).
    pub requester: NodeId,
    /// Intended peer on the far side of the relay path.
    pub target: NodeId,
    /// Last known direct address for `target`, if any.
    pub target_address: Option<SocketAddr>,
    /// Wall-clock time the request was formed (replay protection).
    pub timestamp: SystemTime,
}

/// Positive reply assigning a relay hop and [`RelayAuthorization`] for the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayOffer {
    /// Ancestor or designated relay node id.
    pub relay_node: NodeId,
    /// UDP endpoint where the relay accepts tunneled packets.
    pub relay_address: SocketAddr,
    /// Signed capability to use this relay with a given masking tier.
    pub authorization: RelayAuthorization,
    /// When this offer was issued.
    pub timestamp: SystemTime,
}

/// Active relay session
///
/// **Pure Rust | Zero Unsafe | Complete Implementation**
#[derive(Debug)]
pub struct RelaySession {
    /// Wire identifier echoed in [`crate::relay_protocol::RelayProtocol`] frames.
    pub session_id: uuid::Uuid,
    /// Relay hop identity (matches [`RelayOffer::relay_node`](RelayOffer::relay_node)).
    pub relay_node: NodeId,
    /// Connected relay UDP endpoint.
    pub relay_address: SocketAddr,
    /// Local party in the relayed path.
    pub requester: NodeId,
    /// Remote party id being reached through the relay.
    pub target: NodeId,
    /// Privacy tier negotiated with [`RelayAuthority::determine_masking`](RelayAuthority::determine_masking).
    pub masking_level: MaskingLevel,
    /// When the session became active.
    pub established_at: SystemTime,
    /// Monotonic counter of payload bytes forwarded through this socket.
    pub bytes_relayed: Arc<AtomicU64>,
    /// UDP socket for relay communication (created at bind time)
    socket: Arc<UdpSocket>,
}

impl RelaySession {
    /// Create new relay session
    ///
    /// # Errors
    ///
    /// Returns error if UDP socket binding fails.
    pub async fn new(
        relay_node: NodeId,
        relay_address: SocketAddr,
        requester: NodeId,
        target: NodeId,
        masking_level: MaskingLevel,
    ) -> Result<Self> {
        // Bind to ephemeral port (OS-assigned)
        let socket = UdpSocket::bind("0.0.0.0:0").await.map_err(|e| {
            LineageRelayError::NetworkError(format!(
                "Failed to bind UDP socket for relay session: {e}"
            ))
        })?;

        // Connect to relay address (sets default destination)
        socket.connect(relay_address).await.map_err(|e| {
            LineageRelayError::NetworkError(format!("Failed to connect to relay server: {e}"))
        })?;

        Ok(Self {
            session_id: uuid::Uuid::new_v4(),
            relay_node,
            relay_address,
            requester,
            target,
            masking_level,
            established_at: SystemTime::now(),
            bytes_relayed: Arc::new(AtomicU64::new(0)),
            socket: Arc::new(socket),
        })
    }

    /// Send data through relay (forwards over UDP and updates byte counter).
    ///
    /// # Errors
    ///
    /// Returns error if network send fails.
    pub async fn send(&self, data: &[u8]) -> Result<()> {
        debug!(
            "📤 Sending {} bytes through relay {} (session: {}, masked: {:?})",
            data.len(),
            self.relay_node,
            self.session_id,
            self.masking_level
        );

        // Wrap data in relay protocol
        let packet = RelayProtocol::DataPacket {
            session_id: self.session_id,
            data: data.to_vec(),
        };

        // Encode to wire format
        let encoded = packet.encode();

        // Send to relay server via UDP
        self.socket.send(&encoded).await.map_err(|e| {
            LineageRelayError::NetworkError(format!("Failed to send data through relay: {e}"))
        })?;

        let total =
            self.bytes_relayed.fetch_add(data.len() as u64, Ordering::Relaxed) + data.len() as u64;

        info!(
            "✅ Forwarded {} bytes through relay {} (total: {} bytes)",
            data.len(),
            self.relay_node,
            total
        );

        Ok(())
    }

    /// Refresh session (extend TTL)
    ///
    /// # Errors
    ///
    /// Returns error if network send fails.
    pub async fn refresh(&self) -> Result<()> {
        debug!("🔄 Refreshing relay session {}", self.session_id);

        let refresh_msg = RelayProtocol::Refresh {
            session_id: self.session_id,
        };

        let encoded = refresh_msg.encode();
        self.socket.send(&encoded).await.map_err(|e| {
            LineageRelayError::NetworkError(format!("Failed to refresh relay session: {e}"))
        })?;

        Ok(())
    }

    /// Close relay session
    ///
    /// # Errors
    ///
    /// Returns error if network send fails.
    pub async fn close(&self) -> Result<()> {
        info!("🛑 Closing relay session {}", self.session_id);

        let deallocate_msg = RelayProtocol::Deallocate {
            session_id: self.session_id,
        };

        let encoded = deallocate_msg.encode();
        self.socket.send(&encoded).await.map_err(|e| {
            LineageRelayError::NetworkError(format!("Failed to close relay session: {e}"))
        })?;

        Ok(())
    }

    /// Get relay statistics (total payload bytes sent on this session).
    #[must_use]
    pub fn stats(&self) -> u64 {
        self.bytes_relayed.load(Ordering::Relaxed)
    }
}

/// Relay discovery system
pub struct RelayDiscovery {
    broadcaster: Arc<BirdSongBroadcaster>,
    relay_authority: Arc<RelayAuthority>,
    my_id: NodeId,
    active_sessions: Arc<RwLock<Vec<Arc<RelaySession>>>>,
}

impl RelayDiscovery {
    /// Create new relay discovery
    #[must_use]
    pub fn new(
        broadcaster: Arc<BirdSongBroadcaster>,
        relay_authority: Arc<RelayAuthority>,
        my_id: NodeId,
    ) -> Self {
        Self {
            broadcaster,
            relay_authority,
            my_id,
            active_sessions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Request relay for target peer
    ///
    /// # Errors
    ///
    /// Returns error if no relay is available
    pub async fn request_relay(
        &self,
        target: NodeId,
        target_address: Option<SocketAddr>,
    ) -> Result<Arc<RelaySession>> {
        info!("Requesting relay for target: {}", target);

        // Create relay request
        let request = RelayRequest {
            requester: self.my_id.clone(),
            target: target.clone(),
            target_address,
            timestamp: SystemTime::now(),
        };

        // Serialize request
        let payload = serde_json::to_vec(&request)?;

        // Broadcast to ancestors (only they can decrypt)
        self.broadcaster
            .broadcast(BirdSongType::RelayRequest, &payload, LineageHint::DirectAncestors)
            .await?;

        debug!("Relay request broadcast, waiting for offers...");

        // Wait for relay offers
        let offer = self.wait_for_relay_offer(Duration::from_secs(5)).await?;

        // Create relay session
        let session = Arc::new(
            RelaySession::new(
                offer.relay_node,
                offer.relay_address,
                self.my_id.clone(),
                target,
                offer.authorization.masking_level,
            )
            .await?,
        );

        // Store active session
        self.active_sessions.write().await.push(session.clone());

        info!("Relay session established with {}", session.relay_node);

        Ok(session)
    }

    /// Wait for relay offer (from ancestors)
    ///
    /// 🚨 DEEP DEBT (v3.10.4 - Jan 6, 2026): Polling loop with sleep
    ///
    /// CURRENT: Polls broadcaster every 100ms (blocking, wasteful, introduces latency)
    /// SHOULD BE: Event-driven with watch channel or notification
    ///
    /// Modern Rust Solution:
    /// ```rust,ignore
    /// // In broadcaster: notify on new messages
    /// let (offer_tx, mut offer_rx) = tokio::sync::watch::channel(None);
    ///
    /// // Producer notifies
    /// offer_tx.send(Some(offer))?;
    ///
    /// // Consumer waits (instant notification, zero latency)
    /// timeout(duration, async {
    ///     offer_rx.changed().await?;
    ///     Ok(offer_rx.borrow().clone().unwrap())
    /// }).await?
    /// ```
    ///
    /// Benefits:
    /// - Zero latency (instant notification vs 100ms polling)
    /// - No CPU waste (event-driven, not busy-waiting)
    /// - Cleaner code (no manual polling logic)
    ///
    /// Alternative: Make `broadcaster.get_messages()` await-able (blocking call)
    ///
    /// Status: INCOMPLETE - Requires broadcaster architectural changes
    /// Priority: MEDIUM (relay functionality is experimental)
    async fn wait_for_relay_offer(&self, duration: Duration) -> Result<RelayOffer> {
        // ✅ Event-driven: uses Notify-based wakeup (zero polling, instant latency)
        let messages =
            self.broadcaster.wait_for_message_by_type(BirdSongType::RelayOffer, duration).await?;

        for msg in messages {
            if let Ok(offer) = serde_json::from_slice::<RelayOffer>(&msg.payload)
                && offer.authorization.authorized
            {
                return Ok(offer);
            }
        }

        Err(LineageRelayError::NoRelayAvailable("No authorized relay offers received".to_string()))
    }

    /// Offer relay service (as ancestor)
    ///
    /// # Errors
    ///
    /// Returns error if authorization fails
    pub async fn offer_relay(
        &self,
        request: RelayRequest,
        my_relay_address: SocketAddr,
    ) -> Result<()> {
        info!("Considering relay offer for {}", request.requester);

        // Verify authorization through security provider
        let authorization =
            self.relay_authority.authorize_relay(&self.my_id, &request.requester).await?;

        if !authorization.authorized {
            warn!("Relay denied for {}", request.requester);
            return Err(LineageRelayError::RelayDenied(
                "Not authorized to relay for this node".to_string(),
            ));
        }

        // Create relay offer
        let offer = RelayOffer {
            relay_node: self.my_id.clone(),
            relay_address: my_relay_address,
            authorization,
            timestamp: SystemTime::now(),
        };

        // Serialize offer
        let payload = serde_json::to_vec(&offer)?;

        // Broadcast offer (encrypted for requester)
        let requester_id = request.requester.clone();
        self.broadcaster
            .broadcast(
                BirdSongType::RelayOffer,
                &payload,
                LineageHint::SpecificAncestor(request.requester),
            )
            .await?;

        info!("Relay offer sent to {}", requester_id);

        Ok(())
    }

    /// Get active relay sessions
    pub async fn active_sessions(&self) -> Vec<Arc<RelaySession>> {
        self.active_sessions.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::birdsong::BirdSongBroadcaster;
    use crate::error::LineageRelayError;
    use crate::security::BirdSongCrypto;
    use serde_json::{from_value, to_value};

    #[tokio::test]
    async fn test_relay_session_creation() {
        // Bind a server first to have a valid address to connect to
        let server_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server_socket.local_addr().unwrap();

        let session = RelaySession::new(
            NodeId::from("relay-1"),
            server_addr,
            NodeId::from("requester"),
            NodeId::from("target"),
            MaskingLevel::Masked,
        )
        .await
        .unwrap();

        assert_eq!(session.relay_node.0, "relay-1");
        assert_eq!(session.masking_level, MaskingLevel::Masked);
    }

    #[tokio::test]
    async fn test_relay_session_send() {
        // Bind a server first
        let server_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server_socket.local_addr().unwrap();

        let session = RelaySession::new(
            NodeId::from("relay-1"),
            server_addr,
            NodeId::from("requester"),
            NodeId::from("target"),
            MaskingLevel::Masked,
        )
        .await
        .unwrap();

        // Send will succeed (UDP is connectionless, doesn't fail on send)
        session.send(b"test data").await.unwrap();
        assert_eq!(session.stats(), 9);
    }

    #[tokio::test]
    async fn test_mock_relay_authorization() {
        let authority = RelayAuthority::StubAllow;
        let auth = authority
            .authorize_relay(&NodeId::from("relay"), &NodeId::from("requester"))
            .await
            .unwrap();

        assert!(auth.authorized);
        assert_eq!(auth.masking_level, MaskingLevel::None);
    }

    #[test]
    fn relay_request_serde_roundtrip() {
        let r = RelayRequest {
            requester: NodeId::from("req"),
            target: NodeId::from("tgt"),
            target_address: Some("127.0.0.1:1".parse().unwrap()),
            timestamp: SystemTime::UNIX_EPOCH,
        };
        let v = to_value(&r).unwrap();
        let back: RelayRequest = from_value(v).unwrap();
        assert_eq!(r.requester, back.requester);
        assert_eq!(r.target, back.target);
        assert_eq!(r.target_address, back.target_address);
    }

    #[test]
    fn relay_offer_serde_roundtrip() {
        let offer = RelayOffer {
            relay_node: NodeId::from("relay"),
            relay_address: "192.0.2.1:3478".parse().unwrap(),
            authorization: RelayAuthorization::authorized(
                NodeId::from("relay"),
                NodeId::from("req"),
                MaskingLevel::Full,
                120,
            ),
            timestamp: SystemTime::UNIX_EPOCH,
        };
        let v = to_value(&offer).unwrap();
        let back: RelayOffer = from_value(v).unwrap();
        assert_eq!(offer.relay_node, back.relay_node);
        assert_eq!(offer.relay_address, back.relay_address);
    }

    #[test]
    fn relay_offer_optional_fields_none() {
        let r = RelayRequest {
            requester: NodeId::from("a"),
            target: NodeId::from("b"),
            target_address: None,
            timestamp: SystemTime::now(),
        };
        let js = serde_json::to_string(&r).unwrap();
        assert!(js.contains("null") || js.contains("target_address"));
    }

    #[test]
    fn relay_session_stats_zero_before_send() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let server_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let server_addr = server_socket.local_addr().unwrap();
            let session = RelaySession::new(
                NodeId::from("relay-1"),
                server_addr,
                NodeId::from("requester"),
                NodeId::from("target"),
                MaskingLevel::Masked,
            )
            .await
            .unwrap();
            assert_eq!(session.stats(), 0);
        });
    }

    #[tokio::test]
    async fn offer_relay_denied_when_authority_rejects() {
        use std::sync::Arc;

        let broadcaster = Arc::new(
            BirdSongBroadcaster::new(
                Arc::new(BirdSongCrypto::StubPassthrough),
                NodeId::from("ancestor"),
                "127.0.0.1:0".parse().unwrap(),
                "127.0.0.1:2".parse().unwrap(),
            )
            .await
            .unwrap(),
        );
        let discovery = RelayDiscovery::new(
            broadcaster,
            Arc::new(RelayAuthority::StubDeny),
            NodeId::from("ancestor"),
        );
        let req = RelayRequest {
            requester: NodeId::from("child"),
            target: NodeId::from("peer"),
            target_address: None,
            timestamp: SystemTime::UNIX_EPOCH,
        };
        let err =
            discovery.offer_relay(req, "127.0.0.1:9".parse().unwrap()).await.expect_err("denied");
        assert!(
            matches!(err, LineageRelayError::RelayDenied(_)),
            "expected RelayDenied, got {err:?}"
        );
    }

    #[tokio::test]
    async fn relay_session_refresh_and_close_sends_wire_messages() {
        let server_socket = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let server_addr = server_socket.local_addr().unwrap();
        let session = RelaySession::new(
            NodeId::from("relay-1"),
            server_addr,
            NodeId::from("requester"),
            NodeId::from("target"),
            MaskingLevel::Masked,
        )
        .await
        .unwrap();
        session.refresh().await.unwrap();
        session.close().await.unwrap();
        let mut buf = [0u8; 64];
        let (n1, _) = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            server_socket.recv_from(&mut buf),
        )
        .await
        .expect("timeout")
        .expect("recv 1");
        assert_eq!(buf[0], 0x20);
        let (n2, _) = tokio::time::timeout(
            std::time::Duration::from_millis(200),
            server_socket.recv_from(&mut buf),
        )
        .await
        .expect("timeout")
        .expect("recv 2");
        assert_eq!(buf[0], 0x30);
        assert!(n1 >= 17);
        assert!(n2 >= 17);
    }
}
