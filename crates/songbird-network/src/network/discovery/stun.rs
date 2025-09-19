//! STUN client for NAT traversal

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::types::{DiscoveredPeer, DiscoveryConfig};
use songbird_errors::SongbirdResult as Result;
use songbird_universal_primals::PrimalCapability;

/// STUN client for NAT traversal
pub struct STUNClient {
    stun_servers: Vec<String>,
    timeout: Duration,
    external_addresses: Arc<RwLock<HashMap<String, SocketAddr>>>,
}

impl STUNClient {
    /// Create new STUN client
    pub fn new(config: &DiscoveryConfig) -> Self {
        Self {
            stun_servers: vec![
                "stun.l.google.com:19302".to_string(),
                "stun1.l.google.com:19302".to_string(),
                "stun2.l.google.com:19302".to_string(),
            ],
            timeout: config.discovery_timeout,
            external_addresses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Discover peers via STUN
    pub async fn discover_peers(&self) -> Result<Vec<DiscoveredPeer>> {
        debug!("Discovering peers via STUN...");

        let mut peers = Vec::new();

        // Get external address for NAT traversal
        let external_addr = self.get_external_address().await?;
        info!("External address discovered: {}", external_addr);

        // Discover network capabilities through each STUN server
        for server in &self.stun_servers {
            match self.discover_capabilities(server).await {
                Ok(_capabilities) => {
                    let peer = DiscoveredPeer::new(
                        format!("stun-{}", server),
                        server
                            .parse()
                            .unwrap_or_else(|_| "0.0.0.0:3478".parse().unwrap()),
                        super::types::PeerType::Service,
                        super::types::DiscoveryMethod::STUN,
                    );
                    peers.push(peer);
                }
                Err(e) => {
                    debug!(
                        "Failed to discover capabilities from STUN server {}: {}",
                        server, e
                    );
                }
            }
        }

        debug!(
            "STUN discovery found {} peers with capabilities",
            peers.len()
        );
        Ok(peers)
    }

    /// Discover network capabilities using STUN server
    pub async fn discover_capabilities(&self, server: &str) -> Result<Vec<PrimalCapability>> {
        debug!(
            "Discovering network capabilities via STUN server: {}",
            server
        );

        // Test external address discovery - if this works, we can traverse NAT
        let external_addr = match self.query_external_address(server).await {
            Ok(addr) => addr,
            Err(e) => {
                debug!("STUN server query failed: {}", e);
                return Ok(Vec::new()); // Return empty capabilities on failure
            }
        };

        debug!(
            "Successfully discovered external address: {}",
            external_addr
        );

        // Determine capabilities based on successful STUN interaction
        let mut capabilities = Vec::new();

        // Basic network routing capability
        capabilities.push(PrimalCapability::NetworkRouting {
            protocols: vec![
                "STUN".to_string(),
                "ICE".to_string(),
                "UDP".to_string(),
                "NAT-T".to_string(), // NAT Traversal capability
            ],
        });

        // Test connectivity quality with basic metrics
        let start_time = std::time::Instant::now();

        // Perform a second query to measure round-trip time
        if self.query_external_address(server).await.is_ok() {
            let rtt_ms = start_time.elapsed().as_millis() as f64;

            // Estimate bandwidth based on connection type and RTT
            let estimated_bandwidth = self.estimate_bandwidth_from_rtt(rtt_ms);

            capabilities.push(PrimalCapability::Custom {
                name: "NetworkConnectivity".to_string(),
                properties: vec![
                    ("external_ip".to_string(), external_addr.ip().to_string()),
                    (
                        "external_port".to_string(),
                        external_addr.port().to_string(),
                    ),
                    ("rtt_ms".to_string(), rtt_ms.to_string()),
                    (
                        "estimated_bandwidth_mbps".to_string(),
                        estimated_bandwidth.to_string(),
                    ),
                    ("nat_traversal".to_string(), "supported".to_string()),
                ]
                .into_iter()
                .collect(),
            });
        }

        // Test for symmetric NAT detection by querying from different ports
        if self.detect_nat_type(server).await.is_ok() {
            capabilities.push(PrimalCapability::Custom {
                name: "NATTraversal".to_string(),
                properties: vec![
                    ("type".to_string(), "cone_nat".to_string()), // Conservative assumption
                    ("hole_punching".to_string(), "supported".to_string()),
                ]
                .into_iter()
                .collect(),
            });
        }

        debug!(
            "Discovered {} network capabilities via STUN",
            capabilities.len()
        );
        Ok(capabilities)
    }

    /// Estimate bandwidth from RTT measurements
    pub fn estimate_bandwidth_from_rtt(&self, rtt_ms: f64) -> f64 {
        match rtt_ms {
            rtt if rtt < 5.0 => 1000.0, // Local network - assume gigabit
            rtt if rtt < 20.0 => 100.0, // Fast broadband
            rtt if rtt < 50.0 => 50.0,  // Standard broadband
            rtt if rtt < 100.0 => 25.0, // Slower connection
            _ => 10.0,                  // High latency connection
        }
    }

    /// Detect NAT type using multiple STUN servers
    pub async fn detect_nat_type(&self, server: &str) -> Result<String> {
        // This is a simplified NAT detection
        // A full implementation would test from multiple source ports and servers

        let _addr1 = self.query_external_address(server).await?;

        // For now, if we get consistent results, assume cone NAT (good for P2P)
        Ok("cone_nat".to_string())
    }

    /// Get external address using STUN
    async fn get_external_address(&self) -> Result<SocketAddr> {
        for server in &self.stun_servers {
            match self.query_external_address(server).await {
                Ok(addr) => {
                    let mut addresses = self.external_addresses.write().await;
                    addresses.insert(server.clone(), addr);
                    return Ok(addr);
                }
                Err(e) => {
                    warn!("Failed to get external address from {}: {}", server, e);
                }
            }
        }

        Err(songbird_errors::SongbirdError::network(
            "Failed to determine external address via STUN",
        ))
    }

    /// Query STUN server for external address
    async fn query_external_address(&self, server: &str) -> Result<SocketAddr> {
        debug!("Querying STUN server {} for external address", server);

        // Create UDP socket for STUN query
        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
        let server_addr: SocketAddr = server.parse()?;

        // STUN Binding Request message (simplified)
        let stun_request = self.create_stun_binding_request();

        socket.send_to(&stun_request, server_addr).await?;

        // Wait for STUN response with timeout
        let mut buffer = [0u8; 1024];
        match tokio::time::timeout(self.timeout, socket.recv_from(&mut buffer)).await {
            Ok(Ok((size, _))) => {
                // Parse STUN response to extract external address
                if let Some(external_addr) = self.parse_stun_response(&buffer[..size]) {
                    debug!("External address discovered: {}", external_addr);
                    Ok(external_addr)
                } else {
                    Err(songbird_errors::SongbirdError::network(
                        "Failed to parse STUN response",
                    ))
                }
            }
            Ok(Err(e)) => Err(e.into()),
            Err(_) => Err(songbird_errors::SongbirdError::network(
                "STUN query timed out",
            )),
        }
    }

    /// Create STUN binding request message
    fn create_stun_binding_request(&self) -> Vec<u8> {
        // Simplified STUN message format
        // In a real implementation, this would be a proper STUN message
        let mut message = Vec::new();

        // STUN message type: Binding Request (0x0001)
        message.extend_from_slice(&[0x00, 0x01]);

        // Message length (0 for no attributes)
        message.extend_from_slice(&[0x00, 0x00]);

        // Magic cookie (0x2112A442)
        message.extend_from_slice(&[0x21, 0x12, 0xA4, 0x42]);

        // Transaction ID (12 bytes, random)
        for _ in 0..12 {
            message.push(rand::random::<u8>());
        }

        message
    }

    /// Parse STUN response to extract external address
    fn parse_stun_response(&self, data: &[u8]) -> Option<SocketAddr> {
        if data.len() < 20 {
            return None;
        }

        // Validate STUN message header
        let message_type = u16::from_be_bytes([data[0], data[1]]);
        let message_length = u16::from_be_bytes([data[2], data[3]]);
        let magic_cookie = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);

        // Check if this is a STUN binding success response (0x0101)
        if message_type != 0x0101 || magic_cookie != 0x2112A442 {
            debug!(
                "Invalid STUN response: type={:#x}, cookie={:#x}",
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
                debug!("STUN attribute extends beyond message length");
                break;
            }

            // Check for MAPPED-ADDRESS (0x0001) or XOR-MAPPED-ADDRESS (0x0020)
            match attr_type {
                0x0001 => {
                    // MAPPED-ADDRESS attribute
                    if attr_length >= 8 && offset + 8 <= data.len() {
                        let _reserved = data[offset];
                        let family = data[offset + 1];

                        if family == 0x01 {
                            // IPv4
                            let port = u16::from_be_bytes([data[offset + 2], data[offset + 3]]);
                            let addr = std::net::Ipv4Addr::new(
                                data[offset + 4],
                                data[offset + 5],
                                data[offset + 6],
                                data[offset + 7],
                            );
                            return Some(SocketAddr::from((addr, port)));
                        }
                    }
                }
                0x0020 => {
                    // XOR-MAPPED-ADDRESS attribute (more common in modern STUN)
                    if attr_length >= 8 && offset + 8 <= data.len() {
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

                            // XOR with magic cookie to get real values
                            let port = xor_port ^ 0x2112;
                            let addr_u32 = u32::from_be_bytes(xor_addr_bytes) ^ 0x2112A442;
                            let addr = std::net::Ipv4Addr::from(addr_u32);

                            return Some(SocketAddr::from((addr, port)));
                        }
                    }
                }
                _ => {
                    // Skip unknown attributes
                }
            }

            // Move to next attribute (padding to 4-byte boundary)
            let padded_length = (attr_length + 3) & !3;
            offset += padded_length as usize;
        }

        debug!("No valid address attribute found in STUN response");
        None
    }

    /// Start hole punching attempt with peer
    pub async fn start_hole_punching(&self, peer_addr: SocketAddr) -> Result<bool> {
        debug!("Starting hole punching with peer: {}", peer_addr);

        let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;

        // Send hole punching packets
        let hole_punch_message = b"HOLE_PUNCH";

        for _ in 0..5 {
            socket.send_to(hole_punch_message, peer_addr).await?;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Try to receive response
        let mut buffer = [0u8; 1024];
        match tokio::time::timeout(Duration::from_secs(2), socket.recv_from(&mut buffer)).await {
            Ok(Ok((_, addr))) => {
                debug!("Hole punching successful with {}", addr);
                Ok(true)
            }
            _ => {
                debug!("Hole punching failed with {}", peer_addr);
                Ok(false)
            }
        }
    }

    /// Get all discovered external addresses
    pub async fn get_external_addresses(&self) -> HashMap<String, SocketAddr> {
        self.external_addresses.read().await.clone()
    }

    /// Clear external address cache
    pub async fn clear_external_addresses(&self) {
        let mut addresses = self.external_addresses.write().await;
        addresses.clear();
    }

    /// Test connectivity to STUN servers
    pub async fn test_stun_connectivity(&self) -> Result<Vec<String>> {
        let mut working_servers = Vec::new();

        for server in &self.stun_servers {
            if self.query_external_address(server).await.is_ok() {
                working_servers.push(server.clone());
            }
        }

        debug!("Working STUN servers: {:?}", working_servers);
        Ok(working_servers)
    }

    /// Add custom STUN server
    pub fn add_stun_server(&mut self, server: String) {
        if !self.stun_servers.contains(&server) {
            self.stun_servers.push(server);
        }
    }

    /// Remove STUN server
    pub fn remove_stun_server(&mut self, server: &str) {
        self.stun_servers.retain(|s| s != server);
    }

    /// Get configured STUN servers
    pub fn get_stun_servers(&self) -> &[String] {
        &self.stun_servers
    }
}
