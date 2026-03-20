// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Hole Punch Coordinator
//!
//! Coordinates NAT traversal using signaling channel (Tor, WebSocket, etc.)
//!
//! ## Algorithm
//!
//! 1. Both peers register with rendezvous, sharing STUN-discovered addresses
//! 2. Initiator sends PunchRequest with nonce
//! 3. Responder sends PunchAck with coordinated start time
//! 4. Both start sending UDP packets at the same time
//! 5. First to receive reports PunchResult
//! 6. If failed, fall back to relay mode

use crate::error::{OnionRelayError, Result};
use crate::signaling::{NatType, PeerInfo, SignalingMessage};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::{RwLock, mpsc};
use tokio::time::{sleep, timeout};
use tracing::{debug, info, warn};

/// Configuration for hole punch attempts
///
/// ## STUN Server Configuration
///
/// STUN servers are resolved in this order:
/// 1. Explicitly configured via `stun_servers` field
/// 2. Environment variable `BIOMEOS_STUN_SERVERS` (comma-separated)
/// 3. Self-hosted via `BIOMEOS_STUN_SERVER` environment variable
/// 4. Public STUN servers (default fallback)
///
/// For sovereign operation, configure self-hosted STUN:
/// ```bash
/// export BIOMEOS_STUN_SERVER="my-stun.local:3478"
/// ```
#[derive(Debug, Clone)]
pub struct HolePunchConfig {
    /// Number of simultaneous punch attempts
    pub max_attempts: u32,
    /// Timeout for each attempt
    pub attempt_timeout: Duration,
    /// Delay between punch packets
    pub packet_interval: Duration,
    /// Total timeout for punch coordination
    pub total_timeout: Duration,
    /// STUN servers to use for address discovery (resolved from env or defaults)
    pub stun_servers: Vec<String>,
    /// Timeout waiting for punch ack from peer
    pub ack_timeout: Duration,
}

impl Default for HolePunchConfig {
    fn default() -> Self {
        Self {
            max_attempts: 20,
            attempt_timeout: Duration::from_millis(500),
            packet_interval: Duration::from_millis(50),
            total_timeout: Duration::from_secs(10),
            stun_servers: Self::resolve_stun_servers(),
            ack_timeout: Duration::from_secs(5),
        }
    }
}

impl HolePunchConfig {
    /// Create with custom STUN servers
    pub fn with_stun_servers(mut self, servers: Vec<String>) -> Self {
        self.stun_servers = servers;
        self
    }

    /// Resolve STUN servers from environment or defaults
    ///
    /// Resolution order:
    /// 1. BIOMEOS_STUN_SERVERS (comma-separated)
    /// 2. BIOMEOS_STUN_SERVER (single self-hosted)
    /// 3. Default public servers
    fn resolve_stun_servers() -> Vec<String> {
        let mut servers = Vec::new();

        // 1. Self-hosted first (highest priority, maximum sovereignty)
        if let Ok(self_hosted) = std::env::var("BIOMEOS_STUN_SERVER") {
            servers.push(self_hosted);
        }

        // 2. Custom servers from env
        if let Ok(custom) = std::env::var("BIOMEOS_STUN_SERVERS") {
            servers
                .extend(custom.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
        }

        // 3. Public fallback (only if no custom servers)
        if servers.is_empty() && std::env::var("BIOMEOS_NO_PUBLIC_STUN").is_err() {
            servers.extend(Self::default_public_stun_servers());
        }

        servers
    }

    /// Default public STUN servers (fallback only)
    fn default_public_stun_servers() -> Vec<String> {
        vec![
            "stun.l.google.com:19302".to_string(),
            "stun1.l.google.com:19302".to_string(),
            "stun.cloudflare.com:3478".to_string(),
        ]
    }
}

/// Result of hole punch attempt
#[derive(Debug, Clone)]
pub enum PunchResult {
    /// Direct connection established
    Direct {
        peer_addr: SocketAddr,
        local_socket: Arc<UdpSocket>,
        latency: Duration,
    },
    /// Must use relay (hole punch failed)
    Relay {
        attempts: u32,
    },
}

/// Hole punch coordinator
///
/// Manages the hole punch process using a signaling channel
pub struct HolePunchCoordinator {
    /// Our node ID
    my_node_id: String,
    /// Our STUN-discovered info
    my_info: RwLock<Option<PeerInfo>>,
    /// Known peers from signaling
    peers: RwLock<HashMap<String, PeerInfo>>,
    /// Configuration
    config: HolePunchConfig,
    /// Channel to send signaling messages
    signal_tx: mpsc::Sender<SignalingMessage>,
    /// Channel to receive signaling messages
    signal_rx: RwLock<Option<mpsc::Receiver<SignalingMessage>>>,
}

impl HolePunchCoordinator {
    /// Create new coordinator
    pub fn new(
        my_node_id: String,
        config: HolePunchConfig,
    ) -> (Self, mpsc::Sender<SignalingMessage>, mpsc::Receiver<SignalingMessage>) {
        let (outbound_tx, outbound_rx) = mpsc::channel(100);
        let (inbound_tx, inbound_rx) = mpsc::channel(100);

        let coordinator = Self {
            my_node_id,
            my_info: RwLock::new(None),
            peers: RwLock::new(HashMap::new()),
            config,
            signal_tx: outbound_tx,
            signal_rx: RwLock::new(Some(inbound_rx)),
        };

        (coordinator, inbound_tx, outbound_rx)
    }

    /// Discover our public address via STUN
    pub async fn discover_public_address(&self) -> Result<PeerInfo> {
        info!("🔍 Discovering public address via STUN...");

        // Bind local socket
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let local_addr = socket.local_addr()?;

        // Try each STUN server
        for stun_server in &self.config.stun_servers {
            match self.stun_bind(&socket, stun_server).await {
                Ok(public_addr) => {
                    info!("✅ Discovered public address: {}", public_addr);

                    // Detect NAT type by checking if port varies
                    let nat_type = self.detect_nat_type(&socket).await;

                    let info = PeerInfo {
                        node_id: self.my_node_id.clone(),
                        public_addr,
                        local_addr: Some(local_addr),
                        nat_type,
                        timestamp: SystemTime::now(),
                        capabilities: vec!["relay".to_string()],
                    };

                    *self.my_info.write().await = Some(info.clone());
                    return Ok(info);
                }
                Err(e) => {
                    warn!("⚠️ STUN {} failed: {}", stun_server, e);
                }
            }
        }

        Err(OnionRelayError::StunFailed("All STUN servers failed".to_string()))
    }

    /// Attempt hole punch to peer
    pub async fn punch_to_peer(&self, peer_node_id: &str) -> Result<PunchResult> {
        info!("🥊 Initiating hole punch to {}", peer_node_id);

        // Get our info
        let my_info = self.my_info.read().await.clone().ok_or_else(|| {
            OnionRelayError::Other("Must discover public address first".to_string())
        })?;

        // Get peer info
        let peer_info = self
            .peers
            .read()
            .await
            .get(peer_node_id)
            .cloned()
            .ok_or_else(|| OnionRelayError::PeerNotFound(peer_node_id.to_string()))?;

        // Generate nonce for this attempt
        let nonce: [u8; 16] = rand_nonce();

        // Send punch request via signaling
        let request = SignalingMessage::PunchRequest {
            from: my_info.clone(),
            to_node_id: peer_node_id.to_string(),
            nonce,
        };

        self.signal_tx
            .send(request)
            .await
            .map_err(|e| OnionRelayError::Transport(e.to_string()))?;

        // Wait for ack with start time
        let start_time = self.wait_for_punch_ack(&nonce).await?;

        // Create socket for punching
        let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);

        // Wait until coordinated start time
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| OnionRelayError::Other("System time before UNIX epoch".to_string()))?
            .as_millis() as u64;

        if start_time > now_ms {
            sleep(Duration::from_millis(start_time - now_ms)).await;
        }

        // Execute hole punch
        let result = self.execute_punch(socket.clone(), peer_info.public_addr).await;

        // Report result via signaling
        let result_msg = SignalingMessage::PunchResult {
            nonce,
            success: result.is_ok(),
            connected_addr: result.as_ref().ok().map(|_| peer_info.public_addr),
        };
        let _ = self.signal_tx.send(result_msg).await;

        match result {
            Ok(latency) => {
                info!("✅ Hole punch successful! Latency: {:?}", latency);
                Ok(PunchResult::Direct {
                    peer_addr: peer_info.public_addr,
                    local_socket: socket,
                    latency,
                })
            }
            Err(_) => {
                warn!("⚠️ Hole punch failed, falling back to relay");
                Ok(PunchResult::Relay {
                    attempts: self.config.max_attempts,
                })
            }
        }
    }

    /// Register a peer from signaling
    pub async fn register_peer(&self, peer_info: PeerInfo) {
        info!("📝 Registered peer: {} at {}", peer_info.node_id, peer_info.public_addr);
        self.peers.write().await.insert(peer_info.node_id.clone(), peer_info);
    }

    /// Handle incoming signaling message
    pub async fn handle_message(&self, msg: SignalingMessage) -> Option<SignalingMessage> {
        match msg {
            SignalingMessage::Register {
                peer_info,
                ..
            } => {
                self.register_peer(peer_info).await;
                None
            }
            SignalingMessage::Query {
                target_node_id,
            } => {
                let peer_info = self.peers.read().await.get(&target_node_id).cloned();
                Some(SignalingMessage::PeerInfoResponse {
                    peer_info,
                })
            }
            SignalingMessage::PunchRequest {
                from,
                to_node_id,
                nonce,
            } => {
                if to_node_id == self.my_node_id {
                    // We're the target - send ack with coordinated time
                    self.register_peer(from.clone()).await;

                    let my_info = self.my_info.read().await.clone();
                    if let Some(info) = my_info {
                        // Start in 100ms to allow network propagation
                        let start_at_ms = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64
                            + 100;

                        return Some(SignalingMessage::PunchAck {
                            from: info,
                            nonce,
                            start_at_ms,
                        });
                    }
                }
                None
            }
            SignalingMessage::Heartbeat {
                node_id,
            } => {
                // Update timestamp for peer
                if let Some(peer) = self.peers.write().await.get_mut(&node_id) {
                    peer.timestamp = SystemTime::now();
                }
                None
            }
            _ => None,
        }
    }

    // --- Private methods ---

    async fn stun_bind(&self, _socket: &UdpSocket, server: &str) -> Result<SocketAddr> {
        use songbird_stun::StunClient;

        let client = StunClient::new();
        client
            .discover_public_address(server)
            .await
            .map_err(|e| OnionRelayError::StunFailed(e.to_string()))
    }

    async fn detect_nat_type(&self, socket: &UdpSocket) -> NatType {
        // Quick NAT type detection by checking port allocation
        // Full implementation would test against multiple STUN servers

        if self.config.stun_servers.len() < 2 {
            return NatType::Unknown;
        }

        let addr1 = self.stun_bind(socket, &self.config.stun_servers[0]).await;
        let addr2 = self.stun_bind(socket, &self.config.stun_servers[1]).await;

        match (addr1, addr2) {
            (Ok(a1), Ok(a2)) if a1.port() == a2.port() => {
                debug!("NAT type: Same port for different destinations → Cone NAT");
                NatType::PortRestricted // Conservative estimate
            }
            (Ok(a1), Ok(a2)) => {
                debug!(
                    "NAT type: Different ports ({} vs {}) → Symmetric NAT",
                    a1.port(),
                    a2.port()
                );
                NatType::Symmetric
            }
            _ => NatType::Unknown,
        }
    }

    /// Wait for PunchAck from peer via signaling channel
    ///
    /// This is a real implementation that:
    /// 1. Takes the signal_rx receiver
    /// 2. Waits for matching PunchAck with timeout
    /// 3. Returns coordinated start time from peer
    async fn wait_for_punch_ack(&self, nonce: &[u8; 16]) -> Result<u64> {
        // Take the receiver (one-shot per punch attempt)
        let rx = {
            let mut rx_guard = self.signal_rx.write().await;
            rx_guard.take()
        };

        let Some(mut rx) = rx else {
            warn!("⚠️ No signal receiver available - using fallback timing");
            // Fallback: coordinate 100ms in future
            return Ok(SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| OnionRelayError::Other("System time before UNIX epoch".to_string()))?
                .as_millis() as u64
                + 100);
        };

        // Wait for matching PunchAck with timeout
        let result = timeout(self.config.ack_timeout, async {
            while let Some(msg) = rx.recv().await {
                match msg {
                    SignalingMessage::PunchAck {
                        from: _,
                        nonce: ack_nonce,
                        start_at_ms,
                    } => {
                        // Check nonce matches
                        if &ack_nonce == nonce {
                            debug!("✅ Received PunchAck, start at {}ms", start_at_ms);
                            return Ok(start_at_ms);
                        } else {
                            debug!("⚠️ PunchAck nonce mismatch, continuing...");
                        }
                    }
                    other => {
                        // Handle other messages through the coordinator
                        if let Some(response) = self.handle_message(other).await {
                            let _ = self.signal_tx.send(response).await;
                        }
                    }
                }
            }
            Err(OnionRelayError::SignalingTimeout)
        })
        .await;

        // Return receiver for future use
        *self.signal_rx.write().await = Some(rx);

        match result {
            Ok(Ok(start_time)) => Ok(start_time),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                warn!("⚠️ PunchAck timeout after {:?}", self.config.ack_timeout);
                Err(OnionRelayError::SignalingTimeout)
            }
        }
    }

    async fn execute_punch(
        &self,
        socket: Arc<UdpSocket>,
        peer_addr: SocketAddr,
    ) -> Result<Duration> {
        let punch_msg = b"SONGBIRD_PUNCH_V2";
        let mut recv_buf = vec![0u8; 1024];
        let start = Instant::now();

        for attempt in 0..self.config.max_attempts {
            debug!("  Punch attempt {}/{}", attempt + 1, self.config.max_attempts);

            // Send punch packet
            if let Err(e) = socket.send_to(punch_msg, peer_addr).await {
                warn!("  Send failed: {}", e);
                continue;
            }

            // Try to receive with short timeout
            match timeout(self.config.attempt_timeout, socket.recv_from(&mut recv_buf)).await {
                Ok(Ok((len, from_addr))) => {
                    if from_addr.ip() == peer_addr.ip() {
                        debug!("  Received {} bytes from {}", len, from_addr);
                        return Ok(start.elapsed());
                    }
                }
                Ok(Err(e)) => {
                    debug!("  Recv error: {}", e);
                }
                Err(_) => {
                    // Timeout - expected, continue trying
                }
            }

            sleep(self.config.packet_interval).await;
        }

        Err(OnionRelayError::HolePunchFailed {
            attempts: self.config.max_attempts,
        })
    }
}

/// Result of a relay-assisted coordinated punch attempt
#[derive(Debug, Clone)]
pub enum CoordinatedPunchResult {
    /// Direct connection established — relay can be dropped
    Direct {
        /// Peer's confirmed address
        peer_addr: SocketAddr,
        /// Local socket for direct communication
        local_socket: Arc<UdpSocket>,
        /// Measured latency
        latency: Duration,
    },
    /// Coordinated punch failed — relay remains active (zero disruption)
    KeepRelay {
        /// Number of ports sprayed
        ports_tried: u32,
        /// Reason for failure
        reason: String,
    },
}

impl HolePunchCoordinator {
    /// Relay-assisted coordinated punch
    ///
    /// Uses an active relay session as the signaling channel, combined with
    /// port pattern predictions from STUN probing. If the punch succeeds,
    /// the data path swaps from relay to direct P2P. If it fails, the relay
    /// continues — you never go backwards.
    ///
    /// ## Protocol
    ///
    /// 1. Use port predictions from `stun.probe_port_pattern` for both peers
    /// 2. Coordinate timing via relay signaling channel
    /// 3. Spray predicted ports (± window for prediction error)
    /// 4. Listen for response — first valid reply = success
    /// 5. Report result: Direct (drop relay) or KeepRelay (continue)
    ///
    /// # Arguments
    ///
    /// * `peer_node_id` - Target peer's node ID
    /// * `relay_session` - Active relay session for signaling
    /// * `our_pattern` - Our NAT port allocation pattern
    /// * `peer_predicted_port` - Peer's predicted next port
    /// * `peer_public_ip` - Peer's public IP address
    pub async fn coordinate_relay_punch(
        &self,
        peer_node_id: &str,
        relay_session: &songbird_lineage_relay::relay::RelaySession,
        our_pattern: &songbird_stun::PortPattern,
        peer_predicted_port: u16,
        peer_public_ip: std::net::IpAddr,
    ) -> Result<CoordinatedPunchResult> {
        info!(
            "🎯 Starting relay-assisted coordinated punch to {} (predicted port: {})",
            &peer_node_id[..8.min(peer_node_id.len())],
            peer_predicted_port
        );

        // 1. Bind a new socket for the punch attempt
        let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        let local_addr = socket.local_addr()?;

        // 2. Signal coordination timing via relay
        let start_delay_ms: u64 = 200; // Allow network propagation
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX_EPOCH")
            .as_millis() as u64;
        let start_at = now_ms + start_delay_ms;

        // Build coordination message
        let coord_msg = serde_json::json!({
            "type": "punch_coordinate",
            "from_node_id": self.my_node_id,
            "to_node_id": peer_node_id,
            "our_local_port": local_addr.port(),
            "peer_predicted_port": peer_predicted_port,
            "start_at_ms": start_at,
            "spray_window": 3,
        });

        let coord_bytes = serde_json::to_vec(&coord_msg)
            .map_err(|e| OnionRelayError::Other(format!("Failed to encode coordination: {e}")))?;

        relay_session.send(&coord_bytes).await.map_err(|e| {
            OnionRelayError::Transport(format!("Failed to send coordination via relay: {e}"))
        })?;

        debug!("📡 Sent coordination message via relay (start_at: {}ms)", start_at);

        // 3. Wait for start time
        let current_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX_EPOCH")
            .as_millis() as u64;

        if start_at > current_ms {
            sleep(Duration::from_millis(start_at - current_ms)).await;
        }

        // 4. Spray predicted ports (± window for prediction error)
        let spray_window: i32 = 3;
        let punch_msg = b"SONGBIRD_COORDINATED_PUNCH_V1";
        let mut ports_tried: u32 = 0;
        let start = Instant::now();

        for offset in -spray_window..=spray_window {
            let target_port = (i32::from(peer_predicted_port) + offset).clamp(1, 65535) as u16;
            let target_addr = SocketAddr::new(peer_public_ip, target_port);

            debug!("  🎯 Punch spray → {}:{}", peer_public_ip, target_port);

            if let Err(e) = socket.send_to(punch_msg, target_addr).await {
                debug!("  Send to port {} failed: {}", target_port, e);
                continue;
            }
            ports_tried += 1;
        }

        // Also punch the exact predicted port a few more times for reliability
        for _ in 0..3 {
            let target_addr = SocketAddr::new(peer_public_ip, peer_predicted_port);
            let _ = socket.send_to(punch_msg, target_addr).await;
        }

        // 5. Listen for any response with configurable timeout
        let listen_timeout = self.config.attempt_timeout * 3; // 3x normal for coordinated
        let mut recv_buf = vec![0u8; 1024];

        match timeout(listen_timeout, socket.recv_from(&mut recv_buf)).await {
            Ok(Ok((_len, from_addr))) if from_addr.ip() == peer_public_ip => {
                let latency = start.elapsed();
                info!(
                    "✅ Coordinated punch SUCCESS! {} → {} (latency: {:?})",
                    local_addr, from_addr, latency
                );

                Ok(CoordinatedPunchResult::Direct {
                    peer_addr: from_addr,
                    local_socket: socket,
                    latency,
                })
            }
            Ok(Ok((_len, from_addr))) => {
                warn!(
                    "⚠️ Received response from unexpected IP: {} (expected {})",
                    from_addr.ip(),
                    peer_public_ip
                );
                Ok(CoordinatedPunchResult::KeepRelay {
                    ports_tried,
                    reason: format!(
                        "Response from wrong IP ({}, expected {})",
                        from_addr.ip(),
                        peer_public_ip
                    ),
                })
            }
            Ok(Err(e)) => {
                info!("⚠️ Coordinated punch failed (recv error): {} — relay continues", e);
                Ok(CoordinatedPunchResult::KeepRelay {
                    ports_tried,
                    reason: format!("Network error: {e}"),
                })
            }
            Err(_) => {
                info!(
                    "⚠️ Coordinated punch timed out after {:?} — relay continues ({} ports tried)",
                    listen_timeout, ports_tried
                );

                // Check if our pattern is useful for a hint
                let _our_next = our_pattern.predict_next();

                Ok(CoordinatedPunchResult::KeepRelay {
                    ports_tried,
                    reason: format!(
                        "Timeout after {:?} ({} ports sprayed)",
                        listen_timeout, ports_tried
                    ),
                })
            }
        }
    }
}

/// Generate random nonce
fn rand_nonce() -> [u8; 16] {
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut nonce = [0u8; 16];
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    nonce[..8].copy_from_slice(&now.to_le_bytes()[..8]);
    // Add some randomness from memory address
    let ptr = &nonce as *const _ as usize;
    nonce[8..16].copy_from_slice(&ptr.to_le_bytes());
    nonce
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let config = HolePunchConfig::default();
        let (coord, _tx, _rx) = HolePunchCoordinator::new("test-node".to_string(), config);
        assert_eq!(coord.my_node_id, "test-node");
    }

    #[tokio::test]
    async fn test_peer_registration() {
        let config = HolePunchConfig::default();
        let (coord, _tx, _rx) = HolePunchCoordinator::new("test-node".to_string(), config);

        let peer = PeerInfo::new("peer-1".to_string(), "1.2.3.4:5678".parse().unwrap());
        coord.register_peer(peer).await;

        let peers = coord.peers.read().await;
        assert!(peers.contains_key("peer-1"));
    }
}
