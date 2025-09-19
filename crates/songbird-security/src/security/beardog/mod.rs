//! BearDog Security Integration Module
//!
//! Provides types, traits, and configurations for BearDog security integration.
//! This module contains all BearDog-specific security functionality.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

use songbird_errors::Result;

// Re-export common authentication types and traits
pub use crate::security::*;

// Re-export NodeId type for compatibility
// pub type NodeId = String; // Removed duplicate - already defined in core/types.rs

// ============================================================================
// BEARDOG SECURITY PROVIDER INTEGRATION
// ============================================================================

/// BearDog Security Provider - External security module interface
///
/// This trait allows integration with your in-house BearDog security module
/// for encryption, key management, access control, and audit logging.
#[async_trait]
pub trait BearDogSecurityProvider: Send + Sync {
    /// Encrypt data with BearDog's security context
    async fn encrypt(
        &self,
        data: &[u8],
        context: &BearDogSecurityContext,
    ) -> Result<BearDogEncryptedData>;

    /// Decrypt data with BearDog's security context
    async fn decrypt(
        &self,
        encrypted: &BearDogEncryptedData,
        context: &BearDogSecurityContext,
    ) -> Result<Vec<u8>>;

    /// Derive encryption key using BearDog's key management
    async fn derive_key(&self, key_id: &str, context: &BearDogKeyContext) -> Result<Vec<u8>>;

    /// Generate new encryption key with BearDog
    async fn generate_key(&self, key_spec: &BearDogKeySpec) -> Result<BearDogKeyHandle>;

    /// Verify access permissions using BearDog's access control
    async fn verify_access(
        &self,
        principal: &BearDogPrincipal,
        resource: &BearDogResource,
        action: &BearDogAction,
    ) -> Result<bool>;

    /// Establish secure communication channel
    async fn establish_secure_channel(&self, peer_id: &NodeId) -> Result<BearDogSecureChannel>;

    /// Log security events for audit
    async fn log_security_event(&self, event: &BearDogSecurityEvent) -> Result<()>;

    /// Rotate encryption keys
    async fn rotate_key(&self, key_id: &str) -> Result<BearDogKeyHandle>;

    /// Get compliance report
    async fn get_compliance_report(
        &self,
        period: &BearDogTimePeriod,
    ) -> Result<BearDogComplianceReport>;
}

// ============================================================================
// BEARDOG SECURITY TYPES
// ============================================================================

/// Security context for BearDog operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogSecurityContext {
    pub security_level: BearDogSecurityLevel,
    pub use_bstp: bool,
    pub metadata: HashMap<String, String>,
}

/// Key context for BearDog key operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogKeyContext {
    pub key_purpose: BearDogKeyPurpose,
    pub expiration: Option<DateTime<Utc>>,
    pub access_policy: String,
    pub metadata: HashMap<String, String>,
}

/// Key specification for generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogKeySpec {
    pub algorithm: String,
    pub key_size: usize,
    pub purpose: BearDogKeyPurpose,
    pub rotation_policy: BearDogRotationPolicy,
}

/// Key handle for secure key reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogKeyHandle {
    pub key_id: String,
    pub algorithm: String,
    pub created_at: SystemTime,
}

/// Security principal (user, service, node)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogPrincipal {
    pub id: String,
    pub principal_type: BearDogPrincipalType,
    pub attributes: HashMap<String, String>,
}

/// Resource being accessed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogResource {
    pub id: String,
    pub resource_type: String,
    pub owner: String,
    pub attributes: HashMap<String, String>,
}

/// Action being performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogAction {
    pub name: String,
    pub attributes: HashMap<String, String>,
}

/// Secure communication channel
#[derive(Debug, Clone)]
pub struct BearDogSecureChannel {
    pub channel_id: String,
    pub peer_id: NodeId,
    pub established_at: DateTime<Utc>,
    pub encryption_key: Vec<u8>,
}

/// Security event for audit logging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogSecurityEvent {
    pub event_id: String,
    pub event_type: BearDogSecurityEventType,
    pub principal: BearDogPrincipal,
    pub resource: Option<BearDogResource>,
    pub action: Option<BearDogAction>,
    pub timestamp: DateTime<Utc>,
    pub outcome: BearDogSecurityOutcome,
    pub details: HashMap<String, String>,
}

/// Encrypted data with BearDog security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogEncryptedData {
    pub ciphertext: Vec<u8>,
    pub key_id: String,
    pub algorithm: String,
    pub metadata: HashMap<String, String>,
}

/// Time period for compliance reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogTimePeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

/// Compliance report from BearDog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogComplianceReport {
    pub report_id: String,
    pub period: BearDogTimePeriod,
    pub compliance_level: f32,
    pub violations: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Key rotation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogRotationPolicy {
    pub interval_days: u32,
    pub auto_rotate: bool,
}

/// Node identifier type
pub type NodeId = String;

/// BearDog configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogConfig {
    pub endpoint: String,
    pub api_key: String,
    pub timeout_seconds: u64,
    pub enabled: bool,
}

impl Default for BearDogConfig {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            api_key: String::new(),
            timeout_seconds: 30,
            enabled: false,
        }
    }
}

// ============================================================================
// BEARDOG ENUMS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BearDogSecurityLevel {
    Standard,
    Public,
    Internal,
    High,
    Confidential,
    Secret,
    TopSecret,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BearDogKeyPurpose {
    DataEncryption,
    KeyEncryption,
    DigitalSignature,
    Authentication,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BearDogPrincipalType {
    User,
    Device,
    Service,
    Node,
    System,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BearDogSecurityEventType {
    Authentication,
    Authorization,
    Encryption,
    Decryption,
    KeyGeneration,
    KeyRotation,
    AccessGranted,
    AccessDenied,
    SecurityViolation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BearDogSecurityOutcome {
    Success,
    Failure,
    Denied,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BearDogAuditLevel {
    Minimal,
    Standard,
    Detailed,
    Comprehensive,
    Paranoid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BearDogComplianceMode {
    Standard,
    Strict,
    FIPS140,
    SOC2,
    GDPR,
}

// ============================================================================
// CONCRETE BEARDOG SECURITY PROVIDER IMPLEMENTATION
// ============================================================================

/// Production BearDog Security Provider Implementation
///
/// This provides a concrete implementation of the BearDogSecurityProvider trait
/// that can be used when BearDog is available. It handles actual BearDog API calls.
pub struct ProductionBearDogProvider {
    client: Arc<RwLock<reqwest::Client>>,
    config: BearDogConfig,
    base_url: String,
    api_key: String,
}

impl ProductionBearDogProvider {
    /// Create a new production BearDog provider
    pub async fn new(config: BearDogConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "Failed to create BearDog HTTP client: {}",
                    e
                ))
            })?;

        // Default BearDog endpoints - config fields are String not Option<String>
        let base_url = if config.endpoint.is_empty() {
            "https://localhost:9443".to_string()
        } else {
            config.endpoint.clone()
        };

        let api_key = if config.api_key.is_empty() {
            std::env::var("BEARDOG_API_KEY").unwrap_or_default()
        } else {
            config.api_key.clone()
        };

        if api_key.is_empty() {
            return Err(songbird_errors::SongbirdError::security(
                "BearDog API key not configured - set BEARDOG_API_KEY environment variable",
            ));
        }

        Ok(Self {
            client: Arc::new(RwLock::new(client)),
            config,
            base_url,
            api_key,
        })
    }

    /// Check if BearDog service is available
    pub async fn is_available(&self) -> bool {
        let client = self.client.read().await;
        let health_url = format!("{}/api/v1/health", self.base_url);

        match client
            .get(&health_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}

#[async_trait]
impl BearDogSecurityProvider for ProductionBearDogProvider {
    async fn encrypt(
        &self,
        data: &[u8],
        context: &BearDogSecurityContext,
    ) -> Result<BearDogEncryptedData> {
        let client = self.client.read().await;
        let url = format!("{}/api/v1/crypto/encrypt", self.base_url);

        let request_payload = serde_json::json!({
            "data": base64::encode(data),
            "security_level": context.security_level,
            "use_bstp": context.use_bstp,
            "metadata": context.metadata,
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "BearDog encrypt request failed: {}",
                    e
                ))
            })?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "Failed to parse BearDog encrypt response: {}",
                    e
                ))
            })?;

            let encrypted_data = result
                .get("encrypted_data")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    songbird_errors::SongbirdError::security(
                        "Invalid BearDog encrypt response format",
                    )
                })?;

            let key_id = result
                .get("key_id")
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string();

            Ok(BearDogEncryptedData {
                ciphertext: base64::decode(encrypted_data).map_err(|e| {
                    songbird_errors::SongbirdError::security(format!(
                        "Invalid base64 in BearDog response: {}",
                        e
                    ))
                })?,
                key_id,
                algorithm: "aes-256-gcm".to_string(),
                metadata: HashMap::new(),
            })
        } else {
            Err(songbird_errors::SongbirdError::security(format!(
                "BearDog encryption failed: {}",
                response.status()
            )))
        }
    }

    async fn decrypt(
        &self,
        encrypted: &BearDogEncryptedData,
        context: &BearDogSecurityContext,
    ) -> Result<Vec<u8>> {
        let client = self.client.read().await;
        let url = format!("{}/api/v1/crypto/decrypt", self.base_url);

        let request_payload = serde_json::json!({
            "encrypted_data": base64::encode(&encrypted.ciphertext),
            "key_id": encrypted.key_id,
            "algorithm": encrypted.algorithm,
            "security_level": context.security_level,
            "metadata": encrypted.metadata,
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "BearDog decrypt request failed: {}",
                    e
                ))
            })?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "Failed to parse BearDog decrypt response: {}",
                    e
                ))
            })?;

            let plaintext = result
                .get("plaintext")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    songbird_errors::SongbirdError::security(
                        "Invalid BearDog decrypt response format",
                    )
                })?;

            base64::decode(plaintext).map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "Invalid base64 in BearDog decrypt response: {}",
                    e
                ))
            })
        } else {
            Err(songbird_errors::SongbirdError::security(format!(
                "BearDog decryption failed: {}",
                response.status()
            )))
        }
    }

    async fn derive_key(&self, key_id: &str, context: &BearDogKeyContext) -> Result<Vec<u8>> {
        let client = self.client.read().await;
        let url = format!("{}/api/v1/keys/derive", self.base_url);

        let request_payload = serde_json::json!({
            "key_id": key_id,
            "key_purpose": context.key_purpose,
            "access_policy": context.access_policy,
            "metadata": context.metadata,
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "BearDog key derivation request failed: {}",
                    e
                ))
            })?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "Failed to parse BearDog key derivation response: {}",
                    e
                ))
            })?;

            let derived_key = result
                .get("derived_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    songbird_errors::SongbirdError::security(
                        "Invalid BearDog key derivation response format",
                    )
                })?;

            base64::decode(derived_key).map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "Invalid base64 in BearDog key derivation response: {}",
                    e
                ))
            })
        } else {
            Err(songbird_errors::SongbirdError::security(format!(
                "BearDog key derivation failed: {}",
                response.status()
            )))
        }
    }

    async fn generate_key(&self, key_spec: &BearDogKeySpec) -> Result<BearDogKeyHandle> {
        let client = self.client.read().await;
        let url = format!("{}/api/v1/keys/generate", self.base_url);

        let request_payload = serde_json::json!({
            "algorithm": key_spec.algorithm,
            "key_size": key_spec.key_size,
            "purpose": key_spec.purpose,
            "rotation_policy": key_spec.rotation_policy,
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "BearDog key generation request failed: {}",
                    e
                ))
            })?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "Failed to parse BearDog key generation response: {}",
                    e
                ))
            })?;

            Ok(BearDogKeyHandle {
                key_id: result
                    .get("key_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        songbird_errors::SongbirdError::security(
                            "No key_id in BearDog key generation response",
                        )
                    })?
                    .to_string(),
                algorithm: key_spec.algorithm.clone(),
                created_at: SystemTime::now(),
            })
        } else {
            Err(songbird_errors::SongbirdError::security(format!(
                "BearDog key generation failed: {}",
                response.status()
            )))
        }
    }

    async fn verify_access(
        &self,
        principal: &BearDogPrincipal,
        resource: &BearDogResource,
        action: &BearDogAction,
    ) -> Result<bool> {
        let client = self.client.read().await;
        let url = format!("{}/api/v1/access/verify", self.base_url);

        let request_payload = serde_json::json!({
            "principal": {
                "id": principal.id,
                "type": principal.principal_type,
                "attributes": principal.attributes,
            },
            "resource": {
                "id": resource.id,
                "type": resource.resource_type,
                "owner": resource.owner,
                "attributes": resource.attributes,
            },
            "action": {
                "name": action.name,
                "attributes": action.attributes,
            },
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "BearDog access verification request failed: {}",
                    e
                ))
            })?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "Failed to parse BearDog access verification response: {}",
                    e
                ))
            })?;

            Ok(result
                .get("allowed")
                .and_then(|v| v.as_bool())
                .unwrap_or(false))
        } else {
            // On failure, deny access by default (secure default)
            Ok(false)
        }
    }

    async fn establish_secure_channel(&self, peer_id: &NodeId) -> Result<BearDogSecureChannel> {
        let client = self.client.read().await;
        let url = format!("{}/api/v1/channels/establish", self.base_url);

        let request_payload = serde_json::json!({
            "peer_id": peer_id,
            "channel_type": "BSTP",
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "BearDog secure channel establishment request failed: {}",
                    e
                ))
            })?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "Failed to parse BearDog channel response: {}",
                    e
                ))
            })?;

            let channel_id = result
                .get("channel_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    songbird_errors::SongbirdError::security(
                        "No channel_id in BearDog channel response",
                    )
                })?
                .to_string();

            let encryption_key_b64 = result
                .get("encryption_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    songbird_errors::SongbirdError::security(
                        "No encryption_key in BearDog channel response",
                    )
                })?;

            let encryption_key = base64::decode(encryption_key_b64).map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "Invalid base64 encryption key from BearDog: {}",
                    e
                ))
            })?;

            Ok(BearDogSecureChannel {
                channel_id,
                peer_id: peer_id.clone(),
                established_at: Utc::now(),
                encryption_key,
            })
        } else {
            Err(songbird_errors::SongbirdError::security(format!(
                "BearDog secure channel establishment failed: {}",
                response.status()
            )))
        }
    }

    async fn log_security_event(&self, event: &BearDogSecurityEvent) -> Result<()> {
        let client = self.client.read().await;
        let url = format!("{}/api/v1/audit/log", self.base_url);

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(event)
            .send()
            .await
            .map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "BearDog audit logging request failed: {}",
                    e
                ))
            })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(songbird_errors::SongbirdError::security(format!(
                "BearDog audit logging failed: {}",
                response.status()
            )))
        }
    }

    async fn rotate_key(&self, key_id: &str) -> Result<BearDogKeyHandle> {
        let client = self.client.read().await;
        let url = format!("{}/api/v1/keys/{}/rotate", self.base_url, key_id);

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "BearDog key rotation request failed: {}",
                    e
                ))
            })?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await.map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "Failed to parse BearDog key rotation response: {}",
                    e
                ))
            })?;

            Ok(BearDogKeyHandle {
                key_id: result
                    .get("new_key_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or(key_id)
                    .to_string(),
                algorithm: result
                    .get("algorithm")
                    .and_then(|v| v.as_str())
                    .unwrap_or("aes-256-gcm")
                    .to_string(),
                created_at: SystemTime::now(),
            })
        } else {
            Err(songbird_errors::SongbirdError::security(format!(
                "BearDog key rotation failed: {}",
                response.status()
            )))
        }
    }

    async fn get_compliance_report(
        &self,
        period: &BearDogTimePeriod,
    ) -> Result<BearDogComplianceReport> {
        let client = self.client.read().await;
        let url = format!("{}/api/v1/compliance/report", self.base_url);

        let request_payload = serde_json::json!({
            "start_date": period.start_date,
            "end_date": period.end_date,
        });

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "BearDog compliance report request failed: {}",
                    e
                ))
            })?;

        if response.status().is_success() {
            let report: BearDogComplianceReport = response.json().await.map_err(|e| {
                songbird_errors::SongbirdError::security(format!(
                    "Failed to parse BearDog compliance report: {}",
                    e
                ))
            })?;
            Ok(report)
        } else {
            Err(songbird_errors::SongbirdError::security(format!(
                "BearDog compliance report failed: {}",
                response.status()
            )))
        }
    }
}
