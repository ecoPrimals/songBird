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
/// - Without an endpoint, callers should treat verification as unavailable (use capability discovery)
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
                "Security client created without endpoint (development-only local checks if used)"
            );
        }

        Self {
            endpoint,
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

        info!("✅ Anonymous trust established (Level 0): {}", session_id);

        self.trust_store.write().await.insert(session_id, relationship);
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
        info!("✅ Trust escalated to Identity-Verified (Level 3): {}", session_id);
        info!("   Identity: {}", identity_proof.identity.node_id);

        relationship.identity = Some(identity_proof.identity);
        relationship.last_verified_at = SystemTime::now();
        relationship.expires_at =
            SystemTime::now() + std::time::Duration::from_secs(self.trust_timeouts.identity);

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
        self.security_client.as_ref().ok_or_else(|| {
            anyhow!(
                "hardware attestation requires a configured security provider: set SONGBIRD_SECURITY_PROVIDER or SECURITY_ENDPOINT, or discover the `security` capability (see songbird-config `find_primals_with_capability`)"
            )
        })?;

        {
            let store = self.trust_store.read().await;
            let relationship =
                store.get(session_id).ok_or_else(|| anyhow!("Session not found: {session_id}"))?;
            if relationship.trust_level < TrustLevel::IdentityVerified {
                return Err(anyhow!(
                    "Cannot escalate to hardware-verified from {:?}",
                    relationship.trust_level
                ));
            }
        }

        tracing::debug!(
            hardware_key_len = hardware_proof.hardware_key.len(),
            "hardware attestation received (verification not yet wired to SecurityCapabilityClient)"
        );

        Err(anyhow!(
            "hardware attestation verification is not implemented yet (expected: BTSP / security provider verify_attestation via SecurityCapabilityClient); configure SONGBIRD_SECURITY_PROVIDER when the provider exposes this RPC"
        ))
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
#[path = "escalation_tests.rs"]
mod tests;
