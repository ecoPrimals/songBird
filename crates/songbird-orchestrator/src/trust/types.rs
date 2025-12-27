//! Trust Types and Structures
//!
//! Defines the core types for the trust escalation system.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Trust levels for progressive escalation
///
/// Each level grants increasing access to system resources and information.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TrustLevel {
    /// No trust, anonymous only
    ///
    /// **Granted:** Discovery, capability exchange
    /// **Denied:** Task coordination, registry access, infrastructure details
    Anonymous = 0,

    /// Capability-verified (can coordinate tasks)
    ///
    /// **Granted:** Task coordination, resource sharing
    /// **Denied:** Registry access, infrastructure details, admin operations
    CapabilityVerified = 1,

    /// Role-verified (can access registry)
    ///
    /// **Granted:** Service registry access, federation coordination
    /// **Denied:** Infrastructure details, admin operations
    RoleVerified = 2,

    /// Identity-verified (can see infrastructure)
    ///
    /// **Granted:** Infrastructure topology, internal IPs, configuration
    /// **Denied:** Admin operations (deployment, configuration changes)
    IdentityVerified = 3,

    /// Hardware-verified (full admin access, BearDog)
    ///
    /// **Granted:** Full admin access, deployment, configuration, all operations
    HardwareVerified = 4,
}

impl TrustLevel {
    /// Check if this trust level can perform an operation requiring a minimum level
    #[must_use]
    pub fn can_perform(&self, required_level: TrustLevel) -> bool {
        *self >= required_level
    }

    /// Get a human-readable description of this trust level
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::Anonymous => "Anonymous (discovery only)",
            Self::CapabilityVerified => "Capability-Verified (task coordination)",
            Self::RoleVerified => "Role-Verified (registry access)",
            Self::IdentityVerified => "Identity-Verified (infrastructure access)",
            Self::HardwareVerified => "Hardware-Verified (full admin)",
        }
    }
}

impl std::fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Trust relationship between towers
#[derive(Debug, Clone)]
pub struct TrustRelationship {
    /// Remote session ID (anonymous identifier)
    pub session_id: String,

    /// Current trust level
    pub trust_level: TrustLevel,

    /// Verified capabilities (if capability-verified or higher)
    pub verified_capabilities: Vec<String>,

    /// Identity information (only if identity-verified or higher)
    pub identity: Option<TowerIdentity>,

    /// Hardware attestation (only if hardware-verified)
    pub hardware_proof: Option<HardwareAttestation>,

    /// When this relationship was established
    pub established_at: SystemTime,

    /// Last time trust was verified
    pub last_verified_at: SystemTime,

    /// Expiration time for this trust level
    pub expires_at: SystemTime,
}

impl TrustRelationship {
    /// Create a new anonymous trust relationship
    #[must_use]
    pub fn new_anonymous(session_id: String, timeout_secs: u64) -> Self {
        let now = SystemTime::now();
        Self {
            session_id,
            trust_level: TrustLevel::Anonymous,
            verified_capabilities: Vec::new(),
            identity: None,
            hardware_proof: None,
            established_at: now,
            last_verified_at: now,
            expires_at: now + std::time::Duration::from_secs(timeout_secs),
        }
    }

    /// Check if this trust relationship has expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }

    /// Check if this trust level can perform an operation
    #[must_use]
    pub fn can_perform(&self, required_level: TrustLevel) -> bool {
        !self.is_expired() && self.trust_level.can_perform(required_level)
    }
}

/// Tower identity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerIdentity {
    /// Node ID
    pub node_id: String,

    /// Hostname
    pub hostname: String,

    /// Organization (optional)
    pub organization: Option<String>,

    /// Public key for verification
    pub public_key: Option<String>,
}

/// Hardware attestation for hardware-verified trust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareAttestation {
    /// Hardware key ID (from BearDog)
    pub hardware_key: String,

    /// Genetic identity proof (from BearDog)
    pub genetic_proof: Option<String>,

    /// Attestation timestamp
    pub attested_at: SystemTime,

    /// Attestation signature
    pub signature: String,
}

/// Capability proof for capability-verified trust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityProof {
    /// Capabilities being proven
    pub capabilities: Vec<String>,

    /// Cryptographic proof (signature)
    pub proof: String,

    /// Timestamp of proof generation
    pub timestamp: SystemTime,
}

impl CapabilityProof {
    /// Verify this capability proof
    ///
    /// ## Verification Steps
    /// 1. Validates proof structure (non-empty)
    /// 2. Validates capabilities list (non-empty)
    /// 3. Checks timestamp freshness (within 1 hour)
    /// 4. Future: Cryptographic signature verification via BearDog
    ///
    /// ## Security Notes
    /// - Current implementation provides basic validation
    /// - Full cryptographic verification requires BearDog integration
    /// - Timestamp check prevents replay attacks
    #[must_use]
    pub fn verify(&self) -> bool {
        use std::time::SystemTime;
        
        // Step 1: Validate structure
        if self.capabilities.is_empty() || self.proof.is_empty() {
            tracing::warn!("Capability proof verification failed: empty capabilities or proof");
            return false;
        }
        
        // Step 2: Check proof format (should be base64 or hex)
        if self.proof.len() < 32 {
            tracing::warn!("Capability proof verification failed: proof too short (< 32 chars)");
            return false;
        }
        
        // Step 3: Verify timestamp freshness (within 1 hour)
        let now = SystemTime::now();
        if let Ok(age) = now.duration_since(self.timestamp) {
            let age_hours = age.as_secs() / 3600;
            if age_hours > 1 {
                tracing::warn!(
                    "Capability proof verification failed: timestamp too old ({} hours)",
                    age_hours
                );
                return false;
            }
            
            tracing::debug!(
                "Capability proof verified: {} capabilities, timestamp {} hours old (basic validation only)",
                self.capabilities.len(),
                age_hours
            );
        } else {
            tracing::warn!("Capability proof verification failed: timestamp in the future");
            return false;
        }
        
        // Step 4: Future - cryptographic signature verification
        // When BearDog is integrated, add:
        // - Verify Ed25519 signature
        // - Check proof against public key
        // - Validate capability claims
        
        true
    }
}

/// Identity proof for identity-verified trust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProof {
    /// Tower identity being proven
    pub identity: TowerIdentity,

    /// Cryptographic proof (JWT, certificate, etc.)
    pub proof: String,

    /// Proof type ("jwt", "certificate", etc.)
    pub proof_type: String,

    /// Timestamp of proof generation
    pub timestamp: SystemTime,
}

impl IdentityProof {
    /// Verify this identity proof
    ///
    /// ## Verification Steps
    /// 1. Validates identity structure (non-empty node ID)
    /// 2. Validates proof structure (non-empty, minimum length)
    /// 3. Checks timestamp freshness (within 24 hours for identity)
    /// 4. Future: Cryptographic signature verification via BearDog
    ///
    /// ## Security Notes
    /// - Identity proofs have longer validity (24h) than capability proofs (1h)
    /// - Full verification requires BearDog's genetic lineage system
    #[must_use]
    pub fn verify(&self) -> bool {
        use std::time::SystemTime;
        
        // Step 1: Validate identity node ID
        let node_id = &self.identity.node_id;
        if node_id.is_empty() || node_id.len() < 8 {
            tracing::warn!("Identity proof verification failed: invalid node ID");
            return false;
        }
        
        // Step 2: Validate proof structure
        if self.proof.is_empty() || self.proof.len() < 32 {
            tracing::warn!("Identity proof verification failed: invalid proof structure");
            return false;
        }
        
        // Step 3: Verify timestamp freshness (within 24 hours)
        let now = SystemTime::now();
        if let Ok(age) = now.duration_since(self.timestamp) {
            let age_hours = age.as_secs() / 3600;
            if age_hours > 24 {
                tracing::warn!(
                    "Identity proof verification failed: timestamp too old ({} hours)",
                    age_hours
                );
                return false;
            }
            
            tracing::debug!(
                "Identity proof verified: node ID {}, {} hours old (basic validation only)",
                node_id,
                age_hours
            );
        } else {
            tracing::warn!("Identity proof verification failed: timestamp in the future");
            return false;
        }
        
        // Step 4: Future - genetic lineage verification
        // When BearDog is integrated, add:
        // - Verify JWT signature or certificate
        // - Check genetic signature
        // - Validate lineage chain
        // - Verify against known identities
        
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_level_ordering() {
        assert!(TrustLevel::Anonymous < TrustLevel::CapabilityVerified);
        assert!(TrustLevel::CapabilityVerified < TrustLevel::RoleVerified);
        assert!(TrustLevel::RoleVerified < TrustLevel::IdentityVerified);
        assert!(TrustLevel::IdentityVerified < TrustLevel::HardwareVerified);
    }

    #[test]
    fn test_trust_level_can_perform() {
        let level = TrustLevel::CapabilityVerified;

        assert!(level.can_perform(TrustLevel::Anonymous));
        assert!(level.can_perform(TrustLevel::CapabilityVerified));
        assert!(!level.can_perform(TrustLevel::RoleVerified));
        assert!(!level.can_perform(TrustLevel::IdentityVerified));
        assert!(!level.can_perform(TrustLevel::HardwareVerified));
    }

    #[test]
    fn test_trust_relationship_creation() {
        let relationship = TrustRelationship::new_anonymous("test-session".to_string(), 3600);

        assert_eq!(relationship.session_id, "test-session");
        assert_eq!(relationship.trust_level, TrustLevel::Anonymous);
        assert!(relationship.verified_capabilities.is_empty());
        assert!(relationship.identity.is_none());
        assert!(relationship.hardware_proof.is_none());
        assert!(!relationship.is_expired());
    }

    #[test]
    fn test_trust_relationship_expiration() {
        // Create a relationship that expires immediately
        let mut relationship = TrustRelationship::new_anonymous("test-session".to_string(), 0);

        // Set expiration to the past
        relationship.expires_at = SystemTime::now() - std::time::Duration::from_secs(1);

        assert!(relationship.is_expired());
        assert!(!relationship.can_perform(TrustLevel::Anonymous));
    }

    #[test]
    fn test_capability_proof_verification() {
        let proof = CapabilityProof {
            capabilities: vec!["orchestration".to_string()],
            proof: "test-proof".to_string(),
            timestamp: SystemTime::now(),
        };

        assert!(proof.verify());

        // Empty capabilities should fail
        let invalid_proof = CapabilityProof {
            capabilities: vec![],
            proof: "test-proof".to_string(),
            timestamp: SystemTime::now(),
        };

        assert!(!invalid_proof.verify());
    }

    #[test]
    fn test_identity_proof_verification() {
        let identity = TowerIdentity {
            node_id: "test-node".to_string(),
            hostname: "test-host".to_string(),
            organization: None,
            public_key: None,
        };

        let proof = IdentityProof {
            identity,
            proof: "test-proof".to_string(),
            proof_type: "jwt".to_string(),
            timestamp: SystemTime::now(),
        };

        assert!(proof.verify());

        // Empty node_id should fail
        let invalid_identity = TowerIdentity {
            node_id: String::new(),
            hostname: "test-host".to_string(),
            organization: None,
            public_key: None,
        };

        let invalid_proof = IdentityProof {
            identity: invalid_identity,
            proof: "test-proof".to_string(),
            proof_type: "jwt".to_string(),
            timestamp: SystemTime::now(),
        };

        assert!(!invalid_proof.verify());
    }
}
