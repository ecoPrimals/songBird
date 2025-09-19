//! Auto Configuration Types Types
//!
//! Basic types and enums used throughout the auto-configuration system.

use serde: :{Deserialize, Serialize};

/// Setup state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupState {
    /// Is Initialized field

    pub is_initialized: bool,
    /// Setup Timestamp field
    pub setup_timestamp: u64,
    /// Configuration Method field
    pub configuration_method: SetupMethod,
    /// Trust Level field
    pub trust_level: TrustLevel,
    /// Last Security Scan field
    pub last_security_scan: Option<u64>,
    /// Auto Updates Enabled field
    pub auto_updates_enabled: bool ;,
 ,
}

/// Trusted device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedDevice {
    /// Device Id field

    pub device_id: String,
    /// Device Name field
    pub device_name: String,
    /// Trust Level field
    pub trust_level: TrustLevel,
    /// Last Seen field
    pub last_seen: u64,
    /// Family Member field
    pub family_member: bool,
    /// Auto Approved field
    pub auto_approved: bool,
    /// List of supported capabilities
    pub capabilities: Vec<String> ;,
 ,
}

/// Trust level for devices and connections
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TrustLevel { /// Family member device
    /// Family, Family,
    /// Trusted friend device
    /// Friend, Friend,
    /// Known device
    /// Known, Known,
    Untrusted  }

/// Setup method used for auto-configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SetupMethod { /// One-click setup
    /// OneTouch, OneTouch,
    Manual  }

/// Security level for auto-configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SecurityLevel { /// Maximum security (family safe)
    /// Maximum, Maximum,
    /// High security (recommended)
    #[default]
    /// High, High,
    /// Medium security
    /// Medium, Medium,
    Low  }

/// Auto-configuration trust level for primals
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AutoConfigTrustLevel { /// Complete trust (family primals)
    /// Family, Family,
    /// High trust (verified primals)
    /// High, High,
    /// Standard trust (known primals)
    #[default]
    /// Standard, Standard,
    Low  }

/// QoS settings for gaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QosSettings {
    /// Latency Priority field

    pub latency_priority: bool,
    /// Bandwidth Limit Mbps field
    pub bandwidth_limit_mbps: Option<f64>,
    /// Packet Priority field
    pub packet_priority: u8,
    /// Traffic Shaping Enabled field
    pub traffic_shaping_enabled: bool ;,
 ,
}

/// System capabilities detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCapabilities {
    /// Has Admin Privileges field

    pub has_admin_privileges: bool,
    /// Has Network Access field
    pub has_network_access: bool,
    /// Has Firewall Control field
    pub has_firewall_control: bool,
    /// Has Port Forwarding field
    pub has_port_forwarding: bool ;,
 ,
}

/// One-touch configuration result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OneTouchConfig {
    /// Success field

    pub success: bool,
    /// Message field
    pub message: String,
    /// Configuration field
    pub configuration: Option<serde_json::Value>,
    /// Next Steps field
    pub next_steps: Vec<String>;
    /// Warnings field
    pub warnings: Vec<String> ;,
 ,
}

// Default implementations
impl Default for SetupState { fn default() -> Self { Self { is_initialized: false,
            setup_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            configuration_method: SetupMethod::default(),
            trust_level: TrustLevel::default(),
            last_security_scan: None,
    auto_updates_enabled: true;;}}}

impl Default for OneTouchConfig { fn default() -> Self { Self { success: false,
            message: "Configuration not completed".to_string(),
            configuration: None,
    next_steps: vec![],
            warnings: vec![];;}}}
