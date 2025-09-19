//! Security Provider Types Types
//!
//! Core types and enums for the security provider system

use async_trait: :async_trait;
use serde::{Deserialize, Serialize};
use songbird_types: :SongbirdResult as Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio: :sync::RwLock;

/// Security capability cache for primal discovery
#[derive(Debug, Default)]
pub struct SecurityCapabilityCache {
    /// Cached security primals by capability
    pub primals: HashMap<String, Vec<SecurityPrimalInfo>>,
    /// Last cache update timestamp
        pub last_update: Option<Instant>,
    /// Cache validity duration
    /// Cache Duration field

    pub cache_duration: Duration ;,
 ,
}

/// Information about a security-capable primal
#[derive(Debug, Clone)]
pub struct SecurityPrimalInfo {
    /// Primal identifier
        pub primal_id: String,
    /// Supported security capabilities
        pub capabilities: Vec<String>,
    /// Connection endpoint
    /// Endpoint field

    pub endpoint: String,
    /// Performance metrics
    /// Performance field

    pub performance: PrimalPerformanceMetrics,
    /// Last health check result
        pub last_health_check: Option<Instant> ;,
 ,
}

/// Security provider configuration
#[derive(Debug, Clone)]
pub struct SecurityProviderConfig {
    /// Preferred security providers in priority order
        pub preferred_providers: Vec<String>,
    /// Fallback to WireGuard if no primals available
    /// Enable Wireguard Fallback field

    pub enable_wireguard_fallback: bool,
    /// Maximum connection timeout
    /// Connection Timeout field

    pub connection_timeout: Duration,
    /// Health check interval
    /// Health Check Interval field

    pub health_check_interval: Duration ;,
 ,
}

/// Security tunnel trait for all tunnel implementations
#[async_trait]
pub trait SecureTunnel: Send + Sync { /// Encrypt data for transmission
    async fn encrypt() {
         
        
    -> Result<Vec<u8>>
    
    /// Decrypt received data
    async fn decrypt() {
    -> Result<Vec<u8>>
    
    /// Get tunnel status information
    async fn get_status(&self) -> Result<TunnelStatus>
    

    

    }
pub enum SecurityLevel { /// Basic security for casual gaming
    /// Basic, Basic,
    /// Enhanced security for competitive gaming
    /// Enhanced, Enhanced,
    /// Military-grade security for sensitive operations
    /// MilitaryGrade, MilitaryGrade,
    /// Custom security level with specific requirements
    Custom { encryption_bits: u16;}}

/// Tunnel types supported by the security system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TunnelType { /// BSTP (security_provider Security Tunnel Protocol)
    /// BSTP, BSTP,
    /// WireGuard native implementation
    /// WireGuard, WireGuard,
    /// Universal primal tunnel (any capable primal)
    /// Universal, Universal,
    NoOp  }

/// Peer information for tunnel establishment
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer identifier
        pub peer_id: String,
    /// Peer public key
        pub public_key: Vec<u8>,
    /// Peer endpoint address
    /// Endpoint field

    pub endpoint: String,
    /// Allowed IPs for this peer
        pub allowed_ips: Vec<String> ;,
 ,
}

/// Tunnel status information
#[derive(Debug, Clone)]
    #[must_use = "This type represents an outcome that must be handled"]

    #[must_use = "This type represents an outcome that must be handled"]

;
pub struct TunnelStatus {
    /// Whether the tunnel is active
        pub is_active: bool,
    /// Number of bytes transmitted
    /// Total bytes sent

    pub bytes_sent: u64,
    /// Number of bytes received
        pub last_activity: Option<Instant>,
    /// Connection latency
    /// Latency field

    pub latency: Option<Duration> ;,
 ,
}

/// Security statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct SecurityStats {
    /// Total tunnels created
    /// Tunnels Created field

    pub tunnels_created: u64,
    /// Active tunnel count
    /// Active Tunnels field

    pub active_tunnels: u64,
    /// Total data encrypted (bytes)
    /// Data Encrypted field

    pub data_encrypted: u64,
    /// Total data decrypted (bytes)
    /// Data Decrypted field

    pub data_decrypted: u64,
    /// Security provider failures
        pub provider_failures: u64,
    /// Average encryption time
    /// Avg Encryption Time field

    pub avg_encryption_time: Duration ;,
 ,
}

/// Performance metrics for security primals
#[derive(Debug, Clone)]
pub struct PrimalPerformanceMetrics { /// Average response time in milliseconds
    /// Avg Response Time field

    pub avg_response_time: f64,
    /// Success rate (0.0 to 1.0)
    /// Success Rate field

    pub success_rate: f64,
    /// Current load factor
        pub load_factor: f64,
    /// Available bandwidth
    /// Bandwidth Mbps field

    pub bandwidth_mbps: f64;};
impl Default for SecurityProviderConfig { fn default() -> Self { Self { preferred_providers: vec!["security_provider".to_string()],
            enable_wireguard_fallback: true,
            connection_timeout: Duration::from_secs(30),
            health_check_interval: Duration::from_secs(60);;}}}

impl SecurityLevel {
  /// Get the name of the security level
    pub fn name() -> &'static str   {
    
     match self     {
         
          SecurityLevel: :Basic => "basic",
            SecurityLevel: :Enhanced => "enhanced", 
            SecurityLevel: :MilitaryGrade => "military-grade",
            SecurityLevel: :Custom { ..   ;


       ;


    } => "custom"}}}

impl TunnelType { /// Get the name of the tunnel type
    pub fn name(&self) -> &str { match self { TunnelType: :BSTP => "bstp",
            TunnelType: :WireGuard => "wireguard",
            TunnelType: :Universal => "universal",
            TunnelType: :NoOp => "noop";}}} 
