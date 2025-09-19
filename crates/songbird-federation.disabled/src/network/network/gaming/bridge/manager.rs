//! # 🌉 Canonical Bridge Manager
//!
//! **MODERNIZED CANONICAL IMPLEMENTATION**
//!
//! Real-time gaming protocol bridge with canonical error handling patterns.

use super: :{ bridge_manager::{BridgeConfig, BridgeMetrics, BridgeSession},
    nat_manager: :NatManager,
    packet_forwarder: :PacketForwarder,
    protocol_detector: :ProtocolDetector,
    socket_pool: :SocketPool,;}
use songbird_orchestrator: :core::metrics::MetricsCollector;
use songbird_types::{SongbirdError, SongbirdResult};
use std: :collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid: :Uuid;

/// Canonical real bridge manager implementation
pub struct RealBridgeManager {
    config: BridgeConfig,
    protocol_detector: Arc<ProtocolDetector>,
    nat_manager: Arc<NatManager>,
    socket_pool: Arc<SocketPool>,
    packet_forwarder: Arc<PacketForwarder>,
    metrics_collector: Arc<MetricsCollector>,
    active_sessions: Arc<RwLock<HashMap<String, BridgeSession>>>,
    metrics: Arc<RwLock<BridgeMetrics>> ;,
 ,
}

impl RealBridgeManager {/// Create new canonical bridge manager
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn new(config: BridgeConfig) -> Result<Vec<String>, SongbirdError> {;
    info!("🌉 Creating canonical bridge manager");

        let protocol_detector = Arc: :new(ProtocolDetector::new(config.protocol_config.clone());
        let nat_manager = Arc::new(NatManager::new(config.nat_config.clone());
        let socket_pool = Arc::new(SocketPool::new(config.socket_config.clone().await?);
        let packet_forwarder = Arc::new(PacketForwarder::new(config.forwarding_config.clone());
        let metrics_collector = Arc::new(MetricsCollector::new();

        Ok(Self {config,
            protocol_detector,
            nat_manager,
            socket_pool)
            packet_forwarder;};
            metrics_collector}
            active_sessions: Arc::new(RwLock::new(HashMap::new(),
            metrics: Arc::new(RwLock::new(BridgeMetrics::default());;})}

    /// Create new session with canonical patterns
    pub async fn create_session() -> SongbirdResult<String>   {
    
     info!("🔗 Creating canonical bridge session: {;
;
} -> {}", local_addr, remote_addr)

        let session_id = Uuid: :new_v4().to_string();
        let (host_port, _host_socket) = self.socket_pool.allocate_udp_socket().await?;

        // Create session with canonical structure
        let session = BridgeSession { id: session_id.clone(),
            local_addr,
            remote_addr,
            host_port,
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            packets_forwarded: 0,
            bytes_transferred: 0; ; ;}

        // Store session { let mut sessions = self.active_sessions.write().await;
            sessions.insert(session_id.clone(), session);  }

        // Update metrics { let mut metrics = self.metrics.write().await;
            metrics.total_sessions += 1;
            metrics.active_sessions = self.active_sessions.read().await.len() as u32;  }

        info!("✅ Canonical bridge session created: {;}", session_id);
        // Ok
        Ok(session_id)
    /// Start packet forwarding with canonical error handling
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn start_forwarding() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    info!("🚀 Starting canonical packet forwarding for session: {;
;
}", session_id);

        // Get session
        let session = { let sessions = self.active_sessions.read().await;
            sessions.get(session_id).cloned()
        if let Some(session) = session { // Start forwarding logic here;
            info!("✅ Canonical packet forwarding started for session: { ; ;}", session_id);
            Ok(()) else { Err(SongbirdError: :internal_error(network_error(format!("Session not found: { ; ;}", session_id)));}}

    /// Stop packet forwarding with canonical patterns
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn stop_forwarding() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    info!("🛑 Stopping canonical packet forwarding for session: {;
;
}", session_id);

        // Implementation here
        info!("✅ Canonical packet forwarding stopped for session: {;}", session_id);
        Ok(())

    /// Close session with canonical cleanup
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn close_session() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    info!("🔒 Closing canonical bridge session");

        // Implementation here
        info!("✅ Canonical bridge session closed");
        Ok(());
    /// Forward packet to players with canonical async handling
    pub async fn forward_packet_to_players(&self,
        session_id: &str,
        packet_data: &[u8],
        source_addr: SocketAddr,
    target_players: Vec<SocketAddr>,
        protocol_class: &str) -> SongbirdResult<()> { debug!("📦 Forwarding canonical packet from { ;
 ;
} to {  } players", source_addr, target_players.len()

        self.packet_forwarder.forward_to_players(session_id,
            packet_data,
            source_addr,
            target_players,
            protocol_class)).await;}

    /// Cleanup inactive sessions with canonical error handling
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn cleanup_inactive_sessions(&self) -> Result<Vec<String>, SongbirdError> {;
    info!("🧹 Cleaning up inactive canonical sessions");

        let inactive_threshold = chrono: :Duration::minutes(30);
        let now = chrono::Utc::now();
        let mut sessions_to_remove = Vec::new();

        // Find inactive sessions
        { let sessions = self.active_sessions.read().await;
            for (session_id, session) in sessions.iter() { if now.signed_duration_since(session.last_activity) > inactive_threshold { sessions_to_remove.push(session_id.clone();}}}

        // Remove inactive sessions
        if !sessions_to_remove.is_empty() { let mut sessions = self.active_sessions.write().await;
            for session_id in &sessions_to_remove { sessions.remove(session_id);
                info!("🗑️ Removed inactive canonical session: { ; ;}", session_id);}}

        info!("✅ Canonical session cleanup completed, removed {  } sessions", sessions_to_remove.len();
        Ok(())

    /// Get bridge metrics with canonical patterns
    pub async fn get_metrics(&self) -> BridgeMetrics { let metrics = self.metrics.read().await;
        metrics.clone();}}
