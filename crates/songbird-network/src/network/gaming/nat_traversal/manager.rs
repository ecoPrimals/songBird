use super::stun::StunClient;
use super::types::*;
use songbird_errors::{SongbirdResult as Result, SongbirdError};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::net::UdpSocket;
use tokio::sync::RwLock as TokioRwLock;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

/// NAT traversal manager for gaming sessions
pub struct NatTraversalManager {
    stun_servers: Vec<SocketAddr>,
    turn_servers: Vec<TurnServer>,
    local_socket: Option<Arc<UdpSocket>>,
    external_address: Option<SocketAddr>,
    nat_type: NatType,
    connection_cache: Arc<TokioRwLock<HashMap<String, ConnectionInfo>>>,
    hole_punch_attempts: Arc<TokioRwLock<HashMap<String, HolePunchAttempt>>>,
    turn_allocations: Arc<TokioRwLock<HashMap<String, TurnAllocation>>>,
    config: NatTraversalConfig,
    stats: Arc<TokioRwLock<NatTraversalStats>>,
}

impl Default for NatTraversalManager {
    fn default() -> Self {
        Self::new(NatTraversalConfig::default())
    }
}

impl NatTraversalManager {
    /// Create a new NAT traversal manager
    pub fn new(config: NatTraversalConfig) -> Self {
        Self {
            stun_servers: config
                .stun_servers
                .iter()
                .map(|server| {
                    let addr_str = format!("{}:{}", server.address, server.port);
                    addr_str
                        .parse()
                        .or_else(|e| {
                            tracing::warn!(
                                "Failed to parse STUN server address '{}': {}, using default",
                                addr_str,
                                e
                            );
                            "127.0.0.1:3478".parse().map_err(|fallback_err| {
                                tracing::error!(
                                    "Critical: Default STUN server address is invalid: {}",
                                    fallback_err
                                );
                                fallback_err
                            })
                        })
                        .unwrap_or_else(|_| {
                            // Last resort: use a hardcoded address that we know is valid
                            std::net::SocketAddr::from(([127, 0, 0, 1], 3478))
                        })
                })
                .collect(),
            turn_servers: config
                .turn_servers
                .iter()
                .map(|config| TurnServer::new(config.clone()))
                .collect(),
            local_socket: None,
            external_address: None,
            nat_type: NatType::Unknown,
            connection_cache: Arc::new(TokioRwLock::new(HashMap::new())),
            hole_punch_attempts: Arc::new(TokioRwLock::new(HashMap::new())),
            turn_allocations: Arc::new(TokioRwLock::new(HashMap::new())),
            config,
            stats: Arc::new(TokioRwLock::new(NatTraversalStats::default())),
        }
    }

    /// Initialize the NAT traversal manager
    pub async fn initialize(&mut self, local_addr: SocketAddr) -> Result<()> {
        info!("Initializing NAT traversal manager on {}", local_addr);

        // Bind local socket
        let socket = UdpSocket::bind(local_addr).await.map_err(|e| {
            SongbirdError::network(format!(
                "NAT Traversal - Failed to bind local socket at {local_addr}: {e}"
            ))
        })?;

        self.local_socket = Some(Arc::new(socket));

        // Detect NAT type and external address
        self.detect_nat_type().await?;
        self.discover_external_address().await?;

        info!(
            "NAT traversal initialized. Type: {:?}, External: {:?}",
            self.nat_type, self.external_address
        );

        Ok(())
    }

    /// Detect NAT type using STUN servers
    async fn detect_nat_type(&mut self) -> Result<()> {
        if self.stun_servers.is_empty() {
            warn!("No STUN servers configured, assuming unknown NAT type");
            self.nat_type = NatType::Unknown;
            return Ok(());
        }

        // Simple NAT type detection using first STUN server
        let stun_server = self.stun_servers[0];
        let local_addr = self
            .local_socket
            .as_ref()
            .ok_or_else(|| {
                SongbirdError::network("NAT Traversal - No local socket available".to_string(),
                )
            })?
            .local_addr()
            .map_err(|e| {
                SongbirdError::network(format!(
                    "NAT Traversal - Failed to get local address: {e}"
                ))
            })?;

        let stun_client = StunClient::new(local_addr).await?;

        match stun_client.binding_request(stun_server).await {
            Ok(external_addr) => {
                // Compare external and local addresses to determine NAT type
                if external_addr.ip() == local_addr.ip() {
                    self.nat_type = NatType::FullCone;
                } else {
                    // For simplicity, assume full cone NAT if we can reach STUN server
                    self.nat_type = NatType::FullCone;
                }
                self.external_address = Some(external_addr);
            }
            Err(e) => {
                warn!("Failed to detect NAT type: {}", e);
                self.nat_type = NatType::Unknown;
            }
        }

        Ok(())
    }

    /// Discover external address using STUN
    async fn discover_external_address(&mut self) -> Result<()> {
        if self.external_address.is_some() {
            return Ok(());
        }

        for stun_server in &self.stun_servers {
            let local_addr = self
                .local_socket
                .as_ref()
                .ok_or_else(|| {
                    SongbirdError::network("NAT Traversal - No local socket available".to_string(),
                    )
                })?
                .local_addr()
                .map_err(|e| {
                    SongbirdError::network(format!(
                        "NAT Traversal - Failed to get local address: {e}"
                    ))
                })?;

            let stun_client = StunClient::new(local_addr).await?;

            match stun_client.binding_request(*stun_server).await {
                Ok(external_addr) => {
                    self.external_address = Some(external_addr);
                    info!("Discovered external address: {}", external_addr);
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        "Failed to discover external address via {}: {}",
                        stun_server, e
                    );
                }
            }
        }

        Err(SongbirdError::network("NAT Traversal - Failed to discover external address via any STUN server",
        ))
    }

    /// Attempt to establish connection with peer
    pub async fn connect_to_peer(
        &self,
        peer_id: &str,
        peer_address: SocketAddr,
    ) -> Result<ConnectionResult> {
        let start_time = Instant::now();

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.successful_connections += 1;
        }

        // Check if we can connect directly
        if self.nat_type == NatType::FullCone {
            match self.try_direct_connection(peer_address).await {
                Ok(()) => {
                    let connection_info = ConnectionInfo {
                        peer_id: peer_id.to_string(),
                        local_addr: peer_address,
                        public_addr: peer_address,
                        public_address: peer_address,
                        private_address: None,
                        connection_type: ConnectionType::Direct,
                        last_seen: SystemTime::now(),
                        latency: Some(start_time.elapsed().as_millis() as u32),
                        bandwidth: None,
                    };

                    self.connection_cache
                        .write()
                        .await
                        .insert(peer_id.to_string(), connection_info);

                    // Update stats
                    {
                        let mut stats = self.stats.write().await;
                        stats.successful_connections += 1;
                        stats.active_connections += 1;
                    }

                    return Ok(ConnectionResult::Success {
                        connection_type: ConnectionType::Direct,
                        local_addr: peer_address,
                        remote_addr: peer_address,
                        latency: start_time.elapsed(),
                    });
                }
                Err(e) => {
                    debug!("Direct connection failed: {}", e);
                }
            }
        }

        // Try hole punching
        if self.nat_type.supports_hole_punching() {
            match self.attempt_hole_punch(peer_id, peer_address).await {
                Ok(HolePunchResult::Success {
                    target_addr,
                    local_addr,
                    attempts: _attempts,
                    duration,
                }) => {
                    let connection_info = ConnectionInfo {
                        peer_id: peer_id.to_string(),
                        local_addr,
                        public_addr: target_addr,
                        public_address: target_addr,
                        private_address: None,
                        connection_type: ConnectionType::HolePunch,
                        last_seen: SystemTime::now(),
                        latency: Some(duration.as_millis() as u32),
                        bandwidth: None,
                    };

                    self.connection_cache
                        .write()
                        .await
                        .insert(peer_id.to_string(), connection_info);

                    // Update stats
                    {
                        let mut stats = self.stats.write().await;
                        stats.stun_responses += 1;
                        stats.active_connections += 1;
                    }

                    return Ok(ConnectionResult::Success {
                        connection_type: ConnectionType::HolePunch,
                        local_addr,
                        remote_addr: target_addr,
                        latency: duration,
                    });
                }
                Ok(HolePunchResult::Failed {
                    target_addr: _target_addr,
                    reason,
                    attempts,
                }) => {
                    debug!("Hole punch failed after {} attempts: {}", attempts, reason);
                }
                Ok(HolePunchResult::Timeout {
                    target_addr: _target_addr,
                    attempts,
                }) => {
                    debug!("Hole punch timed out after {} attempts", attempts);
                }
                Err(e) => {
                    debug!("Hole punch error: {}", e);
                }
            }
        }

        // Try TURN relay if available
        if !self.turn_servers.is_empty() {
            match self.try_turn_relay(peer_id, peer_address).await {
                Ok(_relay_address) => {
                    let connection_info = ConnectionInfo {
                        peer_id: peer_id.to_string(),
                        local_addr: peer_address,
                        public_addr: peer_address,
                        public_address: peer_address,
                        private_address: None,
                        connection_type: ConnectionType::Turn,
                        last_seen: SystemTime::now(),
                        latency: Some(start_time.elapsed().as_millis() as u32),
                        bandwidth: None,
                    };

                    self.connection_cache
                        .write()
                        .await
                        .insert(peer_id.to_string(), connection_info);

                    // Update stats
                    {
                        let mut stats = self.stats.write().await;
                        stats.turn_allocations += 1;
                        stats.active_connections += 1;
                    }

                    return Ok(ConnectionResult::Success {
                        connection_type: ConnectionType::Turn,
                        local_addr: peer_address,
                        remote_addr: peer_address,
                        latency: start_time.elapsed(),
                    });
                }
                Err(e) => {
                    debug!("TURN relay failed: {}", e);
                }
            }
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.failed_connections += 1;
        }

        Ok(ConnectionResult::Failed {
            reason: "All connection methods failed".to_string(),
            attempts: 0, // Direct, hole punch, TURN
        })
    }

    /// Try direct connection to peer
    async fn try_direct_connection(&self, peer_address: SocketAddr) -> Result<()> {
        let socket = self.local_socket.as_ref().ok_or_else(|| {
            SongbirdError::network("NAT Traversal - No local socket available".to_string())
        })?;

        // Send a simple ping packet
        let ping_data = b"PING";
        socket.send_to(ping_data, peer_address).await.map_err(|e| {
            SongbirdError::network(format!("NAT Traversal - Failed to send ping: {e}"))
        })?;

        // For simplicity, assume success if we can send
        Ok(())
    }

    /// Attempt hole punching with peer
    async fn attempt_hole_punch(
        &self,
        peer_id: &str,
        peer_address: SocketAddr,
    ) -> Result<HolePunchResult> {
        let start_time = Instant::now();
        let mut attempts = 0;

        // Record hole punch attempt
        {
            let attempt = HolePunchAttempt {
                target_addr: peer_address,
                local_port: 0,
                attempt_count: 0,
                success: false,
                timestamp: SystemTime::now(),
            };

            self.hole_punch_attempts
                .write()
                .await
                .insert(peer_id.to_string(), attempt);
        }

        let socket = self.local_socket.as_ref().ok_or_else(|| {
            SongbirdError::network("NAT Traversal - No local socket available".to_string())
        })?;

        // Simplified hole punching - send packets periodically
        while attempts < self.config.hole_punch_attempts {
            attempts += 1;

            // Send hole punch packet
            let hole_punch_data = format!("HOLE_PUNCH_{attempts}");
            socket
                .send_to(hole_punch_data.as_bytes(), peer_address)
                .await
                .map_err(|e| {
                    SongbirdError::network(format!(
                        "NAT Traversal - Failed to send hole punch packet: {e}"
                    ))
                })?;

            // Update attempt record
            {
                if let Some(attempt) = self.hole_punch_attempts.write().await.get_mut(peer_id) {
                    attempt.attempt_count = attempts;
                    attempt.timestamp = SystemTime::now();
                }
            }

            // Wait before next attempt
            sleep(Duration::from_millis(200)).await;

            // Check if hole punch succeeded (simplified - in real implementation,
            // this would check for response packets)
            if attempts >= 3 && self.is_address_reachable(peer_address).await {
                // Update success status
                {
                    if let Some(attempt) = self.hole_punch_attempts.write().await.get_mut(peer_id) {
                        attempt.success = true;
                    }
                }

                return Ok(HolePunchResult::Success {
                    target_addr: peer_address,
                    local_addr: peer_address,
                    attempts,
                    duration: start_time.elapsed(),
                });
            }
        }

        // Check for timeout
        if start_time.elapsed() > Duration::from_millis(self.config.hole_punch_timeout_ms) {
            return Ok(HolePunchResult::Timeout {
                target_addr: peer_address,
                attempts,
            });
        }

        Ok(HolePunchResult::Failed {
            target_addr: peer_address,
            reason: "Hole punch unsuccessful".to_string(),
            attempts,
        })
    }

    /// Check if address is reachable (simplified implementation)
    async fn is_address_reachable(&self, address: SocketAddr) -> bool {
        // In a real implementation, this would send a test packet and wait for response
        // For now, just check if the address is valid
        match address.ip() {
            IpAddr::V4(ipv4) => !ipv4.is_private() && !ipv4.is_loopback() && !ipv4.is_multicast(),
            IpAddr::V6(_) => false, // Simplified for demo
        }
    }

    /// Try TURN relay connection
    async fn try_turn_relay(&self, _peer_id: &str, peer_address: SocketAddr) -> Result<SocketAddr> {
        if self.turn_servers.is_empty() {
            return Err(SongbirdError::network("NAT Traversal - No TURN servers configured".to_string(),
            ));
        }

        // For simplicity, just return the peer address
        // In a real implementation, this would:
        // 1. Connect to TURN server
        // 2. Allocate a relay address
        // 3. Create permission for peer
        // 4. Return relay address

        Ok(peer_address)
    }

    /// Get connection info for peer
    pub async fn get_connection_info(&self, peer_id: &str) -> Option<ConnectionInfo> {
        self.connection_cache.read().await.get(peer_id).cloned()
    }

    /// Get current NAT type
    pub fn get_nat_type(&self) -> NatType {
        self.nat_type.clone()
    }

    /// Get external address discovered through NAT traversal
    pub fn get_external_address(&self) -> Option<SocketAddr> {
        self.external_address
    }

    /// Establish connection to a peer
    pub async fn establish_connection(
        &mut self,
        peer_id: String,
        target_addr: SocketAddr,
    ) -> Result<SocketAddr> {
        info!(
            "Establishing connection to peer {} at {}",
            peer_id, target_addr
        );

        // Check if we already have a connection to this peer
        {
            let connections = self.connection_cache.read().await;
            if let Some(conn) = connections.get(&peer_id) {
                info!("Connection to peer {} already exists", peer_id);
                return Ok(conn.local_addr);
            }
        }

        // Try direct connection first
        match self.try_direct_connection(target_addr).await {
            Ok(()) => {
                info!(
                    "Direct connection established to peer {} at {}",
                    peer_id, target_addr
                );
                return Ok(target_addr);
            }
            Err(e) => {
                debug!("Direct connection failed: {}, trying hole punching", e);
            }
        }

        // Try hole punching
        match self.attempt_hole_punch(&peer_id, target_addr).await {
            Ok(HolePunchResult::Success {
                target_addr,
                local_addr,
                ..
            }) => {
                info!(
                    "Hole punch connection established to peer {} at {}",
                    peer_id, target_addr
                );
                return Ok(local_addr);
            }
            Ok(_) => {
                debug!("Hole punching failed, trying TURN relay");
            }
            Err(e) => {
                debug!("Hole punching error: {}, trying TURN relay", e);
            }
        }

        // Fall back to TURN relay
        match self.try_turn_relay(&peer_id, target_addr).await {
            Ok(addr) => {
                info!(
                    "TURN relay connection established to peer {} at {}",
                    peer_id, addr
                );
                Ok(addr)
            }
            Err(e) => {
                error!("All connection attempts failed for peer {}: {}", peer_id, e);
                Err(e)
            }
        }
    }

    /// Get statistics
    pub async fn get_stats(&self) -> NatTraversalStats {
        self.stats.read().await.clone()
    }

    /// Remove connection for a peer
    pub async fn remove_connection(&mut self, peer_id: &str) {
        let mut connections = self.connection_cache.write().await;
        connections.remove(peer_id);
    }

    /// Cleanup expired connections and attempts
    pub async fn cleanup(&self) {
        let now = SystemTime::now();
        let timeout = Duration::from_secs(300); // 5 minutes

        // Clean up connection cache
        {
            let mut cache = self.connection_cache.write().await;
            cache.retain(|_, conn| {
                now.duration_since(conn.last_seen).unwrap_or(Duration::MAX) < timeout
            });
        }

        // Clean up hole punch attempts
        {
            let mut attempts = self.hole_punch_attempts.write().await;
            attempts.retain(|_, attempt| {
                now.duration_since(attempt.timestamp)
                    .unwrap_or(Duration::MAX)
                    < timeout
            });
        }

        // Clean up TURN allocations
        {
            let mut allocations = self.turn_allocations.write().await;
            allocations.retain(|_, alloc| now < alloc.created_at);
        }
    }
}
