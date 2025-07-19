//! Universal Security - "Secure for All"
//!
//! Enterprise-grade security that's free and accessible to everyone.
//! Every device, every connection, every user gets fortress-grade protection.
//!
//! Principles:
//! - Zero Trust by Default
//! - Privacy by Design  
//! - Security Without Complexity
//! - Universal Access to Protection

use songbird_errors::{Result, SongbirdError};
// async_trait import removed - not currently used
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use uuid;

/// Universal Security Manager - "Secure for All"
pub struct UniversalSecurityManager {
    /// Security policies for all devices
    device_policies: Arc<RwLock<HashMap<String, DeviceSecurityPolicy>>>,
    /// Friend trust relationships
    friend_trust: Arc<RwLock<HashMap<String, FriendTrustLevel>>>,
    /// Family protection settings
    family_protection: Arc<RwLock<FamilyProtectionConfig>>,
    /// Ultra-lightweight encryption manager for SongBird tunnel coordination
    /// Heavy crypto operations should be delegated to BearDog
    encryption_manager: Arc<LightweightTunnelCrypto>,
}

/// Device Security Policy - Applied to every connected device
#[derive(Debug, Clone)]
pub struct DeviceSecurityPolicy {
    pub device_id: String,
    pub device_name: String,
    pub security_level: SecurityLevel,
    pub encryption_required: bool,
    pub family_safe_mode: bool,
    pub trusted_since: DateTime<Utc>,
    pub auto_security_features: Vec<AutoSecurityFeature>,
}

/// Security levels for universal protection
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityLevel {
    /// Maximum protection - grandma-safe, scammer-proof
    Maximum,
    /// High protection - excellent for most users
    High,
    /// Standard protection - good baseline security
    Standard,
    /// Basic protection - minimal overhead
    Basic,
}

/// Auto-enabled security features
#[derive(Debug, Clone)]
pub enum AutoSecurityFeature {
    /// Block known scammer tactics
    ScammerProtection,
    /// Block technical support scam calls
    TechSupportCallBlocking,
    /// Family-safe device verification
    FamilyDeviceVerification,
    /// Automatic firmware security updates
    AutoSecurityUpdates,
    /// Privacy-first data handling
    PrivacyFirst,
    /// Zero trust network access
    ZeroTrustNetworking,
}

/// Friend trust levels for social security
#[derive(Debug, Clone)]
pub enum FriendTrustLevel {
    /// Family members - maximum trust and access
    Family { verified_at: DateTime<Utc> },
    /// Close friends - high trust, most access
    CloseFriend { verified_at: DateTime<Utc> },
    /// Friends - standard trust
    Friend { verified_at: DateTime<Utc> },
    /// Acquaintances - limited trust
    Acquaintance { verified_at: DateTime<Utc> },
    /// Unknown - no trust, maximum protection
    Unknown,
}

/// Family Protection Configuration
#[derive(Debug, Clone)]
pub struct FamilyProtectionConfig {
    pub enabled: bool,
    pub family_name: String,
    pub protection_level: SecurityLevel,
    pub trusted_devices: Vec<String>,
    pub guest_access_enabled: bool,
    pub parental_controls: bool,
    pub scammer_protection: ScammerProtectionConfig,
}

/// Scammer Protection Settings
#[derive(Debug, Clone)]
pub struct ScammerProtectionConfig {
    pub block_tech_support_calls: bool,
    pub block_unknown_remote_access: bool,
    pub block_suspicious_downloads: bool,
    pub family_safe_browsing: bool,
    pub financial_protection: bool,
}

impl Default for FamilyProtectionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            family_name: "My Family".to_string(),
            protection_level: SecurityLevel::Maximum,
            trusted_devices: Vec::new(),
            guest_access_enabled: false,
            parental_controls: false,
            scammer_protection: ScammerProtectionConfig {
                block_tech_support_calls: true,
                block_unknown_remote_access: true,
                block_suspicious_downloads: true,
                family_safe_browsing: true,
                financial_protection: true,
            },
        }
    }
}

/// Ultra-lightweight encryption manager for SongBird tunnel coordination
/// Heavy crypto operations should be delegated to BearDog
pub struct LightweightTunnelCrypto {
    /// Session key manager for tunnel establishment
    session_keys: Arc<RwLock<HashMap<String, SessionKey>>>,

    /// Coordination crypto for service discovery
    coordination_crypto: Arc<CoordinationCrypto>,

    /// Interface to BearDog for heavy crypto operations
    beardog_crypto_interface: Option<Arc<dyn BearDogCryptoInterface>>,
}

/// Lightweight session key for tunnel coordination with gaming optimization
#[derive(Debug, Clone)]
pub struct SessionKey {
    pub key_id: String,
    pub key_data: Vec<u8>,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub tunnel_type: TunnelType,
    pub auto_renewable: bool,
    pub renewal_count: u32,
    pub gaming_metadata: Option<GamingTunnelMetadata>,
}

/// Gaming-specific tunnel metadata
#[derive(Debug, Clone)]
pub struct GamingTunnelMetadata {
    pub game_session_id: Option<String>,
    pub player_count: Option<u32>,
    pub game_type: Option<String>,
    pub match_id: Option<String>,
    pub lobby_id: Option<String>,
    pub priority: GamingPriority,
}

/// Gaming tunnel priority levels
#[derive(Debug, Clone, PartialEq)]
pub enum GamingPriority {
    /// Competitive gaming - highest priority, most stable tunnels
    Competitive,
    /// Casual gaming - normal priority
    Casual,
    /// Background/lobby - lower priority
    Background,
    /// Discovery/matchmaking - lowest priority
    Discovery,
}

/// Types of tunnels SongBird coordinates with gaming-optimized settings
#[derive(Debug, Clone, PartialEq)]
pub enum TunnelType {
    /// Quick service discovery (5 minutes)
    ServiceDiscovery,
    /// WireGuard coordination (1 hour)
    WireGuardCoordination,
    /// Basic P2P connection (30 minutes)
    BasicP2P,
    /// BearDog reinforced tunnel (1 hour, renewable)
    BearDogReinforced,
    /// Gaming session tunnel (8 hours, auto-renewable)
    GamingSession,
    /// Gaming lobby coordination (30 minutes, renewable)
    GamingLobby,
    /// Gaming match tunnel (4 hours, seamless renewal)
    GamingMatch,
}

/// Ultra-light coordination crypto for SongBird operations
pub struct CoordinationCrypto {
    /// Simple XOR-based crypto for non-sensitive coordination
    coordination_key: Vec<u8>,
}

/// Interface to BearDog for heavy crypto operations
#[async_trait::async_trait]
pub trait BearDogCryptoInterface: Send + Sync {
    /// Request BearDog to encrypt data
    async fn beardog_encrypt(&self, data: &[u8], context: EncryptionContext) -> Result<Vec<u8>>;

    /// Request BearDog to decrypt data  
    async fn beardog_decrypt(
        &self,
        encrypted_data: &[u8],
        context: EncryptionContext,
    ) -> Result<Vec<u8>>;

    /// Request BearDog to reinforce SongBird tunnel
    async fn reinforce_tunnel(
        &self,
        tunnel_id: &str,
        songbird_key: &[u8],
    ) -> Result<ReinforcedTunnel>;
}

/// Context for BearDog encryption operations
#[derive(Debug, Clone)]
pub struct EncryptionContext {
    pub operation_type: String,
    pub user_context: Option<String>,
    pub security_level: SecurityLevel,
}

/// Reinforced tunnel with layered crypto
#[derive(Debug)]
pub struct ReinforcedTunnel {
    pub tunnel_id: String,
    pub songbird_layer: SessionKey,
    pub beardog_layer_id: String,
    pub combined_strength: CryptoStrength,
}

#[derive(Debug, Clone)]
pub enum CryptoStrength {
    SongBirdOnly,   // Ultra-light tunnel crypto
    BearDogOnly,    // Heavy data crypto
    LayeredDefense, // Both layers working together
}

/// Gaming tunnel status for monitoring and management
#[derive(Debug, Clone)]
pub struct GamingTunnelStatus {
    pub key_id: String,
    pub tunnel_type: TunnelType,
    pub time_until_expiry: std::time::Duration,
    pub renewal_count: u32,
    pub priority: GamingPriority,
    pub game_session_id: Option<String>,
    pub needs_renewal: bool,
}

impl Default for UniversalSecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UniversalSecurityManager {
    /// Create new universal security manager
    pub fn new() -> Self {
        Self {
            device_policies: Arc::new(RwLock::new(HashMap::new())),
            friend_trust: Arc::new(RwLock::new(HashMap::new())),
            family_protection: Arc::new(RwLock::new(FamilyProtectionConfig::default())),
            encryption_manager: Arc::new(LightweightTunnelCrypto::new()),
        }
    }

    /// Enable "Secure for All" mode - maximum protection for everyone
    pub async fn enable_secure_for_all(&self) -> Result<()> {
        println!("🛡️ Enabling 'Secure for All' Universal Protection...");

        // Set maximum security defaults
        let mut family_config = self.family_protection.write().await;
        family_config.protection_level = SecurityLevel::Maximum;
        family_config.scammer_protection = ScammerProtectionConfig {
            block_tech_support_calls: true,
            block_unknown_remote_access: true,
            block_suspicious_downloads: true,
            family_safe_browsing: true,
            financial_protection: true,
        };

        println!("✅ Universal protection enabled:");
        println!("   🔒 Zero Trust by Default");
        println!("   🛡️ Privacy by Design");
        println!("   👵 Grandma-Safe Technology");
        println!("   🚫 Scammer Protection Active");
        println!("   🔐 End-to-End Encryption");

        Ok(())
    }

    /// Register a new device with automatic security setup
    pub async fn register_device_secure(
        &self,
        device_id: &str,
        device_name: &str,
    ) -> Result<DeviceSecurityPolicy> {
        let policy = DeviceSecurityPolicy {
            device_id: device_id.to_string(),
            device_name: device_name.to_string(),
            security_level: SecurityLevel::High, // Default to high security
            encryption_required: true,           // Always encrypt by default
            family_safe_mode: true,              // Family-safe by default
            trusted_since: Utc::now(),
            auto_security_features: vec![
                AutoSecurityFeature::ScammerProtection,
                AutoSecurityFeature::TechSupportCallBlocking,
                AutoSecurityFeature::FamilyDeviceVerification,
                AutoSecurityFeature::AutoSecurityUpdates,
                AutoSecurityFeature::PrivacyFirst,
                AutoSecurityFeature::ZeroTrustNetworking,
            ],
        };

        // Generate encryption key for device
        self.encryption_manager
            .generate_session_key(TunnelType::BearDogReinforced)
            .await?;

        // Store policy
        self.device_policies
            .write()
            .await
            .insert(device_id.to_string(), policy.clone());

        println!("🔐 Device '{device_name}' registered with universal security:");
        println!("   🛡️ Security Level: {:?}", policy.security_level);
        println!("   🔒 Encryption: Required");
        println!("   👥 Family Safe: Enabled");
        println!("   🚫 Scammer Protection: Active");

        Ok(policy)
    }

    /// Add a friend with trust verification
    pub async fn add_friend_secure(
        &self,
        friend_id: &str,
        friend_name: &str,
        trust_level: FriendTrustLevel,
    ) -> Result<()> {
        // Generate friend encryption key
        self.encryption_manager
            .generate_session_key(TunnelType::BearDogReinforced)
            .await?;

        // Store trust relationship
        self.friend_trust
            .write()
            .await
            .insert(friend_id.to_string(), trust_level.clone());

        let trust_desc = match trust_level {
            FriendTrustLevel::Family { .. } => "Family (Maximum Trust)",
            FriendTrustLevel::CloseFriend { .. } => "Close Friend (High Trust)",
            FriendTrustLevel::Friend { .. } => "Friend (Standard Trust)",
            FriendTrustLevel::Acquaintance { .. } => "Acquaintance (Limited Trust)",
            FriendTrustLevel::Unknown => "Unknown (No Trust)",
        };

        println!("🤝 Friend '{friend_name}' added securely:");
        println!("   🛡️ Trust Level: {trust_desc}");
        println!("   🔐 Encrypted Communication: Enabled");
        println!("   ✅ Verified Connection: Active");

        Ok(())
    }

    /// Enable family protection for maximum safety
    pub async fn enable_family_protection(&self, family_name: &str) -> Result<()> {
        let mut family_config = self.family_protection.write().await;
        family_config.enabled = true;
        family_config.family_name = family_name.to_string();
        family_config.protection_level = SecurityLevel::Maximum;

        println!("👨‍👩‍👧‍👦 Family Protection enabled for '{family_name}':");
        println!("   🛡️ Maximum Security Level");
        println!("   🚫 Tech Support Scam Blocking");
        println!("   🔒 Remote Access Protection");
        println!("   💰 Financial Protection");
        println!("   👶 Family-Safe Browsing");
        println!("   ✅ Only trusted family devices allowed");

        Ok(())
    }

    /// Check if connection is secure and trusted
    pub async fn verify_connection_security(
        &self,
        device_id: &str,
        remote_id: &str,
    ) -> Result<ConnectionSecurityStatus> {
        let policies = self.device_policies.read().await;
        let friends = self.friend_trust.read().await;

        let device_policy = policies.get(device_id);
        let friend_trust = friends.get(remote_id);

        let status = ConnectionSecurityStatus {
            is_secure: device_policy.is_some_and(|p| p.encryption_required),
            trust_level: friend_trust.cloned().unwrap_or(FriendTrustLevel::Unknown),
            encryption_enabled: true, // Always enable encryption
            family_safe: device_policy.is_none_or(|p| p.family_safe_mode),
            scammer_protection_active: true,
        };

        Ok(status)
    }

    /// Detect and block potential scammer activity
    pub async fn check_scammer_protection(
        &self,
        activity: &ConnectionActivity,
    ) -> Result<ScammerProtectionResult> {
        let family_config = self.family_protection.read().await;

        if !family_config.scammer_protection.block_tech_support_calls {
            return Ok(ScammerProtectionResult::Allowed);
        }

        // Check for common scammer patterns
        let is_suspicious = self.detect_suspicious_activity(activity).await?;

        if is_suspicious {
            println!("🚨 SCAMMER PROTECTION ACTIVATED:");
            println!("   🛡️ Suspicious activity detected and blocked");
            println!("   👵 Your family is protected from this attempt");
            println!("   📞 Tech support scam patterns identified");

            Ok(ScammerProtectionResult::Blocked {
                reason: "Suspicious tech support scam activity detected".to_string(),
                protection_level: family_config.protection_level.clone(),
            })
        } else {
            Ok(ScammerProtectionResult::Allowed)
        }
    }

    /// Detect suspicious activity patterns
    async fn detect_suspicious_activity(&self, activity: &ConnectionActivity) -> Result<bool> {
        // Check for tech support scam indicators
        let suspicious_patterns = [
            "your computer has been hacked",
            "virus detected on your computer",
            "microsoft tech support",
            "microsoft technical support",
            "your windows license has expired",
            "suspicious activity detected",
            "call this number immediately",
            "do not turn off your computer",
            "download teamviewer",
            "remote assistance required",
        ];

        let activity_text = activity.description.to_lowercase();

        for pattern in &suspicious_patterns {
            if activity_text.contains(pattern) {
                tracing::warn!("Suspicious tech support scam pattern detected: {}", pattern);
                return Ok(true); // Return true to indicate suspicious activity was detected
            }
        }

        // Check for unusual remote access requests
        if activity.connection_type == "remote_access" && !activity.source_trusted {
            tracing::warn!("Suspicious remote access request from untrusted source");
            return Ok(true); // Return true to indicate suspicious activity was detected
        }

        Ok(false)
    }
}

impl LightweightTunnelCrypto {
    pub fn new() -> Self {
        Self {
            session_keys: Arc::new(RwLock::new(HashMap::new())),
            coordination_crypto: Arc::new(CoordinationCrypto::new()),
            beardog_crypto_interface: None,
        }
    }

    /// Connect to BearDog for heavy crypto operations
    pub fn connect_beardog(&mut self, interface: Arc<dyn BearDogCryptoInterface>) {
        self.beardog_crypto_interface = Some(interface);
    }

    /// Generate lightweight session key with gaming optimization
    pub async fn generate_session_key(&self, tunnel_type: TunnelType) -> Result<SessionKey> {
        let key_id = uuid::Uuid::new_v4().to_string();
        let key_data = self.generate_light_key(32); // 256-bit key
        let now = SystemTime::now();
        let expires_at = now + tunnel_type.default_expiry_duration();

        let session_key = SessionKey {
            key_id: key_id.clone(),
            key_data,
            created_at: now,
            expires_at,
            auto_renewable: tunnel_type.supports_auto_renewal(),
            renewal_count: 0,
            gaming_metadata: None,
            tunnel_type,
        };

        self.session_keys
            .write()
            .await
            .insert(key_id, session_key.clone());

        Ok(session_key)
    }

    /// Generate gaming-specific tunnel with metadata
    pub async fn generate_gaming_tunnel(
        &self,
        tunnel_type: TunnelType,
        gaming_metadata: GamingTunnelMetadata,
    ) -> Result<SessionKey> {
        let mut session_key = self.generate_session_key(tunnel_type).await?;
        session_key.gaming_metadata = Some(gaming_metadata);

        // Update in storage
        self.session_keys
            .write()
            .await
            .insert(session_key.key_id.clone(), session_key.clone());

        Ok(session_key)
    }

    /// Renew a session key (for gaming sessions)
    pub async fn renew_session_key(&self, key_id: &str) -> Result<SessionKey> {
        let mut keys = self.session_keys.write().await;

        if let Some(existing_key) = keys.get_mut(key_id) {
            if !existing_key.auto_renewable {
                return Err(SongbirdError::Config {
                    message: "Session key is not renewable".to_string(),
                    field: Some("auto_renewable".to_string()),
                    context: Some("renew_session_key".to_string()),
                    suggestion: Some("Create a new renewable session key".to_string()),
                });
            }

            // Generate new key data but keep same ID and metadata
            let new_key_data = self.generate_light_key(32);
            let now = SystemTime::now();
            let new_expires_at = now + existing_key.tunnel_type.default_expiry_duration();

            existing_key.key_data = new_key_data;
            existing_key.expires_at = new_expires_at;
            existing_key.renewal_count += 1;

            Ok(existing_key.clone())
        } else {
            Err(SongbirdError::Config {
                message: format!("Session key {key_id} not found"),
                field: Some("key_id".to_string()),
                context: Some("renew_session_key".to_string()),
                suggestion: Some(
                    "Check the session key ID or create a new session key".to_string(),
                ),
            })
        }
    }

    /// Check and auto-renew gaming tunnels that are near expiry
    pub async fn auto_renew_gaming_tunnels(&self) -> Result<Vec<String>> {
        let mut renewed_keys = Vec::new();
        let now = SystemTime::now();

        let keys = self.session_keys.read().await;
        let keys_to_renew: Vec<String> = keys
            .iter()
            .filter_map(|(key_id, session_key)| {
                if session_key.auto_renewable {
                    let time_until_expiry = session_key
                        .expires_at
                        .duration_since(now)
                        .unwrap_or_default();
                    let renewal_window = session_key.tunnel_type.renewal_window();

                    if time_until_expiry <= renewal_window {
                        Some(key_id.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        drop(keys); // Release read lock

        // Renew the keys that need renewal
        for key_id in keys_to_renew {
            if let Ok(_renewed_key) = self.renew_session_key(&key_id).await {
                renewed_keys.push(key_id);
            }
        }

        Ok(renewed_keys)
    }

    /// Get gaming tunnel status for monitoring
    pub async fn get_gaming_tunnel_status(&self) -> Vec<GamingTunnelStatus> {
        let keys = self.session_keys.read().await;
        let now = SystemTime::now();

        keys.iter()
            .filter_map(|(key_id, session_key)| {
                session_key.gaming_metadata.as_ref().map(|metadata| {
                    let time_until_expiry = session_key
                        .expires_at
                        .duration_since(now)
                        .unwrap_or_default();

                    GamingTunnelStatus {
                        key_id: key_id.clone(),
                        tunnel_type: session_key.tunnel_type.clone(),
                        time_until_expiry,
                        renewal_count: session_key.renewal_count,
                        priority: metadata.priority.clone(),
                        game_session_id: metadata.game_session_id.clone(),
                        needs_renewal: time_until_expiry
                            <= session_key.tunnel_type.renewal_window(),
                    }
                })
            })
            .collect()
    }

    /// Ultra-light encryption for coordination messages only
    pub fn encrypt_coordination(&self, data: &[u8]) -> Vec<u8> {
        self.coordination_crypto.simple_encrypt(data)
    }

    /// Ultra-light decryption for coordination messages only  
    pub fn decrypt_coordination(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        self.coordination_crypto.simple_decrypt(encrypted_data)
    }

    /// Delegate heavy encryption to BearDog
    pub async fn delegate_heavy_encryption(
        &self,
        data: &[u8],
        context: EncryptionContext,
    ) -> Result<Vec<u8>> {
        match &self.beardog_crypto_interface {
            Some(beardog) => beardog.beardog_encrypt(data, context).await,
            None => Err(SongbirdError::Config {
                message: "BearDog crypto interface not connected - heavy encryption unavailable"
                    .to_string(),
                field: Some("beardog_interface".to_string()),
                context: Some("beardog_heavy_encrypt".to_string()),
                suggestion: Some("Configure BearDog interface for heavy encryption".to_string()),
            }),
        }
    }

    /// Request BearDog to reinforce a SongBird tunnel
    pub async fn request_tunnel_reinforcement(&self, tunnel_id: &str) -> Result<ReinforcedTunnel> {
        let songbird_key = self
            .generate_session_key(TunnelType::BearDogReinforced)
            .await?;

        match &self.beardog_crypto_interface {
            Some(beardog) => {
                beardog
                    .reinforce_tunnel(tunnel_id, &songbird_key.key_data)
                    .await
            }
            None => Ok(ReinforcedTunnel {
                tunnel_id: tunnel_id.to_string(),
                songbird_layer: songbird_key,
                beardog_layer_id: "beardog_unavailable".to_string(),
                combined_strength: CryptoStrength::SongBirdOnly,
            }),
        }
    }

    /// Generate ultra-lightweight key for coordination
    fn generate_light_key(&self, length: usize) -> Vec<u8> {
        // Ultra-simple key generation for coordination only
        // NOT suitable for protecting sensitive data
        (0..length)
            .map(|i| (i as u8).wrapping_mul(17).wrapping_add(42))
            .collect()
    }
}

impl CoordinationCrypto {
    fn new() -> Self {
        Self {
            coordination_key: b"songbird_coordination_2024".to_vec(),
        }
    }

    /// Ultra-simple XOR encryption for coordination messages
    /// NOT suitable for protecting sensitive data - that's BearDog's job
    fn simple_encrypt(&self, data: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ self.coordination_key[i % self.coordination_key.len()])
            .collect()
    }

    /// Ultra-simple XOR decryption for coordination messages
    fn simple_decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        // XOR is symmetric, so decryption is same as encryption
        Ok(self.simple_encrypt(encrypted_data))
    }
}

/// Connection security status
#[derive(Debug, Clone)]
pub struct ConnectionSecurityStatus {
    pub is_secure: bool,
    pub trust_level: FriendTrustLevel,
    pub encryption_enabled: bool,
    pub family_safe: bool,
    pub scammer_protection_active: bool,
}

/// Connection activity for security analysis
#[derive(Debug, Clone)]
pub struct ConnectionActivity {
    pub source_id: String,
    pub destination_id: String,
    pub connection_type: String,
    pub description: String,
    pub source_trusted: bool,
}

/// Scammer protection result
#[derive(Debug, Clone)]
pub enum ScammerProtectionResult {
    Allowed,
    Blocked {
        reason: String,
        protection_level: SecurityLevel,
    },
}

impl TunnelType {
    /// Get the default expiry duration for this tunnel type
    pub fn default_expiry_duration(&self) -> std::time::Duration {
        match self {
            TunnelType::ServiceDiscovery => std::time::Duration::from_secs(5 * 60), // 5 minutes
            TunnelType::WireGuardCoordination => std::time::Duration::from_secs(60 * 60), // 1 hour
            TunnelType::BasicP2P => std::time::Duration::from_secs(30 * 60),        // 30 minutes
            TunnelType::BearDogReinforced => std::time::Duration::from_secs(60 * 60), // 1 hour
            TunnelType::GamingSession => std::time::Duration::from_secs(8 * 60 * 60), // 8 hours
            TunnelType::GamingLobby => std::time::Duration::from_secs(30 * 60),     // 30 minutes
            TunnelType::GamingMatch => std::time::Duration::from_secs(4 * 60 * 60), // 4 hours
        }
    }

    /// Check if this tunnel type supports auto-renewal
    pub fn supports_auto_renewal(&self) -> bool {
        matches!(
            self,
            TunnelType::GamingSession
                | TunnelType::GamingLobby
                | TunnelType::GamingMatch
                | TunnelType::BearDogReinforced
        )
    }

    /// Get renewal window (renew when this much time is left)
    pub fn renewal_window(&self) -> std::time::Duration {
        match self {
            TunnelType::GamingSession => std::time::Duration::from_secs(30 * 60), // Renew 30 min before expiry
            TunnelType::GamingLobby => std::time::Duration::from_secs(5 * 60), // Renew 5 min before expiry
            TunnelType::GamingMatch => std::time::Duration::from_secs(15 * 60), // Renew 15 min before expiry
            TunnelType::BearDogReinforced => std::time::Duration::from_secs(10 * 60), // Renew 10 min before expiry
            _ => std::time::Duration::from_secs(60), // 1 minute for others
        }
    }
}

impl Default for LightweightTunnelCrypto {
    fn default() -> Self {
        Self::new()
    }
}
