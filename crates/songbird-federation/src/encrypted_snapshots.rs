//! Encrypted Snapshots Module
//!
//! Federation snapshots with built-in encryption and BearDog integration

use songbird_errors::{Result, SongbirdError};
use crate::security::encryption::ProductionEncryptionProvider;

use crate::security::{
    BearDogEncryptedData, BearDogKeyPurpose, BearDogKeySpec, BearDogRotationPolicy,
    BearDogSecurityContext, BearDogSecurityLevel, BearDogSecurityProvider,
};

// Re-export the NodeId from security module
pub use crate::security::NodeId;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use tokio::sync::RwLock;
use uuid::Uuid;

use tracing::info;

use bincode;

// Placeholder types for missing dependencies
pub type TrustLevel = String;

// use tokio::io::{AsyncReadExt, AsyncWriteExt}; // Unused imports

// ============================================================================
// CORE SNAPSHOT TYPES
// ============================================================================

/// Encrypted snapshot containing service state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSnapshot {
    pub id: String,
    pub service_id: String,
    pub version: u64,
    pub encrypted_data: Vec<u8>,
    pub metadata: SnapshotMetadata,
    pub created_at: DateTime<Utc>,
    pub access_control: AccessControlList,
}

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub snapshot_type: SnapshotType,
    pub size_bytes: u64,
    pub checksum: String,
    pub encryption_algorithm: String,
    pub compression: Option<CompressionType>,
    pub tags: HashMap<String, String>,
    pub name: String,
    pub original_size_bytes: u64,
    pub version: String,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Snapshot type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnapshotType {
    Full,
    Incremental,
    Differential,
    // New variants expected by tests
    Database {
        schema_version: String,
        table_count: u32,
    },
    MLModel {
        model_type: String,
        framework: String,
    },
    Custom {
        custom_type: String,
        metadata: HashMap<String, String>,
    },
}

/// Access control list for snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlList {
    pub owner: NodeId,
    pub access_entries: Vec<NodeAccessEntry>,
    pub default_access: AccessType,
    // New fields expected by tests
    pub read_access: Vec<String>,
    pub write_access: Vec<String>,
    pub public_read: bool,
    pub access_expires_at: Option<DateTime<Utc>>,
}

/// Node access entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAccessEntry {
    pub node_id: NodeId,
    pub access_type: AccessType,
    pub granted_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Access type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessType {
    Read,
    Write,
    Admin,
    None,
}

/// Storage preferences for snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePreferences {
    pub performance_tier: PerformanceTier,
    pub retention_days: u32,
    pub compression_enabled: bool,
    pub encryption_required: bool,
    // New fields expected by tests
    pub preferred_nodes: Vec<String>,
    pub excluded_nodes: Vec<String>,
    pub geographic_region: Option<String>,
    pub preferred_institutions: Vec<String>,
    pub min_storage_trust: TrustLevel,
    pub replication_factor: u32,
}

/// Performance tier for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTier {
    Hot,
    Warm,
    Cold,
    Archive,
}

/// Snapshot request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotRequest {
    pub service_id: String,
    pub request_type: SnapshotRequestType,
    pub filters: Option<SnapshotFilters>,
    pub storage_preferences: StoragePreferences,
}

/// Snapshot request type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SnapshotRequestType {
    Create,
    Restore,
    List,
    Delete,
}

/// Snapshot filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotFilters {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub snapshot_types: Option<Vec<SnapshotType>>,
    pub tags: Option<HashMap<String, String>>,
    // New fields expected by tests
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub min_size_bytes: Option<u64>,
    pub max_size_bytes: Option<u64>,
}

/// Compression type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
    Lz4,
}

/// Snapshot distribution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDistributionStats {
    pub total_snapshots: u64,
    pub total_size_bytes: u64,
    pub nodes_with_copies: Vec<NodeId>,
    pub last_updated: DateTime<Utc>,
}

// ============================================================================
// SNAPSHOT SECURITY PROVIDER TRAIT
// ============================================================================

/// Security provider for snapshot encryption/decryption
#[async_trait]
pub trait SnapshotSecurityProvider: Send + Sync {
    /// Encrypt snapshot data
    async fn encrypt_snapshot(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>>;

    /// Decrypt snapshot data  
    async fn decrypt_snapshot(&self, encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>>;

    /// Generate snapshot key
    async fn generate_snapshot_key(&self, snapshot_id: &str) -> Result<Vec<u8>>;

    /// Verify access permissions
    async fn verify_snapshot_access(
        &self,
        node_id: &NodeId,
        snapshot_id: &str,
        access_type: &AccessType,
    ) -> Result<bool>;
}

/// Snapshot security context
#[derive(Debug, Clone)]
pub struct SnapshotSecurityContext {
    pub snapshot_id: String,
    pub node_id: NodeId,
    pub timestamp: DateTime<Utc>,
    pub access_type: AccessType,
}

// ============================================================================
// PRODUCTION SECURITY ADAPTER
// ============================================================================

/// Production snapshot security adapter using built-in encryption
pub struct ProductionSnapshotSecurityAdapter {
    encryption_user: ProductionEncryptionProvider,
}

impl Default for ProductionSnapshotSecurityAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProductionSnapshotSecurityAdapter {
    pub fn new() -> Self {
        let encryption_config = crate::security::encryption::EncryptionConfig::default();
        Self {
            encryption_user: ProductionEncryptionProvider::new(encryption_config),
        }
    }
}

#[async_trait]
impl SnapshotSecurityProvider for ProductionSnapshotSecurityAdapter {
    async fn encrypt_snapshot(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        let encrypted_data =
            self.encryption_user
                .encrypt(data, key)
                .map_err(|e| SongbirdError::Config {
                    field: Some("encryption".to_string()),
                    message: format!("Encryption failed: {e}"),
                })))?;

        // Serialize the encrypted data for storage
        bincode::serialize(&encrypted_data).map_err(|e| SongbirdError::Config {
            field: Some("serialization".to_string()),
            message: format!("Failed to serialize encrypted data: {e}"),
        })
    }

    async fn decrypt_snapshot(&self, encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        // Deserialize the encrypted data
        let encrypted: crate::security::EncryptedData = bincode::deserialize(encrypted_data)
            .map_err(|e| SongbirdError::Config {
                field: Some("encrypted_data".to_string()),
                message: format!("Failed to deserialize encrypted data: {e}"),
            })))?;

        // Decrypt using the encryption provider
        self.encryption_user
            .decrypt(&encrypted, key)
            .map_err(|e| SongbirdError::Config {
                field: Some("decryption".to_string()),
                message: format!("Decryption failed: {e}"),
            })))
    }

    async fn generate_snapshot_key(&self, _snapshot_id: &str) -> Result<Vec<u8>> {
        self.encryption_user
            .generate_key()
            .map_err(|e| SongbirdError::Config {
                field: Some("key_generation".to_string()),
                message: format!("Key generation failed: {e}"),
            })))
    }

    async fn verify_snapshot_access(
        &self,
        _node_id: &NodeId,
        _snapshot_id: &str,
        _access_type: &AccessType,
    ) -> Result<bool> {
        // Default allow for production adapter
        Ok(true)
    }
}

// ============================================================================
// BEARDOG SECURITY ADAPTER
// ============================================================================

/// BearDog snapshot security adapter
pub struct BearDogSnapshotSecurityAdapter<T: BearDogSecurityProvider> {
    beardog_user: T,
}

impl<T: BearDogSecurityProvider> BearDogSnapshotSecurityAdapter<T> {
    pub fn new(user: T) -> Self {
        Self { beardog_user: user }
    }
}

#[async_trait]
impl<T: BearDogSecurityProvider> SnapshotSecurityProvider for BearDogSnapshotSecurityAdapter<T> {
    async fn encrypt_snapshot(&self, data: &[u8], _key: &[u8]) -> Result<Vec<u8>> {
        let _context = BearDogSecurityContext {
            operation_id: "snapshot_encrypt".to_string(),
            node_id: "0.0.0.0".to_string(),
            timestamp: chrono::Utc::now(),
            security_level: BearDogSecurityLevel::Confidential,
            metadata: HashMap::new(),
        };

        let encrypted = self.beardog_user.encrypt(data, &_context).await?;
        Ok(encrypted.ciphertext)
    }

    async fn decrypt_snapshot(&self, encrypted_data: &[u8], _key: &[u8]) -> Result<Vec<u8>> {
        let _context = BearDogSecurityContext {
            operation_id: "snapshot_decrypt".to_string(),
            node_id: "0.0.0.0".to_string(),
            timestamp: chrono::Utc::now(),
            security_level: BearDogSecurityLevel::Confidential,
            metadata: HashMap::new(),
        };

        let encrypted_data_struct = BearDogEncryptedData {
            algorithm: "AES-256-GCM".to_string(),
            nonce: vec![],
            ciphertext: encrypted_data.to_vec(),
            salt: None,
            key_handle: None,
        };

        self.beardog_user
            .decrypt(&encrypted_data_struct, &_context)
            .await
    }

    async fn generate_snapshot_key(&self, _snapshot_id: &str) -> Result<Vec<u8>> {
        let key_spec = BearDogKeySpec {
            algorithm: "AES-256".to_string(),
            key_size: 256,
            purpose: BearDogKeyPurpose::DataEncryption,
            rotation_policy: BearDogRotationPolicy {
                interval_days: 30,
                auto_rotate: true,
            },
        };

        let key_handle = self.beardog_user.generate_key(&key_spec).await?;
        Ok(key_handle.id.into_bytes())
    }

    async fn verify_snapshot_access(
        &self,
        node_id: &NodeId,
        snapshot_id: &str,
        access_type: &AccessType,
    ) -> Result<bool> {
        // Create BearDog principal, resource, and action
        let principal = crate::security::BearDogPrincipal {
            id: node_id.clone(),
            principal_type: crate::security::BearDogPrincipalType::Node,
            attributes: HashMap::new(),
        };

        let resource = crate::security::BearDogResource {
            id: snapshot_id.to_string(),
            resource_type: "snapshot".to_string(),
            owner: "system".to_string(),
            attributes: HashMap::new(),
        };

        let action = crate::security::BearDogAction {
            name: match access_type {
                AccessType::Read => "read",
                AccessType::Write => "write",
                AccessType::Admin => "admin",
                AccessType::None => return Ok(false),
            }
            .to_string(),
            attributes: HashMap::new(),
        };

        self.beardog_user
            .verify_access(&principal, &resource, &action)
            .await
    }
}

// ============================================================================
// ENCRYPTED SNAPSHOT MANAGER TRAIT
// ============================================================================

/// Encrypted snapshot manager interface
#[async_trait]
pub trait EncryptedSnapshotManager: Send + Sync {
    /// Create encrypted snapshot
    async fn create_snapshot(&self, request: SnapshotRequest) -> Result<EncryptedSnapshot>;

    /// Restore from encrypted snapshot
    async fn restore_snapshot(&self, snapshot_id: &str, target_service: &str) -> Result<()>;

    /// List available snapshots
    async fn list_snapshots(
        &self,
        filters: Option<SnapshotFilters>,
    ) -> Result<Vec<EncryptedSnapshot>>;

    /// Delete snapshot
    async fn delete_snapshot(&self, snapshot_id: &str) -> Result<()>;

    /// Get snapshot statistics
    async fn get_snapshot_stats(&self) -> Result<SnapshotDistributionStats>;
}

// ============================================================================
// DEFAULT IMPLEMENTATION WITH PRODUCTION SECURITY
// ============================================================================

/// Default encrypted snapshot manager using production security
pub struct DefaultEncryptedSnapshotManager {
    security_user: ProductionSnapshotSecurityAdapter,
    snapshots: RwLock<HashMap<String, EncryptedSnapshot>>,
}

impl Default for DefaultEncryptedSnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultEncryptedSnapshotManager {
    pub fn new() -> Self {
        Self {
            security_user: ProductionSnapshotSecurityAdapter::new(),
            snapshots: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl EncryptedSnapshotManager for DefaultEncryptedSnapshotManager {
    async fn create_snapshot(&self, request: SnapshotRequest) -> Result<EncryptedSnapshot> {
        let snapshot_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Create snapshot data from request
        let snapshot_data = self.create_snapshot_data(&request).await?;

        let key = self
            .security_user
            .generate_snapshot_key(&snapshot_id)
            .await?;
        let encrypted_data = self
            .security_user
            .encrypt_snapshot(&snapshot_data, &key)
            .await?;

        let snapshot = EncryptedSnapshot {
            id: snapshot_id.clone(),
            service_id: request.service_id,
            version: 1,
            encrypted_data,
            metadata: SnapshotMetadata {
                snapshot_type: SnapshotType::Full,
                size_bytes: snapshot_data.len() as u64,
                checksum: self.calculate_checksum(&snapshot_data),
                encryption_algorithm: "AES-256-GCM".to_string(),
                compression: Some(CompressionType::Gzip),
                tags: HashMap::new(),
                name: String::new(),
                original_size_bytes: 0,
                version: String::new(),
                expires_at: None,
            },
            created_at: now,
            access_control: AccessControlList {
                owner: crate::config::constants::node_id(),
                access_entries: vec![],
                default_access: AccessType::Read,
                read_access: Vec::new(),
                write_access: Vec::new(),
                public_read: false,
                access_expires_at: None,
            },
        };

        self.snapshots
            .write()
            .await
            .insert(snapshot_id, snapshot.clone());
        Ok(snapshot)
    }

    async fn restore_snapshot(&self, snapshot_id: &str, _target_service: &str) -> Result<()> {
        let snapshots = self.snapshots.read().await;
        if snapshots.contains_key(snapshot_id) {
            info!("Restoring snapshot: {}", snapshot_id);
            Ok(())
        } else {
            Err(SongbirdError::NotFound(Box::new(NotFoundError {
                resource: "snapshot".to_string(),
                message: format!("Snapshot {} not found", snapshot_id),
                searched_paths: None,
                
            })))
        }
    }

    async fn list_snapshots(
        &self,
        _filters: Option<SnapshotFilters>,
    ) -> Result<Vec<EncryptedSnapshot>> {
        let snapshots = self.snapshots.read().await;
        Ok(snapshots.values().cloned().collect())
    }

    async fn delete_snapshot(&self, snapshot_id: &str) -> Result<()> {
        let mut snapshots = self.snapshots.write().await;
        if snapshots.remove(snapshot_id).is_some() {
            info!("Deleted snapshot: {}", snapshot_id);
            Ok(())
        } else {
            Err(SongbirdError::NotFound(Box::new(NotFoundError {
                resource: "snapshot".to_string(),
                message: format!("Snapshot {} not found", snapshot_id),
                searched_paths: None,
                
            })))
        }
    }

    async fn get_snapshot_stats(&self) -> Result<SnapshotDistributionStats> {
        let snapshots = self.snapshots.read().await;
        Ok(SnapshotDistributionStats {
            total_snapshots: snapshots.len() as u64,
            total_size_bytes: snapshots.values().map(|s| s.metadata.size_bytes).sum(),
            nodes_with_copies: vec![crate::config::constants::node_id()],
            last_updated: Utc::now(),
        })
    }
}

impl DefaultEncryptedSnapshotManager {
    /// Create snapshot data from request
    async fn create_snapshot_data(&self, request: &SnapshotRequest) -> Result<Vec<u8>> {
        let mut snapshot = Vec::new();

        // Include service metadata
        let service_metadata = serde_json::json!({
            "service_id": request.service_id,
            "request_type": format!("{request.request_type}"),
            "timestamp": chrono::Utc::now(),
            "preferences": {
                "performance_tier": format!("{request.storage_preferences.performance_tier}"),
                "retention_days": request.storage_preferences.retention_days,
                "compression_enabled": request.storage_preferences.compression_enabled,
                "encryption_required": request.storage_preferences.encryption_required,
            }
        });

        let metadata_bytes = serde_json::to_vec(&service_metadata)?;
        snapshot.extend_from_slice(&metadata_bytes);

        // Include system state placeholder
        let system_state = serde_json::json!({
            "node_id": crate::config::constants::node_id(),
            "timestamp": chrono::Utc::now(),
            "active_services": vec!["gaming", "federation", "discovery"],
            "system_metrics": {
                "cpu_usage": 0.0,
                "memory_usage": 0.0,
                "network_connections": 0,
            }
        });

        let state_bytes = serde_json::to_vec(&system_state)?;
        snapshot.extend_from_slice(&state_bytes);

        Ok(snapshot)
    }

    /// Calculate checksum for data integrity
    fn calculate_checksum(&self, data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{hasher.finish(}"))
    }
}

// Helper structs for the snapshot system
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub address: std::net::SocketAddr,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceInfo {
    pub service_id: String,
    pub service_type: String,
    pub endpoint: String,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlayerInfo {
    pub player_id: String,
    pub display_name: String,
    pub address: std::net::SocketAddr,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TunnelInfo {
    pub tunnel_id: String,
    pub tunnel_type: String,
    pub endpoints: Vec<std::net::SocketAddr>,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GamingState {
    pub active_sessions: u32,
    pub total_players: u32,
    pub protocols_in_use: Vec<String>,
}
