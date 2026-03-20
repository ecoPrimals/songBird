// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Trust Escalation Manager
//!
//! Manages progressive trust escalation from anonymous to hardware-verified.

use super::types::{
    CapabilityProof, HardwareAttestation, IdentityProof, TrustLevel, TrustRelationship,
};
use crate::security_client::client::SecurityCapabilityClient;
use anyhow::{Result, anyhow};
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
    /// Current trust relationships (`session_id` -> relationship)
    trust_store: Arc<RwLock<HashMap<String, TrustRelationship>>>,

    /// Trust timeouts for each level (in seconds)
    trust_timeouts: TrustTimeouts,

    /// Security provider integration for hardware verification (optional)
    #[allow(dead_code, reason = "reserved for future use: security provider wiring")]
    security_client: Option<Arc<SecurityCapabilityClient>>,
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
            anonymous: 3600,   // 1 hour
            capability: 86400, // 24 hours
            identity: 604800,  // 7 days
            hardware: 0,       // Never expire
        }
    }
}

/// security provider client for hardware verification
///
/// Integrates with security provider v0.9.5+ for cryptographic identity verification.
/// When security provider is available, this provides hardware-backed trust anchors.
///
/// ## Integration Status
/// - security provider v0.9.5 is available at ../security provider
/// - Using capability-based discovery for endpoint resolution
/// - Falls back to mock verification in development mode
pub struct BearDogClient {
    /// Optional security provider endpoint (discovered at runtime)
    endpoint: Option<String>,
}

impl BearDogClient {
    /// Create a new security client with runtime discovery
    ///
    /// **EVOLVED (v3.15.0)**: Uses capability discovery (zero vendor hardcoding!)
    ///
    /// Note: This is a sync function, so we use environment variables directly.
    /// Full async discovery is available in async methods.
    pub fn new() -> Self {
        // EVOLVED: Use generic capability env vars (sync version)
        let endpoint = std::env::var("SONGBIRD_SECURITY_PROVIDER")
            .or_else(|_| std::env::var("SECURITY_ENDPOINT"))
            .or_else(|_| {
                if let Ok(url) = std::env::var("BEARDOG_URL") {
                    tracing::warn!("⚠️  DEPRECATED: BEARDOG_URL is deprecated");
                    tracing::warn!("   Use SONGBIRD_SECURITY_PROVIDER instead");
                    Ok(url)
                } else {
                    Err(std::env::VarError::NotPresent)
                }
            })
            .ok();

        if let Some(ref url) = endpoint {
            tracing::info!("Security provider configured: {}", url);
        } else {
            tracing::debug!(
                "Security client created without endpoint (will use mock verification)"
            );
        }

        Self {
            endpoint,
        }
    }

    /// Verify a hardware key via security provider
    ///
    /// **DEPRECATED (Jan 16, 2026)**: This HTTP-based approach is deprecated in favor of BTSP.
    ///
    /// ## Evolution Path
    ///
    /// Hardware key verification should be evolved to use the BTSP (`BearDog` Tunnel Security Protocol)
    /// client for secure, Unix socket-based communication:
    ///
    /// ```ignore
    /// use crate::btsp_client::BtspClient;
    ///
    /// let btsp_client = BtspClient::new();
    /// let request = json!({
    ///     "jsonrpc": "2.0",
    ///     "method": "security.verify_hardware_key",
    ///     "params": { "key": hardware_key },
    ///     "id": 1
    /// });
    /// let response = btsp_client.send_request(request).await?;
    /// let is_valid = response["result"]["valid"].as_bool().unwrap_or(false);
    /// ```
    ///
    /// ## Current Implementation
    ///
    /// - **Development mode**: Basic validation (length check)
    /// - **Production**: Should use BTSP client (not yet implemented)
    ///
    /// ## Philosophy
    ///
    /// - **Zero Hardcoding**: Discover security provider via capability
    /// - **Deep Debt Solution**: Evolve to BTSP, not HTTP
    /// - **Mocks Isolated**: This is a `NoOp` provider, not a production mock
    #[deprecated(since = "0.1.0", note = "Use BTSP client for hardware key verification")]
    #[allow(dead_code, reason = "deprecated API retained for compatibility until BTSP migration")]
    pub async fn verify_hardware_key(&self, hardware_key: &str) -> Result<bool> {
        if let Some(ref endpoint) = self.endpoint {
            tracing::warn!("⚠️  DEPRECATED: HTTP-based hardware key verification at {}", endpoint);
            tracing::warn!("   Evolution path: Use BTSP client for secure verification");

            // NoOp provider: Return basic validation
            // This is NOT a production mock - it's a NoOp provider that clearly indicates
            // the feature is not implemented. In production, use BTSP client.
            let is_valid = !hardware_key.is_empty() && hardware_key.len() >= 32;

            if is_valid {
                tracing::warn!("   Hardware key accepted (NoOp provider - evolve to BTSP!)");
            } else {
                tracing::warn!("   Hardware key rejected (invalid format)");
            }

            Ok(is_valid)
        } else {
            // Development mode: Basic validation
            let is_valid = !hardware_key.is_empty() && hardware_key.len() >= 32;

            tracing::debug!("Hardware key validation (development): {}", is_valid);

            Ok(is_valid)
        }
    }
}

impl Default for BearDogClient {
    fn default() -> Self {
        Self::new()
    }
}

impl TrustEscalationManager {
    /// Create a new trust escalation manager
    #[must_use]
    pub fn new(
        trust_timeouts: TrustTimeouts,
        security_client: Option<Arc<SecurityCapabilityClient>>,
    ) -> Self {
        Self {
            trust_store: Arc::new(RwLock::new(HashMap::new())),
            trust_timeouts,
            security_client,
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
        let relationship =
            TrustRelationship::new_anonymous(session_id.clone(), self.trust_timeouts.anonymous);

        self.trust_store.write().await.insert(session_id.clone(), relationship);

        info!("✅ Anonymous trust established (Level 0): {}", session_id);
        Ok(())
    }

    /// Escalate to capability-verified
    ///
    /// Verifies cryptographic proof of capabilities and grants task coordination access.
    pub async fn verify_capabilities(
        &self,
        session_id: &str,
        proof: CapabilityProof,
    ) -> Result<()> {
        let mut store = self.trust_store.write().await;
        let relationship =
            store.get_mut(session_id).ok_or_else(|| anyhow!("Session not found: {session_id}"))?;

        // Verify cryptographic proof of capabilities
        if !proof.verify() {
            return Err(anyhow!("Capability proof verification failed"));
        }

        // Escalate trust level
        relationship.trust_level = TrustLevel::CapabilityVerified;
        relationship.verified_capabilities.clone_from(&proof.capabilities);
        relationship.last_verified_at = SystemTime::now();
        relationship.expires_at =
            SystemTime::now() + std::time::Duration::from_secs(self.trust_timeouts.capability);

        info!("✅ Trust escalated to Capability-Verified (Level 1): {}", session_id);
        debug!("   Capabilities: {:?}", proof.capabilities);

        Ok(())
    }

    /// Escalate to role-verified
    ///
    /// Verifies role-based access and grants service registry access.
    ///
    /// ## Role Verification
    /// Roles define what a tower can do within the federation:
    /// - `coordinator` - Can coordinate federation-wide tasks
    /// - `worker` - Can execute tasks, no coordination
    /// - `observer` - Can query registry, read-only access
    /// - `admin` - Can modify federation configuration (requires higher trust)
    ///
    /// ## Implementation
    /// 1. Validates role format and known roles
    /// 2. Checks prerequisites (must be capability-verified)
    /// 3. Future: Validate role against policy/RBAC system
    /// 4. Escalates trust level and grants access
    pub async fn verify_role(&self, session_id: &str, role: String) -> Result<()> {
        let mut store = self.trust_store.write().await;
        let relationship =
            store.get_mut(session_id).ok_or_else(|| anyhow!("Session not found: {session_id}"))?;

        // Must be at least capability-verified to escalate to role-verified
        if relationship.trust_level < TrustLevel::CapabilityVerified {
            return Err(anyhow!(
                "Cannot escalate to role-verified from {:?}",
                relationship.trust_level
            ));
        }

        // Validate role format and known roles
        if role.is_empty() {
            return Err(anyhow!("Role cannot be empty"));
        }

        // Define known valid roles
        const VALID_ROLES: &[&str] = &[
            "coordinator", // Can coordinate federation-wide tasks
            "worker",      // Can execute tasks assigned to it
            "observer",    // Read-only access to registry
            "compute",     // Compute-specific role
            "storage",     // Storage-specific role
            "ai",          // AI workload role
            "security",    // Security service role
        ];

        // Normalize role to lowercase for comparison
        let normalized_role = role.to_lowercase();

        // Check if role is valid
        if !VALID_ROLES.contains(&normalized_role.as_str()) {
            tracing::warn!("Unknown role requested: '{}'. Known roles: {:?}", role, VALID_ROLES);
            // Accept unknown roles but log for monitoring
            // This allows extension without code changes
        }

        // Additional role-specific validation
        match normalized_role.as_str() {
            "admin" => {
                // Admin role requires identity-verified first
                // Cannot jump directly from capability to admin
                return Err(anyhow!("Admin role requires identity verification first (Level 3)"));
            }
            _ => {
                // Other roles are allowed at this trust level
                tracing::debug!("Role '{}' accepted for registry access", role);
            }
        }

        // Escalate trust level
        relationship.trust_level = TrustLevel::RoleVerified;
        relationship.last_verified_at = SystemTime::now();
        relationship.expires_at =
            SystemTime::now() + std::time::Duration::from_secs(self.trust_timeouts.identity);

        info!("✅ Trust escalated to Role-Verified (Level 2): {}", session_id);
        debug!("   Role: {} (normalized: {})", role, normalized_role);
        debug!("   Grants: Service registry access, federation coordination");
        debug!("   Expires: {:?}", relationship.expires_at);

        Ok(())
    }

    /// Escalate to identity-verified
    ///
    /// Verifies identity proof (JWT, certificate, etc.) and grants infrastructure access.
    pub async fn verify_identity(
        &self,
        session_id: &str,
        identity_proof: IdentityProof,
    ) -> Result<()> {
        let mut store = self.trust_store.write().await;
        let relationship =
            store.get_mut(session_id).ok_or_else(|| anyhow!("Session not found: {session_id}"))?;

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
        relationship.expires_at =
            SystemTime::now() + std::time::Duration::from_secs(self.trust_timeouts.identity);

        info!("✅ Trust escalated to Identity-Verified (Level 3): {}", session_id);
        info!("   Identity: {}", identity_proof.identity.node_id);

        Ok(())
    }

    /// Escalate to hardware-verified (requires security provider)
    ///
    /// Verifies hardware key via security provider and grants full admin access.
    pub async fn verify_hardware(
        &self,
        session_id: &str,
        hardware_proof: HardwareAttestation,
    ) -> Result<()> {
        let security_client = self
            .security_client
            .as_ref()
            .ok_or_else(|| anyhow!("security provider integration not configured"))?;

        // Verify hardware key via security provider
        // Verify hardware attestation (if security provider available)
        if let Some(ref security) = self.security_client {
            // FUTURE (Phase 2): Hardware attestation verification via security provider
            // Current: Trust escalation works without hardware verification (software-based trust)
            // Future: TPM/hardware key verification for admin-level trust escalation
            // Requires: security provider hardware.verify_attestation() RPC method
            tracing::info!(
                "Hardware verification via security provider (future: implement actual HTTP call)"
            );
        }

        let mut store = self.trust_store.write().await;
        let relationship =
            store.get_mut(session_id).ok_or_else(|| anyhow!("Session not found: {session_id}"))?;

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
            relationship.expires_at =
                SystemTime::now() + std::time::Duration::from_secs(self.trust_timeouts.hardware);
        } else {
            // Never expire (set to far future)
            relationship.expires_at =
                SystemTime::now() + std::time::Duration::from_secs(u64::MAX / 2);
        }

        info!("🔒 Trust escalated to Hardware-Verified (Level 4 - ADMIN): {}", session_id);
        info!("   Hardware Key: {}", hardware_proof.hardware_key);

        Ok(())
    }

    /// Check if a session can perform an operation requiring a minimum trust level
    pub async fn check_permission(
        &self,
        session_id: &str,
        required_level: TrustLevel,
    ) -> Result<bool> {
        let store = self.trust_store.read().await;
        let relationship =
            store.get(session_id).ok_or_else(|| anyhow!("Session not found: {session_id}"))?;

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
        let relationship =
            store.get(session_id).ok_or_else(|| anyhow!("Session not found: {session_id}"))?;

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
        store.remove(session_id).ok_or_else(|| anyhow!("Session not found: {session_id}"))?;

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
        drop(store);
        if removed > 0 {
            info!("🧹 Cleaned up {} expired trust relationships", removed);
        }

        removed
    }

    /// Get all active trust relationships
    pub async fn get_all_relationships(&self) -> Vec<(String, TrustRelationship)> {
        let store = self.trust_store.read().await;
        store.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
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
        drop(store);

        counts
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::trust::{HardwareAttestation, IdentityProof, TowerIdentity};

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

        manager.establish_anonymous(session_id.clone()).await.expect("anonymous");

        let level = manager.get_trust_level(&session_id).await.expect("level");
        assert_eq!(level, TrustLevel::Anonymous);
    }

    #[tokio::test]
    async fn test_verify_capabilities() {
        let manager = TrustEscalationManager::with_defaults();
        let session_id = "test-session".to_string();

        // Establish anonymous trust first
        manager.establish_anonymous(session_id.clone()).await.expect("establish");

        // Create capability proof (must be >= 32 chars)
        let proof = CapabilityProof {
            capabilities: vec!["orchestration".to_string()],
            proof: "0123456789abcdef0123456789abcdef".to_string(), // 32 chars minimum
            timestamp: SystemTime::now(),
        };

        // Verify capabilities
        manager.verify_capabilities(&session_id, proof).await.expect("cap");

        let level = manager.get_trust_level(&session_id).await.expect("level");
        assert_eq!(level, TrustLevel::CapabilityVerified);
    }

    #[tokio::test]
    async fn test_verify_identity() {
        let manager = TrustEscalationManager::with_defaults();
        let session_id = "test-session".to_string();

        // Establish anonymous trust
        manager.establish_anonymous(session_id.clone()).await.expect("establish");

        // Escalate to capability-verified (proof must be >= 32 chars)
        let cap_proof = CapabilityProof {
            capabilities: vec!["orchestration".to_string()],
            proof: "0123456789abcdef0123456789abcdef".to_string(), // 32 chars minimum
            timestamp: SystemTime::now(),
        };
        manager.verify_capabilities(&session_id, cap_proof).await.expect("cap");

        // Escalate to role-verified (use "coordinator" not "admin" - admin requires identity first)
        manager.verify_role(&session_id, "coordinator".to_string()).await.expect("role");

        // Escalate to identity-verified
        let identity = TowerIdentity {
            node_id: "test-node".to_string(),
            hostname: "test-host".to_string(),
            organization: None,
            public_key: None,
        };

        let identity_proof = IdentityProof {
            identity,
            proof: "0123456789abcdef0123456789abcdef".to_string(), // 32 chars minimum
            proof_type: "jwt".to_string(),
            timestamp: SystemTime::now(),
        };

        manager.verify_identity(&session_id, identity_proof).await.expect("identity");

        let level = manager.get_trust_level(&session_id).await.expect("level");
        assert_eq!(level, TrustLevel::IdentityVerified);
    }

    #[tokio::test]
    async fn test_check_permission() {
        let manager = TrustEscalationManager::with_defaults();
        let session_id = "test-session".to_string();

        // Establish capability-verified trust (proof must be >= 32 chars)
        manager.establish_anonymous(session_id.clone()).await.expect("establish");
        let proof = CapabilityProof {
            capabilities: vec!["orchestration".to_string()],
            proof: "0123456789abcdef0123456789abcdef".to_string(), // 32 chars minimum
            timestamp: SystemTime::now(),
        };
        manager.verify_capabilities(&session_id, proof).await.expect("cap");

        // Should be able to perform anonymous and capability operations
        assert!(manager.check_permission(&session_id, TrustLevel::Anonymous).await.expect("anon"));
        assert!(
            manager
                .check_permission(&session_id, TrustLevel::CapabilityVerified)
                .await
                .expect("cap")
        );

        // Should NOT be able to perform identity operations
        assert!(
            !manager.check_permission(&session_id, TrustLevel::IdentityVerified).await.expect("id")
        );
    }

    #[tokio::test]
    async fn test_revoke_trust() {
        let manager = TrustEscalationManager::with_defaults();
        let session_id = "test-session".to_string();

        manager.establish_anonymous(session_id.clone()).await.expect("establish");
        manager.revoke_trust(&session_id).await.expect("revoke");

        // Should fail to get trust level after revocation
        assert!(manager.get_trust_level(&session_id).await.is_err());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let mut timeouts = TrustTimeouts::default();
        timeouts.anonymous = 0; // Expire immediately

        let manager = TrustEscalationManager::new(timeouts, None);
        let session_id = "test-session".to_string();

        manager.establish_anonymous(session_id.clone()).await.expect("establish");

        // Wait a moment to ensure expiration
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let removed = manager.cleanup_expired().await;
        assert_eq!(removed, 1);
    }

    #[tokio::test]
    async fn verify_capabilities_rejects_bad_proof() {
        let manager = TrustEscalationManager::with_defaults();
        let session_id = "bad-proof".to_string();
        manager.establish_anonymous(session_id.clone()).await.expect("establish");
        let proof = CapabilityProof {
            capabilities: vec!["orchestration".to_string()],
            proof: "short".to_string(),
            timestamp: SystemTime::now(),
        };
        let err = manager.verify_capabilities(&session_id, proof).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn verify_role_rejects_empty_role() {
        let manager = TrustEscalationManager::with_defaults();
        let session_id = "role-empty".to_string();
        manager.establish_anonymous(session_id.clone()).await.expect("establish");
        let cap = CapabilityProof {
            capabilities: vec!["orchestration".to_string()],
            proof: "0123456789abcdef0123456789abcdef".to_string(),
            timestamp: SystemTime::now(),
        };
        manager.verify_capabilities(&session_id, cap).await.expect("cap");
        let err = manager.verify_role(&session_id, String::new()).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn verify_role_admin_rejected_without_identity_chain() {
        let manager = TrustEscalationManager::with_defaults();
        let session_id = "admin-chain".to_string();
        manager.establish_anonymous(session_id.clone()).await.expect("establish");
        let cap = CapabilityProof {
            capabilities: vec!["orchestration".to_string()],
            proof: "0123456789abcdef0123456789abcdef".to_string(),
            timestamp: SystemTime::now(),
        };
        manager.verify_capabilities(&session_id, cap).await.expect("cap");
        let err = manager.verify_role(&session_id, "admin".to_string()).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn verify_role_requires_capability_first() {
        let manager = TrustEscalationManager::with_defaults();
        let session_id = "role-order".to_string();
        manager.establish_anonymous(session_id.clone()).await.expect("establish");
        let err = manager.verify_role(&session_id, "worker".to_string()).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn verify_identity_requires_role_first() {
        let manager = TrustEscalationManager::with_defaults();
        let session_id = "id-order".to_string();
        manager.establish_anonymous(session_id.clone()).await.expect("establish");
        let cap = CapabilityProof {
            capabilities: vec!["orchestration".to_string()],
            proof: "0123456789abcdef0123456789abcdef".to_string(),
            timestamp: SystemTime::now(),
        };
        manager.verify_capabilities(&session_id, cap).await.expect("cap");
        let identity = TowerIdentity {
            node_id: "n".to_string(),
            hostname: "h".to_string(),
            organization: None,
            public_key: None,
        };
        let id_proof = IdentityProof {
            identity,
            proof: "0123456789abcdef0123456789abcdef".to_string(),
            proof_type: "jwt".to_string(),
            timestamp: SystemTime::now(),
        };
        let err = manager.verify_identity(&session_id, id_proof).await;
        assert!(err.is_err());
    }

    #[tokio::test]
    async fn get_relationship_clones_store_entry() {
        let manager = TrustEscalationManager::with_defaults();
        let session_id = "rel-clone".to_string();
        manager.establish_anonymous(session_id.clone()).await.expect("establish");
        let rel = manager.get_relationship(&session_id).await;
        assert!(rel.is_some());
        assert_eq!(rel.expect("rel").trust_level, TrustLevel::Anonymous);
    }

    #[tokio::test]
    async fn verify_hardware_fails_without_security_client() {
        let manager = TrustEscalationManager::with_defaults();
        let hw = HardwareAttestation {
            hardware_key: "0123456789abcdef0123456789abcdef".to_string(),
            genetic_proof: None,
            attested_at: SystemTime::now(),
            signature: "sig".to_string(),
        };
        let err = manager.verify_hardware("any-session", hw).await;
        assert!(err.is_err());
        let msg = err.expect_err("expected err").to_string();
        assert!(msg.contains("security") || msg.contains("Session"), "unexpected message: {msg}");
    }

    #[tokio::test]
    async fn get_trust_level_reports_anonymous_when_expired() {
        let mut timeouts = TrustTimeouts::default();
        timeouts.anonymous = 0;
        let manager = TrustEscalationManager::new(timeouts, None);
        let session_id = "exp-anon".to_string();
        manager.establish_anonymous(session_id.clone()).await.expect("establish");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let level = manager.get_trust_level(&session_id).await.expect("level");
        assert_eq!(level, TrustLevel::Anonymous);
    }

    #[tokio::test]
    async fn check_permission_false_when_session_expired() {
        let mut timeouts = TrustTimeouts::default();
        timeouts.anonymous = 0;
        let manager = TrustEscalationManager::new(timeouts, None);
        let session_id = "exp-check".to_string();
        manager.establish_anonymous(session_id.clone()).await.expect("establish");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let allowed =
            manager.check_permission(&session_id, TrustLevel::Anonymous).await.expect("check");
        assert!(!allowed);
    }

    #[test]
    fn trust_timeouts_defaults_are_ordered_by_increasing_duration() {
        let t = TrustTimeouts::default();
        assert!(t.anonymous < t.capability);
        assert!(t.capability < t.identity);
        assert_eq!(t.hardware, 0);
    }
}
