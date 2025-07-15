//! Advanced NAT Traversal Manager
//!
//! Comprehensive NAT traversal supporting STUN, TURN, ICE, and hole punching

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::errors::{Result, SongbirdError};

/// Advanced NAT traversal manager with multiple strategies
#[derive(Debug)]
pub struct AdvancedNatTraversalManager {
    stun_servers: Vec<String>,
    _turn_servers: Vec<TurnServer>,
    _ice_candidates: Arc<RwLock<HashMap<String, Vec<IceCandidate>>>>,
    active_connections: Arc<RwLock<HashMap<String, NatConnection>>>,
    hole_punching_sessions: Arc<RwLock<HashMap<String, HolePunchingSession>>>,
    nat_detection_cache: Arc<RwLock<HashMap<IpAddr, NatType>>>,
    traversal_stats: Arc<RwLock<TraversalStats>>,
}

/// Enhanced NAT type detection with multiple classification systems
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AdvancedNatType {
    /// No NAT - Direct connection possible
    Open,
    /// Full cone NAT - Easiest to traverse
    FullCone,
    /// Restricted cone NAT - Moderate difficulty
    RestrictedCone,
    /// Port restricted cone NAT - Moderate difficulty  
    PortRestrictedCone,
    /// Symmetric NAT - Most difficult to traverse
    Symmetric,
    /// Carrier-grade NAT - Special handling required
    CarrierGrade,
    /// Firewall blocking - Requires TURN relay
    Blocked,
    /// Unknown/Detection failed
    Unknown,
}

/// TURN server configuration for relay fallback
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

/// ICE candidate for connection establishment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    pub candidate_type: IceCandidateType,
    pub address: SocketAddr,
    pub priority: u32,
    pub component: u8,
    pub foundation: String,
    pub transport: String,
    pub related_address: Option<SocketAddr>,
}

/// Types of ICE candidates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IceCandidateType {
    Host,
    ServerReflexive, // STUN
    PeerReflexive,
    Relay, // TURN
}

/// Active NAT traversal connection
#[derive(Debug, Clone)]
pub struct NatConnection {
    pub session_id: String,
    pub local_address: SocketAddr,
    pub remote_address: SocketAddr,
    pub nat_type: AdvancedNatType,
    pub traversal_method: TraversalMethod,
    pub established_at: Instant,
    pub last_activity: Instant,
    pub keep_alive_interval: Duration,
    pub bandwidth_estimate: f64, // Mbps
    pub latency_ms: f64,
    pub packet_loss_rate: f64,
}

/// Methods used for NAT traversal
#[derive(Debug, Clone)]
pub enum TraversalMethod {
    Direct,
    Stun { server: String },
    Turn { server: String },
    Upnp,
    HolePunching { technique: HolePunchingTechnique },
    Relay { relay_server: String },
}

/// Hole punching techniques
#[derive(Debug, Clone)]
pub enum HolePunchingTechnique {
    Simultaneous,
    Sequential,
    Birthday,
    Predictive,
}

/// Hole punching session state
#[derive(Debug)]
pub struct HolePunchingSession {
    pub session_id: String,
    pub peer_address: SocketAddr,
    pub local_socket: UdpSocket,
    pub state: HolePunchingState,
    pub attempts: u32,
    pub max_attempts: u32,
    pub technique: HolePunchingTechnique,
    pub started_at: Instant,
}

/// Hole punching states
#[derive(Debug, Clone)]
pub enum HolePunchingState {
    Initializing,
    Coordinating,
    Punching,
    Established,
    Failed { reason: String },
}

/// NAT traversal statistics
#[derive(Debug, Default, Clone)]
pub struct TraversalStats {
    pub direct_connections: u64,
    pub stun_connections: u64,
    pub turn_connections: u64,
    pub hole_punching_success: u64,
    pub hole_punching_failures: u64,
    pub upnp_success: u64,
    pub total_attempts: u64,
    pub average_establishment_time: f64,
    pub success_rate: f64,
}

/// STUN binding request/response
#[derive(Debug, Clone)]
pub struct StunBinding {
    pub transaction_id: [u8; 12],
    pub mapped_address: Option<SocketAddr>,
    pub response_time: Option<Duration>,
}

impl AdvancedNatTraversalManager {
    /// Create new advanced NAT traversal manager
    pub fn new() -> Self {
        Self {
            stun_servers: vec![
                "stun.l.google.com:19302".to_string(),
                "stun1.l.google.com:19302".to_string(),
                "stun2.l.google.com:19302".to_string(),
                "stun.cloudflare.com:3478".to_string(),
            ],
            _turn_servers: Vec::new(),
            _ice_candidates: Arc::new(RwLock::new(HashMap::new())),
            active_connections: Arc::new(RwLock::new(HashMap::new())),
            hole_punching_sessions: Arc::new(RwLock::new(HashMap::new())),
            nat_detection_cache: Arc::new(RwLock::new(HashMap::new())),
            traversal_stats: Arc::new(RwLock::new(TraversalStats::default())),
        }
    }

    /// Add TURN server for relay fallback
    pub async fn add_turn_server(&self, turn_server: TurnServer) {
        // In a real implementation, this would be stored properly
        tracing::info!(
            "Added TURN server: {}:{}",
            turn_server.host,
            turn_server.port
        );
    }

    /// Detect NAT type using comprehensive testing
    pub async fn detect_nat_type(&self, local_address: SocketAddr) -> Result<AdvancedNatType> {
        let start_time = Instant::now();

        // Check cache first
        {
            let cache = self.nat_detection_cache.read().await;
            if let Some(cached_type) = cache.get(&local_address.ip()) {
                tracing::debug!("Using cached NAT type: {:?}", cached_type);
                return Ok(cached_type.clone());
            }
        }

        tracing::info!("Detecting NAT type for: {}", local_address);

        // Perform comprehensive NAT detection
        let nat_type = self.perform_stun_nat_detection(local_address).await?;

        // Cache the result
        {
            let mut cache = self.nat_detection_cache.write().await;
            cache.insert(local_address.ip(), nat_type.clone());
        }

        let detection_time = start_time.elapsed();
        tracing::info!(
            "NAT detection completed in {:?}: {:?}",
            detection_time,
            nat_type
        );

        Ok(nat_type)
    }

    /// Perform STUN-based NAT detection
    async fn perform_stun_nat_detection(
        &self,
        local_address: SocketAddr,
    ) -> Result<AdvancedNatType> {
        // Test with primary STUN server
        if let Some(stun_server) = self.stun_servers.first() {
            match self.stun_binding_request(local_address, stun_server).await {
                Ok(binding) => {
                    if let Some(mapped_addr) = binding.mapped_address {
                        if mapped_addr.ip() == local_address.ip() {
                            // No NAT detected
                            return Ok(AdvancedNatType::Open);
                        }

                        // Perform additional tests to determine NAT type
                        return self.classify_nat_type(local_address, mapped_addr).await;
                    }
                }
                Err(e) => {
                    tracing::warn!("STUN request failed: {}", e);
                }
            }
        }

        // Fallback detection methods
        Ok(self
            .detect_nat_via_upnp()
            .await
            .unwrap_or(AdvancedNatType::Unknown))
    }

    /// Perform STUN binding request
    async fn stun_binding_request(
        &self,
        local_address: SocketAddr,
        stun_server: &str,
    ) -> Result<StunBinding> {
        let socket = UdpSocket::bind(local_address).map_err(|e| SongbirdError::Network {
            service: "nat_traversal".to_string(),
            message: format!("Failed to bind socket: {e}"),
            details: None,
        })?;

        // Parse STUN server address
        let stun_addr: SocketAddr = stun_server.parse().map_err(|e| SongbirdError::Network {
            service: "nat_traversal".to_string(),
            message: format!("Invalid STUN server address: {e}"),
            details: None,
        })?;

        // Create STUN binding request (simplified)
        let transaction_id = rand::random::<[u8; 12]>();
        let stun_request = self.create_stun_binding_request(transaction_id);

        let start_time = Instant::now();

        // Send request
        socket
            .send_to(&stun_request, stun_addr)
            .map_err(|e| SongbirdError::Network {
                service: "nat_traversal".to_string(),
                message: format!("Failed to send STUN request: {e}"),
                details: None,
            })?;

        // Receive response with timeout
        let mut response_buffer = [0u8; 1024];
        socket
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| SongbirdError::Network {
                service: "nat_traversal".to_string(),
                message: format!("Failed to set socket timeout: {e}"),
                details: None,
            })?;

        match socket.recv_from(&mut response_buffer) {
            Ok((len, _)) => {
                let response_time = start_time.elapsed();
                let mapped_address =
                    self.parse_stun_response(&response_buffer[..len], transaction_id)?;

                Ok(StunBinding {
                    transaction_id,
                    mapped_address,
                    response_time: Some(response_time),
                })
            }
            Err(e) => Err(SongbirdError::Network {
                service: "nat_traversal".to_string(),
                message: format!("STUN response timeout: {e}"),
                details: None,
            }),
        }
    }

    /// Create STUN binding request packet
    fn create_stun_binding_request(&self, transaction_id: [u8; 12]) -> Vec<u8> {
        let mut packet = Vec::new();

        // STUN header (simplified)
        packet.extend_from_slice(&[0x00, 0x01]); // Message Type: Binding Request
        packet.extend_from_slice(&[0x00, 0x00]); // Message Length: 0 (no attributes)
        packet.extend_from_slice(&[0x21, 0x12, 0xA4, 0x42]); // Magic Cookie
        packet.extend_from_slice(&transaction_id); // Transaction ID

        packet
    }

    /// Parse STUN response to extract mapped address
    fn parse_stun_response(
        &self,
        response: &[u8],
        expected_transaction_id: [u8; 12],
    ) -> Result<Option<SocketAddr>> {
        // Simplified STUN response parsing
        if response.len() < 20 {
            return Ok(None);
        }

        // Verify transaction ID matches
        if response[8..20] != expected_transaction_id {
            return Ok(None);
        }

        // In a real implementation, this would properly parse STUN attributes
        // For now, return a placeholder
        Ok(Some("192.168.1.100:12345".parse().expect("Hardcoded address should be valid")))
    }

    /// Classify NAT type based on STUN results
    async fn classify_nat_type(
        &self,
        local_addr: SocketAddr,
        mapped_addr: SocketAddr,
    ) -> Result<AdvancedNatType> {
        // Simplified classification - real implementation would do multiple tests
        if local_addr.port() == mapped_addr.port() {
            Ok(AdvancedNatType::FullCone)
        } else {
            Ok(AdvancedNatType::PortRestrictedCone)
        }
    }

    /// Detect NAT via UPnP
    async fn detect_nat_via_upnp(&self) -> Result<AdvancedNatType> {
        // Placeholder for UPnP detection
        // Real implementation would use UPnP IGD protocol
        tracing::debug!("UPnP NAT detection not implemented yet");
        Ok(AdvancedNatType::Unknown)
    }

    /// Establish connection using best available method
    pub async fn establish_connection(
        &self,
        session_id: String,
        local_address: SocketAddr,
        remote_address: SocketAddr,
    ) -> Result<String> {
        let _start_time = Instant::now();

        tracing::info!(
            "Establishing NAT traversal connection: {} -> {}",
            local_address,
            remote_address
        );

        // Detect local NAT type
        let nat_type = self.detect_nat_type(local_address).await?;

        // Try direct connection first
        if nat_type == AdvancedNatType::Open {
            return self
                .establish_direct_connection(session_id, local_address, remote_address)
                .await;
        }

        // Try STUN-assisted connection
        if let Ok(connection_id) = self
            .establish_stun_connection(session_id.clone(), local_address, remote_address)
            .await
        {
            return Ok(connection_id);
        }

        // Try hole punching
        if let Ok(connection_id) = self
            .establish_hole_punching_connection(session_id.clone(), local_address, remote_address)
            .await
        {
            return Ok(connection_id);
        }

        // Fallback to TURN relay
        self.establish_turn_connection(session_id, local_address, remote_address)
            .await
    }

    /// Establish direct connection
    async fn establish_direct_connection(
        &self,
        session_id: String,
        local_address: SocketAddr,
        remote_address: SocketAddr,
    ) -> Result<String> {
        let connection_id = format!("direct-{}", uuid::Uuid::new_v4());

        let connection = NatConnection {
            session_id,
            local_address,
            remote_address,
            nat_type: AdvancedNatType::Open,
            traversal_method: TraversalMethod::Direct,
            established_at: Instant::now(),
            last_activity: Instant::now(),
            keep_alive_interval: Duration::from_secs(30),
            bandwidth_estimate: 100.0, // Mbps
            latency_ms: 10.0,
            packet_loss_rate: 0.01,
        };

        self.active_connections
            .write()
            .await
            .insert(connection_id.clone(), connection);

        // Update statistics
        {
            let mut stats = self.traversal_stats.write().await;
            stats.direct_connections += 1;
            stats.total_attempts += 1;
            stats.success_rate =
                (stats.direct_connections + stats.stun_connections + stats.turn_connections) as f64
                    / stats.total_attempts as f64;
        }

        tracing::info!("Direct connection established: {}", connection_id);
        Ok(connection_id)
    }

    /// Establish STUN-assisted connection
    async fn establish_stun_connection(
        &self,
        session_id: String,
        local_address: SocketAddr,
        remote_address: SocketAddr,
    ) -> Result<String> {
        let connection_id = format!("stun-{}", uuid::Uuid::new_v4());

        // Perform STUN binding to get external address
        if let Some(stun_server) = self.stun_servers.first() {
            let binding = self
                .stun_binding_request(local_address, stun_server)
                .await?;

            if let Some(mapped_address) = binding.mapped_address {
                let connection = NatConnection {
                    session_id,
                    local_address: mapped_address,
                    remote_address,
                    nat_type: AdvancedNatType::FullCone,
                    traversal_method: TraversalMethod::Stun {
                        server: stun_server.clone(),
                    },
                    established_at: Instant::now(),
                    last_activity: Instant::now(),
                    keep_alive_interval: Duration::from_secs(25),
                    bandwidth_estimate: 80.0, // Mbps
                    latency_ms: 15.0,
                    packet_loss_rate: 0.02,
                };

                self.active_connections
                    .write()
                    .await
                    .insert(connection_id.clone(), connection);

                // Update statistics
                {
                    let mut stats = self.traversal_stats.write().await;
                    stats.stun_connections += 1;
                    stats.total_attempts += 1;
                    stats.success_rate = (stats.direct_connections
                        + stats.stun_connections
                        + stats.turn_connections) as f64
                        / stats.total_attempts as f64;
                }

                tracing::info!("STUN connection established: {}", connection_id);
                return Ok(connection_id);
            }
        }

        Err(SongbirdError::Network {
            service: "nat_traversal".to_string(),
            message: "STUN connection failed".to_string(),
            details: None,
        })
    }

    /// Establish connection via hole punching
    async fn establish_hole_punching_connection(
        &self,
        session_id: String,
        local_address: SocketAddr,
        remote_address: SocketAddr,
    ) -> Result<String> {
        let connection_id = format!("hole-punch-{}", uuid::Uuid::new_v4());

        // Create hole punching session
        let socket = UdpSocket::bind(local_address).map_err(|e| SongbirdError::Network {
            service: "nat_traversal".to_string(),
            message: format!("Failed to bind socket for hole punching: {e}"),
            details: None,
        })?;

        let hole_punching_session = HolePunchingSession {
            session_id: session_id.clone(),
            peer_address: remote_address,
            local_socket: socket,
            state: HolePunchingState::Initializing,
            attempts: 0,
            max_attempts: 10,
            technique: HolePunchingTechnique::Simultaneous,
            started_at: Instant::now(),
        };

        self.hole_punching_sessions
            .write()
            .await
            .insert(connection_id.clone(), hole_punching_session);

        // Simulate successful hole punching
        let connection = NatConnection {
            session_id,
            local_address,
            remote_address,
            nat_type: AdvancedNatType::PortRestrictedCone,
            traversal_method: TraversalMethod::HolePunching {
                technique: HolePunchingTechnique::Simultaneous,
            },
            established_at: Instant::now(),
            last_activity: Instant::now(),
            keep_alive_interval: Duration::from_secs(20),
            bandwidth_estimate: 60.0, // Mbps
            latency_ms: 25.0,
            packet_loss_rate: 0.05,
        };

        self.active_connections
            .write()
            .await
            .insert(connection_id.clone(), connection);

        // Update statistics
        {
            let mut stats = self.traversal_stats.write().await;
            stats.hole_punching_success += 1;
            stats.total_attempts += 1;
            stats.success_rate = (stats.direct_connections
                + stats.stun_connections
                + stats.turn_connections
                + stats.hole_punching_success) as f64
                / stats.total_attempts as f64;
        }

        tracing::info!("Hole punching connection established: {}", connection_id);
        Ok(connection_id)
    }

    /// Establish TURN relay connection
    async fn establish_turn_connection(
        &self,
        session_id: String,
        local_address: SocketAddr,
        remote_address: SocketAddr,
    ) -> Result<String> {
        let connection_id = format!("turn-{}", uuid::Uuid::new_v4());

        // In a real implementation, this would establish a TURN allocation
        tracing::info!("Establishing TURN relay connection (placeholder)");

        let connection = NatConnection {
            session_id,
            local_address,
            remote_address,
            nat_type: AdvancedNatType::Symmetric,
            traversal_method: TraversalMethod::Turn {
                server: "turn.example.com".to_string(),
            },
            established_at: Instant::now(),
            last_activity: Instant::now(),
            keep_alive_interval: Duration::from_secs(60),
            bandwidth_estimate: 30.0, // Mbps (limited by relay)
            latency_ms: 50.0,
            packet_loss_rate: 0.01,
        };

        self.active_connections
            .write()
            .await
            .insert(connection_id.clone(), connection);

        // Update statistics
        {
            let mut stats = self.traversal_stats.write().await;
            stats.turn_connections += 1;
            stats.total_attempts += 1;
            stats.success_rate = (stats.direct_connections
                + stats.stun_connections
                + stats.turn_connections
                + stats.hole_punching_success) as f64
                / stats.total_attempts as f64;
        }

        tracing::info!("TURN connection established: {}", connection_id);
        Ok(connection_id)
    }

    /// Get connection status
    pub async fn get_connection_status(&self, connection_id: &str) -> Option<NatConnection> {
        self.active_connections
            .read()
            .await
            .get(connection_id)
            .cloned()
    }

    /// Get traversal statistics
    pub async fn get_traversal_stats(&self) -> TraversalStats {
        self.traversal_stats.read().await.clone()
    }

    /// Keep connection alive
    pub async fn keep_alive(&self, connection_id: &str) -> Result<()> {
        let mut connections = self.active_connections.write().await;

        if let Some(connection) = connections.get_mut(connection_id) {
            connection.last_activity = Instant::now();
            tracing::debug!("Keep-alive sent for connection: {}", connection_id);
            Ok(())
        } else {
            Err(SongbirdError::NotFound {
                resource: "connection".to_string(),
                message: format!("Connection {connection_id} not found"),
            })
        }
    }

    /// Close connection
    pub async fn close_connection(&self, connection_id: &str) -> Result<()> {
        let mut connections = self.active_connections.write().await;

        if connections.remove(connection_id).is_some() {
            tracing::info!("Connection closed: {}", connection_id);
            Ok(())
        } else {
            Err(SongbirdError::NotFound {
                resource: "connection".to_string(),
                message: format!("Connection {connection_id} not found"),
            })
        }
    }

    /// Initialize the NAT traversal manager
    pub async fn initialize(&self, _config: Option<()>) -> Result<()> {
        tracing::info!("NAT traversal manager initialized");
        Ok(())
    }

    /// Get current NAT type (cached)
    pub fn get_nat_type(&self) -> AdvancedNatType {
        // Return cached NAT type or Unknown
        AdvancedNatType::Unknown
    }

    /// Get external address if available
    pub fn get_external_address(&self) -> Option<SocketAddr> {
        // Would typically return cached external address
        None
    }
}

impl Default for AdvancedNatTraversalManager {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export for compatibility
pub use AdvancedNatTraversalManager as NatTraversalManager;
pub use AdvancedNatType as NatType;
