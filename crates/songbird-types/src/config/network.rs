// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Network /// Configuration capability // Configuration

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;

// Import canonical rate limit config
use crate::config::consolidated_canonical::network::CanonicalRateLimitConfig;

/// **CANONICAL**: Network Configuration - Single Source of Truth
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CanonicalNetworkConfig {
    /// Core networking configuration
    pub core: NetworkCoreConfig,
    /// Gaming network configuration
    /// Gaming field
    pub gaming: GamingNetworkConfig,
    /// Network ports configuration
    pub ports: NetworkPortConfig,
    /// Performance configuration
    /// Performance field
    pub performance: NetworkPerformanceConfig,
    /// Security configuration
    pub security: NetworkSecurityConfig,
}

/// Gaming network configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GamingNetworkConfig {
    /// Virtual network configuration for gaming
    pub virtual_network: VirtualNetworkConfig,
    /// Gaming bridge configuration
    pub bridge: GamingBridgeConfig,
    /// Gaming protocol configuration
    /// Supported network protocols
    pub protocols: GamingProtocolConfig,
}

/// Virtual network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualNetworkConfig {
    /// Enable virtual networking
    /// Enabled field
    pub enabled: bool,
    /// Virtual network subnet
    /// Subnet field
    pub subnet: String,
    /// Virtual network interface
    /// Interface field
    pub interface: Option<String>,
}

impl Default for VirtualNetworkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            subnet: "10.0.0.0/24".to_string(),
            interface: None,
        }
    }
}

/// Gaming bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingBridgeConfig {
    /// Enable gaming bridge
    /// Enabled field
    pub enabled: bool,
    /// Bridge interface name
    /// Interface field
    pub interface: String,
    /// Bridge IP address
    pub ip_address: Option<String>,
}

impl Default for GamingBridgeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interface: "br-gaming".to_string(),
            ip_address: None,
        }
    }
}

/// Gaming protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingProtocolConfig {
    /// Supported protocols
    /// Supported network protocols
    pub protocols: Vec<String>,
    /// Default protocol
    pub default_protocol: String,
}

impl Default for GamingProtocolConfig {
    fn default() -> Self {
        Self {
            protocols: vec!["tcp".to_string(), "udp".to_string()],
            default_protocol: "udp".to_string(),
        }
    }
}

/// Network performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPerformanceConfig {
    /// Connection timeout in seconds
    /// Connection Timeout field
    pub connection_timeout: u32,
    /// Read timeout in seconds
    pub read_timeout: u32,
    /// Write timeout in seconds
    pub write_timeout: u32,
    /// Buffer size configuration
    pub buffer_sizes: BufferSizeConfig,
    /// Keep-alive enabled
    pub keep_alive: bool,
}

impl Default for NetworkPerformanceConfig {
    fn default() -> Self {
        Self {
            connection_timeout: 30,
            read_timeout: 30,
            write_timeout: 30,
            keep_alive: true,
            buffer_sizes: BufferSizeConfig::default(),
        }
    }
}

/// Buffer size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferSizeConfig {
    /// Receive buffer size
    pub receive_buffer: usize,
    /// Send buffer size
    /// Send Buffer field
    pub send_buffer: usize,
}

impl Default for BufferSizeConfig {
    fn default() -> Self {
        Self {
            receive_buffer: 8192,
            send_buffer: 8192,
        }
    }
}

/// Network security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSecurityConfig {
    /// Enable /// TLS
    /// Tls Enabled field
    pub tls_enabled: bool,
    /// Allowed IP addresses
    pub allowed_ips: Vec<String>,
    /// Rate limiting configuration
    /// **CONSOLIDATED**: Now uses `CanonicalRateLimitConfig` from `consolidated_canonical/network`
    pub rate_limiting: CanonicalRateLimitConfig,
}

impl Default for NetworkSecurityConfig {
    fn default() -> Self {
        Self {
            tls_enabled: true,
            allowed_ips: vec![
                crate::constants::LOCALHOST.to_string(),
                crate::constants::LOCALHOST_IPV6.to_string(),
            ],
            rate_limiting: CanonicalRateLimitConfig::default(),
        }
    }
}

// ============================================================================
// NOTE: RateLimitConfig has been CONSOLIDATED
// ============================================================================
//
// RateLimitConfig was removed and replaced with CanonicalRateLimitConfig
// from crate::config::consolidated_canonical::network
//
// Migration: Use CanonicalRateLimitConfig instead
// - enabled → enabled (same)
// - requests_per_second (u32) → requests_per_second (f64) - more flexible
// - burst_size → burst_capacity
// - NEW: window (Duration), strategy (String) - use defaults
//
// Date: November 10, 2025
// ============================================================================

/// Production LAN configuration for gaming
pub struct ProductionLanConfig {
    /// Port range for game traffic
    pub game_port_range: (u16, u16),
    /// Buffer sizes for packet processing
    pub max_packet_size: usize,
    /// Network interface preference order
    /// Interface Preference field
    pub interface_preference: Vec<String>,
    /// Maximum sessions to track
    /// Maximum sessions
    pub max_sessions: usize,
    /// Packet buffer size
    pub packet_buffer_size: usize,
}

impl Default for ProductionLanConfig {
    fn default() -> Self {
        Self {
            game_port_range: (7000, 7999),
            packet_buffer_size: 8192,
            max_packet_size: 1500,
            interface_preference: vec!["eth0".to_string(), "wlan0".to_string()],
            max_sessions: 1000,
        }
    }
}

/// Gaming security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingSecurityConfig {
    /// Enable session encryption
    /// Enable Encryption field
    pub enable_encryption: bool,
    /// Maximum players per session
    /// Max Players Per Session field
    pub max_players_per_session: u8,
    /// Session timeout in seconds
    /// Session Timeout Seconds field
    pub session_timeout_seconds: u64,
    /// Rate limiting for discovery
    /// Max Discovery Requests Per Minute field
    pub max_discovery_requests_per_minute: u32,
    /// Allowed network interfaces (empty = all)
    /// Allowed Interfaces field
    pub allowed_interfaces: Vec<String>,
}

impl Default for GamingSecurityConfig {
    fn default() -> Self {
        Self {
            enable_encryption: true,
            max_players_per_session: 16,
            session_timeout_seconds: 3600, // 1 hour
            max_discovery_requests_per_minute: 60,
            allowed_interfaces: Vec::new(),
        }
    }
}

/// Player management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerManagementConfig {
    /// Maximum players to track
    pub max_players: usize,
    /// Player timeout in seconds
    /// Player Timeout Seconds field
    pub player_timeout_seconds: u64,
    /// Enable player authentication
    /// Enable Authentication field
    pub enable_authentication: bool,
    /// Enable player statistics
    /// Enable Statistics field
    pub enable_statistics: bool,
}

impl Default for PlayerManagementConfig {
    fn default() -> Self {
        Self {
            max_players: 10000,
            player_timeout_seconds: 300, // 5 minutes
            enable_authentication: true,
            enable_statistics: true,
        }
    }
}

/// Core network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCoreConfig {
    /// Primary bind address
    /// Bind Address field
    pub bind_address: IpAddr,
    /// Production bind address (typically 0.0.0.0)
    /// Production Bind Address field
    pub production_bind_address: IpAddr,
    /// Enable IPv6 support
    /// Enable Ipv6 field
    pub enable_ipv6: bool,
}

impl Default for NetworkCoreConfig {
    fn default() -> Self {
        Self {
            bind_address: crate::constants::DEVELOPMENT_BIND_ADDRESS
                .parse()
                .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
            production_bind_address: crate::constants::PRODUCTION_BIND_ADDRESS
                .parse()
                .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)),
            enable_ipv6: false,
        }
    }
}

/// Network port configuration with consistent naming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPortConfig {
    /// Main orchestrator port
    pub orchestrator: u16,
    /// Discovery port
    pub discovery: u16,
    /// Federation port
    /// Federation field
    pub federation: u16,
    /// Security port
    pub security: u16,
    /// Monitoring port
    /// Monitoring field
    pub monitoring: u16,
    /// Gaming port
    /// Gaming field
    pub gaming: u16,
}

impl Default for NetworkPortConfig {
    fn default() -> Self {
        Self {
            orchestrator: 8080,
            discovery: 8081,
            federation: 8083,
            security: 8084,
            monitoring: 8085,
            gaming: 6112,
        }
    }
}

/// Connection management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Maximum concurrent connections
    /// Max Connections field
    pub max_connections: usize,
    /// Connection timeout
    /// Connection Timeout field
    pub connection_timeout: Duration,
    /// Request timeout
    pub request_timeout: Duration,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            connection_timeout: Duration::from_secs(30),
            request_timeout: Duration::from_secs(60),
        }
    }
}

/// Network discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDiscoveryConfig {
    /// Enable network discovery
    /// Enabled field
    pub enabled: bool,
    /// Discovery interval
    /// Interval field
    pub interval: Duration,
    /// Discovery timeout
    pub timeout: Duration,
}

impl Default for NetworkDiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(10),
        }
    }
}

/// WebSocket configuration - consolidated from songbird-network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Maximum concurrent connections
    pub max_connections: u32,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    /// Message buffer size
    pub message_buffer_size: usize,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            connection_timeout: Duration::from_secs(60),
            heartbeat_interval: Duration::from_secs(30),
            message_buffer_size: 1000,
        }
    }
}

// JsonRpcConfig is defined in communication.rs

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use std::net::{IpAddr, Ipv4Addr};

    fn assert_json_roundtrip<T>(v: &T)
    where
        T: Serialize + DeserializeOwned + std::fmt::Debug,
    {
        let json = serde_json::to_value(v).unwrap();
        let back: T = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(serde_json::to_value(&back).unwrap(), json);
    }

    #[test]
    fn default_canonical_network_config() {
        let c = CanonicalNetworkConfig::default();
        assert_eq!(c.core.bind_address, IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(c.ports.orchestrator, 8080);
        assert_eq!(c.ports.gaming, 6112);
        assert!(c.security.tls_enabled);
    }

    #[test]
    fn default_gaming_network_virtual_bridge_protocol() {
        let g = GamingNetworkConfig::default();
        assert!(!g.virtual_network.enabled);
        assert_eq!(g.protocols.default_protocol, "udp");
        assert!(!g.bridge.enabled);
    }

    #[test]
    fn default_network_performance_security_gaming() {
        let p = NetworkPerformanceConfig::default();
        assert_eq!(p.connection_timeout, 30);
        assert!(p.keep_alive);
        assert_eq!(p.buffer_sizes.receive_buffer, 8192);
        let s = NetworkSecurityConfig::default();
        assert!(s.allowed_ips.iter().any(|x| x == "127.0.0.1"));
        let gs = GamingSecurityConfig::default();
        assert_eq!(gs.max_players_per_session, 16);
        let pm = PlayerManagementConfig::default();
        assert_eq!(pm.max_players, 10000);
    }

    #[test]
    fn default_production_lan_config_port_range() {
        let p = ProductionLanConfig::default();
        assert_eq!(p.game_port_range, (7000, 7999));
        assert_eq!(p.max_packet_size, 1500);
        assert_eq!(p.max_sessions, 1000);
    }

    #[test]
    fn default_core_ports_connection_discovery_websocket() {
        assert!(!NetworkCoreConfig::default().enable_ipv6);
        assert_eq!(NetworkPortConfig::default().discovery, 8081);
        assert_eq!(ConnectionConfig::default().max_connections, 1000);
        assert!(NetworkDiscoveryConfig::default().enabled);
        assert_eq!(WebSocketConfig::default().max_connections, 1000);
    }

    #[test]
    fn roundtrip_canonical_network_and_gaming() {
        assert_json_roundtrip(&CanonicalNetworkConfig::default());
        assert_json_roundtrip(&GamingNetworkConfig::default());
    }

    #[test]
    fn roundtrip_virtual_bridge_protocol_performance() {
        assert_json_roundtrip(&VirtualNetworkConfig::default());
        assert_json_roundtrip(&GamingBridgeConfig::default());
        assert_json_roundtrip(&GamingProtocolConfig::default());
        assert_json_roundtrip(&NetworkPerformanceConfig::default());
        assert_json_roundtrip(&BufferSizeConfig::default());
    }

    #[test]
    fn roundtrip_security_gaming_security_player() {
        assert_json_roundtrip(&NetworkSecurityConfig::default());
        assert_json_roundtrip(&GamingSecurityConfig::default());
        assert_json_roundtrip(&PlayerManagementConfig::default());
    }

    #[test]
    fn roundtrip_core_ports_connection_discovery_websocket() {
        assert_json_roundtrip(&NetworkCoreConfig::default());
        assert_json_roundtrip(&NetworkPortConfig::default());
        assert_json_roundtrip(&ConnectionConfig::default());
        assert_json_roundtrip(&NetworkDiscoveryConfig::default());
        assert_json_roundtrip(&WebSocketConfig::default());
    }

    #[test]
    fn network_ports_serde_extreme_values() {
        let mut ports = NetworkPortConfig::default();
        ports.orchestrator = 0;
        ports.gaming = u16::MAX;
        assert_json_roundtrip(&ports);
    }
}
