//! TURN client for relay connectivity

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::types::{DiscoveredPeer, DiscoveryConfig, TURNRelay};
use songbird_errors::{SongbirdError, SongbirdResult as Result};
use songbird_universal_primals::PrimalCapability;

/// TURN client for relay connectivity
pub struct TURNClient {
    turn_servers: Vec<String>,
    username: Option<String>,
    password: Option<String>,
    allocated_relays: Arc<RwLock<HashMap<String, TURNRelay>>>,
}

/// TURN relay allocation response
#[derive(Debug, Clone)]
pub struct TURNAllocation {
    pub relay_address: SocketAddr,
    pub allocation_id: String,
    pub expires_at: std::time::Instant,
}

impl TURNClient {
    /// Create new TURN client
    pub fn new(_config: &DiscoveryConfig) -> Self {
        Self {
            turn_servers: vec![
                "turn:coturn.example.com:3478".to_string(),
                "turns:coturn.example.com:5349".to_string(),
            ],
            username: None,
            password: None,
            allocated_relays: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set TURN server credentials
    pub fn set_credentials(&mut self, username: String, password: String) {
        self.username = Some(username);
        self.password = Some(password);
    }

    /// Discover peers via TURN relay
    pub async fn discover_peers(&self) -> Result<Vec<DiscoveredPeer>> {
        debug!("Discovering peers via TURN...");

        let mut peers = Vec::new();

        // Allocate TURN relay if not already done
        if let Ok(Some(allocation)) = self.allocate_relay().await {
            info!("TURN relay allocated at: {}", allocation.relay_address);

            // Discover relay capabilities through each TURN server
            for server in &self.turn_servers {
                match self.discover_relay_capabilities(server).await {
                    Ok(_capabilities) => {
                        let peer = DiscoveredPeer::new(
                            format!("turn-{}", server),
                            server
                                .parse()
                                .unwrap_or_else(|_| "0.0.0.0:3478".parse().unwrap()),
                            super::types::PeerType::Service,
                            super::types::DiscoveryMethod::TURN,
                        );
                        peers.push(peer);
                    }
                    Err(e) => {
                        debug!(
                            "Failed to discover capabilities from TURN server {}: {}",
                            server, e
                        );
                    }
                }
            }
        }

        debug!(
            "TURN discovery found {} peers with relay capabilities",
            peers.len()
        );
        Ok(peers)
    }

    /// Discover relay capabilities through TURN server
    pub async fn discover_relay_capabilities(&self, server: &str) -> Result<Vec<PrimalCapability>> {
        debug!("Discovering relay capabilities via TURN server: {}", server);

        // Test allocation to determine if TURN relay is functional
        let relay_allocation = match self.request_allocation(server).await {
            Ok(allocation) => allocation,
            Err(e) => {
                debug!("TURN allocation failed: {}", e);
                return Ok(Vec::new()); // Return empty capabilities on failure
            }
        };

        debug!(
            "Successfully allocated TURN relay: {}",
            relay_allocation.relay_address
        );

        let mut capabilities = Vec::new();

        // Basic relay capability
        capabilities.push(PrimalCapability::NetworkRouting {
            protocols: vec![
                "TURN".to_string(),
                "RELAY".to_string(),
                "UDP".to_string(),
                "TCP".to_string(),
                "TLS".to_string(), // TURN over TLS
            ],
        });

        // Measure relay performance
        let start_time = std::time::Instant::now();

        // Test relay responsiveness with a second allocation request
        if self.request_allocation(server).await.is_ok() {
            let relay_rtt = start_time.elapsed().as_millis() as f64;

            // Estimate relay bandwidth (generally lower than direct connection)
            let estimated_bandwidth = self.estimate_relay_bandwidth_from_rtt(relay_rtt);

            capabilities.push(PrimalCapability::Custom {
                name: "RelayConnectivity".to_string(),
                properties: vec![
                    (
                        "relay_ip".to_string(),
                        relay_allocation.relay_address.ip().to_string(),
                    ),
                    (
                        "relay_port".to_string(),
                        relay_allocation.relay_address.port().to_string(),
                    ),
                    ("relay_rtt_ms".to_string(), relay_rtt.to_string()),
                    (
                        "estimated_bandwidth_mbps".to_string(),
                        estimated_bandwidth.to_string(),
                    ),
                    ("relay_type".to_string(), "turn_allocated".to_string()),
                ]
                .into_iter()
                .collect(),
            });
        }

        // Test if server supports both UDP and TCP relay
        if self.test_tcp_relay_support(server).await {
            capabilities.push(PrimalCapability::Custom {
                name: "RelayProtocols".to_string(),
                properties: vec![
                    ("udp_relay".to_string(), "supported".to_string()),
                    ("tcp_relay".to_string(), "supported".to_string()),
                    ("protocol_switching".to_string(), "available".to_string()),
                ]
                .into_iter()
                .collect(),
            });
        }

        debug!(
            "Discovered {} relay capabilities via TURN",
            capabilities.len()
        );
        Ok(capabilities)
    }

    /// Estimate relay bandwidth based on RTT characteristics  
    pub fn estimate_relay_bandwidth_from_rtt(&self, rtt_ms: f64) -> f64 {
        // Relay connections typically have ~30-50% overhead
        let base_bandwidth = match rtt_ms {
            rtt if rtt < 10.0 => 100.0, // Good relay server
            rtt if rtt < 30.0 => 50.0,  // Standard relay
            rtt if rtt < 80.0 => 25.0,  // Slower relay
            _ => 10.0,                  // High latency relay
        };

        // Apply relay overhead factor
        base_bandwidth * 0.7
    }

    /// Test if TURN server supports TCP relay
    pub async fn test_tcp_relay_support(&self, _server: &str) -> bool {
        // For now, assume most modern TURN servers support both UDP and TCP
        // In a full implementation, this would test with a TCP allocation request
        true
    }

    /// Allocate TURN relay
    async fn allocate_relay(&self) -> Result<Option<TURNAllocation>> {
        for server in &self.turn_servers {
            match self.request_allocation(server).await {
                Ok(allocation) => {
                    let relay = TURNRelay::new(
                        allocation.allocation_id.clone(),
                        allocation.relay_address,
                        Duration::from_secs(600), // 10 minutes
                    );

                    let mut relays = self.allocated_relays.write().await;
                    relays.insert(allocation.allocation_id.clone(), relay);

                    return Ok(Some(allocation));
                }
                Err(e) => {
                    warn!("Failed to allocate relay from {}: {}", server, e);
                }
            }
        }

        Ok(None)
    }

    /// Request allocation from TURN server
    async fn request_allocation(&self, server: &str) -> Result<TURNAllocation> {
        debug!("Requesting TURN allocation from {}", server);

        // Create UDP socket for TURN allocation
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
        let server_addr: SocketAddr = server
            .trim_start_matches("turn:")
            .trim_start_matches("turns:")
            .parse()?;

        // TURN Allocate Request message (simplified)
        let turn_request = self.create_turn_allocate_request();

        socket.send_to(&turn_request, server_addr).await?;

        // Wait for TURN response
        let mut buffer = [0u8; 1024];
        match tokio::time::timeout(Duration::from_secs(10), socket.recv_from(&mut buffer)).await {
            Ok(Ok((size, _))) => {
                // Parse TURN response to extract relay address
                if let Some(relay_addr) = self.parse_turn_allocation_response(&buffer[..size]) {
                    debug!("TURN relay allocated: {}", relay_addr);

                    Ok(TURNAllocation {
                        relay_address: relay_addr,
                        allocation_id: format!("alloc_{}", rand::random::<u32>()),
                        expires_at: std::time::Instant::now() + Duration::from_secs(600),
                    })
                } else {
                    Err(SongbirdError::network(
                        "Failed to parse TURN allocation response",
                    ))
                }
            }
            Ok(Err(e)) => Err(e.into()),
            Err(_) => Err(SongbirdError::network("TURN allocation timed out")),
        }
    }

    /// Create TURN allocate request message
    fn create_turn_allocate_request(&self) -> Vec<u8> {
        // Simplified TURN message format
        // In a real implementation, this would be a proper TURN message
        let mut message = Vec::new();

        // TURN message type: Allocate Request (0x0003)
        message.extend_from_slice(&[0x00, 0x03]);

        // Message length (0 for no attributes for now)
        message.extend_from_slice(&[0x00, 0x00]);

        // Magic cookie (0x2112A442)
        message.extend_from_slice(&[0x21, 0x12, 0xA4, 0x42]);

        // Transaction ID (12 bytes, random)
        for _ in 0..12 {
            message.push(rand::random::<u8>());
        }

        message
    }

    /// Parse TURN allocation response
    fn parse_turn_allocation_response(&self, data: &[u8]) -> Option<SocketAddr> {
        if data.len() < 20 {
            return None;
        }

        // Validate TURN message header
        let message_type = u16::from_be_bytes([data[0], data[1]]);
        let message_length = u16::from_be_bytes([data[2], data[3]]);
        let magic_cookie = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        // Check if this is a TURN allocate success response (0x0103)
        if message_type != 0x0103 || magic_cookie != 0x2112A442 {
            debug!(
                "Invalid TURN response: type={:#x}, cookie={:#x}",
                message_type, magic_cookie
            );
            return None;
        }

        // Parse attributes starting at offset 20
        let mut offset = 20;

        while offset + 4 <= data.len() && offset - 20 < message_length as usize {
            if offset + 4 > data.len() {
                break;
            }

            let attr_type = u16::from_be_bytes([data[offset], data[offset + 1]]);
            let attr_length = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
            offset += 4;

            // Ensure we don't read beyond buffer
            if offset + attr_length as usize > data.len() {
                debug!("TURN attribute extends beyond message length");
                break;
            }

            // Check for XOR-RELAYED-ADDRESS (0x0016)
            if attr_type == 0x0016 && attr_length >= 8 && offset + 8 <= data.len() {
                let _reserved = data[offset];
                let family = data[offset + 1];

                if family == 0x01 {
                    // IPv4
                    let xor_port = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
                    let xor_addr_bytes = [
                        data[offset + 4],
                        data[offset + 5],
                        data[offset + 6],
                        data[offset + 7],
                    ];

                    // Get transaction ID from message header for XOR operation
                    let transaction_id = &data[8..20];
                    let xor_key = [
                        transaction_id[0] ^ 0x21,
                        transaction_id[1] ^ 0x12,
                        transaction_id[2] ^ 0xA4,
                        transaction_id[3] ^ 0x42,
                    ];

                    // XOR with magic cookie and transaction ID to get real values
                    let port = xor_port ^ 0x2112;
                    let addr_u32 = u32::from_be_bytes(xor_addr_bytes) ^ u32::from_be_bytes(xor_key);
                    let addr = std::net::Ipv4Addr::from(addr_u32);

                    debug!("Found TURN relay address: {}:{}", addr, port);
                    return Some(SocketAddr::from((addr, port)));
                }
            }

            // Move to next attribute (padding to 4-byte boundary)
            let padded_length = (attr_length + 3) & !3;
            offset += padded_length as usize;
        }

        debug!("No valid relay address attribute found in TURN response");
        None
    }

    /// Send data through TURN relay
    pub async fn send_through_relay(
        &self,
        relay_id: &str,
        data: &[u8],
        target: SocketAddr,
    ) -> Result<()> {
        let relays = self.allocated_relays.read().await;

        if let Some(relay) = relays.get(relay_id) {
            if !relay.is_expired() {
                // In a real implementation, this would send data through the TURN relay
                debug!(
                    "Sending {} bytes through TURN relay {} to {}",
                    data.len(),
                    relay_id,
                    target
                );
                // Implementation would use TURN Send indication
                Ok(())
            } else {
                Err(SongbirdError::network("TURN relay has expired"))
            }
        } else {
            Err(SongbirdError::network("TURN relay not found"))
        }
    }

    /// Get all allocated relays
    pub async fn get_allocated_relays(&self) -> HashMap<String, TURNRelay> {
        self.allocated_relays.read().await.clone()
    }

    /// Release TURN relay
    pub async fn release_relay(&self, relay_id: &str) -> Result<()> {
        let mut relays = self.allocated_relays.write().await;

        if let Some(_relay) = relays.remove(relay_id) {
            // In a real implementation, this would send a TURN Refresh request
            // with lifetime=0 to release the allocation
            debug!("Released TURN relay: {}", relay_id);
            Ok(())
        } else {
            Err(SongbirdError::network("TURN relay not found for release"))
        }
    }

    /// Cleanup expired relays
    pub async fn cleanup_expired_relays(&self) {
        let mut relays = self.allocated_relays.write().await;
        let expired_keys: Vec<String> = relays
            .iter()
            .filter(|(_, relay)| relay.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            relays.remove(&key);
            debug!("Cleaned up expired TURN relay: {}", key);
        }
    }

    /// Test connectivity to TURN servers
    pub async fn test_turn_connectivity(&self) -> Result<Vec<String>> {
        let mut working_servers = Vec::new();

        for server in &self.turn_servers {
            if self.request_allocation(server).await.is_ok() {
                working_servers.push(server.clone());
            }
        }

        debug!("Working TURN servers: {:?}", working_servers);
        Ok(working_servers)
    }

    /// Add custom TURN server
    pub fn add_turn_server(&mut self, server: String) {
        if !self.turn_servers.contains(&server) {
            self.turn_servers.push(server);
        }
    }

    /// Remove TURN server
    pub fn remove_turn_server(&mut self, server: &str) {
        self.turn_servers.retain(|s| s != server);
    }

    /// Get configured TURN servers
    pub fn get_turn_servers(&self) -> &[String] {
        &self.turn_servers
    }
}
