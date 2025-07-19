//! STUN client for NAT traversal

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::super::beardog_integration::PeerCapabilities;
use super::types::DiscoveryConfig;
use songbird_errors::Result;

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
    pub async fn discover_peers(&self) -> Result<Vec<PeerCapabilities>> {
        debug!("Discovering peers via STUN...");

        let mut peers = Vec::new();
        
        // Get external address for NAT traversal
        let external_addr = self.get_external_address().await?;
        info!("External address discovered: {}", external_addr);

        // In a real implementation, this would use techniques like:
        // 1. STUN binding requests to discover external address
        // 2. Coordinated peer discovery through STUN servers
        // 3. Hole punching for direct peer-to-peer connections
        // 4. ICE (Interactive Connectivity Establishment) for optimal path selection

        // For now, we'll simulate peer discovery through STUN coordination
        for server in &self.stun_servers {
            if let Ok(discovered_peer_capabilities) = self.query_stun_server(server).await {
                if let Some(capabilities) = discovered_peer_capabilities {
                    peers.push(capabilities);
                }
            }
        }

        debug!("STUN discovery found {} potential peers", peers.len());
        Ok(peers)
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
        
        Err(songbird_errors::SongbirdError::network_error(
            "Failed to determine external address via STUN"
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
                    Err(songbird_errors::SongbirdError::network_error(
                        "Failed to parse STUN response"
                    ))
                }
            }
            Ok(Err(e)) => Err(e.into()),
            Err(_) => Err(songbird_errors::SongbirdError::network_error(
                "STUN query timed out"
            )),
        }
    }

    /// Query STUN server for peer discovery
    async fn query_stun_server(&self, server: &str) -> Result<Option<PeerCapabilities>> {
        debug!("Querying STUN server {} for peer discovery", server);

        // In a real implementation, this would:
        // 1. Register with the STUN server
        // 2. Query for other registered peers
        // 3. Coordinate hole punching attempts
        // 4. Measure connectivity and capabilities

        // For now, provide mock capabilities based on STUN server response
        if self.query_external_address(server).await.is_ok() {
            Ok(Some(PeerCapabilities {
                protocol_support: vec![
                    "STUN".to_string(),
                    "ICE".to_string(),
                    "UDP".to_string(),
                ],
                bandwidth_mbps: 50, // Conservative estimate for internet peers
                latency_ms: 25,     // Typical internet latency
                gaming_optimized: false,
                security_level: crate::network::beardog_integration::SecurityLevel::Standard,
            }))
        } else {
            Ok(None)
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

        // Very simplified STUN response parsing
        // In a real implementation, this would properly parse STUN attributes
        
        // For now, extract a mock external address based on the response
        // This would normally come from the XOR-MAPPED-ADDRESS attribute
        let ip = std::net::Ipv4Addr::new(203, 0, 113, 1); // Example external IP
        let port = 12345; // Example external port
        
        Some(SocketAddr::from((ip, port)))
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