// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Trust Escalation System
//!
//! Implements progressive trust escalation from anonymous to hardware-verified.
//!
//! ## Trust Levels
//!
//! ```text
//! Level 0: Anonymous          → Can discover, no data shared
//! Level 1: Capability-Verified → Can coordinate tasks
//! Level 2: Role-Verified       → Can access service registry
//! Level 3: Identity-Verified   → Can see infrastructure details
//! Level 4: Hardware-Verified   → Full admin access (security provider)
//! ```
//!
//! ## Progressive Escalation
//!
//! ```text
//! Tower A                          Tower B
//!    |                                |
//!    |  Anonymous Discovery           |
//!    |<==============================>|
//!    |  Trust Level: 0 (Anonymous)    |
//!    |                                |
//!    |  Verify Capabilities           |
//!    |<==============================>|
//!    |  Trust Level: 1 (Capability)   |
//!    |                                |
//!    |  Coordinate Tasks              |
//!    |<==============================>|
//!    |                                |
//!    |  Request Admin Access          |
//!    |------------------------------->|
//!    |  Provide Hardware Key          |
//!    |------------------------------->|
//!    |  Verify with security provider           |
//!    |<==============================>|
//!    |  Trust Level: 4 (Hardware)     |
//!    |  Grant Full Access             |
//!    |<==============================>|
//! ```

pub mod escalation;
pub mod lineage_auth; // Genetic lineage auto-accept logic
pub mod peer_trust; // USB seed integration - peer trust evaluation
pub mod types;
pub mod universal_trust_api; // Universal, provider-agnostic trust API

pub use escalation::{TrustEscalationManager, TrustTimeouts};
pub use lineage_auth::{
    LineageAuthenticator, LineageStatus, PeerAcceptanceDecision, PeerInfo, RejectionSeverity,
    UserRecommendation,
};
pub use peer_trust::{DiscoveredPeer, PeerTrustDecision, evaluate_peer_trust}; // NEW: USB seed integration
pub use types::{
    CapabilityProof, HardwareAttestation, IdentityProof, TowerIdentity, TrustLevel,
    TrustRelationship,
};
pub use universal_trust_api::{
    DiscoveryContext as UniversalDiscoveryContext, EvaluatorInfo,
    IdentityAttestation as UniversalIdentityAttestation, TrustDecision as UniversalTrustDecision,
    UniversalTrustRequest, UniversalTrustResponse,
};

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test assertions")]
    #![allow(clippy::expect_used, reason = "test assertions")]

    use crate::trust::TrustLevel;

    #[test]
    fn trust_level_ordering_and_can_perform() {
        assert!(TrustLevel::HardwareVerified > TrustLevel::Anonymous);
        assert!(TrustLevel::HardwareVerified.can_perform(TrustLevel::RoleVerified));
        assert!(!TrustLevel::CapabilityVerified.can_perform(TrustLevel::IdentityVerified));
    }

    #[test]
    fn trust_level_descriptions_non_empty() {
        for level in [
            TrustLevel::Anonymous,
            TrustLevel::CapabilityVerified,
            TrustLevel::RoleVerified,
            TrustLevel::IdentityVerified,
            TrustLevel::HardwareVerified,
        ] {
            assert!(!level.description().is_empty());
            let s = level.to_string();
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn reexports_resolve_universal_types() {
        let _ = std::mem::size_of::<crate::trust::UniversalTrustRequest>();
        let _ = std::mem::size_of::<crate::trust::UniversalTrustResponse>();
    }
}
