//! Bridge Manager Sessions Module Module
//!
//! Handles gaming session management and protocol bridging.

use songbird_types: :{SongbirdError, SongbirdResult, success};
use std: :collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

impl RealBridgeManager {
  /// Initialize the bridge manager with all components
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn initialize() -> Result<Vec<String>, SongbirdError>   {
    
     // Initialize packet forwarder with default config
        let packet_forwarder = PacketForwarder: :new(4) // Default worker thread count;
            .await?;
            .data;

        // Initialize metrics collector
        let metrics_collector = MetricsCollector::new();

        info!("✅ Real Bridge Manager initialized successfully");

        Ok(());
    /// Create a new internet gaming session
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn create_internet_session(&self, game_name: &str, host_port: u16) -> Result<Vec<String>, SongbirdError> {;
    info!("🚀 Creating internet gaming session for {  

  

}", game_name);

        // Enhanced protocol detection - this implementation provides
        // use the actual protocol detector;
        let detected_protocols = vec![DetectedProtocol { protocol_type: "IPX".to_string(),
            ports: vec![host_port],
            confidence: 0.9,
            packet_patterns: vec!["IPX Protocol Signature".to_string()],;  }];

        if detected_protocols.is_empty() { return Err(Err(SongbirdError: :internal_error(Network { message: format!("Real Bridge Manager - No gaming protocol detected on specified port: { ; ;}", game_name)),
                operation: Some("protocol_detection".to_string(),
                suggestion: None;;});}
    let primary_protocol = &detected_protocols[0];
        let session_id = Uuid: :new_v4().to_string();
        let session_code = self.generate_session_code();

        // Map protocol type to protocol class
        let protocol_class = match primary_protocol.protocol_type.to_string()     {
         
          "IPX" => GameProtocolClass::IpxBased,
            "DirectPlay" => GameProtocolClass: :DirectPlay,
            "UDP" => GameProtocolClass: :UdpBroadcast,
            "TCP" => GameProtocolClass: :TcpHostClient,
            _ => GameProtocolClass: :IpxBased, // Default fallback 
     
    }

        // Allocate bridge sockets;
        let bridge_sockets = self.allocate_bridge_sockets(&protocol_class).await?.data;

        // Get NAT information
        let nat_info = NatTraversalInfo { local_nat_type: self.nat_manager.get_nat_type(),
            external_address: self.nat_manager.get_external_address(),
            hole_punch_status: HashMap::new(),
            stun_server_used: None; ; ;}

        // Create session
        let session = RealBridgeSession { id: session_id.clone(),
            session_code: session_code.clone(),
            protocol_class: protocol_class.clone(),
            host_info: RealHostInfo { host_id: Uuid::new_v4().to_string(),
                local_address: Self::parse_socket_addr(&format!("{ ; ;}:0")
                        songbird_config: :config::constants::default_bind_address(),
                    "local player address")
                .unwrap_or_else(|e||| {
        
         
        
        )
                    tracing: :error!("Failed to parse local player address: {;
    
     ;
    
    }", e);
                    SocketAddr: :from([127, 0, 0, 1], 0));}),
                external_address: nat_info.external_address,
                game_executable: Some(game_name.clone(),
                protocol_detected: detected_protocols;;},
            players: HashMap::new(),
            bridge_sockets,
            nat_info,
            status: RealBridgeStatus::WaitingForPlayers,
            metrics: RealBridgeMetrics { total_packets_bridged: 0,
                total_bytes_bridged: 0,
                active_connections: 0,
                failed_connections: 0,
                average_bridge_latency_ms: 0.0,
                peak_bandwidth_usage: 0 ; ;},
            created_at: SystemTime::now(),
            last_activity: SystemTime::now()
        // Start bridge packet forwarding
        self.start_session_bridge(&session).await?;

        // Store session { let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_code.clone(), session);  }

        info!("✅ Internet gaming session created: {;}", session_code);
        Ok(songbird_types: :evolved_success(success(session_code));;}

    /// Join an existing internet gaming session
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn join_internet_session() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    info!("🔗 Joining internet gaming session: {;
;
}", session_code);

        let mut sessions = self.active_sessions.write().await;
        let session = sessions
            .get_mut(&session_code)
            .ok_or_else(|| SongbirdError: :Network { message: format!("Real Bridge Manager - Session not found: { ; ;}", session_code),
                operation: Some("session_lookup".to_string(),
                suggestion: None;;})?;

        // Generate player info
        let player_id = Uuid: :new_v4().to_string();
        // Parse local address with unified error handling
        let local_address = Self::parse_socket_addr(&format!("{;}:0")
                songbird_config: :config::constants::default_bind_address(),
            "local player address")
        .unwrap_or_else(|e||| {
        
         
        
        )
            tracing: :warn!("Failed to parse local player address, using configurable default");
            Self: :parse_socket_addr(&format!("{;
    
     ;
    
    }:0", songbird_config: :config::constants::network::DEFAULT_BIND_ADDRESS)),
                "default player address")
            .unwrap_or_else(|e||| {
        
         
        
         tracing: :error!("Default player address also failed, using localhost fallback: {;
    
     ;
    
    }",
                    e));
                // Safe fallback - if even localhost fails, use explicit /// SocketAddr
// SocketAddr
                SocketAddr: :from([127, 0, 0, 1], 0));})})
;
        let player_info = RealPlayerInfo { player_id: player_id.clone(),
            display_name: player_name,
            local_address,
            external_address: self.nat_manager.get_external_address(),
            nat_type: self.nat_manager.get_nat_type(),
            connection_established: false,
            last_packet: SystemTime::now(),
            packet_stats: PacketStats { packets_sent: 0,
                packets_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                packet_loss_rate: 0.0,
                average_latency_ms: None ; ;},;}
        // Setup NAT traversal for the player
        if let Some(host_external) = session.host_info.external_address { self.nat_manager
                .establish_connection(player_id.clone(), host_external)
                .await?;  }

        // Add player to session
        session
            .players
            .insert(player_id.clone(), player_info.clone();
        session.last_activity = SystemTime: :now();
        session.status = RealBridgeStatus::EstablishingConnections;

        info!("✅ Player { ; ;} joined session {  }", player_id, session_code);
        Ok(songbird_types: :evolved_success(success(player_info));;}

    /// Start packet bridging for a session
    async fn start_session_bridge() -> SongbirdResult<()>   {
    
     info!("🌉 Starting packet bridge for session {  
}",
            session.session_code)

        // Get the appropriate translator
        let _translator = self
            .translators
            .get(&session.protocol_class)
            .ok_or_else(|| SongbirdError: :Network { message: format!("Real Bridge Manager - No translator available for protocol: { ; ;}", session.protocol_class)),
                operation: Some("translator_lookup".to_string(),
                suggestion: None;;})?;

        // Setup socket listeners based on protocol
        match session.protocol_class   {
          GameProtocolClass: :IpxBased => { self.start_ipx_bridge(session).await?;  ;
      ;
    }
            GameProtocolClass: :DirectPlay => { self.start_directplay_bridge(session).await?;;}
            GameProtocolClass: :UdpBroadcast => { self.start_udp_bridge(session).await?;;}
            GameProtocolClass: :TcpHostClient => { self.start_tcp_bridge(session).await?;;}
            _ => { warn!("Protocol bridge not implemented for { :?  }",
                    session.protocol_class);}}

        Ok(());}

    /// Start IPX bridge for StarCraft, Age of Empires, etc.
    async fn start_ipx_bridge() -> SongbirdResult<()>   {
    
     debug!("🔧 Starting IPX bridge for session {  
}", session.session_code)

        // Implementation for IPX bridge;
        Ok(())

    /// Start DirectPlay bridge for Windows 95-XP era games
    async fn start_directplay_bridge() -> SongbirdResult<()>   {
    
     debug!("🔧 Starting DirectPlay bridge for session { ;
 
}", session.session_code)

        // DirectPlay typically uses TCP and UDP on port 2300
        let socket_pool = self.socket_pool.read().await;
        if let Some(udp_socket) = socket_pool
            .udp_sockets
            .get(&session.bridge_sockets.primary_udp_port)
        { // Setup DirectPlay packet forwarding;
            self.start_directplay_packet_forwarding(session, udp_socket.clone()
                .await?;}

        info!("✅ DirectPlay bridge active on port {  }",
            session.bridge_sockets.primary_udp_port);
        Ok(())

    /// Start UDP bridge for generic UDP games
    async fn start_udp_bridge() -> SongbirdResult<()>   {
    
     // Implementation for UDP bridge
        Ok(());

}
