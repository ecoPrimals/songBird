//! BearDog Security Integration Module
//!
//! Provides types, traits, and configurations for BearDog security integration.
//! This module contains all BearDog-specific security functionality.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

use songbird_errors::Result;

// Re-export NodeId type for compatibility
pub type NodeId = String;

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

/// Encrypted data with BearDog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogEncryptedData {
    pub data: Vec<u8>,
    pub algorithm: String,
    pub key_id: String,
}

/// Time period for compliance reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogTimePeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Compliance report from BearDog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogComplianceReport {
    pub period: BearDogTimePeriod,
    pub encryption_operations: u64,
    pub key_rotations: u64,
    pub access_violations: u64,
    pub compliance_score: f64,
    pub recommendations: Vec<String>,
}

/// Key rotation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogRotationPolicy {
    pub interval_days: u32,
    pub auto_rotate: bool,
}

/// BearDog configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearDogConfig {
    pub endpoint: String,
    pub api_key: String,
    pub security_level: BearDogSecurityLevel,
    pub audit_level: BearDogAuditLevel,
    pub compliance_mode: BearDogComplianceMode,
    pub metadata: HashMap<String, String>,
}

impl Default for BearDogConfig {
    fn default() -> Self {
        Self {
            endpoint: songbird_config::config::constants::network::DEFAULT_BEARDOG_ENDPOINT.to_string(),
            api_key: "your_api_key".to_string(),
            security_level: BearDogSecurityLevel::Internal,
            audit_level: BearDogAuditLevel::Standard,
            compliance_mode: BearDogComplianceMode::Standard,
            metadata: HashMap::new(),
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
