// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (c) 2024-2026 ecoPrimals

//! Circuit state management
//!
//! **Phase 2B**: Circuit building

use crate::directory::RelayInfo;
use std::time::Instant;

/// Circuit purpose
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitPurpose {
    /// General purpose circuit
    General,
    /// Hidden service directory queries
    HSDir,
    /// Hidden service rendezvous
    Rendezvous,
}

/// Circuit state
#[derive(Debug, Clone)]
pub struct Circuit {
    /// Circuit ID (4 bytes)
    pub id: u32,
    /// Circuit purpose
    pub purpose: CircuitPurpose,
    /// Hops in the circuit
    pub hops: Vec<CircuitHop>,
    /// Creation time
    pub created_at: Instant,
}

impl Circuit {
    /// Create new circuit
    #[must_use]
    pub fn new(id: u32, purpose: CircuitPurpose) -> Self {
        Self {
            id,
            purpose,
            hops: Vec::new(),
            created_at: Instant::now(),
        }
    }

    /// Add hop to circuit
    pub fn add_hop(&mut self, hop: CircuitHop) {
        self.hops.push(hop);
    }

    /// Get number of hops
    #[must_use]
    pub const fn hop_count(&self) -> usize {
        self.hops.len()
    }

    /// Check if circuit is complete (3 hops)
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.hops.len() >= 3
    }

    /// Get age in seconds
    #[must_use]
    pub fn age_secs(&self) -> u64 {
        self.created_at.elapsed().as_secs()
    }
}

/// Single hop in a circuit
#[derive(Debug, Clone)]
pub struct CircuitHop {
    /// Relay information
    pub relay: RelayInfo,
    /// Forward digest state (32 bytes)
    pub forward_digest: [u8; 32],
    /// Backward digest state (32 bytes)
    pub backward_digest: [u8; 32],
    /// Forward encryption key (AES-128, 16 bytes)
    pub forward_key: [u8; 16],
    /// Backward encryption key (AES-128, 16 bytes)
    pub backward_key: [u8; 16],
}

impl CircuitHop {
    /// Create new circuit hop from key material
    #[must_use]
    pub const fn new(
        relay: RelayInfo,
        forward_digest: [u8; 32],
        backward_digest: [u8; 32],
        forward_key: [u8; 16],
        backward_key: [u8; 16],
    ) -> Self {
        Self {
            relay,
            forward_digest,
            backward_digest,
            forward_key,
            backward_key,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use super::*;
    use std::net::IpAddr;

    #[test]
    fn test_circuit_creation() {
        let circuit = Circuit::new(1234, CircuitPurpose::General);
        assert_eq!(circuit.id, 1234);
        assert_eq!(circuit.purpose, CircuitPurpose::General);
        assert_eq!(circuit.hop_count(), 0);
        assert!(!circuit.is_complete());
    }

    #[test]
    fn test_circuit_add_hops() {
        let mut circuit = Circuit::new(1, CircuitPurpose::General);

        // Add 3 hops
        for i in 0..3 {
            let relay = RelayInfo {
                nickname: format!("relay{i}"),
                fingerprint: [u8::try_from(i).expect("i in 0..3 fits u8"); 20],
                address: IpAddr::from([127, 0, 0, 1]),
                or_port: 9001,
                dir_port: None,
                flags: crate::directory::RelayFlags::empty(),
                bandwidth: 1_000_000,
                ntor_key: None,
                version: None,
            };

            let i_u8 = u8::try_from(i).expect("i in 0..3 fits u8");
            let hop = CircuitHop::new(relay, [i_u8; 32], [i_u8; 32], [i_u8; 16], [i_u8; 16]);

            circuit.add_hop(hop);
        }

        assert_eq!(circuit.hop_count(), 3);
        assert!(circuit.is_complete());
    }

    #[test]
    fn circuit_purpose_variants_are_distinct() {
        assert_ne!(CircuitPurpose::General, CircuitPurpose::HSDir);
        assert_ne!(CircuitPurpose::HSDir, CircuitPurpose::Rendezvous);
        assert_eq!(CircuitPurpose::Rendezvous, CircuitPurpose::Rendezvous);
    }

    #[test]
    fn circuit_two_hops_is_not_complete() {
        let mut circuit = Circuit::new(7, CircuitPurpose::HSDir);
        for i in 0..2 {
            let relay = RelayInfo {
                nickname: format!("r{i}"),
                fingerprint: [u8::try_from(i).expect("i in 0..2"); 20],
                address: std::net::IpAddr::from([127, 0, 0, 1]),
                or_port: 443,
                dir_port: None,
                flags: crate::directory::RelayFlags::empty(),
                bandwidth: 1,
                ntor_key: None,
                version: None,
            };
            let hop = CircuitHop::new(relay, [0u8; 32], [0u8; 32], [0u8; 16], [0u8; 16]);
            circuit.add_hop(hop);
        }
        assert_eq!(circuit.hop_count(), 2);
        assert!(!circuit.is_complete());
    }

    #[test]
    fn circuit_age_secs_is_well_defined() {
        let c = Circuit::new(1, CircuitPurpose::Rendezvous);
        assert_eq!(c.age_secs(), 0);
    }

    #[test]
    fn circuit_hop_keys_roundtrip_clone() {
        let relay = RelayInfo {
            nickname: "g".to_string(),
            fingerprint: [7u8; 20],
            address: std::net::IpAddr::from([10, 0, 0, 1]),
            or_port: 9001,
            dir_port: Some(9030),
            flags: crate::directory::RelayFlags::empty(),
            bandwidth: 100,
            ntor_key: Some([9u8; 32]),
            version: Some("0.4.8".to_string()),
        };
        let hop = CircuitHop::new(relay.clone(), [1u8; 32], [2u8; 32], [3u8; 16], [4u8; 16]);
        let cloned = hop.clone();
        assert_eq!(cloned.relay.nickname, relay.nickname);
        assert_eq!(cloned.forward_key, [3u8; 16]);
    }
}
