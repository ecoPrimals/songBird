/// Universal Security Types Module
///
/// Contains all data structures, enums, and configuration types for the universal security system
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;
use uuid::Uuid;

/// Device Security Policy - Applied to every connected device
#[derive(Debug, Clone)]
pub struct DeviceSecurityPolicy {
    /// Unique device identifier
    pub device_id: String,
    /// Human-readable device name
    pub device_name: String,
    /// Security level for this device
    pub security_level: SecurityLevel,
    /// Whether encryption is required for this device
    pub encryption_required: bool,
    /// Whether family-safe mode is enabled
    pub family_safe_mode: bool,
    /// When this device was first trusted
    pub trusted_since: DateTime<Utc>,
    /// Auto-enabled security features for this device
    pub auto_security_features: Vec<AutoSecurityFeature>,
}

/// Security levels for devices and connections
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityLevel {
    /// Minimal security - basic protection only
    Minimal,
    /// Standard security - recommended for most users
    Standard,
    /// High security - enhanced protection
    High,
    /// Maximum security - fortress-grade protection
    Maximum,
}

/// Automatically enabled security features
#[derive(Debug, Clone, PartialEq)]
pub enum AutoSecurityFeature {
    /// Real-time scammer detection
    ScammerDetection,
    /// Automatic encryption of sensitive data
    AutoEncryption,
    /// Family-safe content filtering
    FamilySafeFiltering,
    /// Malware protection
    MalwareProtection,
    /// Privacy enhancement
    PrivacyEnhancement,
    /// Connection monitoring
    ConnectionMonitoring,
}

/// Friend trust levels for social gaming
#[derive(Debug, Clone, PartialEq)]
pub enum FriendTrustLevel {
    /// Unknown person - maximum protection
    Unknown,
    /// Acquaintance - standard protection with monitoring
    Acquaintance {
        /// How long we've known this person
        known_duration: std::time::Duration,
        /// Number of positive interactions
        positive_interactions: u32,
    },
    /// Trusted friend - reduced security friction
    TrustedFriend {
        /// When friendship was established
        established: DateTime<Utc>,
        /// Mutual friends count
        mutual_friends: u32,
    },
    /// Family member - highest trust level
    Family {
        /// Family relationship type
        relationship: String,
        /// Verified family member
        verified: bool,
    },
}

/// Scammer protection configuration
#[derive(Debug, Clone)]
pub struct ScammerProtectionConfig {
    /// Enable real-time scammer detection
    pub enabled: bool,
    /// Sensitivity level (0.0 = permissive, 1.0 = strict)
    pub sensitivity: f32,
    /// Automatically block known scammers
    pub auto_block: bool,
    /// Warn users about suspicious activity
    pub warn_users: bool,
    /// Share threat intelligence (anonymized)
    pub share_threat_intel: bool,
    /// Block suspicious file transfers
    pub block_suspicious_files: bool,
    /// Monitor for social engineering attempts
    pub social_engineering_protection: bool,
    /// Protect against fake friend requests
    pub fake_friend_protection: bool,
    /// Age-appropriate protection levels
    pub age_appropriate_protection: AgeProtectionLevel,
}

/// Age-appropriate protection levels
#[derive(Debug, Clone, PartialEq)]
pub enum AgeProtectionLevel {
    /// Child protection (under 13) - maximum safety
    Child,
    /// Teen protection (13-17) - balanced safety and freedom
    Teen,
    /// Adult protection (18+) - user-controlled safety
    Adult,
}

/// Session key for lightweight encryption
#[derive(Debug, Clone)]
pub struct SessionKey {
    /// Key identifier
    pub id: Uuid,
    /// Key material (encrypted)
    pub key_material: Vec<u8>,
    /// When this key was created
    pub created_at: SystemTime,
    /// When this key expires
    pub expires_at: SystemTime,
    /// Gaming session this key belongs to
    pub session_id: Option<String>,
    /// Tunnel type this key is for
    pub tunnel_type: TunnelType,
}

/// Gaming tunnel metadata
#[derive(Debug, Clone)]
pub struct GamingTunnelMetadata {
    /// Tunnel identifier
    pub tunnel_id: String,
    /// Game being played
    pub game_name: String,
    /// Players in this tunnel
    pub players: Vec<String>,
    /// Tunnel creation time
    pub created_at: SystemTime,
    /// Gaming priority level
    pub priority: GamingPriority,
    /// Security level for this tunnel
    pub security_level: SecurityLevel,
}

/// Gaming priority levels
#[derive(Debug, Clone, PartialEq)]
pub enum GamingPriority {
    /// Low priority - background gaming
    Low,
    /// Normal priority - casual gaming
    Normal,
    /// High priority - competitive gaming
    High,
    /// Critical priority - tournament gaming
    Critical,
}

/// Tunnel types for different use cases
#[derive(Debug, Clone, PartialEq)]
pub enum TunnelType {
    /// Gaming tunnel for multiplayer games
    Gaming {
        /// Game protocol being used
        protocol: String,
        /// Whether this is a LAN game
        is_lan: bool,
    },
    /// File sharing tunnel
    FileSharing {
        /// Whether sharing is bidirectional
        bidirectional: bool,
        /// Maximum file size allowed
        max_file_size: u64,
    },
    /// Voice chat tunnel
    VoiceChat {
        /// Audio quality level
        quality: String,
        /// Whether video is enabled
        video_enabled: bool,
    },
    /// General purpose tunnel
    General,
}

/// Crypto capabilities for security providers
#[derive(Debug, Clone, PartialEq)]
pub enum CryptoCapability {
    /// Symmetric encryption/decryption
    SymmetricCrypto,
    /// Asymmetric encryption/decryption
    AsymmetricCrypto,
    /// Digital signatures
    DigitalSignatures,
    /// Key exchange protocols
    KeyExchange,
    /// Hashing and MAC
    Hashing,
}

/// Crypto strength levels
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum CryptoStrength {
    /// Basic strength - adequate for most use cases
    Basic,
    /// Standard strength - recommended level
    Standard,
    /// High strength - enhanced protection
    High,
    /// Military strength - maximum protection
    Military,
}

/// Gaming tunnel status information
#[derive(Debug, Clone)]
pub struct GamingTunnelStatus {
    /// Tunnel identifier
    pub tunnel_id: String,
    /// Current status
    pub status: String,
    /// Number of active connections
    pub active_connections: u32,
    /// Data transferred (bytes)
    pub bytes_transferred: u64,
    /// Tunnel uptime
    pub uptime: std::time::Duration,
    /// Last activity timestamp
    pub last_activity: SystemTime,
    /// Security status
    pub security_status: ConnectionSecurityStatus,
}

/// Connection security status
#[derive(Debug, Clone)]
pub struct ConnectionSecurityStatus {
    /// Whether connection is encrypted
    pub encrypted: bool,
    /// Encryption strength if encrypted
    pub encryption_strength: Option<CryptoStrength>,
    /// Whether connection is authenticated
    pub authenticated: bool,
    /// Trust level of remote party
    pub trust_level: FriendTrustLevel,
    /// Any security warnings
    pub warnings: Vec<String>,
}

/// Connection activity information
#[derive(Debug, Clone)]
pub struct ConnectionActivity {
    /// Remote address (anonymized for privacy)
    pub remote_address_hash: String,
    /// Activity type
    pub activity_type: String,
    /// When activity occurred
    pub timestamp: SystemTime,
    /// Data size involved
    pub data_size: u64,
}

/// Scammer protection result
#[derive(Debug, Clone, PartialEq)]
pub enum ScammerProtectionResult {
    /// Connection is safe
    Safe,
    /// Connection is suspicious - warn user
    Suspicious {
        /// Reason for suspicion
        reason: String,
        /// Confidence level (0.0 - 1.0)
        confidence: f32,
    },
    /// Connection is dangerous - block immediately
    Dangerous {
        /// Specific threat detected
        threat_type: String,
        /// Evidence of threat
        evidence: Vec<String>,
    },
}

/// Coordination crypto for tunnel setup
#[derive(Debug, Clone)]
pub struct CoordinationCrypto {
    /// Lightweight key exchange
    pub key_exchange: HashMap<String, Vec<u8>>,
    /// Session establishment data
    pub session_data: HashMap<String, String>,
    /// Tunnel configuration
    pub tunnel_config: HashMap<String, serde_json::Value>,
}

/// Security provider encryption context
#[derive(Debug, Clone)]
pub struct SecurityProviderEncryptionContext {
    /// Provider identifier
    pub provider_id: String,
    /// Available crypto capabilities
    pub capabilities: Vec<CryptoCapability>,
    /// Maximum strength supported
    pub max_strength: CryptoStrength,
}

/// Reinforced tunnel configuration
#[derive(Debug, Clone)]
pub struct ReinforcedTunnel {
    /// Base tunnel configuration
    pub base_config: GamingTunnelMetadata,
    /// Additional security layers
    pub security_layers: Vec<String>,
    /// Redundancy level
    pub redundancy: u8,
}
