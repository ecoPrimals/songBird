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
    /// TODO: Implement actual cryptographic verification
    /// For now, this is a placeholder that always returns true.
    #[must_use]
    pub fn verify(&self) -> bool {
        // Placeholder: Always verify successfully
        // In production, this should:
        // 1. Verify the cryptographic signature
        // 2. Check that capabilities match the proof
        // 3. Verify the timestamp is recent
        !self.capabilities.is_empty() && !self.proof.is_empty()
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
    /// TODO: Implement actual identity verification
    /// For now, this is a placeholder that always returns true.
    #[must_use]
    pub fn verify(&self) -> bool {
        // Placeholder: Always verify successfully
        // In production, this should:
        // 1. Verify JWT signature or certificate
        // 2. Check that identity matches the proof
        // 3. Verify the timestamp is recent
        // 4. Check against known identities
        !self.identity.node_id.is_empty() && !self.proof.is_empty()
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

