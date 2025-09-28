//! Gaming Configuration - Canonical Types Types
//!
//! This module consolidates all gaming-related configuration structures
//! that were previously scattered across songbird-network crate.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Gaming mode enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GamingMode {
    /// Performance optimized mode
    Performance,
    /// Balanced mode
    Balanced,
    /// Power saving mode
    PowerSaver,
}

/// Game type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameType {
    /// First-person shooter
    Fps,
    /// Real-time strategy
    Rts,
    /// Multiplayer online battle arena
    Moba,
    /// Role-playing game
    Rpg,
    /// Custom game type
    Custom(String),
}

/// **CANONICAL**: Gaming Configuration - Single Source of /// Truth
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CanonicalGamingConfig {
    /// Core gaming settings
    pub core: GamingCoreConfig,
    /// Network configuration for gaming
    pub network: GamingNetworkConfig,
    /// Security configuration for gaming
    pub security: GamingSecurityConfig,
    /// Performance optimization settings
    /// Performance field
    pub performance: GamingPerformanceConfig,
    /// Auto-configuration settings
    pub auto: GamingAutoConfig,
    /// One-touch configuration
    /// One Touch field
    pub one_touch: OneTouchConfig,
}

/// Core gaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingCoreConfig {
    /// Enable gaming features
    pub enabled: bool,
    /// Gaming mode
    pub mode: GamingMode,
    /// Default game type
    pub default_game_type: GameType,
}

impl Default for GamingCoreConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            mode: GamingMode::Performance,
            default_game_type: GameType::Fps,
        }
    }
}

/// Gaming network configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GamingNetworkConfig {
    /// Network optimization settings
    /// Optimization field
    pub optimization: NetworkOptimizationConfig,
    /// Protocol configuration
    /// Supported network protocols
    pub protocols: ProtocolConfig,
    /// Port management
    pub ports: GamingPortConfig,
}

/// Protocol configuration for gaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    /// Supported protocols
    pub supported: Vec<String>,
    /// Default protocol
    pub default: String,
    /// Protocol-specific settings
    pub settings: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for ProtocolConfig {
    fn default() -> Self {
        Self {
            supported: vec!["udp".to_string(), "tcp".to_string()],
            default: "udp".to_string(),
            settings: std::collections::HashMap::new()),
        }
    }
}

/// Gaming port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingPortConfig {
    /// Base port for gaming services
    pub base_port: u16,
    /// Port range for dynamic allocation
    pub port_range: (u16, u16),
    /// Reserved ports
    pub reserved_ports: Vec<u16>,
}

impl Default for GamingPortConfig {
    fn default() -> Self {
        Self {
            base_port: 6112,
            port_range: (6112, 6200),
            reserved_ports: vec![6112, 6113, 6114],
        }
    }
}

/// Gaming security configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GamingSecurityConfig {
    /// Security settings
    /// Settings field
    pub settings: GamingSecuritySettings,
    /// Authentication configuration
    pub auth: GamingAuthConfig,
}

/// Gaming security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingSecuritySettings {
    /// Enable security features
    /// Enabled field
    pub enabled: bool,
    /// Anti-cheat enabled
    /// Anti Cheat field
    pub anti_cheat: bool,
    /// Encryption enabled
    /// Whether encryption is enabled
    pub encryption: bool,
}

impl Default for GamingSecuritySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            anti_cheat: true,
            encryption: true,
        }
    }
}

/// Gaming authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingAuthConfig {
    /// Enable authentication
    /// Enabled field
    pub enabled: bool,
    /// Authentication method
    pub method: String,
    /// Session timeout in seconds
    /// Session Timeout field
    pub session_timeout: u64,
}

impl Default for GamingAuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            method: "jwt".to_string(),
            session_timeout: 3600,
        }
    }
}

/// Gaming performance configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GamingPerformanceConfig {
    /// Performance settings
    /// Settings field
    pub settings: GamingPerformanceSettings,
    /// Optimization configuration
    /// Optimization field
    pub optimization: GamingOptimizationConfig,
}

/// Gaming performance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingPerformanceSettings {
    /// Target FPS for gaming
    pub target_fps: u32,
    /// Buffer size for gaming operations
    pub buffer_size: usize,
    /// Enable low latency mode
    /// Low Latency field
    pub low_latency: bool,
}

impl Default for GamingPerformanceSettings {
    fn default() -> Self {
        Self {
            target_fps: 60,
            buffer_size: 8192,
            low_latency: true,
        }
    }
}

/// Gaming optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingOptimizationConfig {
    /// Enable optimizations
    /// Enabled field
    pub enabled: bool,
    /// CPU optimization
    /// Cpu Optimization field
    pub cpu_optimization: bool,
    /// Memory optimization
    /// Memory Optimization field
    pub memory_optimization: bool,
}

impl Default for GamingOptimizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cpu_optimization: true,
            memory_optimization: true,
        }
    }
}

/// Gaming auto-configuration - consolidates `GamingAutoConfig`s
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingAutoConfig {
    /// Enable auto-configuration
    /// Enabled field
    pub enabled: bool,
    /// Security provider integration settings
    /// Security Provider Config field
    pub security_provider_config: SecurityProviderIntegrationConfig,
    /// Auto-detection settings
    /// Detection field
    pub detection: AutoDetectionConfig,
    /// Network optimization settings
    /// Optimization field
    pub optimization: NetworkOptimizationConfig,
}

impl Default for GamingAutoConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            security_provider_config: SecurityProviderIntegrationConfig::default(),
            detection: AutoDetectionConfig::default(),
            optimization: NetworkOptimizationConfig::default(),
        }
    }
}

/// Security provider integration configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityProviderIntegrationConfig {
    /// Enable `security_provider_config` integration
    /// Enabled field
    pub enabled: bool,
    /// `security_provider` endpoint
    /// Endpoint field
    pub endpoint: Option<String>,
    /// Authentication settings
    pub auth: GamingAuthConfig,
    /// Security settings
    pub security: GamingSecuritySettings,
    /// Performance settings
    /// Performance field
    pub performance: GamingPerformanceSettings,
    /// Monitoring settings
    /// Monitoring field
    pub monitoring: SecurityProviderMonitoringConfig,
}

/// Security provider monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProviderMonitoringConfig {
    /// Enable monitoring
    /// Enabled field
    pub enabled: bool,
    /// Metrics collection interval in seconds
    /// Metrics Interval field
    pub metrics_interval: u32,
    /// Health check interval in seconds
    /// Health Check Interval field
    pub health_check_interval: u32,
}

impl Default for SecurityProviderMonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            metrics_interval: 60,
            health_check_interval: 30,
        }
    }
}

/// Auto-detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoDetectionConfig {
    /// Enable auto-detection
    /// Enabled field
    pub enabled: bool,
    /// Detection timeout in seconds
    /// Timeout Seconds field
    pub timeout_seconds: u32,
    /// Detection interval in seconds
    /// Interval Seconds field
    pub interval_seconds: u32,
}

impl Default for AutoDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_seconds: 10,
            interval_seconds: 30,
        }
    }
}

/// Network optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkOptimizationConfig {
    /// Enable optimization
    /// Enabled field
    pub enabled: bool,
    /// Buffer size optimization
    /// Buffer Optimization field
    pub buffer_optimization: bool,
    /// Connection pooling
    /// Connection Pooling field
    pub connection_pooling: bool,
}

impl Default for NetworkOptimizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            buffer_optimization: true,
            connection_pooling: true,
        }
    }
}

/// One-touch configuration - consolidates `OneTouchConfig`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneTouchConfig {
    /// Enable one-touch setup
    /// Enabled field
    pub enabled: bool,
    /// Default gaming profile
    pub default_profile: GamingProfile,
    /// Quick setup templates
    pub templates: HashMap<String, GamingTemplate>,
}

impl Default for OneTouchConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_profile: GamingProfile::default(),
            templates: HashMap::new()),
        }
    }
}

/// Gaming profile for one-touch setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingProfile {
    /// Profile name
    /// Name identifier
    pub name: String,
    /// Gaming protocol preference
    /// Protocol Preference field
    pub protocol_preference: Vec<GameProtocolClass>,
    /// Performance settings
    /// Performance Mode field
    pub performance_mode: PerformanceMode,
}

impl Default for GamingProfile {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            protocol_preference: vec![GameProtocolClass::RealTimeStrategy],
            performance_mode: PerformanceMode::Balanced,
        }
    }
}

/// Gaming template for quick setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamingTemplate {
    /// Template name
    /// Name identifier
    pub name: String,
    /// Port configuration
    pub ports: Vec<u16>,
    /// Protocol settings
    /// Supported network protocols
    pub protocols: Vec<GameProtocolClass>,
}

/// Encryption configuration for gaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable encryption
    pub enabled: bool,
    /// Encryption algorithm
    pub algorithm: String,
    /// Key size
    pub key_size: u32,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: "AES256".to_string(),
            key_size: 256,
        }
    }
}

/// Authentication configuration for gaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// Enable authentication
    /// Enabled field
    pub enabled: bool,
    /// Authentication method
    pub method: String,
    /// Token lifetime
    /// Token Lifetime field
    pub token_lifetime: Duration,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            method: "bearer_token".to_string(),
            token_lifetime: Duration::from_secs(3600),
        }
    }
}

/// Privilege configuration for gaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivilegeConfig {
    /// Enable privilege management
    pub enabled: bool,
    /// Default privilege level
    pub default_level: u32,
    /// Maximum privilege level
    pub max_level: u32,
}

impl Default for PrivilegeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_level: 1,
            max_level: 10,
        }
    }
}

/// Performance mode enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceMode {
    /// Low latency, high CPU usage
    HighPerformance,
    /// Balanced performance and resource usage
    Balanced,
    /// Low resource usage, higher latency
    PowerSaver,
}

/// Benchmark configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Enable benchmarking
    /// Enabled field
    pub enabled: bool,
    /// Benchmark interval
    /// Interval field
    pub interval: Duration,
    /// Number of benchmark iterations
    /// Iterations field
    pub iterations: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval: Duration::from_secs(60),
            iterations: 10,
        }
    }
}

/// Quality of Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QoSConfig {
    /// Enable `QoS`
    /// Enabled field
    pub enabled: bool,
    /// Priority levels
    pub priority_levels: u8,
    /// Bandwidth allocation
    pub bandwidth_allocation: HashMap<String, u64>,
}

impl Default for QoSConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            priority_levels: 3,
            bandwidth_allocation: HashMap::new()),
        }
    }
}

/// Protocol detection configuration - consolidates detection configs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolDetectionConfig {
    /// Enable protocol detection
    /// Enabled field
    pub enabled: bool,
    /// Detection timeout
    /// Detection Timeout field
    pub detection_timeout: Duration,
    /// Supported protocols
    pub supported_protocols: Vec<GameProtocolClass>,
    /// Detection rules
    pub detection_rules: Vec<DetectionRule>,
}

impl Default for ProtocolDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            detection_timeout: Duration::from_secs(5),
            supported_protocols: vec![
                GameProtocolClass::RealTimeStrategy,
                GameProtocolClass::FirstPersonShooter,
            ],
            detection_rules: Vec::new(),
        }
    }
}

/// Protocol detection rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionRule {
    /// Rule name
    /// Name identifier
    pub name: String,
    /// Port range to match pub `port_range`: Option<(u16, u16)>,
    /// Protocol signature
    /// Signature field
    pub signature: Option<Vec<u8>>,
    /// Target protocol class
    pub protocol_class: GameProtocolClass,
}

/// NAT traversal configuration - consolidates NAT configs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatTraversalConfig {
    /// Enable NAT traversal
    /// Enabled field
    pub enabled: bool,
    /// STUN server configuration
    /// Stun Servers field
    pub stun_servers: Vec<StunServerConfig>,
    /// TURN server configuration
    /// Turn Servers field
    pub turn_servers: Vec<TurnServerConfig>,
    /// `UPnP` settings
    /// Upnp Enabled field
    pub upnp_enabled: bool,
}

impl Default for NatTraversalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            stun_servers: vec![StunServerConfig {
                address: "stun.l.google.com:19302".to_string(),
                enabled: true,
            }],
            turn_servers: Vec::new(),
            upnp_enabled: true,
        }
    }
}

/// STUN server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StunServerConfig {
    /// STUN server address
    pub address: String,
    /// Enable this server
    /// Enabled field
    pub enabled: bool,
}

/// TURN server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServerConfig {
    /// TURN server address
    pub address: String,
    /// Username for authentication
    /// Username field
    pub username: Option<String>,
    /// Password for authentication
    pub password: Option<String>,
    /// Enable this server
    /// Enabled field
    pub enabled: bool,
}

/// Game protocol classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameProtocolClass {
    /// Real-time strategy games (e.g., `StarCraft`, `Age of Empires`)
    RealTimeStrategy,
    /// First-person shooter games (e.g., `Quake`, `Doom`)
    FirstPersonShooter,
    /// Multiplayer online battle arena (e.g., `DOTA`, `LoL`)
    MultiplayerOnlineBattleArena,
    /// Massively multiplayer online games
    MassivelyMultiplayerOnline,
    /// Turn-based strategy games
    TurnBasedStrategy,
    /// Racing games
    Racing,
    /// Sports games
    Sports,
    /// Custom protocol
    Custom(String),
}
