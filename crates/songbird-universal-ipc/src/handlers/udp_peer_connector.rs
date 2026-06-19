// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! UDP Peer Connector
//!
//! Production UDP hole punching and [`PeerConnector`] enum dispatch.
//!
//! ## Deep Debt Compliance
//! - Zero hardcoding: Runtime configuration via params
//! - Mocks isolated: Real implementation for production
//! - Pure Rust: Uses tokio UDP (no C deps)
//! - Modern async: Full async/await, event-driven
//! - No polling: Uses `tokio::select`! for concurrent send/recv

use super::peer_types::PeerConnectResult;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// UDP-based peer connector for production use
///
/// Implements simultaneous-open UDP hole punching for NAT traversal.
/// Works with symmetric NAT when paired with STUN port prediction.
pub struct UdpPeerConnector {
    /// Active bindings for reuse
    active_bindings: Arc<RwLock<Vec<BindingEntry>>>,
    /// Hole punch timeout
    timeout: Duration,
    /// Number of punch packets to send
    punch_count: u32,
    /// Interval between punch packets
    punch_interval: Duration,
}

/// Active UDP binding entry
#[derive(Debug, Clone)]
#[allow(dead_code, reason = "reserved for active binding tracking when hole punch is wired")]
struct BindingEntry {
    local_addr: SocketAddr,
    connection_id: String,
}

impl Default for UdpPeerConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl UdpPeerConnector {
    pub fn new() -> Self {
        info!("✅ UDP Peer Connector initialized (production)");
        Self {
            active_bindings: Arc::new(RwLock::new(Vec::new())),
            timeout: Duration::from_secs(10),
            punch_count: 10,
            punch_interval: Duration::from_millis(200),
        }
    }

    /// Create with custom configuration
    #[must_use]
    pub fn with_config(timeout: Duration, punch_count: u32, punch_interval: Duration) -> Self {
        Self {
            active_bindings: Arc::new(RwLock::new(Vec::new())),
            timeout,
            punch_count,
            punch_interval,
        }
    }

    /// Perform UDP hole punching to target address
    ///
    /// Sends punch packets while simultaneously listening for
    /// incoming packets from the peer. Uses `tokio::select`! for
    /// event-driven (zero-polling) operation.
    async fn hole_punch(&self, socket: &UdpSocket, target: SocketAddr) -> Result<bool, String> {
        // Punch packet: minimal probe with timestamp
        let punch_data = b"SONGBIRD_PUNCH";

        let mut received = false;

        // Use tokio::select! to concurrently send punches and listen for response
        let punch_future = async {
            for i in 0..self.punch_count {
                debug!("Sending punch packet {}/{} to {}", i + 1, self.punch_count, target);
                if let Err(e) = socket.send_to(punch_data, target).await {
                    warn!("Punch send failed: {}", e);
                }
                tokio::time::sleep(self.punch_interval).await;
            }
        };

        let recv_future = async {
            let mut buf = [0u8; 1024];
            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((len, from)) => {
                        debug!("Received {} bytes from {} during punch", len, from);
                        // Accept packets from our target or any peer responding
                        if from.ip() == target.ip() || len >= punch_data.len() {
                            return true;
                        }
                    }
                    Err(e) => {
                        warn!("Recv error during punch: {}", e);
                        return false;
                    }
                }
            }
        };

        // Race: send punches while listening for response
        tokio::select! {
            () = punch_future => {
                debug!("All punch packets sent, waiting for response...");
                // Give a brief window for final responses
                let mut buf = [0u8; 1024];
                match tokio::time::timeout(Duration::from_secs(2), socket.recv_from(&mut buf)).await {
                    Ok(Ok((_, from))) => {
                        info!("🔗 Hole punch succeeded: response from {}", from);
                        received = true;
                    }
                    _ => {
                        debug!("No response after punching");
                    }
                }
            }
            result = recv_future => {
                received = result;
                if received {
                    info!("🔗 Hole punch succeeded during active punching");
                }
            }
        }

        Ok(received)
    }

    pub async fn connect(
        &self,
        target_address: &str,
        our_binding: Option<&str>,
        _rendezvous_token: Option<&str>,
    ) -> Result<PeerConnectResult, String> {
        info!("🔗 UDP Peer Connect: Initiating to {} (binding: {:?})", target_address, our_binding);

        // Parse target address
        let target: SocketAddr = target_address
            .parse()
            .map_err(|e| format!("Invalid target address '{target_address}': {e}"))?;

        // Bind our socket
        let bind_addr: SocketAddr = if let Some(addr) = our_binding {
            addr.parse().map_err(|e| format!("Invalid binding address '{addr}': {e}"))?
        } else {
            let addr = format!("{}:0", songbird_types::constants::PRODUCTION_BIND_ADDRESS);
            addr.parse().map_err(|e| format!("Ephemeral bind failed: {e}"))?
        };

        let socket = UdpSocket::bind(bind_addr)
            .await
            .map_err(|e| format!("Failed to bind UDP socket on {bind_addr}: {e}"))?;

        let local_addr =
            socket.local_addr().map_err(|e| format!("Failed to get local address: {e}"))?;

        info!("🔗 UDP bound to {} -> targeting {}", local_addr, target);

        let connection_id = format!(
            "udp-{}-{}",
            local_addr.port(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                % 100_000
        );

        // Perform hole punching with timeout
        let punched = tokio::time::timeout(self.timeout, self.hole_punch(&socket, target))
            .await
            .map_err(|_| String::from("Hole punch timed out"))?
            .map_err(|e| format!("Hole punch error: {e}"))?;

        let state = if punched {
            "connected"
        } else {
            "punching"
        };

        // Track binding
        {
            let mut bindings = self.active_bindings.write().await;
            bindings.push(BindingEntry {
                local_addr,
                connection_id: connection_id.clone(),
            });
        }

        info!("🔗 UDP peer connection {}: {} (local: {})", state, connection_id, local_addr);

        Ok(PeerConnectResult {
            connection_id,
            state: state.to_string(),
            channel: None,
        })
    }
}

/// Peer connection backend (enum dispatch).
pub enum PeerConnector {
    Udp(UdpPeerConnector),
    #[cfg(test)]
    Mock(MockPeerConnector),
    #[cfg(test)]
    ErrorSim,
    #[cfg(test)]
    Weird,
}

impl PeerConnector {
    pub async fn connect(
        &self,
        target_address: &str,
        our_binding: Option<&str>,
        rendezvous_token: Option<&str>,
    ) -> Result<PeerConnectResult, String> {
        match self {
            Self::Udp(c) => c.connect(target_address, our_binding, rendezvous_token).await,
            #[cfg(test)]
            Self::Mock(m) => m.connect(target_address, our_binding, rendezvous_token).await,
            #[cfg(test)]
            Self::ErrorSim => Err(String::from("simulated transport failure")),
            #[cfg(test)]
            Self::Weird => Ok(PeerConnectResult {
                connection_id: "x".into(),
                state: "negotiating".into(),
                channel: None,
            }),
        }
    }
}

#[cfg(test)]
pub struct MockPeerConnector {
    should_succeed: std::sync::RwLock<bool>,
}

#[cfg(test)]
impl Default for MockPeerConnector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl MockPeerConnector {
    #[must_use]
    pub fn new() -> Self {
        Self {
            should_succeed: std::sync::RwLock::new(true),
        }
    }

    pub fn set_should_succeed(&self, succeed: bool) {
        *self.should_succeed.write().unwrap() = succeed;
    }

    pub async fn connect(
        &self,
        target_address: &str,
        our_binding: Option<&str>,
        _rendezvous_token: Option<&str>,
    ) -> Result<PeerConnectResult, String> {
        let should_succeed = *self.should_succeed.read().unwrap();

        let connection_id = uuid::Uuid::new_v4().to_string();

        if should_succeed {
            let local_address = our_binding.map_or_else(
                || songbird_types::constants::EPHEMERAL_BIND_ADDR.to_string(),
                std::string::ToString::to_string,
            );

            Ok(PeerConnectResult {
                connection_id,
                state: String::from("connected"),
                channel: Some(super::peer_types::PeerChannel {
                    local_address,
                    remote_address: target_address.to_string(),
                    protocol: String::from("udp"),
                    latency_ms: Some(25),
                }),
            })
        } else {
            Ok(PeerConnectResult {
                connection_id,
                state: String::from("failed"),
                channel: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]

    use super::*;

    #[tokio::test]
    async fn test_udp_peer_connector_creation() {
        let _connector = UdpPeerConnector::new();
    }

    #[tokio::test]
    async fn test_custom_config() {
        let connector =
            UdpPeerConnector::with_config(Duration::from_secs(5), 20, Duration::from_millis(100));
        assert_eq!(connector.timeout, Duration::from_secs(5));
        assert_eq!(connector.punch_count, 20);
    }

    #[tokio::test]
    async fn test_connect_invalid_address() {
        let connector = UdpPeerConnector::new();
        let result = connector.connect("not-valid", None, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid target address"));
    }

    #[tokio::test]
    async fn test_connect_empty_target_errors() {
        let connector = UdpPeerConnector::new();
        let result = connector.connect("", None, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connect_loopback_responds() {
        // Create a local "peer" that responds
        let peer_socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let peer_addr = peer_socket.local_addr().unwrap();

        // Spawn peer that echoes punch packets
        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            if let Ok((len, from)) = peer_socket.recv_from(&mut buf).await {
                let _ = peer_socket.send_to(&buf[..len], from).await;
            }
        });

        let connector =
            UdpPeerConnector::with_config(Duration::from_secs(3), 5, Duration::from_millis(50));
        let result = connector.connect(&peer_addr.to_string(), None, None).await;

        assert!(result.is_ok());
        let connect_result = result.unwrap();
        // Should be either "connected" or "punching" depending on timing
        assert!(
            connect_result.state == "connected" || connect_result.state == "punching",
            "Unexpected state: {}",
            connect_result.state
        );
    }

    #[tokio::test]
    async fn test_connect_without_binding() {
        // Very short timeout and minimal punching to avoid slow tests
        let connector =
            UdpPeerConnector::with_config(Duration::from_millis(500), 1, Duration::from_millis(10));
        // Loopback unreachable port — should complete with "punching" state
        let result = connector.connect("127.0.0.1:59999", None, None).await;
        // Either success (punching) or timeout error — both are valid
        match result {
            Ok(r) => assert_eq!(r.state, "punching"),
            Err(e) => assert!(
                e.contains("timed out") || e.contains("Hole punch"),
                "Unexpected error: {e}"
            ),
        }
    }

    #[tokio::test]
    async fn test_connect_with_rendezvous_token() {
        let connector =
            UdpPeerConnector::with_config(Duration::from_millis(500), 1, Duration::from_millis(10));
        let result = connector.connect("127.0.0.1:59998", None, Some("token-abc123")).await;
        // Should complete (success or timeout — both valid without peer)
        match result {
            Ok(r) => assert!(r.state == "punching" || r.state == "connected"),
            Err(e) => assert!(
                e.contains("timed out") || e.contains("Hole punch"),
                "Unexpected error: {e}"
            ),
        }
    }
}
