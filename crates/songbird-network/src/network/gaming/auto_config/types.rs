//! Auto Configuration Types
//!
//! Basic types and enums used throughout the auto-configuration system.

use serde::{Deserialize, Serialize};

/// Setup state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupState {
    pub is_initialized: bool,
    pub setup_timestamp: u64,
    pub configuration_method: SetupMethod,
    pub trust_level: TrustLevel,
    pub last_security_scan: Option<u64>,
    pub auto_updates_enabled: bool,
}

/// Trusted device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    pub device_id: String,
    pub device_name: String,
    pub trust_level: TrustLevel,
    pub last_seen: u64,
    pub family_member: bool,
    pub auto_approved: bool,
    pub capabilities: Vec<String>,
}

/// Trust level for devices and connections
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TrustLevel {
    /// Family member device
    Family,
    /// Trusted friend device
    Friend,
    /// Known device
    Known,
    /// Untrusted device (blocked)
    #[default]
    Untrusted,
}

/// Setup method used for auto-configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SetupMethod {
    /// One-click setup
    OneTouch,
    /// Manual configuration
    #[default]
    Manual,
}

/// Security level for auto-configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SecurityLevel {
    /// Maximum security (family safe)
    Maximum,
    /// High security (recommended)
    #[default]
    High,
    /// Medium security
    Medium,
    /// Low security (gaming optimized)
    Low,
}

/// Auto-configuration trust level for primals
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AutoConfigTrustLevel {
    /// Complete trust (family primals)
    Family,
    /// High trust (verified primals)
    High,
    /// Standard trust (known primals)
    #[default]
    Standard,
    /// Low trust (unverified)
    Low,
}

/// QoS settings for gaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QosSettings {
    pub latency_priority: bool,
    pub bandwidth_limit_mbps: Option<f64>,
    pub packet_priority: u8,
    pub traffic_shaping_enabled: bool,
}

/// System capabilities detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCapabilities {
    pub has_admin_privileges: bool,
    pub has_network_access: bool,
    pub has_firewall_control: bool,
    pub has_port_forwarding: bool,
}

/// One-touch configuration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneTouchConfig {
    pub success: bool,
    pub message: String,
    pub configuration: Option<serde_json::Value>,
    pub next_steps: Vec<String>,
    pub warnings: Vec<String>,
}

// Default implementations
impl Default for SetupState {
    fn default() -> Self {
        Self {
            is_initialized: false,
            setup_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            configuration_method: SetupMethod::default(),
            trust_level: TrustLevel::default(),
            last_security_scan: None,
            auto_updates_enabled: true,
        }
    }
}

impl Default for OneTouchConfig {
    fn default() -> Self {
        Self {
            success: false,
            message: "Configuration not completed".to_string(),
            configuration: None,
            next_steps: vec![],
            warnings: vec![],
        }
    }
}
