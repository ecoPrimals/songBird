//! # 🎮 Gaming Configuration Module
//!
//! **CONSOLIDATED GAMING CONFIG** ✅
//!
//! This module consolidates all gaming-related configuration that was previously
//! scattered across the network crate. This provides a single source of truth
//! for all gaming network configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Gaming network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingConfig  {/// Gaming bridge configuration
    pub bridge: GamingBridgeConfig,
    /// Protocol detection configuration
    pub protocol_detection: ProtocolDetectionConfig,
    /// Session management configuration
    pub session_management: SessionManagementConfig,
    /// Performance optimization settings
    pub performance: GamingPerformanceConfig,
    /// Security settings for gaming networks
    pub security: GamingSecurityConfig,
}

/// Gaming bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingBridgeConfig  {/// Enable the gaming bridge
    pub enabled: bool,
    /// Bind address for the bridge
    pub bind_address: String,
    /// Port range for dynamic port allocation
    pub port_range: (u16, u16)
    /// Maximum concurrent sessions
    pub max_sessions: usize,
    /// Session timeout in seconds
    pub session_timeout_seconds: u64,
    /// Buffer size for packet forwarding
    pub buffer_size: usize,
    /// Enable packet logging for debugging
    pub enable_packet_logging: bool,
}

/// Protocol detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolDetectionConfig  {/// Enable automatic protocol detection
    pub enabled: bool,
    /// Detection timeout in milliseconds
    pub timeout_ms: u64,
    /// Minimum confidence threshold (0.0 to 1.0)
    pub confidence_threshold: f32,
    /// Enable deep packet inspection
    pub deep_inspection: bool,
    /// Maximum packet size to analyze
    pub max_packet_size: usize,
    /// Protocol signature database path
    pub signature_database_path: Option<String>,
}

/// Session management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManagementConfig  {/// Maximum players per session
    pub max_players_per_session: usize,
    /// Maximum concurrent sessions per host
    pub max_sessions_per_host: usize,
    /// Session cleanup interval in seconds
    pub cleanup_interval_seconds: u64,
    /// Inactive session timeout in seconds
    pub inactive_timeout_seconds: u64,
    /// Enable session persistence
    pub enable_persistence: bool,
    /// Session data storage path
    pub session_storage_path: Option<String>,
}

/// Gaming performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingPerformanceConfig  {/// Enable zero-copy packet forwarding
    pub zero_copy_forwarding: bool,
    /// Worker thread count (0 = auto-detect)
    pub worker_threads: usize,
    /// Packet processing batch size
    pub batch_size: usize,
    /// Enable packet coalescing
    pub packet_coalescing: bool,
    /// Coalescing timeout in microseconds
    pub coalescing_timeout_us: u64,
    /// Enable hardware acceleration if available
    pub hardware_acceleration: bool,
}

/// Gaming security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingSecurityConfig  {/// Enable packet filtering
    pub packet_filtering: bool,
    /// Maximum packet rate per session (packets/second)
    pub max_packet_rate: u64,
    /// Enable DDoS protection
    pub ddos_protection: bool,
    /// Rate limiting window in seconds
    pub rate_limit_window_seconds: u64,
    /// Banned IP addresses
    pub banned_ips: Vec<String>,
    /// Enable encryption for sensitive protocols
    pub enable_encryption: bool,
}

/// Legacy protocol support configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyProtocolConfig  {/// Enable IPX protocol support
    pub ipx_support: bool,
    /// Enable DirectPlay protocol support
    pub directplay_support: bool,
    /// Enable NetBIOS protocol support
    pub netbios_support: bool,
    /// IPX network address for emulation
    pub ipx_network_address: Option<[u8; 4]>,
    /// Custom protocol handlers
    pub custom_handlers: HashMap<String, String>)
}

/// Tunnel configuration for VPN-like gaming networks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConfig  {/// Enable tunnel creation
    pub enabled: bool,
    /// Tunnel encryption method
    pub encryption_method: TunnelEncryption,
    /// MTU size for tunnel packets
    pub mtu_size: u16,
    /// Keep-alive interval in seconds
    pub keepalive_interval_seconds: u64,
    /// Tunnel timeout in seconds
    pub tunnel_timeout_seconds: u64,
}

/// Tunnel encryption methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TunnelEncryption  {/// No encryption
    None,
    /// AES-256-GCM encryption
    Aes256Gcm,
    /// ChaCha20-Poly1305 encryption
    ChaCha20Poly1305,
    /// Custom encryption method
    Custom(String)
}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for GamingConfig  {fn default() -> Self  {Self {
            bridge: GamingBridgeConfig::default(),
            protocol_detection: ProtocolDetectionConfig::default(),
            session_management: SessionManagementConfig::default(),
            performance: GamingPerformanceConfig::default(),
            security: GamingSecurityConfig::default(),
        }
    }
}

impl Default for GamingBridgeConfig  {fn default() -> Self  {Self {
            enabled: true,
            bind_address: "0.0.0.0".to_string(),
            port_range: (6112, 6200), // Common gaming port range
            max_sessions: 1000,
            session_timeout_seconds: 3600, // 1 hour
            buffer_size: 65536, // 64KB
            enable_packet_logging: false,
        }
    }
}

impl Default for ProtocolDetectionConfig  {fn default() -> Self  {Self {
            enabled: true,
            timeout_ms: 5000, // 5 seconds
            confidence_threshold: 0.8,
            deep_inspection: true,
            max_packet_size: 1500, // Standard MTU
            signature_database_path: None,
        }
    }
}

impl Default for SessionManagementConfig  {fn default() -> Self  {Self {
            max_players_per_session: 100,
            max_sessions_per_host: 10,
            cleanup_interval_seconds: 300, // 5 minutes
            inactive_timeout_seconds: 1800, // 30 minutes
            enable_persistence: false,
            session_storage_path: None,
        }
    }
}

impl Default for GamingPerformanceConfig  {fn default() -> Self  {Self {
            zero_copy_forwarding: true,
            worker_threads: 0, // Auto-detect
            batch_size: 32,
            packet_coalescing: true,
            coalescing_timeout_us: 100, // 100 microseconds
            hardware_acceleration: false, // Conservative default
        }
    }
}

impl Default for GamingSecurityConfig  {fn default() -> Self  {Self {
            packet_filtering: true,
            max_packet_rate: 1000, // 1000 packets/second
            ddos_protection: true,
            rate_limit_window_seconds: 60,
            banned_ips: Vec::new(),
            enable_encryption: false, // Performance vs security tradeoff
        }
    }
}

impl Default for LegacyProtocolConfig  {fn default() -> Self  {Self {
            ipx_support: true,
            directplay_support: true,
            netbios_support: true,
            ipx_network_address: Some([0x00, 0x00, 0x00, 0x01]), // Default IPX network
            custom_handlers: HashMap::new()),
        }
    }
}

impl Default for TunnelConfig  {fn default() -> Self  {Self {
            enabled: false, // Disabled by default for performance
            encryption_method: TunnelEncryption::Aes256Gcm,
            mtu_size: 1400, // Conservative MTU to avoid fragmentation
            keepalive_interval_seconds: 30,
            tunnel_timeout_seconds: 300, // 5 minutes
        }
    }
}

impl Default for TunnelEncryption {
    fn default() -> Self {
        Self::Aes256Gcm
    }
}

// ============================================================================
// CONFIGURATION BUILDERS
// ============================================================================

impl GamingConfig {
    /// Create a new gaming configuration with sensible defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration optimized for low-latency gaming
    pub fn low_latency() -> Self {
        Self {
            bridge: GamingBridgeConfig {
                buffer_size: 32768, // Smaller buffer for lower latency
                ..Default::default()
            })
            performance: GamingPerformanceConfig  {zero_copy_forwarding: true,
                packet_coalescing: false, // Disable coalescing for lower latency
                coalescing_timeout_us: 10, // Very short timeout if coalescing is enabled
                ..Default::default()
            })
            ..Default::default()
        }
    }

    /// Create a configuration optimized for high throughput
    pub fn high_throughput() -> Self {
        Self {
            bridge: GamingBridgeConfig {
                buffer_size: 131072, // Larger buffer for throughput
                ..Default::default()
            })
            performance: GamingPerformanceConfig  {zero_copy_forwarding: true,
                batch_size: 64, // Larger batches
                packet_coalescing: true,
                coalescing_timeout_us: 500, // Longer timeout for better coalescing
                ..Default::default()
            })
            ..Default::default()
        }
    }

    /// Create a secure gaming configuration
    pub fn secure() -> Self  {Self {security: GamingSecurityConfig {
                packet_filtering: true,
                max_packet_rate: 500, // More restrictive rate limiting
                ddos_protection: true,
                enable_encryption: true, // Enable encryption
                ..Default::default()
            })
            ..Default::default()
        }
    }
}

// ============================================================================
// Gaming constants are now consolidated in songbird-types::constants::GamingConstants
// Use those constants instead of defining them here.