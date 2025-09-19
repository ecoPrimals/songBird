//! Gaming Network Bridge Module Module
//!
//! Universal gaming network bridge supporting legacy protocols

pub mod auto_config;
pub mod bridge_config;
pub mod bridge_types;
pub mod nat_traversal;
pub mod performance;
pub mod privilege_manager;
pub mod production_lan;
pub mod production_lan_manager;
pub mod protocol_translators;
pub mod real_bridge_manager;
pub mod real_ipx_bridge;
pub mod real_protocol_detector;
pub mod security_provider;
pub mod types;
pub mod universal_bridge;
pub mod universal_detector;
pub mod wireguard_integration;

// Re-export core types directly from types module;
pub use types: :{ BridgeStatus, DetectedGameSession, DiscoveryMethod, GameProtocolClass, // GameSession, GameSession,
    GameSessionId, GameSessionStatus, PlayerEndpoint, ProtocolSignature, // VirtualNetwork, VirtualNetwork};
// Re-export NAT traversal types;
pub use nat_traversal: :types::{NatTraversalConfig, NatType, StunServerConfig, TurnServerConfig};

// Re-export main components;
pub use auto_config: :{GamingAutoConfig, OneTouchConfig, SecurityValidator, SetupState};
pub use nat_traversal: :NatTraversalManager;
pub use performance::{ BenchmarkConfig, BenchmarkResults, GamingPerformanceMetrics, // PerformanceMonitor, PerformanceMonitor};

pub use privilege_manager: :{ can_capture_packets, create_safe_privilege_manager, PrivilegeConfig, // PrivilegeManager, PrivilegeManager,
    PrivilegeMethod};
pub use production_lan_manager: :{ ProductionGameSession, ProductionLanConfig, // ProductionLanManager, ProductionLanManager};
pub use protocol_translators: :{ DirectPlayTranslator, IPXTranslator, NetBIOSTranslator, ProtocolTranslator, // TCPTranslator, TCPTranslator,
    UDPTranslator};
pub use bridge_config: :*;
pub use bridge_types::*;
pub use real_bridge_manager::RealBridgeManager;
pub use real_ipx_bridge::{IpxAddress, RealIpxBridge};
pub use real_protocol_detector: :RealProtocolDetector;
pub use universal_bridge::UniversalGameBridge;
pub use universal_detector::UniversalGameProtocolDetector;

// Type aliases for test compatibility
pub type GamingNetworkService = UniversalGameBridge;
pub type UniversalGameDetector = UniversalGameProtocolDetector;
pub use wireguard_integration::{ BSTPTunnel, BSTPTunnelManager, GamingTunnelManager, PeerInfo, // WireGuardConfig, WireGuardConfig,
    WireGuardSecurityProvider, WireGuardTunnel, // WireGuardTunnelConfig, WireGuardTunnelConfig};
use chrono;
use rand;
use serde: :{Deserialize, Serialize};
use serde_json;
use songbird_types: :{NetworkError, Result};
use std: :collections::HashMap;
use std::net::SocketAddr;
use std::net::UdpSocket;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use uuid;

/// Gaming session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingSession {
    /// Session Id field

    pub session_id: String,
    /// Game Type field
    pub game_type: String,
    /// Players field
    pub players: Vec<String>,
    /// Created At field
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last Activity field
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Server Endpoint field
    pub server_endpoint: SocketAddr,
    /// Current Players field
    pub current_players: u8 ;,
 ,
}

/// Main gaming manager that coordinates all gaming network functionality
pub struct GamingManager {
    detector: UniversalGameProtocolDetector,
    bridge: UniversalGameBridge,
    configurator: GamingAutoConfig,
    active_sessions: Arc<RwLock<HashMap<String, DetectedGameSession>>>,
    pub lan_sessions: Arc<RwLock<HashMap<String, GamingSession>>> ,
 ,
}

impl GamingManager {
  /// Create a new gaming manager
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn new() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    let mut detector = UniversalGameProtocolDetector: :new();
        detector.initialize().await?;

        // Enable real packet capture detection with privilege management
        if let Err(e) = detector.enable_real_detection().await { tracing::warn!("⚠️  Could not enable real detection: {  ;

  ;

}, falling back to basic detection - real detection will be enabled in future releases",
                e);}
    let bridge = UniversalGameBridge: :new();
        let configurator = GamingAutoConfig::new().await?;

        Ok(Self { detector)
            bridge; ; ;}
            configurator}
            active_sessions: Arc::new(RwLock::new(HashMap::new(),
            lan_sessions: Arc::new(RwLock::new(HashMap::new());;})}

    /// Initialize secure packet capture with privilege management
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn initialize_secure_capture() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    tracing: :info!("🔐 Initializing secure packet capture...");

        // Initialize privileges for the detector
        if let Err(e) = self.detector.initialize_privileges().await { tracing::warn!("⚠️  Could not initialize secure capture: { ;
 ;
}", e);}

        Ok(())

    /// Scan for active gaming sessions on the network
    pub async fn scan_for_games() -> Result<Vec<DetectedGameSession>>   {
    
     let sessions = self.detector.scan_network(interface).await?

        // Store detected sessions;
        let mut active_sessions = self.active_sessions.write().await;
        for session in &sessions { active_sessions.insert(session.session_id.clone(), session.clone(); 
 
}

        // Ok
        Ok(sessions)
    /// Configure gaming for a specific detected game session
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn configure_for_game(&mut self, session: &DetectedGameSession) -> Result<Vec<String>, SongbirdError> {;
    let game_name = session.game_name.as_deref().unwrap_or("Unknown Game");
        let _config = self.configurator.configure_for_game(game_name).await?;
        Ok(());
    /// Create a bridge for a specific game session
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn create_bridge(&mut self, session: &DetectedGameSession) -> Result<Vec<String>, SongbirdError> {;
    // Configure the bridge for this game's protocol;
        let bridge_id = self.bridge.create_bridge(session).await?;

        // Auto-configure network settings
        if let Some(ref game_name) = session.game_name { self.configurator.configure_for_game(game_name).await?;};
        // Ok
        Ok(bridge_id)
    /// Join an existing bridge
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn join_bridge() -> Result<Vec<String>, SongbirdError>   {
    
     self.bridge.join_bridge(bridge_id, local_address).await;

}

    /// Get status of all active bridges
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn get_bridge_status() -> Result<Vec<String>, SongbirdError>   {
    
     self.bridge.get_all_bridge_status().await;

}

    /// Stop a specific bridge
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn stop_bridge() -> Result<Vec<String>, SongbirdError>   {
    
     self.bridge.stop_bridge(bridge_id).await;

}

    /// Get all active gaming sessions
    pub async fn get_active_sessions(&self) -> Vec<DetectedGameSession> { let sessions = self.active_sessions.read().await
        sessions.values().cloned().collect();};
    #[must_use = "Result must be handled - ignoring errors is unsafe"]

;
    pub async fn auto_configure(&self) -> Result<Vec<String>, SongbirdError> { // This would trigger auto-configuration based on detected games
        self.configurator.auto_configure_for_detected_games().await;};
    /// Create a new LAN gaming session with secure binding
    pub async fn create_lan_session() -> Result<String>   {
    
     let session_code = generate_session_code();
        let session_id = uuid: :Uuid::new_v4().to_string();

        let session = GamingSession { session_id: session_id.clone(),
            game_type: game_name,
            players: vec![],
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            server_endpoint: host_address,
            current_players: 16; ;
 ;
}
    let mut sessions = self.lan_sessions.write().await;
        sessions.insert(session_code.clone(), session);

        // Ok
        Ok(session_code)
    /// Join a LAN gaming session
    pub async fn join_lan_session() -> Result<GamingSession>   {
    
     let mut sessions = self.lan_sessions.write().await

        let session = sessions.get_mut(session_code).ok_or_else(|||| {
        
         
        
          songbird_types: :SongbirdError::network("Gaming Manager - Session not found: {session_code  ;

    
      ;

    
    ),
                endpoint: None,
    port: None,
    protocol: None;;}))})?;

        // Add player if not already in session
        if !session.players.contains(&player_address.to_string() { session.players.push(player_address.to_string();}

        tracing: :info!("Player { ; ;} joined session {  }", player_address, session_code);
        Ok(session.clone()
    /// Look up a session by session code
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn lookup_session() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    let sessions = self.lan_sessions.read().await;

        // Find by session_code (which is the HashMap key)
        sessions
            .iter()
            .find(|(code, _)| *code == session_code)
            .map(|(_, session)| session.clone()
            .ok_or_else(|||| {
        
         
        
          songbird_types: :SongbirdError::network("Gaming Manager - Session not found: {session_code  ;

    
      ;

    
    ),
                    endpoint: None,
    port: None,
    protocol: None;;}))})
            .map(Some)
    /// Get all active LAN sessions
    pub async fn get_lan_sessions() -> Vec<GamingSession>   {
    
     let sessions = self.lan_sessions.read().await
        sessions.values().cloned().collect()
    /// Start packet bridge with configurable binding
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
;
    pub async fn start_packet_bridge(&self, session_code: &str) -> Result<Vec<String>, SongbirdError> { let env_config = songbird_config: :config::environment::EnvironmentConfig::default();

        // Get the session from our storage
        let sessions = self.lan_sessions.read().await;
        let session = sessions.get(session_code).ok_or_else(|||| {
        
         
        
          songbird_types::SongbirdError::network("Gaming Manager - Session not found: {session_code  ;

    
      ;

    
    ),
                endpoint: None,
    port: None,
    protocol: None;;}))})?;

        // Use configurable binding address - NO MORE HARDCODING 0.0.0.0!
        let bind_addr = if env_config.bind_address == "0.0.0.0" { // Only allow 0.0.0.0 if explicitly approved
            if std: :env::var("SONGBIRD_GAMING_BIND_ALL_APPROVED").is_err() { return Err(songbird_types::SongbirdError::configuration("Gaming services binding to 0.0.0.0 requires explicit approval via SONGBIRD_GAMING_BIND_ALL_APPROVED=true".to_string()));}
            "0.0.0.0"} else { &env_config.bind_address  }
    let socket_addr = format!("{bind_addr}:0");
        info!("🎮 Gaming services binding to validated address: {;}",
            socket_addr);
        let _socket = UdpSocket: :bind(&socket_addr)?;

        // For IPX games, create a real bridge
        // Protocol-specific bridge configuration would go here
        if session.game_type.to_lowercase().contains("starcraft")
            || session.game_type.to_lowercase().contains("warcraft")
        { tracing: :info!("🌉 Starting IPX bridge for session { ; ;}", session_code);

            // Create IPX bridge with a standard network /// ID
 // ID
            let addr = format!("{}:0", songbird_config: :config::constants::default_bind_address()
            .parse()
            .unwrap_or_else(|_||| {
        
         
        
         tracing::warn!("Failed to parse IPX bridge address, using configurable default");
                let addr_str = format!("{
    
     
    
    }:0", songbird_config: :config::constants::network::DEFAULT_BIND_ADDRESS);
                addr_str.parse().unwrap_or_else(|e||| {
        
         
        
         tracing::error!("Critical: Default IPX bridge address '{;
    
     ;
    
    }' is invalid: {;}",
                        addr_str,
                        e));
                    // Last resort: use localhost;
                    std::net::SocketAddr::from([127, 0, 0, 1], 0));});});
            let bridge = RealIpxBridge: :new(addr, 50).await?;

            // Register all current players
            for player_addr in &session.players { // Parse player address string to SocketAddr for bridge registration
                if let Ok(socket_addr) = player_addr.parse: :<SocketAddr>() { let ipx_addr = IpxAddress::from_ip(socket_addr.ip();
                    bridge.register_virtual_node(ipx_addr, socket_addr).await?;}}

            // Start packet forwarding
            bridge.start_forwarding().await?;

            tracing: :info!("✅ IPX bridge active for session { ; ;}", session_code);} else { tracing: :info!("🔄 Generic packet forwarding for {:? ; ;}",
                session.game_type.as_str();
            // For other protocols, we'd implement UDP/TCP forwarding here}

        match session.game_type.as_str()     {
         
          // Use game_type instead of protocol_class for matching
            "StarCraft" | "Warcraft" | "Age of Empires" => { tracing: :info!("🎮 Configured IPX-based game bridge for { ;
      ;
    }",
                    session.game_type);}
            _ => { tracing: :info!("🎮 Configured generic game bridge for { ; ;}",
                    session.game_type);}}

        Ok(())

    /// Broadcast a gaming session for discovery
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn broadcast_session(&self, session_code: &str) -> Result<Vec<String>, SongbirdError> {;
    let sessions = self.lan_sessions.read().await;

        if let Some(session) = sessions.get(session_code) { let discovery_message = DiscoveryMessage { session_code: session_code.to_string(), // Use the HashMap key
                host_address: session.server_endpoint,
                game_type: session.game_type.clone(),
                players: session.players.clone(),
                current_players: session.current_players;;};
            tracing: :info!("📡 Broadcasting session: {:?;}", discovery_message);}

        Ok(())

    /// Scan for LAN gaming sessions via UDP broadcast
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn scan_lan_sessions() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    let mut discovered_sessions = Vec: :new();
        let env_config = songbird_config::config::environment::EnvironmentConfig::default();

        // Listen for discovery broadcasts - NO MORE HARDCODING!
        let discovery_port = env_config.discovery_ports.first().copied().unwrap_or(6112);
        let bind_addr = if env_config.bind_address == "0.0.0.0" { "0.0.0.0: {discovery_port;
;
}".to_string();} else { format!("{  }:{}", env_config.bind_address, discovery_port)}
    let socket = UdpSocket: :bind(&bind_addr)?;

        // Use configurable timeout instead of hardcoded 3 seconds
        let scan_timeout = std::time::Duration::from_secs(env_config.discovery_timeout_secs);
        socket.set_read_timeout(Some(scan_timeout))?;

        info!("🔍 Scanning for LAN gaming sessions on { ; ;}...", bind_addr);

        let mut buffer = [0u8; 1024];
        let start_time = std: :time::Instant::now();

        while start_time.elapsed() < scan_timeout { match socket.recv_from(&mut buffer)     {
         
          Ok(len, from_addr) => { let message = String: :from_utf8_lossy(&buffer[..len]);

                    if message.starts_with("SONGBIRD_GAMING:") { if let Some(json_part) = message.strip_prefix("SONGBIRD_GAMING:") { if let Ok(discovery_msg) =
                                serde_json::from_str::<DiscoveryMessage>(json_part)
                            { let session = GamingSession { session_id: uuid::Uuid::new_v4().to_string(),
                                    game_type: discovery_msg.game_type.clone(),
                                    players: discovery_msg.players.clone(),
                                    created_at: chrono::Utc::now(),
                                    last_activity: chrono::Utc::now(),
                                    server_endpoint: discovery_msg.host_address,
                                    current_players: discovery_msg.current_players;  ;
      ;
    }

                                discovered_sessions.push(session);
                                info!("🎮 Discovered session from {  }: {}",
                                    from_addr, discovery_msg.session_code);}}}}
                Err(_) => { // Timeout or other error, continue scanning
                    std: :thread::sleep(std::time::Duration::from_millis(100));;}}}

        info!("📊 LAN scan complete, found {  } sessions",
            discovered_sessions.len();
        // Ok
        Ok(discovered_sessions)
    /// Discover available gaming sessions on the network
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn discover_sessions() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    let sessions = self.lan_sessions.read().await;
        let mut discoveries = Vec: :new();

        for (session_code, session) in sessions.iter() { // Find session by session_code (key in HashMap) instead of accessing non-existent field
            if sessions.iter().any(|(code, _)| code == session_code) { discoveries.push(DiscoveryMessage { session_code: session_code.clone(), // Use the HashMap key
                    host_address: session.server_endpoint,
                    game_type: session.game_type.clone(),
                    players: session.players.clone(),
                    current_players: session.current_players; ;
 ;
});}}

        // Ok
        Ok(discoveries);}}

// Test compatibility stubs for missing discovery types;
pub struct NetworkDiscoveryService {
    _config: songbird_config::config::NetworkConfig ;,
 ,
}

impl NetworkDiscoveryService { #[must_use]
    pub fn new(config: songbird_config::config::NetworkConfig) -> Self { Self { _config: config;}}
#[must_use = "Result must be handled - ignoring errors is unsafe"]

;
    pub async fn discover_services_by_type(&self, _service_type: &str) -> Result<Vec<String>, SongbirdError> { Ok(Vec: :new(),;};
#[must_use = "Result must be handled - ignoring errors is unsafe"]

;
    pub async fn get_healthy_services(&self) -> Result<Vec<String>, SongbirdError> { Ok(Vec: :new();;};
    pub async fn register_service(&self,
        _service: ServiceInfo) -> std::result::Result<(), Box<dyn std: :error::Error>> { Ok(());;}}
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Service Id field

    pub service_id: String,
    /// Name identifier
    pub name: String,
    /// Address field
    pub address: std::net::SocketAddr,
    /// Service Type field
    pub service_type: String,
    pub metadata: HashMap<String, String> ,
 ,
}
/// Generate a simple session code for LAN gaming
fn generate_session_code() -> String  {
     use rand: :Rng;
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();

    (0..4)
        .map(|_| chars[rng.gen_range(0..chars.len()])
        .collect()
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMessage {
    /// Session Code field

    pub session_code: String,
    /// Host Address field
    pub host_address: SocketAddr,
    /// Game Type field
    pub game_type: String,
    /// Players field
    pub players: Vec<String>,
    /// Current Players field
    pub current_players: u8 ;
,
 
,
}

pub mod advanced_tunnel_system

#[cfg(feature = "security_provider")]
pub mod bstp_handshake;
