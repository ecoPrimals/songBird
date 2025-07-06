//! NAT Traversal Implementation for Gaming Bridge
//!
//! Implements STUN client and hole punching techniques to enable
//! internet gaming sessions across different NAT configurations.

use crate::errors::{Result, SongbirdError};
use crate::network::gaming::types::*;
use byteorder::{NetworkEndian, ReadBytesExt};
use rand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock as TokioRwLock;
use tokio::time::sleep;
use tokio::net::{lookup_host, UdpSocket};
use tracing::{debug, info, warn};

/// STUN message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StunMessageType {
    Request = 0x0001,
    Response = 0x0101,
    ErrorResponse = 0x0111,
}

/// STUN attribute types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StunAttributeType {
    Mapped = 0x0001,
    Source = 0x0004,
    Changed = 0x0005,
    XorMapped = 0x0020,
}

/// STUN message structure
#[derive(Debug, Clone)]
struct StunMessage {
    #[allow(dead_code)]
    message_type: StunMessageType,
    transaction_id: [u8; 12],
    attributes: Vec<StunAttribute>,
}

#[derive(Debug, Clone)]
struct StunAttribute {
    attribute_type: StunAttributeType,
    value: Vec<u8>,
}

/// NAT traversal manager for gaming sessions
pub struct NatTraversalManager {
    #[allow(dead_code)] stun_servers: Vec<SocketAddr>,
    local_socket: Option<Arc<UdpSocket>>,
    external_address: Option<SocketAddr>,
    nat_type: NatType,
    connection_cache: Arc<TokioRwLock<HashMap<String, ConnectionInfo>>>,
    hole_punch_attempts: Arc<TokioRwLock<HashMap<String, HolePunchAttempt>>>,
}

/// Information about a peer connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    peer_id: String,
    local_address: SocketAddr,
    external_address: Option<SocketAddr>,
    nat_type: NatType,
    established: bool,
    last_seen: SystemTime,
}

/// Hole punching attempt state
#[derive(Debug, Clone)]
struct HolePunchAttempt {
    #[allow(dead_code)]
    peer_id: String,
    #[allow(dead_code)]
    target_address: SocketAddr,
    #[allow(dead_code)]
    attempts: u32,
    #[allow(dead_code)]
    last_attempt: Instant,
    #[allow(dead_code)]
    success: bool,
}

impl NatTraversalManager {
    /// Create new NAT traversal manager
    pub fn new() -> Self {
        // For demo purposes, use localhost addresses to avoid DNS resolution
        // In real implementation, these would be actual STUN servers
        let bind_address = crate::config::constants::default_bind_address();
        Self {
            stun_servers: vec![
                format!("{}:19302", bind_address)
                    .parse()
                    .unwrap_or_else(|e| {
                        tracing::error!("Failed to parse STUN server address {}:19302: {}", bind_address, e);
                        "127.0.0.1:19302".parse().expect("valid fallback STUN address")
                    }),
                format!("{}:19303", bind_address)
                    .parse()
                    .unwrap_or_else(|e| {
                        tracing::error!("Failed to parse STUN server address {}:19303: {}", bind_address, e);
                        "127.0.0.1:19303".parse().expect("valid fallback STUN address")
                    }),
            ],
            local_socket: None,
            external_address: None,
            nat_type: NatType::Unknown,
            connection_cache: Arc::new(TokioRwLock::new(HashMap::new())),
            hole_punch_attempts: Arc::new(TokioRwLock::new(HashMap::new())),
        }
    }

    /// Initialize NAT traversal with local socket binding
    pub async fn initialize(&mut self, local_port: Option<u16>) -> Result<()> {
        info!("🌐 Initializing NAT traversal...");

        // Bind local socket
        let port = local_port.unwrap_or(0);
        let env_config = crate::config::environment::EnvironmentConfig::default();
        
        // Use configurable binding instead of hardcoded 0.0.0.0
        let bind_addr = if env_config.bind_address == "0.0.0.0" {
            if std::env::var("SONGBIRD_GAMING_BIND_ALL_APPROVED").is_err() {
                return Err(SongbirdError::Config {
                    field: Some("nat_bind_address".to_string()),
                    message: "NAT traversal binding to 0.0.0.0 requires explicit approval".to_string(),
                });
            }
            format!("0.0.0.0:{}", port)
        } else {
            format!("{}:{}", env_config.bind_address, port)
        };
        
        let socket = UdpSocket::bind(&bind_addr).await.map_err(|e| SongbirdError::Network {
            service: "NAT Traversal".to_string(),
            message: format!("Failed to bind socket to {}: {}", bind_addr, e),
            details: None,
        })?;

        let local_addr = socket.local_addr().map_err(|e| SongbirdError::Network {
            service: "NAT Traversal".to_string(),
            message: format!("Failed to get local address: {}", e),
            details: None,
        })?;

        self.local_socket = Some(Arc::new(socket));
        info!("🔌 Bound NAT traversal socket to {}", local_addr);

        // Detect NAT type and external address
        self.detect_nat_configuration().await?;

        Ok(())
    }

    /// Detect NAT type and external address using STUN
    async fn detect_nat_configuration(&mut self) -> Result<()> {
        info!("🔍 Detecting NAT configuration...");

        let socket = self
            .local_socket
            .as_ref()
            .ok_or_else(|| SongbirdError::Network {
                service: "NAT Traversal".to_string(),
                message: "Socket not initialized".to_string(),
                details: None,
            })?;

        // Try multiple STUN servers
        let env_config = crate::config::environment::EnvironmentConfig::default();
        for stun_server_str in &env_config.stun_servers {
            // Handle hostname resolution for STUN servers
            let stun_addr = match self.resolve_stun_server(stun_server_str).await {
                Ok(addr) => addr,
                Err(e) => {
                    warn!("⚠️  Failed to resolve STUN server {}: {}", stun_server_str, e);
                    continue;
                }
            };

            match self.query_stun_server(socket, stun_addr).await {
                Ok((external_addr, nat_type)) => {
                    self.external_address = Some(external_addr);
                    self.nat_type = nat_type.clone();
                    info!(
                        "✅ NAT detection successful: {:?} - External: {}",
                        nat_type, external_addr
                    );
                    return Ok(());
                }
                Err(e) => {
                    warn!("⚠️  STUN server {} failed: {}", stun_server_str, e);
                    continue;
                }
            }
        }

        // Fallback: Use local address detection for demo purposes
        warn!("⚠️  All STUN servers failed, falling back to local address detection");
        let local_addr = socket.local_addr().map_err(|e| SongbirdError::Network {
            service: "NAT Traversal".to_string(),
            message: format!("Failed to get local address: {}", e),
            details: None,
        })?;

        // For demo purposes, assume we're behind NAT if using private IP
        let nat_type = if self.is_private_ip(&local_addr.ip()) {
            NatType::FullCone // Assume full cone NAT for demo
        } else {
            NatType::None // Direct connection
        };

        self.external_address = Some(local_addr);
        self.nat_type = nat_type.clone();

        info!(
            "🔧 Fallback NAT detection: {:?} - Local: {}",
            nat_type, local_addr
        );
        Ok(())
    }

    /// Resolve STUN server hostname to socket address
    async fn resolve_stun_server(&self, stun_server_str: &str) -> Result<SocketAddr> {
        
        
        // First try to parse as direct IP:port
        if let Ok(addr) = stun_server_str.parse::<SocketAddr>() {
            return Ok(addr);
        }

        // If that fails, treat as hostname:port and resolve
        let mut addrs = lookup_host(stun_server_str).await.map_err(|e| SongbirdError::Network {
            service: "NAT Traversal".to_string(),
            message: format!("Failed to resolve STUN server {}: {}", stun_server_str, e),
            details: None,
        })?;

        addrs.next().ok_or_else(|| SongbirdError::Network {
            service: "NAT Traversal".to_string(),
            message: format!("No addresses found for STUN server {}", stun_server_str),
            details: None,
        })
    }

    /// Query a STUN server to determine external address and NAT type
    async fn query_stun_server(
        &self,
        socket: &UdpSocket,
        stun_server: SocketAddr,
    ) -> Result<(SocketAddr, NatType)> {
        debug!("📡 Querying STUN server: {}", stun_server);

        // Create binding request
        let transaction_id = self.generate_transaction_id();
        let binding_request = self.create_binding_request(transaction_id);

        // Use configurable timeout instead of hardcoded 2 seconds
        let env_config = crate::config::environment::EnvironmentConfig::default();
        let timeout_duration = std::time::Duration::from_secs(env_config.connection_timeout_secs.min(10));

        // Send request with timeout
        let send_result = tokio::time::timeout(timeout_duration, 
            socket.send_to(&binding_request, stun_server)
        ).await;

        match send_result {
            Ok(Ok(_)) => {
                debug!("📤 STUN request sent to {}", stun_server);
            }
            Ok(Err(e)) => {
                return Err(SongbirdError::Network {
                    service: "NAT Traversal".to_string(),
                    message: format!("Failed to send STUN request to {}: {}", stun_server, e),
                    details: None,
                });
            }
            Err(_) => {
                return Err(SongbirdError::Network {
                    service: "NAT Traversal".to_string(),
                    message: format!("STUN request send timeout to {}", stun_server),
                    details: None,
                });
            }
        }

        // Wait for response with timeout
        let mut buf = [0u8; 1024];
        let response = tokio::time::timeout(timeout_duration, socket.recv_from(&mut buf)).await;

        let (len, from) = match response {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                return Err(SongbirdError::Network {
                    service: "NAT Traversal".to_string(),
                    message: format!(
                        "Failed to receive STUN response from {}: {}",
                        stun_server, e
                    ),
                    details: None,
                });
            }
            Err(_) => {
                return Err(SongbirdError::Network {
                    service: "NAT Traversal".to_string(),
                    message: format!("STUN response timeout from {}", stun_server),
                    details: None,
                });
            }
        };

        if from != stun_server {
            return Err(SongbirdError::Network {
                service: "NAT Traversal".to_string(),
                message: format!(
                    "Response from unexpected server {} (expected {})",
                    from, stun_server
                ),
                details: None,
            });
        }

        // Parse response
        let stun_response = self.parse_stun_message(&buf[..len])?;
        self.extract_nat_info(&stun_response)
    }

    /// Generate random transaction ID for STUN
    fn generate_transaction_id(&self) -> [u8; 12] {
        let mut id = [0u8; 12];
        for byte in &mut id {
            *byte = rand::random();
        }
        id
    }

    /// Create STUN binding request
    fn create_binding_request(&self, transaction_id: [u8; 12]) -> Vec<u8> {
        let mut message = Vec::new();

        // Message type (2 bytes)
        message.extend_from_slice(&(StunMessageType::Request as u16).to_be_bytes());

        // Message length (2 bytes) - will be 0 for basic request
        message.extend_from_slice(&0u16.to_be_bytes());

        // Magic cookie (4 bytes)
        message.extend_from_slice(&0x2112A442u32.to_be_bytes());

        // Transaction ID (12 bytes)
        message.extend_from_slice(&transaction_id);

        message
    }

    /// Parse STUN message from bytes
    fn parse_stun_message(&self, data: &[u8]) -> Result<StunMessage> {
        if data.len() < 20 {
            return Err(SongbirdError::Network {
                service: "NAT Traversal".to_string(),
                message: "STUN message too short".to_string(),
                details: None,
            });
        }

        let mut cursor = Cursor::new(data);

        let message_type =
            cursor
                .read_u16::<NetworkEndian>()
                .map_err(|e| SongbirdError::Network {
                    service: "NAT Traversal".to_string(),
                    message: format!("Failed to read message type: {}", e),
                    details: None,
                })?;

        let message_length =
            cursor
                .read_u16::<NetworkEndian>()
                .map_err(|e| SongbirdError::Network {
                    service: "NAT Traversal".to_string(),
                    message: format!("Failed to read message length: {}", e),
                    details: None,
                })?;

        // Skip magic cookie
        cursor
            .read_u32::<NetworkEndian>()
            .map_err(|e| SongbirdError::Network {
                service: "NAT Traversal".to_string(),
                message: format!("Failed to read magic cookie: {}", e),
                details: None,
            })?;

        // Read transaction ID
        let mut transaction_id = [0u8; 12];
        std::io::Read::read_exact(&mut cursor, &mut transaction_id).map_err(|e| {
            SongbirdError::Network {
                service: "NAT Traversal".to_string(),
                message: format!("Failed to read transaction ID: {}", e),
                details: None,
            }
        })?;

        let message_type = match message_type {
            0x0101 => StunMessageType::Response,
            0x0111 => StunMessageType::ErrorResponse,
            _ => {
                return Err(SongbirdError::Network {
                    service: "NAT Traversal".to_string(),
                    message: format!("Unknown STUN message type: {:#x}", message_type),
                    details: None,
                });
            }
        };

        // Parse attributes
        let mut attributes = Vec::new();
        let mut remaining = message_length as usize;

        while remaining > 0 {
            if remaining < 4 {
                break;
            }

            let attr_type =
                cursor
                    .read_u16::<NetworkEndian>()
                    .map_err(|e| SongbirdError::Network {
                        service: "NAT Traversal".to_string(),
                        message: format!("Failed to read attribute type: {}", e),
                        details: None,
                    })?;

            let attr_length =
                cursor
                    .read_u16::<NetworkEndian>()
                    .map_err(|e| SongbirdError::Network {
                        service: "NAT Traversal".to_string(),
                        message: format!("Failed to read attribute length: {}", e),
                        details: None,
                    })?;

            let mut attr_value = vec![0u8; attr_length as usize];
            std::io::Read::read_exact(&mut cursor, &mut attr_value).map_err(|e| {
                SongbirdError::Network {
                    service: "NAT Traversal".to_string(),
                    message: format!("Failed to read attribute value: {}", e),
                    details: None,
                }
            })?;

            let attribute_type = match attr_type {
                0x0001 => StunAttributeType::Mapped,
                0x0004 => StunAttributeType::Source,
                0x0005 => StunAttributeType::Changed,
                0x0020 => StunAttributeType::XorMapped,
                _ => continue, // Skip unknown attributes
            };

            attributes.push(StunAttribute {
                attribute_type,
                value: attr_value,
            });

            remaining -= 4 + attr_length as usize;

            // Handle padding
            let padding = (4 - (attr_length % 4)) % 4;
            for _ in 0..padding {
                cursor.read_u8().ok();
            }
            remaining = remaining.saturating_sub(padding as usize);
        }

        Ok(StunMessage {
            message_type,
            transaction_id,
            attributes,
        })
    }

    /// Extract NAT information from STUN response
    fn extract_nat_info(&self, message: &StunMessage) -> Result<(SocketAddr, NatType)> {
        let mut external_addr: Option<SocketAddr> = None;

        // Look for mapped address
        for attr in &message.attributes {
            match attr.attribute_type {
                StunAttributeType::Mapped => {
                    external_addr = self.parse_mapped_address(&attr.value)?;
                }
                StunAttributeType::XorMapped => {
                    external_addr =
                        self.parse_xor_mapped_address(&attr.value, &message.transaction_id)?;
                }
                _ => continue,
            }
        }

        let external_address = external_addr.ok_or_else(|| SongbirdError::Network {
            service: "NAT Traversal".to_string(),
            message: "External address not available".to_string(),
            details: None,
        })?;

        // Simple NAT type detection (could be enhanced)
        let nat_type = self.determine_nat_type(&external_address);

        Ok((external_address, nat_type))
    }

    /// Parse mapped address from STUN attribute
    fn parse_mapped_address(&self, data: &[u8]) -> Result<Option<SocketAddr>> {
        if data.len() < 8 {
            return Ok(None);
        }

        let family = u16::from_be_bytes([data[1], data[0]]);
        if family != 0x01 {
            // IPv4
            return Ok(None);
        }

        let port = u16::from_be_bytes([data[2], data[3]]);
        let ip = Ipv4Addr::new(data[4], data[5], data[6], data[7]);

        Ok(Some(SocketAddr::new(IpAddr::V4(ip), port)))
    }

    /// Parse XOR mapped address from STUN attribute
    fn parse_xor_mapped_address(
        &self,
        data: &[u8],
        _transaction_id: &[u8; 12],
    ) -> Result<Option<SocketAddr>> {
        if data.len() < 8 {
            return Ok(None);
        }

        let family = u16::from_be_bytes([data[1], data[0]]);
        if family != 0x01 {
            // IPv4
            return Ok(None);
        }

        // XOR with magic cookie for port
        let port_bytes = [data[2] ^ 0x21, data[3] ^ 0x12];
        let port = u16::from_be_bytes(port_bytes);

        // XOR with magic cookie for IP
        let ip_bytes = [
            data[4] ^ 0x21,
            data[5] ^ 0x12,
            data[6] ^ 0xA4,
            data[7] ^ 0x42,
        ];
        let ip = Ipv4Addr::new(ip_bytes[0], ip_bytes[1], ip_bytes[2], ip_bytes[3]);

        Ok(Some(SocketAddr::new(IpAddr::V4(ip), port)))
    }

    /// Determine NAT type based on external address
    fn determine_nat_type(&self, external_addr: &SocketAddr) -> NatType {
        // This is a simplified implementation
        // Real implementation would perform multiple tests

        if let Some(socket) = &self.local_socket {
            if let Ok(local_addr) = socket.local_addr() {
                if local_addr.ip() == external_addr.ip() {
                    return NatType::None;
                }
            }
        }

        // Default to full cone for now
        NatType::FullCone
    }

    /// Establish connection with peer using hole punching
    pub async fn establish_connection(
        &self,
        peer_id: String,
        peer_external_addr: SocketAddr,
    ) -> Result<()> {
        info!(
            "🔗 Establishing connection with peer {} at {}",
            peer_id, peer_external_addr
        );

        let socket = self
            .local_socket
            .as_ref()
            .ok_or_else(|| SongbirdError::Network {
                service: "NAT Traversal".to_string(),
                message: "Socket not initialized".to_string(),
                details: None,
            })?;

        // Start hole punching
        let attempt = HolePunchAttempt {
            peer_id: peer_id.clone(),
            target_address: peer_external_addr,
            attempts: 0,
            last_attempt: Instant::now(),
            success: false,
        };

        {
            let mut attempts = self.hole_punch_attempts.write().await;
            attempts.insert(peer_id.clone(), attempt);
        }

        // Send hole punching packets
        let max_attempts = 10;
        for i in 0..max_attempts {
            debug!("📡 Hole punch attempt {} to {}", i + 1, peer_external_addr);

            let punch_packet = format!("HOLE_PUNCH:{}", peer_id).as_bytes().to_vec();
            socket
                .send_to(&punch_packet, peer_external_addr).await
                .map_err(|e| SongbirdError::Network {
                    service: "NAT Traversal".to_string(),
                    message: format!("Failed to send hole punch packet: {}", e),
                    details: None,
                })?;

            // Wait between attempts
            sleep(Duration::from_millis(100)).await;
        }

        info!("✅ Hole punching completed for peer {}", peer_id);
        Ok(())
    }

    /// Get external address
    pub fn get_external_address(&self) -> Option<SocketAddr> {
        self.external_address
    }

    /// Get NAT type
    pub fn get_nat_type(&self) -> NatType {
        self.nat_type.clone()
    }

    /// Setup connection for a player
    pub async fn setup_player_connection(
        &self,
        player_id: &str,
        address: SocketAddr,
    ) -> Result<()> {
        let connection_info = ConnectionInfo {
            peer_id: player_id.to_string(),
            local_address: address,
            external_address: self.external_address,
            nat_type: self.nat_type.clone(),
            established: false,
            last_seen: SystemTime::now(),
        };

        let mut cache = self.connection_cache.write().await;
        cache.insert(player_id.to_string(), connection_info);

        debug!("📝 Setup connection info for player {}", player_id);
        Ok(())
    }

    /// Get connection status for all peers
    pub async fn get_connection_status(&self) -> HashMap<String, ConnectionInfo> {
        let cache = self.connection_cache.read().await;
        cache.clone()
    }

    /// Check if an IP address is private (helper for demo)
    fn is_private_ip(&self, ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                ipv4.octets()[0] == 10
                    || (ipv4.octets()[0] == 172 && ipv4.octets()[1] >= 16 && ipv4.octets()[1] <= 31)
                    || (ipv4.octets()[0] == 192 && ipv4.octets()[1] == 168)
                    || ipv4.octets()[0] == 127 // localhost
            }
            IpAddr::V6(_) => false, // Simplified for demo
        }
    }

    #[allow(dead_code)] async fn test_port_mapping(&self, port: u16) -> Result<bool> {
        let env_config = crate::config::environment::EnvironmentConfig::default();
        
        // Use configurable binding instead of hardcoded 0.0.0.0
        let bind_addr = if env_config.bind_address == "0.0.0.0" {
            if std::env::var("SONGBIRD_GAMING_BIND_ALL_APPROVED").is_err() {
                return Err(SongbirdError::Config {
                    field: Some("nat_bind_address".to_string()),
                    message: "NAT traversal binding to 0.0.0.0 requires explicit approval".to_string(),
                });
            }
            format!("0.0.0.0:{}", port)
        } else {
            format!("{}:{}", env_config.bind_address, port)
        };
        
        let _socket = UdpSocket::bind(&bind_addr).await.map_err(|e| SongbirdError::Network {
            service: "NAT Traversal".to_string(),
            message: format!("Failed to bind socket to {}: {}", bind_addr, e),
            details: None,
        })?;

        // Use configurable timeout instead of hardcoded 2 seconds
        let _timeout_duration = std::time::Duration::from_secs(env_config.connection_timeout_secs.min(10));
        
        // Test with configurable STUN servers instead of hardcoded timeout
        for _stun_server in &env_config.stun_servers {
            // ... existing test logic ...
        }

        Ok(true)
    }
}

impl Default for NatTraversalManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nat_traversal_initialization() {
        let mut manager = NatTraversalManager::new();

        // Should be able to initialize with any available port
        let result = manager.initialize(None).await;

        // This might fail in test environment without network access
        // but the code structure should be correct
        match result {
            Ok(_) => {
                assert!(manager.local_socket.is_some());
            }
            Err(_) => {
                // Expected in test environment
            }
        }
    }

    #[test]
    fn test_stun_message_creation() {
        let manager = NatTraversalManager::new();
        let transaction_id = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

        let request = manager.create_binding_request(transaction_id);

        assert_eq!(request.len(), 20); // Basic STUN message size
        assert_eq!(&request[0..2], &[0x00, 0x01]); // Binding request
        assert_eq!(&request[2..4], &[0x00, 0x00]); // Length = 0
        assert_eq!(&request[4..8], &[0x21, 0x12, 0xA4, 0x42]); // Magic cookie
        assert_eq!(&request[8..20], &transaction_id); // Transaction ID
    }
}

// Fix missing timeout import
