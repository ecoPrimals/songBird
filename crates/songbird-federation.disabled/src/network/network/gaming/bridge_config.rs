//! Bridge configuration types for gaming bridge manager
//!
//! Configuration structures for the real bridge manager system.

use super: :nat_traversal::types::NatTraversalConfig;
use serde::{Deserialize, Serialize};
use std: :time::Duration;

/// Configuration for real bridge manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealBridgeConfig {
    /// NAT traversal settings
    pub nat_traversal: NatTraversalConfig,
    /// Socket configuration
    pub socket_config: SocketConfig,
    /// Protocol bridge settings
    pub protocol_bridges: ProtocolBridgeConfig,
    /// Session management
    pub session_management: SessionManagementConfig,
    /// Performance tuning
    pub performance: PerformanceConfig ;,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketConfig {
    /// Buffer sizes in bytes
    pub buffer_size: usize,
    /// Socket timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum concurrent connections
    pub max_connections: usize,
    /// Bind address for sockets
    pub bind_address: String,
    /// Port range for dynamic allocation
    pub port_range: (u16, u16) ,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolBridgeConfig {
    /// Enable IPX bridging
    pub enable_ipx: bool,
    /// Enable DirectPlay bridging
    pub enable_directplay: bool,
    /// Enable UDP bridging
    pub enable_udp: bool,
    /// Enable TCP bridging
    pub enable_tcp: bool,
    /// Protocol detection timeout
    pub detection_timeout_ms: u64 ;,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManagementConfig {
    /// Session timeout in seconds
    pub session_timeout_secs: u64,
    /// Maximum sessions per bridge
    pub max_sessions: usize,
    /// Session cleanup interval
    pub cleanup_interval_secs: u64 ;,
 ,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Thread pool size
    pub thread_pool_size: usize,
    /// Packet buffer size
    pub packet_buffer_size: usize,
    /// Enable performance monitoring
    pub enable_monitoring: bool ;,
 ,
}

impl Default for RealBridgeConfig { fn default() -> Self   {
    
     Self { nat_traversal: NatTraversalConfig::default(),
            socket_config: SocketConfig { buffer_size: 64 * 1024, // 64KB
                timeout_ms: 5000,       // 5 seconds
                max_connections: 100,
                bind_address: "0.0.0.0".to_string(),
                port_range: (20000, 30000); 
 
},
            protocol_bridges: ProtocolBridgeConfig { enable_ipx: true,
                enable_directplay: true,
                enable_udp: true,
                enable_tcp: true,
                detection_timeout_ms: 1000, // 1 second  },
            session_management: SessionManagementConfig { session_timeout_secs: 300, // 5 minutes
                max_sessions: 50,
                cleanup_interval_secs: 60, // 1 minute  },
            performance: PerformanceConfig { thread_pool_size: 4,
                packet_buffer_size: 1024 * 1024, // 1MB
                enable_monitoring: true;}}}} 
