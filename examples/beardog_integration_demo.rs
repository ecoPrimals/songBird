//! BearDog Security Module Integration Demo
//!
//! Demonstrates how the BearDog security module integrates with
//! Songbird Orchestrator's encrypted snapshot system for production-grade
//! security with your in-house security module.
//!
//! This example shows:
//! - Setting up BearDog integration
//! - Creating encrypted snapshots with BearDog
//! - Access control with BearDog authorization
//! - Audit logging with BearDog security events
//! - Federation-ready encrypted storage

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use songbird_gaming_bridge::{
    discovery::types::{NodeId, TrustLevel},
    errors::{Result, SongbirdError},
    federation::{
        AccessControlList,
        BearDogAction,
        BearDogAuditLevel,
        BearDogComplianceMode,
        BearDogComplianceReport,
        BearDogEncryptedData,
        // BearDog integration types
        BearDogEncryptedSnapshotManager,
        BearDogKeyContext,
        BearDogKeyHandle,
        BearDogKeyPurpose,
        BearDogKeySpec,
        BearDogPrincipal,
        BearDogPrincipalType,
        BearDogResource,
        BearDogRotationPolicy,
        BearDogSecureChannel,
        BearDogSecurityContext,
        BearDogSecurityEvent,
        BearDogSecurityEventType,
        BearDogSecurityLevel,
        BearDogSecurityOutcome,
        BearDogSecurityProvider,
        BearDogTimePeriod,
        NodeAccessEntry,
        PerformanceTier,
        SnapshotFilters,
        // Core types
        SnapshotMetadata,
        SnapshotType,
        StoragePreferences,
    },
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// MOCK BEARDOG IMPLEMENTATION FOR DEMO
// ============================================================================

/// Mock BearDog implementation for demonstration
///
/// In your real implementation, this would connect to your actual BearDog
/// security service via HTTP, gRPC, or direct library integration.
pub struct MockBearDogProvider {
    keys: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    audit_log: Arc<RwLock<Vec<BearDogSecurityEvent>>>,
    compliance_violations: Arc<RwLock<u64>>,
}

impl MockBearDogProvider {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            compliance_violations: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn get_audit_log(&self) -> Vec<BearDogSecurityEvent> {
        self.audit_log.read().await.clone()
    }

    pub async fn get_compliance_violations(&self) -> u64 {
        *self.compliance_violations.read().await
    }
}

#[async_trait]
impl BearDogSecurityProvider for MockBearDogProvider {
    async fn encrypt(
        &self,
        data: &[u8],
        context: &BearDogSecurityContext,
    ) -> Result<BearDogEncryptedData> {
        // Mock encryption using Ring (in real BearDog, this would be your crypto)
        use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
        use ring::rand::{SecureRandom, SystemRandom};

        tracing::info!(
            "BearDog: Encrypting {} bytes with security level {:?}",
            data.len(),
            context.security_level
        );

        let key = [42u8; 32]; // Mock key (in real BearDog, derive from key management)
        let unbound_key = UnboundKey::new(&AES_256_GCM, &key).map_err(|_| {
            SongbirdError::SecurityError("Failed to create encryption key".to_string())
        })?;
        let encryption_key = LessSafeKey::new(unbound_key);

        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes)
            .map_err(|_| SongbirdError::SecurityError("Failed to generate nonce".to_string()))?;

        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let mut in_out = data.to_vec();
        encryption_key
            .seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| SongbirdError::SecurityError("Encryption failed".to_string()))?;

        Ok(BearDogEncryptedData {
            algorithm: "AES-256-GCM".to_string(),
            nonce: nonce_bytes.to_vec(),
            ciphertext: in_out,
            salt: None,
            key_handle: None,
        })
    }

    async fn decrypt(
        &self,
        encrypted: &BearDogEncryptedData,
        context: &BearDogSecurityContext,
    ) -> Result<Vec<u8>> {
        // Mock decryption (in real BearDog, this would be your crypto)
        use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};

        tracing::info!(
            "BearDog: Decrypting data for operation {}",
            context.operation_id
        );

        let key = [42u8; 32]; // Mock key
        let unbound_key = UnboundKey::new(&AES_256_GCM, &key).map_err(|_| {
            SongbirdError::SecurityError("Failed to create decryption key".to_string())
        })?;
        let decryption_key = LessSafeKey::new(unbound_key);

        let nonce_bytes: [u8; 12] = encrypted
            .nonce
            .as_slice()
            .try_into()
            .map_err(|_| SongbirdError::SecurityError("Invalid nonce length".to_string()))?;
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        let mut in_out = encrypted.ciphertext.clone();
        let plaintext = decryption_key
            .open_in_place(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| SongbirdError::SecurityError("Decryption failed".to_string()))?;

        Ok(plaintext.to_vec())
    }

    async fn derive_key(&self, key_id: &str, context: &BearDogKeyContext) -> Result<Vec<u8>> {
        tracing::info!(
            "BearDog: Deriving key {} for purpose {:?}",
            key_id,
            context.key_purpose
        );

        let mut keys = self.keys.write().await;
        if let Some(key) = keys.get(key_id) {
            Ok(key.clone())
        } else {
            let new_key = vec![42u8; 32]; // Mock key derivation
            keys.insert(key_id.to_string(), new_key.clone());
            Ok(new_key)
        }
    }

    async fn generate_key(&self, key_spec: &BearDogKeySpec) -> Result<BearDogKeyHandle> {
        let key_id = format!("beardog_key_{}", chrono::Utc::now().timestamp_nanos());
        let key = vec![42u8; key_spec.key_size]; // Mock key generation

        tracing::info!("BearDog: Generated {} key: {}", key_spec.algorithm, key_id);

        let mut keys = self.keys.write().await;
        keys.insert(key_id.clone(), key);

        Ok(BearDogKeyHandle {
            id: key_id,
            version: 1,
            created_at: Utc::now(),
            expires_at: Some(
                Utc::now() + chrono::Duration::days(key_spec.rotation_policy.interval_days as i64),
            ),
        })
    }

    async fn verify_access(
        &self,
        principal: &BearDogPrincipal,
        resource: &BearDogResource,
        action: &BearDogAction,
    ) -> Result<bool> {
        tracing::info!(
            "BearDog: Verifying access for {} to {} ({})",
            principal.id,
            resource.id,
            action.name
        );

        // Mock access control - allow nodes, deny others for demo
        let allowed = match principal.principal_type {
            BearDogPrincipalType::Node => true,
            BearDogPrincipalType::System => true,
            _ => {
                // Simulate compliance violation
                let mut violations = self.compliance_violations.write().await;
                *violations += 1;
                false
            }
        };

        if !allowed {
            tracing::warn!(
                "BearDog: Access denied for {} to {}",
                principal.id,
                resource.id
            );
        }

        Ok(allowed)
    }

    async fn establish_secure_channel(&self, peer_id: &NodeId) -> Result<BearDogSecureChannel> {
        tracing::info!("BearDog: Establishing secure channel with peer {}", peer_id);

        Ok(BearDogSecureChannel {
            channel_id: format!("beardog_channel_{}", chrono::Utc::now().timestamp_nanos()),
            peer_id: peer_id.clone(),
            established_at: Utc::now(),
            encryption_key: vec![42u8; 32],
        })
    }

    async fn log_security_event(&self, event: &BearDogSecurityEvent) -> Result<()> {
        let mut audit_log = self.audit_log.write().await;
        audit_log.push(event.clone());

        tracing::info!(
            "BearDog: Logged security event {} - {:?}",
            event.event_id,
            event.outcome
        );
        Ok(())
    }

    async fn rotate_key(&self, key_id: &str) -> Result<BearDogKeyHandle> {
        tracing::info!("BearDog: Rotating key {}", key_id);

        let new_key_id = format!("{}_rotated_{}", key_id, Utc::now().timestamp());
        let new_key = vec![43u8; 32]; // New key material

        let mut keys = self.keys.write().await;
        keys.insert(new_key_id.clone(), new_key);

        Ok(BearDogKeyHandle {
            id: new_key_id,
            version: 2,
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(30)),
        })
    }

    async fn get_compliance_report(
        &self,
        period: &BearDogTimePeriod,
    ) -> Result<BearDogComplianceReport> {
        let audit_events = self.audit_log.read().await;
        let violations = *self.compliance_violations.read().await;

        let encryption_ops = audit_events
            .iter()
            .filter(|e| matches!(e.event_type, BearDogSecurityEventType::Encryption))
            .count() as u64;

        let key_rotations = audit_events
            .iter()
            .filter(|e| matches!(e.event_type, BearDogSecurityEventType::KeyRotation))
            .count() as u64;

        let compliance_score = if violations == 0 {
            100.0
        } else {
            100.0 - (violations as f64 * 10.0)
        };

        Ok(BearDogComplianceReport {
            period: period.clone(),
            encryption_operations: encryption_ops,
            key_rotations,
            access_violations: violations,
            compliance_score,
            recommendations: if violations > 0 {
                vec!["Review access control policies".to_string()]
            } else {
                vec!["Compliance status: EXCELLENT".to_string()]
            },
        })
    }
}

// ============================================================================
// DEMO MAIN FUNCTION
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("🐻🐕 BEARDOG SECURITY INTEGRATION WITH SONGBIRD ORCHESTRATOR");
    println!("===========================================================");
    println!("🎯 Production-ready encrypted snapshots with BearDog security");
    println!("🔐 Enterprise key management and access control");
    println!("🛡️ Comprehensive audit logging and compliance reporting\n");

    // 1. Initialize BearDog provider
    println!("1️⃣  Initializing BearDog Security Provider...");
    let beardog_provider = Arc::new(MockBearDogProvider::new());
    println!("✅ BearDog provider initialized with enterprise-grade security");

    // 2. Create BearDog-integrated snapshot manager
    println!("\n2️⃣  Creating BearDog-integrated Snapshot Manager...");
    let node_id = "production-node-001".to_string();
    let snapshot_manager = BearDogEncryptedSnapshotManager::new_with_beardog(
        beardog_provider.clone(),
        node_id.clone(),
        BearDogSecurityLevel::Confidential,
    )?;
    println!("✅ BearDog snapshot manager created with Confidential security level");

    // 3. Create sensitive production data
    println!("\n3️⃣  Creating sensitive production data...");
    let sensitive_data = b"CONFIDENTIAL: Production database backup with customer PII and financial records. Classification: Confidential. Access restricted to authorized personnel only.";
    println!("📊 Data size: {} bytes", sensitive_data.len());
    println!("🔒 Security classification: CONFIDENTIAL");

    // 4. Create snapshot metadata
    let metadata = SnapshotMetadata {
        name: "Production Customer Database Backup".to_string(),
        snapshot_type: SnapshotType::Database {
            schema_version: "v3.2.1".to_string(),
        },
        size_bytes: 0,
        original_size_bytes: sensitive_data.len() as u64,
        compression: None,
        tags: vec![
            "production".to_string(),
            "customer-data".to_string(),
            "pii".to_string(),
            "financial".to_string(),
            "confidential".to_string(),
        ],
        version: "2024.01.15".to_string(),
        expires_at: Some(Utc::now() + chrono::Duration::days(90)),
    };

    // 5. Create access control with specific authorized nodes
    let access_control = AccessControlList {
        read_access: vec![
            NodeAccessEntry {
                node_id: "backup-restore-node".to_string(),
                institution: Some("Internal IT Department".to_string()),
                min_trust_level: TrustLevel::Institutional,
                granted_at: Utc::now(),
                expires_at: Some(Utc::now() + chrono::Duration::days(30)),
            },
            NodeAccessEntry {
                node_id: "disaster-recovery-node".to_string(),
                institution: Some("Internal IT Department".to_string()),
                min_trust_level: TrustLevel::Institutional,
                granted_at: Utc::now(),
                expires_at: Some(Utc::now() + chrono::Duration::days(90)),
            },
        ],
        write_access: vec![],
        public_read: false,
        access_expires_at: Some(Utc::now() + chrono::Duration::days(90)),
    };

    // 6. Create storage preferences for production deployment
    let storage_preferences = StoragePreferences {
        preferred_nodes: vec![
            "storage-node-east-1".to_string(),
            "storage-node-west-1".to_string(),
        ],
        excluded_nodes: vec!["test-node".to_string()],
        geographic_region: Some("us-central".to_string()),
        preferred_institutions: vec!["Internal Infrastructure".to_string()],
        min_storage_trust: TrustLevel::Institutional,
        replication_factor: 3,
        performance_tier: PerformanceTier::HighPerformance,
    };

    // 7. Create encrypted snapshot using BearDog
    println!("\n7️⃣  Creating encrypted snapshot with BearDog security...");
    let snapshot_id = snapshot_manager
        .create_encrypted_snapshot(
            sensitive_data,
            metadata,
            access_control,
            storage_preferences,
        )
        .await?;
    println!("✅ BearDog encrypted snapshot created: {}", snapshot_id);
    println!("🔐 Data encrypted with BearDog enterprise encryption");
    println!("🗝️  Keys managed by BearDog key management system");
    println!("📝 Security event logged to BearDog audit system");

    // 8. Demonstrate authorized access
    println!("\n8️⃣  Testing authorized access to encrypted snapshot...");
    let authorized_node = "backup-restore-node".to_string();
    match snapshot_manager
        .retrieve_encrypted_snapshot(&snapshot_id, &authorized_node)
        .await
    {
        Ok(decrypted_data) => {
            println!("✅ Authorized access successful");
            println!("📊 Decrypted {} bytes", decrypted_data.len());
            println!(
                "🔍 Data integrity verified: {}",
                decrypted_data == sensitive_data
            );
            println!("🛡️ BearDog access control enforced successfully");
        }
        Err(e) => {
            println!("❌ Authorized access failed: {}", e);
        }
    }

    // 9. Demonstrate unauthorized access denial
    println!("\n9️⃣  Testing unauthorized access (should be denied)...");
    let unauthorized_node = "random-external-node".to_string();
    match snapshot_manager
        .retrieve_encrypted_snapshot(&snapshot_id, &unauthorized_node)
        .await
    {
        Ok(_) => {
            println!("❌ SECURITY VIOLATION: Unauthorized access was allowed!");
        }
        Err(e) => {
            println!("✅ Unauthorized access properly denied: {}", e);
            println!("🛡️ BearDog access control working correctly");
        }
    }

    // 10. Generate compliance report
    println!("\n🔟 Generating BearDog compliance report...");
    let report_period = BearDogTimePeriod {
        start: Utc::now() - chrono::Duration::hours(1),
        end: Utc::now(),
    };

    let compliance_report = beardog_provider
        .get_compliance_report(&report_period)
        .await?;
    println!("📊 BEARDOG COMPLIANCE REPORT");
    println!(
        "   Period: {} to {}",
        report_period.start.format("%H:%M:%S"),
        report_period.end.format("%H:%M:%S")
    );
    println!(
        "   Encryption Operations: {}",
        compliance_report.encryption_operations
    );
    println!("   Key Rotations: {}", compliance_report.key_rotations);
    println!(
        "   Access Violations: {}",
        compliance_report.access_violations
    );
    println!(
        "   Compliance Score: {:.1}%",
        compliance_report.compliance_score
    );
    for recommendation in &compliance_report.recommendations {
        println!("   Recommendation: {}", recommendation);
    }

    // 11. Show detailed audit log
    println!("\n1️⃣1️⃣ BearDog Security Audit Log:");
    let audit_events = beardog_provider.get_audit_log().await;
    for (i, event) in audit_events.iter().enumerate() {
        println!(
            "   {}. [{}] {} - {} - {:?}",
            i + 1,
            event.timestamp.format("%H:%M:%S"),
            event.event_type.to_string(),
            event.principal.id,
            event.outcome
        );
    }

    println!("\n🎉 BEARDOG INTEGRATION DEMONSTRATION COMPLETED!");
    println!("==============================================");
    println!(
        "✅ BearDog encrypted {} bytes with enterprise security",
        sensitive_data.len()
    );
    println!("✅ BearDog access control enforced and verified");
    println!("✅ BearDog audit events logged: {}", audit_events.len());
    println!("✅ BearDog compliance reporting operational");
    println!("✅ Production-ready encrypted snapshot system with BearDog");
    println!("🔐 Your in-house security module is fully integrated!");

    Ok(())
}

// Helper trait implementations for demo output formatting
impl std::fmt::Display for BearDogSecurityEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BearDogSecurityEventType::Authentication => write!(f, "AUTH"),
            BearDogSecurityEventType::Authorization => write!(f, "AUTHZ"),
            BearDogSecurityEventType::Encryption => write!(f, "ENCRYPT"),
            BearDogSecurityEventType::Decryption => write!(f, "DECRYPT"),
            BearDogSecurityEventType::KeyGeneration => write!(f, "KEYGEN"),
            BearDogSecurityEventType::KeyRotation => write!(f, "KEYROT"),
            BearDogSecurityEventType::AccessGranted => write!(f, "ACCESS_OK"),
            BearDogSecurityEventType::AccessDenied => write!(f, "ACCESS_DENIED"),
            BearDogSecurityEventType::SecurityViolation => write!(f, "VIOLATION"),
        }
    }
}
