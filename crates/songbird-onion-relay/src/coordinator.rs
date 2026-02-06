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
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, timeout};
use tracing::{debug, info, warn};

/// Configuration for hole punch attempts
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
    /// STUN servers to use for address discovery
    pub stun_servers: Vec<String>,
}

impl Default for HolePunchConfig {
    fn default() -> Self {
        Self {
            max_attempts: 20,
            attempt_timeout: Duration::from_millis(500),
            packet_interval: Duration::from_millis(50),
            total_timeout: Duration::from_secs(10),
            stun_servers: vec![
                "stun.l.google.com:19302".to_string(),
                "stun1.l.google.com:19302".to_string(),
            ],
        }
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
        let my_info = self.my_info.read().await.clone()
            .ok_or_else(|| OnionRelayError::Other("Must discover public address first".to_string()))?;
        
        // Get peer info
        let peer_info = self.peers.read().await.get(peer_node_id).cloned()
            .ok_or_else(|| OnionRelayError::PeerNotFound(peer_node_id.to_string()))?;
        
        // Generate nonce for this attempt
        let nonce: [u8; 16] = rand_nonce();
        
        // Send punch request via signaling
        let request = SignalingMessage::PunchRequest {
            from: my_info.clone(),
            to_node_id: peer_node_id.to_string(),
            nonce,
        };
        
        self.signal_tx.send(request).await
            .map_err(|e| OnionRelayError::Transport(e.to_string()))?;
        
        // Wait for ack with start time
        let start_time = self.wait_for_punch_ack(&nonce).await?;
        
        // Create socket for punching
        let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        
        // Wait until coordinated start time
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
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
            SignalingMessage::Register { peer_info, .. } => {
                self.register_peer(peer_info).await;
                None
            }
            SignalingMessage::Query { target_node_id } => {
                let peer_info = self.peers.read().await.get(&target_node_id).cloned();
                Some(SignalingMessage::PeerInfoResponse { peer_info })
            }
            SignalingMessage::PunchRequest { from, to_node_id, nonce } => {
                if to_node_id == self.my_node_id {
                    // We're the target - send ack with coordinated time
                    self.register_peer(from.clone()).await;
                    
                    let my_info = self.my_info.read().await.clone();
                    if let Some(info) = my_info {
                        // Start in 100ms to allow network propagation
                        let start_at_ms = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64 + 100;
                        
                        return Some(SignalingMessage::PunchAck {
                            from: info,
                            nonce,
                            start_at_ms,
                        });
                    }
                }
                None
            }
            SignalingMessage::Heartbeat { node_id } => {
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
    
    async fn stun_bind(&self, socket: &UdpSocket, server: &str) -> Result<SocketAddr> {
        use songbird_stun::StunClient;
        
        let client = StunClient::new();
        client.discover_public_address(server).await
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
                debug!("NAT type: Different ports ({} vs {}) → Symmetric NAT", a1.port(), a2.port());
                NatType::Symmetric
            }
            _ => NatType::Unknown,
        }
    }
    
    async fn wait_for_punch_ack(&self, nonce: &[u8; 16]) -> Result<u64> {
        // In real impl, this would wait on the signal_rx channel
        // For now, return a time 100ms in the future
        let start = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64 + 100;
        
        Ok(start)
    }
    
    async fn execute_punch(&self, socket: Arc<UdpSocket>, peer_addr: SocketAddr) -> Result<Duration> {
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
        
        Err(OnionRelayError::HolePunchFailed { attempts: self.config.max_attempts })
    }
}

/// Generate random nonce
fn rand_nonce() -> [u8; 16] {
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut nonce = [0u8; 16];
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
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
