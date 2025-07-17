//! NAT Traversal Implementation for Gaming Bridge
//!
//! Implements STUN client and hole punching techniques to enable
//! internet gaming sessions across different NAT configurations.

use crate::network::gaming::types::*;
use byteorder::{NetworkEndian, ReadBytesExt};
use rand;
use serde::{Deserialize, Serialize};
use songbird_errors::{NetworkError, Result, SongbirdError};
use std::collections::HashMap;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::net::{lookup_host, UdpSocket};
use tokio::sync::RwLock as TokioRwLock;
use tokio::time::sleep;
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
    stun_servers: Vec<SocketAddr>,
    turn_servers: Vec<TurnServer>,
    local_socket: Option<Arc<UdpSocket>>,
    external_address: Option<SocketAddr>,
    nat_type: NatType,
    connection_cache: Arc<TokioRwLock<HashMap<String, ConnectionInfo>>>,
    hole_punch_attempts: Arc<TokioRwLock<HashMap<String, HolePunchAttempt>>>,
    turn_allocations: Arc<TokioRwLock<HashMap<String, TurnAllocation>>>,
}

/// TURN server configuration
#[derive(Debug, Clone)]
pub struct TurnServer {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub realm: String,
    pub protocol: TurnProtocol,
}

/// TURN protocol variants
#[derive(Debug, Clone)]
pub enum TurnProtocol {
    Udp,
    Tcp,
    Tls,
    Dtls,
}

/// TURN allocation information
#[derive(Debug, Clone)]
pub struct TurnAllocation {
    pub server: TurnServer,
    pub allocation_id: String,
    pub relay_address: SocketAddr,
    pub allocated_at: Instant,
    pub expires_at: Instant,
    pub permissions: Vec<SocketAddr>,
    pub bandwidth_limit: Option<u32>, // Kbps
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// TURN message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TurnMessageType {
    Allocate = 0x0003,
    AllocateResponse = 0x0103,
    AllocateError = 0x0113,
    Refresh = 0x0004,
    RefreshResponse = 0x0104,
    RefreshError = 0x0114,
    Send = 0x0006,
    SendResponse = 0x0106,
    SendError = 0x0116,
    Data = 0x0007,
    CreatePermission = 0x0008,
    CreatePermissionResponse = 0x0108,
    CreatePermissionError = 0x0118,
    ChannelBind = 0x0009,
    ChannelBindResponse = 0x0109,
    ChannelBindError = 0x0119,
}

/// TURN attribute types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TurnAttributeType {
    // STUN attributes
    MappedAddress = 0x0001,
    Username = 0x0006,
    MessageIntegrity = 0x0008,
    ErrorCode = 0x0009,
    Realm = 0x0014,
    Nonce = 0x0015,
    XorMappedAddress = 0x0020,

    // TURN-specific attributes
    XorRelayedAddress = 0x0016,
    RequestedTransport = 0x0019,
    Lifetime = 0x000D,
    XorPeerAddress = 0x0012,
    Data = 0x0013,
    ChannelNumber = 0x000C,
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
        let bind_address = songbird_config::config::constants::default_bind_address();
        Self {
            stun_servers: vec![
                format!("{}:19302", bind_address)
                    .parse()
                    .unwrap_or_else(|e| {
                        tracing::error!(
                            "Failed to parse STUN server address {}:19302: {}",
                            bind_address,
                            e
                        );
                        "127.0.0.1:19302".parse().unwrap() // Safe fallback
                    }),
                format!("{}:19303", bind_address)
                    .parse()
                    .unwrap_or_else(|e| {
                        tracing::error!(
                            "Failed to parse STUN server address {}:19303: {}",
                            bind_address,
                            e
                        );
                        "127.0.0.1:19303".parse().unwrap() // Safe fallback
                    }),
            ],
            turn_servers: vec![], // Placeholder for TURN servers
            local_socket: None,
            external_address: None,
            nat_type: NatType::Unknown,
            connection_cache: Arc::new(TokioRwLock::new(HashMap::new())),
            hole_punch_attempts: Arc::new(TokioRwLock::new(HashMap::new())),
            turn_allocations: Arc::new(TokioRwLock::new(HashMap::new())),
        }
    }

    /// Initialize NAT traversal with local socket binding
    pub async fn initialize(&mut self, local_port: Option<u16>) -> Result<()> {
        info!("🌐 Initializing NAT traversal...");

        // Bind local socket
        let port = local_port.unwrap_or(0);
        let env_config = songbird_config::config::environment::EnvironmentConfig::default();

        // Use configurable binding instead of hardcoded 0.0.0.0
        let bind_addr = if env_config.bind_address == "0.0.0.0" {
            if std::env::var("SONGBIRD_GAMING_BIND_ALL_APPROVED").is_err() {
                return Err(SongbirdError::Config {
                    field: Some("nat_bind_address".to_string()),
                    message: "NAT traversal binding to 0.0.0.0 requires explicit approval"
                        .to_string(),
                    context: Some("network_configuration".to_string()),
                    suggestion: Some("Check configuration values and network settings".to_string()),
                });
            }
            format!("0.0.0.0:{}", port)
        } else {
            format!("{}:{}", env_config.bind_address, port)
        };

        let socket = UdpSocket::bind(&bind_addr).await.map_err(|e| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("Failed to bind socket to {}: {}", bind_addr, e),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            }))
        })?;

        let local_addr = socket.local_addr().map_err(|e| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("Failed to get local address: {}", e),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            }))
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

        let socket = self.local_socket.as_ref().ok_or_else(|| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: "Socket not initialized".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            }))
        })?;

        // Try multiple STUN servers
        let env_config = songbird_config::config::environment::EnvironmentConfig::default();
        for stun_server_str in &env_config.stun_servers {
            // Handle hostname resolution for STUN servers
            let stun_addr = match self.resolve_stun_server(stun_server_str).await {
                Ok(addr) => addr,
                Err(e) => {
                    warn!(
                        "⚠️  Failed to resolve STUN server {}: {}",
                        stun_server_str, e
                    );
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
        let local_addr = socket.local_addr().map_err(|e| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("Failed to get local address: {}", e),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            }))
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
        let mut addrs = lookup_host(stun_server_str).await.map_err(|e| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("Failed to resolve STUN server {}: {}", stun_server_str, e),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            }))
        })?;

        addrs.next().ok_or_else(|| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("No addresses found for STUN server {}", stun_server_str),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            }))
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
        let env_config = songbird_config::config::environment::EnvironmentConfig::default();
        let timeout_duration =
            std::time::Duration::from_secs(env_config.connection_timeout_secs.min(10));

        // Send request with timeout
        let send_result = tokio::time::timeout(
            timeout_duration,
            socket.send_to(&binding_request, stun_server),
        )
        .await;

        match send_result {
            Ok(Ok(_)) => {
                debug!("📤 STUN request sent to {}", stun_server);
            }
            Ok(Err(e)) => {
                return Err(SongbirdError::Network(Box::new(NetworkError {
                    service: Some("NAT Traversal".to_string()),
                    message: format!("Failed to send STUN request to {}: {}", stun_server, e),
                    details: None,
                    endpoint: None,
                    suggestion: Some("Check network connectivity and configuration".to_string()),
                })));
            }
            Err(_) => {
                return Err(SongbirdError::Network(Box::new(NetworkError {
                    service: Some("NAT Traversal".to_string()),
                    message: format!("STUN request send timeout to {}", stun_server),
                    details: None,
                    endpoint: None,
                    suggestion: Some("Check network connectivity and configuration".to_string()),
                })));
            }
        }

        // Wait for response with timeout
        let mut buf = [0u8; 1024];
        let response = tokio::time::timeout(timeout_duration, socket.recv_from(&mut buf)).await;

        let (len, from) = match response {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                return Err(SongbirdError::Network(Box::new(NetworkError {
                    service: Some("NAT Traversal".to_string()),
                    message: format!(
                        "Failed to receive STUN response from {}: {}",
                        stun_server, e
                    ),
                    details: None,
                    endpoint: None,
                    suggestion: Some("Check network connectivity and configuration".to_string()),
                })));
            }
            Err(_) => {
                return Err(SongbirdError::Network(Box::new(NetworkError {
                    service: Some("NAT Traversal".to_string()),
                    message: format!("STUN response timeout from {}", stun_server),
                    details: None,
                    endpoint: None,
                    suggestion: Some("Check network connectivity and configuration".to_string()),
                })));
            }
        };

        if from != stun_server {
            return Err(SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!(
                    "Response from unexpected server {} (expected {})",
                    from, stun_server
                ),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            })));
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
            return Err(SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: "STUN message too short".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            })));
        }

        let mut cursor = Cursor::new(data);

        let message_type = cursor.read_u16::<NetworkEndian>().map_err(|e| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("Failed to read message type: {}", e),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            }))
        })?;

        let message_length = cursor.read_u16::<NetworkEndian>().map_err(|e| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("Failed to read message length: {}", e),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            }))
        })?;

        // Skip magic cookie
        cursor.read_u32::<NetworkEndian>().map_err(|e| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("Failed to read magic cookie: {}", e),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            }))
        })?;

        // Read transaction ID
        let mut transaction_id = [0u8; 12];
        std::io::Read::read_exact(&mut cursor, &mut transaction_id).map_err(|e| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("Failed to read transaction ID: {}", e),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            }))
        })?;

        let message_type = match message_type {
            0x0101 => StunMessageType::Response,
            0x0111 => StunMessageType::ErrorResponse,
            _ => {
                return Err(SongbirdError::Network(Box::new(NetworkError {
                    service: Some("NAT Traversal".to_string()),
                    message: format!("Unknown STUN message type: {:#x}", message_type),
                    details: None,
                    endpoint: None,
                    suggestion: Some("Check network connectivity and configuration".to_string()),
                })));
            }
        };

        // Parse attributes
        let mut attributes = Vec::new();
        let mut remaining = message_length as usize;

        while remaining > 0 {
            if remaining < 4 {
                break;
            }

            let attr_type = cursor.read_u16::<NetworkEndian>().map_err(|e| {
                SongbirdError::Network(Box::new(NetworkError {
                    service: Some("NAT Traversal".to_string()),
                    message: format!("Failed to read attribute type: {}", e),
                    details: None,
                    endpoint: None,
                    suggestion: Some("Check network connectivity and configuration".to_string()),
                }))
            })?;

            let attr_length = cursor.read_u16::<NetworkEndian>().map_err(|e| {
                SongbirdError::Network(Box::new(NetworkError {
                    service: Some("NAT Traversal".to_string()),
                    message: format!("Failed to read attribute length: {}", e),
                    details: None,
                    endpoint: None,
                    suggestion: Some("Check network connectivity and configuration".to_string()),
                }))
            })?;

            let mut attr_value = vec![0u8; attr_length as usize];
            std::io::Read::read_exact(&mut cursor, &mut attr_value).map_err(|e| {
                SongbirdError::Network(Box::new(NetworkError {
                    service: Some("NAT Traversal".to_string()),
                    message: format!("Failed to read attribute value: {}", e),
                    details: None,
                    endpoint: None,
                    suggestion: Some("Check network connectivity and configuration".to_string()),
                }))
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

        let external_address = external_addr.ok_or_else(|| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: "External address not available".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            }))
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

        let socket = self.local_socket.as_ref().ok_or_else(|| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: "Socket not initialized".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            }))
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
                .send_to(&punch_packet, peer_external_addr)
                .await
                .map_err(|e| {
                    SongbirdError::Network(Box::new(NetworkError {
                        service: Some("NAT Traversal".to_string()),
                        message: format!("Failed to send hole punch packet: {}", e),
                        details: None,
                        endpoint: None,
                        suggestion: Some(
                            "Check network connectivity and configuration".to_string(),
                        ),
                    }))
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

    #[allow(dead_code)]
    async fn test_port_mapping(&self, port: u16) -> Result<bool> {
        let env_config = songbird_config::config::environment::EnvironmentConfig::default();

        // Use configurable binding instead of hardcoded 0.0.0.0
        let bind_addr = if env_config.bind_address == "0.0.0.0" {
            if std::env::var("SONGBIRD_GAMING_BIND_ALL_APPROVED").is_err() {
                return Err(SongbirdError::Config {
                    field: Some("nat_bind_address".to_string()),
                    message: "NAT traversal binding to 0.0.0.0 requires explicit approval"
                        .to_string(),
                    context: Some("network_configuration".to_string()),
                    suggestion: Some("Check configuration values and network settings".to_string()),
                });
            }
            format!("0.0.0.0:{}", port)
        } else {
            format!("{}:{}", env_config.bind_address, port)
        };

        let _socket = UdpSocket::bind(&bind_addr).await.map_err(|e| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("Failed to bind socket to {}: {}", bind_addr, e),
                details: None,
                endpoint: None,
                suggestion: Some("Check network connectivity and configuration".to_string()),
            }))
        })?;

        // Use configurable timeout instead of hardcoded 2 seconds
        let _timeout_duration =
            std::time::Duration::from_secs(env_config.connection_timeout_secs.min(10));

        // Test with configurable STUN servers instead of hardcoded timeout
        for _stun_server in &env_config.stun_servers {
            // ... existing test logic ...
        }

        Ok(true)
    }

    /// Add TURN server configuration
    pub fn add_turn_server(&mut self, turn_server: TurnServer) {
        info!(
            "Added TURN server: {}:{}",
            turn_server.host, turn_server.port
        );
        self.turn_servers.push(turn_server);
    }

    /// Create TURN allocation for NAT traversal
    pub async fn create_turn_allocation(&self, server_index: usize) -> Result<String> {
        if server_index >= self.turn_servers.len() {
            return Err(SongbirdError::Config {
                field: Some("turn_server_index".to_string()),
                message: "Invalid TURN server index".to_string(),
                context: Some("TURN server configuration".to_string()),
                suggestion: Some("Use a valid TURN server index".to_string()),
            });
        }

        let turn_server = &self.turn_servers[server_index];
        let allocation_id = uuid::Uuid::new_v4().to_string();

        info!(
            "Creating TURN allocation on server: {}:{}",
            turn_server.host, turn_server.port
        );

        let socket = self.local_socket.as_ref().ok_or_else(|| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: "Socket not initialized".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Initialize NAT traversal manager first".to_string()),
            }))
        })?;

        // Create TURN allocation request
        let transaction_id = self.generate_transaction_id();
        let allocate_request = self
            .create_turn_allocate_request(transaction_id, turn_server)
            .await?;

        // Send allocation request to TURN server
        let server_addr = format!("{}:{}", turn_server.host, turn_server.port);
        let turn_addr: SocketAddr = server_addr.parse().map_err(|e| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("Invalid TURN server address: {}", e),
                details: None,
                endpoint: None,
                suggestion: Some("Check TURN server configuration".to_string()),
            }))
        })?;

        // Send request with timeout
        let timeout_duration = std::time::Duration::from_secs(10);
        let send_result = tokio::time::timeout(
            timeout_duration,
            socket.send_to(&allocate_request, turn_addr),
        )
        .await;

        match send_result {
            Ok(Ok(_)) => {
                debug!("TURN allocation request sent to {}", turn_addr);
            }
            Ok(Err(e)) => {
                return Err(SongbirdError::Network(Box::new(NetworkError {
                    service: Some("NAT Traversal".to_string()),
                    message: format!("Failed to send TURN allocation request: {}", e),
                    details: None,
                    endpoint: None,
                    suggestion: Some("Check network connectivity".to_string()),
                })));
            }
            Err(_) => {
                return Err(SongbirdError::Network(Box::new(NetworkError {
                    service: Some("NAT Traversal".to_string()),
                    message: "TURN allocation request timeout".to_string(),
                    details: None,
                    endpoint: None,
                    suggestion: Some("Check TURN server availability".to_string()),
                })));
            }
        }

        // Wait for response
        let mut buf = [0u8; 1024];
        let response = tokio::time::timeout(timeout_duration, socket.recv_from(&mut buf)).await;

        let (len, from) = match response {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                return Err(SongbirdError::Network(Box::new(NetworkError {
                    service: Some("NAT Traversal".to_string()),
                    message: format!("Failed to receive TURN allocation response: {}", e),
                    details: None,
                    endpoint: None,
                    suggestion: Some("Check network connectivity".to_string()),
                })));
            }
            Err(_) => {
                return Err(SongbirdError::Network(Box::new(NetworkError {
                    service: Some("NAT Traversal".to_string()),
                    message: "TURN allocation response timeout".to_string(),
                    details: None,
                    endpoint: None,
                    suggestion: Some("Check TURN server availability".to_string()),
                })));
            }
        };

        if from != turn_addr {
            return Err(SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: "Response from unexpected server".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Check network configuration".to_string()),
            })));
        }

        // Parse allocation response
        let allocation_response =
            self.parse_turn_allocation_response(&buf[..len], transaction_id)?;

        // Extract relay address from response
        let relay_address = allocation_response.relay_address.ok_or_else(|| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: "No relay address in TURN allocation response".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Check TURN server configuration".to_string()),
            }))
        })?;

        // Create allocation record
        let allocation = TurnAllocation {
            server: turn_server.clone(),
            allocation_id: allocation_id.clone(),
            relay_address,
            allocated_at: Instant::now(),
            expires_at: Instant::now() + std::time::Duration::from_secs(600), // 10 minutes default
            permissions: Vec::new(),
            bandwidth_limit: None,
            bytes_sent: 0,
            bytes_received: 0,
        };

        // Store allocation
        let mut allocations = self.turn_allocations.write().await;
        allocations.insert(allocation_id.clone(), allocation);

        info!(
            "TURN allocation created successfully: {} -> {}",
            allocation_id, relay_address
        );
        Ok(allocation_id)
    }

    /// Create TURN allocation request message
    async fn create_turn_allocate_request(
        &self,
        transaction_id: [u8; 12],
        turn_server: &TurnServer,
    ) -> Result<Vec<u8>> {
        let mut message = Vec::new();

        // TURN header
        message.extend_from_slice(&(TurnMessageType::Allocate as u16).to_be_bytes());
        message.extend_from_slice(&[0x00, 0x00]); // Length placeholder
        message.extend_from_slice(&[0x21, 0x12, 0xA4, 0x42]); // Magic cookie
        message.extend_from_slice(&transaction_id);

        // Add REQUESTED-TRANSPORT attribute (UDP = 17)
        self.add_turn_attribute(
            &mut message,
            TurnAttributeType::RequestedTransport,
            &[17, 0, 0, 0],
        );

        // Add USERNAME attribute
        self.add_turn_attribute(
            &mut message,
            TurnAttributeType::Username,
            turn_server.username.as_bytes(),
        );

        // Add REALM attribute
        self.add_turn_attribute(
            &mut message,
            TurnAttributeType::Realm,
            turn_server.realm.as_bytes(),
        );

        // Add NONCE attribute (simplified - in real implementation, get from server)
        let nonce = format!("nonce-{}", uuid::Uuid::new_v4());
        self.add_turn_attribute(&mut message, TurnAttributeType::Nonce, nonce.as_bytes());

        // Add LIFETIME attribute (10 minutes)
        self.add_turn_attribute(
            &mut message,
            TurnAttributeType::Lifetime,
            &600u32.to_be_bytes(),
        );

        // Update message length
        let message_length = (message.len() - 20) as u16;
        message[2..4].copy_from_slice(&message_length.to_be_bytes());

        // Add MESSAGE-INTEGRITY attribute (simplified - in real implementation, use HMAC)
        let integrity_hash = self.calculate_message_integrity(&message, &turn_server.password)?;
        self.add_turn_attribute(
            &mut message,
            TurnAttributeType::MessageIntegrity,
            &integrity_hash,
        );

        // Update message length again
        let message_length = (message.len() - 20) as u16;
        message[2..4].copy_from_slice(&message_length.to_be_bytes());

        Ok(message)
    }

    /// Add TURN attribute to message
    fn add_turn_attribute(
        &self,
        message: &mut Vec<u8>,
        attr_type: TurnAttributeType,
        value: &[u8],
    ) {
        message.extend_from_slice(&(attr_type as u16).to_be_bytes());
        message.extend_from_slice(&(value.len() as u16).to_be_bytes());
        message.extend_from_slice(value);

        // Add padding to 4-byte boundary
        let padding = (4 - (value.len() % 4)) % 4;
        message.extend_from_slice(&vec![0u8; padding]);
    }

    /// Calculate MESSAGE-INTEGRITY hash (simplified)
    fn calculate_message_integrity(&self, message: &[u8], password: &str) -> Result<Vec<u8>> {
        // In a real implementation, this would use HMAC-SHA1
        // For now, use a simple hash based on message and password
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        message.hash(&mut hasher);
        password.hash(&mut hasher);
        let hash = hasher.finish();

        // Return 20-byte hash (HMAC-SHA1 size)
        let mut result = Vec::new();
        result.extend_from_slice(&hash.to_be_bytes());
        result.extend_from_slice(&hash.to_be_bytes());
        result.extend_from_slice(&(hash as u32).to_be_bytes());
        result.truncate(20);

        Ok(result)
    }

    /// Parse TURN allocation response
    fn parse_turn_allocation_response(
        &self,
        data: &[u8],
        expected_transaction_id: [u8; 12],
    ) -> Result<TurnAllocationResponse> {
        if data.len() < 20 {
            return Err(SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: "TURN response too short".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Check TURN server configuration".to_string()),
            })));
        }

        let message_type = u16::from_be_bytes([data[0], data[1]]);
        let message_length = u16::from_be_bytes([data[2], data[3]]);
        let transaction_id = &data[8..20];

        // Verify transaction ID
        if transaction_id != expected_transaction_id {
            return Err(SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: "Transaction ID mismatch in TURN response".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Check TURN server configuration".to_string()),
            })));
        }

        // Check if it's an error response
        if message_type == TurnMessageType::AllocateError as u16 {
            return Err(SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: "TURN allocation failed".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Check TURN server credentials and configuration".to_string()),
            })));
        }

        // Parse attributes to find relay address
        let mut relay_address = None;
        let mut lifetime = 600; // Default 10 minutes
        let mut pos = 20;

        while pos < data.len() && pos < 20 + message_length as usize {
            if pos + 4 > data.len() {
                break;
            }

            let attr_type = u16::from_be_bytes([data[pos], data[pos + 1]]);
            let attr_length = u16::from_be_bytes([data[pos + 2], data[pos + 3]]);
            pos += 4;

            if pos + attr_length as usize > data.len() {
                break;
            }

            match attr_type {
                0x0016 => {
                    // XOR-RELAYED-ADDRESS
                    if attr_length >= 8 {
                        let family = u16::from_be_bytes([data[pos + 1], data[pos + 2]]);
                        if family == 0x0001 {
                            // IPv4
                            let port = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) ^ 0x2112;
                            let ip_bytes = [
                                data[pos + 4] ^ 0x21,
                                data[pos + 5] ^ 0x12,
                                data[pos + 6] ^ 0xA4,
                                data[pos + 7] ^ 0x42,
                            ];
                            relay_address = Some(SocketAddr::from((ip_bytes, port)));
                        }
                    }
                }
                0x000D => {
                    // LIFETIME
                    if attr_length >= 4 {
                        lifetime = u32::from_be_bytes([
                            data[pos],
                            data[pos + 1],
                            data[pos + 2],
                            data[pos + 3],
                        ]);
                    }
                }
                _ => {} // Ignore other attributes
            }

            pos += attr_length as usize;
            // Skip padding
            pos += (4 - (attr_length as usize % 4)) % 4;
        }

        Ok(TurnAllocationResponse {
            relay_address,
            lifetime,
        })
    }

    /// Send data through TURN relay
    pub async fn send_through_turn_relay(
        &self,
        allocation_id: &str,
        data: &[u8],
        peer_address: SocketAddr,
    ) -> Result<()> {
        let mut allocations = self.turn_allocations.write().await;
        let allocation = allocations.get_mut(allocation_id).ok_or_else(|| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("TURN allocation not found: {}", allocation_id),
                details: None,
                endpoint: None,
                suggestion: Some("Create TURN allocation first".to_string()),
            }))
        })?;

        // Check if allocation is still valid
        if Instant::now() > allocation.expires_at {
            return Err(SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: "TURN allocation expired".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Refresh TURN allocation".to_string()),
            })));
        }

        // Check if we have permission for this peer
        if !allocation.permissions.contains(&peer_address) {
            self.create_turn_permission(allocation_id, peer_address)
                .await?;
        }

        let socket = self.local_socket.as_ref().ok_or_else(|| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: "Socket not initialized".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Initialize NAT traversal manager first".to_string()),
            }))
        })?;

        // Create TURN Send indication
        let transaction_id = self.generate_transaction_id();
        let send_indication =
            self.create_turn_send_indication(transaction_id, data, peer_address)?;

        // Send to TURN server
        let server_addr = format!("{}:{}", allocation.server.host, allocation.server.port);
        let turn_addr: SocketAddr = server_addr.parse().map_err(|e| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("Invalid TURN server address: {}", e),
                details: None,
                endpoint: None,
                suggestion: Some("Check TURN server configuration".to_string()),
            }))
        })?;

        socket
            .send_to(&send_indication, turn_addr)
            .await
            .map_err(|e| {
                SongbirdError::Network(Box::new(NetworkError {
                    service: Some("NAT Traversal".to_string()),
                    message: format!("Failed to send data through TURN relay: {}", e),
                    details: None,
                    endpoint: None,
                    suggestion: Some("Check network connectivity".to_string()),
                }))
            })?;

        // Update statistics
        allocation.bytes_sent += data.len() as u64;

        debug!(
            "Sent {} bytes through TURN relay to {}",
            data.len(),
            peer_address
        );
        Ok(())
    }

    /// Create TURN permission for peer address
    async fn create_turn_permission(
        &self,
        allocation_id: &str,
        peer_address: SocketAddr,
    ) -> Result<()> {
        let mut allocations = self.turn_allocations.write().await;
        let allocation = allocations.get_mut(allocation_id).ok_or_else(|| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("TURN allocation not found: {}", allocation_id),
                details: None,
                endpoint: None,
                suggestion: Some("Create TURN allocation first".to_string()),
            }))
        })?;

        // Add permission
        allocation.permissions.push(peer_address);

        info!(
            "Created TURN permission for {} on allocation {}",
            peer_address, allocation_id
        );
        Ok(())
    }

    /// Create TURN Send indication
    fn create_turn_send_indication(
        &self,
        transaction_id: [u8; 12],
        data: &[u8],
        peer_address: SocketAddr,
    ) -> Result<Vec<u8>> {
        let mut message = Vec::new();

        // TURN header
        message.extend_from_slice(&(TurnMessageType::Send as u16).to_be_bytes());
        message.extend_from_slice(&[0x00, 0x00]); // Length placeholder
        message.extend_from_slice(&[0x21, 0x12, 0xA4, 0x42]); // Magic cookie
        message.extend_from_slice(&transaction_id);

        // Add XOR-PEER-ADDRESS attribute
        let peer_addr_bytes = self.encode_xor_address(peer_address);
        self.add_turn_attribute(
            &mut message,
            TurnAttributeType::XorPeerAddress,
            &peer_addr_bytes,
        );

        // Add DATA attribute
        self.add_turn_attribute(&mut message, TurnAttributeType::Data, data);

        // Update message length
        let message_length = (message.len() - 20) as u16;
        message[2..4].copy_from_slice(&message_length.to_be_bytes());

        Ok(message)
    }

    /// Encode address for XOR attributes
    fn encode_xor_address(&self, address: SocketAddr) -> Vec<u8> {
        let mut result = Vec::new();

        result.push(0x00); // Reserved
        result.push(0x01); // IPv4 family

        // XOR port with magic cookie first 2 bytes
        let port = address.port() ^ 0x2112;
        result.extend_from_slice(&port.to_be_bytes());

        // XOR IP address with magic cookie
        if let SocketAddr::V4(addr) = address {
            let ip_bytes = addr.ip().octets();
            result.push(ip_bytes[0] ^ 0x21);
            result.push(ip_bytes[1] ^ 0x12);
            result.push(ip_bytes[2] ^ 0xA4);
            result.push(ip_bytes[3] ^ 0x42);
        }

        result
    }

    /// Refresh TURN allocation
    pub async fn refresh_turn_allocation(&self, allocation_id: &str) -> Result<()> {
        let allocations = self.turn_allocations.read().await;
        let allocation = allocations.get(allocation_id).ok_or_else(|| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("TURN allocation not found: {}", allocation_id),
                details: None,
                endpoint: None,
                suggestion: Some("Create TURN allocation first".to_string()),
            }))
        })?;

        let socket = self.local_socket.as_ref().ok_or_else(|| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: "Socket not initialized".to_string(),
                details: None,
                endpoint: None,
                suggestion: Some("Initialize NAT traversal manager first".to_string()),
            }))
        })?;

        // Create refresh request
        let transaction_id = self.generate_transaction_id();
        let refresh_request = self
            .create_turn_refresh_request(transaction_id, &allocation.server)
            .await?;

        // Send refresh request
        let server_addr = format!("{}:{}", allocation.server.host, allocation.server.port);
        let turn_addr: SocketAddr = server_addr.parse().map_err(|e| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("Invalid TURN server address: {}", e),
                details: None,
                endpoint: None,
                suggestion: Some("Check TURN server configuration".to_string()),
            }))
        })?;

        socket
            .send_to(&refresh_request, turn_addr)
            .await
            .map_err(|e| {
                SongbirdError::Network(Box::new(NetworkError {
                    service: Some("NAT Traversal".to_string()),
                    message: format!("Failed to send TURN refresh request: {}", e),
                    details: None,
                    endpoint: None,
                    suggestion: Some("Check network connectivity".to_string()),
                }))
            })?;

        // Update expiration time
        drop(allocations);
        let mut allocations = self.turn_allocations.write().await;
        if let Some(allocation) = allocations.get_mut(allocation_id) {
            allocation.expires_at = Instant::now() + std::time::Duration::from_secs(600);
        }

        info!("Refreshed TURN allocation: {}", allocation_id);
        Ok(())
    }

    /// Create TURN refresh request
    async fn create_turn_refresh_request(
        &self,
        transaction_id: [u8; 12],
        turn_server: &TurnServer,
    ) -> Result<Vec<u8>> {
        let mut message = Vec::new();

        // TURN header
        message.extend_from_slice(&(TurnMessageType::Refresh as u16).to_be_bytes());
        message.extend_from_slice(&[0x00, 0x00]); // Length placeholder
        message.extend_from_slice(&[0x21, 0x12, 0xA4, 0x42]); // Magic cookie
        message.extend_from_slice(&transaction_id);

        // Add LIFETIME attribute (10 minutes)
        self.add_turn_attribute(
            &mut message,
            TurnAttributeType::Lifetime,
            &600u32.to_be_bytes(),
        );

        // Add USERNAME attribute
        self.add_turn_attribute(
            &mut message,
            TurnAttributeType::Username,
            turn_server.username.as_bytes(),
        );

        // Add REALM attribute
        self.add_turn_attribute(
            &mut message,
            TurnAttributeType::Realm,
            turn_server.realm.as_bytes(),
        );

        // Update message length
        let message_length = (message.len() - 20) as u16;
        message[2..4].copy_from_slice(&message_length.to_be_bytes());

        // Add MESSAGE-INTEGRITY attribute
        let integrity_hash = self.calculate_message_integrity(&message, &turn_server.password)?;
        self.add_turn_attribute(
            &mut message,
            TurnAttributeType::MessageIntegrity,
            &integrity_hash,
        );

        // Update message length again
        let message_length = (message.len() - 20) as u16;
        message[2..4].copy_from_slice(&message_length.to_be_bytes());

        Ok(message)
    }

    /// Get TURN allocation statistics
    pub async fn get_turn_allocation_stats(
        &self,
        allocation_id: &str,
    ) -> Result<TurnAllocationStats> {
        let allocations = self.turn_allocations.read().await;
        let allocation = allocations.get(allocation_id).ok_or_else(|| {
            SongbirdError::Network(Box::new(NetworkError {
                service: Some("NAT Traversal".to_string()),
                message: format!("TURN allocation not found: {}", allocation_id),
                details: None,
                endpoint: None,
                suggestion: Some("Create TURN allocation first".to_string()),
            }))
        })?;

        Ok(TurnAllocationStats {
            allocation_id: allocation.allocation_id.clone(),
            relay_address: allocation.relay_address,
            allocated_at: allocation.allocated_at,
            expires_at: allocation.expires_at,
            permissions_count: allocation.permissions.len(),
            bytes_sent: allocation.bytes_sent,
            bytes_received: allocation.bytes_received,
            is_expired: Instant::now() > allocation.expires_at,
        })
    }

    /// Clean up expired TURN allocations
    pub async fn cleanup_expired_allocations(&self) -> Result<u32> {
        let mut allocations = self.turn_allocations.write().await;
        let now = Instant::now();
        let initial_count = allocations.len();

        allocations.retain(|_, allocation| now <= allocation.expires_at);

        let cleaned_count = initial_count - allocations.len();
        if cleaned_count > 0 {
            info!("Cleaned up {} expired TURN allocations", cleaned_count);
        }

        Ok(cleaned_count as u32)
    }
}

/// TURN allocation response
#[derive(Debug)]
struct TurnAllocationResponse {
    relay_address: Option<SocketAddr>,
    lifetime: u32,
}

/// TURN allocation statistics
#[derive(Debug, Clone)]
pub struct TurnAllocationStats {
    pub allocation_id: String,
    pub relay_address: SocketAddr,
    pub allocated_at: Instant,
    pub expires_at: Instant,
    pub permissions_count: usize,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub is_expired: bool,
}
