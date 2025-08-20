use chrono::{DateTime, Utc};
/// Universal Security Manager Module
///
/// Contains the main UniversalSecurityManager and high-level security orchestration logic
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::crypto::{LightweightTunnelCrypto, SecurityProviderManager};
use super::types::*;
use songbird_errors::SongbirdError;

/// Family protection configuration
#[derive(Debug, Clone, Default)]
pub struct FamilyProtectionConfig {
    pub enabled: bool,
    pub parental_controls: bool,
    pub content_filtering: bool,
}

/// Universal Security Manager - "Secure for All"
pub struct UniversalSecurityManager {
    /// Security policies for all devices
    device_policies: Arc<RwLock<HashMap<String, DeviceSecurityPolicy>>>,
    /// Friend trust relationships
    friend_trust: Arc<RwLock<HashMap<String, FriendTrustLevel>>>,
    /// Family protection settings - using local config
    family_protection: Arc<RwLock<FamilyProtectionConfig>>,
    /// Ultra-lightweight encryption manager for SongBird tunnel coordination
    /// Heavy crypto operations should be delegated to security providers
    encryption_manager: Arc<LightweightTunnelCrypto>,
    /// Security provider manager
    provider_manager: Arc<SecurityProviderManager>,
    /// Scammer protection configuration
    scammer_protection: Arc<RwLock<ScammerProtectionConfig>>,
}

impl UniversalSecurityManager {
    /// Create a new Universal Security Manager
    pub fn new() -> Self {
        Self {
            device_policies: Arc::new(RwLock::new(HashMap::new())),
            friend_trust: Arc::new(RwLock::new(HashMap::new())),
            family_protection: Arc::new(RwLock::new(FamilyProtectionConfig::default())),
            encryption_manager: Arc::new(LightweightTunnelCrypto::new()),
            provider_manager: Arc::new(SecurityProviderManager::new()),
            scammer_protection: Arc::new(RwLock::new(ScammerProtectionConfig::default())),
        }
    }

    /// Register a new device with security policy
    pub async fn register_device(&self) -> SongbirdResult<()> {
        let policy = DeviceSecurityPolicy {
            device_id: device_id.clone(),
            device_name,
            security_level: security_level.clone(),
            encryption_required: matches!(
                security_level,
                SecurityLevel::High | SecurityLevel::Maximum
            ),
            family_safe_mode: true, // Default to safe
            trusted_since: Utc::now(),
            auto_security_features: self.get_auto_features_for_level(&security_level),
        };

        let mut policies = self.device_policies.write().await;
        policies.insert(device_id, policy);

        Ok(())
    }

    /// Get security policy for a device
    pub async fn get_device_policy(&self) -> Option<DeviceSecurityPolicy> {
        let policies = self.device_policies.read().await;
        policies.get(device_id).cloned()
    }

    /// Update friend trust level
    pub async fn update_friend_trust(&self) -> SongbirdResult<()> {
        let mut trust = self.friend_trust.write().await;
        trust.insert(friend_id, trust_level);
        Ok(())
    }

    /// Get friend trust level
    pub async fn get_friend_trust(&self) -> FriendTrustLevel {
        let trust = self.friend_trust.read().await;
        trust
            .get(friend_id)
            .cloned()
            .unwrap_or(FriendTrustLevel::Unknown)
    }

    /// Check if connection should be allowed
    pub async fn check_connection_security(&self) -> SongbirdResult<ScammerProtectionResult> {
        let trust_level = self.get_friend_trust(remote_id).await;
        let scammer_config = self.scammer_protection.read().await;

        if !scammer_config.enabled {
            return Ok(songbird_errors::evolved_success(ScammerProtectionResult::Safe));
        }

        // Check based on trust level
        match trust_level {
            FriendTrustLevel::Family { .. } | FriendTrustLevel::TrustedFriend { .. } => {
                Ok(songbird_errors::evolved_success(ScammerProtectionResult::Safe))
            }
            FriendTrustLevel::Acquaintance {
                positive_interactions,
                ..
            } => {
                if positive_interactions >= 5 {
                    Ok(songbird_errors::evolved_success(ScammerProtectionResult::Safe))
                } else {
                    Ok(ScammerProtectionResult::Suspicious {
                        reason: "Limited interaction history".to_string(),
                        confidence: 0.3,
                    })
                }
            }
            FriendTrustLevel::Unknown => {
                // Apply scammer detection logic
                self.detect_potential_scammer(remote_id, connection_type, &scammer_config)
                    .await
            }
        }
    }

    /// Create a secure gaming tunnel
    pub async fn create_gaming_tunnel(&self) -> SongbirdResult<()> {
        let metadata = GamingTunnelMetadata {
            tunnel_id: tunnel_id.clone(),
            game_name,
            players,
            created_at: std::time::SystemTime::now(),
            priority: GamingPriority::Normal,
            security_level: security_level.clone(),
        };

        // Register tunnel with encryption manager
        self.encryption_manager.register_tunnel(metadata).await?;

        // Generate session key based on security level
        let tunnel_type = TunnelType::Gaming {
            protocol: "universal".to_string(),
            is_lan: true,
        };

        self.encryption_manager
            .generate_session_key(&tunnel_id, tunnel_type)
            .await?;

        Ok(())
    }

    /// Get gaming tunnel status
    pub async fn get_tunnel_status(&self) -> Option<GamingTunnelStatus> {
        self.encryption_manager.get_tunnel_status(tunnel_id).await
    }

    /// Update family protection settings
    pub async fn update_family_protection(&self) -> SongbirdResult<()> {
        let mut protection = self.family_protection.write().await;
        *protection = config;
        Ok(())
    }

    /// Get current family protection settings
    pub async fn get_family_protection(&self) -> FamilyProtectionConfig {
        let protection = self.family_protection.read().await;
        protection.clone()
    }

    /// Update scammer protection configuration
    pub async fn update_scammer_protection(&self) -> SongbirdResult<()> {
        let mut protection = self.scammer_protection.write().await;
        *protection = config;
        Ok(())
    }

    /// Register a security provider
    pub async fn register_security_provider(&self) -> SongbirdResult<()> {
        self.provider_manager.register_provider(context).await
    }

    /// Get available security providers for a capability
    pub async fn get_security_providers(&self) -> Vec<String> {
        self.provider_manager
            .get_providers_for_capability(capability)
            .await
    }

    /// Cleanup expired resources
    pub async fn cleanup_expired(&self) -> SongbirdResult<u32> {
        let cleaned_keys = self.encryption_manager.cleanup_expired_keys().await?;

        // Could add more cleanup logic here

        Ok(songbird_errors::evolved_success(cleaned_keys))
    }

    // Private helper methods

    /// Get auto security features for a security level
    fn get_auto_features_for_level(&self, level: &SecurityLevel) -> Vec<AutoSecurityFeature> {
        match level {
            SecurityLevel::Minimal => vec![AutoSecurityFeature::ScammerDetection],
            SecurityLevel::Standard => vec![
                AutoSecurityFeature::ScammerDetection,
                AutoSecurityFeature::FamilySafeFiltering,
                AutoSecurityFeature::ConnectionMonitoring,
            ],
            SecurityLevel::High => vec![
                AutoSecurityFeature::ScammerDetection,
                AutoSecurityFeature::AutoEncryption,
                AutoSecurityFeature::FamilySafeFiltering,
                AutoSecurityFeature::MalwareProtection,
                AutoSecurityFeature::ConnectionMonitoring,
            ],
            SecurityLevel::Maximum => vec![
                AutoSecurityFeature::ScammerDetection,
                AutoSecurityFeature::AutoEncryption,
                AutoSecurityFeature::FamilySafeFiltering,
                AutoSecurityFeature::MalwareProtection,
                AutoSecurityFeature::PrivacyEnhancement,
                AutoSecurityFeature::ConnectionMonitoring,
            ],
        }
    }

    /// Detect potential scammer using various signals
    async fn detect_potential_scammer(&self) -> SongbirdResult<ScammerProtectionResult> {
        let mut suspicion_score = 0.0f32;
        let mut warnings = Vec::new();

        // Check for suspicious patterns (simplified heuristics)
        if remote_id.contains("admin") || remote_id.contains("support") {
            suspicion_score += 0.3;
            warnings.push("Suspicious username pattern".to_string());
        }

        if connection_type == "file_transfer" && config.block_suspicious_files {
            suspicion_score += 0.2;
            warnings.push("Unsolicited file transfer".to_string());
        }

        // Apply age-appropriate protection
        match config.age_appropriate_protection {
            AgeProtectionLevel::Child => suspicion_score += 0.4, // More protective
            AgeProtectionLevel::Teen => suspicion_score += 0.2,
            AgeProtectionLevel::Adult => {} // No additional suspicion
        }

        // Determine result based on suspicion score and sensitivity
        let adjusted_score = suspicion_score * config.sensitivity;

        if adjusted_score >= 0.8 {
            Ok(ScammerProtectionResult::Dangerous {
                threat_type: "High risk connection".to_string(),
                evidence: warnings,
            })
        } else if adjusted_score >= 0.4 {
            Ok(ScammerProtectionResult::Suspicious {
                reason: warnings.join(", "),
                confidence: adjusted_score,
            })
        } else {
            Ok(songbird_errors::evolved_success(ScammerProtectionResult::Safe))
        }
    }
}

impl Default for UniversalSecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

// Default implementation for ScammerProtectionConfig
impl Default for ScammerProtectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sensitivity: 0.5,
            auto_block: false, // Don't auto-block by default
            warn_users: true,
            share_threat_intel: false, // Privacy by default
            block_suspicious_files: true,
            social_engineering_protection: true,
            fake_friend_protection: true,
            age_appropriate_protection: AgeProtectionLevel::Adult,
        }
    }
}
