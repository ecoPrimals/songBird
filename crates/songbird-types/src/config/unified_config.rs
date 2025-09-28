//! # 🔧 Unified Configuration System
//!
//! **CANONICAL CONFIGURATION CONSOLIDATION** ✅
//!
//! This module provides a comprehensive, hierarchical configuration system that
//! consolidates all fragmented configuration structures across the Songbird ecosystem.
//!
//! **MODULARIZED**: Large configuration sections extracted to focused modules ✅
//! - `system.rs` - System configuration
//! - `network.rs` - Network configuration  
//! - `security.rs` - Security configuration
//! - Additional modules for performance, gaming, observability, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

// Import modularized configuration types
use super::system::{SystemConfig, LogLevel};
use super::network::NetworkConfig;
use super::security::SecurityConfig;
use super::performance::CanonicalPerformanceConfig as PerformanceConfig;

// ============================================================================
// MAIN UNIFIED CONFIGURATION
// ============================================================================

/// **CANONICAL**: Main Songbird configuration - single source of truth
/// 
/// This is now an alias to the consolidated canonical config.
/// Use `CanonicalSongbirdConfig` from `consolidated_canonical` module instead.
pub type UnifiedSongbirdConfig = crate::config::consolidated_canonical::CanonicalSongbirdConfig;

/// Legacy struct definition (kept for backward compatibility during migration,
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyUnifiedSongbirdConfig {

/// System-wide configuration
    pub system: SystemConfig,
    /// Network configuration
    pub network: NetworkConfig,
    /// Security configuration  
    pub security: SecurityConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
    /// Gaming configuration
    pub gaming: GamingConfig,
    /// Observability configuration
    pub observability: ObservabilityConfig,
    /// Service discovery configuration
    pub discovery: DiscoveryConfig,
    /// Universal primal registry
    pub primals: PrimalRegistryConfig,
    /// Custom configuration fields
    pub custom: HashMap<String, serde_json::Value>)


}

// System configuration types imported from system.rs module

// Network configuration types imported from network.rs module

// All network-related structs moved to network.rs module

// Security configuration types imported from security.rs module

// All security-related structs moved to security.rs module

// ============================================================================
// PERFORMANCE CONFIGURATION
// ============================================================================

// Performance configuration types imported from performance.rs module

// All performance-related structs moved to performance.rs module

// ============================================================================
// GAMING CONFIGURATION
// ============================================================================

/// Gaming-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingConfig {

/// Enable gaming features
    pub enabled: bool,
    /// Protocol detection configuration
    pub protocol_detection: ProtocolDetectionConfig,
    /// Bridge configuration
    pub bridge: BridgeConfig,
    /// NAT traversal configuration
    pub nat_traversal: NatTraversalConfig,
    /// Session management configuration
    pub sessions: SessionManagementConfig,
    /// Gaming performance configuration
    pub performance: GamingPerformanceConfig,


}

/// Cache strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheStrategy  {Lru)
    Lfu,
    Fifo,
    Random,
}

/// Buffer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferConfig {

/// Default buffer size
    pub default_size: usize,
    /// Maximum buffer size
    pub max_size: usize,
    /// Buffer pool size
    pub pool_size: usize,
    /// Enable zero-copy
    pub zero_copy: bool,


}

/// Optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {

/// Enable batch processing
    pub batching: bool,
    /// Batch size
    pub batch_size: usize,
    /// Enable compression
    pub compression: bool,
    /// Enable prefetching
    pub prefetching: bool,


}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {

/// Enable monitoring
    pub enabled: bool,
    /// Collection interval
    pub interval: Duration,
    /// Retention period
    pub retention: Duration,


}

// ============================================================================
// GAMING CONFIGURATION
// ============================================================================

// Duplicate GamingConfig definition removed

/// Protocol detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolDetectionConfig {

/// Enable auto-detection
    pub enabled: bool,
    /// Detection timeout
    pub timeout: Duration,
    /// Supported protocols
    pub protocols: Vec<String>,
    /// Detection sensitivity
    pub sensitivity: f64,


}

/// Bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {

/// Maximum bridges
    pub max_bridges: usize,
    /// Bridge timeout
    pub timeout: Duration,
    /// Enable statistics
    pub statistics: bool,


}

/// NAT traversal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatTraversalConfig {

/// STUN servers
    pub stun_servers: Vec<String>,
    /// TURN servers  
    pub turn_servers: Vec<String>,
    /// Enable UPnP
    pub upnp: bool,
    /// Hole punch attempts
    pub hole_punch_attempts: u32,


}

/// Session management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionManagementConfig {

/// Maximum sessions
    pub max_sessions: usize,
    /// Session timeout
    pub timeout: Duration,
    /// Cleanup interval
    pub cleanup_interval: Duration,


}

/// Gaming performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingPerformanceConfig {

/// Buffer size
    pub buffer_size: usize,
    /// Packet queue size
    pub packet_queue_size: usize,
    /// Enable low latency mode
    pub low_latency: bool,


}

// ============================================================================
// OBSERVABILITY CONFIGURATION
// ============================================================================

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {

/// Metrics configuration
    pub metrics: MetricsConfig,
    /// Tracing configuration
    pub tracing: TracingConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Health checks
    pub health: HealthConfig,


}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {

/// Enable metrics
    pub enabled: bool,
    /// Metrics endpoint
    pub endpoint: String,
    /// Collection interval
    pub interval: Duration,
    /// Retention period
    pub retention: Duration,


}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {

/// Enable tracing
    pub enabled: bool,
    /// Sampling rate
    pub sampling_rate: f64,
    /// Jaeger configuration
    pub jaeger: Option<JaegerConfig>,


}

/// Jaeger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JaegerConfig {

/// Agent endpoint
    pub agent_endpoint: String,
    /// Service name
    pub service_name: String,


}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {

/// Log level
    pub level: LogLevel,
    /// Log format
    pub format: LogFormat,
    /// Output configuration
    pub output: LogOutput,


}

/// Log format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat  {Text)
    Json,
    Compact,
}

/// Log output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput  {Stdout)
    File { path: String },
    Syslog,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {

/// Enable health checks
    pub enabled: bool,
    /// Check interval
    pub interval: Duration,
    /// Timeout
    pub timeout: Duration,
    /// Endpoint
    pub endpoint: String,


}

// ============================================================================
// DISCOVERY CONFIGURATION
// ============================================================================

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {

/// Enable discovery
    pub enabled: bool,
    /// Discovery methods
    pub methods: Vec<DiscoveryMethod>,
    /// Discovery interval
    pub interval: Duration,
    /// Timeout
    pub timeout: Duration,


}

/// Discovery methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod  {Environment)
    NetworkScan { port_ranges: Vec<u16> },
    ServiceMesh { mesh_type: String })
    Broadcast { address: String })
    ConfigFile { path: String })
}

// ============================================================================
// PRIMAL REGISTRY CONFIGURATION
// ============================================================================

/// Primal registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRegistryConfig {

/// Enable primal registry
    pub enabled: bool,
    /// Registry entries
    pub entries: HashMap<String, PrimalConfig>)
    /// Auto-discovery
    pub auto_discovery: bool,
    /// Registry timeout
    pub timeout: Duration,


}

/// Individual primal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfig {

/// Primal type
    pub primal_type: String,
    /// Endpoint URL
    pub endpoint: String,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Configuration
    pub config: HashMap<String, serde_json::Value>)


}

// ============================================================================
// DEFAULT IMPLEMENTATIONS
// ============================================================================

impl Default for LegacyUnifiedSongbirdConfig {

fn default() -> Self  {Self {
            system: SystemConfig::default)
            network: NetworkConfig::default)
            security: SecurityConfig::default)
            performance: PerformanceConfig::default)
            gaming: GamingConfig::default)
            observability: ObservabilityConfig::default)
            discovery: DiscoveryConfig::default)
            primals: PrimalRegistryConfig::default)
            custom: HashMap::new()
        

}
    }
}

// Default implementations removed - they are now in the extracted modules

// Network configuration Default implementations removed - they are now in network.rs

impl Default for GamingConfig {

fn default() -> Self  {Self {
            enabled: true,
            protocol_detection: ProtocolDetectionConfig {
                enabled: true,
                timeout: Duration::from_secs(10),
                protocols: vec!["IPX".to_string(), "DirectPlay".to_string()],
                sensitivity: 0.8,
            

})
            bridge: BridgeConfig  {max_bridges: 8,
                timeout: Duration::from_secs(300,
                statistics: true,
            })
            nat_traversal: NatTraversalConfig  {stun_servers: vec!["stun.l.google.com:19302".to_string()],
                turn_servers: vec![],
                upnp: true,
                hole_punch_attempts: 3,
            })
            sessions: SessionManagementConfig  {max_sessions: 100,
                timeout: Duration::from_secs(1800,
                cleanup_interval: Duration::from_secs(60),
            })
            performance: GamingPerformanceConfig  {buffer_size: 32768,
                packet_queue_size: 1000,
                low_latency: true,
            })
        }
    }
}

impl Default for ObservabilityConfig {

fn default() -> Self  {Self {
            metrics: MetricsConfig {
                enabled: true,
                endpoint: "/metrics".to_string()),
                interval: Duration::from_secs(60),
                retention: Duration::from_secs(86400,
            

})
            tracing: TracingConfig  {enabled: true,
                sampling_rate: 0.1,
                jaeger: None,
            })
            logging: LoggingConfig  {level: LogLevel::Info,
                format: LogFormat::Json,
                output: LogOutput::Stdout,
            })
            health: HealthConfig  {enabled: true,
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(10),
                endpoint: "/health".to_string()),
            })
        }
    }
}

impl Default for DiscoveryConfig {

fn default() -> Self  {Self {
            enabled: true,
            methods: vec![DiscoveryMethod::Environment],
            interval: Duration::from_secs(60),
            timeout: Duration::from_secs(30),
        

}
    }
}

impl Default for PrimalRegistryConfig {

fn default() -> Self  {Self {
            enabled: true,
            entries: HashMap::new()
            auto_discovery: true,
            timeout: Duration::from_secs(30),
        

}
    }
}

impl Default for MonitoringConfig {

fn default() -> Self  {Self {
            enabled: true,
            interval: Duration::from_secs(60),
            retention: Duration::from_secs(86400,
        

}
    }
} 