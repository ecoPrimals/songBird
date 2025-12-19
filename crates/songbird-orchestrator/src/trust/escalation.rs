//! Trust Escalation Manager
//!
//! Manages progressive trust escalation from anonymous to hardware-verified.

use super::types::{
    CapabilityProof, HardwareAttestation, IdentityProof, TrustLevel,
    TrustRelationship,
};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Trust escalation manager
///
/// Manages trust relationships between towers and handles progressive trust escalation.
#[derive(Clone)]
pub struct TrustEscalationManager {
    /// Current trust relationships (session_id -> relationship)
    trust_store: Arc<RwLock<HashMap<String, TrustRelationship>>>,

    /// Trust timeouts for each level (in seconds)
    trust_timeouts: TrustTimeouts,

    /// BearDog integration for hardware verification (optional)
    #[allow(dead_code)]
    beardog_client: Option<Arc<BearDogClient>>,
}

impl std::fmt::Debug for TrustEscalationManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrustEscalationManager")
            .field("trust_timeouts", &self.trust_timeouts)
            .field("trust_store_count", &"<locked>")
            .finish()
    }
}

/// Trust timeouts for different trust levels
#[derive(Debug, Clone)]
pub struct TrustTimeouts {
    /// Anonymous sessions expire after this (default: 3600 = 1 hour)
    pub anonymous: u64,
    /// Capability sessions expire after this (default: 86400 = 24 hours)
    pub capability: u64,
    /// Identity sessions expire after this (default: 604800 = 7 days)
    pub identity: u64,
    /// Hardware sessions never expire (default: 0 = never)
    pub hardware: u64,
}

impl Default for TrustTimeouts {
    fn default() -> Self {
        Self {
            anonymous: 3600,      // 1 hour
            capability: 86400,    // 24 hours
            identity: 604800,     // 7 days
            hardware: 0,          // Never expire
        }
    }
}

/// BearDog client for hardware verification (placeholder)
///
/// TODO: Implement actual BearDog integration
pub struct BearDogClient;

impl BearDogClient {
    /// Verify a hardware key via BearDog
    ///
    /// TODO: Implement actual BearDog verification
    #[allow(dead_code)]
    pub async fn verify_hardware_key(&self, _hardware_key: &str) -> Result<bool> {
        // Placeholder: Always verify successfully
        // In production, this should call BearDog API
        Ok(true)
    }
}

impl TrustEscalationManager {
    /// Create a new trust escalation manager
    #[must_use]
    pub fn new(trust_timeouts: TrustTimeouts, beardog_client: Option<Arc<BearDogClient>>) -> Self {
        Self {
            trust_store: Arc::new(RwLock::new(HashMap::new())),
            trust_timeouts,
            beardog_client,
        }
    }

    /// Create with default timeouts
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(TrustTimeouts::default(), None)
    }

    /// Establish initial anonymous trust
    ///
    /// This is the entry point for all new connections.
    /// No verification is required - we simply record that we've seen this session.
    pub async fn establish_anonymous(&self, session_id: String) -> Result<()> {
        let relationship = TrustRelationship::new_anonymous(session_id.clone(), self.trust_timeouts.anonymous);

        self.trust_store.write().await.insert(session_id.clone(), relationship);

        info!("✅ Anonymous trust established (Level 0): {}", session_id);
        Ok(())
    }

    /// Escalate to capability-verified
    ///
    /// Verifies cryptographic proof of capabilities and grants task coordination access.
    pub async fn verify_capabilities(&self, session_id: &str, proof: CapabilityProof) -> Result<()> {
        let mut store = self.trust_store.write().await;
        let relationship = store
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        // Verify cryptographic proof of capabilities
        if !proof.verify() {
            return Err(anyhow!("Capability proof verification failed"));
        }

        // Escalate trust level
        relationship.trust_level = TrustLevel::CapabilityVerified;
        relationship.verified_capabilities = proof.capabilities.clone();
        relationship.last_verified_at = SystemTime::now();
        relationship.expires_at = SystemTime::now()
            + std::time::Duration::from_secs(self.trust_timeouts.capability);

        info!(
            "✅ Trust escalated to Capability-Verified (Level 1): {}",
            session_id
        );
        debug!("   Capabilities: {:?}", proof.capabilities);

        Ok(())
    }

    /// Escalate to role-verified
    ///
    /// Verifies role-based access and grants service registry access.
    pub async fn verify_role(&self, session_id: &str, role: String) -> Result<()> {
        let mut store = self.trust_store.write().await;
        let relationship = store
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        // Must be at least capability-verified to escalate to role-verified
        if relationship.trust_level < TrustLevel::CapabilityVerified {
            return Err(anyhow!(
                "Cannot escalate to role-verified from {:?}",
                relationship.trust_level
            ));
        }

        // TODO: Implement actual role verification
        // For now, we accept any non-empty role
        if role.is_empty() {
            return Err(anyhow!("Role cannot be empty"));
        }

        // Escalate trust level
        relationship.trust_level = TrustLevel::RoleVerified;
        relationship.last_verified_at = SystemTime::now();
        relationship.expires_at = SystemTime::now()
            + std::time::Duration::from_secs(self.trust_timeouts.identity);

        info!("✅ Trust escalated to Role-Verified (Level 2): {}", session_id);
        debug!("   Role: {}", role);

        Ok(())
    }

    /// Escalate to identity-verified
    ///
    /// Verifies identity proof (JWT, certificate, etc.) and grants infrastructure access.
    pub async fn verify_identity(&self, session_id: &str, identity_proof: IdentityProof) -> Result<()> {
        let mut store = self.trust_store.write().await;
        let relationship = store
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        // Must be at least role-verified to escalate to identity-verified
        if relationship.trust_level < TrustLevel::RoleVerified {
            return Err(anyhow!(
                "Cannot escalate to identity-verified from {:?}",
                relationship.trust_level
            ));
        }

        // Verify identity proof (JWT, certificate, etc.)
        if !identity_proof.verify() {
            return Err(anyhow!("Identity proof verification failed"));
        }

        // Escalate trust level
        relationship.trust_level = TrustLevel::IdentityVerified;
        relationship.identity = Some(identity_proof.identity.clone());
        relationship.last_verified_at = SystemTime::now();
        relationship.expires_at = SystemTime::now()
            + std::time::Duration::from_secs(self.trust_timeouts.identity);

        info!(
            "✅ Trust escalated to Identity-Verified (Level 3): {}",
            session_id
        );
        info!("   Identity: {}", identity_proof.identity.node_id);

        Ok(())
    }

    /// Escalate to hardware-verified (requires BearDog)
    ///
    /// Verifies hardware key via BearDog and grants full admin access.
    pub async fn verify_hardware(&self, session_id: &str, hardware_proof: HardwareAttestation) -> Result<()> {
        let beardog = self
            .beardog_client
            .as_ref()
            .ok_or_else(|| anyhow!("BearDog integration not configured"))?;

        // Verify hardware key via BearDog
        if !beardog.verify_hardware_key(&hardware_proof.hardware_key).await? {
            return Err(anyhow!("Hardware attestation failed"));
        }

        let mut store = self.trust_store.write().await;
        let relationship = store
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        // Must be at least identity-verified to escalate to hardware-verified
        if relationship.trust_level < TrustLevel::IdentityVerified {
            return Err(anyhow!(
                "Cannot escalate to hardware-verified from {:?}",
                relationship.trust_level
            ));
        }

        // Escalate trust level
        relationship.trust_level = TrustLevel::HardwareVerified;
        relationship.hardware_proof = Some(hardware_proof.clone());
        relationship.last_verified_at = SystemTime::now();

        // Hardware-verified sessions never expire (unless explicitly revoked)
        if self.trust_timeouts.hardware > 0 {
            relationship.expires_at = SystemTime::now()
                + std::time::Duration::from_secs(self.trust_timeouts.hardware);
        } else {
            // Never expire (set to far future)
            relationship.expires_at = SystemTime::now()
                + std::time::Duration::from_secs(u64::MAX / 2);
        }

        info!(
            "🔒 Trust escalated to Hardware-Verified (Level 4 - ADMIN): {}",
            session_id
        );
        info!("   Hardware Key: {}", hardware_proof.hardware_key);

        Ok(())
    }

    /// Check if a session can perform an operation requiring a minimum trust level
    pub async fn check_permission(&self, session_id: &str, required_level: TrustLevel) -> Result<bool> {
        let store = self.trust_store.read().await;
        let relationship = store
            .get(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        if relationship.is_expired() {
            warn!(
                "Trust relationship expired: {} (level: {:?})",
                session_id, relationship.trust_level
            );
            return Ok(false);
        }

        Ok(relationship.trust_level.can_perform(required_level))
    }

    /// Get current trust level for a session
    pub async fn get_trust_level(&self, session_id: &str) -> Result<TrustLevel> {
        let store = self.trust_store.read().await;
        let relationship = store
            .get(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        if relationship.is_expired() {
            // Return Anonymous if expired
            return Ok(TrustLevel::Anonymous);
        }

        Ok(relationship.trust_level)
    }

    /// Get trust relationship details
    pub async fn get_relationship(&self, session_id: &str) -> Option<TrustRelationship> {
        let store = self.trust_store.read().await;
        store.get(session_id).cloned()
    }

    /// Remove a trust relationship (revoke trust)
    pub async fn revoke_trust(&self, session_id: &str) -> Result<()> {
        let mut store = self.trust_store.write().await;
        store
            .remove(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        info!("🗑️  Trust revoked: {}", session_id);
        Ok(())
    }

    /// Cleanup expired trust relationships (should be called periodically)
    pub async fn cleanup_expired(&self) -> usize {
        let mut store = self.trust_store.write().await;
        let initial_count = store.len();

        store.retain(|session_id, relationship| {
            if relationship.is_expired() {
                debug!("🗑️  Removing expired trust relationship: {}", session_id);
                false
            } else {
                true
            }
        });

        let removed = initial_count - store.len();
        if removed > 0 {
            info!("🧹 Cleaned up {} expired trust relationships", removed);
        }

        removed
    }

    /// Get all active trust relationships
    pub async fn get_all_relationships(&self) -> Vec<(String, TrustRelationship)> {
        let store = self.trust_store.read().await;
        store
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Get count of relationships at each trust level
    pub async fn get_trust_level_counts(&self) -> HashMap<TrustLevel, usize> {
        let store = self.trust_store.read().await;
        let mut counts = HashMap::new();

        for relationship in store.values() {
            if !relationship.is_expired() {
                *counts.entry(relationship.trust_level).or_insert(0) += 1;
            }
        }

        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trust_escalation_manager_creation() {
        let manager = TrustEscalationManager::with_defaults();
        let counts = manager.get_trust_level_counts().await;
        assert!(counts.is_empty());
    }

    #[tokio::test]
    async fn test_establish_anonymous_trust() {
        let manager = TrustEscalationManager::with_defaults();
        let session_id = "test-session".to_string();

        manager.establish_anonymous(session_id.clone()).await.unwrap();

        let level = manager.get_trust_level(&session_id).await.unwrap();
        assert_eq!(level, TrustLevel::Anonymous);
    }

    #[tokio::test]
    async fn test_verify_capabilities() {
        let manager = TrustEscalationManager::with_defaults();
        let session_id = "test-session".to_string();

        // Establish anonymous trust first
        manager.establish_anonymous(session_id.clone()).await.unwrap();

        // Create capability proof
        let proof = CapabilityProof {
            capabilities: vec!["orchestration".to_string()],
            proof: "test-proof".to_string(),
            timestamp: SystemTime::now(),
        };

        // Verify capabilities
        manager.verify_capabilities(&session_id, proof).await.unwrap();

        let level = manager.get_trust_level(&session_id).await.unwrap();
        assert_eq!(level, TrustLevel::CapabilityVerified);
    }

    #[tokio::test]
    async fn test_verify_identity() {
        let manager = TrustEscalationManager::with_defaults();
        let session_id = "test-session".to_string();

        // Establish anonymous trust
        manager.establish_anonymous(session_id.clone()).await.unwrap();

        // Escalate to capability-verified
        let cap_proof = CapabilityProof {
            capabilities: vec!["orchestration".to_string()],
            proof: "test-proof".to_string(),
            timestamp: SystemTime::now(),
        };
        manager.verify_capabilities(&session_id, cap_proof).await.unwrap();

        // Escalate to role-verified
        manager.verify_role(&session_id, "admin".to_string()).await.unwrap();

        // Escalate to identity-verified
        let identity = TowerIdentity {
            node_id: "test-node".to_string(),
            hostname: "test-host".to_string(),
            organization: None,
            public_key: None,
        };

        let identity_proof = IdentityProof {
            identity,
            proof: "test-proof".to_string(),
            proof_type: "jwt".to_string(),
            timestamp: SystemTime::now(),
        };

        manager.verify_identity(&session_id, identity_proof).await.unwrap();

        let level = manager.get_trust_level(&session_id).await.unwrap();
        assert_eq!(level, TrustLevel::IdentityVerified);
    }

    #[tokio::test]
    async fn test_check_permission() {
        let manager = TrustEscalationManager::with_defaults();
        let session_id = "test-session".to_string();

        // Establish capability-verified trust
        manager.establish_anonymous(session_id.clone()).await.unwrap();
        let proof = CapabilityProof {
            capabilities: vec!["orchestration".to_string()],
            proof: "test-proof".to_string(),
            timestamp: SystemTime::now(),
        };
        manager.verify_capabilities(&session_id, proof).await.unwrap();

        // Should be able to perform anonymous and capability operations
        assert!(manager
            .check_permission(&session_id, TrustLevel::Anonymous)
            .await
            .unwrap());
        assert!(manager
            .check_permission(&session_id, TrustLevel::CapabilityVerified)
            .await
            .unwrap());

        // Should NOT be able to perform identity operations
        assert!(!manager
            .check_permission(&session_id, TrustLevel::IdentityVerified)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_revoke_trust() {
        let manager = TrustEscalationManager::with_defaults();
        let session_id = "test-session".to_string();

        manager.establish_anonymous(session_id.clone()).await.unwrap();
        manager.revoke_trust(&session_id).await.unwrap();

        // Should fail to get trust level after revocation
        assert!(manager.get_trust_level(&session_id).await.is_err());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let mut timeouts = TrustTimeouts::default();
        timeouts.anonymous = 0; // Expire immediately

        let manager = TrustEscalationManager::new(timeouts, None);
        let session_id = "test-session".to_string();

        manager.establish_anonymous(session_id.clone()).await.unwrap();

        // Wait a moment to ensure expiration
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let removed = manager.cleanup_expired().await;
        assert_eq!(removed, 1);
    }
}

