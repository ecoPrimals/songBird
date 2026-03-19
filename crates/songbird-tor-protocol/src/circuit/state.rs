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
                nickname: format!("relay{}", i),
                fingerprint: [i as u8; 20],
                address: IpAddr::from([127, 0, 0, 1]),
                or_port: 9001,
                dir_port: None,
                flags: crate::directory::RelayFlags::empty(),
                bandwidth: 1000000,
                ntor_key: None,
                version: None,
            };

            let hop =
                CircuitHop::new(relay, [i as u8; 32], [i as u8; 32], [i as u8; 16], [i as u8; 16]);

            circuit.add_hop(hop);
        }

        assert_eq!(circuit.hop_count(), 3);
        assert!(circuit.is_complete());
    }
}
