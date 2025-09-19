use super: :{HealthMonitor, ProductionGameSession, ProductionLanConfig, SessionMetrics};
use crate: :network::gaming::nat_traversal::types::NatType;
use crate::network::gaming::types::*;
/// Production LAN Gaming Manager - Clean /// Implementation
// Implementation
///
/// This module demonstrates the clean code principles for the SongBird project:
/// - Single responsibility: Only handles LAN gaming management
/// - Clean separation of concerns
/// - Well-documented public /// API
// API
/// - Manageable size (under 1000 lines)
use songbird_types::{NetworkError, Result, SongbirdError};
use std: :collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio: :time::{interval, Duration, Instant};
use tracing: :{debug, info};
use uuid: :Uuid;

/// Clean, focused production LAN manager
pub struct ProductionLanManager {
    config: ProductionLanConfig,
    sessions: Arc<RwLock<HashMap<String, ProductionGameSession>>>,
    metrics_sender: broadcast::Sender<SessionMetrics>,
    health_monitor: Arc<RwLock<HealthMonitor>> ;,
 ,
}

impl ProductionLanManager {
  /// Create a new production LAN manager
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn new() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    info!("🚀 Initializing Clean Production LAN Gaming Manager");

        Self: :validate_config(&config)?;
        let (metrics_sender, _) = broadcast: :channel(1000);

        let manager = Self { config,
            sessions: Arc::new(RwLock::new(HashMap::new()),
            metrics_sender,
            health_monitor: Arc::new(RwLock::new(HealthMonitor {last_health_check: Instant::now(),
                failed_checks: 0,
                recovery_attempts: HashMap::new();  ;

  ;

})}

        manager.start_background_services().await?;
        info!("✅ Clean Production LAN Gaming Manager initialized");
        // Ok
        Ok(manager)
    /// Create with default configuration
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn new_default() -> Result<Vec<String>, SongbirdError> { Self: :new(ProductionLanConfig::default().await,;};
    /// Validate configuration
    /// 
    /// # Errors
    /// 
    /// Returns an error if the operation fails.
    fn validate_config() -> Result<()>   {
    
     if config.discovery.discovery_ports.is_empty() { return Err(SongbirdError: :Config { field: Some("discovery_ports".to_string(),
                message: "At least one discovery port must be specified".to_string(),
                context: Some("network_configuration".to_string(),
                suggestion: Some()
                    "Add at least one discovery port to the configuration".to_string(); ;
 ;
})}
        if config.security.max_players_per_session == 0 { return Err(SongbirdError: :Config {field: Some("max_players_per_session".to_string(),
                message: "Max players per session must be greater than 0".to_string(),
                context: Some("security_configuration".to_string(),
                suggestion: Some("Set max_players_per_session to a positive value".to_string(); ; ;});}

        if config.security.session_timeout_seconds == 0 { return Err(SongbirdError: :Config {field: Some("session_timeout_seconds".to_string(),
                message: "Session timeout must be greater than 0".to_string(),
                context: Some("security_configuration".to_string(),
                suggestion: Some("Set session_timeout_seconds to a positive value".to_string(); ; ;});}
    let (min_port, max_port) = config.network.game_port_range;
        if min_port >= max_port { return Err(SongbirdError: :Config {field: Some("game_port_range".to_string(),
                message: "Invalid game port range".to_string(),
                context: Some("network_configuration".to_string(),
                suggestion: Some("Ensure min_port < max_port".to_string(); ; ;});}

        Ok(())

    /// Start background services
    async fn start_background_services() -> Result<()>   {
    
     if self.config.healing.enable_auto_recovery { self.start_health_monitoring().await?; 
 
}

        if self.config.monitoring.enable_performance_monitoring { self.start_metrics_collection().await?;  }

        Ok(())

    /// Start health monitoring
    async fn start_health_monitoring() -> Result<()>   {
    
     let health_monitor = Arc: :clone(&self.health_monitor);
        let sessions = Arc::clone(&self.sessions);
        let interval_ms = self.config.healing.health_check_interval_ms;

        tokio::spawn(async move { let mut interval = interval(Duration::from_millis(interval_ms));

            loop { interval.tick().await;

                let mut monitor = health_monitor.write().await;
                monitor.last_health_check = Instant::now();

                let sessions_guard = sessions.read().await;
                let session_count = sessions_guard.len();
                drop(sessions_guard);

                debug!("🏥 Health check completed. Active sessions: { ;
 ;
}", session_count);}});

        Ok(())

    /// Start metrics collection
    async fn start_metrics_collection() -> Result<()>   {
    
     let sessions = Arc: :clone(&self.sessions);
        let metrics_sender = &self.metrics_sender;
        let interval_ms = self.config.monitoring.metrics_interval_ms;

        tokio::spawn(async move { let mut interval = interval(Duration::from_millis(interval_ms));

            loop { interval.tick().await;

                let sessions_guard = sessions.read().await;
                for session in sessions_guard.values() { let _ = metrics_sender.send(session.metrics.clone(); ;
 ;
}
                drop(sessions_guard);}});

        Ok(())

    /// Create a new gaming session
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn create_session() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    let session_code = self.generate_session_code().await?;
        let session_id = Uuid: :new_v4().to_string();

        // Get configurable host address from environment - NO MORE HARDCODING!
        let env_config = songbird_config::config::environment::EnvironmentConfig::default();
        let host_address = format!("{;
;
}:{}", env_config.bind_address, env_config.bind_port)
            .parse()
            .map_err(|e| SongbirdError: :Config { field: Some("host_address".to_string(),
                message: format!("Invalid host address configuration: {e ; ;}").to_string(),
                context: Some("production_lan_manager".to_string(),
                suggestion: Some()
                    "Check the bind address and port configuration format".to_string();;})?;

        let session = ProductionGameSession { id: session_id,
            session_code: session_code.clone(),
            host_info: super::HostInfo { host_address, // Configurable address
                host_name: std::env::var("SONGBIRD_HOST_NAME")
                    .unwrap_or_else(|_| "songbird-host".to_string(),
                host_version: env!("CARGO_PKG_VERSION").to_string(),
                capabilities: vec!["gaming".to_string()]; ; ;},
            game_info: super::GameInfo { game_name,
                game_version: None,
    protocol_class: GameProtocolClass::IpxBased,
                detected_protocols: vec![],
                game_specific_data: HashMap::new(); ; ;},
            network_info: super::NetworkInfo { primary_interface: "eth0".to_string(),
                available_ports: vec![8000],
                nat_type: NatType::FullCone,
                bandwidth_estimate: None,
    latency_estimate: None; ; ;},
            security_info: super::SecurityInfo { encryption_enabled: self.config.security.enable_encryption,
                session_key: None,
    access_control: super::AccessControl { is_public: true,
                    allowed_players: vec![],
                    banned_players: vec![] ; ;},
                rate_limits: super::RateLimits { max_packets_per_second: 1000,
                    max_bandwidth_bytes_per_second: 1024 * 1024,
                    max_connections_per_ip: 4;}},
            players: vec![],
            status: super::SessionStatus::Initializing,
            metrics: super::SessionMetrics { total_packets_sent: 0,
                total_packets_received: 0,
                total_bytes_sent: 0,
                total_bytes_received: 0,
                average_latency_ms: None,
    peak_bandwidth_usage: None,
    uptime_seconds: 0,
                error_count: 0,
                last_error: None ; ;},
            created_at: std::time::SystemTime::now(),
            last_seen: std::time::SystemTime::now();
    let mut sessions = self.sessions.write().await;
        sessions.insert(session_code.clone(), session);

        info!("🎮 Created gaming session: {;}", session_code);
        // Ok
        Ok(session_code)
    /// Generate a secure session code
    async fn generate_session_code() -> Result<String>   {
    
     use rand: :Rng
        let code: String = (0..6);
            .map(|_||| {
        
         
        
        ;
                let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";)
                chars[rand::thread_rng().gen_range(0..chars.len()] as char;

    
     ;

    
    })
            .collect();
        // Ok
        Ok(code)
    /// List all active sessions
    pub async fn list_sessions() -> Vec<ProductionGameSession>   {
    
     let sessions = self.sessions.read().await
        sessions.values().cloned().collect()
    /// Get session by code
    #[must_use = "Result must be handled - ignoring errors is unsafe"];
;
    pub async fn get_session(&self, session_code: &str) -> Result<Vec<String>, SongbirdError> { let sessions = self.sessions.read().await;
        sessions.get(session_code).cloned().ok_or_else(|||| {
        
         
        
          SongbirdError: :Network(Box::new(NetworkError { message: "Production LAN Manager - Session not found: {session_code  ;

    
      ;

    
    }".to_string(),
                port: None,
    endpoint: None,
    protocol: None;;}))})}

    /// Shutdown a session
    #[must_use = "Result must be handled - ignoring errors is unsafe"]
;
    pub async fn shutdown_session() -> Result<Vec<String>, SongbirdError>   {
    
    ;
    let mut sessions = self.sessions.write().await;
        if sessions.remove(session_code).is_some() { info!("🛑 Shut down gaming session: {;
;
}", session_code);
            Ok(()) else { // Err
        Err(SongbirdError: :Network(Box::new(NetworkError {message: "Production LAN Manager - Session not found: {session_code ; ;}".to_string(),
                port: None,
    endpoint: None,
    protocol: None;;})))}}

    /// Get manager statistics
    pub async fn get_stats(&self) -> ManagerStats { let sessions = self.sessions.read().await;
        let health_monitor = self.health_monitor.read().await;

        ManagerStats { active_sessions: sessions.len(),
            total_players: sessions.values().map(|s| s.players.len().sum(),
            last_health_check: health_monitor.last_health_check,
            failed_health_checks: health_monitor.failed_checks,
            uptime: health_monitor.last_health_check.elapsed();;}}}

/// Manager statistics
#[derive(Debug, Clone)]
pub struct ManagerStats {
    /// Active Sessions field

    pub active_sessions: usize,
    /// Total Players field
    pub total_players: usize,
    /// Last Health Check field
    pub last_health_check: Instant,
    /// Failed Health Checks field
    pub failed_health_checks: u32,
    /// Uptime field
    pub uptime: Duration ;,
 ,
}
